# Architecture — Pincher

> *A reflex runtime for agents — vector DB as runtime, LLM as compiler. The core loop is Teach → Match → Execute, with SQLite-backed vector storage and sandboxed execution.*

## Design Goals

1. **Offline-first** — Core reflex matching must work without internet, without cloud, without LLM. Embeddings are local; hash fallback covers the zero-dependency case.
2. **Self-improving** — Every invocation updates confidence scores. The system gets faster and more reliable the more it's used.
3. **Portable** — A `.nail` file packs the entire agent identity. Move between machines with a single file.
4. **Safe** — Sandboxed execution (bubblewrap, Landlock) with veto engine blocking dangerous patterns.

## High-Level Overview

```
┌──────────────────────────────────────────────────────────────────┐
│                        PINCHER RUNTIME                           │
│                                                                  │
│  User Intent: "show running containers"                          │
│       │                                                          │
│       ▼                                                          │
│  ┌─────────────────────┐                                         │
│  │  Reflex Engine       │───────────────────────┐                │
│  │  (Teach→Match→Exec)  │                       │                │
│  └─────────┬───────────┘                       │                │
│            │                                    │                │
│       ┌────▼────┐                         ┌────▼────┐           │
│       │  Embed  │                         │  Teach  │           │
│       │ (ONNX/  │                         │ (Store  │           │
│       │  Hash)  │                         │ Reflex) │           │
│       └────┬────┘                         └─────────┘           │
│            │                                                      │
│       ┌────▼────┐                                  ┌──────────┐  │
│       │  Match  │  0.55–0.80 → Confirm              │ LLM      │  │
│       │ (vec    │───▶ ┌──────────────────────┐      │ Compiler │  │
│       │  search)│     │ Confidence scoring    │      │(optional)│  │
│       │         │     │ Veto engine check     │      └──────────┘  │
│       └────┬────┘     │ Sandbox (bwrap)       │                   │
│            │          │ Execute + log result  │                   │
│            │          └──────────────────────┘                   │
│            ▼                                                      │
│  ┌──────────────────────────────────────────────────┐             │
│  │                 SQLite (reflexes.db)              │             │
│  │  ┌─────────────────────────────────────────┐     │             │
│  │  │  reflexes table: intents, embeddings,    │     │             │
│  │  │  confidence, invoke_count, action_sql    │     │             │
│  │  └─────────────────────────────────────────┘     │             │
│  │  │  sessions, actions, shell fingerprints       │             │
│  └──────────────────────────────────────────────────┘             │
└──────────────────────────────────────────────────────────────────┘
```

## Core Components

### Reflex Engine (`pincher-core/src/reflex/`)

**Purpose:** The heart of Pincher — matches intents against learned reflexes, executes matched commands, updates confidence scores.

**Key types:**
- `ReflexEngine` — Main engine orchestrating teach, match, execute
- `Reflex` — A single learned reflex (intent, action, embedding, confidence)
- `MatchType` — Exact (≥0.80), Similar (0.55–0.80), Novel (<0.55)
- `MatchThresholds` — Configurable similarity thresholds
- `Execution` — Execution result with exit code, stdout, timing

### Embedding (`pincher-core/src/embed/`)

**Purpose:** Converts text intents to 384-dimensional vectors for similarity search.

**Key types:**
- `EmbedError`, `EmbedResult` — Embedding operation results
- Two backends: ONNX Runtime (all-MiniLM-L6-v2) and hash-based fallback

### Database (`pincher-core/src/db/`)

**Purpose:** SQLite-backed persistent store for reflexes, sessions, and shell fingerprints.

**Key types:**
- `Database` — SQLite database handle with vector search (sqlite-vec)
- `ReflexRow`, `ActionLogRow`, `SessionRow`, `ShellRow` — Schema types
- Vector operations: `embed_to_bytes`, `bytes_to_embed`

### Sandbox (`pincher-core/src/sandbox/`)

**Purpose:** Isolated execution environment for reflex commands.

**Key types:**
- Bubblewrap-based sandbox with configurable mounts and permissions
- Falls back to bare `std::process::Command` with warning
- Feature-gated with `landlock` for kernel-level sandboxing

### Security & Veto (`pincher-core/src/security/`)

**Purpose:** Pre-execution command validation and signed capability tokens.

**Key types:**
- `VetoEngine` — Pattern-based blocking (rm -rf /, mkfs, fork bombs)
- `Capability`, `CapabilityManifest`, `CapabilityToken` — Signed tokens
- `LandlockRule`, `SandboxConfig` — Sandbox configuration

### Migration / Packing (`pincher-core/src/migration/`)

**Purpose:** Portable agent identity — pack/unpack `.nail` files.

**Key types:**
- `.nail` format: tar.zst archive with BLAKE3 checksums
- Contains: manifest.json, reflexes.db, identity.json, config.toml
- `AgentIdentity`, `AgentPreferences`, `NailManifest`

### RPC (`pincher-core/src/rpc/`)

**Purpose:** JSON-RPC server for programmatic control.

**Key types:**
- `JsonRpcRequest`, `JsonRpcResponse` — Protocol types
- `EngineCommand` — Supported commands
- `start_rpc_server` — Entry point for RPC server

### CLI Frontend (`pincher-cli/`)

**Purpose:** Clap-based CLI exposing all functionality.

**Commands:** `status`, `doctor`, `teach`, `do`, `reflexes`, `compile`, `mature`, `bench`, `shell-info`, `pack`, `unpack`, `run`, `publish`, `update`, `gastrolith`

## Data Flow

```
"show running containers"
         │
         ▼
[1] Embed intent → 384-dim vector
         │
         ▼
[2] Vector search in SQLite (sqlite-vec)
         │
         ├──≥ 0.80 (Exact) → Execute directly (~50ms, $0)
         ├──0.55–0.80 (Similar) → Confirm + Execute (~3s, ~$0.001)
         └──< 0.55 (Novel) → Route to LLM-as-Compiler → Store new reflex
         │
         ▼
[3] Veto engine check → Sandbox (bwrap) → Execute
         │
         ▼
[4] Log result → Update confidence → Done
```

## Key Design Decisions

### Decision 1: SQLite as Vector Store

- **Context:** Needed persistent vector search without external infrastructure
- **Chosen approach:** `sqlite-vec` extension stores embeddings in a standard SQLite database
- **Trade-offs:** No distributed search, but zero dependencies and instant setup

### Decision 2: Confidence Scoring

- **Context:** How to make the system self-improving without complex ML
- **Chosen approach:** Additive confidence updates — on success add `0.05 × (1 − current)`; on failure subtract `0.10 × current`. Result clamped to `[0.05, 0.95]`. Implemented in `pincher-core/src/reflex/confidence.rs`.
- **Wiring caveat:** The formula is real and unit-tested, but the current `do`/`do_command` execution path does **not** call `confidence_update` — it only increments `invoke_count`. The update is wired into the separate `execute()` method, which the CLI does not invoke. Confidence therefore remains at its taught value until that wiring is completed.
- **Trade-offs:** Simple and effective, but doesn't account for task difficulty variation

### Decision 3: .nail Portable Format

- **Context:** Agent identity must move between machines
- **Chosen approach:** tar.zst with BLAKE3 checksums — self-contained, verifiable
- **Trade-offs:** Larger than a pure config file, but includes full database and identity

## Dependencies

| Dependency | Why We Use It | Notes |
|-----------|---------------|-------|
| `sqlite-vec` | Vector search in SQLite | Extends SQLite with vec0 virtual table |
| `ort` | ONNX Runtime for embeddings | Feature-gated (`onnx`) |
| `bubblewrap` | Sandboxed execution | External binary |
| `tokio` | Async runtime | CLI and RPC server |
| `serde` / `serde_json` | Serialization | Everywhere |
| `ternary-types` | Ternary fleet integration | Shared types |

## Extension Points

- **New embedding backend** — Implement the `embed` trait (currently ONNX and hash)
- **New sandbox backend** — Add a new sandbox implementation in `sandbox/` (e.g., nsjail, gVisor)
- **Custom veto rules** — Add patterns to the veto engine for domain-specific blocking
- **WASM guest modules** — The `wasmtime` feature enables WASM-based reflex execution

## See Also

- [GETTING_STARTED.md](./GETTING_STARTED.md) — Build and run
- [LOW_LEVEL.md](./LOW_LEVEL.md) — Internal details
- [API_REFERENCE.md](./API_REFERENCE.md) — Full API
- [docs/](./docs/) — ADRs, roadmap, plans
