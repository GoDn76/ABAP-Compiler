import React, { useState, useEffect, useRef, useCallback } from "react";

// ============================================================================
// DEFAULT STARTER FILES FOR THE VIRTUAL FILE SYSTEM (VFS)
// ============================================================================
export const DEFAULT_VFS_FILES = {
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
`
};

export const SAMPLE_POSTGRES_DDL = `-- Run this in your Supabase or Neon SQL Editor:
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

// ============================================================================
// MAIN REACT PLAYGROUND COMPONENT
// ============================================================================
export default function AbapcStudioApp() {
  // 1. Virtual File System (VFS) in localStorage
  const [files, setFiles] = useState(() => {
    try {
      const saved = localStorage.getItem("abapc_vfs_files");
      if (saved) {
        const parsed = JSON.parse(saved);
        if (parsed && typeof parsed === "object" && Object.keys(parsed).length > 0) {
          return parsed;
        }
      }
    } catch (e) {
      console.warn("Failed to load VFS from localStorage", e);
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

  // 2. BYODB (Bring Your Own Database)
  const [dbUrl, setDbUrl] = useState(() => {
    try {
      return localStorage.getItem("abapc_byodb_url") || "";
    } catch {
      return "";
    }
  });
  const [showDbPassword, setShowDbPassword] = useState(false);
  const [dbTestResult, setDbTestResult] = useState(null);

  // 3. Modals and Navigation
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

  // 5. File Manager UI Helpers
  const [fileSearch, setFileSearch] = useState("");
  const [newFileName, setNewFileName] = useState("");
  const [isCreatingFile, setIsCreatingFile] = useState(false);

  // References
  const debounceTimerRef = useRef(null);
  const editorRef = useRef(null);
  const gutterRef = useRef(null);

  // ==========================================================================
  // DEBOUNCED AUTO-SAVE (1500ms) & MANUAL SAVE
  // ==========================================================================
  const persistActiveFile = useCallback(
    (fileName, content) => {
      setFiles((prev) => {
        const next = { ...prev, [fileName]: content };
        try {
          localStorage.setItem("abapc_vfs_files", JSON.stringify(next));
        } catch (e) {
          console.error("VFS localStorage write error", e);
        }
        return next;
      });
      setSaveStatus("saved");
    },
    []
  );

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

  // Immediate flush when switching files
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

  // Flush before unload
  useEffect(() => {
    const handleBeforeUnload = () => {
      if (debounceTimerRef.current) {
        clearTimeout(debounceTimerRef.current);
        try {
          const updated = { ...files, [activeFileName]: editorContent };
          localStorage.setItem("abapc_vfs_files", JSON.stringify(updated));
        } catch {}
      }
    };
    window.addEventListener("beforeunload", handleBeforeUnload);
    return () => window.removeEventListener("beforeunload", handleBeforeUnload);
  }, [files, activeFileName, editorContent]);

  // Rate Limiter Countdown Effect
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

  // Gutter sync
  const lines = editorContent.split("\n");
  const lineCount = lines.length;

  const handleScroll = () => {
    if (editorRef.current && gutterRef.current) {
      gutterRef.current.scrollTop = editorRef.current.scrollTop;
    }
  };

  // Tab key indent support
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

  // ==========================================================================
  // FILE CREATION / DELETION / RESET
  // ==========================================================================
  const handleCreateFile = () => {
    let name = newFileName.trim();
    if (!name) return;
    if (!name.endsWith(".abap")) {
      name += ".abap";
    }
    if (files[name]) {
      alert(`File "${name}" already exists!`);
      return;
    }
    const starter = `REPORT ${name.replace(".abap", "")}.\n\nWRITE: / 'File: ${name}'.\n`;
    const updated = { ...files, [name]: starter };
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
    if (!confirm(`Are you sure you want to delete ${fileName}?`)) return;

    const updated = { ...files };
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

  // ==========================================================================
  // BYODB SETTINGS SAVE & TEST
  // ==========================================================================
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
          message: `Connected successfully! Engine: ${data.database_type}`
        });
      } else {
        setDbTestResult({
          success: false,
          message: `Unable to connect: ${data.database_type || "Scoped query failed. Fallback active."}`
        });
      }
    } catch (e) {
      setDbTestResult({
        success: false,
        message: `Network or host error: ${String(e)}`
      });
    }
  };

  // ==========================================================================
  // EXECUTION: POST /api/run
  // ==========================================================================
  const runCode = async () => {
    if (isRunning || rateLimitTimer !== null) return;

    // Immediately flush pending edits before running
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
          `429 Rate Limit Reached: ${errorJson.message || "Exceeded 3 requests/minute per IP."} Please wait ${retryAfter}s.`
        );
        setActiveInspectorTab("diagnostics");
        return;
      }

      if (response.status === 408) {
        const errorJson = await response.json();
        setErrorMessage(
          `408 Execution Timeout: ${errorJson.message || "Execution exceeded 5-second limit; possible infinite loop."}`
        );
        setActiveInspectorTab("diagnostics");
        return;
      }

      if (!response.ok) {
        const errorJson = await response.json().catch(() => ({}));
        throw new Error(errorJson.error || errorJson.message || `HTTP ${response.status}`);
      }

      const data = await response.json();
      setExecutionResult(data);

      if (!data.ok && data.diagnostics && data.diagnostics.length > 0) {
        setActiveInspectorTab("diagnostics");
      }
    } catch (err) {
      setErrorMessage(`Execution error: ${err.message || String(err)}`);
      setActiveInspectorTab("diagnostics");
    } finally {
      setIsRunning(false);
    }
  };

  // Compute VFS Storage size
  const vfsSizeKB = (JSON.stringify(files).length / 1024).toFixed(1);

  // Filtered file keys
  const filteredFileNames = Object.keys(files).filter((f) =>
    f.toLowerCase().includes(fileSearch.toLowerCase())
  );

  return (
    <div className="abapc-container">
      {/* ==================================================================== */}
      {/* TOPBAR / APP HEADER                                                 */}
      {/* ==================================================================== */}
      <header className="abapc-topbar">
        <div className="abapc-brand">
          <div className="abapc-logo">A</div>
          <div>
            <div className="brand-title">
              ABAPC Studio <span className="version-pill">v5.0.1</span>
            </div>
            <div className="brand-subtitle">
              <span className="stateless-badge">100% Stateless • BYODB</span>
              Local ABAP + Cloud Compiler
            </div>
          </div>
        </div>

        <div className="abapc-actions">
          {/* Database Status Button */}
          <button
            className={`btn-db-indicator ${dbUrl.trim() ? "active" : "mock"}`}
            onClick={() => setShowDbModal(true)}
            title="Configure Bring Your Own Database (PostgreSQL)"
          >
            {dbUrl.trim() ? (
              <>
                <span className="dot dot-green"></span>
                <span>PostgreSQL (BYODB)</span>
              </>
            ) : (
              <>
                <span className="dot dot-amber"></span>
                <span>Local Mock Engine</span>
              </>
            )}
          </button>

          {/* In-App Guide Button */}
          <button
            className="btn-secondary"
            onClick={() => setShowGuideModal(true)}
            title="Open In-App BYODB & Architecture Guide"
          >
            📖 Architecture & SQL Guide
          </button>

          {/* Compiler Mode Dropdown */}
          <select
            className="select-mode"
            value={compilerMode}
            onChange={(e) => setCompilerMode(e.target.value)}
            title="Compiler language mode"
          >
            <option value="standard">Standard ABAP</option>
            <option value="cloud">ABAP Cloud (Strict Release APIs)</option>
          </select>

          {/* Run Button */}
          <button
            className="btn-primary"
            onClick={runCode}
            disabled={isRunning || rateLimitTimer !== null}
            title="Run active file (Ctrl+Enter)"
          >
            {isRunning ? (
              "⏳ Running..."
            ) : rateLimitTimer !== null ? (
              `⏳ Rate Limited (${rateLimitTimer}s)`
            ) : (
              "▶ Run Code"
            )}
          </button>
        </div>
      </header>

      {/* ==================================================================== */}
      {/* MAIN 3-PANE WORKSPACE                                               */}
      {/* ==================================================================== */}
      <main className="abapc-workspace">
        {/* ------------------------------------------------------------------ */}
        {/* LEFT PANE: VIRTUAL FILE SYSTEM SIDEBAR                            */}
        {/* ------------------------------------------------------------------ */}
        <aside className="abapc-sidebar">
          <div className="sidebar-head">
            <div className="sidebar-title">
              <span>VIRTUAL FILES</span>
              <span className="file-count-badge">{Object.keys(files).length}</span>
            </div>
            <button
              className="btn-icon"
              onClick={() => setIsCreatingFile(!isCreatingFile)}
              title="Create new .abap file"
            >
              +
            </button>
          </div>

          {/* New File Inline Form */}
          {isCreatingFile && (
            <div className="new-file-box">
              <input
                type="text"
                className="input-new-file"
                placeholder="z_my_prog.abap"
                value={newFileName}
                onChange={(e) => setNewFileName(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === "Enter") handleCreateFile();
                  if (e.key === "Escape") setIsCreatingFile(false);
                }}
                autoFocus
              />
              <div className="new-file-actions">
                <button className="btn-small-primary" onClick={handleCreateFile}>
                  Add
                </button>
                <button className="btn-small-cancel" onClick={() => setIsCreatingFile(false)}>
                  ✕
                </button>
              </div>
            </div>
          )}

          {/* File Filter / Search */}
          <div className="file-search-wrap">
            <input
              type="text"
              className="input-file-search"
              placeholder="Filter files..."
              value={fileSearch}
              onChange={(e) => setFileSearch(e.target.value)}
            />
          </div>

          {/* File Explorer List */}
          <div className="file-tree">
            {filteredFileNames.map((name) => {
              const isActive = name === activeFileName;
              return (
                <div
                  key={name}
                  className={`file-item ${isActive ? "active" : ""}`}
                  onClick={() => switchActiveFile(name)}
                >
                  <span className="file-icon">📄</span>
                  <span className="file-name" title={name}>
                    {name}
                  </span>
                  <button
                    className="file-delete-btn"
                    onClick={(e) => handleDeleteFile(name, e)}
                    title={`Delete ${name}`}
                  >
                    ×
                  </button>
                </div>
              );
            })}
          </div>

          {/* Storage Meter & Reset */}
          <div className="sidebar-footer">
            <div className="storage-meter">
              <div className="storage-label">
                <span>VFS Storage</span>
                <span>{vfsSizeKB} KB / 5 MB</span>
              </div>
              <div className="meter-track">
                <div
                  className="meter-bar"
                  style={{ width: `${Math.min(100, (vfsSizeKB / 5000) * 100)}%` }}
                ></div>
              </div>
            </div>
            <button className="btn-link-reset" onClick={handleResetVFS}>
              ↺ Reset Starter Files
            </button>
          </div>
        </aside>

        {/* ------------------------------------------------------------------ */}
        {/* CENTER PANE: CODE EDITOR                                          */}
        {/* ------------------------------------------------------------------ */}
        <section className="abapc-editor-pane">
          {/* File Tab & Debounce Status Header */}
          <div className="editor-tabbar">
            <div className="active-tab-chip">
              <span className="tab-icon">📄</span>
              <span className="tab-name">{activeFileName}</span>
            </div>

            <div className="editor-status-indicators">
              {/* Manual Save Button (shows when Auto-Save is OFF or unsaved changes exist) */}
              {!autoSaveEnabled && (
                <button
                  className="btn-manual-save"
                  onClick={handleManualSave}
                  title="Save active file to browser (Ctrl+S)"
                >
                  💾 Save (Ctrl+S)
                </button>
              )}

              {/* Auto-Save Toggle Pill */}
              <button
                className={`btn-autosave-toggle ${autoSaveEnabled ? "active" : "disabled"}`}
                onClick={toggleAutoSave}
                title={autoSaveEnabled ? "Auto-Save is ON (Click to turn OFF)" : "Auto-Save is OFF (Click to turn ON)"}
              >
                <span className="toggle-pill-dot"></span>
                {autoSaveEnabled ? "Auto-Save: ON" : "Auto-Save: OFF"}
              </button>

              {/* Status Indicator */}
              <div className={`save-status-pill ${saveStatus}`}>
                {saveStatus === "saving" ? (
                  <>
                    <span className="pulse-amber">○</span>
                    <span>Saving in 1.5s...</span>
                  </>
                ) : saveStatus === "unsaved" ? (
                  <>
                    <span className="pulse-amber">●</span>
                    <span>Unsaved changes</span>
                  </>
                ) : (
                  <>
                    <span className="dot-green">●</span>
                    <span>{autoSaveEnabled ? "Saved to browser" : "Saved (Manual)"}</span>
                  </>
                )}
              </div>

              <span className="metrics-pill">
                {lineCount} lines • {editorContent.length} chars
              </span>
            </div>
          </div>

          {/* Ephemeral Privacy Notice Banner */}
          <div className="ephemeral-banner">
            <span className="banner-icon">🔒</span>
            <span>
              <strong>Ephemeral Sandbox:</strong> Zero server disk retention. Code is parsed in-memory;
              RAII drop-guard deletes temp files instantly upon execution.
            </span>
          </div>

          {/* Code Editor Surface */}
          <div className="editor-surface">
            <div ref={gutterRef} className="editor-gutter">
              {lines.map((_, idx) => (
                <div key={idx} className="gutter-num">
                  {idx + 1}
                </div>
              ))}
            </div>

            <textarea
              ref={editorRef}
              className="editor-textarea"
              spellCheck="false"
              autoCapitalize="off"
              autoComplete="off"
              autoCorrect="off"
              value={editorContent}
              onChange={handleEditorChange}
              onScroll={handleScroll}
              onKeyDown={handleKeyDown}
              placeholder="Write your ABAP code here..."
            />
          </div>

          {/* Keyboard Shortcuts Hint */}
          <div className="editor-shortcuts-bar">
            <span>
              <kbd>Ctrl</kbd>/<kbd>⌘</kbd> + <kbd>Enter</kbd> Run
            </span>
            <span>
              <kbd>Ctrl</kbd> + <kbd>S</kbd> Save
            </span>
            <span>
              <kbd>Tab</kbd> Indent 2 spaces
            </span>
            <span>
              <kbd>F6</kbd> Fast Run
            </span>
            <span className="autosave-hint">Auto-saves to localStorage after 1.5s idle</span>
          </div>
        </section>

        {/* ------------------------------------------------------------------ */}
        {/* RIGHT PANE: TABBED RESULTS & INSPECTION                           */}
        {/* ------------------------------------------------------------------ */}
        <aside className="abapc-results-pane">
          {/* Tab Navigation */}
          <div className="results-tabbar">
            <button
              className={`res-tab ${activeInspectorTab === "output" ? "active" : ""}`}
              onClick={() => setActiveInspectorTab("output")}
            >
              Output
            </button>
            <button
              className={`res-tab ${activeInspectorTab === "diagnostics" ? "active" : ""}`}
              onClick={() => setActiveInspectorTab("diagnostics")}
            >
              Diagnostics
              {executionResult?.diagnostics?.length ? (
                <span className="diag-badge-count">{executionResult.diagnostics.length}</span>
              ) : null}
            </button>
            <button
              className={`res-tab ${activeInspectorTab === "variables" ? "active" : ""}`}
              onClick={() => setActiveInspectorTab("variables")}
            >
              Variables
            </button>
            <button
              className={`res-tab ${activeInspectorTab === "ir" ? "active" : ""}`}
              onClick={() => setActiveInspectorTab("ir")}
            >
              Bytecode (IR)
            </button>
          </div>

          {/* Rate Limit Alert Notification */}
          {rateLimitTimer !== null && (
            <div className="alert-rate-limit">
              <div className="alert-title">⚠️ 429 Too Many Requests</div>
              <div className="alert-desc">
                IP rate limiter triggered (maximum 3 requests/minute). You can execute again in{" "}
                <strong>{rateLimitTimer} seconds</strong>.
              </div>
            </div>
          )}

          {/* Error Message Notification */}
          {errorMessage && (
            <div className="alert-error-bar">
              <span>{errorMessage}</span>
            </div>
          )}

          {/* Inspector Tab Content */}
          <div className="inspector-content">
            {/* TAB: OUTPUT */}
            {activeInspectorTab === "output" && (
              <div className="inspector-output-view">
                {/* Stats Summary Bar */}
                {executionResult && (
                  <div className="stat-cards-grid">
                    <div className="stat-card">
                      <span className="stat-num">{executionResult.ok ? "SUCCESS" : "FAILED"}</span>
                      <span className="stat-lbl">VM Status</span>
                    </div>
                    <div className="stat-card">
                      <span className="stat-num">{executionResult.runtime?.elapsed_ms ?? 0} ms</span>
                      <span className="stat-lbl">Execution Time</span>
                    </div>
                    <div className="stat-card">
                      <span className="stat-num">{executionResult.runtime?.steps ?? 0}</span>
                      <span className="stat-lbl">Opcodes Run</span>
                    </div>
                    <div className="stat-card">
                      <span className="stat-num">{executionResult.database_type || "Mock"}</span>
                      <span className="stat-lbl">Database Engine</span>
                    </div>
                  </div>
                )}

                {/* Console Terminal */}
                <div className="terminal-wrap">
                  {executionResult?.output && executionResult.output.length > 0 ? (
                    executionResult.output.map((line, idx) => (
                      <div key={idx} className="terminal-line">
                        {line}
                      </div>
                    ))
                  ) : (
                    <div className="terminal-empty">
                      {executionResult
                        ? "Program executed successfully but produced no WRITE output."
                        : "Click '▶ Run Code' (Ctrl+Enter) to execute the active program."}
                    </div>
                  )}
                </div>
              </div>
            )}

            {/* TAB: DIAGNOSTICS */}
            {activeInspectorTab === "diagnostics" && (
              <div className="inspector-diagnostics-view">
                {executionResult?.diagnostics && executionResult.diagnostics.length > 0 ? (
                  executionResult.diagnostics.map((diag, idx) => (
                    <div key={idx} className={`diag-item severity-${diag.severity}`}>
                      <div className="diag-header">
                        <span className={`pill-severity ${diag.severity}`}>
                          {diag.severity.toUpperCase()}
                        </span>
                        <span className="diag-code">{diag.code}</span>
                        {diag.start && (
                          <span className="diag-pos">
                            Line {diag.start.line}, Col {diag.start.column}
                          </span>
                        )}
                      </div>
                      <div className="diag-message">{diag.message}</div>
                    </div>
                  ))
                ) : (
                  <div className="terminal-empty">
                    {executionResult ? "No diagnostics or syntax errors found." : "No diagnostics."}
                  </div>
                )}
              </div>
            )}

            {/* TAB: VARIABLES */}
            {activeInspectorTab === "variables" && (
              <div className="inspector-variables-view">
                {executionResult?.variables &&
                Object.keys(executionResult.variables).length > 0 ? (
                  <pre className="json-tree">
                    {JSON.stringify(executionResult.variables, null, 2)}
                  </pre>
                ) : (
                  <div className="terminal-empty">
                    No variables in scope or program has not been executed yet.
                  </div>
                )}
              </div>
            )}

            {/* TAB: BYTECODE (IR) */}
            {activeInspectorTab === "ir" && (
              <div className="inspector-ir-view">
                {executionResult?.ir ? (
                  <pre className="json-tree">{JSON.stringify(executionResult.ir, null, 2)}</pre>
                ) : (
                  <div className="terminal-empty">
                    Bytecode IR will be displayed here after compilation.
                  </div>
                )}
              </div>
            )}
          </div>
        </aside>
      </main>

      {/* ==================================================================== */}
      {/* MODAL 1: BRING YOUR OWN DATABASE (BYODB) SETTINGS                   */}
      {/* ==================================================================== */}
      {showDbModal && (
        <div className="modal-backdrop" onClick={() => setShowDbModal(false)}>
          <div className="modal-card" onClick={(e) => e.stopPropagation()}>
            <div className="modal-header">
              <div className="modal-title-wrap">
                <span className="modal-icon">⚙</span>
                <h3>Bring Your Own Database (BYODB)</h3>
              </div>
              <button className="btn-close" onClick={() => setShowDbModal(false)}>
                ×
              </button>
            </div>

            <div className="modal-body">
              <p className="modal-desc">
                Provide an external PostgreSQL connection string to execute Open SQL (<code>SELECT</code>)
                queries directly against your live database.
              </p>

              <div className="callout-security">
                <strong>🔒 Zero-Retention Guarantee:</strong> Your connection string is saved strictly in
                your browser&apos;s <code>localStorage</code>. When you click Run, the server establishes a
                scoped connection, executes the translated SQL, and closes it immediately. Credentials are
                never stored on disk.
              </div>

              {/* Password Masked DB URL Input */}
              <div className="form-group">
                <label className="form-label">PostgreSQL Connection URL (db_url):</label>
                <div className="input-with-button">
                  <input
                    type={showDbPassword ? "text" : "password"}
                    className="input-text-db"
                    placeholder="postgresql://user:password@db.xxxx.supabase.co:5432/postgres?sslmode=require"
                    value={dbUrl}
                    onChange={(e) => handleSaveDbUrl(e.target.value)}
                  />
                  <button
                    className="btn-toggle-mask"
                    onClick={() => setShowDbPassword(!showDbPassword)}
                    title="Toggle password visibility"
                  >
                    {showDbPassword ? "Hide" : "Show"}
                  </button>
                </div>
              </div>

              {/* Quick Template Buttons */}
              <div className="quick-templates-wrap">
                <span className="templates-label">Templates:</span>
                <button
                  className="btn-template"
                  onClick={() =>
                    handleSaveDbUrl(
                      "postgresql://postgres:[PASSWORD]@db.[PROJECT-REF].supabase.co:5432/postgres?sslmode=require"
                    )
                  }
                >
                  Supabase
                </button>
                <button
                  className="btn-template"
                  onClick={() =>
                    handleSaveDbUrl(
                      "postgresql://[USER]:[PASSWORD]@[ENDPOINT].neon.tech/neondb?sslmode=require"
                    )
                  }
                >
                  Neon
                </button>
                <button
                  className="btn-template"
                  onClick={() =>
                    handleSaveDbUrl("postgresql://postgres:postgres@localhost:5432/postgres")
                  }
                >
                  Localhost
                </button>
                <button className="btn-template-clear" onClick={() => handleSaveDbUrl("")}>
                  Clear (Use Mock DB)
                </button>
              </div>

              {/* Test Connection Button & Result */}
              <div className="test-connection-section">
                <button
                  className="btn-test-db"
                  onClick={handleTestDbConnection}
                  disabled={!dbUrl.trim()}
                >
                  Test Connection
                </button>

                {dbTestResult && (
                  <div
                    className={`test-result-box ${
                      dbTestResult.testing
                        ? "testing"
                        : dbTestResult.success
                        ? "success"
                        : "failure"
                    }`}
                  >
                    {dbTestResult.message}
                  </div>
                )}
              </div>
            </div>

            <div className="modal-footer">
              <button className="btn-primary" onClick={() => setShowDbModal(false)}>
                Done
              </button>
            </div>
          </div>
        </div>
      )}

      {/* ==================================================================== */}
      {/* MODAL 2: IN-APP GUIDE (STATENESS, SUPABASE/NEON & OPEN SQL)        */}
      {/* ==================================================================== */}
      {showGuideModal && (
        <div className="modal-backdrop" onClick={() => setShowGuideModal(false)}>
          <div className="modal-card modal-large" onClick={(e) => e.stopPropagation()}>
            <div className="modal-header">
              <div className="modal-title-wrap">
                <span className="modal-icon">📖</span>
                <h3>Stateless Architecture & BYODB User Guide</h3>
              </div>
              <button className="btn-close" onClick={() => setShowGuideModal(false)}>
                ×
              </button>
            </div>

            {/* Guide Tabs */}
            <div className="guide-nav">
              <button
                className={`guide-tab-btn ${guideTab === "architecture" ? "active" : ""}`}
                onClick={() => setGuideTab("architecture")}
              >
                1. Stateless Architecture
              </button>
              <button
                className={`guide-tab-btn ${guideTab === "supabase_neon" ? "active" : ""}`}
                onClick={() => setGuideTab("supabase_neon")}
              >
                2. Supabase & Neon Setup
              </button>
              <button
                className={`guide-tab-btn ${guideTab === "opensql_demo" ? "active" : ""}`}
                onClick={() => setGuideTab("opensql_demo")}
              >
                3. Open SQL Demo Schema
              </button>
            </div>

            <div className="modal-body guide-body">
              {/* GUIDE TAB 1: ARCHITECTURE */}
              {guideTab === "architecture" && (
                <div className="guide-section">
                  <h4>Stateless Execution Playground (CodePen Model)</h4>
                  <p>
                    ABAPC Studio 5.0.1 is designed with a strict zero-retention architecture. No user
                    programs or connection credentials are saved to the server disk.
                  </p>

                  <div className="architecture-grid">
                    <div className="arch-card">
                      <div className="arch-card-icon">💻</div>
                      <h5>1. Browser VFS</h5>
                      <p>
                        Your multi-file directory structure resides entirely in your browser&apos;s{" "}
                        <code>localStorage</code>. Edits auto-save with a 1500ms debounce.
                      </p>
                    </div>

                    <div className="arch-card">
                      <div className="arch-card-icon">⚡</div>
                      <h5>2. Ephemeral Workers</h5>
                      <p>
                        When you click <strong>Run</strong>, your code is sent to{" "}
                        <code>POST /api/run</code> and protected with an RAII{" "}
                        <code>AutoCleanFile</code> drop-guard.
                      </p>
                    </div>

                    <div className="arch-card">
                      <div className="arch-card-icon">🛡️</div>
                      <h5>3. Scoped DB & Rate Limiting</h5>
                      <p>
                        PostgreSQL connections are scoped per request. Redis limits execution to 3
                        requests/minute per IP, with a 5-second infinite-loop guard.
                      </p>
                    </div>
                  </div>
                </div>
              )}

              {/* GUIDE TAB 2: SUPABASE & NEON */}
              {guideTab === "supabase_neon" && (
                <div className="guide-section">
                  <h4>How to get a Free Cloud PostgreSQL Connection String</h4>
                  <div className="provider-steps">
                    <h5>Option A: Supabase (Free Tier)</h5>
                    <ol>
                      <li>
                        Go to <a href="https://supabase.com" target="_blank" rel="noreferrer">supabase.com</a> and sign in.
                      </li>
                      <li>Create a new project.</li>
                      <li>
                        Navigate to <strong>Project Settings</strong> → <strong>Database</strong> →{" "}
                        <strong>Connection String</strong>.
                      </li>
                      <li>
                        Select <strong>URI</strong> mode and copy the string:
                        <pre className="code-snippet">
                          postgresql://postgres:[YOUR-PASSWORD]@db.xxxx.supabase.co:5432/postgres?sslmode=require
                        </pre>
                      </li>
                      <li>Paste into the Database (BYODB) settings panel.</li>
                    </ol>

                    <h5>Option B: Neon (Serverless Postgres)</h5>
                    <ol>
                      <li>
                        Go to <a href="https://neon.tech" target="_blank" rel="noreferrer">neon.tech</a> and create a free project.
                      </li>
                      <li>On the project dashboard, locate the <strong>Connection details</strong>.</li>
                      <li>
                        Select <strong>Connection string</strong>:
                        <pre className="code-snippet">
                          postgresql://[USER]:[PASSWORD]@[ENDPOINT].neon.tech/neondb?sslmode=require
                        </pre>
                      </li>
                    </ol>
                  </div>
                </div>
              )}

              {/* GUIDE TAB 3: OPEN SQL DEMO */}
              {guideTab === "opensql_demo" && (
                <div className="guide-section">
                  <h4>PostgreSQL DDL & Sample Query</h4>
                  <p>Execute this DDL in your cloud SQL editor to create the standard SFLIGHT table:</p>
                  <pre className="code-snippet">{SAMPLE_POSTGRES_DDL}</pre>

                  <div className="guide-actions-bar">
                    <button
                      className="btn-primary"
                      onClick={() => {
                        const demoName = "z_supabase_demo.abap";
                        if (!files[demoName]) {
                          const updated = {
                            ...files,
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
                          };
                          setFiles(updated);
                          try {
                            localStorage.setItem("abapc_vfs_files", JSON.stringify(updated));
                          } catch {}
                        }
                        setShowGuideModal(false);
                        switchActiveFile(demoName);
                      }}
                    >
                      + Load Demo into Virtual File System
                    </button>
                  </div>
                </div>
              )}
            </div>

            <div className="modal-footer">
              <button className="btn-secondary" onClick={() => setShowGuideModal(false)}>
                Close Guide
              </button>
            </div>
          </div>
        </div>
      )}

      {/* ==================================================================== */}
      {/* FOOTER BAR                                                          */}
      {/* ==================================================================== */}
      <footer className="abapc-footer">
        <span>ABAPC Studio 5.0.1 • Stateless Execution Playground</span>
        <span>Bring Your Own Database (BYODB) Enabled</span>
        <span>Localhost 127.0.0.1:7345</span>
      </footer>
    </div>
  );
}
