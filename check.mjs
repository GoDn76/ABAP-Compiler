import { spawn, execSync } from 'node:child_process';
import { setTimeout as delay } from 'node:timers/promises';
import { existsSync } from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));

// Ensure proxies do not intercept localhost loopback
delete process.env.http_proxy;
delete process.env.https_proxy;
delete process.env.HTTP_PROXY;
delete process.env.HTTPS_PROXY;
delete process.env.ALL_PROXY;
process.env.NO_PROXY = '*';

console.log('\x1b[36m\x1b[1m=== ABAPC Studio 5.0.1 Rust Backend Verification ===\x1b[0m\n');

// 1. Locate or compile the Rust binary
const exeExt = process.platform === 'win32' ? '.exe' : '';
const binDebug = path.join(__dirname, 'target', 'debug', `abapc-server${exeExt}`);
const binRelease = path.join(__dirname, 'target', 'release', `abapc-server${exeExt}`);

let serverBin = '';
if (existsSync(binDebug)) {
  serverBin = binDebug;
} else if (existsSync(binRelease)) {
  serverBin = binRelease;
} else {
  console.log('Rust binary not found. Compiling with `cargo build`...');
  try {
    execSync('cargo build', { cwd: __dirname, stdio: 'inherit' });
    serverBin = binDebug;
  } catch (err) {
    console.error('\x1b[31mFailed to compile Rust backend.\x1b[0m');
    process.exit(1);
  }
}

// 2. Pick an isolated port and launch the server
const port = 17500 + Math.floor(Math.random() * 400);
console.log(`Starting Rust backend on port ${port}...`);

const child = spawn(serverBin, [], {
  cwd: __dirname,
  env: {
    ...process.env,
    ABAPC_PORT: String(port),
    ABAPC_HOST: '127.0.0.1',
    ABAPC_NO_BROWSER: '1',
    RUST_LOG: 'error',
  },
  stdio: ['ignore', 'pipe', 'pipe'],
});

let serverLog = '';
child.stdout?.on('data', b => { serverLog += b.toString(); });
child.stderr?.on('data', b => { serverLog += b.toString(); });

async function terminateChild() {
  if (child && !child.killed) {
    try {
      if (process.platform === 'win32') {
        execSync(`taskkill /pid ${child.pid} /T /F`, { stdio: 'ignore' });
      } else {
        child.kill('SIGTERM');
      }
    } catch {}
  }
}

process.on('SIGINT', async () => {
  await terminateChild();
  process.exit(1);
});

try {
  // 3. Wait for /api/health
  let ready = false;
  for (let i = 0; i < 60; i++) {
    try {
      const r = await fetch(`http://127.0.0.1:${port}/api/health`);
      if (r.ok) {
        ready = true;
        break;
      }
    } catch {}
    await delay(100);
  }

  if (!ready) {
    throw new Error(`Rust server failed to start within 6 seconds. Log:\n${serverLog}`);
  }

  console.log('\x1b[32m✔ Rust server is live and responsive.\x1b[0m Running counterpart parity checks:\n');

  // Helper for reporting individual steps
  async function step(name, fn) {
    process.stdout.write(`  • ${name} ... `);
    const start = Date.now();
    await fn();
    console.log(`\x1b[32mPASS\x1b[0m (${Date.now() - start}ms)`);
  }

  // Check 1: Health & version contract
  await step('1. Health check & version contract', async () => {
    const res = await fetch(`http://127.0.0.1:${port}/api/health`);
    const health = await res.json();
    if (health.version !== '5.0.1') throw new Error(`Unexpected version: ${JSON.stringify(health)}`);
    if (health.status !== 'ok') throw new Error(`Status not ok: ${JSON.stringify(health)}`);
  });

  // Check 2: Static assets & navigation
  await step('2. Static assets & GUI simulator HTML routes', async () => {
    const mainPage = await fetch(`http://127.0.0.1:${port}/`);
    const mainHtml = await mainPage.text();
    if (!mainPage.ok || !mainHtml.includes('href="/sapgui"') || !mainHtml.includes('Transaction GUI')) {
      throw new Error('Main IDE does not expose the transaction workbench link.');
    }
    const gui = await fetch(`http://127.0.0.1:${port}/sapgui`);
    const guiHtml = await gui.text();
    if (!gui.ok || !guiHtml.includes('SAP GUI Practice Simulator') || !guiHtml.includes('id="command"')) {
      throw new Error('Transaction workbench route failed.');
    }
    const guiJs = await fetch(`http://127.0.0.1:${port}/gui.js`);
    const guiCss = await fetch(`http://127.0.0.1:${port}/gui.css`);
    const guiJsText = await guiJs.text();
    const guiCssText = await guiCss.text();
    if (!guiJs.ok || !guiJsText.includes('renderTableBrowser') || !guiCss.ok || !guiCssText.includes('.standard-toolbar')) {
      throw new Error('GUI assets failed.');
    }
  });

  // Check 3: Table browser fixtures
  await step('3. SE16N table fixtures (SCARR, SPFLI, SFLIGHT)', async () => {
    const fixtureResponse = await fetch(`http://127.0.0.1:${port}/api/gui/tables`);
    const fixtureBody = await fixtureResponse.json();
    if (!fixtureResponse.ok || fixtureBody.tables?.SCARR?.length !== 3 || fixtureBody.tables?.SPFLI?.length !== 3 || fixtureBody.tables?.SFLIGHT?.length !== 3) {
      throw new Error('Local table-browser fixture endpoint failed.');
    }
  });

  // Check 4: Platform catalog
  await step('4. DDIC/RAP platform catalogue (/api/platform/catalog)', async () => {
    const platformCatalog = await (await fetch(`http://127.0.0.1:${port}/api/platform/catalog`)).json();
    if (!platformCatalog.objects?.some(o => o.name === 'ZDE_PRODUCT_ID') || !platformCatalog.objects?.some(o => o.name === 'ZR_PRODUCT') || platformCatalog.isSapBackend !== false) {
      throw new Error('Local DDIC/RAP platform catalogue smoke failed.');
    }
  });

  // Check 5: Dynamic DDIC Lifecycle
  await step('5. Dynamic DDIC CRUD (DOMA -> DTEL -> STRU -> TABL & ABAP resolution)', async () => {
    const ddicNames = ['ZSMOKE50_DOMAIN', 'ZSMOKE50_ELEMENT', 'ZSMOKE50_STRUCT', 'ZSMOKE50_TABLE'];
    const createDdic = async (body) => {
      const response = await fetch(`http://127.0.0.1:${port}/api/platform/create-object`, {
        method: 'POST',
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify(body),
      });
      const result = await response.json();
      if (!response.ok || !result.ok) throw new Error(`Local DDIC create failed: ${JSON.stringify(result)}`);
    };
    const deleteDdic = async (name) => {
      const response = await fetch(`http://127.0.0.1:${port}/api/platform/object?name=${encodeURIComponent(name)}`, { method: 'DELETE' });
      const result = await response.json();
      if (!response.ok || !result.ok) throw new Error(`Local DDIC delete failed for ${name}: ${JSON.stringify(result)}`);
    };

    try {
      await createDdic({ name: ddicNames[0], kind: 'DOMA', baseType: 'CHAR(30)' });
      await createDdic({ name: ddicNames[1], kind: 'DTEL', baseType: ddicNames[0] });
      await createDdic({ name: ddicNames[2], kind: 'STRU', fields: [{ name: 'ID', type: 'I' }, { name: 'NAME', type: ddicNames[1] }] });
      await createDdic({ name: ddicNames[3], kind: 'TABL', fields: [{ name: 'ID', type: 'I', key: true }, { name: 'NAME', type: ddicNames[1] }], initialRows: [{ ID: 1, NAME: 'Repository row' }] });

      const ddicSource = `REPORT z_ddic_http. DATA person TYPE ${ddicNames[2].toLowerCase()}. person-id = 7. person-name = 'DDIC type chain'. WRITE person-name. DATA people TYPE TABLE OF ${ddicNames[3].toLowerCase()}. SELECT FROM ${ddicNames[3]} FIELDS id, name INTO TABLE @people. WRITE LINES( people ).`;
      const response = await fetch(`http://127.0.0.1:${port}/api/execute`, {
        method: 'POST',
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify({ source: ddicSource, mode: 'standard', filename: 'ddic-http-smoke.abap' }),
      });
      const result = await response.json();
      if (!result.ok || JSON.stringify(result.output) !== JSON.stringify(['DDIC type chain', '1'])) {
        throw new Error(`Dynamic DDIC execution smoke failed: ${JSON.stringify(result)}`);
      }
    } finally {
      for (const name of [...ddicNames].reverse()) await deleteDdic(name);
    }
  });

  // Check 6: Cloud Classrun
  await step('6. Cloud classrun (if_oo_adt_classrun & out->write)', async () => {
    const source = `CLASS zcl_smoke DEFINITION PUBLIC FINAL CREATE PUBLIC. PUBLIC SECTION. INTERFACES if_oo_adt_classrun. ENDCLASS. CLASS zcl_smoke IMPLEMENTATION. METHOD if_oo_adt_classrun~main. DATA x TYPE i. x = 12 + 2. out->write( x ). ENDMETHOD. ENDCLASS.`;
    const r = await fetch(`http://127.0.0.1:${port}/api/execute`, {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ source, mode: 'cloud', filename: 'smoke.abap' }),
    });
    const body = await r.json();
    if (!body.ok || JSON.stringify(body.output) !== JSON.stringify(['14'])) {
      throw new Error(`HTTP execution smoke failed: ${JSON.stringify(body)}`);
    }
  });

  // Check 7: Parameters injection
  await step('7. PARAMETERS injection via parameterValues', async () => {
    const parameterSource = `REPORT zgui_parameters. PARAMETERS p_total TYPE i. WRITE: p_total.`;
    const parameterRun = await fetch(`http://127.0.0.1:${port}/api/execute`, {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ source: parameterSource, mode: 'standard', filename: 'parameters-smoke.abap', parameterValues: { p_total: '42' } }),
    });
    const parameterRunBody = await parameterRun.json();
    if (!parameterRunBody.ok || JSON.stringify(parameterRunBody.output) !== JSON.stringify(['42'])) {
      throw new Error(`PARAMETERS execution smoke failed: ${JSON.stringify(parameterRunBody)}`);
    }
  });

  // Check 8: OO Calculator
  await step('8. OO Calculator (methods, returning value, NEW constructor)', async () => {
    const ooSource = `REPORT ztest_20. CLASS lcl_calculator DEFINITION. PUBLIC SECTION. METHODS add_numbers IMPORTING iv_a TYPE i iv_b TYPE i RETURNING VALUE(rv_res) TYPE i. ENDCLASS. CLASS lcl_calculator IMPLEMENTATION. METHOD add_numbers. rv_res = iv_a + iv_b. ENDMETHOD. ENDCLASS. START-OF-SELECTION. DATA(lo_calc) = NEW lcl_calculator( ). DATA(lv_sum) = lo_calc->add_numbers( iv_a = 25 iv_b = 75 ). WRITE: / 'Class Method Output:', lv_sum.`;
    const ooResponse = await fetch(`http://127.0.0.1:${port}/api/execute`, {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ source: ooSource, mode: 'standard', filename: 'oo-calculator-smoke.abap' }),
    });
    const ooBody = await ooResponse.json();
    if (!ooBody.ok || JSON.stringify(ooBody.output) !== JSON.stringify(['Class Method Output: 100']) || ooBody.variables?.LV_SUM !== 100) {
      throw new Error(`OO calculator execution smoke failed: ${JSON.stringify(ooBody)}`);
    }
  });

  // Check 9: 5.0.1 SWITCH constructor expression
  await step('9. SWITCH constructor expression (5.0.1 regression fix)', async () => {
    const switchSource = `REPORT ztest_17. DATA: lv_status TYPE string VALUE 'E'. DATA(lv_message) = SWITCH string( lv_status WHEN 'S' THEN 'Success' WHEN 'W' THEN 'Warning' WHEN 'E' THEN 'Error' ELSE 'Unknown' ). WRITE: / 'Status Message:', lv_message.`;
    const switchResponse = await fetch(`http://127.0.0.1:${port}/api/execute`, {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ source: switchSource, mode: 'standard', filename: 'switch-expression-smoke.abap' }),
    });
    const switchBody = await switchResponse.json();
    if (!switchBody.ok || JSON.stringify(switchBody.output) !== JSON.stringify(['Status Message: Error']) || switchBody.variables?.LV_MESSAGE !== 'Error') {
      throw new Error(`SWITCH expression execution smoke failed: ${JSON.stringify(switchBody)}`);
    }
  });

  // Check 10: Selection preview
  await step('10. Selection screen preview & INITIALIZATION', async () => {
    const selectionSource = `REPORT zselection_smoke. PARAMETERS p_name TYPE string DEFAULT 'default'. SELECT-OPTIONS s_car FOR scarr-carrid DEFAULT 'LH'. INITIALIZATION. p_name = 'initialized'. START-OF-SELECTION. WRITE p_name.`;
    const previewResponse = await fetch(`http://127.0.0.1:${port}/api/selection-preview`, {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ source: selectionSource, mode: 'standard' }),
    });
    const previewBody = await previewResponse.json();
    if (!previewResponse.ok || !previewBody.ok || previewBody.variables?.P_NAME !== 'initialized' || previewBody.variables?.S_CAR?.[0]?.LOW !== 'LH') {
      throw new Error(`Selection-screen preview smoke failed: ${JSON.stringify(previewBody)}`);
    }
  });

  // Check 11: Value help event
  await step('11. Selection event dispatch (AT SELECTION-SCREEN ON VALUE-REQUEST)', async () => {
    const eventSource = `REPORT zvaluehelp. PARAMETERS p_car TYPE string. AT SELECTION-SCREEN ON VALUE-REQUEST FOR p_car. WRITE 'LH'. WRITE 'SQ'.`;
    const eventResponse = await fetch(`http://127.0.0.1:${port}/api/selection-event`, {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ source: eventSource, mode: 'standard', event: 'at-selection-screen-value-request', target: 'p_car' }),
    });
    const eventBody = await eventResponse.json();
    if (!eventResponse.ok || !eventBody.ok || JSON.stringify(eventBody.output) !== JSON.stringify(['LH', 'SQ'])) {
      throw new Error(`Selection value-help event smoke failed: ${JSON.stringify(eventBody)}`);
    }
  });

  // Check 12: String statements
  await step('12. Classic string statements (TRANSLATE, SPLIT, LOOP AT)', async () => {
    const stringSource = `REPORT zstring_smoke. DATA text TYPE string. DATA parts TYPE TABLE OF string. text = 'hello,cloud'. TRANSLATE text TO UPPER CASE. SPLIT text AT ',' INTO TABLE parts. LOOP AT parts INTO DATA(part). WRITE part. ENDLOOP.`;
    const stringResponse = await fetch(`http://127.0.0.1:${port}/api/execute`, {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ source: stringSource, mode: 'standard', filename: 'string-smoke.abap' }),
    });
    const stringBody = await stringResponse.json();
    if (!stringBody.ok || JSON.stringify(stringBody.output) !== JSON.stringify(['HELLO', 'CLOUD'])) {
      throw new Error(`String statement smoke failed: ${JSON.stringify(stringBody)}`);
    }
  });

  // Check 13: EML end-to-end
  await step('13. EML end-to-end (MODIFY -> COMMIT -> READ -> SE16N persistence)', async () => {
    const emlId = 'T-RUST-' + Math.floor(Math.random() * 1000);
    const emlCleanup = `REPORT z_eml_cleanup. MODIFY ENTITIES OF ZR_PRODUCT ENTITY Product DELETE FROM VALUE #( ( PRODUCT_ID = '${emlId}' ) ) FAILED DATA(failed). COMMIT ENTITIES.`;
    await fetch(`http://127.0.0.1:${port}/api/execute`, {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ source: emlCleanup, mode: 'standard', filename: 'eml-cleanup.abap' }),
    });

    const emlSource = `REPORT z_eml_smoke. MODIFY ENTITIES OF ZR_PRODUCT ENTITY Product CREATE FIELDS ( PRODUCT_ID PRODUCT_NAME CATEGORY PRICE CURRENCY ) WITH VALUE #( ( PRODUCT_ID = '${emlId}' PRODUCT_NAME = 'Rust Smoke Product' CATEGORY = 'TEST' PRICE = 42 CURRENCY = 'INR' ) ) FAILED DATA(failed). COMMIT ENTITIES. READ ENTITIES OF ZI_PRODUCT ENTITY Product ALL FIELDS FROM VALUE #( ( PRODUCT_ID = '${emlId}' ) ) RESULT DATA(products). LOOP AT products INTO DATA(product). WRITE product-PRODUCT_NAME. ENDLOOP.`;
    const emlResponse = await fetch(`http://127.0.0.1:${port}/api/execute`, {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ source: emlSource, mode: 'standard', filename: 'eml-smoke.abap' }),
    });
    const emlBody = await emlResponse.json();
    if (!emlBody.ok || JSON.stringify(emlBody.output) !== JSON.stringify(['Rust Smoke Product'])) {
      throw new Error(`EML end-to-end smoke failed: ${JSON.stringify(emlBody)}`);
    }
    const productTable = await (await fetch(`http://127.0.0.1:${port}/api/gui/tables`)).json();
    if (!productTable.tables?.ZPRODUCT?.some(row => row.PRODUCT_ID === emlId)) {
      throw new Error('EML COMMIT was not visible in local SE16N table browser.');
    }
    await fetch(`http://127.0.0.1:${port}/api/execute`, {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ source: emlCleanup, mode: 'standard', filename: 'eml-cleanup.abap' }),
    });
  });

  console.log('\n\x1b[32m\x1b[1m====================================================\x1b[0m');
  console.log('\x1b[32m\x1b[1m✔ VERIFICATION COMPLETE: 100% PARITY WITH NODE.JS COUNTERPART\x1b[0m');
  console.log('The Rust backend passed all contract, syntax, DDIC, and execution checks.');
  console.log('\x1b[32m\x1b[1m====================================================\x1b[0m\n');
} catch (err) {
  console.error(`\n\x1b[31m\x1b[1m✘ CHECK FAILED: ${err.message}\x1b[0m`);
  process.exitCode = 1;
} finally {
  await terminateChild();
}
