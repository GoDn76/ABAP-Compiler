/**
 * ABAPC Studio 5.0.1 - Stateless Execution Playground & React IDE
 * 100% Client-Side Virtual File System (localStorage) with 1500ms Debounced Auto-Save
 * Ephemeral Execution via POST /api/run (AutoCleanFile RAII Drop Guard)
 * Bring Your Own Database (BYODB) Scoped Connection Architecture
 */

(function () {
  "use strict";

  // Self-Contained Resilient React 18 Engine
  // If React & ReactDOM are not preloaded by CDN, provide the built-in micro-reactive engine
  let React = window.React;
  let ReactDOM = window.ReactDOM;

  if (!React || !ReactDOM) {
    const hooks = [];
    let hookIdx = 0;
    let rootContainer = null;
    let rootComp = null;
    let renderScheduled = false;
    let pendingEffects = [];
    let currentVNode = null;

    function scheduleRender() {
      if (renderScheduled) return;
      renderScheduled = true;
      requestAnimationFrame(runRender);
    }

    function runRender() {
      renderScheduled = false;
      if (!rootContainer || !rootComp) return;
      hookIdx = 0;
      pendingEffects = [];
      const newVNode = rootComp();
      patch(rootContainer, currentVNode, newVNode, 0);
      currentVNode = newVNode;
      for (const eff of pendingEffects) {
        try { eff(); } catch (e) { console.error("Effect error:", e); }
      }
    }

    function flatten(children) {
      const res = [];
      for (const c of children) {
        if (Array.isArray(c)) {
          res.push(...flatten(c));
        } else if (c !== null && c !== undefined && c !== false) {
          res.push(c);
        }
      }
      return res;
    }

    React = {
      createElement: function (type, props, ...children) {
        return {
          type,
          props: props || {},
          children: flatten(children)
        };
      },
      useState: function (initial) {
        const i = hookIdx++;
        if (hooks[i] === undefined) {
          hooks[i] = typeof initial === "function" ? initial() : initial;
        }
        const setState = (newVal) => {
          const next = typeof newVal === "function" ? newVal(hooks[i]) : newVal;
          if (next !== hooks[i]) {
            hooks[i] = next;
            scheduleRender();
          }
        };
        return [hooks[i], setState];
      },
      useRef: function (initial) {
        const i = hookIdx++;
        if (hooks[i] === undefined) {
          hooks[i] = { current: initial };
        }
        return hooks[i];
      },
      useCallback: function (fn, deps) {
        const i = hookIdx++;
        const prev = hooks[i];
        if (!prev || !deps || deps.some((d, idx) => d !== prev.deps[idx])) {
          hooks[i] = { fn, deps };
          return fn;
        }
        return prev.fn;
      },
      useMemo: function (fn, deps) {
        const i = hookIdx++;
        const prev = hooks[i];
        if (!prev || !deps || deps.some((d, idx) => d !== prev.deps[idx])) {
          hooks[i] = { val: fn(), deps };
          return hooks[i].val;
        }
        return prev.val;
      },
      useEffect: function (fn, deps) {
        const i = hookIdx++;
        const prev = hooks[i];
        const changed = !prev || !deps || deps.some((d, idx) => d !== prev.deps[idx]);
        if (changed) {
          pendingEffects.push(() => {
            if (prev && typeof prev.cleanup === "function") {
              try { prev.cleanup(); } catch (e) { console.error(e); }
            }
            const cleanup = fn();
            hooks[i] = { deps, cleanup };
          });
        }
      }
    };

    function updateDOMProps(dom, oldProps, newProps) {
      dom._prevProps = newProps;
      if (dom._listeners) {
        for (const [evt, handler] of Object.entries(dom._listeners)) {
          dom.removeEventListener(evt, handler);
        }
      }
      dom._listeners = {};

      for (const [key, value] of Object.entries(newProps || {})) {
        if (key.startsWith("on") && typeof value === "function") {
          const evt = key.slice(2).toLowerCase();
          dom.addEventListener(evt, value);
          dom._listeners[evt] = value;
        } else if (key === "className") {
          dom.className = value || "";
        } else if (key === "style" && typeof value === "object") {
          Object.assign(dom.style, value);
        } else if (key === "value" && (dom.tagName === "INPUT" || dom.tagName === "TEXTAREA" || dom.tagName === "SELECT")) {
          if (dom.value !== String(value ?? "")) {
            dom.value = value ?? "";
          }
        } else if (key === "checked" && dom.tagName === "INPUT") {
          dom.checked = !!value;
        } else if (key === "disabled") {
          dom.disabled = !!value;
        } else if (key !== "children" && key !== "key" && key !== "ref") {
          if (value !== null && value !== undefined && value !== false) {
            dom.setAttribute(key, value === true ? "" : String(value));
          } else {
            dom.removeAttribute(key);
          }
        }
      }

      if (newProps && newProps.ref && typeof newProps.ref === "object") {
        newProps.ref.current = dom;
      }
    }

    function patch(parent, oldVNode, newVNode, index = 0) {
      const oldDOM = parent.childNodes[index];

      if (!newVNode) {
        if (oldDOM) parent.removeChild(oldDOM);
        return null;
      }

      if (typeof newVNode === "string" || typeof newVNode === "number") {
        const text = String(newVNode);
        if (oldDOM && oldDOM.nodeType === 3) {
          if (oldDOM.nodeValue !== text) oldDOM.nodeValue = text;
          return oldDOM;
        }
        const textDOM = document.createTextNode(text);
        if (oldDOM) parent.replaceChild(textDOM, oldDOM);
        else parent.appendChild(textDOM);
        return textDOM;
      }

      if (!oldDOM || oldDOM.nodeType !== 1 || oldDOM.tagName.toLowerCase() !== newVNode.type.toLowerCase()) {
        const newDOM = document.createElement(newVNode.type);
        updateDOMProps(newDOM, {}, newVNode.props);
        for (let i = 0; i < newVNode.children.length; i++) {
          patch(newDOM, null, newVNode.children[i], i);
        }
        if (oldDOM) parent.replaceChild(newDOM, oldDOM);
        else parent.appendChild(newDOM);
        return newDOM;
      }

      // Update existing DOM element
      updateDOMProps(oldDOM, oldDOM._prevProps || {}, newVNode.props);

      const oldLen = oldDOM.childNodes.length;
      const newLen = newVNode.children.length;
      const maxLen = Math.max(oldLen, newLen);

      for (let i = maxLen - 1; i >= newLen; i--) {
        if (oldDOM.childNodes[i]) oldDOM.removeChild(oldDOM.childNodes[i]);
      }
      for (let i = 0; i < newLen; i++) {
        patch(oldDOM, null, newVNode.children[i], i);
      }

      return oldDOM;
    }

    ReactDOM = {
      createRoot: function (container) {
        rootContainer = container;
        return {
          render: function (vnode) {
            container.innerHTML = "";
            currentVNode = null;
            if (typeof vnode.type === "function") {
              rootComp = vnode.type;
              runRender();
            } else {
              patch(container, null, vnode, 0);
            }
          }
        };
      }
    };
  }

  const { useState, useEffect, useRef, useCallback, useMemo } = React;
  const h = React.createElement;

  // ==========================================================================
  // DEFAULT STARTER FILES (Virtual File System Seed)
  // ==========================================================================
  const DEFAULT_VFS_FILES = {
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

  const SAMPLE_POSTGRES_DDL = `-- Execute this DDL in your Supabase or Neon SQL Editor:
CREATE TABLE IF NOT EXISTS sflight (
  carrid VARCHAR(10) NOT NULL,
  connid INTEGER NOT NULL,
  fldate DATE DEFAULT CURRENT_DATE,
  price NUMERIC(10,2) NOT NULL,
  currency VARCHAR(5) DEFAULT 'USD',
  PRIMARY KEY (carrid, connid)
);

INSERT INTO sflight (carrid, connid, fldate, price, currency) VALUES
  ('LH', 400, '2026-10-01', 520.00, 'EUR'),
  ('LH', 401, '2026-10-02', 610.00, 'EUR'),
  ('SQ', 88,  '2026-10-03', 430.00, 'USD')
ON CONFLICT (carrid, connid) DO NOTHING;`;

  // ==========================================================================
  // ROOT REACT COMPONENT: AbapcStudioApp
  // ==========================================================================
  function AbapcStudioApp() {
    // 1. Virtual File System (localStorage)
    const [files, setFiles] = useState(() => {
      try {
        const saved = localStorage.getItem("abapc_vfs_files");
        if (saved) {
          const parsed = JSON.parse(saved);
          if (parsed && typeof parsed === "object" && Object.keys(parsed).length > 0) {
            let updated = false;
            if (!parsed["zcl_cloud_classrun.abap"]) {
              parsed["zcl_cloud_classrun.abap"] = DEFAULT_VFS_FILES["zcl_cloud_classrun.abap"];
              updated = true;
            }
            if (!parsed["zcl_demo_cloud.abap"]) {
              parsed["zcl_demo_cloud.abap"] = DEFAULT_VFS_FILES["zcl_demo_cloud.abap"];
              updated = true;
            }
            if (updated) localStorage.setItem("abapc_vfs_files", JSON.stringify(parsed));
            return parsed;
          }
        }
      } catch (e) {
        console.warn("Error reading VFS from localStorage", e);
      }
      return DEFAULT_VFS_FILES;
    });

    const [activeFileName, setActiveFileName] = useState(() => {
      try {
        const saved = localStorage.getItem("abapc_active_file");
        if (saved && files[saved]) return saved;
      } catch {}
      return Object.keys(files)[0] || "z_stateless_main.abap";
    });

    const [editorContent, setEditorContent] = useState(() => files[activeFileName] || "");
    const [saveStatus, setSaveStatus] = useState("saved"); // "saved" | "saving" | "unsaved"
    const [autoSaveEnabled, setAutoSaveEnabled] = useState(() => {
      try {
        const saved = localStorage.getItem("abapc_autosave_enabled");
        return saved !== null ? saved === "true" : true;
      } catch {
        return true;
      }
    });

    // 2. BYODB Configuration (localStorage)
    const [dbUrl, setDbUrl] = useState(() => {
      try {
        return localStorage.getItem("abapc_byodb_url") || "";
      } catch {
        return "";
      }
    });
    const [showDbPassword, setShowDbPassword] = useState(false);
    const [dbTestResult, setDbTestResult] = useState(null);

    // 3. Modals and Tabs
    const [showDbModal, setShowDbModal] = useState(false);
    const [showGuideModal, setShowGuideModal] = useState(false);
    const [guideTab, setGuideTab] = useState("architecture"); // "architecture" | "supabase_neon" | "opensql_demo"
    const [activeInspectorTab, setActiveInspectorTab] = useState("output"); // "output" | "diagnostics" | "variables" | "ir"

    // 4. Execution State
    const [compilerMode, setCompilerMode] = useState("standard"); // "standard" | "cloud"
    const [isRunning, setIsRunning] = useState(false);
    const [rateLimitTimer, setRateLimitTimer] = useState(null);
    const [executionResult, setExecutionResult] = useState(null);
    const [errorMessage, setErrorMessage] = useState(null);

    // 5. Sidebar Helpers
    const [fileSearch, setFileSearch] = useState("");
    const [newFileName, setNewFileName] = useState("");
    const [isCreatingFile, setIsCreatingFile] = useState(false);

    // Refs
    const debounceTimerRef = useRef(null);
    const editorRef = useRef(null);
    const gutterRef = useRef(null);

    // Debounced Auto-Save Handler (1500ms)
    const persistActiveFile = useCallback((fileName, content) => {
      setFiles((prev) => {
        const next = Object.assign({}, prev, { [fileName]: content });
        try {
          localStorage.setItem("abapc_vfs_files", JSON.stringify(next));
        } catch (e) {
          console.error("VFS localStorage save error", e);
        }
        return next;
      });
      setSaveStatus("saved");
    }, []);

    const handleManualSave = useCallback(() => {
      if (debounceTimerRef.current) clearTimeout(debounceTimerRef.current);
      persistActiveFile(activeFileName, editorContent);
    }, [activeFileName, editorContent, persistActiveFile]);

    const toggleAutoSave = () => {
      setAutoSaveEnabled((prev) => {
        const next = !prev;
        try {
          localStorage.setItem("abapc_autosave_enabled", String(next));
        } catch {}
        if (!next && debounceTimerRef.current) {
          clearTimeout(debounceTimerRef.current);
        } else if (next) {
          persistActiveFile(activeFileName, editorContent);
        }
        return next;
      });
    };

    const handleEditorChange = (e) => {
      const val = e.target.value;
      setEditorContent(val);

      if (autoSaveEnabled) {
        setSaveStatus("saving");
        if (debounceTimerRef.current) {
          clearTimeout(debounceTimerRef.current);
        }
        debounceTimerRef.current = setTimeout(() => {
          persistActiveFile(activeFileName, val);
        }, 1500);
      } else {
        setSaveStatus("unsaved");
      }
    };

    // Switching files flushes immediately
    const switchActiveFile = (newFile) => {
      if (newFile === activeFileName) return;
      if (debounceTimerRef.current) {
        clearTimeout(debounceTimerRef.current);
        persistActiveFile(activeFileName, editorContent);
      }
      setActiveFileName(newFile);
      setEditorContent(files[newFile] || "");
      setSaveStatus("saved");
      try {
        localStorage.setItem("abapc_active_file", newFile);
      } catch {}
    };

    // Window beforeunload flush
    useEffect(() => {
      const handleBeforeUnload = () => {
        if (debounceTimerRef.current) {
          clearTimeout(debounceTimerRef.current);
          try {
            const updated = Object.assign({}, files, { [activeFileName]: editorContent });
            localStorage.setItem("abapc_vfs_files", JSON.stringify(updated));
          } catch {}
        }
      };
      window.addEventListener("beforeunload", handleBeforeUnload);
      return () => window.removeEventListener("beforeunload", handleBeforeUnload);
    }, [files, activeFileName, editorContent]);

    // Rate Limiter Timer Countdown
    useEffect(() => {
      if (rateLimitTimer === null || rateLimitTimer <= 0) return;
      const interval = setInterval(() => {
        setRateLimitTimer((prev) => {
          if (prev <= 1) {
            clearInterval(interval);
            return null;
          }
          return prev - 1;
        });
      }, 1000);
      return () => clearInterval(interval);
    }, [rateLimitTimer]);

    // Editor scroll and gutter sync
    const lines = editorContent.split("\n");
    const lineCount = lines.length;

    const handleScroll = () => {
      if (editorRef.current && gutterRef.current) {
        gutterRef.current.scrollTop = editorRef.current.scrollTop;
      }
    };

    // Keyboard shortcuts: Ctrl+Enter, F6, Tab indent
    const handleKeyDown = (e) => {
      if ((e.ctrlKey || e.metaKey) && e.key === "Enter") {
        e.preventDefault();
        runCode();
      } else if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "s") {
        e.preventDefault();
        handleManualSave();
      } else if (e.key === "F6") {
        e.preventDefault();
        runCode();
      } else if (e.key === "Tab") {
        e.preventDefault();
        const textarea = editorRef.current;
        const start = textarea.selectionStart;
        const end = textarea.selectionEnd;
        const updated = editorContent.substring(0, start) + "  " + editorContent.substring(end);
        setEditorContent(updated);
        setSaveStatus("saving");
        setTimeout(() => {
          textarea.selectionStart = textarea.selectionEnd = start + 2;
        }, 0);
        if (debounceTimerRef.current) clearTimeout(debounceTimerRef.current);
        debounceTimerRef.current = setTimeout(() => {
          persistActiveFile(activeFileName, updated);
        }, 1500);
      }
    };

    // File Management Functions
    const handleCreateFile = () => {
      let name = newFileName.trim();
      if (!name) return;
      if (!name.endsWith(".abap")) {
        name += ".abap";
      }
      if (files[name]) {
        alert("File '" + name + "' already exists!");
        return;
      }
      const starter = "REPORT " + name.replace(".abap", "") + ".\n\nWRITE: / 'File: " + name + "'.\n";
      const updated = Object.assign({}, files, { [name]: starter });
      setFiles(updated);
      try {
        localStorage.setItem("abapc_vfs_files", JSON.stringify(updated));
      } catch {}
      setNewFileName("");
      setIsCreatingFile(false);
      switchActiveFile(name);
    };

    const handleDeleteFile = (fileName, e) => {
      e.stopPropagation();
      const keys = Object.keys(files);
      if (keys.length <= 1) {
        alert("Cannot delete the only remaining file in your Virtual File System.");
        return;
      }
      if (!confirm("Are you sure you want to delete '" + fileName + "'?")) return;

      const updated = Object.assign({}, files);
      delete updated[fileName];
      setFiles(updated);
      try {
        localStorage.setItem("abapc_vfs_files", JSON.stringify(updated));
      } catch {}

      if (fileName === activeFileName) {
        const remainingKeys = Object.keys(updated);
        switchActiveFile(remainingKeys[0]);
      }
    };

    const handleResetVFS = () => {
      if (!confirm("Reset Virtual File System to default example files? Your current edits will be replaced.")) return;
      setFiles(DEFAULT_VFS_FILES);
      const first = Object.keys(DEFAULT_VFS_FILES)[0];
      setActiveFileName(first);
      setEditorContent(DEFAULT_VFS_FILES[first]);
      try {
        localStorage.setItem("abapc_vfs_files", JSON.stringify(DEFAULT_VFS_FILES));
        localStorage.setItem("abapc_active_file", first);
      } catch {}
    };

    // BYODB Functions
    const handleSaveDbUrl = (url) => {
      setDbUrl(url);
      try {
        localStorage.setItem("abapc_byodb_url", url);
      } catch {}
    };

    const handleTestDbConnection = async () => {
      setDbTestResult({ testing: true, message: "Testing scoped connection to PostgreSQL..." });
      try {
        const res = await fetch("/api/run", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({
            code: "REPORT z_db_ping.\nWRITE 'DB_PING_OK'.",
            db_url: dbUrl.trim() || null,
            mode: compilerMode,
            filename: "z_db_ping.abap"
          })
        });
        const data = await res.json();
        if (res.ok && data.database_connected) {
          setDbTestResult({
            success: true,
            message: "Connected successfully! Engine: " + data.database_type
          });
        } else {
          setDbTestResult({
            success: false,
            message: "Unable to connect: " + (data.database_type || "Scoped query failed. Local mock active.")
          });
        }
      } catch (e) {
        setDbTestResult({
          success: false,
          message: "Network or server error: " + String(e)
        });
      }
    };

    // ========================================================================
    // EXECUTION HANDLER: POST /api/run
    // ========================================================================
    const runCode = async () => {
      if (isRunning || rateLimitTimer !== null) return;

      // Flush any pending edits immediately
      if (debounceTimerRef.current) {
        clearTimeout(debounceTimerRef.current);
        persistActiveFile(activeFileName, editorContent);
      }

      setIsRunning(true);
      setErrorMessage(null);
      setActiveInspectorTab("output");

      try {
        const response = await fetch("/api/run", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({
            code: editorContent,
            db_url: dbUrl.trim() || null,
            mode: compilerMode,
            filename: activeFileName
          })
        });

        if (response.status === 429) {
          const errorJson = await response.json();
          const retryAfter = errorJson.retry_after || 60;
          setRateLimitTimer(retryAfter);
          setErrorMessage(
            "429 Rate Limit Reached: " +
              (errorJson.message || "Exceeded 3 requests/minute per IP.") +
              " Please wait " +
              retryAfter +
              "s."
          );
          setActiveInspectorTab("diagnostics");
          return;
        }

        if (response.status === 408) {
          const errorJson = await response.json();
          setErrorMessage(
            "408 Execution Timeout: " +
              (errorJson.message || "Execution exceeded 5-second limit; possible infinite loop.")
          );
          setActiveInspectorTab("diagnostics");
          return;
        }

        if (!response.ok) {
          const errorJson = await response.json().catch(() => ({}));
          throw new Error(errorJson.error || errorJson.message || "HTTP " + response.status);
        }

        const data = await response.json();
        setExecutionResult(data);

        if (!data.ok && data.diagnostics && data.diagnostics.length > 0) {
          setActiveInspectorTab("diagnostics");
        }
      } catch (err) {
        setErrorMessage("Execution error: " + (err.message || String(err)));
        setActiveInspectorTab("diagnostics");
      } finally {
        setIsRunning(false);
      }
    };

    // Storage calculation
    const vfsSizeKB = (JSON.stringify(files).length / 1024).toFixed(1);

    // Filtered file keys
    const filteredFileNames = Object.keys(files).filter((f) =>
      f.toLowerCase().includes(fileSearch.toLowerCase())
    );

    // ========================================================================
    // RENDER TREE
    // ========================================================================
    return h(
      "div",
      { className: "abapc-container" },

      // 1. TOPBAR
      h(
        "header",
        { className: "abapc-topbar" },
        h(
          "div",
          { className: "abapc-brand" },
          h("div", { className: "abapc-logo" }, "A"),
          h(
            "div",
            null,
            h(
              "div",
              { className: "brand-title" },
              "ABAPC Studio ",
              h("span", { className: "version-pill" }, "v5.0.1")
            ),
            h(
              "div",
              { className: "brand-subtitle" },
              h("span", { className: "stateless-badge" }, "100% Stateless • BYODB"),
              " Local ABAP + Cloud Compiler"
            )
          )
        ),
        h(
          "div",
          { className: "abapc-actions" },
          // SAP GUI link
          h(
            "a",
            {
              className: "btn-secondary btn-sapgui",
              href: "/sapgui",
              title: "Open SAP GUI-style transaction workbench"
            },
            "▦ Transaction GUI"
          ),
          // BYODB Status Pill
          h(
            "button",
            {
              className: "btn-db-indicator " + (dbUrl.trim() ? "active" : "mock"),
              onClick: () => setShowDbModal(true),
              title: "Configure Bring Your Own Database (PostgreSQL)"
            },
            dbUrl.trim()
              ? [h("span", { key: "d", className: "dot dot-green" }), h("span", { key: "t" }, "PostgreSQL (BYODB)")]
              : [h("span", { key: "d", className: "dot dot-amber" }), h("span", { key: "t" }, "Local Mock Engine")]
          ),
          // Guide Button
          h(
            "button",
            {
              className: "btn-secondary",
              onClick: () => setShowGuideModal(true),
              title: "Open In-App BYODB & Architecture Guide"
            },
            "📖 Architecture & SQL Guide"
          ),
          // Language Mode
          h(
            "select",
            {
              className: "select-mode",
              value: compilerMode,
              onChange: (e) => setCompilerMode(e.target.value),
              title: "Compiler language mode"
            },
            h("option", { value: "standard" }, "Standard ABAP"),
            h("option", { value: "cloud" }, "ABAP Cloud (Strict Release APIs)")
          ),
          // Run Button
          h(
            "button",
            {
              className: "btn-primary",
              onClick: runCode,
              disabled: isRunning || rateLimitTimer !== null,
              title: "Run active file (Ctrl+Enter)"
            },
            isRunning
              ? "⏳ Running..."
              : rateLimitTimer !== null
              ? "⏳ Rate Limited (" + rateLimitTimer + "s)"
              : "▶ Run Code"
          )
        )
      ),

      // 2. MAIN WORKSPACE
      h(
        "main",
        { className: "abapc-workspace" },

        // LEFT SIDEBAR: VFS
        h(
          "aside",
          { className: "abapc-sidebar" },
          h(
            "div",
            { className: "sidebar-head" },
            h(
              "div",
              { className: "sidebar-title" },
              h("span", null, "VIRTUAL FILES"),
              h("span", { className: "file-count-badge" }, Object.keys(files).length)
            ),
            h(
              "button",
              {
                className: "btn-icon",
                onClick: () => setIsCreatingFile(!isCreatingFile),
                title: "Create new .abap file"
              },
              "+"
            )
          ),

          // Inline New File Form
          isCreatingFile &&
            h(
              "div",
              { className: "new-file-box" },
              h("input", {
                type: "text",
                className: "input-new-file",
                placeholder: "z_new_prog.abap",
                value: newFileName,
                onChange: (e) => setNewFileName(e.target.value),
                onKeyDown: (e) => {
                  if (e.key === "Enter") handleCreateFile();
                  if (e.key === "Escape") setIsCreatingFile(false);
                },
                autoFocus: true
              }),
              h(
                "div",
                { className: "new-file-actions" },
                h("button", { className: "btn-small-primary", onClick: handleCreateFile }, "Add"),
                h("button", { className: "btn-small-cancel", onClick: () => setIsCreatingFile(false) }, "✕")
              )
            ),

          // File Search Filter
          h(
            "div",
            { className: "file-search-wrap" },
            h("input", {
              type: "text",
              className: "input-file-search",
              placeholder: "Filter files...",
              value: fileSearch,
              onChange: (e) => setFileSearch(e.target.value)
            })
          ),

          // File Explorer List
          h(
            "div",
            { className: "file-tree" },
            filteredFileNames.map((name) => {
              const isActive = name === activeFileName;
              return h(
                "div",
                {
                  key: name,
                  className: "file-item " + (isActive ? "active" : ""),
                  onClick: () => switchActiveFile(name)
                },
                h("span", { className: "file-icon" }, "📄"),
                h("span", { className: "file-name", title: name }, name),
                h(
                  "button",
                  {
                    className: "file-delete-btn",
                    onClick: (e) => handleDeleteFile(name, e),
                    title: "Delete " + name
                  },
                  "×"
                )
              );
            })
          ),

          // Storage Meter & Reset
          h(
            "div",
            { className: "sidebar-footer" },
            h(
              "div",
              { className: "storage-meter" },
              h(
                "div",
                { className: "storage-label" },
                h("span", null, "VFS Storage"),
                h("span", null, vfsSizeKB + " KB / 5 MB")
              ),
              h(
                "div",
                { className: "meter-track" },
                h("div", {
                  className: "meter-bar",
                  style: { width: Math.min(100, (vfsSizeKB / 5000) * 100) + "%" }
                })
              )
            ),
            h(
              "button",
              {
                className: "btn-link-reset",
                onClick: handleResetVFS,
                title: "Revert Virtual File System to defaults"
              },
              "↺ Reset Starter Files"
            )
          )
        ),

        // CENTER: CODE EDITOR
        h(
          "section",
          { className: "abapc-editor-pane" },

          // Tab & Status Bar
          h(
            "div",
            { className: "editor-tabbar" },
            h(
              "div",
              { className: "active-tab-chip" },
              h("span", { className: "tab-icon" }, "📄"),
              h("span", { className: "tab-name" }, activeFileName)
            ),
            h(
              "div",
              { className: "editor-status-indicators" },
              // Manual Save button (shows when Auto-Save is OFF or if there are unsaved edits)
              !autoSaveEnabled &&
                h(
                  "button",
                  {
                    className: "btn-manual-save",
                    onClick: handleManualSave,
                    title: "Save active file to browser (Ctrl+S)"
                  },
                  "💾 Save (Ctrl+S)"
                ),
              // Auto-Save Toggle Pill
              h(
                "button",
                {
                  className: "btn-autosave-toggle " + (autoSaveEnabled ? "active" : "disabled"),
                  onClick: toggleAutoSave,
                  title: autoSaveEnabled ? "Auto-Save is ON (Click to turn OFF)" : "Auto-Save is OFF (Click to turn ON)"
                },
                h("span", { className: "toggle-pill-dot" }),
                autoSaveEnabled ? "Auto-Save: ON" : "Auto-Save: OFF"
              ),
              // Status Indicator Pill
              h(
                "div",
                { className: "save-status-pill " + saveStatus },
                saveStatus === "saving"
                  ? [h("span", { key: "d", className: "pulse-amber" }, "○"), h("span", { key: "t" }, "Saving in 1.5s...")]
                  : saveStatus === "unsaved"
                  ? [h("span", { key: "d", className: "pulse-amber" }, "●"), h("span", { key: "t" }, "Unsaved changes")]
                  : [h("span", { key: "d", className: "dot-green" }, "●"), h("span", { key: "t" }, autoSaveEnabled ? "Saved to browser" : "Saved (Manual)")]
              ),
              h("span", { className: "metrics-pill" }, lineCount + " lines • " + editorContent.length + " chars")
            )
          ),

          // Ephemeral Banner
          h(
            "div",
            { className: "ephemeral-banner" },
            h("span", { className: "banner-icon" }, "🔒"),
            h(
              "span",
              null,
              h("strong", null, "Ephemeral Sandbox: "),
              "Zero server disk retention. Code is parsed in-memory; RAII drop-guard deletes temp files instantly upon execution."
            )
          ),

          // Editor Surface
          h(
            "div",
            { className: "editor-surface" },
            h(
              "div",
              { ref: gutterRef, className: "editor-gutter" },
              lines.map((_, idx) => h("div", { key: idx, className: "gutter-num" }, idx + 1))
            ),
            h("textarea", {
              ref: editorRef,
              className: "editor-textarea",
              spellCheck: "false",
              autoCapitalize: "off",
              autoComplete: "off",
              autoCorrect: "off",
              value: editorContent,
              onChange: handleEditorChange,
              onScroll: handleScroll,
              onKeyDown: handleKeyDown,
              placeholder: "Write your ABAP code here..."
            })
          ),

          // Shortcuts Legend
          h(
            "div",
            { className: "editor-shortcuts-bar" },
            h("span", null, h("kbd", null, "Ctrl"), " / ", h("kbd", null, "⌘"), " + ", h("kbd", null, "Enter"), " Run"),
            h("span", null, h("kbd", null, "Ctrl"), " + ", h("kbd", null, "S"), " Save"),
            h("span", null, h("kbd", null, "Tab"), " Indent 2 spaces"),
            h("span", null, h("kbd", null, "F6"), " Fast Run"),
            h("span", { className: "autosave-hint" }, "Auto-saves to localStorage after 1.5s idle")
          )
        ),

        // RIGHT: RESULTS INSPECTOR
        h(
          "aside",
          { className: "abapc-results-pane" },

          // Tabs
          h(
            "div",
            { className: "results-tabbar" },
            h(
              "button",
              {
                className: "res-tab " + (activeInspectorTab === "output" ? "active" : ""),
                onClick: () => setActiveInspectorTab("output")
              },
              "Output"
            ),
            h(
              "button",
              {
                className: "res-tab " + (activeInspectorTab === "diagnostics" ? "active" : ""),
                onClick: () => setActiveInspectorTab("diagnostics")
              },
              "Diagnostics",
              executionResult && executionResult.diagnostics && executionResult.diagnostics.length > 0
                ? h("span", { className: "diag-badge-count" }, executionResult.diagnostics.length)
                : null
            ),
            h(
              "button",
              {
                className: "res-tab " + (activeInspectorTab === "variables" ? "active" : ""),
                onClick: () => setActiveInspectorTab("variables")
              },
              "Variables"
            ),
            h(
              "button",
              {
                className: "res-tab " + (activeInspectorTab === "ir" ? "active" : ""),
                onClick: () => setActiveInspectorTab("ir")
              },
              "Bytecode (IR)"
            )
          ),

          // Rate Limit Alert Banner
          rateLimitTimer !== null &&
            h(
              "div",
              { className: "alert-rate-limit" },
              h("div", { className: "alert-title" }, "⚠️ 429 Too Many Requests"),
              h(
                "div",
                { className: "alert-desc" },
                "IP rate limiter triggered (maximum 3 requests/minute). You can execute again in ",
                h("strong", null, rateLimitTimer + " seconds"),
                "."
              )
            ),

          // Error Message Banner
          errorMessage &&
            h("div", { className: "alert-error-bar" }, h("span", null, errorMessage)),

          // Tab Views
          h(
            "div",
            { className: "inspector-content" },

            // TAB: OUTPUT
            activeInspectorTab === "output" &&
              h(
                "div",
                { className: "inspector-output-view" },
                executionResult &&
                  h(
                    "div",
                    { className: "stat-cards-grid" },
                    h(
                      "div",
                      { className: "stat-card" },
                      h("span", { className: "stat-num" }, executionResult.ok ? "SUCCESS" : "FAILED"),
                      h("span", { className: "stat-lbl" }, "VM Status")
                    ),
                    h(
                      "div",
                      { className: "stat-card" },
                      h("span", { className: "stat-num" }, (executionResult.runtime?.elapsed_ms ?? 0) + " ms"),
                      h("span", { className: "stat-lbl" }, "Execution Time")
                    ),
                    h(
                      "div",
                      { className: "stat-card" },
                      h("span", { className: "stat-num" }, executionResult.runtime?.steps ?? 0),
                      h("span", { className: "stat-lbl" }, "Opcodes Run")
                    ),
                    h(
                      "div",
                      { className: "stat-card" },
                      h("span", { className: "stat-num" }, executionResult.database_type || "Mock"),
                      h("span", { className: "stat-lbl" }, "Database Engine")
                    )
                  ),
                h(
                  "div",
                  { className: "terminal-wrap" },
                  executionResult && executionResult.output && executionResult.output.length > 0
                    ? executionResult.output.map((line, idx) =>
                        h("div", { key: idx, className: "terminal-line" }, line)
                      )
                    : h(
                        "div",
                        { className: "terminal-empty" },
                        executionResult
                          ? "Program executed successfully but produced no WRITE output."
                          : "Click '▶ Run Code' (Ctrl+Enter) to execute the active program."
                      )
                )
              ),

            // TAB: DIAGNOSTICS
            activeInspectorTab === "diagnostics" &&
              h(
                "div",
                { className: "inspector-diagnostics-view" },
                executionResult && executionResult.diagnostics && executionResult.diagnostics.length > 0
                  ? executionResult.diagnostics.map((diag, idx) =>
                      h(
                        "div",
                        { key: idx, className: "diag-item severity-" + diag.severity },
                        h(
                          "div",
                          { className: "diag-header" },
                          h("span", { className: "pill-severity " + diag.severity }, diag.severity.toUpperCase()),
                          h("span", { className: "diag-code" }, diag.code),
                          diag.start &&
                            h("span", { className: "diag-pos" }, "Line " + diag.start.line + ", Col " + diag.start.column)
                        ),
                        h("div", { className: "diag-message" }, diag.message)
                      )
                    )
                  : h(
                      "div",
                      { className: "terminal-empty" },
                      executionResult ? "No diagnostics or syntax errors found." : "No diagnostics."
                    )
              ),

            // TAB: VARIABLES
            activeInspectorTab === "variables" &&
              h(
                "div",
                { className: "inspector-variables-view" },
                executionResult && executionResult.variables && Object.keys(executionResult.variables).length > 0
                  ? h("pre", { className: "json-tree" }, JSON.stringify(executionResult.variables, null, 2))
                  : h("div", { className: "terminal-empty" }, "No variables in scope or program has not been executed yet.")
              ),

            // TAB: BYTECODE (IR)
            activeInspectorTab === "ir" &&
              h(
                "div",
                { className: "inspector-ir-view" },
                executionResult && executionResult.ir
                  ? h("pre", { className: "json-tree" }, JSON.stringify(executionResult.ir, null, 2))
                  : h("div", { className: "terminal-empty" }, "Bytecode IR will be displayed here after compilation.")
              )
          )
        )
      ),

      // 3. MODAL: BYODB SETTINGS
      showDbModal &&
        h(
          "div",
          { className: "modal-backdrop", onClick: () => setShowDbModal(false) },
          h(
            "div",
            { className: "modal-card", onClick: (e) => e.stopPropagation() },
            h(
              "div",
              { className: "modal-header" },
              h(
                "div",
                { className: "modal-title-wrap" },
                h("span", { className: "modal-icon" }, "⚙"),
                h("h3", null, "Bring Your Own Database (BYODB)")
              ),
              h("button", { className: "btn-close", onClick: () => setShowDbModal(false) }, "×")
            ),
            h(
              "div",
              { className: "modal-body" },
              h(
                "p",
                { className: "modal-desc" },
                "Provide an external PostgreSQL connection string to execute Open SQL (",
                h("code", null, "SELECT"),
                ") queries directly against your live database."
              ),
              h(
                "div",
                { className: "callout-security" },
                h("strong", null, "🔒 Zero-Retention Guarantee: "),
                "Your connection string is saved strictly in your browser's ",
                h("code", null, "localStorage"),
                ". When you click Run, the server establishes a scoped connection, executes the translated SQL, and closes it immediately. Credentials are never stored on disk."
              ),
              // Password Masked DB URL Input
              h(
                "div",
                { className: "form-group" },
                h("label", { className: "form-label" }, "PostgreSQL Connection URL (db_url):"),
                h(
                  "div",
                  { className: "input-with-button" },
                  h("input", {
                    type: showDbPassword ? "text" : "password",
                    className: "input-text-db",
                    placeholder: "postgresql://user:password@db.xxxx.supabase.co:5432/postgres?sslmode=require",
                    value: dbUrl,
                    onChange: (e) => handleSaveDbUrl(e.target.value)
                  }),
                  h(
                    "button",
                    {
                      className: "btn-toggle-mask",
                      onClick: () => setShowDbPassword(!showDbPassword),
                      title: "Toggle password visibility"
                    },
                    showDbPassword ? "Hide" : "Show"
                  )
                )
              ),
              // Template Shortcuts
              h(
                "div",
                { className: "quick-templates-wrap" },
                h("span", { className: "templates-label" }, "Templates:"),
                h(
                  "button",
                  {
                    className: "btn-template",
                    onClick: () =>
                      handleSaveDbUrl(
                        "postgresql://postgres:[PASSWORD]@db.[PROJECT-REF].supabase.co:5432/postgres?sslmode=require"
                      )
                  },
                  "Supabase"
                ),
                h(
                  "button",
                  {
                    className: "btn-template",
                    onClick: () =>
                      handleSaveDbUrl("postgresql://[USER]:[PASSWORD]@[ENDPOINT].neon.tech/neondb?sslmode=require")
                  },
                  "Neon"
                ),
                h(
                  "button",
                  {
                    className: "btn-template",
                    onClick: () => handleSaveDbUrl("postgresql://postgres:postgres@localhost:5432/postgres")
                  },
                  "Localhost"
                ),
                h(
                  "button",
                  { className: "btn-template-clear", onClick: () => handleSaveDbUrl("") },
                  "Clear (Use Mock DB)"
                )
              ),
              // Test Connection
              h(
                "div",
                { className: "test-connection-section" },
                h(
                  "button",
                  {
                    className: "btn-test-db",
                    onClick: handleTestDbConnection,
                    disabled: !dbUrl.trim()
                  },
                  "Test Connection"
                ),
                dbTestResult &&
                  h(
                    "div",
                    {
                      className:
                        "test-result-box " +
                        (dbTestResult.testing
                          ? "testing"
                          : dbTestResult.success
                          ? "success"
                          : "failure")
                    },
                    dbTestResult.message
                  )
              )
            ),
            h(
              "div",
              { className: "modal-footer" },
              h("button", { className: "btn-primary", onClick: () => setShowDbModal(false) }, "Done")
            )
          )
        ),

      // 4. MODAL: IN-APP GUIDE
      showGuideModal &&
        h(
          "div",
          { className: "modal-backdrop", onClick: () => setShowGuideModal(false) },
          h(
            "div",
            { className: "modal-card modal-large", onClick: (e) => e.stopPropagation() },
            h(
              "div",
              { className: "modal-header" },
              h(
                "div",
                { className: "modal-title-wrap" },
                h("span", { className: "modal-icon" }, "📖"),
                h("h3", null, "Stateless Architecture & BYODB User Guide")
              ),
              h("button", { className: "btn-close", onClick: () => setShowGuideModal(false) }, "×")
            ),
            h(
              "div",
              { className: "guide-nav" },
              h(
                "button",
                {
                  className: "guide-tab-btn " + (guideTab === "architecture" ? "active" : ""),
                  onClick: () => setGuideTab("architecture")
                },
                "1. Stateless Architecture"
              ),
              h(
                "button",
                {
                  className: "guide-tab-btn " + (guideTab === "supabase_neon" ? "active" : ""),
                  onClick: () => setGuideTab("supabase_neon")
                },
                "2. Supabase & Neon Setup"
              ),
              h(
                "button",
                {
                  className: "guide-tab-btn " + (guideTab === "opensql_demo" ? "active" : ""),
                  onClick: () => setGuideTab("opensql_demo")
                },
                "3. Open SQL Demo Schema"
              )
            ),
            h(
              "div",
              { className: "modal-body guide-body" },
              // Guide Tab 1
              guideTab === "architecture" &&
                h(
                  "div",
                  { className: "guide-section" },
                  h("h4", null, "Stateless Execution Playground (CodePen Model)"),
                  h(
                    "p",
                    null,
                    "ABAPC Studio 5.0.1 is designed with a strict zero-retention architecture. No user programs or connection credentials are saved to the server disk."
                  ),
                  h(
                    "div",
                    { className: "architecture-grid" },
                    h(
                      "div",
                      { className: "arch-card" },
                      h("div", { className: "arch-card-icon" }, "💻"),
                      h("h5", null, "1. Browser VFS"),
                      h(
                        "p",
                        null,
                        "Your multi-file directory structure resides entirely in your browser's ",
                        h("code", null, "localStorage"),
                        ". Edits auto-save with a 1500ms debounce."
                      )
                    ),
                    h(
                      "div",
                      { className: "arch-card" },
                      h("div", { className: "arch-card-icon" }, "⚡"),
                      h("h5", null, "2. Ephemeral Workers"),
                      h(
                        "p",
                        null,
                        "When you click ",
                        h("strong", null, "Run"),
                        ", your code is sent to ",
                        h("code", null, "POST /api/run"),
                        " and protected with an RAII ",
                        h("code", null, "AutoCleanFile"),
                        " drop-guard."
                      )
                    ),
                    h(
                      "div",
                      { className: "arch-card" },
                      h("div", { className: "arch-card-icon" }, "🛡️"),
                      h("h5", null, "3. Scoped DB & Rate Limiting"),
                      h(
                        "p",
                        null,
                        "PostgreSQL connections are scoped per request. Redis limits execution to 3 requests/minute per IP, with a 5-second infinite-loop guard."
                      )
                    )
                  )
                ),
              // Guide Tab 2
              guideTab === "supabase_neon" &&
                h(
                  "div",
                  { className: "guide-section" },
                  h("h4", null, "How to get a Free Cloud PostgreSQL Connection String"),
                  h(
                    "div",
                    { className: "provider-steps" },
                    h("h5", null, "Option A: Supabase (Free Tier)"),
                    h(
                      "ol",
                      null,
                      h("li", null, "Go to supabase.com and create an account."),
                      h("li", null, "Create a new project."),
                      h("li", null, "Go to Project Settings → Database → Connection String."),
                      h(
                        "li",
                        null,
                        "Select URI mode and copy the string:",
                        h(
                          "pre",
                          { className: "code-snippet" },
                          "postgresql://postgres:[PASSWORD]@db.xxxx.supabase.co:5432/postgres?sslmode=require"
                        )
                      ),
                      h("li", null, "Paste into the Database (BYODB) settings panel.")
                    ),
                    h("h5", null, "Option B: Neon (Serverless Postgres)"),
                    h(
                      "ol",
                      null,
                      h("li", null, "Go to neon.tech and create a free project."),
                      h("li", null, "On the project dashboard, locate Connection details."),
                      h(
                        "li",
                        null,
                        "Select Connection string:",
                        h(
                          "pre",
                          { className: "code-snippet" },
                          "postgresql://[USER]:[PASSWORD]@[ENDPOINT].neon.tech/neondb?sslmode=require"
                        )
                      )
                    )
                  )
                ),
              // Guide Tab 3
              guideTab === "opensql_demo" &&
                h(
                  "div",
                  { className: "guide-section" },
                  h("h4", null, "PostgreSQL DDL & Sample Query"),
                  h("p", null, "Execute this DDL in your cloud SQL editor to create the standard SFLIGHT table:"),
                  h("pre", { className: "code-snippet" }, SAMPLE_POSTGRES_DDL),
                  h(
                    "div",
                    { className: "guide-actions-bar" },
                    h(
                      "button",
                      {
                        className: "btn-primary",
                        onClick: () => {
                          const demoName = "z_supabase_demo.abap";
                          if (!files[demoName]) {
                            const updated = Object.assign({}, files, {
                              [demoName]: `REPORT z_supabase_demo.

TYPES: BEGIN OF ty_flight,
         carrid TYPE string,
         connid TYPE i,
         price  TYPE i,
       END OF ty_flight.

DATA: lt_flights TYPE TABLE OF ty_flight,
      ls_flight  TYPE ty_flight.

WRITE: / 'Querying BYODB PostgreSQL SFLIGHT Table...'.

SELECT carrid connid price
  FROM sflight
  WHERE price >= 400
  INTO TABLE @lt_flights.

WRITE: / |Fetched { sy-dbcnt } flights:|.
LOOP AT lt_flights INTO ls_flight.
  WRITE: / |  * Flight { ls_flight-carrid }-{ ls_flight-connid }: \${ ls_flight-price }|.
ENDLOOP.
`
                            });
                            setFiles(updated);
                            try {
                              localStorage.setItem("abapc_vfs_files", JSON.stringify(updated));
                            } catch {}
                          }
                          setShowGuideModal(false);
                          switchActiveFile(demoName);
                        }
                      },
                      "+ Load Demo into Virtual File System"
                    )
                  )
                )
            ),
            h(
              "div",
              { className: "modal-footer" },
              h("button", { className: "btn-secondary", onClick: () => setShowGuideModal(false) }, "Close Guide")
            )
          )
        ),

      // 5. FOOTER
      h(
        "footer",
        { className: "abapc-footer" },
        h("span", null, "ABAPC Studio 5.0.1 • Stateless Execution Playground"),
        h("span", null, "Bring Your Own Database (BYODB) Enabled"),
        h("span", null, "Localhost 127.0.0.1:7345")
      )
    );
  }

  // Mount React App
  function mount() {
    const rootEl = document.getElementById("root");
    if (!rootEl) return;
    if (ReactDOM.createRoot) {
      const root = ReactDOM.createRoot(rootEl);
      root.render(h(AbapcStudioApp));
    } else {
      ReactDOM.render(h(AbapcStudioApp), rootEl);
    }
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", mount);
  } else {
    mount();
  }
})();
