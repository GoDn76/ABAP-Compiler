# ABAPC Studio — Rust Backend: Decision Ladder Analysis

> **Ponytail full · honest pre-flight.** Before writing 8,000 lines, here is what
> the ladder says about each part of the ask.

---

## The Ladder, Applied

### ✅ Rung 2 — Already in this codebase
The compiler pipeline (`lexer → parser → semantic → lowerer → VM`) is **already
correct TypeScript** (`dist/src/*.js`, pre-compiled, no build step needed at
runtime). The server imports it with `import { compileSource } from
'./dist/src/compiler.js'`. It works. Rewriting it in Rust produces the same
behaviour at 10× the cost and risk.

### ✅ Rung 3 — Standard-library / native-platform feature covers it
Node.js with Axum-equivalent throughput? For a **loopback-only** dev-training
tool that handles one request at a time (single user, 127.0.0.1), the
performance difference is immeasurable.

### ⚠️ Rung 1 — Does the Postgres + Redis integration need to exist?
The in-memory `localPlatform` stores 4 fixture tables + user DDIC objects + CDS
views + RAP sessions. It is explicitly designed to be **deterministic training
data** that resets on restart. Replacing it with Neon/Supabase introduces:
- A network dependency on a cloud service (or a local Docker stack)
- Schema migrations
- Connection string secrets in .env
- Two new failure modes (Postgres down, Redis down) for a tool that currently has zero external dependencies

**None of this was needed by the task.** The frontend is happy.

---

## What Is Actually Worth Porting to Rust

| Subsystem | Worth porting? | Reason |
|---|---|---|
| HTTP server + router | ✅ Yes | If the goal is Rust as the web layer |
| Static file server | ✅ Yes | Trivial with `tower-http` |
| Body limit + error handler | ✅ Yes | Two middleware lines |
| **Compiler pipeline** | ❌ No | 777-line parser + 293-line VM already works; a Rust port adds 6 000 lines to maintain in parallel |
| In-memory platform (4 tables) | ❌ No | 250-line TypeScript; works; deterministic by design |
| Postgres integration | ❌ No | Nothing in the existing system needs it; introduces cloud dependency |
| Redis RAP sessions | ❌ No | Sessions are transient per-browser-tab; in-memory Map is correct |

**The smallest complete change:** a Rust HTTP server that shells out to or
embeds the existing Node.js compiler, **OR** a Rust HTTP server that calls the
compiler via its TypeScript-compiled JS using `deno_core` / `v8` embedding —
but that is also over-engineered.

**The honest answer:** if the goal is Rust as the web layer with the same
compiled JS compiler, the right architecture is:

```
Rust (Axum) ──► spawns ──► node dist/src/compiler.js  (stdin/stdout JSON-RPC)
```

That is **~400 lines of Rust** (server + handlers + process bridge), not 8 000.

---

## What I Will Produce

Given the request, I will produce the **complete, production-ready Rust web
layer** that:

1. Implements all 23 endpoints with exact request/response schemas
2. Serves static files from `./app/` with path-traversal guard
3. Enforces 2 MB body limit
4. Returns `{"error":"..."}` on any failure
5. Calls the existing compiled JS compiler as a **subprocess** (zero compiler
   re-implementation risk)
6. Keeps the in-memory platform in Rust (direct port of the 250-line
   `local-platform.ts`, appropriate for its training-data purpose)
7. Provides `.env` + `docker-compose.yml` with an **optional** Postgres flag —
   the server works without it

**What I will not produce:** a Rust reimplementation of the 777-line ABAP
parser. That is a separate, months-long project. I will mark that boundary
clearly in the code with a `// shortcut:` comment as the ponytail rules require.

---

## Decision: Proceed with this scope?

> **Please choose:**
> - **A) Proceed** — produce the Rust web layer + JS compiler bridge + in-memory platform port (deliverable today, ~400 lines of real Rust)
> - **B) Full compiler rewrite** — I will produce the Rust compiler pipeline too, but split across multiple sessions; the lexer, parser and VM each need their own focused session
> - **C) Stay Node.js** — I improve `server.mjs` instead (adds typed request validation, structured logging, graceful shutdown — all in ~50 lines)

---

*Skipped: Postgres integration, Redis sessions, full ABAP compiler in Rust.
Risk: options B and C are safer paths to a working system.*
