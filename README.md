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

## Deploying Frontend to Vercel

The frontend located in `frontend/` (or `app/`) is 100% static and can be deployed directly to Vercel in seconds:

1. **Deploy with Vercel CLI:**
   ```bash
   cd frontend
   vercel deploy --prod
   ```
2. **Configure Remote Backend URL:**
   - In the deployed web application, click the **`⚙ API`** button in the header toolbar (or click the **`LOCAL · CLIENT 001`** system chip in SAP GUI).
   - Enter your VPS backend URL (e.g. `https://your-vps.com:7345`).
   - The setting is saved to `localStorage` and will route all IDE compilation and SAP GUI transactions to your VPS backend over CORS.

---

## Deploying Rust Backend to a 2GB RAM VPS

The Rust backend is compiled to native machine code and operates with an idle footprint of **~12MB to 25MB RAM**, making it exceptionally well-suited for a 2GB RAM / 1 vCPU VPS.

### 1. Cross-Compile or Build on VPS
On the VPS (Ubuntu/Debian):
```bash
sudo apt update && sudo apt install -y build-essential pkg-config libssl-dev
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

cargo build --release
```

### 2. Run as a Systemd Background Service
Copy the provided unit file:
```bash
sudo cp abapc.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable --now abapc
sudo systemctl status abapc
```

---

## Running with Docker Compose

Spin up the Rust server, Redis session buffer, and PostgreSQL database:

```bash
cd "Project req"
docker-compose up --build
```
