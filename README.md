# pincher

Local-first reflex engine that stores intent-to-action pairs, matches incoming natural-language intents with vector similarity, and executes the matched action without calling an LLM.

## Quickstart

Install from source:

```bash
# One-line installer (builds from source)
curl -fsSL https://raw.githubusercontent.com/purplepincher/pincher/main/install.sh | bash
```

Or build manually:

```bash
git clone https://github.com/purplepincher/pincher.git
cd pincher
cargo build --release -p pincher-cli
cp target/release/pincher ~/.local/bin/
```

Verify the install and try the built-in reflexes:

```bash
pincher status
pincher doctor
pincher do "system.info"
pincher reflexes
```

`cargo install pincher` is not available — the crates are not on crates.io yet.

## Usage

### Teach a new reflex

`pincher teach` interactively prompts for an intent and an action, then stores them as a reflex.

```bash
$ pincher teach
Intent: say hello
Action (e.g., system.info, ls -la, or SQL): $ echo hello
✅ Taught: intent="say hello" → action="$ echo hello" (reflex_id=..., confidence=0.50)
```

Actions that start with `$` run as shell commands; anything else is treated as a SQL statement or dispatched to a built-in reflex.

### Execute an exact intent

```bash
$ PINCHER_LOG_LEVEL=error pincher do "say hello"
🔍 Executing intent: say hello
✅ Execution result:
  Output:     hello
  Confidence: 0.50
  Match type: exact
  Latency:    36 ms
  Reflex ID:  b7bb13dc-6a8f-4324-8d56-572d18769e20
```

### Run a built-in reflex

The database is seeded with built-in reflexes such as `system.info`, `process.list`, `git.status`, `docker.ps`, and `env.get`.

```bash
$ PINCHER_LOG_LEVEL=error pincher do "system.info"
🔍 Executing intent: system.info
✅ Execution result:
  Output:     {
  "cpu_count": 8,
  "hostname": "Eileen",
  "os": "Ubuntu",
  "os_version": "24.04",
  "ram_total_mb": 3533,
  "ram_used_mb": 1607,
  "uptime_secs": 19288
}
  Confidence: 1.00
  Match type: exact
  Latency:    53 ms
```

### List stored reflexes

```bash
$ pincher reflexes
📋 Stored reflexes (11 total):
  1. system.info (confidence: 1.00, invoke_count: 1) [0c56a79b]
  2. file.read   (confidence: 1.00, invoke_count: 0) [b8c06948]
  ...
  11. say hello  (confidence: 0.50, invoke_count: 1) [b7bb13dc]
```

### Pack and move state

```bash
pincher pack --output agent.nail
pincher run --bundle agent.nail "system.info"
pincher unpack --bundle agent.nail
```

## How it works

1. **Store** — `pincher teach` embeds the intent into a 384-dimensional vector and stores the intent-action pair in a local SQLite database (`~/.pincher/reflexes.db` by default). The database uses the `sqlite-vec` extension for vector search.
2. **Match** — `pincher do <intent>` embeds the input and searches the stored vectors. The matcher classifies the best result as:
   - **Exact** — cosine similarity ≥ 0.80
   - **Similar** — cosine similarity 0.55–0.80 (the result is returned with a warning that it may need review)
   - **Novel** — cosine similarity < 0.55 (no reflex fires)
3. **Execute** — matched actions run through a capability-based sandbox (bubblewrap/landlock when available) or a restricted fallback executor. Built-in intents such as `system.info` map to Rust functions rather than shell commands.
4. **Learn** — every successful execution increments `invoke_count` and nudges confidence up; failures nudge it down. Confidence is clamped to `[0.05, 0.95]`.

The matching layer is deterministic: without the ONNX model, the engine uses a SHA-256 trigram hash fallback in the same 384-dimensional space. This still matches exact intents but is far less semantically aware than the ONNX path.

## Configuration and options

### CLI flags

- `--db <path>` / `PINCHER_DB` — database path (default: `~/.pincher/reflexes.db`)
- `--log-level <level>` / `PINCHER_LOG_LEVEL` — tracing log level (default: `warn`)

### Subcommands

| Command | Purpose |
|---------|---------|
| `pincher status` | Engine health, reflex count, database path |
| `pincher doctor` | Diagnostic: ONNX model, SQLite, sandbox, disk, fingerprint |
| `pincher teach` | Interactive prompt to store a new intent→action reflex |
| `pincher do "..."` | Execute natural language through the reflex engine |
| `pincher reflexes` | List stored reflexes with confidence and invocation counts |
| `pincher compile --workspace <path>` | Read a `pincher.toml` manifest and compile to WASM |
| `pincher mature --manifest <path>` | Adversarial fuzzing seed expansion |
| `pincher bench` | Embedding latency benchmark |
| `pincher shell-info` | Hardware fingerprint |
| `pincher pack --output <file>` | Bundle agent state into a `.nail` archive |
| `pincher unpack --bundle <file>` | Extract and verify a `.nail` archive |
| `pincher run --bundle <file> <input>` | Execute a bundle against user input |
| `pincher publish --bundle <file> --token <token>` | Publish a bundle to a registry |
| `pincher update` | Check installed bundles for updates |
| `pincher gastrolith <create|validate|migrate>` | Checkpoint migration helpers |

### Cargo features

Build with optional features:

```bash
cargo build --release -p pincher-cli --features onnx,landlock
```

| Feature | Effect |
|---------|--------|
| `onnx` | Load `all-MiniLM-L6-v2` via ONNX Runtime for real semantic embeddings |
| `landlock` | Enable Linux Landlock sandboxing (kernel 5.13+) |
| `wasmtime` | Execute WASM guest reflex modules |
| `ternary-kernel` | SIMD-optimized cosine/L2 kernels for aarch64 NEON |

To use ONNX, download the model first:

```bash
# With the onnx feature enabled, the engine looks in ~/.pincher/models by default
curl -L -o ~/.pincher/models/all-MiniLM-L6-v2-int8.onnx \
  https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2/resolve/main/onnx/model_int8.onnx
```

## Limitations

- **No embedded LLM.** `pincher` does not call an LLM on a miss. Novel intents are surfaced to the caller; an external sidecar or workflow must turn them into new reflexes via `teach`.
- **Semantic matching needs ONNX.** The default hash fallback matches exact intents reliably but does not understand paraphrases well.
- **Sandboxing is optional.** If `bwrap` and Landlock are unavailable, commands fall back to a restricted `std::process::Command` executor.
- **Linux-first.** Several security and fingerprinting features assume a Linux environment.
- **Not on crates.io.** Distribution is currently source-only via `install.sh` or a `cargo build` from this repository.

## Project layout

- `pincher-core/` — runtime: reflex engine, vector store, embeddings, sandbox, migration, RPC
- `pincher-cli/` — the `pincher` binary
- `hybrid-bridge/` — internal communication crate (not published)
- `ARCHITECTURE.md` — deeper design notes
- `docs/ROADMAP.md` — planned work
- `CONTRIBUTING.md` — contribution guidelines
- `GETTING_STARTED.md` — longer tutorial

## License

MIT OR Apache-2.0
