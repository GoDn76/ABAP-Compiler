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

## Running with Docker Compose

Spin up the Rust server, Redis session buffer, and PostgreSQL database:

```bash
cd "Project req"
docker-compose up --build
```
