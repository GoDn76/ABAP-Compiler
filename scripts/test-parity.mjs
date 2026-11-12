#!/usr/bin/env node
/**
 * Parity Test Suite for ABAPC Studio 5.0.1
 * Verifies that the Rust backend behaves identically to the TypeScript/Node.js backend.
 *
 * Usage:
 *   node scripts/test-parity.mjs [--port 7345] [--url http://127.0.0.1:7345]
 */

import process from 'node:process';

const args = process.argv.slice(2);
let targetUrl = 'http://127.0.0.1:7345';

for (let i = 0; i < args.length; i++) {
  if (args[i] === '--port' && args[i + 1]) {
    targetUrl = `http://127.0.0.1:${args[i + 1]}`;
    i++;
  } else if (args[i] === '--url' && args[i + 1]) {
    targetUrl = args[i + 1].replace(/\/$/, '');
    i++;
  }
}

const GREEN = '\x1b[32m';
const RED = '\x1b[31m';
const YELLOW = '\x1b[33m';
const CYAN = '\x1b[36m';
const BOLD = '\x1b[1m';
const RESET = '\x1b[0m';

console.log(`${BOLD}${CYAN}=== ABAPC Studio 5.0.1 Backend Parity Verification ===${RESET}`);
console.log(`Target Backend: ${BOLD}${targetUrl}${RESET}\n`);

let passed = 0;
let failed = 0;

async function check(name, testFn) {
  process.stdout.write(`  [TEST] ${name} ... `);
  const start = Date.now();
  try {
    await testFn();
    const ms = Date.now() - start;
    console.log(`${GREEN}PASS${RESET} (${ms}ms)`);
    passed++;
  } catch (err) {
    const ms = Date.now() - start;
    console.log(`${RED}FAIL${RESET} (${ms}ms)`);
    console.log(`         ${RED}Error: ${err.message}${RESET}`);
    failed++;
  }
}

async function fetchJson(path, options = {}) {
  const url = `${targetUrl}${path}`;
  const res = await fetch(url, options);
  const text = await res.text();
  let json;
  try {
    json = JSON.parse(text);
  } catch {
    throw new Error(`Invalid JSON from ${path} (status ${res.status}): ${text.slice(0, 150)}`);
  }
  return { res, json, status: res.status };
}

async function runAll() {
  // 1. Health Endpoint
  await check('1. Health check & version contract', async () => {
    const { res, json } = await fetchJson('/api/health');
    if (!res.ok) throw new Error(`Health status ${res.status}`);
    if (json.version !== '5.0.1') throw new Error(`Expected version 5.0.1, got ${json.version}`);
    if (json.status !== 'ok') throw new Error(`Expected status ok, got ${json.status}`);
  });

  // 2. Static File & CORS Preflight Check
  await check('2. Static file serving & CORS preflight', async () => {
    const res = await fetch(`${targetUrl}/`);
    if (!res.ok) throw new Error(`GET / returned status ${res.status}`);
    const html = await res.text();
    if (!html.includes('ABAPC Studio')) throw new Error('Root HTML missing title "ABAPC Studio"');

    const guiRes = await fetch(`${targetUrl}/sapgui`);
    if (!guiRes.ok) throw new Error(`GET /sapgui returned status ${guiRes.status}`);

    // Test CORS OPTIONS preflight
    const corsRes = await fetch(`${targetUrl}/api/compile`, {
      method: 'OPTIONS',
      headers: {
        'Origin': 'https://abapc-studio.vercel.app',
        'Access-Control-Request-Method': 'POST',
      },
    });
    const allowOrigin = corsRes.headers.get('access-control-allow-origin');
    if (corsRes.status >= 400 || !allowOrigin) {
      throw new Error(`CORS headers missing on OPTIONS preflight: status ${corsRes.status}, origin ${allowOrigin}`);
    }
  });

  // 3. DDIC Table Browser Fixture
  await check('3. SE16N table fixtures (/api/gui/tables)', async () => {
    const { res, json } = await fetchJson('/api/gui/tables');
    if (!res.ok) throw new Error(`Status ${res.status}`);
    if (!json.tables?.SCARR || json.tables.SCARR.length < 3) throw new Error('SCARR fixture incomplete');
    if (!json.tables?.SPFLI || json.tables.SPFLI.length < 3) throw new Error('SPFLI fixture incomplete');
    if (!json.tables?.SFLIGHT || json.tables.SFLIGHT.length < 3) throw new Error('SFLIGHT fixture incomplete');
  });

  // 4. Platform Catalog
  await check('4. DDIC/RAP platform catalogue (/api/platform/catalog)', async () => {
    const { res, json } = await fetchJson('/api/platform/catalog');
    if (!res.ok) throw new Error(`Status ${res.status}`);
    const names = (json.objects || []).map(o => o.name);
    if (!names.includes('ZDE_PRODUCT_ID')) throw new Error('Missing ZDE_PRODUCT_ID');
    if (!names.includes('ZR_PRODUCT')) throw new Error('Missing ZR_PRODUCT');
  });

  // 5. Dynamic DDIC CRUD
  await check('5. Dynamic DDIC lifecycle (DOMA, DTEL, STRU, TABL)', async () => {
    const names = ['ZTEST_DOMA', 'ZTEST_DTEL', 'ZTEST_STRU', 'ZTEST_TABL'];
    try {
      // DOMA
      let r = await fetchJson('/api/platform/create-object', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ name: names[0], kind: 'DOMA', baseType: 'CHAR(20)' }),
      });
      if (!r.json.ok) throw new Error(`Create DOMA failed: ${JSON.stringify(r.json)}`);

      // DTEL
      r = await fetchJson('/api/platform/create-object', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ name: names[1], kind: 'DTEL', baseType: names[0] }),
      });
      if (!r.json.ok) throw new Error(`Create DTEL failed: ${JSON.stringify(r.json)}`);

      // STRU
      r = await fetchJson('/api/platform/create-object', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ name: names[2], kind: 'STRU', fields: [{ name: 'ID', type: 'I' }, { name: 'NAME', type: names[1] }] }),
      });
      if (!r.json.ok) throw new Error(`Create STRU failed: ${JSON.stringify(r.json)}`);

      // TABL
      r = await fetchJson('/api/platform/create-object', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          name: names[3],
          kind: 'TABL',
          fields: [{ name: 'ID', type: 'I', key: true }, { name: 'NAME', type: names[1] }],
          initialRows: [{ ID: 10, NAME: 'Rust DDIC' }],
        }),
      });
      if (!r.json.ok) throw new Error(`Create TABL failed: ${JSON.stringify(r.json)}`);

      // Execute code utilizing this DDIC table
      const code = `REPORT zddic_test. DATA row TYPE ${names[2].toLowerCase()}. row-id = 10. row-name = 'Rust DDIC'. WRITE row-name. DATA tab TYPE TABLE OF ${names[3].toLowerCase()}. SELECT FROM ${names[3]} FIELDS id, name INTO TABLE @tab. WRITE LINES( tab ).`;
      const exec = await fetchJson('/api/execute', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ source: code, mode: 'standard', filename: 'ddic_parity.abap' }),
      });
      if (!exec.json.ok) throw new Error(`Execution failed: ${JSON.stringify(exec.json)}`);
      if (JSON.stringify(exec.json.output) !== JSON.stringify(['Rust DDIC', '1'])) {
        throw new Error(`Unexpected output: ${JSON.stringify(exec.json.output)}`);
      }
    } finally {
      // Clean up
      for (const name of [...names].reverse()) {
        await fetch(`${targetUrl}/api/platform/object?name=${encodeURIComponent(name)}`, { method: 'DELETE' });
      }
    }
  });

  // 6. Cloud Classrun (if_oo_adt_classrun)
  await check('6. Cloud classrun (if_oo_adt_classrun~main & out->write)', async () => {
    const code = `CLASS zcl_cloud_test DEFINITION PUBLIC FINAL CREATE PUBLIC.
  PUBLIC SECTION.
    INTERFACES if_oo_adt_classrun.
ENDCLASS.
CLASS zcl_cloud_test IMPLEMENTATION.
  METHOD if_oo_adt_classrun~main.
    DATA lv_val TYPE i.
    lv_val = 15 + 25.
    out->write( lv_val ).
  ENDMETHOD.
ENDCLASS.`;
    const { json } = await fetchJson('/api/execute', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ source: code, mode: 'cloud', filename: 'cloud_test.abap' }),
    });
    if (!json.ok) throw new Error(`Execution failed: ${JSON.stringify(json.diagnostics)}`);
    if (JSON.stringify(json.output) !== JSON.stringify(['40'])) {
      throw new Error(`Expected output ['40'], got ${JSON.stringify(json.output)}`);
    }
  });

  // 7. PARAMETERS injection
  await check('7. PARAMETERS injection via parameterValues', async () => {
    const code = `REPORT zparam_test. PARAMETERS p_num TYPE i. WRITE: p_num.`;
    const { json } = await fetchJson('/api/execute', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        source: code,
        mode: 'standard',
        filename: 'params.abap',
        parameterValues: { p_num: '99' },
      }),
    });
    if (!json.ok) throw new Error(`Execution failed: ${JSON.stringify(json)}`);
    if (JSON.stringify(json.output) !== JSON.stringify(['99'])) {
      throw new Error(`Expected output ['99'], got ${JSON.stringify(json.output)}`);
    }
  });

  // 8. OO Calculator
  await check('8. Object-Oriented Calculator (NEW, method call, returning value)', async () => {
    const code = `REPORT zcalc.
CLASS lcl_calc DEFINITION.
  PUBLIC SECTION.
    METHODS multiply IMPORTING iv_x TYPE i iv_y TYPE i RETURNING VALUE(rv_res) TYPE i.
ENDCLASS.
CLASS lcl_calc IMPLEMENTATION.
  METHOD multiply.
    rv_res = iv_x * iv_y.
  ENDMETHOD.
ENDCLASS.
START-OF-SELECTION.
  DATA(lo) = NEW lcl_calc( ).
  DATA(lv_res) = lo->multiply( iv_x = 7 iv_y = 6 ).
  WRITE: / 'Result:', lv_res.`;

    const { json } = await fetchJson('/api/execute', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ source: code, mode: 'standard', filename: 'calc.abap' }),
    });
    if (!json.ok) throw new Error(`Execution failed: ${JSON.stringify(json)}`);
    if (JSON.stringify(json.output) !== JSON.stringify(['Result: 42'])) {
      throw new Error(`Expected output ['Result: 42'], got ${JSON.stringify(json.output)}`);
    }
    if (json.variables?.LV_RES !== 42) {
      throw new Error(`Expected LV_RES = 42, got ${json.variables?.LV_RES}`);
    }
  });

  // 9. SWITCH Constructor Expression (5.0.1 fix)
  await check('9. SWITCH constructor expression (5.0.1 regression fix)', async () => {
    const code = `REPORT zswitch.
DATA: lv_code TYPE string VALUE 'OK'.
DATA(lv_text) = SWITCH string( lv_code WHEN 'OK' THEN 'Accepted' WHEN 'ERR' THEN 'Rejected' ELSE 'Unknown' ).
WRITE: / 'Status:', lv_text.`;

    const { json } = await fetchJson('/api/execute', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ source: code, mode: 'standard', filename: 'switch.abap' }),
    });
    if (!json.ok) throw new Error(`Execution failed: ${JSON.stringify(json)}`);
    if (JSON.stringify(json.output) !== JSON.stringify(['Status: Accepted'])) {
      throw new Error(`Expected output ['Status: Accepted'], got ${JSON.stringify(json.output)}`);
    }
    if (json.variables?.LV_TEXT !== 'Accepted') {
      throw new Error(`Expected LV_TEXT = 'Accepted', got ${json.variables?.LV_TEXT}`);
    }
  });

  // 10. Selection Preview
  await check('10. Selection screen preview & INITIALIZATION (/api/selection-preview)', async () => {
    const code = `REPORT zsel.
PARAMETERS p_val TYPE string DEFAULT 'def'.
SELECT-OPTIONS s_val FOR scarr-carrid DEFAULT 'AA'.
INITIALIZATION.
  p_val = 'init_val'.
START-OF-SELECTION.
  WRITE p_val.`;

    const { json } = await fetchJson('/api/selection-preview', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ source: code, mode: 'standard' }),
    });
    if (!json.ok) throw new Error(`Selection preview failed: ${JSON.stringify(json)}`);
    if (json.variables?.P_VAL !== 'init_val') {
      throw new Error(`Expected P_VAL = 'init_val', got ${json.variables?.P_VAL}`);
    }
    if (json.variables?.S_VAL?.[0]?.LOW !== 'AA') {
      throw new Error(`Expected S_VAL LOW = 'AA', got ${JSON.stringify(json.variables?.S_VAL)}`);
    }
  });

  // 11. Selection Event Dispatch
  await check('11. Value help event dispatch (/api/selection-event)', async () => {
    const code = `REPORT zf4.
PARAMETERS p_airline TYPE string.
AT SELECTION-SCREEN ON VALUE-REQUEST FOR p_airline.
  WRITE 'DL'.
  WRITE 'UA'.`;

    const { json } = await fetchJson('/api/selection-event', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        source: code,
        mode: 'standard',
        event: 'at-selection-screen-value-request',
        target: 'p_airline',
      }),
    });
    if (!json.ok) throw new Error(`Event failed: ${JSON.stringify(json)}`);
    if (JSON.stringify(json.output) !== JSON.stringify(['DL', 'UA'])) {
      throw new Error(`Expected output ['DL', 'UA'], got ${JSON.stringify(json.output)}`);
    }
  });

  // 12. Classic String Processing
  await check('12. Classic string processing (TRANSLATE, SPLIT, LOOP)', async () => {
    const code = `REPORT zstr.
DATA lv_raw TYPE string.
DATA lt_tokens TYPE TABLE OF string.
lv_raw = 'alpha;beta;gamma'.
TRANSLATE lv_raw TO UPPER CASE.
SPLIT lv_raw AT ';' INTO TABLE lt_tokens.
LOOP AT lt_tokens INTO DATA(token).
  WRITE token.
ENDLOOP.`;

    const { json } = await fetchJson('/api/execute', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ source: code, mode: 'standard', filename: 'strings.abap' }),
    });
    if (!json.ok) throw new Error(`Execution failed: ${JSON.stringify(json)}`);
    if (JSON.stringify(json.output) !== JSON.stringify(['ALPHA', 'BETA', 'GAMMA'])) {
      throw new Error(`Expected output ['ALPHA', 'BETA', 'GAMMA'], got ${JSON.stringify(json.output)}`);
    }
  });

  // 13. EML CRUD & Persistence
  await check('13. EML end-to-end (MODIFY -> COMMIT -> READ -> SE16N)', async () => {
    const testId = 'PARITY-' + Math.floor(Math.random() * 10000);
    const emlCode = `REPORT zeml.
MODIFY ENTITIES OF ZR_PRODUCT ENTITY Product
  CREATE FIELDS ( PRODUCT_ID PRODUCT_NAME CATEGORY PRICE CURRENCY )
  WITH VALUE #( ( PRODUCT_ID = '${testId}' PRODUCT_NAME = 'Rust Parity' CATEGORY = 'PARITY' PRICE = 99 CURRENCY = 'USD' ) )
  FAILED DATA(failed).
COMMIT ENTITIES.
READ ENTITIES OF ZI_PRODUCT ENTITY Product ALL FIELDS
  FROM VALUE #( ( PRODUCT_ID = '${testId}' ) )
  RESULT DATA(products).
LOOP AT products INTO DATA(item).
  WRITE item-PRODUCT_NAME.
ENDLOOP.`;

    const { json } = await fetchJson('/api/execute', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ source: emlCode, mode: 'standard', filename: 'eml.abap' }),
    });
    if (!json.ok) throw new Error(`EML execute failed: ${JSON.stringify(json)}`);
    if (JSON.stringify(json.output) !== JSON.stringify(['Rust Parity'])) {
      throw new Error(`Expected output ['Rust Parity'], got ${JSON.stringify(json.output)}`);
    }

    // Check visibility in SE16N
    const tableRes = await fetchJson('/api/gui/tables');
    const rows = tableRes.json.tables?.ZPRODUCT || [];
    if (!rows.some(r => r.PRODUCT_ID === testId)) {
      throw new Error(`Committed EML record ${testId} was not found in SE16N ZPRODUCT`);
    }

    // Cleanup record
    const cleanup = `REPORT zclean.
MODIFY ENTITIES OF ZR_PRODUCT ENTITY Product
  DELETE FROM VALUE #( ( PRODUCT_ID = '${testId}' ) )
  FAILED DATA(failed).
COMMIT ENTITIES.`;
    await fetchJson('/api/execute', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ source: cleanup, mode: 'standard', filename: 'cleanup.abap' }),
    });
  });

  // 14. 2MB Payload Limit Guard
  await check('14. 2MB Payload limit enforcement', async () => {
    const largeStr = 'A'.repeat(2 * 1024 * 1024 + 1024);
    const res = await fetch(`${targetUrl}/api/compile`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ source: largeStr, mode: 'standard' }),
    });
    if (res.status !== 413 && res.status !== 400 && res.status !== 500) {
      throw new Error(`Expected 413 Payload Too Large, got HTTP ${res.status}`);
    }
  });

  console.log(`\n${BOLD}====================================================${RESET}`);
  if (failed === 0) {
    console.log(`${BOLD}${GREEN}✔ ALL ${passed} PARITY CHECKS PASSED SUCCESSFULLY!${RESET}`);
    console.log(`${CYAN}The backend is fully compatible with ABAPC Studio 5.0.1.${RESET}`);
    process.exit(0);
  } else {
    console.log(`${BOLD}${RED}✘ ${failed} OF ${passed + failed} CHECKS FAILED.${RESET}`);
    process.exit(1);
  }
}

runAll().catch(err => {
  console.error(`Fatal parity runner error: ${err.stack || err}`);
  process.exit(1);
});
