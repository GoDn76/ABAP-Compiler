# ABAPC Studio 5.0.1 — System Architecture & API Contract Document

> [!NOTE]
> Produced by reverse-engineering the live source tree at
> `c:\Users\Gaurav Urmaliya\Downloads\ABAPC-Studio-5.0.1-SWITCH-Expression-Fix`.
> Every claim below is derived directly from `server.mjs`, `app/app.js`, `app/gui.js`,
> `src/compiler.ts`, `src/diagnostics.ts`, `src/ir.ts`, `src/vm.ts`, and `src/local-platform.ts`.

---

## 1. System Overview

```
┌───────────────────────────────────────────────────────────────────────┐
│                        Browser (127.0.0.1)                            │
│                                                                       │
│  /index.html ──► app.js         (Compiler IDE frontend)               │
│  /sapgui      ──► gui.js        (SAP GUI Practice Simulator frontend)  │
│                                                                       │
│  Both frontends call the same REST API via native fetch()             │
└──────────────────────────────┬────────────────────────────────────────┘
                               │  HTTP  (no TLS, localhost-only)
                               │  Content-Type: application/json
                               │  No auth token / no CORS headers needed
                               ▼
┌───────────────────────────────────────────────────────────────────────┐
│                    server.mjs  (Node.js HTTP server)                  │
│                                                                       │
│  Routing: manual if-chain on (method + pathname)                      │
│  Body limit: 2 MB (MAX_BODY constant)                                 │
│  Default port: 7345  (env: ABAPC_PORT)  — auto-increments on EADDRINUSE │
│  Binds to: 127.0.0.1 ONLY  (never 0.0.0.0)                           │
│                                                                       │
│  Static files served from: ./app/                                     │
│  Execution archive:        ./executed codes/                          │
│  Example files:            ./examples/                                │
│  Platform state:           ./runtime/local-platform-state.json        │
│  API manifest:             ./runtime/released-apis.json               │
└───────┬────────────────────────────────────────────────────────────────┘
        │  imports
        ├──► dist/src/compiler.js   (compileSource)
        ├──► dist/src/diagnostics.js (formatDiagnostic)
        └──► dist/src/local-platform.js (localPlatform singleton)
```

---

## 2. CORS & Transport Security

| Property | Value |
|---|---|
| Bind address | `127.0.0.1` only — the server is never publicly accessible |
| CORS headers | **None set** — requests are same-origin (`http://127.0.0.1:<port>`) |
| Authentication | **None** — no tokens, cookies, or sessions |
| TLS | **None** — plain HTTP only |
| Response headers always set | `Cache-Control: no-store`, `X-Content-Type-Options: nosniff` |
| Request body encoding | `UTF-8` raw JSON; server uses `JSON.parse()` after `Buffer.concat()` |

> [!IMPORTANT]
> Because there is no CORS and no auth, any new backend replacement must bind to `127.0.0.1`
> (or add CORS + auth if the bind address changes).

---

## 3. Static Routing

| Method | Path | Served from |
|---|---|---|
| `GET` | `/` | `app/index.html` |
| `GET` | `/sapgui` | `app/gui.html` |
| `GET` | `/transactions` | `app/gui.html` (alias) |
| `GET` | `/app.js` | `app/app.js` |
| `GET` | `/gui.js` | `app/gui.js` |
| `GET` | `/style.css` | `app/style.css` |
| `GET` | `/gui.css` | `app/gui.css` |

Path traversal is blocked: the resolved file path must remain inside `./app/`.

---

## 4. API Contracts — Complete Endpoint Reference

### 4.1 `GET /api/health`

**Purpose:** Liveness probe.  
**Called by:** Not called by either frontend at startup; available for tooling.

**Response `200`:**
```json
{
  "ok": true,
  "name": "ABAPC Studio",
  "version": "5.0.1"
}
```

---

### 4.2 `GET /api/config`

**Purpose:** Fetch version + API manifest counts. Called at IDE startup (`app.js` → `config()`) and in the SAP GUI simulator on page load (`gui.js` async IIFE).

**Response `200`:**
```json
{
  "version": "5.0.1",
  "manifest": {
    "functions": [],
    "classes": [],
    "interfaces": [],
    "tables": []
  },
  "limits": {
    "maxBodyBytes": 2097152,
    "maxSteps": 10000000
  },
  "localOnly": true
}
```
> `manifest` is the parsed contents of `runtime/released-apis.json`.  
> The IDE footer renders: `Cloud manifest: <total entry count>`.

---

### 4.3 `GET /api/executed`

**Purpose:** List the last ≤ 300 archived `.abap` files from `executed codes/`.  
**Called by:** `app.js` on load and after each Run; `gui.js` SM37 / execution archive screen.

**Response `200`:**
```json
{
  "files": [
    {
      "name": "2026-11-13_00-45-12-003_cloud_program.abap",
      "size": 312,
      "modified": "2026-11-13T00:45:12.003Z"
    }
  ]
}
```
Files are sorted descending by `modified`.

---

### 4.4 `GET /api/code?name=<filename>`

**Purpose:** Retrieve the source text of a single archived execution.  
**Called by:** `app.js` history click; `gui.js` archive open-in-editor.

**Query parameter:** `name` — bare filename; must not contain `/`, `\`, or `..`.

**Response `200`:**
```json
{
  "name": "2026-11-13_00-45-12-003_cloud_program.abap",
  "source": "REPORT zhello.\n..."
}
```

**Error `400`:**
```json
{ "error": "Invalid file name." }
```

**Error `404`:**
```json
{ "error": "Executed code file not found." }
```

---

### 4.5 `GET /api/examples`

**Purpose:** List all `.abap` files in the `examples/` directory.  
**Called by:** `app.js` on load; `gui.js` on SE38 render.

**Response `200`:**
```json
{
  "files": [
    "cloud_classrun.abap",
    "selection_parameters.abap",
    "switch_expression.abap"
  ]
}
```
Sorted alphabetically.

---

### 4.6 `GET /api/example?name=<filename>`

**Purpose:** Fetch the source of a named example file.  
**Called by:** `app.js` `<select id="example">` change handler; `gui.js` SE38 example selector.

**Query parameter:** `name` — bare filename; must not contain `/`, `\`, or `..`.

**Response `200`:**
```json
{
  "name": "switch_expression.abap",
  "source": "CLASS zcl_switch_demo DEFINITION..."
}
```

**Error `400` / `404`:** Same structure as `GET /api/code`.

---

### 4.7 `POST /api/compile`

**Purpose:** Compile ABAP source **without** executing it. Returns diagnostics and an IR dump.  
**Called by:**  
- `app.js` `build()` function (Build button / F6 key).  
- `gui.js` SE38 "Check" button.  
- `gui.js` "Execute" button (first pass, to detect selection-screen parameters).

**Request body:**
```json
{
  "source":   "REPORT zhello.\nWRITE 'Hello'.",
  "mode":     "standard",
  "filename": "program.abap"
}
```

| Field | Type | Required | Notes |
|---|---|---|---|
| `source` | `string` | ✓ | Full ABAP source text; empty string is accepted |
| `mode` | `"standard"` \| `"cloud"` | optional | Defaults to `"standard"` if any other value |
| `filename` | `string` | optional | Used only for archive naming on `/api/execute`; ignored here |

**Response `200`:**
```json
{
  "diagnostics": [
    {
      "severity": "error",
      "code":     "SYN001",
      "message":  "Expected END-OF-DEFINITION.",
      "start":    { "line": 3, "column": 1, "offset": 42 },
      "end":      { "line": 3, "column": 5, "offset": 46 }
    }
  ],
  "ir": {
    "vars": [
      {
        "name":      "LV_TOTAL",
        "type":      { "kind": "i" },
        "constant":  false,
        "parameter": false
      }
    ],
    "ops": [ { "kind": "push", "value": 2 }, "..." ],
    "mode": "standard",
    "entryPoint": null,
    "classes": {}
  },
  "stats": {
    "tokens":      14,
    "diagnostics": 1,
    "errors":      1,
    "warnings":    0,
    "elapsedMs":   3
  },
  "manifestCounts": {
    "functions":  0,
    "classes":    0,
    "interfaces": 0,
    "tables":     0
  }
}
```

> `ir` is `null` if lexing or parsing fails before IR lowering.  
> The IDE uses `ir.vars.filter(v => v.parameter)` to detect selection-screen parameters.

---

### 4.8 `POST /api/execute`

**Purpose:** Compile **and** run the source; archive it to `executed codes/`.  
**Called by:** `app.js` `run()` function (Run button / Ctrl+Enter / Ctrl+S); `gui.js` SE38 "Execute" button (after selection-screen parameters are collected).

**Request body:**
```json
{
  "source":          "REPORT zhello.\nWRITE 'Hello'.",
  "mode":            "cloud",
  "filename":        "program.abap",
  "parameterValues": {
    "P_DATE":    "2026-10-01",
    "P_CARRID":  [
      { "sign": "I", "option": "EQ", "low": "LH", "high": "" }
    ]
  },
  "maxSteps": 2000000
}
```

| Field | Type | Required | Notes |
|---|---|---|---|
| `source` | `string` | ✓ | |
| `mode` | `"standard"` \| `"cloud"` | optional | |
| `filename` | `string` | optional | Sanitised; becomes part of the archive filename |
| `parameterValues` | `object` | optional | Key = parameter name (case-insensitive). Value is a plain scalar for `PARAMETERS`, or an array of range-row objects for `SELECT-OPTIONS`. Missing keys default to empty/zero. |
| `maxSteps` | `number` | optional | Clamped to `[1 000 000, 10 000 000]`; default `2 000 000` |

**Range-row object structure** (inside `parameterValues` for a SELECT-OPTIONS parameter):
```json
{
  "sign":   "I",
  "option": "EQ",
  "low":    "LH",
  "high":   ""
}
```
`sign` ∈ `{I, E}` (Include/Exclude); `option` ∈ `{EQ,NE,GT,GE,LT,LE,BT,NB,CP,NP}`.

**Response `200`:**
```json
{
  "ok":     true,
  "mode":   "cloud",
  "savedFile": "executed codes/2026-11-13_00-45-12-003_cloud_program.abap",
  "diagnostics": [],
  "diagnosticText": [],
  "output": [ "Hello", "14" ],
  "variables": {
    "LV_TOTAL": 14,
    "SY": { "SUBRC": 0, "TABIX": 0, "DBCNT": 0 }
  },
  "ir": { "vars": [], "ops": [], "mode": "cloud", "entryPoint": null, "classes": {} },
  "stats": {
    "tokens": 8,
    "diagnostics": 0,
    "errors": 0,
    "warnings": 0,
    "elapsedMs": 2
  },
  "runtime": {
    "steps": 4,
    "elapsedMs": 1,
    "selectedRows": 0
  },
  "manifestCounts": { "functions": 0, "classes": 0, "interfaces": 0, "tables": 0 }
}
```

| Field | Notes |
|---|---|
| `ok` | `true` iff `stats.errors === 0` |
| `savedFile` | Relative path of the archived `.abap` file, or the best-effort path on error |
| `output` | Array of strings — one entry per `WRITE` / `out->write()` call |
| `variables` | Snapshot of the top-level variable store after execution (key = name in UPPERCASE) |
| `ir` | Same shape as `/api/compile`; `null` on hard parse failure |
| `runtime` | `null` if the program did not reach execution (compile error) |
| `diagnosticText` | Human-readable string per diagnostic: `"3:1 ERROR SYN001: ..."` |

> [!NOTE]
> When execution succeeds **and** the source contains `COMMIT ENTITIES`, the server persists
> `localPlatform` state to `runtime/local-platform-state.json`.

**Error `500`** (body read or unexpected throw):
```json
{ "error": "Invalid JSON request." }
```

---

### 4.9 `POST /api/selection-preview`

**Purpose:** Run the program up to (but not including) the `apply_parameters` VM op — this evaluates `INITIALIZATION` and any `AT SELECTION-SCREEN OUTPUT` logic to discover default values for selection-screen parameters before they are shown.  
**Called by:** `gui.js` SE38 "Execute" flow, after detecting parameters via `/api/compile`.

**Request body:** Same as `/api/compile` (`source`, `mode`, `filename`).

**Response `200`:**
```json
{
  "ok": true,
  "diagnostics": [],
  "variables": {
    "P_DATE": "2026-10-01"
  },
  "output": [],
  "stats": { "tokens": 20, "diagnostics": 0, "errors": 0, "warnings": 0, "elapsedMs": 4 }
}
```

---

### 4.10 `POST /api/selection-event`

**Purpose:** Fire an `AT SELECTION-SCREEN VALUE-REQUEST` or `AT SELECTION-SCREEN HELP-REQUEST` event for a specific parameter, to populate F4 value-help or the `?` field-help dialog.  
**Called by:** `gui.js` `openValueHelp()` and `openFieldHelp()` functions.

**Request body:**
```json
{
  "source":          "...",
  "mode":            "cloud",
  "filename":        "program.abap",
  "event":           "at-selection-screen-value-request",
  "target":          "P_DATE",
  "parameterValues": { "P_DATE": "2026-10-01" }
}
```

| Field | Type | Required | Notes |
|---|---|---|---|
| `event` | `string` | ✓ | Must be `"at-selection-screen-value-request"` or `"at-selection-screen-help-request"` |
| `target` | `string` | ✓ | Parameter name; optionally suffixed with `-LOW` or `-HIGH` for ranges |
| `parameterValues` | `object` | optional | Current UI state to pass into the event |

**Validation errors `400`:**
```json
{ "error": "Unsupported selection-screen event." }
{ "error": "Invalid selection-screen target." }
```

**Response `200`:** Same shape as `/api/selection-preview`.  
The frontend reads `response.output` as the list of F4 candidate values.

---

### 4.11 `POST /api/stop`

**Purpose:** Stub endpoint — immediately acknowledges a stop request (no-op; kept for future incremental execution support).  
**Called by:** Not called by either current frontend.

**Response `200`:**
```json
{ "ok": true }
```

---

### 4.12 `GET /api/gui/tables`

**Purpose:** Return all local fixture tables and their rows for the SE16N Table Browser.  
**Called by:** `gui.js` `getTables()` (called by SE16N, SE11, and F4 value-help).

**Response `200`:**
```json
{
  "tables": {
    "SCARR":   [ { "CARRID": "LH", "CARRNAME": "Lufthansa", "CURRCODE": "EUR" } ],
    "SPFLI":   [ { "CARRID": "LH", "CONNID": 400, "COUNTRYFR": "DE", "CITYFROM": "Frankfurt", "COUNTRYTO": "US", "CITYTO": "New York" } ],
    "SFLIGHT": [ { "CARRID": "LH", "CONNID": 400, "FLDATE": "2026-10-01", "PRICE": 520 } ],
    "ZPRODUCT":[ { "PRODUCT_ID": "P-100", "PRODUCT_NAME": "Keyboard", "CATEGORY": "ACCESSORY", "PRICE": 1200, "CURRENCY": "INR" } ]
  },
  "source":   "ABAPC local relational fixtures",
  "readOnly": true
}
```

---

### 4.13 `GET /api/platform/catalog`

**Purpose:** List all local DDIC objects (TABLs, STRUs, DTELs, DOMAs, VIEWs, BDEFs) for SE11.  
**Called by:** `gui.js` `renderDictionary()`.

**Response `200`:**
```json
{
  "objects": [
    {
      "name":        "SCARR",
      "kind":        "TABL",
      "description": "Airline master data (fictional local fixture)",
      "fields": [
        { "name": "CARRID", "type": "CHAR(3)", "key": true, "description": "Airline ID" }
      ],
      "active": true
    }
  ]
}
```

---

### 4.14 `GET /api/platform/object?name=<objectName>`

**Purpose:** Fetch a single local DDIC object by name.  
**Called by:** `gui.js` SE11 detail panel.

**Response `200`:** Single `LocalObject` (same shape as one entry in `catalog.objects`).

**Response `404`:**
```json
{ "error": "Local repository object 'ZFOO' was not found." }
```

---

### 4.15 `POST /api/platform/create-object`

**Purpose:** Create a new local DDIC training object (TABL, STRU, DTEL, or DOMA).  
**Called by:** `gui.js` SE11 "Create object" button.

**Request body:**
```json
{
  "name":        "ZMY_TABLE",
  "kind":        "TABL",
  "description": "My training table",
  "baseType":    null,
  "fields": [
    { "name": "ID",   "type": "I",       "key": true, "description": "Primary key" },
    { "name": "NAME", "type": "CHAR(40)",              "description": "Name" }
  ],
  "initialRows": [
    { "ID": 1, "NAME": "Alpha" }
  ]
}
```

| Field | Required | Notes |
|---|---|---|
| `name` | ✓ | Must match `/^[ZY][A-Z0-9_]{0,29}$/` |
| `kind` | ✓ | `TABL`, `STRU`, `DTEL`, or `DOMA` |
| `description` | optional | |
| `baseType` | optional | Used for DTEL/DOMA (e.g. `"CHAR(20)"`, `"I"`) |
| `fields` | optional | Array of field descriptors; must be non-empty for TABL/STRU |
| `initialRows` | optional | Pre-populated rows; only effective for TABL |

**Response `201`:**
```json
{
  "ok": true,
  "object": { "name": "ZMY_TABLE", "kind": "TABL", "description": "My training table", "fields": [] },
  "reported": "Local TABL object ZMY_TABLE created."
}
```

**Error `500` (via global catch):** validation errors bubble as HTTP 500 with `{ "error": "..." }`.

---

### 4.16 `DELETE /api/platform/object?name=<objectName>`

**Purpose:** Delete a local DDIC object.  
**Called by:** Not currently wired to any frontend button (available for direct calls).

**Response `200`:**
```json
{
  "ok": true,
  "object": { "name": "ZMY_TABLE", "kind": "TABL", "description": "..." },
  "reported": "Local object ZMY_TABLE deleted."
}
```

---

### 4.17 `GET /api/cds/views`

**Purpose:** List all activated local CDS view entities.  
**Called by:** `gui.js` ZCDS workbench `refresh()`.

**Response `200`:**
```json
{
  "views": [
    {
      "name":       "ZI_CARRIER_PRACTICE",
      "source":     "SCARR",
      "fields":     [ { "source": "CARRID", "alias": "CarrierId", "key": true } ],
      "annotations": [],
      "sourceText": "define view entity ZI_CARRIER_PRACTICE ...",
      "active":     true
    }
  ]
}
```

---

### 4.18 `POST /api/cds/activate`

**Purpose:** Parse and activate (or replace) a local CDS DDL definition.  
**Called by:** `gui.js` ZCDS "Activate locally" button.

**Request body:**
```json
{
  "source":  "define view entity ZI_CARRIER_PRACTICE as select from SCARR { ... }",
  "replace": true
}
```

**Response `200`:**
```json
{
  "ok":      true,
  "view":    { "name": "ZI_CARRIER_PRACTICE", "source": "SCARR", "fields": [], "active": true },
  "reported":"CDS view entity ZI_CARRIER_PRACTICE activated in the local repository."
}
```

---

### 4.19 `POST /api/cds/query`

**Purpose:** Execute a simple projection query against an activated CDS view.  
**Called by:** `gui.js` ZCDS "Preview rows" button.

**Request body:**
```json
{
  "name":   "ZI_CARRIER_PRACTICE",
  "filter": { "CarrierId": "LH" }
}
```

**Response `200`:**
```json
{
  "rows":   [ { "CarrierId": "LH", "CarrierName": "Lufthansa", "CurrencyCode": "EUR" } ],
  "source": "ABAPC local relational fixture"
}
```

---

### 4.20 `POST /api/rap/read`

**Purpose:** Read all entity rows, applying the session's staged changes (transactional view).  
**Called by:** `gui.js` ZRAP "Read entities" button.

**Request body:**
```json
{
  "entity":    "ZI_PRODUCT",
  "sessionId": "practice-abc123xyz",
  "filter":    { "CATEGORY": "ACCESSORY" }
}
```

`sessionId` must match `/^[A-Za-z0-9_-]{1,80}$/`; defaults to `"default"`.

**Response `200`:**
```json
{
  "rows": [
    { "PRODUCT_ID": "P-100", "PRODUCT_NAME": "Keyboard", "CATEGORY": "ACCESSORY", "PRICE": 1200, "CURRENCY": "INR" }
  ],
  "pending": [
    { "kind": "CREATE", "key": "P-400", "data": { "PRODUCT_ID": "P-400", "PRICE": 2499 } }
  ]
}
```

---

### 4.21 `POST /api/rap/modify`

**Purpose:** Stage a CREATE, UPDATE, or DELETE operation in the session buffer.  
**Called by:** `gui.js` ZRAP "Stage CREATE / UPDATE / DELETE" buttons.

**Request body:**
```json
{
  "entity":    "ZI_PRODUCT",
  "sessionId": "practice-abc123xyz",
  "operation": "CREATE",
  "data": {
    "PRODUCT_ID":   "P-400",
    "PRODUCT_NAME": "USB-C Hub",
    "CATEGORY":     "ACCESSORY",
    "PRICE":        2499,
    "CURRENCY":     "INR"
  }
}
```

`operation` must be `CREATE`, `UPDATE`, or `DELETE`.

**Response `200`:**
```json
{
  "ok":      true,
  "reported":"CREATE staged for P-400.",
  "pending": [ { "kind": "CREATE", "key": "P-400", "data": { "PRODUCT_ID": "P-400" } } ]
}
```

---

### 4.22 `POST /api/rap/commit`

**Purpose:** Apply all staged operations atomically to the local table, then persist state.  
**Called by:** `gui.js` ZRAP "Commit entities" button.

**Request body:**
```json
{ "sessionId": "practice-abc123xyz" }
```

**Response `200`:**
```json
{
  "ok":      true,
  "reported":"3 operation(s) committed.",
  "committed":3,
  "failed":  0
}
```

---

### 4.23 `POST /api/rap/rollback`

**Purpose:** Discard all staged operations in the session buffer (does not touch the table).  
**Called by:** `gui.js` ZRAP "Rollback entities" button.

**Request body:**
```json
{ "sessionId": "practice-abc123xyz" }
```

**Response `200`:**
```json
{
  "ok":      true,
  "reported":"1 pending operation(s) discarded.",
  "discarded":1
}
```

---

## 5. Uniform Error Envelope

All unhandled exceptions inside the request handler are caught by the global `try/catch` and returned as:

```
HTTP 500
Content-Type: application/json
```
```json
{ "error": "<exception message string>" }
```

Validation errors specific to an endpoint also use `{ "error": "..." }` with an appropriate 4xx status code.

---

## 6. Frontend API Call Inventory

### 6.1 `app.js` (Compiler IDE — `/`)

| Trigger | Method + URL | Payload |
|---|---|---|
| Page load | `GET /api/executed` | — |
| Page load | `GET /api/examples` | — |
| Page load | `GET /api/config` | — |
| History item click | `GET /api/code?name=<filename>` | — |
| Example selector change | `GET /api/example?name=<name>` | — |
| Build button / F6 | `POST /api/compile` | `{source, mode, filename}` |
| Run button / Ctrl+Enter | `POST /api/execute` | `{source, mode, filename}` |
| ↻ Refresh history | `GET /api/executed` | — |

Transport helper (defined in `app.js`):
```js
fetch(url, { method:'POST', headers:{'Content-Type':'application/json'}, body:JSON.stringify(body) })
```

### 6.2 `gui.js` (SAP GUI Simulator — `/sapgui`)

| Trigger | Method + URL | Payload |
|---|---|---|
| Page load IIFE | `GET /api/config` | — |
| Page load IIFE | `GET /api/examples` | — |
| SE38 load | `GET /api/example?name=<preset>` | — |
| SE38 "Check" | `POST /api/compile` | `{source, mode, filename}` |
| SE38 "Execute" (pass 1 — detect params) | `POST /api/compile` | `{source, mode, filename}` |
| SE38 "Execute" (pass 2 — get defaults) | `POST /api/selection-preview` | `{source, mode, filename}` |
| SE38 "Execute" (pass 3 — run with values) | `POST /api/execute` | `{source, mode, filename, parameterValues}` |
| F4 value-help button | `POST /api/selection-event` | `{source, mode, filename, event:'at-selection-screen-value-request', target, parameterValues}` |
| `?` field-help button | `POST /api/selection-event` | `{source, mode, filename, event:'at-selection-screen-help-request', target, parameterValues:{}}` |
| SE16N / F4 lookup | `GET /api/gui/tables` | — |
| SE11 load | `GET /api/platform/catalog` | — |
| SE11 object select | `GET /api/platform/object?name=<name>` | — |
| SE11 "Create object" | `POST /api/platform/create-object` | `{name, kind, description, baseType, fields, initialRows}` |
| ZCDS load / refresh | `GET /api/cds/views` | — |
| ZCDS "Activate" | `POST /api/cds/activate` | `{source, replace:true}` |
| ZCDS "Preview rows" | `POST /api/cds/query` | `{name, filter:{}}` |
| ZRAP load / "Read entities" | `POST /api/rap/read` | `{entity, sessionId, filter:{}}` |
| ZRAP "Stage CREATE" | `POST /api/rap/modify` | `{entity, sessionId, operation:'CREATE', data}` |
| ZRAP "Stage UPDATE" | `POST /api/rap/modify` | `{entity, sessionId, operation:'UPDATE', data}` |
| ZRAP "Stage DELETE" | `POST /api/rap/modify` | `{entity, sessionId, operation:'DELETE', data}` |
| ZRAP "Commit" | `POST /api/rap/commit` | `{sessionId}` |
| ZRAP "Rollback" | `POST /api/rap/rollback` | `{sessionId}` |
| SM37 load | `GET /api/executed` | — |
| SM37 "Open in editor" | `GET /api/code?name=<filename>` | — |

Transport helpers (defined in `gui.js`):
```js
async function jsonGet(url)  { const r = await fetch(url); ... }
async function jsonPost(url, body) { const r = await fetch(url, {method:'POST', headers:{'Content-Type':'application/json'}, body:JSON.stringify(body)}); ... }
```

Both throw an `Error` with `d.error || "Request failed (<status>)"` if `!r.ok`.

---

## 7. State Management

### 7.1 `app.js` — No persistent state object

All state is held as plain DOM values (`textarea#editor`, `select#mode`, `input#filename`). There is no Redux, no store, and no session storage usage. The only "state" that persists across page loads is the execution archive on disk.

### 7.2 `gui.js` — Explicit `state` object + Web Storage

```js
const state = {
  current:          'S000',      // Active transaction code
  stack:            [],          // Navigation back-stack
  recent:           [],          // Last 8 transaction codes (← localStorage)
  tables:           {},          // Cached GET /api/gui/tables response
  metadata:         {},          // Not currently populated
  examples:         [],          // Cached GET /api/examples list
  executions:       [],          // Cached GET /api/executed list
  lastDiagnostics:  [],          // Last execution's diagnostic array (← sessionStorage)
  lastOutput:       [],          // Last execution's output lines (← sessionStorage)
  editorPreset:     'cloud_classrun.abap',
  pendingSource:    null,        // Source to load on next SE38 render
  editorFilename:   'program.abap',
};
```

| Key | Storage | Contents |
|---|---|---|
| `abapc-recent-tx` | `localStorage` | JSON array of last 8 transaction codes |
| `abapc-last-diagnostics` | `sessionStorage` | JSON array of last `Diagnostic[]` |
| `abapc-last-output` | `sessionStorage` | JSON array of last output strings |
| `abapc-editor-draft` | `localStorage` | `{ source, filename, mode, savedAt }` |
| `abapc-practice-done` | `localStorage` | `{ [taskId]: boolean }` |
| `abapc-rap-session` | `sessionStorage` | Random session ID for ZRAP EML workbench |

---

## 8. Backend Parser / Compiler Pipeline

The compilation pipeline is defined in `src/compiler.ts` → `compileSource()`.

```
                           compileSource(source, options)
                                        │
                    ┌───────────────────▼──────────────────────┐
                    │  1. Lexer  (src/lexer.ts)                 │
                    │     Tokenises the raw ABAP text.          │
                    │     Produces: tokens[], diagnostics[]     │
                    └───────────────────┬──────────────────────┘
                              hasErrors │ no → continue
                    ┌───────────────────▼──────────────────────┐
                    │  2. Parser  (src/parser.ts)               │
                    │     Builds a typed AST (Program node).    │
                    │     Produces: program, diagnostics[]      │
                    └───────────────────┬──────────────────────┘
                              hasErrors │ no → continue
                    ┌───────────────────▼──────────────────────┐
                    │  3. Policy Validator  (src/policy.ts)     │
                    │     Checks ABAP Cloud vs Standard rules   │
                    │     (e.g. forbidden keywords).            │
                    └───────────────────┬──────────────────────┘
                    ┌───────────────────▼──────────────────────┐
                    │  4. SemanticAnalyzer  (src/semantic.ts)   │
                    │     Type resolution, scope, symbol check. │
                    │     Produces: diagnostics[]               │
                    └───────────────────┬──────────────────────┘
                              hasErrors │ no → continue
                    ┌───────────────────▼──────────────────────┐
                    │  5. Lowerer  (src/ir.ts)                  │
                    │     AST → ProgramIR (flat op-code list).  │
                    │     Handles class/method lowering,        │
                    │     selection-screen events, cloud entry. │
                    └───────────────────┬──────────────────────┘
                           run===false  │ or previewSelection
                    ┌───────────────────▼──────────────────────┐
                    │  6. VM  (src/vm.ts)                       │
                    │     Stack-based interpreter executes ops. │
                    │     In-memory DB: SCARR / SPFLI /         │
                    │       SFLIGHT / ZPRODUCT (VM_DATABASE).   │
                    │     EML ops call localPlatform directly.  │
                    │     Produces: VMResult {output, variables,│
                    │       steps, elapsedMs, selectedRows}     │
                    └───────────────────────────────────────────┘
```

### Error Trapping

| Stage | Error type | Diagnostic code example | Behaviour |
|---|---|---|---|
| Lexer | Illegal character, unterminated string | `LEX001` | Added to `DiagnosticBag`; parsing skipped |
| Parser | Syntax error | `SYN001` | Added to bag; IR/VM skipped |
| Policy | Cloud-forbidden keyword | `POL001` | Added to bag |
| Semantic | Undefined symbol, type mismatch | `SEM001` | Added to bag; IR/VM skipped |
| VM (runtime) | Step limit, division by zero, unhandled | `RUN001` | Caught; pushed to bag as `severity:"error"` |

All diagnostics share the same `Diagnostic` interface:
```ts
{ severity: "error" | "warning" | "info";
  code:     string;      // e.g. "SYN001"
  message:  string;
  start:    { line, column, offset };
  end:      { line, column, offset }; }
```

### `compilePayload` Wrapper (`server.mjs`)

`server.mjs` wraps `compileSource` in `compilePayload()` which:
1. Reads `runtime/released-apis.json` on every call (no caching).
2. Injects `localPlatform.getCompilerTypes()` as `externalTypes`.
3. Clamps `maxSteps` to `[1 000 000, 10 000 000]`.
4. Appends `manifestCounts` to the result.

After `/api/execute`, if `stats.errors === 0` and the source contains `COMMIT ENTITIES`,
`savePlatformState()` is called to persist the `localPlatform` singleton to disk.

---

## 9. Local Platform (`localPlatform` singleton)

Defined in `src/local-platform.ts`. Manages:

| Subsystem | Description |
|---|---|
| **Tables** | 4 built-in fixture tables + user-created TABLs. Mutable via RAP/EML. |
| **Custom DDIC objects** | TABLs, STRUs, DTELs, DOMAs created via `/api/platform/create-object`. |
| **CDS Views** | Activated in-memory projection views (`ZI_*`). |
| **RAP Sessions** | Per-session staged operation queues (key: `sessionId`). |
| **Compiler types** | `getCompilerTypes()` returns all custom objects as `TypeRef` entries injected into the compiler. |
| **State persistence** | `exportState()` / `importState()` serialise to/from `runtime/local-platform-state.json`. |

---

## 10. Diagnostic Severity Model

```
severity = "error"   → stats.errors++   → ok = false → program did not execute
severity = "warning" → stats.warnings++ → ok may still be true
severity = "info"    → not currently generated by the compiler
```

---

## 11. Files & Their Roles (Summary)

| File | Role |
|---|---|
| `server.mjs` | HTTP server, router, compilePayload orchestrator, static file server |
| `app/index.html` | Compiler IDE shell |
| `app/app.js` | Compiler IDE logic — Build, Run, history, examples |
| `app/gui.html` | SAP GUI Practice Simulator shell |
| `app/gui.js` | All SAP GUI transaction screens + all API calls for the simulator |
| `src/lexer.ts` | ABAP tokeniser |
| `src/tokens.ts` | Token type definitions |
| `src/parser.ts` | Recursive-descent parser → AST |
| `src/ast.ts` | AST node type definitions |
| `src/semantic.ts` | Symbol resolution, type checking |
| `src/policy.ts` | Cloud vs Standard ABAP language mode enforcement |
| `src/ir.ts` | AST-to-IR lowerer; IR op-code definitions; `ProgramIR` type |
| `src/vm.ts` | Stack-based virtual machine; built-in `VM_DATABASE` |
| `src/diagnostics.ts` | `Diagnostic` type, `DiagnosticBag`, `formatDiagnostic` |
| `src/local-platform.ts` | DDIC / CDS / RAP in-memory simulation; state persistence |
| `src/compiler.ts` | Public `compileSource()` orchestrator |
| `src/cli.ts` | CLI wrapper (not used by the server) |
| `runtime/released-apis.json` | ABAP Cloud released-API manifest (empty by default) |
| `runtime/local-platform-state.json` | Persisted table/CDS/RAP state |
| `dist/src/` | Compiled JS output of `src/` (TypeScript → JS via `tsc`) |
