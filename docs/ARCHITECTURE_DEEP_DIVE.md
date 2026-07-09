# Architecture Deep Dive

> Companion to the top-level [README.md](../README.md) and [ARCHITECTURE.md](../ARCHITECTURE.md).
> This document explains *why* certain design decisions were made and maps which
> subsystems are wired into the CLI vs. available only as library APIs.

Every claim below was verified against source code and (where possible) passing
tests on 2026-07-08. The honesty-marker convention from the README applies:

- ✅ real today — traced to working code
- ⚠️ real but conditional — works, but needs something external
- 🔮 aspirational / later phase — described as a direction, not implemented

---

## 1. The three-layer execution-safety model

When `pincher do` matches a reflex and is about to execute an action, the
action passes through **two** safety gates before it runs, and a **third**
layer exists as a standalone library module that is not yet wired into the
execution path. Understanding which layer does what is essential before
adding new safety rules.

### Layer 1 — Veto engine (`pincher-core/src/security/veto.rs`) ✅

The veto engine is a **pre-execution policy gate**. Before any action runs,
`ReflexEngine::check_veto` constructs an `ExecutionContext` and asks the
`VetoEngine` for a `VetoDecision`:

| Decision | What happens |
|----------|-------------|
| `Allow` | Execution proceeds. |
| `RequireConfirmation(reason)` | Execution proceeds, but the reason is logged at `debug` level. The CLI does not currently pause for interactive confirmation — it treats this the same as `Allow` with a log entry. |
| `Deny(reason)` | Execution is aborted; `EngineError::Vetoed` is returned. |

The default rule set (`default_veto_rules()`) blocks:

- Destructive commands: `rm -rf /`, `rm -rf /*`, `rm -rf ~`, `mkfs`, `dd if=`
- System directory writes: `/etc`, `/sys`, `/proc`, `/boot`, `/dev`
- Network commands: `curl`, `wget`, `ssh`, `nc` (blocked as `ForbiddenPattern`)
- Package managers: `apt-get install`, `yum install`, `pip install`
- Evasion techniques: `base64 -d`, `eval`, `exec`, `powershell -enc`, `python -c`, `perl -e`
- A 100 MB file-size cap (`MaxFileSize`)

**Key design decision — pluggable policy trait.** The decision logic lives
behind a `VetoPolicy` trait, not hardcoded in the engine. The default
implementation is `RuleBasedVetoPolicy`, but a downstream consumer can
implement `VetoPolicy` and construct `VetoEngine::with_policy(...)` to swap
in a custom policy (e.g. one that consults an external allowlist or an LLM
confirmation step) without touching the engine's dispatch code. A blanket
`impl VetoPolicy for Box<P>` allows runtime polymorphism.

**Non-obvious behaviour:** the veto engine checks the *intent string* for
built-in reflexes (e.g. `system.info`) but checks the *action string* for
user-taught reflexes. This means a built-in intent like `file.read` is
never vetoed by its action, because the veto sees `"file.read"` not the
underlying path operation. The path-level safety for built-ins is handled
separately inside `builtin_file_read` (which blocks `/etc/shadow`,
`/etc/ssh`, `/root/.ssh`, etc.).

### Layer 2 — Sandbox (`pincher-core/src/security/sandbox.rs` + `sandbox/bwrap.rs`) ✅

The sandbox is a **runtime isolation layer**. After the veto allows an
action, shell commands are routed through `execute_action_shell`, which
builds a `CapabilityManifest` and attempts to execute inside a sandbox:

1. **Bubblewrap (`bwrap`)** — if available on `$PATH`, the command runs
   inside a lightweight namespace sandbox with restricted filesystem mounts
   (`/usr`, `/bin`, `/lib`, `/tmp` read; `.` read), no network, and an
   executable whitelist.
2. **Fallback** — if `bwrap` is not found or fails, the command runs via
   `std::process::Command` with `env_clear()`, a minimal `PATH`
   (`/usr/bin:/bin`), `HOME=/tmp`, `cwd=/tmp`, and stdout truncated to 4 KiB.

**Important limitation:** the bwrap sandbox path currently returns
`"Sandboxed command executed: <command>"` as its output string rather than
capturing the command's actual stdout. This means that *when bwrap is
available*, the CLI will report a generic success message instead of the
command's real output. When bwrap is *not* available, the fallback executor
*does* capture real stdout. This is a known asymmetry — see the README's
"Limitations" section.

The `landlock` feature gate enables Linux Landlock (kernel 5.13+) as an
additional filesystem-access restriction, but it is optional and not
enabled by default.

### Layer 3 — Immunology system (`pincher-core/src/immunology/`) ✅ (standalone)

The immunology system is a **threat-detection and learning layer** that is
real and unit-tested but is **not wired into the `do`/`teach` execution
path**. It exists as a public library API.

It has two subsystems:

- **Antigen detection** (`antigen.rs`) — scans text for four threat
  categories using regex patterns:
  - `PromptInjection` — "ignore previous instructions", "system override", etc.
  - `MaliciousAction` — `DROP TABLE`, `sh -c` with unsanitized input, etc.
  - `ResourceAbuse` — invocation-frequency / memory thresholds
  - `StaleReflex` — confidence decay below a living threshold

- **Immune memory** (`memory.rs`) — persists "antibodies" (learned
  rejection patterns) in SQLite with generation counts and last-seen
  timestamps, so threat knowledge survives restarts.

**Why it's standalone:** the immunology system is designed to be called by
an orchestration layer (or the `pincher-infer` Python sidecar) that decides
when to scan intents and what to do with detected threats. The core CLI
does not yet make those calls.

---

## 2. The embedding fallback chain ✅

`pincher do` converts the intent text into a 384-dimensional vector before
searching for matches. The `Embedder` type has a three-level fallback chain:

| Level | Condition | Behaviour |
|-------|-----------|-----------|
| 1. ONNX model loaded | `onnx` feature enabled **and** model file present | Real `all-MiniLM-L6-v2` embeddings via ONNX Runtime. Best semantic quality. |
| 2. ONNX feature enabled, model missing | `onnx` feature on but no `.onnx` file in `~/.pincher/models/` | Falls back to deterministic SHA-256 trigram hash. Logs a warning. |
| 3. ONNX feature disabled | Default build (no `--features onnx`) | Same hash fallback, with a compile-time notice. |

In all three modes, the embedding is 384-dimensional and the `cosine_similarity`
function works identically. The hash fallback is **deterministic** (same
input → same vector) and matches exact intents reliably, but it has poor
semantic awareness — paraphrases like "greet me" vs. "say hello" will not
match well without the ONNX model.

**Non-obvious detail in the matcher:** `match_reflex_with_thresholds` has a
special case for stored embeddings that are all zeros (which can happen if
a reflex was inserted before the embedder was available). In that case it
re-embeds the stored intent text on the fly during similarity computation
rather than comparing against the zero vector.

---

## 3. The confidence-update wiring gap

This is the single most important non-obvious behaviour for a contributor to
understand.

The README's "How it works" describes a four-step loop: Store → Match →
Execute → Learn. The "Learn" step claims that every execution updates the
reflex's confidence score. Here is what the code actually does:

| Code path | Increments `invoke_count`? | Calls `confidence_update`? |
|-----------|:---:|:---:|
| `do_command` → `execute_reflex` (the CLI `do` path) | ✅ Yes | ❌ No |
| `execute(reflex, input)` (public API, not called by CLI) | ✅ Yes | ✅ Yes |

The `confidence_update` method (`pincher-core/src/reflex/engine.rs:532`) and
the `update_confidence` formula (`pincher-core/src/reflex/confidence.rs`)
are both real and unit-tested:

```
success:  new = min(0.95, current + 0.05 * (1.0 - current))
failure:  new = max(0.05, current - 0.10 * current)
```

But because `do_command` routes through `execute_reflex` (the private helper
at line ~290) rather than the public `execute` method (line ~165), the
confidence score never changes on the `pincher do` path. User-taught
reflexes stay at 0.50; built-ins stay at 1.00.

**What this means for contributors:** if you want to wire confidence updates
into the `do` path, the cleanest approach is to add a
`self.confidence_update(&reflex.id, true)?;` call inside `execute_reflex`,
mirroring what `execute` already does. The method is already `&mut self`
and the `confidence_update` call is infallible from a compilation standpoint
(it returns `EngineResult`, which `execute_reflex` can propagate with `?`).

---

## 4. Library modules not wired into the CLI

`pincher-core` exposes seven modules as public APIs that the `pincher` CLI
does not exercise. They are real, compiled, and (where applicable) tested:

| Module | Purpose | Wired into CLI? | Has tests? |
|--------|---------|:---:|:---:|
| `immunology` | Threat detection + immune memory | ❌ | ✅ |
| `resource` | PID resource controller (Normal/Light/Critical) | ❌ | ✅ |
| `route` | Ternary-weighted room-routing graph | ❌ | ✅ |
| `rpc` | UDS JSON-RPC server for Python sidecar | ❌ | Compiled only |
| `capability` | Signed capability tokens | ❌ | Compiled only |
| `carapace` | WASM host/guest scaffolding | ❌ (needs `wasmtime`) | ✅ (feature-gated) |
| `intent` | Intent-contract schema | ❌ | Compiled only |

These modules are "building blocks available to library consumers" rather
than "features the CLI uses today." A newcomer evaluating pincher for
production use should check whether the specific module they need is
actually wired in.

---

## 5. The `compile` and `mature` simulations

Two CLI subcommands print success output without performing their named
work:

### `pincher compile --workspace <path>` 🔮

The command reads the workspace, checks for a `pincher.toml` manifest, then
prints:

```
[*] Dispatching compilation tasks to the cloud compiler engine...
[+] Rust source code synthesized successfully based on Intent contract rules.
[*] Invoking toolchain compiler for target: wasm32-wasip1
[SUCCESS] WASM binary compilation finished.
```

No Rust code is synthesized, no `wasm32-wasip1` toolchain is invoked, and
no `.wasm` file is produced. The `[SUCCESS]` line is unconditional. The
final `ℹ️` line acknowledges this, but only after the success message.

### `pincher mature --manifest <path>` 🔮

The command reads the manifest, counts non-empty lines, multiplies by 4,
and prints:

```
[+] Expanded seed matrix into {lines × 4} semantic test coordinates.
[SUCCESS] Deep vector space serialization complete. {lines × 4} nodes loaded.
```

No embeddings are computed, no vectors are written to the database, and no
fuzzing occurs. The "expanded seed matrix" size is a pure function of line
count.

**Why these exist:** both commands are placeholders that validate the CLI
plumbing (argument parsing, file existence checks, output formatting) while
the real implementation is deferred to a future phase. They should not be
relied upon for any actual compilation or fuzzing work.

---

## 6. The `.nail` portable format ✅

The `.nail` format is a `tar.zst` archive containing the agent's complete
state. It is real, tested (`test_pack_unpack_roundtrip`, `test_verify_nail`),
and used by the `pack`, `unpack`, `run`, and `gastrolith migrate` commands.

| File in archive | Contents | Checksummed? |
|-----------------|----------|:---:|
| `manifest.json` | Format version, fingerprint, timestamp, reflex count, BLAKE3 checksums | — (contains the checksums) |
| `reflexes.db` | The SQLite reflex database | ✅ BLAKE3 |
| `identity.json` | Agent name, preferences, creation timestamp | ✅ BLAKE3 |
| `config.toml` | Agent configuration | ✅ BLAKE3 |

On unpack, `verify_nail` recomputes BLAKE3 hashes for each file and
compares against the manifest. If any checksum mismatches, unpacking
fails with a `ChecksumMismatch` error. The hardware fingerprint is also
stored in the manifest, and `compatibility_score` can compare fingerprints
to assess whether a `.nail` bundle will run well on a different machine.
