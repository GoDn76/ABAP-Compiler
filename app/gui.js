const $ = id => document.getElementById(id);
function readSessionArray(key) { try { const value = JSON.parse(sessionStorage.getItem(key) || 'null'); return Array.isArray(value) ? value : []; } catch { return []; } }
function readRecentTx() { try { const value = JSON.parse(localStorage.getItem('abapc-recent-tx') || '[]'); return Array.isArray(value) ? value.filter(x => typeof x === 'string') : []; } catch { return []; } }

// Shared Virtual File System (VFS) Helpers & Starters
const DEFAULT_VFS_FILES = {
  // --- Standard ABAP Reports ---
  "z_stateless_main.abap": `REPORT z_stateless_main.

" =====================================================================
" 1. STATELESS EXECUTION & EPHEMERAL VM PLAYGROUND
" CodePen-style architecture with Bring Your Own Database (BYODB)
" =====================================================================

DATA: lv_developer TYPE string VALUE 'Full-Stack Rust/ABAP Engineer',
      lv_count     TYPE i VALUE 0.

WRITE: / '-------------------------------------------------------',
       / '  ABAPC STUDIO 5.0.1 - STATELESS COMPILER ENGINE',
       / '-------------------------------------------------------'.

WRITE: / 'Active Mode: Standard ABAP',
       / 'Role:', lv_developer.

DO 5 TIMES.
  lv_count = lv_count + 1.
  WRITE: / |  [*] Step { lv_count }: VM execution step verified.|.
ENDDO.

WRITE: / '-------------------------------------------------------',
       / 'Security Guarantee: Zero disk retention on server.',
       / 'RAII AutoCleanFile drop-guard wiped temp files in 0ms.'.
`,

  "z_oo_math.abap": `REPORT z_oo_math.

" =====================================================================
" 2. OBJECT-ORIENTED ABAP & RESILIENT EXCEPTION HANDLING
" Tests classes, method dispatch, and TRY/CATCH blocks
" =====================================================================

CLASS lcl_math DEFINITION.
  PUBLIC SECTION.
    CLASS-METHODS: divide IMPORTING iv_num1 TYPE i iv_num2 TYPE i
                          RETURNING VALUE(rv_result) TYPE i.
ENDCLASS.

CLASS lcl_math IMPLEMENTATION.
  METHOD divide.
    TRY.
        rv_result = iv_num1 / iv_num2.
      CATCH cx_sy_zerodivide.
        rv_result = -1. " Graceful fallback for division by zero
    ENDTRY.
  ENDMETHOD.
ENDCLASS.

DATA: lv_calc TYPE i.

WRITE: / '--- Testing Valid Division ---'.
lv_calc = lcl_math=>divide( iv_num1 = 100 iv_num2 = 5 ).
WRITE: / '100 / 5 =', lv_calc.

WRITE: / '--- Testing Zero Division (Caught Exception) ---'.
lv_calc = lcl_math=>divide( iv_num1 = 77 iv_num2 = 0 ).
WRITE: / '77 / 0 =', lv_calc, '(Fallback value -1 returned)'.
`,

  "z_byodb_sql.abap": `REPORT z_byodb_sql.

" =====================================================================
" 3. BRING YOUR OWN DATABASE (BYODB) - OPEN SQL
" Connect Supabase or Neon in Database Settings, or use Local Mock!
" =====================================================================

TYPES: BEGIN OF ty_flight,
         carrid TYPE string,
         connid TYPE i,
         price  TYPE i,
       END OF ty_flight.

DATA: lt_flights TYPE TABLE OF ty_flight,
      ls_flight  TYPE ty_flight.

WRITE: / 'Querying active database engine (BYODB or Local Mock)...'.

SELECT carrid connid price
  FROM sflight
  WHERE price > 450
  INTO TABLE @lt_flights.

IF sy-subrc = 0.
  WRITE: / |Found { sy-dbcnt } flights with price > 450:|.
  LOOP AT lt_flights INTO ls_flight.
    WRITE: / |[Flight] Carrier: { ls_flight-carrid } | &&
             |ConnID: { ls_flight-connid } | &&
             |Price: \${ ls_flight-price }|.
  ENDLOOP.
ELSE.
  WRITE: / 'No matching flight records found.'.
ENDIF.
`,

  // --- ABAP Cloud (if_oo_adt_classrun) ---
  "zcl_cloud_classrun.abap": `CLASS zcl_cloud_classrun DEFINITION PUBLIC FINAL CREATE PUBLIC.
  PUBLIC SECTION.
    INTERFACES if_oo_adt_classrun.
ENDCLASS.

CLASS zcl_cloud_classrun IMPLEMENTATION.
  METHOD if_oo_adt_classrun~main.
    out->write( '-------------------------------------------------------' ).
    out->write( '  ABAPC STUDIO 5.0.1 - ABAP CLOUD RUNTIME (ADT)' ).
    out->write( '-------------------------------------------------------' ).
    out->write( 'Environment: Modern ABAP Cloud OOP' ).
    out->write( 'Interface: if_oo_adt_classrun' ).
    
    DATA lv_calc TYPE i.
    lv_calc = 42 * 2.
    out->write( |Calculation Result (42 * 2): { lv_calc }| ).
    out->write( 'Execution completed successfully in cloud mode.' ).
  ENDMETHOD.
ENDCLASS.
`,

  "zcl_demo_cloud.abap": `CLASS zcl_demo_cloud DEFINITION PUBLIC FINAL CREATE PUBLIC.
  PUBLIC SECTION.
    INTERFACES if_oo_adt_classrun.
ENDCLASS.

CLASS zcl_demo_cloud IMPLEMENTATION.
  METHOD if_oo_adt_classrun~main.
    DATA: lv_tier TYPE string VALUE 'ENTERPRISE'.
    DATA(lv_discount) = SWITCH #( lv_tier
      WHEN 'BASIC'      THEN 5
      WHEN 'PRO'        THEN 15
      WHEN 'ENTERPRISE' THEN 30
      ELSE 0 ).
    out->write( |Customer Tier: { lv_tier }| ).
    out->write( |Calculated Discount: { lv_discount } percent| ).
  ENDMETHOD.
ENDCLASS.
`
};

function getFileCategory(name, content = '') {
  const n = String(name || '').toLowerCase();
  const c = String(content || '').toLowerCase();
  if (n.startsWith('zcl_') || n.startsWith('zcx_') || n.includes('cloud') || c.includes('if_oo_adt_classrun') || (c.includes('class ') && c.includes('definition') && !c.includes('report '))) {
    return 'cloud';
  }
  return 'report';
}

function getVFSFiles() {
  try {
    const raw = localStorage.getItem('abapc_vfs_files');
    if (raw) {
      const parsed = JSON.parse(raw);
      if (parsed && typeof parsed === 'object' && Object.keys(parsed).length > 0) {
        let updated = false;
        if (!parsed['zcl_cloud_classrun.abap']) {
          parsed['zcl_cloud_classrun.abap'] = DEFAULT_VFS_FILES['zcl_cloud_classrun.abap'];
          updated = true;
        }
        if (!parsed['zcl_demo_cloud.abap']) {
          parsed['zcl_demo_cloud.abap'] = DEFAULT_VFS_FILES['zcl_demo_cloud.abap'];
          updated = true;
        }
        if (updated) saveVFSFiles(parsed);
        return parsed;
      }
    }
  } catch (e) {}
  return { ...DEFAULT_VFS_FILES };
}

function saveVFSFiles(files) {
  try {
    localStorage.setItem('abapc_vfs_files', JSON.stringify(files));
  } catch (e) {}
}

function getActiveVFSFileName() {
  try {
    const active = localStorage.getItem('abapc_active_file');
    const vfs = getVFSFiles();
    if (active && vfs[active]) return active;
    return Object.keys(vfs)[0] || 'z_stateless_main.abap';
  } catch (e) {
    return 'z_stateless_main.abap';
  }
}

function setActiveVFSFileName(name) {
  try {
    localStorage.setItem('abapc_active_file', name);
  } catch (e) {}
}

function getByodbUrl() {
  try {
    return localStorage.getItem('abapc_byodb_url') || '';
  } catch (e) {
    return '';
  }
}

function prettyPrintAbap(source) {
  const keywords = [
    'REPORT', 'CLASS', 'ENDCLASS', 'METHOD', 'ENDMETHOD', 'DEFINITION', 'IMPLEMENTATION',
    'PUBLIC SECTION', 'PROTECTED SECTION', 'PRIVATE SECTION', 'INTERFACES', 'FINAL', 'CREATE PUBLIC',
    'DATA', 'TYPES', 'BEGIN OF', 'END OF', 'CONSTANTS', 'FIELD-SYMBOLS',
    'TYPE', 'VALUE', 'LIKE', 'TABLE OF', 'STANDARD TABLE OF',
    'SELECT', 'SINGLE', 'FROM', 'WHERE', 'INTO TABLE', 'INTO', 'CORRESPONDING FIELDS OF', 'UP TO', 'ROWS',
    'LOOP AT', 'ENDLOOP', 'ASSIGNING', 'TRANSPORTING', 'NO FIELDS',
    'READ TABLE', 'WITH KEY', 'BINARY SEARCH', 'APPEND', 'TO', 'INITIAL LINE', 'LINES OF',
    'IF', 'ELSE', 'ELSEIF', 'ENDIF', 'CASE', 'WHEN', 'ENDCASE', 'DO', 'ENDDO', 'WHILE', 'ENDWHILE',
    'TRY', 'CATCH', 'ENDTRY', 'CLEANUP', 'RAISE EXCEPTION',
    'CHECK', 'EXIT', 'CONTINUE', 'RETURN', 'CLEAR', 'FREE', 'REFRESH', 'ASSIGN',
    'WRITE:', 'WRITE', 'ULINE', 'SKIP', 'NEW-LINE', 'FORMAT'
  ];
  const lines = source.split('\n');
  return lines.map(line => {
    const trimmed = line.trim();
    if (trimmed.startsWith('"') || trimmed.startsWith('*')) return line;
    let res = line;
    for (const kw of keywords) {
      const regex = new RegExp(`(?<=[^a-zA-Z0-9_#-]|^)${kw.replace('-', '\\-')}(?=[^a-zA-Z0-9_#-]|\$)`, 'gi');
      res = res.replace(regex, kw);
    }
    return res;
  }).join('\n');
}

const state = {
  current: 'S000', stack: [], recent: readRecentTx(), tables: {}, metadata: {}, examples: [], executions: [],
  lastDiagnostics: readSessionArray('abapc-last-diagnostics'),
  lastOutput: readSessionArray('abapc-last-output'),
  editorPreset: '', pendingSource: null, editorFilename: getActiveVFSFileName(),
};
const txInfo = {
  S000: ['ABAPC Easy Access', 'Local development and runtime tools'],
  SE38: ['ABAP Editor', 'Edit, build, and execute programs with the ABAPC local runtime'],
  SE80: ['Object Navigator', 'Browse local examples and compiler project artifacts'],
  SE11: ['Dictionary Explorer', 'Inspect local fixture-table metadata (not SAP DDIC)'],
  SE16N: ['Table Browser', 'Browse the deterministic local SQL fixture tables'],
  SM37: ['Execution Archive', 'Review source files automatically archived by ABAPC'],
  ST22: ['Runtime Diagnostics', 'View diagnostics captured during this browser session'],
  SE93: ['Transaction Catalog', 'Browse the local transaction shortcuts available here'],
  SAPLOGON: ['Real SAP System', 'Requirements for connecting to an actual SAP system'],
  ZCERT: ['Certification Practice Center', 'Guided exercises for SAP GUI navigation and ABAP development'],
  SE24: ['Class Builder', 'Inspect a simulated local class definition and public components'],
  SE37: ['Function Builder', 'Explore local sample function interfaces and execute a training function'],
  SE91: ['Message Maintenance', 'Look up local sample message classes and message texts'],
  ZCDS: ['CDS View Builder', 'Activate and query local CDS projection views'],
  ZRAP: ['RAP / EML Workbench', 'Practice local business-object reads, staged changes, commit and rollback'],
  MM03: ['Display Material', 'Display fictional local material-master training data'],
  VA03: ['Display Sales Order', 'Display fictional local sales-order header and items'],
  ME23N: ['Display Purchase Order', 'Display fictional local purchasing document data'],
  MMBE: ['Stock Overview', 'Display fictional stock quantities by plant and storage location'],
};
const tableMeta = {
  SCARR: { description: 'Airline master data (local fixture)', fields: [['CARRID','CHAR','Airline ID'],['CARRNAME','STRING','Airline name'],['CURRCODE','CHAR','Currency code']] },
  SPFLI: { description: 'Flight connection data (local fixture)', fields: [['CARRID','CHAR','Airline ID'],['CONNID','INT','Connection number'],['COUNTRYFR','CHAR','Departure country'],['CITYFROM','STRING','Departure city'],['COUNTRYTO','CHAR','Arrival country'],['CITYTO','STRING','Arrival city']] },
  SFLIGHT: { description: 'Flight schedule data (local fixture)', fields: [['CARRID','CHAR','Airline ID'],['CONNID','INT','Connection number'],['FLDATE','DATE','Flight date'],['PRICE','DEC','Sample ticket price']] },
};
const esc = v => String(v ?? '').replaceAll('&','&amp;').replaceAll('<','&lt;').replaceAll('>','&gt;').replaceAll('"','&quot;').replaceAll("'",'&#39;');
const setStatus = text => { $('status-message').textContent = text; };
function goTo(tx, push=true) {
  const code = normalizeTx(tx);
  if (!txInfo[code]) { setStatus(`Unknown local transaction: ${code || '(empty)'}`); notice(`Transaction <b>${esc(code || '(empty)')}</b> is not available in this local workbench. Use <b>SE38</b>, <b>SE16N</b>, <b>SE11</b>, <b>SE80</b>, <b>SM37</b>, <b>ST22</b>, or <b>SE93</b>.`, 'error'); return; }
  if (push && code !== state.current) state.stack.push(state.current);
  state.current = code;
  if (code !== 'S000') { state.recent = [code, ...state.recent.filter(x => x !== code)].slice(0, 8); try { localStorage.setItem('abapc-recent-tx', JSON.stringify(state.recent)); } catch {} }
  renderRecentTree();
  $('command').value = code;
  $('breadcrumbs').textContent = `ABAPC Training / ${code}`;
  $('screen-heading').textContent = txInfo[code][0];
  $('screen-subtitle').textContent = txInfo[code][1];
  $('screen-code').textContent = code;
  document.querySelectorAll('.tree-node[data-tcode]').forEach(b => b.classList.toggle('selected', b.dataset.tcode === code));
  const scr = $('screen');
  if (scr) {
    if (code === 'SE38') scr.classList.add('screen-editor-active');
    else scr.classList.remove('screen-editor-active');
  }
  renderCurrent();
  setStatus(`${code} ready · local simulator`);
}
function normalizeTx(raw) {
  let s = String(raw || '').trim().toUpperCase();
  if (!s || s === '/' || s === '/N' || s === '/NEX' || s === '/EXIT') return 'S000';
  s = s.replace(/^\/NEX(?:\s+|$)/, '').replace(/^\/N\s*/, '').replace(/^\/O\s*/, '').replace(/^\/T\s*/, '').trim();
  if (!s) return 'S000';
  return s.split(/\s+/)[0];
}
function renderRecentTree() {
  const host = $('recent-tree'); if (!host) return;
  host.innerHTML = state.recent.length ? state.recent.map(code => `<button class="tree-node" data-recent-tx="${esc(code)}"><span class="tree-expander">›</span><span class="tree-glyph">↻</span>${esc(code)}</button>`).join('') : '<div class="tree-empty">Your recent transaction codes appear here.</div>';
  host.querySelectorAll('[data-recent-tx]').forEach(b => b.addEventListener('click', () => goTo(b.dataset.recentTx)));
  const list = $('recent-command-list'); if (list) list.innerHTML = state.recent.map(code => `<option value="${esc(code)}"></option>`).join('');
}
function notice(message, type='info') {
  const old = $('transient-notice'); if (old) old.remove();
  const el = document.createElement('div'); el.id='transient-notice'; el.className=`notice ${type}`; el.innerHTML=message;
  $('screen').prepend(el);
}
async function jsonGet(url) { const r = await fetch(url); const d = await r.json(); if (!r.ok) throw new Error(d.error || `Request failed (${r.status})`); return d; }
async function jsonPost(url, body) { const r = await fetch(url,{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify(body)}); const d=await r.json(); if (!r.ok) throw new Error(d.error || `Request failed (${r.status})`); return d; }
function renderCurrent() {
  switch (state.current) {
    case 'S000': renderHome(); break;
    case 'SE38': renderEditor(); break;
    case 'SE16N': renderTableBrowser(); break;
    case 'SE11': renderDictionary(); break;
    case 'SE80': renderObjectNavigator(); break;
    case 'SM37': renderExecutionArchive(); break;
    case 'ST22': renderDiagnostics(); break;
    case 'SE93': renderTransactions(); break;
    case 'SAPLOGON': renderRealSapInfo(); break;
    case 'ZCERT': renderPracticeCenter(); break;
    case 'SE24': renderClassBuilder(); break;
    case 'SE37': renderFunctionBuilder(); break;
    case 'SE91': renderMessageCatalog(); break;
    case 'ZCDS': renderCdsWorkbench(); break;
    case 'ZRAP': renderRapWorkbench(); break;
    case 'MM03': case 'VA03': case 'ME23N': case 'MMBE': renderMockBusinessTx(state.current); break;
    default: renderHome();
  }
}
function renderHome() {
  $('screen').innerHTML = `
    <div class="welcome-banner"><h2>Welcome to SAP GUI Practice Simulator</h2><p>A local, SAP GUI-style training environment for practicing command-field navigation, ABAP development concepts, dictionary/table inspection, and selected business-document display flows. All records are fictional training data.</p></div>
    <div class="section-label">SAP Menu</div>
    <div class="cards">
      <button class="card" data-open="ZCERT"><div class="card-icon">▤</div><strong>Certification Practice</strong><p>Guided tasks for transaction navigation, ABAP basics, and table inspection.</p></button>
      <button class="card" data-open="SE38"><div class="card-icon">▧</div><strong>ABAP Editor <span class="tcode">SE38</span></strong><p>Write, build, and execute supported ABAP source in the local ABAPC runtime.</p></button>
      <button class="card" data-open="SE24"><div class="card-icon">◇</div><strong>Class Builder <span class="tcode">SE24</span></strong><p>Inspect simulated class components and load a class-run example.</p></button>
      <button class="card" data-open="SE11"><div class="card-icon">▤</div><strong>Dictionary Explorer <span class="tcode">SE11</span></strong><p>Inspect local table structures and field metadata.</p></button>
      <button class="card" data-open="SE16N"><div class="card-icon">▦</div><strong>Table Browser <span class="tcode">SE16N</span></strong><p>Filter deterministic sample data from SCARR, SPFLI, SFLIGHT and ZPRODUCT.</p></button>
      <button class="card" data-open="ZCDS"><div class="card-icon">◫</div><strong>CDS View Builder <span class="tcode">ZCDS</span></strong><p>Activate and query a local CDS view entity projection.</p></button>
      <button class="card" data-open="ZRAP"><div class="card-icon">⇄</div><strong>RAP / EML Workbench <span class="tcode">ZRAP</span></strong><p>Stage business-object changes and practice commit or rollback.</p></button>
      <button class="card" data-open="MM03"><div class="card-icon">▤</div><strong>Material Display <span class="tcode">MM03</span></strong><p>Practice a master-data display workflow with local example materials.</p></button>
    </div>
    <div class="section-label">Quick Transaction Start</div>
    <div class="panel"><div class="panel-body"><div class="toolbar-row"><button class="secondary-btn" data-open="SE38">SE38 · ABAP Editor</button><button class="secondary-btn" data-open="SE11">SE11 · Dictionary</button><button class="secondary-btn" data-open="SE16N">SE16N · Table Browser</button><button class="secondary-btn" data-open="SE24">SE24 · Class Builder</button><button class="secondary-btn" data-open="SE37">SE37 · Function Builder</button><button class="secondary-btn" data-open="ZCDS">ZCDS · CDS</button><button class="secondary-btn" data-open="ZRAP">ZRAP · RAP / EML</button><button class="secondary-btn" data-open="ZCERT">Practice tasks</button></div><div class="small-note">Enter a transaction in the Command field and press Enter. Supported local shortcuts include <span class="mono">/nSE38</span>, <span class="mono">/oSE16N</span>, and <span class="mono">/n</span> to return to Easy Access.</div></div></div>
    <div class="section-label">Learning Scope</div>
    <div class="panel"><div class="panel-body"><div class="two-col"><div><b>Available locally</b><p class="small-note">Transaction-code navigation, ABAPC editor/runtime, class and function examples, local metadata/table browsing, fictional material/sales/purchasing displays, execution history, and diagnostics.</p></div><div><b>Not an SAP backend</b><p class="small-note">This simulator does not implement SAP logon, authorizations, transport requests, customizing, real DDIC/repository access, database commits, or real SAP business transactions.</p></div></div></div></div>`;
  bindOpenButtons();
}
function bindOpenButtons() { document.querySelectorAll('[data-open]').forEach(b=>b.addEventListener('click',()=>goTo(b.dataset.open))); }
async function loadExamples() { try { const d=await jsonGet('/api/examples'); state.examples=d.files||[]; return state.examples; } catch { return []; } }
async function renderEditor(presetName = state.editorPreset || '') {
  const screen = $('screen');
  screen.classList.add('screen-editor-active');
  screen.innerHTML = '<div class="empty-state">Loading editor…</div>';

  const vfs = getVFSFiles();
  let currentFile = state.editorFilename || getActiveVFSFileName();
  if (!vfs[currentFile]) {
    currentFile = Object.keys(vfs)[0] || 'z_stateless_main.abap';
  }
  state.editorFilename = currentFile;
  setActiveVFSFileName(currentFile);

  let source = state.pendingSource !== null ? state.pendingSource : (vfs[currentFile] || '');
  if (state.pendingSource !== null) state.pendingSource = null;

  let currentCategory = getFileCategory(currentFile, source);
  let vfsFilter = 'all'; // 'all' | 'cloud' | 'report'
  let vfsSearch = '';
  let autoSaveEnabled = localStorage.getItem('abapc_autosave_enabled') !== '0';
  let activeOutputTab = 'output'; // 'output' | 'diagnostics' | 'variables' | 'ir'
  let lastOutputLines = state.lastOutput || [];
  let lastDiagnosticsList = state.lastDiagnostics || [];
  let lastVariablesMap = {};
  let lastIrData = null;

  let parameterSpecs = null;
  let parameterSource = null;
  let parameterMode = null;
  state.editorPreset = presetName || state.editorPreset;

  const defaultMode = currentCategory === 'cloud' ? 'cloud' : 'standard';

  // Render 3-part layout container: VFS Pane (Middle) + Editor Stage (Right: Top Editor + Bottom Output)
  screen.innerHTML = `
    <div class="sap-editor-container" id="sap-editor-container">
      <!-- Middle Column: (SEPARATE for CLOUD and REPORT) File System -->
      <aside class="sap-vfs-pane" id="sap-vfs-pane">
        <div class="vfs-header">
          <strong><span>📁</span> Repository Files</strong>
          <span style="display:flex;align-items:center;gap:6px;">
            <span class="vfs-badge report" id="vfs-total-count">0 files</span>
            <button class="sap-pane-toggle-btn" id="btn-toggle-vfs-collapse" title="Collapse File System Pane">◀</button>
          </span>
        </div>
        <div class="vfs-cat-tabs">
          <button class="vfs-cat-btn active" data-vfs-filter="all" id="vfs-tab-all">All</button>
          <button class="vfs-cat-btn" data-vfs-filter="cloud" id="vfs-tab-cloud">☁ Cloud</button>
          <button class="vfs-cat-btn" data-vfs-filter="report" id="vfs-tab-report">📄 Report</button>
        </div>
        <div class="vfs-actions-bar">
          <button id="vfs-btn-new-report" title="Create a new Standard ABAP Report (REPORT)">+ New Report</button>
          <button id="vfs-btn-new-cloud" title="Create a new ABAP Cloud Class (if_oo_adt_classrun)">+ New Cloud</button>
        </div>
        <div class="vfs-search-box">
          <input type="text" id="vfs-search-input" placeholder="Search programs (e.g. z_...)" spellcheck="false" />
        </div>
        <div class="vfs-file-scroll" id="vfs-file-list"></div>
      </aside>

      <!-- Collapsed rail for VFS when folded -->
      <div class="vfs-collapsed-rail" id="vfs-collapsed-rail" title="Expand Virtual File System" style="display:none;">
        <span class="rail-icon">▶</span>
        <span class="vertical-title">VFS FILES</span>
      </div>

      <!-- Draggable Horizontal Splitter between VFS and Editor -->
      <div class="sap-horizontal-resizer" id="sap-horizontal-resizer" title="Drag to adjust File System width"></div>

      <!-- Right Stage: CODE EDITOR (top) & OUTPUT SCROLLABLE (bottom) -->
      <main class="sap-editor-stage">
        <!-- Top: CODE EDITOR -->
        <section class="sap-editor-top" id="sap-editor-top">
          <div class="sap-editor-toolbar">
            <div class="editor-active-file-pill">
              <span id="editor-active-icon">${currentCategory === 'cloud' ? '☁' : '📄'}</span>
              <span id="editor-active-name">${esc(currentFile)}</span>
              <span class="vfs-badge ${currentCategory}" id="editor-active-badge">${currentCategory.toUpperCase()}</span>
            </div>
            <span class="editor-status-pill saved" id="editor-save-status">✓ Saved</span>

            <label for="gui-mode" style="font-size:10px;margin-left:4px;color:#456280;">Language</label>
            <select id="gui-mode" style="height:23px;font-size:11px;padding:1px 4px;border:1px solid #9db0c4;background:#fff;">
              <option value="standard" ${defaultMode === 'standard' ? 'selected' : ''}>Standard ABAP</option>
              <option value="cloud" ${defaultMode === 'cloud' ? 'selected' : ''}>ABAP Cloud</option>
            </select>

            <button class="secondary-btn small-btn" id="btn-autosave-toggle" title="Toggle 1500ms debounced auto-save">${autoSaveEnabled ? 'Auto-Save: ON' : 'Auto-Save: OFF'}</button>
            <button class="secondary-btn small-btn" id="gui-vfs-save" title="Save file to Virtual File System (Ctrl+S)">💾 Save</button>
            <button class="secondary-btn small-btn" id="gui-build" title="Check syntax (Ctrl+F2)">⚙ Check</button>
            <button class="secondary-btn small-btn" id="gui-pretty" title="Pretty Printer - uppercase keywords (Shift+F1)">✨ Pretty Printer</button>
            <button class="secondary-btn small-btn" id="gui-byodb" title="Database Settings (BYODB PostgreSQL)">🗄 BYODB</button>

            <span style="flex:1;"></span>

            <button class="primary-btn small-btn" id="gui-run" style="font-weight:bold;padding:4px 12px;" title="Execute program in ephemeral VM (F8)">▶ Execute (F8)</button>
          </div>

          <div id="gui-parameters" style="overflow-y:auto;padding:4px 8px;background:#f8fafc;border-bottom:1px solid #c9d8e6;display:none;"></div>
          <div class="sap-params-resizer" id="sap-params-resizer" title="Drag to adjust Selection Screen height" style="display:none;"></div>

          <div class="editor-body-wrap">
            <div class="code-gutter" id="gui-gutter"></div>
            <textarea class="code-editor-input" id="gui-source" spellcheck="false" aria-label="ABAP Code Editor">${esc(source)}</textarea>
          </div>
        </section>

        <!-- Draggable Vertical Splitter between Editor and Output -->
        <div class="sap-vertical-resizer" id="sap-vertical-resizer" title="Drag to adjust Editor and Output Console height"></div>

        <!-- Bottom: OUTPUT SCROLLABLE -->
        <section class="sap-output-bottom" id="sap-output-bottom">
          <div class="sap-output-header">
            <span class="output-title"><span>▶</span> Output</span>
            <span class="output-metric-pill info" id="output-status-pill">Ready</span>
            <span id="output-meta-time" style="font-size:10px;color:#94a3b8;">Time: —</span>
            <span id="output-meta-steps" style="font-size:10px;color:#94a3b8;">Steps: —</span>
            <span id="output-meta-db" style="font-size:10px;color:#94a3b8;">DB: Ephemeral</span>

            <span style="flex:1;"></span>

            <button class="output-tab-btn active" data-output-tab="output">📋 Output List</button>
            <button class="output-tab-btn" data-output-tab="diagnostics">⚠ Diagnostics (<span id="output-diag-count">0</span>)</button>
            <button class="output-tab-btn" data-output-tab="variables">🔍 Variables</button>
            <button class="output-tab-btn" data-output-tab="ir">⚡ IR Code</button>

            <button class="secondary-btn small-btn" id="btn-copy-output" title="Copy output to clipboard" style="height:21px;padding:2px 6px;font-size:10px;">📋 Copy</button>
            <button class="secondary-btn small-btn" id="btn-clear-output" title="Clear console output" style="height:21px;padding:2px 6px;font-size:10px;">✕ Clear</button>
            <button class="sap-pane-toggle-btn" id="btn-toggle-output-collapse" title="Collapse / Expand Output Console">▼ Output</button>
          </div>

          <div class="output-scrollable-body" id="gui-output-body">
            <div class="spool-line" style="color:#64748b;">Ready. Select Execute (F8) to run this program in the ephemeral ABAPC VM.</div>
          </div>
        </section>
      </main>
    </div>
  `;

  // Update Line Numbers Gutter
  const updateGutter = () => {
    const src = $('gui-source');
    const gtr = $('gui-gutter');
    if (!src || !gtr) return;
    const lines = (src.value || '').split('\n').length;
    gtr.innerHTML = Array.from({ length: lines }, (_, i) => i + 1).join('<br>');
    gtr.scrollTop = src.scrollTop;
  };

  updateGutter();
  $('gui-source').addEventListener('scroll', () => {
    const gtr = $('gui-gutter');
    if (gtr) gtr.scrollTop = $('gui-source').scrollTop;
  });

  // Render VFS File Explorer List (Separated into CLOUD and REPORT)
  const renderVfsList = () => {
    const curVfs = getVFSFiles();
    const allKeys = Object.keys(curVfs).sort((a, b) => a.localeCompare(b));
    const query = vfsSearch.trim().toLowerCase();

    const cloudFiles = [];
    const reportFiles = [];

    for (const name of allKeys) {
      if (query && !name.toLowerCase().includes(query)) continue;
      const cat = getFileCategory(name, curVfs[name]);
      if (cat === 'cloud') cloudFiles.push({ name, cat, code: curVfs[name] });
      else reportFiles.push({ name, cat, code: curVfs[name] });
    }

    $('vfs-total-count').textContent = `${allKeys.length} files`;
    $('vfs-tab-all').textContent = `All (${allKeys.length})`;
    $('vfs-tab-cloud').textContent = `☁ Cloud (${cloudFiles.length})`;
    $('vfs-tab-report').textContent = `📄 Report (${reportFiles.length})`;

    const host = $('vfs-file-list');
    let html = '';

    // 1. Cloud Section
    if (vfsFilter === 'all' || vfsFilter === 'cloud') {
      html += `
        <div class="vfs-section-title">
          <span>☁ ABAP CLOUD OBJECTS</span>
          <span style="font-weight:normal;opacity:0.8;">${cloudFiles.length}</span>
        </div>
      `;
      if (!cloudFiles.length) {
        html += `<div style="padding:6px 10px;font-size:10px;color:#8599ad;font-style:italic;">No cloud classes.</div>`;
      } else {
        html += cloudFiles.map(f => {
          const lines = (f.code || '').split('\n').length;
          const isActive = f.name === currentFile;
          return `
            <div class="vfs-file-item ${isActive ? 'active' : ''}" data-vfs-open="${esc(f.name)}">
              <span>☁</span>
              <span class="vfs-file-name" title="${esc(f.name)}">${esc(f.name)}</span>
              <span class="vfs-badge cloud">CLOUD</span>
              <span class="vfs-file-meta">${lines}L</span>
              <button class="vfs-del-btn" data-vfs-del="${esc(f.name)}" title="Delete file">✕</button>
            </div>
          `;
        }).join('');
      }
    }

    // 2. Report Section
    if (vfsFilter === 'all' || vfsFilter === 'report') {
      html += `
        <div class="vfs-section-title">
          <span>📄 STANDARD ABAP REPORTS</span>
          <span style="font-weight:normal;opacity:0.8;">${reportFiles.length}</span>
        </div>
      `;
      if (!reportFiles.length) {
        html += `<div style="padding:6px 10px;font-size:10px;color:#8599ad;font-style:italic;">No reports.</div>`;
      } else {
        html += reportFiles.map(f => {
          const lines = (f.code || '').split('\n').length;
          const isActive = f.name === currentFile;
          return `
            <div class="vfs-file-item ${isActive ? 'active' : ''}" data-vfs-open="${esc(f.name)}">
              <span>📄</span>
              <span class="vfs-file-name" title="${esc(f.name)}">${esc(f.name)}</span>
              <span class="vfs-badge report">REPORT</span>
              <span class="vfs-file-meta">${lines}L</span>
              <button class="vfs-del-btn" data-vfs-del="${esc(f.name)}" title="Delete file">✕</button>
            </div>
          `;
        }).join('');
      }
    }

    host.innerHTML = html;

    // Bind item click
    host.querySelectorAll('[data-vfs-open]').forEach(item => {
      item.addEventListener('click', (e) => {
        if (e.target.closest('[data-vfs-del]')) return;
        const targetName = item.dataset.vfsOpen;
        if (targetName === currentFile) return;

        // Auto-flush current file
        const liveVfs = getVFSFiles();
        liveVfs[currentFile] = $('gui-source').value;
        saveVFSFiles(liveVfs);

        // Switch
        currentFile = targetName;
        state.editorFilename = targetName;
        setActiveVFSFileName(targetName);
        const newCode = liveVfs[targetName] || '';
        $('gui-source').value = newCode;
        currentCategory = getFileCategory(targetName, newCode);

        $('editor-active-icon').textContent = currentCategory === 'cloud' ? '☁' : '📄';
        $('editor-active-name').textContent = targetName;
        $('editor-active-badge').className = `vfs-badge ${currentCategory}`;
        $('editor-active-badge').textContent = currentCategory.toUpperCase();
        $('gui-mode').value = currentCategory === 'cloud' ? 'cloud' : 'standard';

        $('editor-save-status').className = 'editor-status-pill saved';
        $('editor-save-status').textContent = '✓ Saved';

        updateGutter();
        renderVfsList();
        resetSelectionScreen();
        setStatus(`Loaded: ${targetName}`);
      });
    });

    // Bind delete click
    host.querySelectorAll('[data-vfs-del]').forEach(btn => {
      btn.addEventListener('click', (e) => {
        e.stopPropagation();
        const delName = btn.dataset.vfsDel;
        const liveVfs = getVFSFiles();
        const keys = Object.keys(liveVfs);
        if (keys.length <= 1) {
          alert('Cannot delete the only remaining program in the Virtual File System.');
          return;
        }
        if (!confirm(`Are you sure you want to delete ${delName} from the Virtual File System?`)) return;

        delete liveVfs[delName];
        saveVFSFiles(liveVfs);

        if (delName === currentFile) {
          const next = Object.keys(liveVfs)[0];
          currentFile = next;
          state.editorFilename = next;
          setActiveVFSFileName(next);
          $('gui-source').value = liveVfs[next] || '';
          currentCategory = getFileCategory(next, liveVfs[next]);
          $('editor-active-icon').textContent = currentCategory === 'cloud' ? '☁' : '📄';
          $('editor-active-name').textContent = next;
          $('editor-active-badge').className = `vfs-badge ${currentCategory}`;
          $('editor-active-badge').textContent = currentCategory.toUpperCase();
          $('gui-mode').value = currentCategory === 'cloud' ? 'cloud' : 'standard';
          updateGutter();
        }
        renderVfsList();
        showToast(`Deleted ${delName}`);
      });
    });
  };

  renderVfsList();

  // Category filter tabs
  document.querySelectorAll('[data-vfs-filter]').forEach(tab => {
    tab.addEventListener('click', () => {
      document.querySelectorAll('[data-vfs-filter]').forEach(t => t.classList.remove('active'));
      tab.classList.add('active');
      vfsFilter = tab.dataset.vfsFilter;
      renderVfsList();
    });
  });

  // Search box
  $('vfs-search-input').addEventListener('input', (e) => {
    vfsSearch = e.target.value;
    renderVfsList();
  });

  // + New Report Button
  $('vfs-btn-new-report').addEventListener('click', () => {
    let name = prompt('New Standard ABAP Report name (e.g. Z_MY_REPORT.abap):');
    if (!name || !name.trim()) return;
    name = name.trim();
    if (!name.endsWith('.abap')) name += '.abap';
    const curVfs = getVFSFiles();
    if (curVfs[name]) {
      alert(`Program "${name}" already exists in Virtual File System.`);
      return;
    }
    const cleanRep = name.replace('.abap', '').toLowerCase();
    curVfs[name] = `REPORT ${cleanRep}.

" =====================================================================
" STANDARD ABAP EXECUTABLE REPORT
" =====================================================================

DATA: lv_msg TYPE string VALUE 'Hello from SE38 Report'.

WRITE: / '-------------------------------------------------------',
       / lv_msg,
       / 'Executed in Stateless Ephemeral VM.',
       / '-------------------------------------------------------'.
`;
    saveVFSFiles(curVfs);
    currentFile = name;
    state.editorFilename = name;
    setActiveVFSFileName(name);
    $('gui-source').value = curVfs[name];
    currentCategory = 'report';
    $('editor-active-icon').textContent = '📄';
    $('editor-active-name').textContent = name;
    $('editor-active-badge').className = 'vfs-badge report';
    $('editor-active-badge').textContent = 'REPORT';
    $('gui-mode').value = 'standard';
    updateGutter();
    renderVfsList();
    showToast(`Created report: ${name}`);
  });

  // + New Cloud Class Button
  $('vfs-btn-new-cloud').addEventListener('click', () => {
    let name = prompt('New ABAP Cloud Class name (e.g. ZCL_MY_SERVICE.abap):');
    if (!name || !name.trim()) return;
    name = name.trim();
    if (!name.endsWith('.abap')) name += '.abap';
    const curVfs = getVFSFiles();
    if (curVfs[name]) {
      alert(`Class "${name}" already exists in Virtual File System.`);
      return;
    }
    const cleanCls = name.replace('.abap', '').toLowerCase();
    curVfs[name] = `CLASS ${cleanCls} DEFINITION PUBLIC FINAL CREATE PUBLIC.
  PUBLIC SECTION.
    INTERFACES if_oo_adt_classrun.
ENDCLASS.

CLASS ${cleanCls} IMPLEMENTATION.
  METHOD if_oo_adt_classrun~main.
    out->write( '-------------------------------------------------------' ).
    out->write( '  ABAPC STUDIO 5.0.1 - ABAP CLOUD RUNTIME' ).
    out->write( '-------------------------------------------------------' ).
    out->write( 'Interface: if_oo_adt_classrun' ).
    out->write( 'Hello ABAP Cloud!' ).
  ENDMETHOD.
ENDCLASS.
`;
    saveVFSFiles(curVfs);
    currentFile = name;
    state.editorFilename = name;
    setActiveVFSFileName(name);
    $('gui-source').value = curVfs[name];
    currentCategory = 'cloud';
    $('editor-active-icon').textContent = '☁';
    $('editor-active-name').textContent = name;
    $('editor-active-badge').className = 'vfs-badge cloud';
    $('editor-active-badge').textContent = 'CLOUD';
    $('gui-mode').value = 'cloud';
    updateGutter();
    renderVfsList();
    showToast(`Created cloud class: ${name}`);
  });

  // Selection Screen Helpers
  const resetSelectionScreen = () => {
    parameterSpecs = null;
    parameterSource = null;
    parameterMode = null;
    const p = $('gui-parameters');
    if (p) {
      p.innerHTML = '';
      p.style.display = 'none';
    }
    const pr = $('sap-params-resizer');
    if (pr) pr.style.display = 'none';
    $('gui-run').textContent = '▶ Execute (F8)';
  };

  const rangeRowMarkup = (v, i, rowIndex, initialRow = null) => {
    const noIntervals = !!v.noIntervals;
    const sign = String(initialRow?.SIGN ?? initialRow?.sign ?? 'I').toUpperCase();
    const option = String(initialRow?.OPTION ?? initialRow?.option ?? 'EQ').toUpperCase();
    const low = String(initialRow?.LOW ?? initialRow?.low ?? '');
    const high = String(initialRow?.HIGH ?? initialRow?.high ?? '');
    const opts = (noIntervals ? ['EQ', 'NE', 'GT', 'GE', 'LT', 'LE', 'CP', 'NP'] : ['EQ', 'NE', 'GT', 'GE', 'LT', 'LE', 'BT', 'NB', 'CP', 'NP']).map(o => `<option value="${o}" ${o === option ? 'selected' : ''}>${o}</option>`).join('');
    return `<div class="range-row" data-range-row data-range-index="${rowIndex}">
      <select aria-label="Include or exclude" data-range-part="sign"><option value="I" ${sign === 'I' ? 'selected' : ''}>Include</option><option value="E" ${sign === 'E' ? 'selected' : ''}>Exclude</option></select>
      <select aria-label="Selection option" data-range-part="option">${opts}</select>
      <input aria-label="Low value" placeholder="Low value" value="${esc(low)}" data-range-part="low" data-f4-name="${esc(v.name)}" data-f4-row="${rowIndex}" />
      ${noIntervals ? '' : `<input aria-label="High value" placeholder="High value" value="${esc(high)}" data-range-part="high" ${['BT', 'NB'].includes(option) ? '' : 'disabled'} />`}
      <button class="secondary-btn small-btn" type="button" data-range-f4 title="Value help">F4</button>
      ${v.noExtension ? '' : '<button class="secondary-btn small-btn" type="button" data-range-remove title="Remove row">−</button>'}
    </div>`;
  };

  const syncRangeRow = (row) => {
    const option = row.querySelector('[data-range-part="option"]')?.value || 'EQ';
    const high = row.querySelector('[data-range-part="high"]');
    if (!high) return;
    const enabled = ['BT', 'NB'].includes(option);
    high.disabled = !enabled;
    if (!enabled) high.value = '';
  };

  const bindRangeRows = (root) => root.querySelectorAll('[data-range-row]').forEach(row => {
    syncRangeRow(row);
    const select = row.querySelector('[data-range-part="option"]');
    if (select && select.dataset.bound !== '1') {
      select.dataset.bound = '1';
      select.addEventListener('change', () => syncRangeRow(row));
    }
  });

  const drawSelectionScreen = (specs) => {
    const host = $('gui-parameters');
    const pResizer = $('sap-params-resizer');
    host.style.display = 'block';
    if (pResizer) pResizer.style.display = 'flex';
    const savedH = localStorage.getItem('abapc_params_height') || '180';
    host.style.height = savedH + 'px';
    host.style.maxHeight = 'none';
    const visible = specs.filter(v => !v.noDisplay);
    const radioDefault = v => v.initialValue === true || String(v.initialValue ?? '').toUpperCase() === 'X' || String(v.initialValue ?? '').toUpperCase() === 'TRUE';
    const fields = visible.map((v, i) => {
      const type = v.type?.name || v.type?.kind || 'string';
      const def = v.initialValue === undefined ? (type === 'i' || type === 'int8' || type.startsWith('decfloat') ? '0' : '') : String(v.initialValue);
      const label = String(v.name || '').toUpperCase();
      if (v.selectionOptions) {
        const defaultRows = Array.isArray(v.initialValue) && v.initialValue.length ? v.initialValue : [null];
        return `<label>${esc(label)}${v.obligatory ? ' *' : ''}<button class="secondary-btn small-btn field-help-btn" type="button" data-help-param="${esc(v.name)}" title="Field help">?</button><div class="small-note range-reference">${esc(v.forField || 'Selection range')}</div></label><div class="range-host" data-range-host data-range-name="${esc(v.name)}" data-range-required="${v.obligatory ? '1' : '0'}" data-range-no-extension="${v.noExtension ? '1' : '0'}"> <div class="range-rows">${defaultRows.map((row, j) => rangeRowMarkup(v, i, j, row)).join('')}</div>${v.noExtension ? '' : `<button class="secondary-btn small-btn" type="button" data-range-add="${i}">+ Add range</button>`}</div>`;
      }
      if (v.radiobuttonGroup) {
        const checked = radioDefault(v);
        return `<label for="gui-param-${i}">${esc(label)}${v.obligatory ? ' *' : ''}<button class="secondary-btn small-btn field-help-btn" type="button" data-help-param="${esc(v.name)}" title="Field help">?</button><div class="small-note">Group ${esc(v.radiobuttonGroup)}</div></label><div><input id="gui-param-${i}" type="radio" name="gui-radio-${esc(v.radiobuttonGroup)}" data-param-name="${esc(v.name)}" data-param-radio-group="${esc(v.radiobuttonGroup)}" ${checked ? 'checked' : ''}/></div>`;
      }
      if (v.checkbox) {
        return `<label for="gui-param-${i}">${esc(label)}${v.obligatory ? ' *' : ''}<button class="secondary-btn small-btn field-help-btn" type="button" data-help-param="${esc(v.name)}" title="Field help">?</button></label><div><input id="gui-param-${i}" type="checkbox" data-param-name="${esc(v.name)}" data-param-required="${v.obligatory ? '1' : '0'}" ${def === 'X' || def === 'true' || def === '1' ? 'checked' : ''}/></div>`;
      }
      const inputType = ['i', 'int8', 'decfloat16', 'decfloat34'].includes(type) ? 'number' : 'text';
      const valueInput = `<input id="gui-param-${i}" data-param-name="${esc(v.name)}" data-param-required="${v.obligatory ? '1' : '0'}" data-param-type="${esc(type)}" type="${inputType}" value="${esc(def)}" ${v.obligatory ? 'required' : ''}/>`;
      return `<label for="gui-param-${i}">${esc(label)}${v.obligatory ? ' *' : ''}<button class="secondary-btn small-btn field-help-btn" type="button" data-help-param="${esc(v.name)}" title="Field help">?</button></label><div class="parameter-input-wrap">${valueInput}${v.valueHelp ? `<button class="secondary-btn small-btn" type="button" data-f4-param="${esc(v.name)}">F4</button>` : ''}</div>`;
    }).join('');

    host.innerHTML = `<div class="panel selection-screen-panel"><div class="panel-title" style="display:flex;justify-content:space-between;align-items:center;"><span>Selection Screen <span class="muted">· 1000 · local simulation</span></span><button class="sap-pane-toggle-btn" id="btn-toggle-params-collapse" type="button" title="Collapse / Expand Parameters">▼ Parameters</button></div><div class="panel-body" id="params-panel-body"><p class="small-note">Enter report parameters and ranges, then choose <b>Run with inputs</b>. F4 uses local training values.</p><div class="sap-form-grid">${fields}</div></div></div>`;
    const toggleBtn = $('btn-toggle-params-collapse');
    const panelBody = $('params-panel-body');
    if (toggleBtn && panelBody) {
      toggleBtn.addEventListener('click', () => {
        const isHidden = panelBody.style.display === 'none';
        if (isHidden) {
          panelBody.style.display = 'block';
          toggleBtn.textContent = '▼ Parameters';
          const savedH = localStorage.getItem('abapc_params_height') || '180';
          host.style.height = savedH + 'px';
          if (pResizer) pResizer.style.display = 'flex';
        } else {
          panelBody.style.display = 'none';
          toggleBtn.textContent = '▲ Parameters';
          host.style.height = 'auto';
          if (pResizer) pResizer.style.display = 'none';
        }
      });
    }
    bindRangeRows(host);
    host.querySelectorAll('[data-range-add]').forEach(button => button.addEventListener('click', () => {
      const hostEl = button.closest('[data-range-host]');
      if (!hostEl) return;
      const name = hostEl.dataset.rangeName;
      const v = specs.find(x => String(x.name).toUpperCase() === String(name).toUpperCase());
      if (!v) return;
      const rows = hostEl.querySelector('.range-rows');
      if (!rows) return;
      const count = rows.querySelectorAll('[data-range-row]').length;
      rows.insertAdjacentHTML('beforeend', rangeRowMarkup(v, Number(button.dataset.rangeAdd), count));
      const newest = rows.lastElementChild;
      if (newest) {
        newest.querySelector('[data-range-remove]')?.addEventListener('click', () => newest.remove());
        bindRangeRows(newest);
      }
      bindF4Buttons(rows);
    }));
    host.querySelectorAll('[data-range-remove]').forEach(button => button.addEventListener('click', () => button.closest('[data-range-row]')?.remove()));
    bindF4Buttons(host);
  };

  const openValueHelp = async (name, input) => {
    const v = parameterSpecs?.find(x => String(x.name).toUpperCase() === String(name).toUpperCase());
    let values = [];
    try {
      const eventResult = await jsonPost('/api/selection-event', { source: $('gui-source').value, mode: $('gui-mode').value, filename: currentFile, event: 'at-selection-screen-value-request', target: name, parameterValues: { [name]: input.value } });
      if (eventResult.ok && Array.isArray(eventResult.output) && eventResult.output.length) values = eventResult.output.map(String).map(x => x.trim()).filter(Boolean);
    } catch {}
    try {
      const tables = await getTables();
      const reference = String(v?.forField || '').toUpperCase().split('-');
      const table = reference.length > 1 ? reference[0] : '';
      const field = reference.length > 1 ? reference.slice(1).join('-') : '';
      if (!values.length && table && field && Array.isArray(tables[table])) values = tables[table].map(row => row[field] ?? row[field.toUpperCase()]).filter(x => x !== undefined && x !== null).map(String);
    } catch {}
    if (!values.length) {
      const key = String(v?.valueHelp || v?.forField || name).toUpperCase();
      if (key.includes('CARR') || key.includes('AIRLINE')) values = ['AA', 'LH', 'SQ'];
      else if (key.includes('COUNTRY')) values = ['DE', 'IN', 'JP', 'SG', 'US'];
      else if (key.includes('DATE') || key.includes('DATUM')) values = ['2026-10-01', '2026-10-02', '2026-10-03'];
      else values = ['A', 'B', 'C', '100', '200', '300'];
    }
    values = [...new Set(values)];
    document.querySelector('.f4-overlay')?.remove();
    const overlay = document.createElement('div');
    overlay.className = 'f4-overlay';
    overlay.innerHTML = `<div class="f4-dialog" role="dialog" aria-modal="true" aria-label="Value help"><div class="f4-title">Value Help <span>${esc(String(name).toUpperCase())}</span><button type="button" data-f4-close>×</button></div><input class="f4-search" placeholder="Search values…" aria-label="Search values"><div class="f4-values">${values.map(val => `<button type="button" class="f4-value" data-f4-value="${esc(val)}">${esc(val)}</button>`).join('')}</div><div class="small-note">Local values for practice only.</div></div>`;
    document.body.append(overlay);
    overlay.addEventListener('click', e => {
      if (e.target === overlay || e.target.closest('[data-f4-close]')) { overlay.remove(); return; }
      const item = e.target.closest('[data-f4-value]');
      if (item) {
        input.value = item.dataset.f4Value;
        input.dispatchEvent(new Event('input', { bubbles: true }));
        overlay.remove();
        input.focus();
      }
    });
    overlay.querySelector('.f4-search').addEventListener('input', e => {
      const q = e.target.value.toLowerCase();
      overlay.querySelectorAll('[data-f4-value]').forEach(b => b.hidden = !b.dataset.f4Value.toLowerCase().includes(q));
    });
    overlay.querySelector('.f4-search').focus();
  };

  const openFieldHelp = async (name) => {
    let message = 'Local ABAPC selection field. Enter a value manually or use F4 for available local training values.';
    try {
      const result = await jsonPost('/api/selection-event', { source: $('gui-source').value, mode: $('gui-mode').value, filename: currentFile, event: 'at-selection-screen-help-request', target: name, parameterValues: {} });
      if (result.ok && result.output?.length) message = result.output.map(String).join('\n');
    } catch {}
    document.querySelector('.f4-overlay')?.remove();
    const overlay = document.createElement('div');
    overlay.className = 'f4-overlay';
    overlay.innerHTML = `<div class="f4-dialog" role="dialog" aria-modal="true" aria-label="Field help"><div class="f4-title">Field Help <span>${esc(String(name).toUpperCase())}</span><button type="button" data-f4-close>×</button></div><pre class="f4-help-text">${esc(message)}</pre><div class="small-note">Help is evaluated by the local ABAPC runtime.</div></div>`;
    document.body.append(overlay);
    overlay.addEventListener('click', e => { if (e.target === overlay || e.target.closest('[data-f4-close]')) overlay.remove(); });
  };

  const bindF4Buttons = (root) => {
    root.querySelectorAll('[data-range-f4]').forEach(button => {
      if (button.dataset.bound === '1') return;
      button.dataset.bound = '1';
      button.addEventListener('click', () => {
        const row = button.closest('[data-range-row]');
        const low = row?.querySelector('[data-range-part="low"]');
        if (low) openValueHelp(row.closest('[data-range-host]')?.dataset.rangeName, low);
      });
    });
    root.querySelectorAll('[data-f4-param]').forEach(button => {
      if (button.dataset.bound === '1') return;
      button.dataset.bound = '1';
      button.addEventListener('click', () => {
        const input = [...document.querySelectorAll('[data-param-name]')].find(x => x.dataset.paramName?.toUpperCase() === button.dataset.f4Param.toUpperCase());
        if (input) openValueHelp(button.dataset.f4Param, input);
      });
    });
    root.querySelectorAll('[data-help-param]').forEach(button => {
      if (button.dataset.bound === '1') return;
      button.dataset.bound = '1';
      button.addEventListener('click', () => openFieldHelp(button.dataset.helpParam));
    });
  };

  // Auto-Save Toggle
  $('btn-autosave-toggle').addEventListener('click', () => {
    autoSaveEnabled = !autoSaveEnabled;
    try { localStorage.setItem('abapc_autosave_enabled', autoSaveEnabled ? '1' : '0'); } catch {}
    $('btn-autosave-toggle').textContent = autoSaveEnabled ? 'Auto-Save: ON' : 'Auto-Save: OFF';
    showToast(`Auto-save ${autoSaveEnabled ? 'enabled' : 'disabled'}`);
  });

  // Debounced Auto-Save & Manual Save
  let autoSaveTimer = null;
  $('gui-source').addEventListener('input', () => {
    resetSelectionScreen();
    updateGutter();

    if (autoSaveEnabled) {
      $('editor-save-status').className = 'editor-status-pill saving';
      $('editor-save-status').textContent = '● Saving...';
      clearTimeout(autoSaveTimer);
      autoSaveTimer = setTimeout(() => {
        const curVfs = getVFSFiles();
        curVfs[currentFile] = $('gui-source').value;
        saveVFSFiles(curVfs);
        $('editor-save-status').className = 'editor-status-pill saved';
        $('editor-save-status').textContent = '✓ Saved';
        setStatus(`Auto-saved ${currentFile}`);
      }, 1500);
    } else {
      $('editor-save-status').className = 'editor-status-pill unsaved';
      $('editor-save-status').textContent = '● Unsaved changes';
    }
  });

  // Tab & Keyboard Indentation
  $('gui-source').addEventListener('keydown', (e) => {
    if (e.key === 'Tab') {
      e.preventDefault();
      const el = e.target;
      const start = el.selectionStart;
      const end = el.selectionEnd;
      if (e.shiftKey) {
        const lineStart = el.value.lastIndexOf('\n', start - 1) + 1;
        if (el.value.slice(lineStart, lineStart + 2) === '  ') {
          el.value = el.value.slice(0, lineStart) + el.value.slice(lineStart + 2);
          el.selectionStart = el.selectionEnd = Math.max(lineStart, start - 2);
        }
      } else {
        el.value = el.value.substring(0, start) + '  ' + el.value.substring(end);
        el.selectionStart = el.selectionEnd = start + 2;
      }
      el.dispatchEvent(new Event('input', { bubbles: true }));
    } else if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 's') {
      e.preventDefault();
      $('gui-vfs-save')?.click();
    } else if (e.key === 'F8') {
      e.preventDefault();
      $('gui-run')?.click();
    } else if ((e.ctrlKey || e.metaKey) && e.key === 'F2') {
      e.preventDefault();
      $('gui-build')?.click();
    } else if (e.shiftKey && e.key === 'F1') {
      e.preventDefault();
      $('gui-pretty')?.click();
    }
  });

  // Save Button
  $('gui-vfs-save').addEventListener('click', () => {
    const curVfs = getVFSFiles();
    curVfs[currentFile] = $('gui-source').value;
    saveVFSFiles(curVfs);
    $('editor-save-status').className = 'editor-status-pill saved';
    $('editor-save-status').textContent = '✓ Saved';
    showToast(`Saved ${currentFile} to Virtual File System`);
    setStatus(`VFS: ${currentFile} saved`);
  });

  // Pretty Printer (Shift+F1)
  $('gui-pretty').addEventListener('click', () => {
    const formatted = prettyPrintAbap($('gui-source').value);
    $('gui-source').value = formatted;
    updateGutter();
    const curVfs = getVFSFiles();
    curVfs[currentFile] = formatted;
    saveVFSFiles(curVfs);
    $('editor-save-status').className = 'editor-status-pill saved';
    $('editor-save-status').textContent = '✓ Saved';
    showToast('Pretty Printer: ABAP syntax formatted and saved');
  });

  // Output Tabs Rendering Function
  const renderOutputView = () => {
    const body = $('gui-output-body');
    if (!body) return;

    if (activeOutputTab === 'output') {
      if (!lastOutputLines.length) {
        body.innerHTML = '<div class="spool-line" style="color:#64748b;">Ready. Select Execute (F8) to run this program in the ephemeral ABAPC VM.</div>';
        return;
      }
      body.innerHTML = lastOutputLines.map(line => {
        if (line.startsWith('---') || line.startsWith('===')) {
          return `<div class="spool-line" style="color:#64748b;font-weight:bold;">${esc(line)}</div>`;
        }
        if (line.includes('SUCCESS') || line.includes('Verified') || line.includes('Clean')) {
          return `<div class="spool-line spool-ok">${esc(line)}</div>`;
        }
        if (line.includes('FAILED') || line.includes('Error') || line.includes('error') || line.includes('Division by zero')) {
          return `<div class="spool-line spool-err">${esc(line)}</div>`;
        }
        return `<div class="spool-line">${esc(line)}</div>`;
      }).join('');
    } else if (activeOutputTab === 'diagnostics') {
      if (!lastDiagnosticsList.length) {
        body.innerHTML = '<div class="spool-line spool-ok">✓ Syntax check clean. No errors or warnings found.</div>';
        return;
      }
      body.innerHTML = '<div class="output-diagnostics-view">' + lastDiagnosticsList.map(d =>
        `<div class="diag-card-item" style="border-left:3px solid ${d.severity === 'warning' ? '#f59e0b' : '#ef4444'};"><div style="font-weight:bold;color:${d.severity === 'warning' ? '#fbbf24' : '#f87171'};font-size:11px;">${esc(d.severity || 'error').toUpperCase()} · ${esc(d.code || 'COMPILER')}${d.start ? ` [Line ${esc(d.start.line)}, Col ${esc(d.start.column)}]` : ''}</div><div style="color:#e2e8f0;margin-top:3px;font-size:12px;">${esc(d.message || '')}</div></div>`
      ).join('') + '</div>';
    } else if (activeOutputTab === 'variables') {
      const vars = Object.entries(lastVariablesMap);
      if (!vars.length) {
        body.innerHTML = '<div class="spool-line" style="color:#64748b;">No variables captured in this run.</div>';
        return;
      }
      body.innerHTML = `
        <table style="width:100%;border-collapse:collapse;font-size:11px;">
          <thead>
            <tr style="border-bottom:1px solid #334155;color:#94a3b8;text-align:left;">
              <th style="padding:4px 6px;">Variable</th>
              <th style="padding:4px 6px;">Value</th>
            </tr>
          </thead>
          <tbody>
            ${vars.map(([k, v]) => `
              <tr style="border-bottom:1px solid #1e293b;">
                <td style="padding:4px 6px;color:#38bdf8;font-weight:bold;">${esc(k)}</td>
                <td style="padding:4px 6px;color:#f1f5f9;">${esc(JSON.stringify(v))}</td>
              </tr>
            `).join('')}
          </tbody>
        </table>
      `;
    } else if (activeOutputTab === 'ir') {
      if (!lastIrData) {
        body.innerHTML = '<div class="spool-line" style="color:#64748b;">No IR bytecode generated yet. Execute or Check the program.</div>';
        return;
      }
      body.innerHTML = `<pre style="margin:0;color:#94a3b8;font-size:11px;">${esc(typeof lastIrData === 'string' ? lastIrData : JSON.stringify(lastIrData, null, 2))}</pre>`;
    }
  };

  // Output Tab Switching
  document.querySelectorAll('[data-output-tab]').forEach(tab => {
    tab.addEventListener('click', () => {
      document.querySelectorAll('[data-output-tab]').forEach(t => t.classList.remove('active'));
      tab.classList.add('active');
      activeOutputTab = tab.dataset.outputTab;
      renderOutputView();
    });
  });

  // Copy Output
  $('btn-copy-output').addEventListener('click', async () => {
    const body = $('gui-output-body');
    if (!body) return;
    try {
      await navigator.clipboard.writeText(body.innerText || '');
      showToast('Copied output to clipboard');
    } catch {
      showToast('Failed to copy output');
    }
  });

  // Clear Output
  $('btn-clear-output').addEventListener('click', () => {
    lastOutputLines = [];
    lastDiagnosticsList = [];
    lastVariablesMap = {};
    $('output-diag-count').textContent = '0';
    renderOutputView();
    showToast('Console output cleared');
  });

  // Check / Compile (Ctrl+F2)
  $('gui-build').addEventListener('click', async () => {
    setStatus('Checking syntax…');
    const body = { source: $('gui-source').value, mode: $('gui-mode').value, filename: currentFile };
    try {
      const d = await jsonPost('/api/compile', body);
      lastDiagnosticsList = d.diagnostics || [];
      lastIrData = d.ir || null;
      $('output-diag-count').textContent = String(lastDiagnosticsList.length);
      const errs = d.stats?.errors || 0;

      const statusPill = $('output-status-pill');
      if (statusPill) {
        statusPill.className = `output-metric-pill ${errs ? 'failed' : 'success'}`;
        statusPill.textContent = errs ? 'CHECK FAILED' : 'CHECK CLEAN';
      }

      if (errs > 0) {
        activeOutputTab = 'diagnostics';
        document.querySelectorAll('[data-output-tab]').forEach(t => t.classList.toggle('active', t.dataset.outputTab === 'diagnostics'));
      } else {
        lastOutputLines = ['[Syntax Check] Clean. No errors or warnings found.'];
        activeOutputTab = 'output';
        document.querySelectorAll('[data-output-tab]').forEach(t => t.classList.toggle('active', t.dataset.outputTab === 'output'));
      }
      renderOutputView();
      setStatus(errs ? 'Syntax check failed' : 'Syntax check passed');
    } catch (e) {
      lastOutputLines = [`[Syntax Check Error] ${String(e)}`];
      renderOutputView();
      setStatus('Syntax check error');
    }
  });

  // Execute (F8 / POST /api/run)
  $('gui-run').addEventListener('click', async () => {
    const runBtn = $('gui-run');
    runBtn.disabled = true;
    setStatus('Executing in ephemeral VM…');
    const statusPill = $('output-status-pill');
    if (statusPill) {
      statusPill.className = 'output-metric-pill info';
      statusPill.textContent = 'RUNNING…';
    }

    try {
      const sourceNow = $('gui-source').value;
      const modeNow = $('gui-mode').value;
      const curVfs = getVFSFiles();
      curVfs[currentFile] = sourceNow;
      saveVFSFiles(curVfs);

      if (parameterSource !== sourceNow || parameterMode !== modeNow) {
        parameterSpecs = null;
        parameterSource = null;
        parameterMode = null;
      }

      if (!parameterSpecs) {
        const checked = await jsonPost('/api/compile', { source: sourceNow, mode: modeNow, filename: currentFile });
        lastDiagnosticsList = checked.diagnostics || [];
        lastIrData = checked.ir || null;
        $('output-diag-count').textContent = String(lastDiagnosticsList.length);

        if ((checked.stats?.errors || 0) > 0) {
          if (statusPill) {
            statusPill.className = 'output-metric-pill failed';
            statusPill.textContent = 'COMPILATION ERROR';
          }
          lastOutputLines = [`Compilation failed (${checked.stats.errors} error(s)). Check Diagnostics tab.`];
          activeOutputTab = 'diagnostics';
          document.querySelectorAll('[data-output-tab]').forEach(t => t.classList.toggle('active', t.dataset.outputTab === 'diagnostics'));
          renderOutputView();
          setStatus('Compilation failed');
          return;
        }

        const detectedBase = (checked.ir?.vars || []).filter(v => v.parameter);
        if (detectedBase.length) {
          let defaults = {};
          try {
            const preview = await jsonPost('/api/selection-preview', { source: sourceNow, mode: modeNow, filename: currentFile });
            if (preview.ok) defaults = preview.variables || {};
          } catch {}
          const detected = detectedBase.map(v => {
            const key = String(v.name || '').toUpperCase();
            return Object.prototype.hasOwnProperty.call(defaults, key) ? { ...v, initialValue: defaults[key] } : v;
          });
          parameterSpecs = detected;
          parameterSource = sourceNow;
          parameterMode = modeNow;
          drawSelectionScreen(detected);
          runBtn.textContent = '▶ Run with inputs';
          lastOutputLines = [`Selection screen generated: ${detected.length} parameter(s). Enter values above, then select Run with inputs.`];
          activeOutputTab = 'output';
          document.querySelectorAll('[data-output-tab]').forEach(t => t.classList.toggle('active', t.dataset.outputTab === 'output'));
          renderOutputView();
          setStatus('Selection screen ready · enter report parameters');
          return;
        }
      }

      const parameterValues = {};
      if (parameterSpecs?.length) {
        const radioGroups = new Map();
        for (const v of parameterSpecs) {
          if (v.noDisplay) continue;
          if (v.selectionOptions) {
            const host = [...document.querySelectorAll('[data-range-host]')].find(x => x.dataset.rangeName?.toUpperCase() === String(v.name).toUpperCase());
            if (!host) { parameterValues[v.name] = []; continue; }
            const rows = [...host.querySelectorAll('[data-range-row]')].map(row => ({
              sign: row.querySelector('[data-range-part="sign"]')?.value || 'I',
              option: row.querySelector('[data-range-part="option"]')?.value || 'EQ',
              low: row.querySelector('[data-range-part="low"]')?.value || '',
              high: v.noIntervals ? '' : (row.querySelector('[data-range-part="high"]')?.value || '')
            })).filter(row => row.low.trim() !== '' || row.high.trim() !== '');
            if (v.obligatory && !rows.length) {
              host.querySelector('[data-range-part="low"]')?.focus();
              alert(`At least one range is required for ${v.name}.`);
              setStatus(`Missing required selection ${v.name}`);
              return;
            }
            parameterValues[v.name] = rows;
            continue;
          }
          const el = [...document.querySelectorAll('[data-param-name]')].find(x => x.dataset.paramName?.toUpperCase() === String(v.name).toUpperCase());
          if (!el) continue;
          if (v.radiobuttonGroup) {
            const group = String(v.radiobuttonGroup).toUpperCase();
            if (!radioGroups.has(group)) radioGroups.set(group, []);
            radioGroups.get(group).push({ v, el });
            continue;
          }
          const value = el.type === 'checkbox' ? (el.checked ? 'X' : '') : el.value;
          if (v.obligatory && String(value).trim() === '') {
            el.focus();
            alert(`Required parameter ${v.name} is empty.`);
            setStatus(`Missing required parameter ${v.name}`);
            return;
          }
          parameterValues[v.name] = value;
        }
        for (const [group, entries] of radioGroups) {
          const chosen = entries.find(x => x.el.checked);
          for (const entry of entries) parameterValues[entry.v.name] = chosen && chosen.v.name === entry.v.name ? 'X' : '';
          if (entries.some(x => x.v.obligatory) && !chosen) {
            entries[0]?.el.focus();
            alert(`Choose one option in radio-button group ${group}.`);
            setStatus(`Radio-button group ${group} requires a selection`);
            return;
          }
        }
      }

      const dbUrl = getByodbUrl();
      const d = await jsonPost('/api/run', {
        code: sourceNow,
        mode: modeNow,
        filename: currentFile,
        db_url: dbUrl.trim() || null,
        parameterValues
      });

      lastDiagnosticsList = d.diagnostics || [];
      lastOutputLines = d.output || [];
      lastVariablesMap = d.variables || {};
      $('output-diag-count').textContent = String(lastDiagnosticsList.length);

      if (statusPill) {
        statusPill.className = `output-metric-pill ${d.ok ? 'success' : 'failed'}`;
        statusPill.textContent = d.ok ? 'SUCCESS (sy-subrc = 0)' : 'FAILED';
      }

      $('output-meta-time').textContent = `Time: ${d.runtime?.elapsed_ms ?? 0} ms`;
      $('output-meta-steps').textContent = `Steps: ${d.runtime?.steps ?? 0}`;
      $('output-meta-db').textContent = `DB: ${d.database_type || 'Local Mock'}`;

      activeOutputTab = 'output';
      document.querySelectorAll('[data-output-tab]').forEach(t => t.classList.toggle('active', t.dataset.outputTab === 'output'));
      renderOutputView();

      setStatus(d.ok ? `Execution finished (${d.runtime?.elapsed_ms ?? 0} ms)` : 'Execution failed');
    } catch (e) {
      if (statusPill) {
        statusPill.className = 'output-metric-pill failed';
        statusPill.textContent = 'RUN ERROR';
      }
      lastOutputLines = [`Execution error: ${String(e)}`];
      activeOutputTab = 'output';
      renderOutputView();
      setStatus('Execution failed');
    } finally {
      runBtn.disabled = false;
    }
  });

  // BYODB Settings Modal (Bring Your Own Database)
  $('gui-byodb')?.addEventListener('click', () => {
    openByodbModal();
  });

  // Draggable Horizontal Resizer (VFS Pane width)
  const vfsPane = $('sap-vfs-pane');
  const hResizer = $('sap-horizontal-resizer');
  if (vfsPane && hResizer) {
    const savedVfsWidth = localStorage.getItem('abapc_vfs_width');
    if (savedVfsWidth) vfsPane.style.width = savedVfsWidth + 'px';
    let isDraggingH = false;
    let startX = 0;
    let startW = 0;
    hResizer.addEventListener('mousedown', (e) => {
      isDraggingH = true;
      startX = e.clientX;
      startW = vfsPane.offsetWidth;
      hResizer.classList.add('dragging');
      document.body.style.cursor = 'col-resize';
      document.body.style.userSelect = 'none';
    });
    window.addEventListener('mousemove', (e) => {
      if (!isDraggingH) return;
      const newW = Math.max(160, Math.min(500, startW + (e.clientX - startX)));
      vfsPane.style.width = newW + 'px';
      try { localStorage.setItem('abapc_vfs_width', newW); } catch {}
    });
    window.addEventListener('mouseup', () => {
      if (isDraggingH) {
        isDraggingH = false;
        hResizer.classList.remove('dragging');
        document.body.style.cursor = '';
        document.body.style.userSelect = '';
      }
    });
  }

  // Draggable Vertical Resizer (Editor vs Output height)
  const editorTop = $('sap-editor-top');
  const outputBottom = $('sap-output-bottom');
  const vResizer = $('sap-vertical-resizer');
  if (editorTop && outputBottom && vResizer) {
    const savedEditorH = localStorage.getItem('abapc_editor_height');
    if (savedEditorH) {
      editorTop.style.flex = 'none';
      editorTop.style.height = savedEditorH + 'px';
      outputBottom.style.flex = '1';
    }
    let isDraggingV = false;
    let startY = 0;
    let startH = 0;
    vResizer.addEventListener('mousedown', (e) => {
      isDraggingV = true;
      startY = e.clientY;
      startH = editorTop.offsetHeight;
      vResizer.classList.add('dragging');
      document.body.style.cursor = 'row-resize';
      document.body.style.userSelect = 'none';
    });
    window.addEventListener('mousemove', (e) => {
      if (!isDraggingV) return;
      const newH = Math.max(140, Math.min(window.innerHeight - 180, startH + (e.clientY - startY)));
      editorTop.style.flex = 'none';
      editorTop.style.height = newH + 'px';
      outputBottom.style.flex = '1';
      try { localStorage.setItem('abapc_editor_height', newH); } catch {}
    });
    window.addEventListener('mouseup', () => {
      if (isDraggingV) {
        isDraggingV = false;
        vResizer.classList.remove('dragging');
        document.body.style.cursor = '';
        document.body.style.userSelect = '';
      }
    });
  }

  // Draggable Parameters Resizer (Selection Screen height)
  const paramsHost = $('gui-parameters');
  const paramsResizer = $('sap-params-resizer');
  if (paramsHost && paramsResizer) {
    let isDraggingP = false;
    let startYP = 0;
    let startHP = 0;
    paramsResizer.addEventListener('mousedown', (e) => {
      isDraggingP = true;
      startYP = e.clientY;
      startHP = paramsHost.offsetHeight;
      paramsResizer.classList.add('dragging');
      document.body.style.cursor = 'row-resize';
      document.body.style.userSelect = 'none';
    });
    window.addEventListener('mousemove', (e) => {
      if (!isDraggingP) return;
      const maxH = Math.max(160, Math.floor(window.innerHeight * 0.65));
      const newH = Math.max(50, Math.min(maxH, startHP + (e.clientY - startYP)));
      paramsHost.style.height = newH + 'px';
      try { localStorage.setItem('abapc_params_height', newH); } catch {}
    });
    window.addEventListener('mouseup', () => {
      if (isDraggingP) {
        isDraggingP = false;
        paramsResizer.classList.remove('dragging');
        document.body.style.cursor = '';
        document.body.style.userSelect = '';
      }
    });

    if (window.ResizeObserver) {
      const ro = new ResizeObserver((entries) => {
        for (const entry of entries) {
          if (entry.target === paramsHost && paramsHost.style.display !== 'none' && !isDraggingP) {
            const h = Math.round(entry.contentRect.height);
            if (h > 40) {
              try { localStorage.setItem('abapc_params_height', h); } catch {}
            }
          }
        }
      });
      ro.observe(paramsHost);
    }
  }

  // Collapsible VFS Pane
  const btnVfsCollapse = $('btn-toggle-vfs-collapse');
  const vfsRail = $('vfs-collapsed-rail');
  const toggleVfs = (collapse) => {
    if (!vfsPane) return;
    const isCol = collapse !== undefined ? collapse : !vfsPane.classList.contains('collapsed');
    vfsPane.classList.toggle('collapsed', isCol);
    if (hResizer) hResizer.style.display = isCol ? 'none' : 'block';
    if (vfsRail) vfsRail.style.display = isCol ? 'flex' : 'none';
    try { localStorage.setItem('abapc_vfs_collapsed', isCol ? '1' : '0'); } catch {}
  };
  btnVfsCollapse?.addEventListener('click', (e) => { e.stopPropagation(); toggleVfs(true); });
  vfsRail?.addEventListener('click', () => toggleVfs(false));
  if (localStorage.getItem('abapc_vfs_collapsed') === '1') toggleVfs(true);

  // Collapsible Output Console
  const btnOutputCollapse = $('btn-toggle-output-collapse');
  const toggleOutput = (collapse) => {
    if (!outputBottom) return;
    const isCol = collapse !== undefined ? collapse : !outputBottom.classList.contains('collapsed');
    outputBottom.classList.toggle('collapsed', isCol);
    if (btnOutputCollapse) btnOutputCollapse.textContent = isCol ? '▲ Output' : '▼ Output';
    if (isCol) {
      editorTop.style.flex = '1';
      editorTop.style.height = 'auto';
    } else {
      const savedH = localStorage.getItem('abapc_editor_height');
      if (savedH) {
        editorTop.style.flex = 'none';
        editorTop.style.height = savedH + 'px';
        outputBottom.style.flex = '1';
      }
    }
    try { localStorage.setItem('abapc_output_collapsed', isCol ? '1' : '0'); } catch {}
  };
  btnOutputCollapse?.addEventListener('click', () => toggleOutput());
  if (localStorage.getItem('abapc_output_collapsed') === '1') toggleOutput(true);
}

function openByodbModal() {
  const existing = document.getElementById('byodb-modal-overlay');
  if (existing) existing.remove();

  const currentUrl = getByodbUrl();
  const overlay = document.createElement('div');
  overlay.id = 'byodb-modal-overlay';
  overlay.className = 'f4-overlay';
  overlay.innerHTML = `
    <div class="f4-dialog" style="width:min(580px, 96vw);">
      <div class="f4-title">
        <span>🗄 Bring Your Own Database (BYODB) · PostgreSQL</span>
        <button id="byodb-close-btn" title="Close">✕</button>
      </div>
      <div style="font-size:12px;color:#334155;margin-bottom:12px;line-height:1.5;">
        Connect your ephemeral ABAPC VM to a live PostgreSQL instance (e.g. Localhost, Supabase, or Neon).
        When configured, Open SQL statements (<code style="background:#e2e8f0;padding:1px 4px;border-radius:3px;">SELECT ... FROM ...</code>) will query your live database.
      </div>

      <div style="margin-bottom:10px;">
        <label for="byodb-input-url" style="display:block;font-size:11px;font-weight:bold;color:#1e293b;margin-bottom:4px;">
          PostgreSQL Connection String
        </label>
        <div style="display:flex;gap:6px;">
          <input type="text" id="byodb-input-url" value="${esc(currentUrl)}" placeholder="postgres://postgres:password@localhost:5432/postgres" style="flex:1;height:30px;padding:4px 8px;font-family:monospace;font-size:11px;border:1px solid #94a3b8;" />
          <button class="secondary-btn" id="byodb-clear-btn" title="Clear and use Local Mock DB" style="font-size:11px;padding:2px 8px;">Clear</button>
        </div>
        <div style="font-size:10px;color:#64748b;margin-top:4px;">
          Format: <code style="color:#0284c7;">postgres://username:password@host:port/database</code>
        </div>
      </div>

      <div id="byodb-test-status" style="margin-bottom:12px;padding:8px 10px;border-radius:4px;font-size:11px;display:none;"></div>

      <div style="background:#f1f5f9;border:1px solid #cbd5e1;border-radius:4px;padding:8px 10px;font-size:11px;color:#475569;margin-bottom:14px;line-height:1.4;">
        <b>Localhost Troubleshooting Guide:</b>
        <ul style="margin:4px 0 0 16px;padding:0;">
          <li>If testing on <b>localhost:5432</b>, ensure your PostgreSQL Windows service is running (open <code>services.msc</code> &rarr; check <code>postgresql-x64-XX</code> status is <i>Running</i>).</li>
          <li>If PostgreSQL is installed on a different port (e.g. <code>5433</code>), specify <code>localhost:5433</code>.</li>
          <li>For instant cloud testing without local setup, you can paste a free connection string from <a href="https://neon.tech" target="_blank" style="color:#0284c7;text-decoration:underline;">Neon.tech</a> or <a href="https://supabase.com" target="_blank" style="color:#0284c7;text-decoration:underline;">Supabase</a>.</li>
          <li>If the field is empty or connection fails, the compiler automatically uses the built-in deterministic Mock tables (<code>SCARR</code>, <code>SPFLI</code>, <code>SFLIGHT</code>, <code>ZPRODUCT</code>).</li>
        </ul>
      </div>

      <div style="display:flex;justify-content:flex-end;gap:8px;">
        <button class="secondary-btn" id="byodb-test-btn" style="min-width:110px;">🔌 Test Connection</button>
        <button class="primary-btn" id="byodb-save-btn" style="min-width:80px;">💾 Save</button>
      </div>
    </div>
  `;

  document.body.appendChild(overlay);

  const close = () => overlay.remove();
  $('byodb-close-btn').addEventListener('click', close);
  overlay.addEventListener('click', (e) => {
    if (e.target === overlay) close();
  });

  $('byodb-clear-btn').addEventListener('click', () => {
    $('byodb-input-url').value = '';
    localStorage.removeItem('abapc_byodb_url');
    const statusBox = $('byodb-test-status');
    statusBox.style.display = 'block';
    statusBox.style.background = '#e0f2fe';
    statusBox.style.color = '#0369a1';
    statusBox.style.border = '1px solid #7dd3fc';
    statusBox.textContent = 'Cleared. System will use the built-in Local Mock Database.';
    showToast('Database URL cleared');
  });

  $('byodb-save-btn').addEventListener('click', () => {
    const val = $('byodb-input-url').value.trim();
    if (val) {
      localStorage.setItem('abapc_byodb_url', val);
      showToast('Database URL saved');
    } else {
      localStorage.removeItem('abapc_byodb_url');
      showToast('Using Local Mock Database');
    }
    close();
  });

  $('byodb-test-btn').addEventListener('click', async () => {
    const testBtn = $('byodb-test-btn');
    const statusBox = $('byodb-test-status');
    const url = $('byodb-input-url').value.trim();

    if (!url) {
      statusBox.style.display = 'block';
      statusBox.style.background = '#fef3c7';
      statusBox.style.color = '#92400e';
      statusBox.style.border = '1px solid #fcd34d';
      statusBox.textContent = 'No database URL entered. Local mock fixtures (SCARR, SPFLI, ZPRODUCT) are active.';
      return;
    }

    testBtn.disabled = true;
    testBtn.textContent = 'Testing…';
    statusBox.style.display = 'block';
    statusBox.style.background = '#f8fafc';
    statusBox.style.color = '#475569';
    statusBox.style.border = '1px solid #cbd5e1';
    statusBox.textContent = 'Connecting to ' + url.replace(/:[^:@]+@/, ':****@') + '…';

    try {
      const resp = await jsonPost('/api/run', {
        code: 'REPORT z_db_test. WRITE: / \'DB connection ping\'.',
        db_url: url,
        filename: 'z_db_test.abap'
      });

      if (resp.database_connected) {
        statusBox.style.background = '#dcfce7';
        statusBox.style.color = '#166534';
        statusBox.style.border = '1px solid #86efac';
        statusBox.innerHTML = `<b>✓ Connection Successful!</b><br>Engine: ${esc(resp.database_type || 'PostgreSQL (BYODB)')}. Open SQL queries will now execute against your database.`;
        localStorage.setItem('abapc_byodb_url', url);
      } else {
        const diag = (resp.diagnostics || []).find(d => d.code === 'DB001');
        const reason = diag ? diag.message : (resp.database_type || 'Connection failed');
        statusBox.style.background = '#fee2e2';
        statusBox.style.color = '#991b1b';
        statusBox.style.border = '1px solid #fca5a5';
        statusBox.innerHTML = `<b>✕ Connection Failed</b><br>${esc(reason)}<br><small style="margin-top:4px;display:block;">If testing localhost, make sure your PostgreSQL service is started and listening on the specified port.</small>`;
      }
    } catch (e) {
      statusBox.style.background = '#fee2e2';
      statusBox.style.color = '#991b1b';
      statusBox.style.border = '1px solid #fca5a5';
      statusBox.innerHTML = `<b>✕ Request Failed</b>: ${esc(e.message || String(e))}`;
    } finally {
      testBtn.disabled = false;
      testBtn.textContent = '🔌 Test Connection';
    }
  });
}
function showGuiOutput(type,text){const el=$('gui-output');if(!el)return;el.className='console';el.innerHTML=`<span class="${type}">${esc(text)}</span>`;}
function renderGuiDiagnostics(diags){const el=$('gui-diags');if(!el)return;if(!diags.length){el.innerHTML='<div class="small-note">No compiler diagnostics.</div>';return;}el.innerHTML=diags.map(d=>`<div class="diag-line"><b>${esc(d.severity||'error')} · ${esc(d.code||'COMPILER')}</b> ${d.start?`L${esc(d.start.line)}:${esc(d.start.column)}`:''}<div>${esc(d.message||'')}</div></div>`).join('');}
async function getTables(){if(Object.keys(state.tables).length)return state.tables;const d=await jsonGet('/api/gui/tables');state.tables=d.tables||{};return state.tables;}
async function renderTableBrowser(){
  $('screen').innerHTML='<div class="empty-state">Loading local fixture tables…</div>';
  let tables={};try{tables=await getTables();}catch(e){$('screen').innerHTML=`<div class="notice error">${esc(e.message)}</div>`;return;}
  const names=Object.keys(tables);const current=names.includes(window.__abapcTable)?window.__abapcTable:(names[0]||'');
  $('screen').innerHTML=`<div class="toolbar-row"><label for="table-name">Table</label><select id="table-name">${names.map(n=>`<option ${n===current?'selected':''}>${esc(n)}</option>`).join('')}</select><label for="table-filter">Filter</label><input id="table-filter" placeholder="Search values or FIELD=value"/><button class="secondary-btn" id="apply-filter">Apply filter</button><button class="secondary-btn" id="clear-filter">Clear</button></div><div class="small-note" id="table-info">Read-only deterministic demo data. No connection to HANA or SAP DDIC.</div><div id="table-results" style="margin-top:12px"></div>`;
  const draw=()=>{const name=$('table-name').value;window.__abapcTable=name;const rows=tables[name]||[];const q=$('table-filter').value.trim().toLowerCase();let filtered=rows;
    const equal=q.match(/^([a-z0-9_]+)\s*=\s*(.*)$/i);if(equal){const key=equal[1].toUpperCase(),val=equal[2].replace(/^['"]|['"]$/g,'');filtered=rows.filter(r=>String(r[key]??'').toLowerCase()===val.toLowerCase());}else if(q){filtered=rows.filter(r=>Object.values(r).some(v=>String(v).toLowerCase().includes(q)));}
    $('table-info').textContent=`${tableMeta[name]?.description||'Local fixture'} · ${filtered.length} of ${rows.length} row(s) · read-only`;
    const cols=[...new Set(rows.flatMap(r=>Object.keys(r)))];if(!cols.length){$('table-results').innerHTML='<div class="empty-state">No rows available.</div>';return;}
    $('table-results').innerHTML=`<div class="table-wrap"><table class="data-table"><thead><tr>${cols.map(c=>`<th>${esc(c)}</th>`).join('')}</tr></thead><tbody>${filtered.map(r=>`<tr>${cols.map(c=>`<td>${esc(r[c]??'')}</td>`).join('')}</tr>`).join('')}</tbody></table></div>`;
  };
  $('table-name').addEventListener('change',draw);$('apply-filter').addEventListener('click',draw);$('table-filter').addEventListener('keydown',e=>{if(e.key==='Enter')draw();});$('clear-filter').addEventListener('click',()=>{$('table-filter').value='';draw();});draw();
}
async function renderDictionary(){
  let catalog;try{catalog=await jsonGet('/api/platform/catalog');}catch(e){$('screen').innerHTML=`<div class="notice error">${esc(e.message)}</div>`;return;}
  const objects=catalog.objects||[];$('screen').innerHTML=`<div class="notice info">SE11-style local repository explorer. You can create Z/Y training objects and use local structures as compiler types. These are persisted in ABAPC's repository model, not SAP DDIC objects.</div><div class="panel"><div class="panel-title">Create local DDIC training object</div><div class="panel-body"><div class="toolbar-row"><select id="ddic-new-kind"><option value="TABL">TABL · Table</option><option value="STRU">STRU · Structure</option><option value="DTEL">DTEL · Data element</option><option value="DOMA">DOMA · Domain</option></select><input id="ddic-new-name" placeholder="ZMY_OBJECT"/><input id="ddic-new-description" placeholder="Description (optional)"/><button class="primary-btn" id="ddic-create-object">Create object</button></div><div class="toolbar-row" id="ddic-base-row" hidden><label for="ddic-new-base">Base type</label><input id="ddic-new-base" placeholder="CHAR(20), I, DEC(13,2), STRING"/></div><div id="ddic-fields-row"><label for="ddic-new-fields">Fields JSON · use key:true for at least one table key</label><textarea id="ddic-new-fields" class="code-editor" rows="5" spellcheck="false">[\n  { "name": "ID", "type": "I", "key": true, "description": "Primary key" },\n  { "name": "NAME", "type": "CHAR(40)", "description": "Description" }\n]</textarea></div><div class="small-note" id="ddic-create-status">Local training object creation · names must begin with Z or Y.</div></div></div><div class="toolbar-row"><label for="ddic-object">Repository object</label><select id="ddic-object">${objects.map(o=>`<option value="${esc(o.name)}">${esc(o.name)} · ${esc(o.kind)}</option>`).join('')}</select><span class="small-note">${objects.length} local objects</span></div><div id="ddic-detail"></div>`;
  const draw=async()=>{const n=$('ddic-object').value;try{const o=await jsonGet('/api/platform/object?name='+encodeURIComponent(n));let rowCount='—';if(o.kind==='TABL'){const t=await getTables();rowCount=(t[n]||[]).length;}const fields=o.fields||[];$('ddic-detail').innerHTML=`<div class="panel"><div class="panel-title">${esc(o.name)} <span class="muted">· ${esc(o.kind)} · ${esc(o.active===false?'Inactive':'Active local object')}</span></div><div class="panel-body"><div class="metric-row"><div class="metric"><strong>${esc(o.kind)}</strong><span>Object type</span></div><div class="metric"><strong>${esc(fields.length)}</strong><span>Fields</span></div><div class="metric"><strong>${esc(rowCount)}</strong><span>Local sample rows</span></div></div><p>${esc(o.description||'')}</p>${o.source?`<p class="small-note">Source object: <b>${esc(o.source)}</b></p>`:''}${fields.length?`<div class="field-list"><div class="field-head">Field</div><div class="field-head">Type</div><div class="field-head">Description / key</div>${fields.map(f=>`<div class="mono">${esc(f.name)}</div><div>${esc(f.type)}</div><div>${esc(f.description||'')}${f.key?' · KEY':''}</div>`).join('')}</div>`:'<div class="empty-state">This local object has no field list in the training catalogue.</div>'}</div></div>`;}catch(e){$('ddic-detail').innerHTML=`<div class="notice error">${esc(e.message)}</div>`;}};
  $('ddic-object').addEventListener('change',draw);
  const updateCreateFields=()=>{const base=['DTEL','DOMA'].includes($('ddic-new-kind').value);$('ddic-base-row').hidden=!base;$('ddic-fields-row').hidden=base;};
  $('ddic-new-kind').addEventListener('change',updateCreateFields);updateCreateFields();
  $('ddic-create-object').addEventListener('click',async()=>{try{const kind=$('ddic-new-kind').value;let fields=[];if(['TABL','STRU'].includes(kind)){fields=JSON.parse($('ddic-new-fields').value||'[]');if(!Array.isArray(fields))throw new Error('Fields JSON must be an array.');}const result=await jsonPost('/api/platform/create-object',{name:$('ddic-new-name').value,kind,description:$('ddic-new-description').value,baseType:$('ddic-new-base').value,fields});showToast(result.reported);await renderDictionary();}catch(e){$('ddic-create-status').textContent=e.message;notice(esc(e.message),'error');}});
  draw();
}

async function renderCdsWorkbench(){
  const defaultSource=`@EndUserText.label: 'Carrier projection for ABAPC practice'
define view entity ZI_CARRIER_PRACTICE as select from SCARR {
  key CARRID as CarrierId,
      CARRNAME as CarrierName,
      CURRCODE as CurrencyCode
}`;
  $('screen').innerHTML=`<div class="notice info">Local CDS DDL subset: activate a projection view entity on a local DDIC table, then preview its rows. Associations, joins, expressions, annotations with parameters and SAP activation semantics are not implemented.</div><div class="panel"><div class="panel-title">CDS Data Definition</div><div class="panel-body"><div class="toolbar-row"><label for="cds-name">View entity</label><input id="cds-name" value="ZI_CARRIER_PRACTICE"/><button class="primary-btn" id="cds-activate">Activate locally</button></div><textarea id="cds-source" class="code-editor" style="min-height:230px" spellcheck="false">${esc(defaultSource)}</textarea><div class="toolbar-row"><button class="secondary-btn" id="cds-query">Preview rows</button><span id="cds-status" class="small-note">Not activated in this session yet.</span></div><div id="cds-result"></div></div></div><div class="panel"><div class="panel-title">Activated local CDS views</div><div class="panel-body" id="cds-list">Loading…</div></div>`;
  const refresh=async()=>{const d=await jsonGet('/api/cds/views');$('cds-list').innerHTML=d.views.length?`<div class="object-list">${d.views.map(v=>`<div class="object-row"><span>◫</span><span>${esc(v.name)}</span><small>${esc(v.source)} · ${v.fields.length} fields</small></div>`).join('')}</div>`:'<div class="empty-state">No custom CDS views activated yet.</div>';};
  $('cds-activate').addEventListener('click',async()=>{try{const d=await jsonPost('/api/cds/activate',{source:$('cds-source').value,replace:true});$('cds-name').value=d.view.name;$('cds-status').textContent=d.reported;await refresh();showToast(d.reported);}catch(e){$('cds-status').textContent=e.message;notice(esc(e.message),'error');}});
  $('cds-query').addEventListener('click',async()=>{try{const d=await jsonPost('/api/cds/query',{name:$('cds-name').value});if(!d.rows.length){$('cds-result').innerHTML='<div class="empty-state">No rows matched.</div>';return;}const cols=Object.keys(d.rows[0]);$('cds-result').innerHTML=`<div class="table-wrap"><table class="data-table"><thead><tr>${cols.map(c=>`<th>${esc(c)}</th>`).join('')}</tr></thead><tbody>${d.rows.map(r=>`<tr>${cols.map(c=>`<td>${esc(r[c])}</td>`).join('')}</tr>`).join('')}</tbody></table></div><p class="small-note">${d.rows.length} row(s) from local source table.</p>`;}catch(e){notice(esc(e.message),'error');}});
  await refresh();
}

async function renderRapWorkbench(){
  const sessionId=sessionStorage.getItem('abapc-rap-session')||`practice-${Math.random().toString(36).slice(2,12)}`;sessionStorage.setItem('abapc-rap-session',sessionId);
  $('screen').innerHTML=`<div class="notice info">Local RAP / EML practice service for ZI_PRODUCT. CREATE, UPDATE and DELETE are staged per browser session. COMMIT applies the batch atomically to ABAPC's local database; ROLLBACK discards pending operations. This is not the SAP RAP runtime.</div><div class="panel"><div class="panel-title">Entity Manipulation Language · Local Practice</div><div class="panel-body"><div class="toolbar-row"><label for="rap-entity">Entity</label><select id="rap-entity"><option>ZI_PRODUCT</option><option>ZC_PRODUCT</option></select><button class="primary-btn" id="rap-read">Read entities</button><button class="secondary-btn" id="rap-commit">Commit entities</button><button class="secondary-btn" id="rap-rollback">Rollback entities</button></div><div class="section-label">Modify entities</div><div class="sap-form-grid"><label for="rap-id">Product ID</label><input id="rap-id" value="P-400"/><label for="rap-name">Product name</label><input id="rap-name" value="USB-C Hub"/><label for="rap-category">Category</label><input id="rap-category" value="ACCESSORY"/><label for="rap-price">Price (INR)</label><input id="rap-price" type="number" min="0" value="2499"/><label for="rap-currency">Currency</label><input id="rap-currency" value="INR"/></div><div class="toolbar-row"><button class="secondary-btn" id="rap-create">Stage CREATE</button><button class="secondary-btn" id="rap-update">Stage UPDATE</button><button class="secondary-btn" id="rap-delete">Stage DELETE</button></div><div id="rap-message" class="small-note">Session ${esc(sessionId)} · no operation staged.</div></div></div><div class="panel"><div class="panel-title">Entity result / current transactional view</div><div class="panel-body" id="rap-rows"><div class="empty-state">Choose Read Entities to display products.</div></div></div><div class="panel"><div class="panel-title">Pending changes</div><div class="panel-body" id="rap-pending"><div class="empty-state">No pending changes.</div></div></div>`;
  const entity=()=>$('rap-entity').value;
  const displayRows=rows=>{if(!rows.length){$('rap-rows').innerHTML='<div class="empty-state">No products found.</div>';return;}const cols=Object.keys(rows[0]);$('rap-rows').innerHTML=`<div class="table-wrap"><table class="data-table"><thead><tr>${cols.map(c=>`<th>${esc(c)}</th>`).join('')}</tr></thead><tbody>${rows.map(r=>`<tr>${cols.map(c=>`<td>${esc(r[c])}</td>`).join('')}</tr>`).join('')}</tbody></table></div>`;};
  const displayPending=async()=>{const d=await jsonPost('/api/rap/read',{entity:entity(),sessionId});const p=d.pending||[];$('rap-pending').innerHTML=p.length?`<div class="object-list">${p.map((x,i)=>`<div class="object-row"><span>${i+1}.</span><span>${esc(x.kind)} · ${esc(x.key)}</span><small>${x.data?esc(JSON.stringify(x.data)):'—'}</small></div>`).join('')}</div>`:'<div class="empty-state">No pending changes.</div>';};
  const read=async()=>{const d=await jsonPost('/api/rap/read',{entity:entity(),sessionId});displayRows(d.rows);await displayPending();};
  const data=()=>({PRODUCT_ID:$('rap-id').value.trim(),PRODUCT_NAME:$('rap-name').value.trim(),CATEGORY:$('rap-category').value.trim(),PRICE:Number($('rap-price').value),CURRENCY:$('rap-currency').value.trim()});
  const stage=async operation=>{try{const d=await jsonPost('/api/rap/modify',{entity:entity(),sessionId,operation,data:data()});$('rap-message').textContent=d.reported;await displayPending();await read();}catch(e){$('rap-message').textContent=e.message;}};
  $('rap-read').addEventListener('click',()=>read().catch(e=>$('rap-message').textContent=e.message));
  $('rap-create').addEventListener('click',()=>stage('CREATE'));$('rap-update').addEventListener('click',()=>stage('UPDATE'));$('rap-delete').addEventListener('click',()=>stage('DELETE'));
  $('rap-commit').addEventListener('click',async()=>{try{const d=await jsonPost('/api/rap/commit',{sessionId});$('rap-message').textContent=d.reported;await read();}catch(e){$('rap-message').textContent=e.message;}});
  $('rap-rollback').addEventListener('click',async()=>{try{const d=await jsonPost('/api/rap/rollback',{sessionId});$('rap-message').textContent=d.reported;await read();}catch(e){$('rap-message').textContent=e.message;}});
  await read();
}
async function renderObjectNavigator(){
  const vfs = getVFSFiles();
  const vfsKeys = Object.keys(vfs);
  const names = state.examples.length ? state.examples : await loadExamples();
  $('screen').innerHTML = `
    <div class="two-col">
      <div class="panel">
        <div class="panel-title">
          <span>Virtual File System Repository (SE80)</span>
          <button class="secondary-btn small-btn" id="se80-new-vfs" title="Create a new program in the Virtual File System">+ New Program</button>
        </div>
        <div class="panel-body">
          <p class="small-note">Shared client-side Virtual File System. Click any program to open it directly in SE38 ABAP Editor.</p>
          <div class="object-list" id="vfs-object-list">
            ${vfsKeys.map(n => {
              const lines = (vfs[n] || '').split('\n').length;
              const bytes = (vfs[n] || '').length;
              return `<button class="object-row" data-vfs-file="${esc(n)}" title="Open ${esc(n)} in SE38">
                <span>📄</span>
                <span><b>${esc(n)}</b></span>
                <small>${lines} lines · ${bytes} B</small>
              </button>`;
            }).join('') || '<div class="empty-state">No virtual programs found.</div>'}
          </div>
        </div>
      </div>
      <div>
        <div class="panel">
          <div class="panel-title">Built-in Sample Programs</div>
          <div class="panel-body">
            <p class="small-note">Supplied reference templates for ABAP Cloud and Standard syntax.</p>
            <div class="object-list">
              ${names.map(n => `<button class="object-row" data-example="${esc(n)}"><span>▤</span><span>${esc(n)}</span><small>Reference source</small></button>`).join('')}
            </div>
          </div>
        </div>
        <div class="notice info" style="margin-top:12px">All programs in the Virtual File System are persisted in your browser's localStorage and shared in real-time between ABAPC Studio and SAP GUI SE38/SE80.</div>
      </div>
    </div>`;

  document.querySelectorAll('[data-vfs-file]').forEach(b => {
    b.addEventListener('click', () => {
      const file = b.dataset.vfsFile;
      setActiveVFSFileName(file);
      state.editorFilename = file;
      goTo('SE38', false);
      setStatus(`Opened VFS program ${file} in SE38`);
    });
  });

  document.querySelectorAll('[data-example]').forEach(b => {
    b.addEventListener('click', async () => {
      try {
        const d = await jsonGet('/api/example?name=' + encodeURIComponent(b.dataset.example));
        state.pendingSource = d.source;
        state.editorFilename = b.dataset.example;
        goTo('SE38', false);
        setStatus(`Opened sample ${b.dataset.example} in SE38`);
      } catch (e) {
        notice(e.message, 'error');
      }
    });
  });

  $('se80-new-vfs')?.addEventListener('click', () => {
    let name = prompt('Enter new ABAP program name (e.g. Z_MY_REPORT.abap):');
    if (!name || !name.trim()) return;
    name = name.trim();
    if (!name.endsWith('.abap')) name += '.abap';
    const curVfs = getVFSFiles();
    curVfs[name] = `REPORT ${name.replace('.abap', '').toUpperCase()}.\n\nWRITE: / 'Created in SE80 Object Navigator'.\n`;
    saveVFSFiles(curVfs);
    setActiveVFSFileName(name);
    state.editorFilename = name;
    goTo('SE38', false);
  });
}
async function renderExecutionArchive(){
  $('screen').innerHTML='<div class="empty-state">Loading archived programs…</div>';
  try{const d=await jsonGet('/api/executed');state.executions=d.files||[];}catch(e){$('screen').innerHTML=`<div class="notice error">${esc(e.message)}</div>`;return;}
  $('screen').innerHTML=`<div class="toolbar-row"><button class="secondary-btn" id="refresh-archive">↻ Refresh</button><span class="small-note">${state.executions.length} most recent archived source file(s). The archive stores source, not full SAP background-job metadata.</span></div><div class="table-wrap"><table class="data-table"><thead><tr><th>Source file</th><th>Modified</th><th>Size</th><th>Action</th></tr></thead><tbody>${state.executions.map((f,i)=>`<tr><td class="mono">${esc(f.name)}</td><td>${esc(new Date(f.modified).toLocaleString())}</td><td>${esc(f.size)} B</td><td><button class="link-button" data-archive="${i}">Open in editor</button></td></tr>`).join('')||'<tr><td colspan="4">No archived executions yet.</td></tr>'}</tbody></table></div>`;
  $('refresh-archive').addEventListener('click',()=>renderExecutionArchive());document.querySelectorAll('[data-archive]').forEach(b=>b.addEventListener('click',async()=>{const f=state.executions[Number(b.dataset.archive)];if(!f)return;try{const d=await jsonGet('/api/code?name='+encodeURIComponent(f.name));state.pendingSource=d.source;state.editorFilename=f.name;state.editorPreset='';goTo('SE38',false);setStatus(`Loaded archived source ${f.name}`);}catch(e){notice(esc(e.message),'error');}}));
}
function renderDiagnostics(){const list=state.lastDiagnostics||[];const output=state.lastOutput||[];$('screen').innerHTML=`<div class="notice info">This view shows diagnostics captured by the current browser session's last ABAPC execution. It is not SAP transaction ST22 and does not contain SAP short dumps.</div><div class="panel"><div class="panel-title">Last execution output</div><div class="panel-body"><div class="console">${esc(output.join('\n')||'No output captured in this browser session.')}</div></div></div><div class="section-label">Compiler/runtime diagnostics (${list.length})</div>${list.length?list.map(d=>`<div class="diag-line"><b>${esc(d.severity||'error')} · ${esc(d.code||'COMPILER')} ${d.start?`L${esc(d.start.line)}:${esc(d.start.column)}`:''}</b><div>${esc(d.message||'')}</div></div>`).join(''):'<div class="empty-state">No diagnostics recorded. Run a program from SE38 to populate this panel.</div>'}<div class="toolbar-row" style="margin-top:13px"><button class="secondary-btn" id="diag-refresh">Refresh session view</button><button class="secondary-btn" id="diag-clear">Clear session diagnostics</button></div>`;
  $('diag-refresh').addEventListener('click',()=>{state.lastDiagnostics=readSessionArray('abapc-last-diagnostics');state.lastOutput=readSessionArray('abapc-last-output');renderDiagnostics();});$('diag-clear').addEventListener('click',()=>{state.lastDiagnostics=[];state.lastOutput=[];sessionStorage.removeItem('abapc-last-diagnostics');sessionStorage.removeItem('abapc-last-output');renderDiagnostics();setStatus('Session diagnostics cleared');});
}

function renderPracticeCenter() {
  const tasks = [
    ['task-nav','Launch a transaction using the command field','Enter /nSE11, press Enter, then inspect the local SCARR definition.'],
    ['task-filter','Filter local table data','Open SE16N and filter SCARR with CARRID=LH.'],
    ['task-abap','Open the ABAP editor','Open SE38, choose cloud_classrun.abap and run it in ABAP Cloud mode.'],
    ['task-types','Inspect a local type definition','Open SE11 and switch between SCARR, SPFLI and SFLIGHT.'],
    ['task-class','Explore a class definition','Open SE24 and display ZCL_DEMO_GREETER.'],
    ['task-doc','Display a business document','Open VA03 and display training order 50000001.'],
  ];
  const done = (()=>{try{return JSON.parse(localStorage.getItem('abapc-practice-done')||'{}')}catch{return {}}})();
  $('screen').innerHTML = `<div class="welcome-banner"><h2>Certification Practice Center</h2><p>Practice the common navigation patterns and core development concepts before your training system access becomes available. These tasks teach workflow and vocabulary; they do not substitute for a real SAP training tenant.</p></div><div class="panel"><div class="panel-title">Guided practice checklist</div><div class="panel-body" id="practice-list">${tasks.map(([id,title,desc])=>`<label class="practice-step"><input type="checkbox" data-task="${id}" ${done[id]?'checked':''}><span><strong>${title}</strong><p>${desc}</p></span></label>`).join('')}<div class="toolbar-row" style="margin-top:10px"><button class="secondary-btn" id="reset-practice">Reset checklist</button><span class="small-note" id="practice-progress"></span></div></div></div><div class="panel"><div class="panel-title">Command field practice</div><div class="panel-body"><p class="small-note"><b>/nSE38</b> — navigate in the current session · <b>/oSE16N</b> — open the transaction in this simulator's current view · <b>/n</b> — Easy Access · <b>SE24</b> — Class Builder · <b>ME23N</b> — local purchase-order display.</p><p class="small-note">Function keys: F3 Back · Shift+F3 Easy Access · F12 Cancel · Ctrl+S save editor draft · Ctrl+G focus command.</p></div></div>`;
  const update=()=>{const checks=[...$('practice-list').querySelectorAll('[data-task]')];const map={};checks.forEach(c=>map[c.dataset.task]=c.checked);try{localStorage.setItem('abapc-practice-done',JSON.stringify(map))}catch{};$('practice-progress').textContent=`${checks.filter(c=>c.checked).length} of ${checks.length} completed`;};
  $('practice-list').querySelectorAll('[data-task]').forEach(c=>c.addEventListener('change',update));$('reset-practice').addEventListener('click',()=>{$('practice-list').querySelectorAll('[data-task]').forEach(c=>c.checked=false);update();});update();
}
function renderClassBuilder() {
  $('screen').innerHTML = `<div class="notice info">Training simulator: class metadata below is illustrative and is not loaded from an SAP repository.</div><div class="toolbar-row"><label for="class-name">Class</label><input id="class-name" value="ZCL_DEMO_GREETER" /><button class="primary-btn" id="class-display">Display</button><button class="secondary-btn" id="class-open-code">Open class-run example in SE38</button></div><div id="class-detail"></div>`;
  const classes={ZCL_DEMO_GREETER:{description:'Simple example class for practicing OO ABAP vocabulary.',visibility:'PUBLIC',final:'X',interfaces:'IF_OO_ADT_CLASSRUN',methods:[['IF_OO_ADT_CLASSRUN~MAIN','PUBLIC','Entry point for ABAP console examples'],['FORMAT_GREETING','PRIVATE','Returns a greeting string']]},ZCL_PRODUCT_SERVICE:{description:'Illustrative service class for internal-table practice.',visibility:'PUBLIC',final:'X',interfaces:'—',methods:[['GET_PRODUCTS','PUBLIC','Returns training product rows'],['CALCULATE_TOTAL','PUBLIC','Calculates a value from rows']]}};
  const draw=()=>{const name=$('class-name').value.trim().toUpperCase();const c=classes[name];$('class-detail').innerHTML=c?`<div class="panel"><div class="panel-title">Class ${esc(name)} · Display</div><div class="panel-body"><div class="sap-form-grid"><label>Class description</label><div>${esc(c.description)}</div><label>Visibility</label><div>${esc(c.visibility)}</div><label>Final</label><div>${esc(c.final)}</div><label>Interfaces</label><div>${esc(c.interfaces)}</div></div><div class="sap-tabs"><button class="active" data-class-tab="methods">Methods</button><button data-class-tab="attributes">Attributes</button><button data-class-tab="interfaces">Interfaces</button></div><div id="class-tab-pane"><table class="data-table"><thead><tr><th>Method</th><th>Visibility</th><th>Description</th></tr></thead><tbody>${c.methods.map(m=>`<tr>${m.map(v=>`<td>${esc(v)}</td>`).join('')}</tr>`).join('')}</tbody></table></div></div></div>`:`<div class="message-strip error">Class ${esc(name||'(empty)')} is not in the local training catalog. Try ZCL_DEMO_GREETER or ZCL_PRODUCT_SERVICE.</div>`;
    document.querySelectorAll('[data-class-tab]').forEach(b=>b.addEventListener('click',()=>{document.querySelectorAll('[data-class-tab]').forEach(x=>x.classList.toggle('active',x===b));const pane=$('class-tab-pane');if(b.dataset.classTab==='methods')pane.innerHTML=`<table class="data-table"><thead><tr><th>Method</th><th>Visibility</th><th>Description</th></tr></thead><tbody>${c.methods.map(m=>`<tr>${m.map(v=>`<td>${esc(v)}</td>`).join('')}</tr>`).join('')}</tbody></table>`;else if(b.dataset.classTab==='attributes')pane.innerHTML='<div class="small-note">No public instance attributes declared in this teaching example.</div>';else pane.innerHTML=`<table class="data-table"><thead><tr><th>Interface</th><th>Purpose</th></tr></thead><tbody><tr><td>${esc(c.interfaces)}</td><td>Illustrative class interface metadata</td></tr></tbody></table>`;}));};
  $('class-display').addEventListener('click',draw);$('class-name').addEventListener('keydown',e=>{if(e.key==='Enter')draw()});$('class-open-code').addEventListener('click',()=>{state.pendingSource=`CLASS zcl_demo_greeter DEFINITION\n  PUBLIC\n  FINAL\n  CREATE PUBLIC.\n\n  PUBLIC SECTION.\n    INTERFACES if_oo_adt_classrun.\nENDCLASS.\n\nCLASS zcl_demo_greeter IMPLEMENTATION.\n  METHOD if_oo_adt_classrun~main.\n    out->write( 'Hello from ABAPC training' ).\n  ENDMETHOD.\nENDCLASS.`;state.editorFilename='zcl_demo_greeter.abap';state.editorPreset='';goTo('SE38');});draw();
}
function renderFunctionBuilder() {
  const funcs={Z_DEMO_ADD:{group:'ZFG_TRAINING',description:'Adds two integer values (local simulator function).',params:[['IV_A','IMPORTING','TYPE I'],['IV_B','IMPORTING','TYPE I'],['EV_SUM','EXPORTING','TYPE I']]},Z_DEMO_GREETING:{group:'ZFG_TRAINING',description:'Returns a training greeting for a supplied name.',params:[['IV_NAME','IMPORTING','TYPE STRING'],['EV_TEXT','EXPORTING','TYPE STRING']]}};
  $('screen').innerHTML=`<div class="notice info">These are simulated function-module signatures. They are not function modules activated in SAP.</div><div class="toolbar-row"><label for="function-name">Function module</label><select id="function-name">${Object.keys(funcs).map(n=>`<option>${n}</option>`).join('')}</select><button class="primary-btn" id="function-display">Display</button><button class="secondary-btn" id="function-test">Test training function</button></div><div id="function-detail"></div>`;
  const draw=()=>{const name=$('function-name').value,f=funcs[name];$('function-detail').innerHTML=`<div class="panel"><div class="panel-title">Function Builder · ${esc(name)}</div><div class="panel-body"><div class="sap-form-grid"><label>Function group</label><div>${esc(f.group)}</div><label>Short text</label><div>${esc(f.description)}</div><label>Processing type</label><div>Normal function module (simulated)</div></div><div class="section-label">Interface</div><table class="data-table"><thead><tr><th>Parameter</th><th>Direction</th><th>Typing</th></tr></thead><tbody>${f.params.map(r=>`<tr>${r.map(v=>`<td>${esc(v)}</td>`).join('')}</tr>`).join('')}</tbody></table><div id="function-test-result"></div></div></div>`;};
  $('function-display').addEventListener('click',draw);$('function-name').addEventListener('change',draw);$('function-test').addEventListener('click',()=>{if(! $('function-detail').innerHTML)draw();const name=$('function-name').value;const r=name==='Z_DEMO_ADD'?`<div class="message-strip">Test result: IV_A = 12, IV_B = 30 → EV_SUM = 42</div>`:`<div class="message-strip">Test result: IV_NAME = ABAP → EV_TEXT = Hello, ABAP!</div>`;$('function-test-result').innerHTML=r;});draw();
}
function renderMessageCatalog() {
  const messages=[['ZABAP','001','Program &1 completed successfully','S'],['ZABAP','002','Material &1 was not found','E'],['ZABAP','003','Local training data is read-only','I'],['ZABAP','004','Order &1 is displayed in simulation mode','S']];
  $('screen').innerHTML=`<div class="notice info">Local sample message class catalog. No SAP message repository is connected.</div><div class="toolbar-row"><label for="msg-class">Message class</label><input id="msg-class" value="ZABAP"/><label for="msg-search">Search text</label><input id="msg-search" placeholder="successfully, Material…"/><button class="secondary-btn" id="msg-search-btn">Find</button></div><div id="message-results"></div>`;
  const draw=()=>{const cl=$('msg-class').value.trim().toUpperCase(),q=$('msg-search').value.trim().toLowerCase();const rows=messages.filter(m=>(!cl||m[0]===cl)&&(!q||m[2].toLowerCase().includes(q)||m[1].includes(q)));$('message-results').innerHTML=`<div class="table-wrap"><table class="data-table"><thead><tr><th>Class</th><th>Number</th><th>Message text</th><th>Type</th></tr></thead><tbody>${rows.map(r=>`<tr>${r.map(v=>`<td>${esc(v)}</td>`).join('')}</tr>`).join('')||'<tr><td colspan="4">No matching local messages.</td></tr>'}</tbody></table></div>`;};$('msg-search-btn').addEventListener('click',draw);$('msg-search').addEventListener('keydown',e=>{if(e.key==='Enter')draw()});draw();
}
function renderMockBusinessTx(tx) {
  const data={
    MM03:{label:'Material',field:'Material number',placeholder:'MAT-100',records:{'MAT-100':{description:'Stainless Steel Bolt M8',type:'ROH · Raw material',base:'EA',plant:'1000 · Training Plant',storage:'0001 · Main Store',price:'2.40 EUR',stock:'1,250 EA'},'MAT-200':{description:'Safety Helmet Yellow',type:'HAWA · Trading goods',base:'EA',plant:'1000 · Training Plant',storage:'0002 · PPE Store',price:'18.90 EUR',stock:'84 EA'},'MAT-300':{description:'Optical Mouse USB',type:'HAWA · Trading goods',base:'EA',plant:'1100 · Training Plant East',storage:'0001 · Main Store',price:'12.50 EUR',stock:'216 EA'}}},
    VA03:{label:'Sales order',field:'Sales order',placeholder:'50000001',records:{'50000001':{description:'Customer order — Northwind Training GmbH',customer:'C1000 · Northwind Training GmbH',date:'2026-10-12',status:'Open · Training status',items:[['10','MAT-200','Safety Helmet Yellow','10 EA','189.00 EUR'],['20','MAT-300','Optical Mouse USB','5 EA','62.50 EUR']]},'50000002':{description:'Customer order — Contoso Learning AG',customer:'C1002 · Contoso Learning AG',date:'2026-10-18',status:'Complete · Training status',items:[['10','MAT-100','Stainless Steel Bolt M8','100 EA','240.00 EUR']]}}},
    ME23N:{label:'Purchase order',field:'Purchase order',placeholder:'4500000010',records:{'4500000010':{description:'Standard purchase order — Training Supplies',vendor:'V1000 · Demo Industrial Supplies',date:'2026-10-10',porg:'1000 · Training Purchasing',status:'Open · Simulated',items:[['10','MAT-100','Stainless Steel Bolt M8','500 EA','1,200.00 EUR'],['20','MAT-200','Safety Helmet Yellow','20 EA','378.00 EUR']]},'4500000011':{description:'Standard purchase order — Learning Equipment',vendor:'V1002 · Example Office Goods',date:'2026-10-21',porg:'1000 · Training Purchasing',status:'Released · Simulated',items:[['10','MAT-300','Optical Mouse USB','40 EA','500.00 EUR']]}}},
    MMBE:{label:'Material',field:'Material number',placeholder:'MAT-100',records:{'MAT-100':{description:'Stainless Steel Bolt M8',stock:[['1000','0001','Main Store','1,000 EA'],['1000','0002','Production Store','250 EA']]},'MAT-200':{description:'Safety Helmet Yellow',stock:[['1000','0002','PPE Store','84 EA']]},'MAT-300':{description:'Optical Mouse USB',stock:[['1100','0001','Main Store','216 EA']]}}}
  }[tx];
  if(!data){$('screen').innerHTML='<div class="empty-state">Transaction not configured.</div>';return;}
  $('screen').innerHTML=`<div class="notice info">Fictional master/document data for GUI practice. Display-only; no posting, stock update, accounting impact, or SAP connection.</div><div class="panel"><div class="panel-title">Selection</div><div class="panel-body"><div class="sap-form-grid"><label for="mock-doc-id">${esc(data.field)}</label><input id="mock-doc-id" value="${esc(data.placeholder)}"/><span></span><div class="toolbar-row"><button class="primary-btn" id="mock-display">Display</button><button class="secondary-btn" id="mock-clear">Clear</button></div></div></div></div><div id="mock-document"></div>`;
  const draw=()=>{const id=$('mock-doc-id').value.trim().toUpperCase();const r=data.records[id];if(!r){$('mock-document').innerHTML=`<div class="message-strip error">${esc(data.field)} ${esc(id||'(empty)')} not found in local sample data. Try ${Object.keys(data.records).map(esc).join(', ')}.</div>`;setStatus('No local training record found');return;}let body='';
    if(tx==='MM03') body=`<div class="sap-tabs"><button class="active">Basic Data</button><button>Plant Data</button><button>Accounting</button></div><div class="sap-form-grid"><label>Material</label><div class="mono">${esc(id)}</div><label>Description</label><div>${esc(r.description)}</div><label>Material type</label><div>${esc(r.type)}</div><label>Base unit</label><div>${esc(r.base)}</div><label>Plant</label><div>${esc(r.plant)}</div><label>Storage location</label><div>${esc(r.storage)}</div><label>Valuation price</label><div>${esc(r.price)}</div><label>Local stock</label><div>${esc(r.stock)}</div></div>`;
    else if(tx==='MMBE') body=`<div><b>${esc(id)}</b> — ${esc(r.description)}</div><div class="section-label">Stock by plant / storage location</div><table class="data-table"><thead><tr><th>Plant</th><th>Storage location</th><th>Description</th><th>Unrestricted use</th></tr></thead><tbody>${r.stock.map(x=>`<tr>${x.map(y=>`<td>${esc(y)}</td>`).join('')}</tr>`).join('')}</tbody></table>`;
    else {const partner=tx==='VA03'?'Sold-to party':'Vendor';const partnerValue=tx==='VA03'?r.customer:r.vendor;body=`<div class="sap-tabs"><button class="active">Overview</button><button>Items</button><button>Status</button></div><div class="sap-form-grid"><label>${esc(data.field)}</label><div class="mono">${esc(id)}</div><label>Description</label><div>${esc(r.description)}</div><label>${partner}</label><div>${esc(partnerValue)}</div><label>Document date</label><div>${esc(r.date)}</div><label>${tx==='VA03'?'Overall status':'Purchasing organization'}</label><div>${esc(tx==='VA03'?r.status:r.porg)}</div>${tx==='ME23N'?`<label>Document status</label><div>${esc(r.status)}</div>`:''}</div><div class="section-label">Document Items</div><table class="data-table"><thead><tr><th>Item</th><th>Material</th><th>Short text</th><th>Quantity</th><th>Net value</th></tr></thead><tbody>${r.items.map(x=>x.map(y=>`<td>${esc(y)}</td>`).join('')).map((row,i)=>`<tr>${row}</tr>`).join('')}</tbody></table>`;}
    $('mock-document').innerHTML=`<div class="panel"><div class="panel-title">${esc(tx)} · ${esc(id)} — ${esc(r.description)}</div><div class="panel-body">${body}<div class="small-note" style="margin-top:10px">Simulation result only. This record is not stored in SAP and cannot be used as evidence of a posted business transaction.</div></div></div>`;setStatus(`${tx}: displayed local training record ${id}`);};
  $('mock-display').addEventListener('click',draw);$('mock-doc-id').addEventListener('keydown',e=>{if(e.key==='Enter')draw()});$('mock-clear').addEventListener('click',()=>{$('mock-doc-id').value='';$('mock-document').innerHTML='';$('mock-doc-id').focus()});draw();
}
function renderTransactions(){const rows=Object.entries(txInfo).map(([code,info])=>[code,info[0],info[1]]);$('screen').innerHTML=`<div class="notice info">Use the local code catalog to navigate this training simulator. A trailing * in the side tree denotes a fictional/local equivalent, not an SAP-delivered transaction.</div><div class="toolbar-row"><label for="tx-search">Find transaction</label><input id="tx-search" placeholder="Code or description"/><button class="secondary-btn" id="tx-search-clear">Clear</button></div><div class="table-wrap"><table class="data-table"><thead><tr><th>Code</th><th>Local screen</th><th>Function / scope</th><th>Action</th></tr></thead><tbody id="tx-rows">${rows.map(r=>`<tr data-search="${esc(r.join(' ').toLowerCase())}"><td class="tcode">${esc(r[0])}</td><td>${esc(r[1])}</td><td>${esc(r[2])}</td><td><button class="link-button" data-tx="${esc(r[0])}">Open</button></td></tr>`).join('')}</tbody></table></div>`;document.querySelectorAll('[data-tx]').forEach(b=>b.addEventListener('click',()=>goTo(b.dataset.tx)));$('tx-search').addEventListener('input',()=>{const q=$('tx-search').value.trim().toLowerCase();document.querySelectorAll('#tx-rows tr').forEach(r=>r.hidden=!r.dataset.search.includes(q));});$('tx-search-clear').addEventListener('click',()=>{$('tx-search').value='';$('tx-search').dispatchEvent(new Event('input'))});}
function renderRealSapInfo(){$('screen').innerHTML=`<div class="welcome-banner"><h2>Connect to a real SAP system</h2><p>ABAPC is an independent local compiler. Adding a mock screen cannot provide SAP GUI protocols, system logon, SAP application services, DDIC, or live business data.</p></div><div class="notice">A real SAP GUI installation must connect to an SAP application server that you are authorized to use. SAP GUI itself is distributed and documented by SAP. See the official documentation links below.</div><div class="panel"><div class="panel-title">What you need</div><div class="panel-body"><ol><li>An SAP system (on-premises AS ABAP or a supported SAP environment) and permission to log on.</li><li>Connection details from the system administrator: server/message server, system number or group, client, and approved authentication method.</li><li>The appropriate SAP GUI client or supported browser-based SAP GUI for HTML on an SAP system configured to provide it.</li><li>For ABAP Cloud development, the correct ABAP environment and development tools; the local ABAPC VM is not that backend.</li></ol><p class="small-note">Do not enter SAP passwords into this local simulator. This page does not store credentials or attempt a network connection to SAP.</p><div class="toolbar-row"><a class="btn" href="https://help.sap.com/docs/r/product/sap_gui_for_windows" target="_blank" rel="noopener">SAP GUI for Windows docs ↗</a><a class="btn" href="https://www.sap.com/products/try-sap/trials-downloads.html" target="_blank" rel="noopener">SAP trials and downloads ↗</a><a class="btn" href="https://learning.sap.com/courses/introducing-sap-abap-platform-fundamentals/introducing-the-sap-gui-1" target="_blank" rel="noopener">How SAP GUI works ↗</a></div></div></div><div class="panel"><div class="panel-title">Connection profile (reference only)</div><div class="panel-body"><div class="session-config"><label>Application server / message server<input disabled placeholder="Obtained from SAP administrator" /></label><label>System number / group<input disabled placeholder="For example, 00 or PUBLIC" /></label><label>System ID (SID)<input disabled placeholder="3-character system ID" /></label><label>Client<input disabled placeholder="3-digit client" /></label></div><p class="disabled-note">Connection is disabled because ABAPC does not implement SAP GUI's DIAG protocol or an SAP backend integration.</p></div></div>`;}
$('go').addEventListener('click',()=>goTo($('command').value));$('command').addEventListener('keydown',e=>{if(e.key==='Enter'||(e.ctrlKey&&e.key.toLowerCase()==='g')){e.preventDefault();goTo($('command').value);}});
document.querySelectorAll('.tree-node[data-tcode]').forEach(b=>b.addEventListener('click',()=>goTo(b.dataset.tcode)));
$('back').addEventListener('click',()=>goTo(state.stack.pop()||'S000',false));$('exit').addEventListener('click',()=>goTo('S000'));$('refresh').addEventListener('click',()=>renderCurrent());$('open-studio').addEventListener('click',()=>location.href='/');
$('cancel').addEventListener('click',()=>{if(state.current==='SE38'){notice('Editor remains open; use Back to leave without running.','info')}else goTo('S000');});
$('save').addEventListener('click', () => {
  if (state.current === 'SE38') {
    $('gui-vfs-save')?.click();
  } else {
    saveLocalDraft();
  }
});
$('print').addEventListener('click', () => window.print());
$('recent-toggle').addEventListener('click', () => {
  if (!state.recent.length) { showToast('No recent transactions yet.'); return; }
  const selected = prompt('Recent transaction codes:\n' + state.recent.join('  ·  ') + '\n\nEnter a code to open it:');
  if (selected) goTo(selected);
});

// Collapsible SAP Easy Access Rail Logic
const treePane = $('sap-tree-pane');
const guiLayout = $('gui-layout');
const treeCollapse = $('tree-collapse');
const treeCollapsedRail = $('tree-collapsed-rail');

function toggleTree(collapsed) {
  if (!treePane || !guiLayout) return;
  if (collapsed === undefined) {
    collapsed = !treePane.classList.contains('collapsed');
  }
  treePane.classList.toggle('collapsed', collapsed);
  guiLayout.classList.toggle('tree-collapsed', collapsed);
  if (treeCollapse) treeCollapse.textContent = collapsed ? '▶' : '◀';
  if (treeCollapsedRail) treeCollapsedRail.style.display = collapsed ? 'flex' : 'none';
  try {
    localStorage.setItem('abapc_easy_access_collapsed', collapsed ? '1' : '0');
  } catch {}
}

if (treeCollapse) treeCollapse.addEventListener('click', (e) => { e.stopPropagation(); toggleTree(); });
if (treeCollapsedRail) treeCollapsedRail.addEventListener('click', () => toggleTree(false));

const initialTreeCollapsed = localStorage.getItem('abapc_easy_access_collapsed') === '1';
if (initialTreeCollapsed) toggleTree(true);

$('open-editor')?.addEventListener('click', () => goTo('SE38'));
$('help-screen').addEventListener('click', () => showToast(`Current screen: ${state.current}. Use the command field for a transaction code. This is a local training simulator.`));
document.querySelectorAll('[data-menu]').forEach(b => b.addEventListener('click', () => {
  const m = b.dataset.menu;
  if (m === 'Help') showToast('Help: F1-style guidance is local only. Use the ? button for screen help.');
  else if (m === 'Favorites') goTo('S000');
  else if (m === 'Edit') goTo('SE38');
  else if (m === 'System') goTo('SE93');
  else if (m === 'Environment') goTo('ZCERT');
  else goTo('S000');
}));
document.querySelectorAll('[data-app-command]').forEach(b => b.addEventListener('click', () => {
  const a = b.dataset.appCommand;
  if (a === 'back') { goTo(state.stack.pop() || 'S000', false); return; }
  if (a === 'help') { showToast(`Help for ${state.current}: consult local practice notes or open ZCERT.`); return; }
  const runButton = document.getElementById('gui-run');
  const displayButton = document.getElementById('mock-display');
  if (runButton) { runButton.click(); return; }
  if (displayButton) { displayButton.click(); return; }
  showToast('No execute action is defined for this screen.');
}));
function showToast(text) {
  const el = $('sap-message-toast');
  el.textContent = text;
  el.classList.add('visible');
  clearTimeout(window.__sapToastTimer);
  window.__sapToastTimer = setTimeout(() => el.classList.remove('visible'), 3000);
  setStatus(text);
}
function saveLocalDraft() {
  const source = $('gui-source');
  if (!source) { showToast('No editor draft on this screen. Open SE38 first.'); return; }
  try {
    const draft = { source: source.value, filename: $('gui-filename')?.value || 'program.abap', mode: $('gui-mode')?.value || 'cloud', savedAt: new Date().toISOString() };
    localStorage.setItem('abapc-editor-draft', JSON.stringify(draft));
    showToast(`Draft saved locally: ${draft.filename}`);
  } catch {
    showToast('Could not save draft to this browser storage.');
  }
}
window.addEventListener('keydown', e => {
  if (e.shiftKey && e.key === 'F3') { e.preventDefault(); goTo('S000'); }
  else if (e.key === 'F3') { e.preventDefault(); goTo(state.stack.pop() || 'S000', false); }
  else if (e.key === 'F12') { e.preventDefault(); $('cancel').click(); }
  else if (e.key === 'F1') { e.preventDefault(); $('help-screen').click(); }
  else if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 's') { e.preventDefault(); $('save').click(); }
  else if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'g') { e.preventDefault(); $('command').focus(); $('command').select(); }
});
(async () => {
  try {
    const c = await jsonGet('/api/config');
    $('clock').textContent = `${c.version || 'local'} · INS`;
  } catch {
    $('clock').textContent = 'INS';
  }
  await loadExamples();
  renderRecentTree();
  const urlParams = new URLSearchParams(window.location.search);
  const startTx = urlParams.get('tx') || (window.location.hash ? window.location.hash.replace('#', '') : 'SE38');
  goTo(startTx, false);
})();
