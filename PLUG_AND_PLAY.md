# PLUG_AND_PLAY — Pincher

> **A portable reflex runtime for agents. Teach, match, execute — learn as you go.**

## What Is This?

Pincher snaps into any shell and adds adaptive cognition. You teach it reflexes (intent → action pairs), it stores them as vectors in SQLite, and matches new intents against learned reflexes. It's a vector DB as runtime and LLM as compiler — no daemon, no cloud dependency.

## Why Should You Care?

- **Works offline** — ONNX embeddings run locally; hash-based fallback works without any model
- **Gets smarter with use** — confidence scores update on every match; the system optimizes itself
- **Portable agent identity** — pack your entire rig into a `.nail` file, move it to another machine
- **Sandboxed execution** — bubblewrap isolation with veto engine prevents dangerous commands

## Quick Start

```bash
git clone https://github.com/SuperInstance/pincher.git
cd pincher
cargo build --release -p pincher-cli
./target/release/pincher status
```

## ✨ Key Features

- Teach → Match → Execute reflex engine with confidence scoring
- SQLite-backed vector store (384-dim embeddings via sqlite-vec)
- Bubblewrap sandbox with veto-based pre-execution blocking
- `.nail` portable agent packing (tar.zst + BLAKE3 checksums)
- CLI-driven: `pincher do "..."`, `pincher teach`, `pincher reflexes`
- JSON-RPC server for programmatic control

## Next Steps

| Guide | What It Covers |
|-------|----------------|
| [`GETTING_STARTED.md`](./GETTING_STARTED.md) | Build, run, teach your first reflex |
| [`ARCHITECTURE.md`](./ARCHITECTURE.md) | How it works under the hood |
| [`API_REFERENCE.md`](./API_REFERENCE.md) | Every public function, type, and trait |
| [`LOW_LEVEL.md`](./LOW_LEVEL.md) | Internals, performance, porting guide |

## Status

**v0.1.0 — Active development.** Core reflex engine is production-grade. WASM guest execution is wired but experimental. Landlock sandboxing needs production testing.
