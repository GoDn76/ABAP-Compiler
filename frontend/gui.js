const $ = id => document.getElementById(id);
function readSessionArray(key) { try { const value = JSON.parse(sessionStorage.getItem(key) || 'null'); return Array.isArray(value) ? value : []; } catch { return []; } }
function readRecentTx() { try { const value = JSON.parse(localStorage.getItem('abapc-recent-tx') || '[]'); return Array.isArray(value) ? value.filter(x => typeof x === 'string') : []; } catch { return []; } }
const state = {
  current: 'S000', stack: [], recent: readRecentTx(), tables: {}, metadata: {}, examples: [], executions: [],
  lastDiagnostics: readSessionArray('abapc-last-diagnostics'),
  lastOutput: readSessionArray('abapc-last-output'),
  editorPreset: 'cloud_classrun.abap', pendingSource: null, editorFilename: 'program.abap',
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
const API_BASE = (window.API_BASE_URL || localStorage.getItem('ABAPC_API_BASE') || '').replace(/\/$/, '');
const apiUrl = path => (path.startsWith('http') ? path : (API_BASE ? `${API_BASE}${path}` : path));
async function jsonGet(url) { const r = await fetch(apiUrl(url)); const d = await r.json(); if (!r.ok) throw new Error(d.error || `Request failed (${r.status})`); return d; }
async function jsonPost(url, body) { const r = await fetch(apiUrl(url),{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify(body)}); const d=await r.json(); if (!r.ok) throw new Error(d.error || `Request failed (${r.status})`); return d; }
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
async function renderEditor(presetName=state.editorPreset || 'cloud_classrun.abap') {
  const screen=$('screen'); screen.innerHTML='<div class="empty-state">Loading editor…</div>';
  let source=state.pendingSource ?? `CLASS zcl_cloud_demo DEFINITION\n  PUBLIC\n  FINAL\n  CREATE PUBLIC.\n\n  PUBLIC SECTION.\n    INTERFACES if_oo_adt_classrun.\nENDCLASS.\n\nCLASS zcl_cloud_demo IMPLEMENTATION.\n  METHOD if_oo_adt_classrun~main.\n    DATA lv_total TYPE i.\n    lv_total = 10 + 20.\n    out->write( lv_total ).\n  ENDMETHOD.\nENDCLASS.`;
  const pending = state.pendingSource !== null;
  if (pending) state.pendingSource = null;
  let parameterSpecs = null;
  let parameterSource = null;
  let parameterMode = null;
  state.editorPreset = presetName || state.editorPreset;
  let examples=state.examples.length?state.examples:await loadExamples();
  try { if (!pending && presetName) { const d=await jsonGet('/api/example?name='+encodeURIComponent(presetName)); source=d.source; } } catch {}
  screen.innerHTML=`
    <div class="toolbar-row"><label for="gui-example">Example</label><select id="gui-example"><option value="">Choose example…</option>${examples.map(n=>`<option value="${esc(n)}" ${n===presetName?'selected':''}>${esc(n)}</option>`).join('')}</select><label for="gui-mode">Language</label><select id="gui-mode"><option value="standard">Standard ABAP</option><option value="cloud" selected>ABAP Cloud</option></select><label for="gui-filename">File</label><input id="gui-filename" value="${esc(state.editorFilename || 'program.abap')}" aria-label="File name"/><button class="secondary-btn" id="gui-build">Check</button><button class="secondary-btn" id="gui-restore-draft">Restore draft</button><button class="primary-btn" id="gui-run">▶ Execute</button></div>
    <div id="gui-parameters"></div>
    <textarea class="code-editor" id="gui-source" spellcheck="false" aria-label="ABAP code editor">${esc(source)}</textarea>
    <div class="section-label">Execution output</div><div class="console" id="gui-output">Ready. Select Execute to run this source in the ABAPC local runtime.</div>
    <div class="section-label">Diagnostics</div><div id="gui-diags"><div class="empty-state">No diagnostics yet.</div></div>
    <p class="small-note">Every Execute request uses the ABAPC compiler/runtime and is archived under <span class="mono">executed codes/</span>. This is a local subset, not SAP application-server execution.</p>`;
  const resetSelectionScreen=()=>{parameterSpecs=null;parameterSource=null;parameterMode=null;$('gui-parameters').innerHTML='';$('gui-run').textContent='▶ Execute';};
  const rangeRowMarkup=(v,i,rowIndex,initialRow=null)=>{
    const noIntervals=!!v.noIntervals;
    const sign=String(initialRow?.SIGN??initialRow?.sign??'I').toUpperCase();
    const option=String(initialRow?.OPTION??initialRow?.option??'EQ').toUpperCase();
    const low=String(initialRow?.LOW??initialRow?.low??'');
    const high=String(initialRow?.HIGH??initialRow?.high??'');
    const opts=(noIntervals?['EQ','NE','GT','GE','LT','LE','CP','NP']:['EQ','NE','GT','GE','LT','LE','BT','NB','CP','NP']).map(o=>`<option value="${o}" ${o===option?'selected':''}>${o}</option>`).join('');
    return `<div class="range-row" data-range-row data-range-index="${rowIndex}">
      <select aria-label="Include or exclude" data-range-part="sign"><option value="I" ${sign==='I'?'selected':''}>Include</option><option value="E" ${sign==='E'?'selected':''}>Exclude</option></select>
      <select aria-label="Selection option" data-range-part="option">${opts}</select>
      <input aria-label="Low value" placeholder="Low value" value="${esc(low)}" data-range-part="low" data-f4-name="${esc(v.name)}" data-f4-row="${rowIndex}" />
      ${noIntervals?'':`<input aria-label="High value" placeholder="High value" value="${esc(high)}" data-range-part="high" ${['BT','NB'].includes(option)?'':'disabled'} />`}
      <button class="secondary-btn small-btn" type="button" data-range-f4 title="Value help">F4</button>
      ${v.noExtension?'':'<button class="secondary-btn small-btn" type="button" data-range-remove title="Remove row">−</button>'}
    </div>`;
  };
  const syncRangeRow=(row)=>{
    const option=row.querySelector('[data-range-part="option"]')?.value||'EQ';
    const high=row.querySelector('[data-range-part="high"]');if(!high)return;
    const enabled=['BT','NB'].includes(option);high.disabled=!enabled;if(!enabled)high.value='';
  };
  const bindRangeRows=(root)=>root.querySelectorAll('[data-range-row]').forEach(row=>{
    syncRangeRow(row);const select=row.querySelector('[data-range-part="option"]');
    if(select&&select.dataset.bound!=='1'){select.dataset.bound='1';select.addEventListener('change',()=>syncRangeRow(row));}
  });
  const drawSelectionScreen=(specs)=>{
    const host=$('gui-parameters');
    const visible=specs.filter(v=>!v.noDisplay);
    const radioDefault=v=>v.initialValue===true||String(v.initialValue??'').toUpperCase()==='X'||String(v.initialValue??'').toUpperCase()==='TRUE';
    const fields=visible.map((v,i)=>{
      const type=v.type?.name||v.type?.kind||'string';
      const def=v.initialValue===undefined?(type==='i'||type==='int8'||type.startsWith('decfloat')?'0':''):String(v.initialValue);
      const label=String(v.name||'').toUpperCase();
      if(v.selectionOptions){
        const defaultRows=Array.isArray(v.initialValue)&&v.initialValue.length?v.initialValue:[null];
        return `<label>${esc(label)}${v.obligatory?' *':''}<button class="secondary-btn small-btn field-help-btn" type="button" data-help-param="${esc(v.name)}" title="Field help">?</button><div class="small-note range-reference">${esc(v.forField||'Selection range')}</div></label><div class="range-host" data-range-host data-range-name="${esc(v.name)}" data-range-required="${v.obligatory?'1':'0'}" data-range-no-extension="${v.noExtension?'1':'0'}"> <div class="range-rows">${defaultRows.map((row,j)=>rangeRowMarkup(v,i,j,row)).join('')}</div>${v.noExtension?'':`<button class="secondary-btn small-btn" type="button" data-range-add="${i}">+ Add range</button>`}</div>`;
      }
      if(v.radiobuttonGroup){
        const checked=radioDefault(v);
        return `<label for="gui-param-${i}">${esc(label)}${v.obligatory?' *':''}<button class="secondary-btn small-btn field-help-btn" type="button" data-help-param="${esc(v.name)}" title="Field help">?</button><div class="small-note">Group ${esc(v.radiobuttonGroup)}</div></label><div><input id="gui-param-${i}" type="radio" name="gui-radio-${esc(v.radiobuttonGroup)}" data-param-name="${esc(v.name)}" data-param-radio-group="${esc(v.radiobuttonGroup)}" ${checked?'checked':''}/></div>`;
      }
      if(v.checkbox){return `<label for="gui-param-${i}">${esc(label)}${v.obligatory?' *':''}<button class="secondary-btn small-btn field-help-btn" type="button" data-help-param="${esc(v.name)}" title="Field help">?</button></label><div><input id="gui-param-${i}" type="checkbox" data-param-name="${esc(v.name)}" data-param-required="${v.obligatory?'1':'0'}" ${def==='X'||def==='true'||def==='1'?'checked':''}/></div>`;}
      const inputType=['i','int8','decfloat16','decfloat34'].includes(type)?'number':'text';
      const valueInput=`<input id="gui-param-${i}" data-param-name="${esc(v.name)}" data-param-required="${v.obligatory?'1':'0'}" data-param-type="${esc(type)}" type="${inputType}" value="${esc(def)}" ${v.obligatory?'required':''}/>`;
      return `<label for="gui-param-${i}">${esc(label)}${v.obligatory?' *':''}<button class="secondary-btn small-btn field-help-btn" type="button" data-help-param="${esc(v.name)}" title="Field help">?</button></label><div class="parameter-input-wrap">${valueInput}${v.valueHelp?`<button class="secondary-btn small-btn" type="button" data-f4-param="${esc(v.name)}">F4</button>`:''}</div>`;
    }).join('');
    host.innerHTML=`<div class="panel selection-screen-panel"><div class="panel-title">Selection Screen <span class="muted">· 1000 · local simulation</span></div><div class="panel-body"><p class="small-note">Enter report parameters and ranges, then choose <b>Run with inputs</b>. F4 uses local training values.</p><div class="sap-form-grid">${fields}</div><div class="small-note">Fields marked * are mandatory. Selection options support include/exclude and EQ, NE, GT, GE, LT, LE, BT, NB, CP, NP. Values are passed to the local ABAPC runtime, not to an SAP application server.</div></div></div>`;
    bindRangeRows(host);
    host.querySelectorAll('[data-range-add]').forEach(button=>button.addEventListener('click',()=>{
      const hostEl=button.closest('[data-range-host]');if(!hostEl)return;const name=hostEl.dataset.rangeName;const v=specs.find(x=>String(x.name).toUpperCase()===String(name).toUpperCase());if(!v)return;
      const rows=hostEl.querySelector('.range-rows');if(!rows)return;const count=rows.querySelectorAll('[data-range-row]').length;rows.insertAdjacentHTML('beforeend',rangeRowMarkup(v,Number(button.dataset.rangeAdd),count));const newest=rows.lastElementChild;if(newest){newest.querySelector('[data-range-remove]')?.addEventListener('click',()=>newest.remove());bindRangeRows(newest);}bindF4Buttons(rows);
    }));
    host.querySelectorAll('[data-range-remove]').forEach(button=>button.addEventListener('click',()=>button.closest('[data-range-row]')?.remove()));
    bindF4Buttons(host);
  };
  const openValueHelp=async(name,input)=>{
    const v=parameterSpecs?.find(x=>String(x.name).toUpperCase()===String(name).toUpperCase());
    let values=[];
    try{
      const eventResult=await jsonPost('/api/selection-event',{source:$('gui-source').value,mode:$('gui-mode').value,filename:$('gui-filename').value,event:'at-selection-screen-value-request',target:name,parameterValues:{[name]:input.value}});
      if(eventResult.ok&&Array.isArray(eventResult.output)&&eventResult.output.length)values=eventResult.output.map(String).map(x=>x.trim()).filter(Boolean);
    }catch{}
    try{
      const tables=await getTables();
      const reference=String(v?.forField||'').toUpperCase().split('-');
      const table=reference.length>1?reference[0]:'';const field=reference.length>1?reference.slice(1).join('-'):'';
      if(!values.length&&table&&field&&Array.isArray(tables[table]))values=tables[table].map(row=>row[field]??row[field.toUpperCase()]).filter(x=>x!==undefined&&x!==null).map(String);
    }catch{}
    if(!values.length){
      const key=String(v?.valueHelp||v?.forField||name).toUpperCase();
      if(key.includes('CARR')||key.includes('AIRLINE'))values=['AA','LH','SQ'];
      else if(key.includes('COUNTRY'))values=['DE','IN','JP','SG','US'];
      else if(key.includes('DATE')||key.includes('DATUM'))values=['2026-10-01','2026-10-02','2026-10-03'];
      else values=['A','B','C','100','200','300'];
    }
    values=[...new Set(values)];
    document.querySelector('.f4-overlay')?.remove();
    const overlay=document.createElement('div');overlay.className='f4-overlay';
    overlay.innerHTML=`<div class="f4-dialog" role="dialog" aria-modal="true" aria-label="Value help"><div class="f4-title">Value Help <span>${esc(String(name).toUpperCase())}</span><button type="button" data-f4-close>×</button></div><input class="f4-search" placeholder="Search values…" aria-label="Search values"><div class="f4-values">${values.map(value=>`<button type="button" class="f4-value" data-f4-value="${esc(value)}">${esc(value)}</button>`).join('')}</div><div class="small-note">Local values for practice only.</div></div>`;
    document.body.append(overlay);
    overlay.addEventListener('click',e=>{if(e.target===overlay||e.target.closest('[data-f4-close]')){overlay.remove();return;}const item=e.target.closest('[data-f4-value]');if(item){input.value=item.dataset.f4Value;input.dispatchEvent(new Event('input',{bubbles:true}));overlay.remove();input.focus();}});
    overlay.querySelector('.f4-search').addEventListener('input',e=>{const q=e.target.value.toLowerCase();overlay.querySelectorAll('[data-f4-value]').forEach(b=>b.hidden=!b.dataset.f4Value.toLowerCase().includes(q));});
    overlay.querySelector('.f4-search').focus();
  };
  const openFieldHelp=async(name)=>{
    let message='Local ABAPC selection field. Enter a value manually or use F4 for available local training values.';
    try{
      const result=await jsonPost('/api/selection-event',{source:$('gui-source').value,mode:$('gui-mode').value,filename:$('gui-filename').value,event:'at-selection-screen-help-request',target:name,parameterValues:{}});
      if(result.ok&&result.output?.length)message=result.output.map(String).join('\n');
    }catch{}
    document.querySelector('.f4-overlay')?.remove();const overlay=document.createElement('div');overlay.className='f4-overlay';
    overlay.innerHTML=`<div class="f4-dialog" role="dialog" aria-modal="true" aria-label="Field help"><div class="f4-title">Field Help <span>${esc(String(name).toUpperCase())}</span><button type="button" data-f4-close>×</button></div><pre class="f4-help-text">${esc(message)}</pre><div class="small-note">Help is evaluated by the local ABAPC runtime where a matching help-request event exists.</div></div>`;
    document.body.append(overlay);overlay.addEventListener('click',e=>{if(e.target===overlay||e.target.closest('[data-f4-close]'))overlay.remove();});
  };
  const bindF4Buttons=(root)=>{
    root.querySelectorAll('[data-range-f4]').forEach(button=>{if(button.dataset.bound==='1')return;button.dataset.bound='1';button.addEventListener('click',()=>{const row=button.closest('[data-range-row]');const low=row?.querySelector('[data-range-part="low"]');if(low)openValueHelp(row.closest('[data-range-host]')?.dataset.rangeName,low);});});
    root.querySelectorAll('[data-f4-param]').forEach(button=>{if(button.dataset.bound==='1')return;button.dataset.bound='1';button.addEventListener('click',()=>{const input=[...document.querySelectorAll('[data-param-name]')].find(x=>x.dataset.paramName?.toUpperCase()===button.dataset.f4Param.toUpperCase());if(input)openValueHelp(button.dataset.f4Param,input);});});
    root.querySelectorAll('[data-help-param]').forEach(button=>{if(button.dataset.bound==='1')return;button.dataset.bound='1';button.addEventListener('click',()=>openFieldHelp(button.dataset.helpParam));});
  };
  $('gui-source').addEventListener('input',resetSelectionScreen);
  $('gui-mode').addEventListener('change',resetSelectionScreen);
  $('gui-filename').addEventListener('input', e=>{state.editorFilename=e.target.value;});
  $('gui-restore-draft').addEventListener('click',()=>{try{const d=JSON.parse(localStorage.getItem('abapc-editor-draft')||'null');if(!d||typeof d.source!=='string'){showGuiOutput('err','No saved local draft found.');return;}$('gui-source').value=d.source;resetSelectionScreen();$('gui-filename').value=d.filename||'program.abap';$('gui-mode').value=d.mode==='standard'?'standard':'cloud';state.editorFilename=$('gui-filename').value;showGuiOutput('ok',`Restored local draft: ${state.editorFilename}`);setStatus('Local editor draft restored');}catch{showGuiOutput('err','Could not restore local draft.');}});
  $('gui-example').addEventListener('change', async e=>{if(!e.target.value)return;try{const d=await jsonGet('/api/example?name='+encodeURIComponent(e.target.value));$('gui-source').value=d.source;resetSelectionScreen();$('gui-filename').value=e.target.value;state.editorFilename=e.target.value;state.editorPreset=e.target.value;setStatus(`Loaded ${e.target.value}`);}catch(err){showGuiOutput('err',String(err));}});
  $('gui-build').addEventListener('click', async()=>{
    setStatus('Building source…'); const body={source:$('gui-source').value,mode:$('gui-mode').value,filename:$('gui-filename').value};
    try { const d=await jsonPost('/api/compile',body); renderGuiDiagnostics(d.diagnostics||[]); const errs=d.stats?.errors||0; showGuiOutput(errs?'err':'ok',errs?`Build failed (${errs} error(s)).`:'Build succeeded. No execution performed.'); setStatus(errs?'Build failed':'Build succeeded'); }
    catch(e){showGuiOutput('err',String(e));setStatus('Build failed');}
  });
  $('gui-run').addEventListener('click', async()=>{
    const run=$('gui-run');run.disabled=true;setStatus('Executing…');
    try {
      const sourceNow=$('gui-source').value, modeNow=$('gui-mode').value, filenameNow=$('gui-filename').value;
      if(parameterSource!==sourceNow||parameterMode!==modeNow){parameterSpecs=null;parameterSource=null;parameterMode=null;}
      if(!parameterSpecs){
        const checked=await jsonPost('/api/compile',{source:sourceNow,mode:modeNow,filename:filenameNow});
        renderGuiDiagnostics(checked.diagnostics||[]);
        if((checked.stats?.errors||0)>0){showGuiOutput('err',`Compilation failed (${checked.stats.errors} error(s)).`);setStatus('Compilation failed');return;}
        const detectedBase=(checked.ir?.vars||[]).filter(v=>v.parameter);
        if(detectedBase.length){
          let defaults={};
          try{const preview=await jsonPost('/api/selection-preview',{source:sourceNow,mode:modeNow,filename:filenameNow});if(preview.ok)defaults=preview.variables||{};}catch{}
          const detected=detectedBase.map(v=>{const key=String(v.name||'').toUpperCase();return Object.prototype.hasOwnProperty.call(defaults,key)?{...v,initialValue:defaults[key]}:v;});
          parameterSpecs=detected;parameterSource=sourceNow;parameterMode=modeNow;drawSelectionScreen(detected);run.textContent='▶ Run with inputs';showGuiOutput('ok',`Selection screen generated: ${detected.length} parameter(s). Initialization defaults were evaluated where supported. Enter values above, then select Run with inputs.`);setStatus('Selection screen ready · enter report parameters');return;
        }
      }
      const parameterValues={};
      if(parameterSpecs?.length){
        const radioGroups=new Map();
        for(const v of parameterSpecs){
          if(v.noDisplay)continue;
          if(v.selectionOptions){
            const host=[...document.querySelectorAll('[data-range-host]')].find(x=>x.dataset.rangeName?.toUpperCase()===String(v.name).toUpperCase());if(!host){parameterValues[v.name]=[];continue;}
            const rows=[...host.querySelectorAll('[data-range-row]')].map(row=>({sign:row.querySelector('[data-range-part="sign"]')?.value||'I',option:row.querySelector('[data-range-part="option"]')?.value||'EQ',low:row.querySelector('[data-range-part="low"]')?.value||'',high:v.noIntervals?'':(row.querySelector('[data-range-part="high"]')?.value||'')})).filter(row=>row.low.trim()!==''||row.high.trim()!=='');
            if(v.obligatory&&!rows.length){host.querySelector('[data-range-part="low"]')?.focus();showGuiOutput('err',`At least one range is required for ${v.name}.`);setStatus(`Missing required selection ${v.name}`);return;}
            parameterValues[v.name]=rows;continue;
          }
          const el=[...document.querySelectorAll('[data-param-name]')].find(x=>x.dataset.paramName?.toUpperCase()===String(v.name).toUpperCase());if(!el)continue;
          if(v.radiobuttonGroup){const group=String(v.radiobuttonGroup).toUpperCase();if(!radioGroups.has(group))radioGroups.set(group,[]);radioGroups.get(group).push({v,el});continue;}
          const value=el.type==='checkbox'?(el.checked?'X':''):el.value;
          if(v.obligatory&&String(value).trim()===''){el.focus();showGuiOutput('err',`Required parameter ${v.name} is empty.`);setStatus(`Missing required parameter ${v.name}`);return;}
          parameterValues[v.name]=value;
        }
        for(const [group,entries] of radioGroups){const chosen=entries.find(x=>x.el.checked);for(const entry of entries)parameterValues[entry.v.name]=chosen&&chosen.v.name===entry.v.name?'X':'';if(entries.some(x=>x.v.obligatory)&&!chosen){entries[0]?.el.focus();showGuiOutput('err',`Choose one option in radio-button group ${group}.`);setStatus(`Radio-button group ${group} requires a selection`);return;}}
      }
      const d=await jsonPost('/api/execute',{source:sourceNow,mode:modeNow,filename:filenameNow,parameterValues}); state.lastDiagnostics=d.diagnostics||[];state.lastOutput=d.output||[]; sessionStorage.setItem('abapc-last-diagnostics',JSON.stringify(state.lastDiagnostics));sessionStorage.setItem('abapc-last-output',JSON.stringify(state.lastOutput));
      renderGuiDiagnostics(state.lastDiagnostics); const lines=[`Status: ${d.ok?'OK':'FAILED'}`,...(d.output||[]),d.savedFile?`Archived: ${d.savedFile}`:'',d.stats?`Tokens: ${d.stats.tokens??0} · Time: ${d.stats.elapsedMs??0} ms · VM steps: ${d.runtime?.steps??0}`:''].filter(Boolean); showGuiOutput(d.ok?'ok':'err',lines.join('\n'));setStatus(d.ok?'Execution finished':'Execution failed');
    } catch(e){state.lastDiagnostics=[{severity:'error',code:'IDE',message:String(e),start:{line:1,column:1}}];sessionStorage.setItem('abapc-last-diagnostics',JSON.stringify(state.lastDiagnostics));renderGuiDiagnostics(state.lastDiagnostics);showGuiOutput('err',String(e));setStatus('Execution failed');} finally {run.disabled=false;}
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
async function renderObjectNavigator(){const names=state.examples.length?state.examples:await loadExamples();$('screen').innerHTML=`<div class="two-col"><div class="panel"><div class="panel-title">Local example programs</div><div class="panel-body"><p class="small-note">Open a supplied sample in the local ABAP editor. Example files are not SAP repository objects.</p><div class="object-list" id="object-list">${names.map(n=>`<button class="object-row" data-example="${esc(n)}"><span>▤</span><span>${esc(n)}</span><small>ABAP source</small></button>`).join('')||'<div class="empty-state">No examples found.</div>'}</div></div></div><div><div class="panel"><div class="panel-title">Workspace components</div><div class="panel-body"><div class="object-list"><div class="object-row">📁 <span>src/</span><small>compiler TypeScript</small></div><div class="object-row">📁 <span>dist/</span><small>compiled JS</small></div><div class="object-row">📁 <span>runtime/</span><small>local API manifest</small></div><div class="object-row">📁 <span>executed codes/</span><small>run archive</small></div><div class="object-row">📁 <span>docs/</span><small>specifications</small></div></div></div></div><div class="notice info">Object navigation is limited to files included in this local package; it does not activate, transport, or retrieve SAP repository objects.</div></div></div>`;
  document.querySelectorAll('[data-example]').forEach(b=>b.addEventListener('click',()=>{state.editorPreset=b.dataset.example;state.editorFilename=b.dataset.example;goTo('SE38',false);setStatus(`Opened ${b.dataset.example} in editor`);}));
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
$('save').addEventListener('click',()=>saveLocalDraft());$('print').addEventListener('click',()=>window.print());$('recent-toggle').addEventListener('click',()=>{if(!state.recent.length){showToast('No recent transactions yet.');return;}const selected=prompt('Recent transaction codes:\n'+state.recent.join('  ·  ')+'\n\nEnter a code to open it:');if(selected)goTo(selected)});
$('tree-collapse').addEventListener('click',()=>{const tree=$('tree-scroll');const collapsed=tree.classList.toggle('tree-collapsed');$('tree-collapse').textContent=collapsed?'+':'−';});
$('help-screen').addEventListener('click',()=>showToast(`Current screen: ${state.current}. Use the command field for a transaction code. This is a local training simulator.`));
document.querySelectorAll('[data-menu]').forEach(b=>b.addEventListener('click',()=>{const m=b.dataset.menu;if(m==='Help')showToast('Help: F1-style guidance is local only. Use the ? button for screen help.');else if(m==='Favorites')goTo('S000');else if(m==='Edit')goTo('SE38');else if(m==='System')goTo('SE93');else if(m==='Environment')goTo('ZCERT');else goTo('S000');}));
document.querySelectorAll('[data-app-command]').forEach(b=>b.addEventListener('click',()=>{const a=b.dataset.appCommand;if(a==='back'){goTo(state.stack.pop()||'S000',false);return;}if(a==='help'){showToast(`Help for ${state.current}: consult local practice notes or open ZCERT.`);return;}const runButton=document.getElementById('gui-run');const displayButton=document.getElementById('mock-display');if(runButton){runButton.click();return;}if(displayButton){displayButton.click();return;}showToast('No execute action is defined for this screen.');}));
function showToast(text){const el=$('sap-message-toast');el.textContent=text;el.classList.add('visible');clearTimeout(window.__sapToastTimer);window.__sapToastTimer=setTimeout(()=>el.classList.remove('visible'),3000);setStatus(text);}
function saveLocalDraft(){const source=$('gui-source');if(!source){showToast('No editor draft on this screen. Open SE38 first.');return;}try{const draft={source:source.value,filename:$('gui-filename')?.value||'program.abap',mode:$('gui-mode')?.value||'cloud',savedAt:new Date().toISOString()};localStorage.setItem('abapc-editor-draft',JSON.stringify(draft));showToast(`Draft saved locally: ${draft.filename}`)}catch{showToast('Could not save draft to this browser storage.')}}
window.addEventListener('keydown',e=>{if(e.shiftKey&&e.key==='F3'){e.preventDefault();goTo('S000')}else if(e.key==='F3'){e.preventDefault();goTo(state.stack.pop()||'S000',false)}else if(e.key==='F12'){e.preventDefault();$('cancel').click()}else if(e.key==='F1'){e.preventDefault();$('help-screen').click()}else if(e.ctrlKey&&e.key.toLowerCase()==='s'){e.preventDefault();saveLocalDraft()}else if(e.ctrlKey&&e.key.toLowerCase()==='g'){e.preventDefault();$('command').focus();$('command').select()}});
const chip=document.querySelector('.system-chip');if(chip){chip.style.cursor='pointer';chip.title='Click to configure Backend URL (Vercel / VPS)';chip.addEventListener('click',()=>{const cur=localStorage.getItem('ABAPC_API_BASE')||(window.API_BASE_URL||'');const next=prompt('Enter ABAPC Backend URL (e.g. https://your-vps.com:7345, or empty for same-origin):',cur);if(next!==null){if(next.trim()){localStorage.setItem('ABAPC_API_BASE',next.trim().replace(/\/$/,''));}else{localStorage.removeItem('ABAPC_API_BASE');}location.reload();}});}
(async()=>{try{const c=await jsonGet('/api/config');$('clock').textContent=`${c.version||'local'} · INS`;}catch{$('clock').textContent='INS';}await loadExamples();renderRecentTree();goTo('S000',false);})();
