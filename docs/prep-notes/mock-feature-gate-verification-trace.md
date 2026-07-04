# Mock/chaos feature gate — verification trace

This document proves (by source-level reasoning, not by running tests — no
Rust toolchain is available in the environment that produced this change,
per `TASK.md`) that gating `hybrid-bridge`'s testing/mock modules behind a
non-default `mocks` Cargo feature removes them from the stable public API
*without* breaking any existing test in the workspace.

## What the change consists of

1. **`hybrid-bridge/Cargo.toml`**
   - New `[features]` section declaring `mocks = []` (no implicit
     dependencies, **not** a default feature).
   - New `[dev-dependencies]` self-reference:
     `hybrid-bridge = { path = ".", version = "0.1.0", features = ["mocks"] }`.
     This exists solely to enable `mocks` while compiling the crate's own
     integration tests (rationale in "Why the self dev-dependency" below).
2. **`hybrid-bridge/src/lib.rs`** — every declaration or re-export of the
   testing modules is wrapped in `#[cfg(any(test, feature = "mocks"))]`:
   - `pub mod chaos;`
   - `pub mod mock_matrix;`
   - `pub mod mock_room;`
   - `pub mod mock_veto;`
   - prelude re-exports `pub use crate::mock_room::MockRoomAgent;` and
     `pub use crate::mock_veto::MockVetoEngine;`
   - crate-level re-exports `pub use mock_room::MockRoomAgent;` and
     `pub use mock_veto::MockVetoEngine;`
3. **`docs/CRATES_IO_READINESS.md`** — the `hybrid-bridge` "Dev-only code in
   public API" row is flipped 🔴 → ✅ with a one-line note; open question #5
   is updated to record that this sub-item is resolved.

The contents of `chaos.rs`, `mock_matrix.rs`, `mock_room.rs`, and
`mock_veto.rs` were **not** touched, nor was anything in `pincher-core` or
`pincher-cli` (per `TASK.md`'s scope).

## Call-site survey (grep-verified)

Repo-wide search for `chaos::`, `mock_matrix::`, `mock_room::`,
`mock_veto::`, `MockRoomAgent`, `MockVetoEngine`, `MockMatrixEngine`,
`MockTensor`:

| Location | Kind | Depends on gated modules? |
|----------|------|---------------------------|
| `hybrid-bridge/src/lib.rs` unit tests (`test_prelude_exports`, `lib.rs:176-177`) | `#[cfg(test)] mod tests` inside the lib | **Yes** — references `prelude::MockRoomAgent` / `prelude::MockVetoEngine`. Covered by the `test` arm of the cfg (see scenario A). |
| `hybrid-bridge/tests/integration_tests.rs` (`:34-36`, many call sites) | integration test target | **Yes** — `use hybrid_bridge::mock_matrix::MockMatrixEngine; use hybrid_bridge::mock_room::MockRoomAgent; use hybrid_bridge::mock_veto::MockVetoEngine;` Covered by the `mocks` feature pulled in via dev-dependency (scenario B). |
| `hybrid-bridge/tests/full_pipeline_test.rs` (`:21-23`, many call sites) | integration test target | **Yes** — same three imports; covered the same way as above. |
| `hybrid-bridge/src/engine.rs` (`:679`, `:738`, …) | `#[cfg(test)] mod tests` inside `engine.rs` | **No.** These are *locally defined* test stubs (`struct MockMatrixEngine;` unit struct, `struct MockRoomAgent { ticker: String }`) built with `use super::*;`. They shadow the names but never import `crate::mock_*`; gating the public mock modules does not affect them. |
| `pincher-core/**`, `pincher-cli/**` | production crates | **No** — zero references. (`pincher-core/src/security/veto.rs:206` only mentions `hybrid_bridge::engine` in a doc comment.) |
| `examples/**` | runnable examples | **No matches** for any gated symbol. |
| `hybrid-bridge/API.md`, `EXAMPLES.md`, `QUALITY_REPORT.md` | prose docs | Non-compiled; unaffected by cfg. (Left as-is per scope; they describe the mock modules as public, which is now only true under `--features mocks`. A follow-up doc pass can note that, but it is out of scope for this change.) |

Conclusion: the only compiled consumers are (a) this crate's own unit tests
and (b) this crate's own integration tests. No external crate and no
non-test source file in `hybrid-bridge/src` references any gated module.

## The Rust subtlety that motivated the design

`#[cfg(test)]` is enabled by `rustc --test`. When `cargo test` builds an
**integration test** target (`tests/*.rs`), that target is compiled with
`--test`, but the **library is compiled as its dependency without `--test`**
— so `cfg(test)` is *not* active inside `src/lib.rs` as seen by the
integration tests. Therefore:

- Gating with only `#[cfg(test)]` would hide the modules from the
  integration tests → `tests/integration_tests.rs` and
  `tests/full_pipeline_test.rs` would fail to compile (`unresolved module
  mock_matrix`, etc.).
- Gating with `#[cfg(any(test, feature = "mocks"))]` plus the
  `[dev-dependencies]` self-reference fixes both: the `test` arm covers the
  lib's own unit-test compilation, and the `mocks` feature (requested by the
  dev-dependency, active during `cargo test`) covers the dependency
  compilation that the integration tests link against.

This is the standard Cargo idiom for "a crate's own tests need one of its
features, but downstream must opt in."

## Traced build scenarios

### A. `cargo test -p hybrid-bridge` (unit tests in `src/`)

- The lib is compiled with `--test` ⇒ `cfg(test)` active ⇒
  `cfg(any(test, feature = "mocks"))` is `true` ⇒ `chaos`, `mock_matrix`,
  `mock_room`, `mock_veto` and the prelude/crate-level re-exports are all
  present.
- `test_prelude_exports` (`lib.rs:161`) references `prelude::MockRoomAgent`
  and `prelude::MockVetoEngine` ⇒ both resolve. **Compiles.**
- The dev-dependency also enables `mocks`, which is redundant here but
  harmless.

### B. `cargo test -p hybrid-bridge` (integration tests in `tests/`)

- The lib is compiled as a dependency for each `tests/*.rs` target.
  `cfg(test)` is **not** set on the lib in this build, but the
  `[dev-dependencies]` self-reference requests `features = ["mocks"]`, and
  dev-dependencies are active under `cargo test` ⇒ `cfg(feature = "mocks")`
  is `true` ⇒ the gated modules and re-exports are present in the linked
  rlib.
- `tests/integration_tests.rs:34-36` and `tests/full_pipeline_test.rs:21-23`
  `use hybrid_bridge::mock_{matrix,room,veto}::*` ⇒ all resolve.
  **Compiles and runs.**

### C. `cargo build -p hybrid-bridge` (plain library build)

- `cfg(test)` off, no feature requested ⇒ `mocks` off ⇒ the four modules and
  their re-exports are absent.
- `src/bridge.rs`, `src/cli.rs`, `src/datafeed.rs`, `src/engine.rs`,
  `src/error.rs`, `src/ternary_bridge.rs`, `src/types.rs` contain **zero**
  references to `crate::chaos` / `crate::mock_*` (verified by ripgrep) ⇒ no
  dangling `use`. **Compiles** with the mock modules excluded from the
  public API.

### D. Downstream `cargo add hybrid-bridge` (no features)

- Identical to scenario C: `mocks` is not a default feature, so the
  downstream build sees no `chaos` / `mock_*` modules. They are no longer
  part of the stable public API. **This is the crates.io-readiness fix.**

### E. Downstream that wants the mocks

- `hybrid-bridge = { version = "0.1.0", features = ["mocks"] }` ⇒
  `cfg(feature = "mocks")` true ⇒ modules present. Opt-in works for
  consumers that genuinely need the in-memory trait implementations.

## Verification limitation

No `cargo`/`rustc` is present in this environment (`which cargo` ⇒ 127), so
the above is a manual source-level trace, not a green test run. A reviewer
with a toolchain should run:

```
cargo build -p hybrid-bridge                       # scenario C: mocks excluded, still compiles
cargo test -p hybrid-bridge                        # scenarios A + B: unit + integration tests pass
cargo build -p hybrid-bridge --features mocks      # scenario E: opt-in still compiles
# Confirm the public API no longer mentions the mocks without the feature:
cargo doc -p hybrid-bridge --no-deps               # chaos/mock_* absent from the rendered docs
```
