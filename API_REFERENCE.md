# API Reference — Pincher

> *Public API surface of the `pincher-core` library. This is the programmatic interface — the CLI (`pincher-cli`) wraps these types. MSRV: per `rust-toolchain.toml`.*

---

## `pincher-core`

### Re-exports

The crate root re-exports the most important types for convenience:

```rust
pub use embed::{cosine_similarity, download_model, EmbedError, EmbedResult, EMBEDDING_DIM};
pub use reflex::{EngineError, EngineResult, EngineStatus, Execution, MatchError, MatchThresholds, MatchType, Reflex, ReflexEngine};
pub use db::{schema::{ReflexRow, SessionRow, ShellRow}, Database, DbError, DbResult};
pub use resource::{PidController, ResourceBudget, ResourceController, ResourceError, ResourceMetrics, ResourceResult, ResourceState, ResourceThresholds};
pub use security::{veto::{VetoDecision, VetoEngine, VetoError, VetoResult, VetoRule}, LandlockRule, SandboxConfig};
pub use capability::{CapabilityManifest, CapabilityToken};
pub use migration::{pack_nail, unpack_nail, verify_nail, fingerprint, compatibility_score, NailManifest, AgentIdentity};
pub use rpc::{start_rpc_server, JsonRpcRequest, JsonRpcResponse, RpcError, RpcResponse};
pub use route::{shortest_paths, all_pairs_shortest_paths, spectral_clustering, modularity, Room, RoomGraph, TernaryGraph};
```

---

### Reflex Engine (`reflex/`)

#### `ReflexEngine`

```rust
pub struct ReflexEngine { /* private fields */ }
```

The central runtime orchestrating teach → match → execute. Contains the database handle, embedder, sandbox, and confidence tracker.

**Methods:**
| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `(db_path: &str) -> Result<Self>` | Create engine with SQLite DB at path |
| `teach` | `(&self, intent, action) -> Result<Reflex>` | Store a new reflex |
| `match_intent` | `(&self, intent) -> Result<MatchResult>` | Find best match for intent |
| `execute` | `(&self, reflex) -> Result<Execution>` | Run reflex in sandbox |
| `do_intent` | `(&self, intent) -> Result<Execution>` | Teach → Match → Execute in one call |
| `reflexes` | `(&self) -> Result<Vec<Reflex>>` | List all stored reflexes |
| `status` | `(&self) -> EngineStatus` | Engine health and metadata |

---

#### `Reflex`

```rust
pub struct Reflex {
    pub id: String,
    pub intent: String,
    pub action_sql: String,
    pub confidence: f64,
    pub invoke_count: u64,
    pub created_at: String,
    pub updated_at: String,
}
```

A single learned reflex — intent text, action command, and metadata.

---

#### `MatchType`

```rust
pub enum MatchType {
    Exact,   // score ≥ 0.80
    Similar, // 0.55 ≤ score < 0.80
    Novel,   // score < 0.55
}
```

Match quality classification.

---

#### `MatchThresholds`

```rust
pub struct MatchThresholds {
    pub exact: f64,   // default: 0.80
    pub similar: f64, // default: 0.55
}
```

Configurable similarity boundaries.

---

#### `Execution`

```rust
pub struct Execution {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u64,
    pub sandboxed: bool,
    pub confidence_delta: f64,
}
```

Result of a reflex execution.

---

### Embedding (`embed/`)

#### `cosine_similarity`

```rust
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f64
```

Cosine similarity between two vectors. Used by the matcher.

#### `EMBEDDING_DIM`

```rust
pub const EMBEDDING_DIM: usize = 384;
```

Dimension of all embedding vectors (all-MiniLM-L6-v2 output size).

---

### Database (`db/`)

#### `Database`

```rust
pub struct Database { /* private fields */ }
```

SQLite-backed store with `sqlite-vec` extension for vector search.

**Methods:**
| Method | Signature | Description |
|--------|-----------|-------------|
| `open` | `(path: &str) -> Result<Self>` | Open or create database |
| `insert_reflex` | `(reflex: &ReflexRow) -> Result<()>` | Store a reflex row |
| `search_similar` | `(embedding: &[f32], limit: u64) -> Result<Vec<ReflexRow>>` | Vector search by embedding |
| `update_confidence` | `(id: &str, delta: f64) -> Result<()>` | Adjust confidence |

---

#### Schema Types

```rust
pub struct ReflexRow {
    pub id: String,
    pub intent: String,
    pub action_sql: String,
    pub embedding: Vec<u8>,
    pub confidence: f64,
    pub invoke_count: u64,
}

pub struct SessionRow {
    pub id: String,
    pub created_at: String,
    pub active: bool,
}

pub struct ShellRow {
    pub fingerprint: String,
    pub shell_type: String,
    pub features: String,
}
```

---

### Sandbox (`sandbox/`)

#### `SandboxConfig`

```rust
pub struct SandboxConfig {
    pub enabled: bool,
    pub read_only_paths: Vec<String>,
    pub writable_paths: Vec<String>,
    pub allow_network: bool,
    pub allow_exec: Vec<String>,
}
```

Sandbox isolation configuration. Uses bubblewrap when available.

---

### Security & Veto (`security/`)

#### `VetoEngine`

```rust
pub struct VetoEngine { /* private fields */ }
```

Pattern-based pre-execution command validation.

**Patterns blocked:** rm -rf /, mkfs, dd if=/dev/zero, fork bombs, etc.

```rust
pub enum VetoDecision {
    Allow,
    Block(String), // reason
}
```

---

### Migration (`migration/`)

#### `.nail` Format Functions

```rust
pub fn pack_nail(output_path: &str) -> PackResult<()>
pub fn unpack_nail(bundle_path: &str) -> PackResult<()>
pub fn verify_nail(bundle_path: &str) -> bool
pub fn fingerprint() -> FingerprintResult<ShellFingerprint>
pub fn compatibility_score(a: &ShellFingerprint, b: &ShellFingerprint) -> f64
```

---

### RPC (`rpc/`)

#### `start_rpc_server`

```rust
pub async fn start_rpc_server(addr: &str) -> Result<()>
```

Starts a JSON-RPC server. Accepts `EngineCommand` requests.

```rust
pub enum EngineCommand {
    Do(String),       // Execute intent
    Teach(String, String), // intent, action_sql
    Status,
    ReflexList,
}
```

---

### Routing (`route/`)

```rust
pub fn shortest_paths(graph: &TernaryGraph, source: usize) -> Vec<f64>
pub fn all_pairs_shortest_paths(graph: &TernaryGraph) -> Vec<Vec<f64>>
pub fn spectral_clustering(graph: &TernaryGraph, k: usize) -> Vec<usize>
pub fn modularity(graph: &TernaryGraph, clusters: &[usize]) -> f64
```

Graph algorithms for ternary fleet routing and topology analysis.

---

### Resource Controller (`resource/`)

```rust
pub struct ResourceController { /* private fields */ }
pub struct ResourceBudget { pub cpu: f64, pub memory: f64, pub disk: f64 }
pub struct PidController { /* PID controller for resource regulation */ }
```

Resource management with proportional–integral–derivative control.

---

### Capability (`capability/`)

```rust
pub struct CapabilityManifest {
    pub id: String,
    pub permissions: Vec<Permission>,
    pub issuer: String,
    pub expires_at: String,
}

pub struct CapabilityToken {
    pub manifest: CapabilityManifest,
    pub signature: Vec<u8>,
}
```

Signed capability tokens for secure agent operations.

---

## Feature Gates

| Feature | What It Enables | Default? |
|---------|----------------|----------|
| `onnx` | Real ONNX Runtime embeddings (all-MiniLM-L6-v2) | No |
| `landlock` | Linux Landlock sandboxing (kernel 5.13+) | No |
| `wasmtime` | WASM guest module execution | No |

## Minimum Supported Rust Version (MSRV)

Per `rust-toolchain.toml`.
