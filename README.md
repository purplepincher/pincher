<div align="center">
<img src="assets/hermit-crab.jpg" width="320" alt="Hermit crab" />
</div>

# pincher

*A hermit crab doesn't grow a new shell. It finds one that fits, moves in, and makes it home.*

---

pincher is a reflex engine. A reflex is what happens when input meets pattern before thought arrives. Most AI systems think first — route every message through an LLM, burn tokens and milliseconds on questions that have already been answered a hundred times. pincher does the opposite. It responds in <50ms, without an LLM, every time, at zero marginal cost. The LLM is the thinking layer. pincher is the *shell* — the fast, durable structure that catches the simple cases so the thinking layer only fires when it needs to.

There is no question of replacement. Warm-blooded thought and the spinal reflex are not in competition. The cortex teaches the spinal cord. The spinal cord gets faster. The shell protects the signal.

---

## The Shell

A hermit crab's shell is not its body. The shell is infrastructure — important, necessary, but replaceable. The crab carries its body from shell to shell. The reflexes, not the runtime.

pincher is that shell for AI agents. It sits between the agent's intent and the world, intercepting patterns before they reach expensive machinery. Every intent enters the shell. Within a few milliseconds, the shell decides: do I already know how to handle this?

The answer comes from a vector database — every intent the agent has ever encountered, embedded into a 384-dimensional space, scored by confidence. A known reflex fires directly. A semi-known reflex asks for confirmation. An unknown reflex escalates to the LLM, which compiles a new reflex and stores it for next time.

This is not caching. Caching returns the same answer to the same question. pincher returns the right answer to *similar* questions, because it matches on semantic embedding, not exact string. "Show me what's running" and "what processes are active" map to the same reflex. That's not cache. That's *understanding* — baked into the shell, not farmed out to a model.

```
                     ┌──────────────────────────────────────┐
                     │            pincher                    │
                     │   ┌──────────────────────────────┐   │
You say ────▶ [1] ──▶│   │  Reflex Engine               │   │
something     Embed  │   │  ┌─────┐   ┌────────┐       │   │
             (384D)  │   │  │Match│──▶│Execute  │       │   │
                     │   │  │≥0.80│   │Directly │       │   │
                     │   │  └─────┘   └────────┘       │   │
                     │   │  ┌─────┐   ┌────────┐       │   │
                     │   │  │Match│──▶│Confirm  │       │   │
                     │   │  │0.55-│   │+ Execute│       │   │
                     │   │  │0.80 │   └────────┘       │   │
                     │   │  └─────┘   ┌────────┐       │   │
                     │   │  ┌─────┐   │LLM     │       │   │
                     │   │  │Match│──▶│Compiles│       │   │
                     │   │  │<0.55│   │New     │       │   │
                     │   │  └─────┘   │Reflex  │       │   │
                     │   │            └────────┘       │   │
                     │   │  ┌──────────────────────┐   │   │
                     │   │  │ Veto Engine           │   │   │
                     │   │  │ Security → Sandbox   │   │   │
                     │   │  └──────────────────────┘   │   │
                     │   └──────────────────────────────┘   │
                     │              │                       │
                     │              ▼                       │
                     │   ┌──────────────────────────────┐   │
                     │   │    Reflex Database           │   │
                     │   │    (SQLite + sqlite-vec)    │   │
                     │   └──────────────────────────────┘   │
                     │              │                       │
                     │              ▼                       │
                     │   ┌──────────────────────────────┐   │
                     │   │    .nail Bundle               │   │
                     │   │    (Portable Agent State)     │   │
                     │   └──────────────────────────────┘   │
                     └──────────────────────────────────────┘
```

Three-tier compute:

```
Fast  (ms):   Embedding match + reflex execution (no LLM)
Medium (s):   Confirmation + optional execution (low confidence)
Slow   (s):   LLM compilation → new reflex (learning event)
```

Every cycle through this engine makes the agent faster and cheaper. The reflex database grows. The match scores climb. The LLM gets called less and less.

---

## The Signal

A signal is what passes through the shell. A pulse of light through fiber. A word through the air. A number through a wire. The signal is transient by its nature — it exists only in motion, and when the motion stops, the signal is gone.

In pincher, the signal is the **pinch**: a structured message that carries intent from the agent to the reflex engine. It is the atomic unit of the reflex architecture.

```
┌───────────────────────────────────────────┐
│          A Pinch (the signal)             │
├───────────────────────────────────────────┤
│ trigger:  "list running containers"       │
│ intent:   list container resources        │
│ context:  { host: "prod-01", ... }        │
│ action:   docker ps ...                   │
│ safety:   sandbox=none, network=local    │
└───────────────────────────────────────────┘
```

The pinch flows through the shell:
1. **Trigger** — the raw input from the agent
2. **Embed** — converted to a 384-dimensional vector
3. **Match** — compared against every known reflex
4. **Act** — the matched action executes in a sandbox
5. **Learn** — success strengthens the reflex; failure degrades it

The reflex database is the reef. Every teach is a polyp. Every match is a generation. Every confidence adjustment is adaptation.

---

## The Architecture

pincher is a Rust workspace, three crates:

**`pincher-core`** — all the runtime logic. Reflex engine, vector store, embeddings, sandbox, migration, RPC, security, resource control. Feature-gated for optional components.

**`pincher-cli`** — the `pincher` binary. Clap-based, async via tokio, thin wrapper over the core library.

**`hybrid-bridge`** — internal communication backbone between components. Not currently published.

```
pincher-core/src/
├── capability/   # Capability manifests and signed tokens
├── carapace/     # WASM sandbox bridge for guest code
├── db/           # SQLite vector store with sqlite-vec
├── dynamics/     # Veto engine
├── embed/        # ONNX embeddings (all-MiniLM-L6-v2) + hash fallback
├── immunology/   # Pattern-based immune system
├── intent/       # Declarative intent-to-action contracts (Intent.toml)
├── kernel/       # SIMD-optimized compute kernels
├── migration/    # .nail pack/unpack with BLAKE3 + tar.zst
├── reflex/       # The reflex engine (match, execute, teach, confidence)
├── resource/     # Resource budgets and PID controller
├── route/        # Spectral clustering, label propagation, room graphs
├── rpc/          # JSON-RPC server for programmatic control
├── sandbox/      # Bubblewrap isolation
├── security/     # Veto engine, landlock rules
└── shell/        # Hardware fingerprinting
```

#### Feature Flags

| Flag | What It Unlocks |
|------|----------------|
| `onnx` | Real ONNX Runtime embeddings (all-MiniLM-L6-v2) |
| `landlock` | Linux Landlock sandboxing (kernel 5.13+) |
| `wasmtime` | WASM guest module execution |
| `ternary-kernel` | SIMD-optimized cosine/L2/scale kernels (aarch64 NEON) |

Without any features, pincher uses hash-based embedding fallback. It works. It's just less semantically aware — it'll match exact intents perfectly and similar intents less well.

pincher is part of the purplepincher edge-tier family; for a related bare-metal sensor engine, see [plato-engine-block-c](https://github.com/purplepincher/plato-engine-block-c).

---

## Quick Start

Pincher embeds your intents into a 384-dim vector space, fires known reflexes in under 50ms with zero LLM calls, and learns from every miss.

```bash
# One-line install (builds from source)
curl -fsSL https://raw.githubusercontent.com/purplepincher/pincher/main/install.sh | bash
```

```bash
# Or build from source manually
git clone https://github.com/purplepincher/pincher.git
cd pincher
cargo build --release -p pincher-cli
cp target/release/pincher ~/.local/bin/
```

> **Note:** `cargo install pincher` is not available yet — the crates are not published on crates.io. Use `install.sh` or build from source for now.

First five minutes:

```bash
pincher status                 # Is it alive?
pincher reflexes               # What does it know?
pincher do "list files"        # Run intent through the reflex engine
pincher teach                  # Teach it something new (interactive)
pincher doctor                 # Full diagnostic
pincher pack --output crab.nail   # Pack agent state
```

`pincher teach` is genuinely interactive: it prompts for an intent, then an action, stores the pair as a reflex, and loops until you type `quit` or `exit`. It is not a one-line pipeable command.

The CLI:

| Command | What It Does |
|---------|-------------|
| `pincher status` | Engine health, reflex count, database path |
| `pincher doctor` | Full diagnostic — ONNX, SQLite, disk, embeddings |
| `pincher teach` | Interactive: store a new intent→action reflex |
| `pincher do "..."` | Execute natural language through the reflex engine |
| `pincher reflexes` | List all stored reflexes with confidence scores |
| `pincher compile` | Compile workspace manifest → WASM reflex |
| `pincher mature` | Adversarial fuzzing to grow vector coverage |
| `pincher bench` | Benchmark suite (embed latency, teach/match cycles) |
| `pincher shell-info` | Hardware fingerprint of the current machine |
| `pincher pack` | Bundle agent state → portable `.nail` file |
| `pincher unpack` | Load a `.nail` bundle onto this machine |
| `pincher run` | Execute a bundle against user input |
| `pincher publish` | Publish bundle to the reflex registry |
| `pincher gastrolith` | Checkpoint migration management |

Every `pincher do` is a learning event. If the intent matches, the reflex fires and confidence rises. If it doesn't, the LLM compiles a new reflex and stores it. Either way, the agent knows more afterward than it did before.

---

## License

MIT OR Apache-2.0
