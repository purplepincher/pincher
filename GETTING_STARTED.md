# GETTING STARTED — Pincher

> *Estimated time to complete: 10 minutes*

## Prerequisites

- **Rust toolchain** (see `rust-toolchain.toml`)
- Cargo
- (Optional) `bubblewrap` for sandboxed execution
- (Optional) ONNX Runtime for real neural embeddings

## Installation

### From Source

```bash
git clone https://github.com/SuperInstance/pincher.git
cd pincher
cargo build --release -p pincher-cli
cp target/release/pincher ~/.local/bin/
```

### One-Line Install

```bash
curl -fsSL https://raw.githubusercontent.com/SuperInstance/pincher/main/install.sh | bash
```

### Feature Flags

```bash
# Minimal build (hash embeddings only, no sandbox)
cargo build --release -p pincher-cli

# Full build with everything
cargo build --release -p pincher-cli --features "onnx,landlock,wasmtime"
```

| Feature | What It Enables |
|---------|----------------|
| `onnx` | all-MiniLM-L6-v2 ONNX embeddings (better matching) |
| `landlock` | Linux kernel Landlock sandboxing (kernel 5.13+) |
| `wasmtime` | WASM guest module execution |

## Your First 10 Minutes

### 1. Check the Engine

```bash
pincher status
```

This shows engine status, reflex count, and database path.

### 2. Run a Health Check

```bash
pincher doctor
```

Checks ONNX model presence, SQLite integrity, disk space, and embedding readiness.

### 3. Execute Your First Intent

```bash
pincher do "list files in current directory"
```

The engine will attempt to match this against known reflexes. On first run, it may route to the LLM compiler (if configured) or use built-in dispatchers.

### 4. Teach a New Reflex

```bash
pincher teach
```

Follow the interactive prompt to teach a new intent → command pair. The engine will store it with a default confidence of 0.55.

### 5. List Your Reflexes

```bash
pincher reflexes
```

Shows all stored reflexes with their confidence scores, invoke counts, and timestamps.

## Common Patterns

### Using Built-in Intents

The engine ships with built-in dispatchers for common operations:

```bash
pincher do "show system info"
pincher do "what's my disk usage"
pincher do "list docker containers"
pincher do "check git status"
```

### Packing Your Agent

```bash
# Pack the whole rig into a portable .nail file
pincher pack --output my-agent.nail

# Unpack on another machine
pincher unpack --bundle my-agent.nail

# Execute a bundle against input
pincher run --bundle my-agent.nail "check disk space"
```

### Running Without ONNX

If you build without `onnx`, Pincher uses hash-based embedding fallback. Matching will be less precise but works completely offline with zero model download.

## Troubleshooting

| Problem | Likely Cause | Solution |
|---------|-------------|----------|
| `bwrap: command not found` | Bubblewrap not installed | `sudo apt install bubblewrap` or build without sandbox |
| Embedding mismatch / poor matching | ONNX model not downloaded | Run `pincher doctor` to download, or build with `onnx` feature |
| `pincher: command not found` | Binary not in PATH | `cp target/release/pincher ~/.local/bin/` |
| Database locked | Another process using the DB | Check for other pincher instances, set `PINCHER_DB` to a different path |

## Next Steps

- [ARCHITECTURE.md](./ARCHITECTURE.md) — The reflex engine design
- [API_REFERENCE.md](./API_REFERENCE.md) — Full API reference
- [LOW_LEVEL.md](./LOW_LEVEL.md) — Internal implementation details
- [examples/](./examples/) — Example reflexes and usage patterns
