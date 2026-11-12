# ABAPC Studio 5.0.1 - High-Performance Rust Backend

This directory contains the production-ready Rust backend replacement for the Node.js `server.mjs` and TypeScript compiler pipeline.

The frontend (`app/app.js`, `app/gui.js`, `app/index.html`, `app/gui.html`) remains completely untouched and unaware of the backend switch.

---

## Architecture Overview

1. **Rust Server Core (`src/main.rs`, `src/server.rs`, `src/config.rs`)**:
   - Axum-based web server binding to `127.0.0.1:7345` (auto-incrementing up to `7365` if in use).
   - Secure static file server serving `./app/` with path-traversal prevention.
   - Enforces a 2MB maximum request body limit.
   - Sets security headers: `Cache-Control: no-store` and `X-Content-Type-Options: nosniff`.
   - Automatic browser launch to the SAP GUI Simulator (`http://127.0.0.1:7345/sapgui`).

2. **Strict API Contracts (`src/api/`)**:
   - Strictly typed request/response handlers with `serde_json` for all 23 mapped endpoints:
     - **System / File**: `/api/health`, `/api/config`, `/api/executed`, `/api/code`, `/api/examples`, `/api/example`
     - **Compilation & Execution**: `/api/compile`, `/api/execute`, `/api/selection-preview`, `/api/selection-event`
     - **Dictionary / RAP / CDS**: `/api/gui/tables`, `/api/platform/catalog`, `/api/platform/create-object`, `/api/platform/object`, `/api/cds/views`, `/api/cds/activate`, `/api/cds/query`, `/api/rap/read`, `/api/rap/modify`, `/api/rap/commit`, `/api/rap/rollback`
     - **Control**: `/api/stop`

3. **ABAP Compiler Pipeline (`src/compiler/`)**:
   - **Lexer**: Case-insensitive ABAP lexer handling compound keywords (`SELECT-OPTIONS`, `NO-INTERVALS`, etc.), strings with `''` escape, `*` column-1 comments, and `"` inline comments.
   - **Parser**: Recursive descent parser supporting statement terminators (`.`), chained colons/commas (`DATA: x TYPE i, y TYPE string.`), classical reports, events, and the 5.0.1 `SWITCH #(...)` expression fix.
   - **Semantic Analyzer**: Scope-aware symbol table, class hierarchy checks, inheritance validation, and visibility rules.
   - **Lowerer**: Compiles AST into linear stack-based bytecode `ProgramIR`.
   - **VM**: Bytecode execution engine with stack memory, variable state tracking, `SY` system struct (`SY-SUBRC`, `SY-TABIX`, `SY-DBCNT`), `WRITE` output buffer, and selection-screen parameter injection.

4. **Platform & Database Integration (`src/platform/`, `src/db/`)**:
   - In-memory local platform store with relational fixtures (`SCARR`, `SPFLI`, `SFLIGHT`, `ZPRODUCT`).
   - CDS view activation and projection querying.
   - RAP transactional session buffer (staging `CREATE`, `UPDATE`, `DELETE` operations before `COMMIT ENTITIES`).
   - Postgres and Redis integration modules (`db/postgres.rs`, `db/redis.rs`).

---

## Local Development & Testing

### Prerequisites
- [Rust](https://rustup.rs/) (edition 2021, Rust 1.80+)

### Running Locally with Cargo
From the workspace root or inside `Project req`:

```bash
# Run the Rust server (binds to 127.0.0.1:7345)
cargo run --manifest-path "Project req/Cargo.toml"
```

Or `cd "Project req"`:
```bash
cargo run
```

Access the application in your browser:
- **SAP GUI Simulator**: `http://127.0.0.1:7345/sapgui`
- **Compiler IDE**: `http://127.0.0.1:7345`

### Building for Release
```bash
cargo build --release --manifest-path "Project req/Cargo.toml"
```

---

## Parity Verification Testing

To verify that the Rust backend behaves identically to the TypeScript/Node.js backend, run the parity test suite:

### 1. Run via Node.js Parity Runner (15 End-to-End Scenarios)
With the Rust backend running on port 7345:
```bash
node scripts/test-parity.mjs
```
Or specify a custom port or remote URL:
```bash
node scripts/test-parity.mjs --port 7346
node scripts/test-parity.mjs --url https://your-vps.com:7345
```
Or on Windows PowerShell:
```powershell
.\scripts\test-parity.ps1 -Port 7345
```

### 2. Run Native Rust Unit & Integration Tests
```bash
cargo test
```

---

## Local Execution & Running Commands

You can run both the frontend and backend together using the built-in Axum static file server, or run them in separate processes for independent frontend development.

### Method 1: Unified Execution (Recommended)
The Rust server serves both the complete REST API and the frontend UI (`app/` or `frontend/`):

```bash
cd "Project req"
cargo run
```
*(Or optimized: `cargo run --release`)*

Once running:
- **Compiler IDE (ABAP Editor)**: [http://127.0.0.1:7345](http://127.0.0.1:7345)
- **SAP GUI Practice Simulator**: [http://127.0.0.1:7345/sapgui](http://127.0.0.1:7345/sapgui)

*Note: If port 7345 is already in use by another server, the Rust server will automatically bind to the next available port (e.g. 7346).*

---

### Method 2: Running Frontend and Backend Separately

If you prefer to run the frontend independently:

1. **Start the Rust Backend:**
   ```bash
   cd "Project req"
   cargo run
   ```
   *(Backend starts on `http://127.0.0.1:7345` with full CORS enabled)*

2. **Serve the Frontend:**
   In another terminal, serve the `frontend/` folder with any local HTTP server:
   ```bash
   cd "Project req/frontend"
   npx serve -l 3000
   # or using Python:
   # python -m http.server 3000
   ```
3. Open `http://localhost:3000`. In the top toolbar, click **`⚙ API`** and enter `http://127.0.0.1:7345` to route all editor compiles and SAP GUI transactions to your Rust backend.

---

### Method 3: Running with Docker Compose

Spin up the Rust server, Redis session buffer, and PostgreSQL database locally:

```bash
cd "Project req"
docker-compose up --build
```
