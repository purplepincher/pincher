# Agent Interface — pincher

> **Role:** Fleet runtime node — ternary reflex engine, sandbox executor, veto engine
> **Language:** Rust
> **Build:** `cargo build --release`

---

## What an Agent Can Do Here

### Primary Actions

| Action | Entry Point | Description |
|--------|-------------|-------------|
| Evaluate a reflex | `cargo run -- evaluate <reflex-name> --input <data>` | Run a ternary-weighted reflex through the engine |
| Build the engine | `cargo build --release` | Compile pincher core with sandbox support |
| Run test suite | `cargo test` | Unit + integration tests for reflex engine |
| Run clippy | `cargo clippy -- -D warnings` | Strict lint pass |
| Publish a nail | `./scripts/publish_nail.sh` | Format + commit results as a Nail message |
| Check veto rules | `cargo run -- veto-rules` | Dump current veto engine configuration |
| Status check | `cargo run -- status` | Runtime health, reflex count, sandbox state |

### Secondary Actions

- **Create a new reflex:** Add entry to `reflexes/` with ternary weights, register in engine
- **Inspect session state:** Check `SESSION-STATE.md` for active reflex state
- **Simulate runs:** Run `SIMULATION_RUNS*.md` scenarios via `cargo run -- simulate <id>`

---

## Environment Variables Required

```bash
# Required
GITHUB_TOKEN=<ghp_...>           # GitHub API access
DEEPINFRA_API_KEY=<key>          # LLM inference API
OPENAI_API_KEY=<key>             # Fallback LLM API

# Optional but strongly recommended
DEEPINFRA_API_URL=https://api.deepinfra.com/v1
OPENAI_API_URL=<your-openai-url> # Custom endpoint

# Pincher Runtime
PINCHER_RAM_LIMIT_MB=1024
PINCHER_CPU_LIMIT_PERCENT=80
PINCHER_LOG_LEVEL=info

# Fleet Integration
FLEET_HOMESERVER=<matrix-server>
FLEET_MATRIX_USER=<user>
FLEET_MATRIX_PASSWORD=<password>
I2I_VESSEL=/tmp/i2i-vessel
```

---

## Entry Points

### CLI
```bash
# Full build
cargo build --release && ./target/release/pincher

# Direct cargo run
cargo run -- [subcommand] [args]
```

### Library (crate)
```rust
use pincher_core::route::RouteState;
use pincher_core::engine::ReflexEngine;
use pincher_core::security::VetoEngine;

// Embed the reflex engine
let mut engine = ReflexEngine::new();
let result = engine.evaluate(&[("key", "value")]);
```

### Tests
```bash
cargo test                    # All tests
cargo test --test integration # Integration tests only
cargo test veto_engine        # Veto engine specific tests
```

---

## How to Report Back Results

After completing work in this repo, use the **Nail protocol** to report back:

1. **Write a Nail:** Create `memory/<date>-nail-<summary>.md` in construct-coordination:
   ```markdown
   ## Nail: [Summary]
   
   **Origin:** pincher
   **Timestamp:** <ISO-8601>
   **Action:** <what was done>
   **Result:** <output/status>
   **Artifacts:** <any files created/modified>
   ```
2. **Commit & push** your changes with a descriptive message
3. **Log to daily memory** — add entry in `memory/YYYY-MM-DD.md`

---

## Inter-repo Communication

| Repo | Dialogue | Protocol |
|------|----------|----------|
| **construct-coordination** | File-based notes + Nail messages | `notes/pincher/` |
| **i2i-vessel** | Inter-instance binary messaging | Unix socket at `I2I_VESSEL` |
| **fleet-command-bridge** | Fleet-level commands | Matrix messages |
| **oracle1** (via i2i) | Orchestration requests | i2i binary protocol |

---

## Dev Container

This repo includes a `.devcontainer/` with Rust toolchain pre-installed. Open in Codespaces or VS Code Dev Containers for instant L3 readiness.

```bash
# Open in codespace
gh codespace create --repo purplepincher/pincher
```
