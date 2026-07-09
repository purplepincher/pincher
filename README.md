# pincher

Local-first reflex engine: store intent-to-action pairs, match incoming natural-language intents by vector similarity, and execute the matched action without calling an LLM. A genuine miss is handed back to the caller so an LLM or sidecar can turn it into a new reflex.

## Background

LLM-backed systems face a trade-off. Calling the model for every request is flexible but slow and expensive, even for trivial tasks. Hardcoding the common paths is fast and cheap, but breaks on any phrasing or adjacent request the developer didn't foresee. `pincher` sits between these two approaches.

It keeps a local, learnable map of **reflexes** — intent-to-action pairs — and fires them in milliseconds when the intent is recognized. When the intent is not recognized, it does not guess; it returns a **novel** result so a smarter layer can decide what to do next. The LLM stops being the first responder for every request and becomes the compiler for the requests you have not taught yet.

## The core idea, taught in place

A **reflex** is one piece of knowledge: a natural-language **intent** (what the user wants) paired with an **action** (what to do about it). `pincher teach` stores that pair in a local SQLite database. `pincher do <intent>` looks it up and runs the action.

The lookup is not a string comparison. The engine converts the intent text into a 384-dimensional **embedding** — a vector that captures meaning — using either an ONNX model (`all-MiniLM-L6-v2`) or a deterministic SHA-256 trigram hash fallback if the model is absent. It then searches stored reflex embeddings with **cosine similarity** (a score from -1 to 1 measuring how close two vectors point in the same direction) and classifies the best match into one of three buckets:

- **Exact** — cosine similarity ≥ 0.80. The engine runs the action immediately.
- **Similar** — cosine similarity between 0.55 and 0.80. The engine runs the action but flags it as uncertain; the output is annotated so a caller can review or refine it.
- **Novel** — cosine similarity < 0.55. Nothing fires. The engine returns the best score it found and lets the caller route to an LLM, a human, or `pincher teach`.

These thresholds live in `MatchThresholds` in `pincher-core/src/reflex/matcher.rs` and are calibrated for `all-MiniLM-L6-v2`.

### A concrete example of a known intent vs. a genuine miss

Suppose you teach this reflex:

```bash
$ pincher teach
Intent: say hello
Action (e.g., system.info, ls -la, or SQL): $ echo hello
```

`pincher teach` embeds `"say hello"` and stores the pair with a default confidence of **0.50**. Now the engine sees three different inputs:

1. `"say hello"` — exact string match found in the database. The matcher computes the real cosine similarity between the query embedding and the stored embedding, classifies it as **Exact**, and runs `$ echo hello`. No network call.
2. `"greet me"` — no exact string match, but the embedding is close enough to `"say hello"` that cosine similarity lands in the 0.55–0.80 range. The engine classifies it as **Similar**, runs the action, and marks the output with a warning that a human or LLM should review it.
3. `"what is the weather in Tokyo?"` — the embedding is far from every stored reflex. The best similarity is below 0.55. The engine classifies the input as **Novel**, does not run anything, and returns the miss to the caller. That is the genuine miss: the point where an LLM is actually useful, because there is no local reflex to fire.

This is the reflex/escalation pattern. Known intents stay local, fast, and cheap. Unknown intents escalate cleanly.

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
cargo build --release -p pincher
cp target/release/pincher ~/.local/bin/
```

Verify the install and try the built-in reflexes:

```bash
pincher status
pincher doctor
pincher do "system.info"
pincher reflexes
```

`cargo install pincher` is not available — the crates are not on crates.io yet. You can confirm this with `cargo search pincher`; no `pincher` or `pincher-core` package is published.

## Usage

### Teach a new reflex

`pincher teach` interactively prompts for an intent and an action, then stores them as a reflex.

```bash
$ pincher teach
🤖 Interactive Teach Mode
  Enter an intent (what you want to do) and an action (how to do it).
  Type 'quit' on either prompt to exit.

Intent: say hello
Action (e.g., system.info, ls -la, or SQL): $ echo hello
✅ Taught: intent="say hello" → action="$ echo hello" (reflex_id=b7bb13dc-6a8f-4324-8d56-572d18769e20, confidence=0.50)
```

Actions that start with `$` run as shell commands; SQL statements run against the local SQLite database; built-in intents dispatch to Rust functions; anything else is treated as a shell command.

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

The database is seeded with 10 built-in reflexes: `system.info`, `file.read`, `file.write`, `process.list`, `process.kill`, `network.ping`, `git.status`, `git.diff`, `docker.ps`, and `env.get`. Built-in intents map to Rust functions rather than shell commands.

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
  Match type: builtin
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
2. **Match** — `pincher do <intent>` embeds the input and searches stored vectors. The matcher first tries an exact string match, then falls back to sqlite-vec nearest-neighbor search and re-ranks the top candidates with cosine similarity. It classifies the best result as:
   - **Exact** — cosine similarity ≥ 0.80
   - **Similar** — cosine similarity 0.55–0.80 (the result is returned with a warning that it may need review)
   - **Novel** — cosine similarity < 0.55 (no reflex fires)
3. **Execute** — before any action runs, the **veto engine** checks it. A veto is a safety policy that can **Allow**, **Deny**, or **RequireConfirmation** for a command. The default rule set blocks patterns such as `rm -rf /`, `mkfs`, `dd if=...`, writes to `/etc`, `/sys`, `/proc`, `/boot`, or `/dev`, network commands like `curl`, `wget`, `ssh`, and `nc`, and encoded execution tricks like `base64 -d | sh`, `eval`, and `exec`. If the action passes the veto, it runs through a **sandbox** — a capability-based isolation layer that uses bubblewrap (`bwrap`) and Linux Landlock (when the `landlock` feature is enabled) to restrict what the command can see and do. If neither is available, it falls back to a restricted `std::process::Command` executor with a minimal environment.
4. **Learn** — every successful execution increments `invoke_count`. The confidence formula is implemented and tested (success adds 5% of the remaining gap to 1.0, failure subtracts 10% of the current value, clamped to `[0.05, 0.95]`), **but the current `do`/`do_command` path does not yet invoke it**: only `invoke_count` is updated on each run. The `confidence_update` method exists on `ReflexEngine` and is exercised by unit tests, but it is wired only into the separate `execute()` code path, which the CLI does not call. Confidence therefore stays at its taught value (0.50 for user reflexes, 1.00 for built-ins) until that wiring is completed. This is tracked as a known gap, not an intended design.

The matching layer is deterministic: without the ONNX model, the engine uses a SHA-256 trigram hash fallback in the same 384-dimensional space. This still matches exact intents reliably but is far less semantically aware than the ONNX path.

## Configuration and options

### CLI flags

- `--db <path>` / `PINCHER_DB` — database path (default: `~/.pincher/reflexes.db`)
- `--log-level <level>` / `PINCHER_LOG_LEVEL` — tracing log level (default: `warn`)

### Subcommands

The status markers below indicate what is verified against the actual code, not just what the command name implies:

- ✅ **real today** — traced to working code and (where applicable) passing tests
- ⚠️ **real but conditional** — works, but needs something external (a server, a key)
- 🔮 **aspirational / later phase** — described as a direction; the command prints output but does not do the work it names

| Command | Purpose |
|---------|---------|
| `pincher status` | ✅ Engine health, reflex count, database path |
| `pincher doctor` | ✅ Diagnostic: ONNX model, SQLite, sandbox, disk, fingerprint |
| `pincher teach` | ✅ Interactive prompt to store a new intent→action reflex |
| `pincher do "..."` | ✅ Execute natural language through the reflex engine |
| `pincher reflexes` | ✅ List stored reflexes with confidence and invocation counts |
| `pincher compile --workspace <path>` | 🔮 Reads a `pincher.toml` manifest and *prints* a compilation report, but does not actually invoke a WASM toolchain. The output is a simulation — it always reports `[SUCCESS]` regardless of input. Real WASM compilation is a planned future phase. |
| `pincher mature --manifest <path>` | 🔮 Reads a manifest and *prints* an "expanded seed matrix" whose size is derived from line count (`lines × 4`). No actual embedding, fuzzing, or database writes occur. Real adversarial fuzzing is a planned future phase. |
| `pincher bench` | ✅ Embedding latency benchmark — embeds five sample phrases and reports average latency. Does not exercise the teach/match/execute pipeline. |
| `pincher shell-info` | ✅ Hardware fingerprint |
| `pincher pack --output <file>` | ✅ Bundle agent state into a `.nail` archive |
| `pincher unpack --bundle <file>` | ✅ Extract and verify a `.nail` archive |
| `pincher run --bundle <file> <input>` | ✅ Execute a bundle against user input |
| `pincher publish --bundle <file> --token <token>` | ⚠️ Publishes a bundle to a registry via `curl`. Works, but requires a running registry server at `PINCHER_REGISTRY_URL` (default `https://registry.pincher.dev`, which is not publicly available). |
| `pincher update` | ⚠️ Checks installed bundles for updates via `curl`. Works, but requires a running registry server (see `publish` above). |
| `pincher gastrolith <create\|validate\|migrate>` | ✅ Checkpoint migration helpers — `create` writes a JSON checkpoint, `validate` checks it, `migrate` verifies a `.nail` bundle and unpacks it. |

### Cargo features

Build with optional features:

```bash
cargo build --release -p pincher --features onnx,landlock
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

## Library modules beyond the CLI

`pincher-core` exposes several modules as public library APIs that are **real and unit-tested** but are **not wired into the `pincher` CLI's teach/do path**. They are available to anyone depending on `pincher-core` as a Rust crate; the CLI currently exercises only the reflex engine, embedding, database, veto/sandbox, and migration subsystems.

| Module | What it does | Status |
|--------|-------------|--------|
| `immunology` | Antigen detection (prompt-injection, malicious-action, resource-abuse, stale-reflex patterns via regex) and persistent immune memory (antibodies stored in SQLite with generation counts). | ✅ Real, tested, standalone |
| `resource` | PID resource controller with a three-state degradation model (Normal / Light / Critical) that smooths CPU/RAM readings and decides whether to skip the LLM sidecar. | ✅ Real, tested, standalone |
| `route` | Ternary-weighted room-routing graph (+1 / 0 / −1 edges) with shortest-path search, spectral clustering, label-propagation community detection, and signed modularity scoring. | ✅ Real, tested, standalone |
| `rpc` | Unix-domain-socket JSON-RPC server (`ping`, `embed`, `match`, `teach`, `status`) for driving the engine from a Python sidecar. | ✅ Real, compiled, standalone |
| `capability` | Signed capability tokens and capability manifests for granting scoped permissions. | ✅ Real, compiled |
| `carapace` | Host/guest scaffolding for WASM guest reflex modules (requires the `wasmtime` feature). | ✅ Real, feature-gated |
| `intent` | Intent-contract schema and validation types. | ✅ Real, compiled |

A newcomer should treat these as "building blocks available in the library" rather than "features the CLI uses today." See [`EDUCATIONAL_NOTES.md`](./EDUCATIONAL_NOTES.md) for a deeper look at the design patterns behind them.

## Tests

`pincher-core` contains 184 `#[test]` functions in source. With default features (no `onnx`/`landlock`/`wasmtime`/`ternary-kernel`), **174 run and pass**; the remaining ~10 are gated behind optional feature flags (e.g. the WASM `carapace` guest tests and the `ternary-kernel` SIMD tests). `hybrid-bridge` has 64. There are also 3 end-to-end tests in `tests/e2e_runtime_test.rs`, but that file imports `BundleSecurityEngine`, which does not exist in `pincher-core`, so the end-to-end suite does not compile as written. The runnable suites are:

```bash
cargo test -p pincher-core --lib    # 174 passed, 0 failed (default features)
cargo test -p hybrid-bridge --lib   # 64 passed, 0 failed
```

Full workspace builds are heavy because of the `sqlite-vec` (compiles a C extension) and optional `ort` (ONNX Runtime) dependencies; a clean `cargo test` can take several minutes.

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
