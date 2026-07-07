# Educational-writing notes for this README rewrite

This document explains the specific choices made while rewriting `README.md` against the standard in `STYLE_BRIEF.md`.

## Goal of the rewrite

The old README stated what `pincher` does, but it named the mechanism before a newcomer could see why the mechanism was worth having. Terms such as "reflex," "novel intent," "veto," and "sandbox" appeared without being built in place. The rewrite's job was to keep every real technical detail while making the argument followable for someone who has never thought about "when should my app call an LLM vs. handle something itself."

## Choices and their rationale

### 1. Open with a relatable tension, not the product name

The document begins with two everyday failures — a voice assistant that phones the cloud to turn on a light, and a support bot that is slow for "what are your hours?" — before naming `pincher`. This follows the `STYLE_BRIEF.md` instruction to "motivate before you mechanize." The two failures map directly onto the two extremes the reflex/escalation pattern avoids: paying an LLM for every request, and hardcoding every request.

### 2. Define every piece of jargon at first use, inline

Every specialized term is defined in the same sentence where it first appears:

- **Reflex**: "a natural-language intent (what the user wants) paired with an action (what to do about it)."
- **Embedding**: "a 384-dimensional vector that captures meaning."
- **Cosine similarity**: "a score from -1 to 1 measuring how close two vectors point in the same direction."
- **Veto**: "a safety policy that can Allow, Deny, or RequireConfirmation for a command."
- **Sandbox**: "a capability-based isolation layer that uses bubblewrap (`bwrap`) and Linux Landlock ... to restrict what the command can see and do."

No glossary or footnote is used, because the brief explicitly rejects deferred definitions.

### 3. Teach "known intent" and "genuine miss" with a real source example

The matcher logic lives in `pincher-core/src/reflex/matcher.rs`. The concrete walkthrough uses the exact threshold values from that file (0.80 for Exact, 0.55 for Similar) and the exact flow it implements: exact-string fast path, sqlite-vec nearest-neighbor search, cosine re-ranking, and classification. The three example inputs — `"say hello"`, `"greet me"`, and `"what is the weather in Tokyo?"` — illustrate the three `MatchResult` variants (`Exact`, `Similar`, `Novel`) as they are defined in the source.

### 4. Preserve and verify real facts and numbers

The following claims were checked against the source rather than copied from the old README:

- Embedding dimension: `EMBEDDING_DIM = 384` in `pincher-core/src/embed/onnx.rs` and `pincher-core/src/db/schema.rs`.
- Match thresholds: `MatchThresholds::default()` sets `exact = 0.80` and `similar = 0.55` in `pincher-core/src/reflex/matcher.rs`.
- Default confidence on teach: `0.5` in `ReflexEngine::teach` (`pincher-core/src/reflex/engine.rs`). The old README and `GETTING_STARTED.md` both said 0.55; the source says 0.50, so the README now says 0.50.
- Confidence update rule: success adds `0.05 * (1.0 - current)`, failure subtracts `0.10 * current`, clamped to `[0.05, 0.95]` in `pincher-core/src/reflex/confidence.rs`.
- Built-in reflexes: exactly the 10 intents listed in `is_builtin_intent` and `BUILTIN_REFLEXES` in `pincher-core/src/reflex/engine.rs` and `pincher-core/src/db/schema.rs`.
- Crates.io status: verified with `cargo search pincher`; no `pincher`/`pincher-core`/`pincher-cli` package is published. The old README's claim that the crates are not on crates.io remains accurate.
- CLI output format: every example block matches the `println!` statements in `pincher-cli/src/main.rs` (`cmd_status`, `cmd_teach`, `cmd_do`, `cmd_reflexes`).

### 5. Correct one source-code-level finding

The end-to-end test file `tests/e2e_runtime_test.rs` imports `BundleSecurityEngine` from `pincher_core`. That type is not re-exported anywhere in `pincher-core/src/lib.rs`, so the import will fail to compile. The README notes this explicitly rather than repeating the old document's untested claim about test counts. Unit-test counts were obtained by grepping for `#[test]` in each crate (184 in `pincher-core`, 64 in `hybrid-bridge`).

### 6. Avoid the two-tier trap

There is no "TL;DR" or "simple version." The same document is meant to be readable by a newcomer and precise enough for a working engineer. Precision is achieved by attaching every abstraction to a concrete example or a source-file reference, not by stripping detail.

### 7. What was not changed

No new features, claims, or capabilities were invented. The project layout, license, feature flags, limitations, and subcommand table all match the current source and Cargo manifests. The only intentional corrections were to numbers that the source contradicts (the default confidence) and to test-status wording that reflects what is actually in the repository.

## Verification limits

I attempted to run the test suite and `cargo check -p pincher-cli` to verify compilation. Both timed out after 300 seconds because the workspace depends on heavy crates such as `ort` (ONNX Runtime) and `sqlite-vec`. The README therefore relies on direct source inspection for API and output-format claims, and it is honest about the unverified end-to-end test import.
