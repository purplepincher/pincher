# Git Dependency Resolution Options

> Investigation into the `ternary-types` and `silo-core` git dependencies that block crates.io publishing.

## TL;DR

- **`silo-core` is not published on crates.io and is genuinely unused in this workspace.** It can be removed from `Cargo.toml` outright.
- **`ternary-types` is not published on crates.io and is actively used** in `pincher-core/src/route/mod.rs` and `hybrid-bridge/src/ternary_bridge.rs`. It must either be published upstream, vendored into this workspace, or replaced before any workspace member can be published.

---

## 1. crates.io Publish Status

### `ternary-types`

**Status: not published on crates.io.**

- Direct check: `https://crates.io/crates/ternary-types` returns HTTP 404.
- Web search for `site:crates.io ternary-types` did not surface a crate by this name; results were unrelated crates (`ternary-signal`, `ternary-coordination`, `ternary-tree`, etc.).
- Source repo: [`SuperInstance/ternary-types`](https://github.com/SuperInstance/ternary-types)
  - Default branch is `master`.
  - Current `Cargo.toml` version is `0.2.0`.
  - This workspace pins rev `fa01da449de07108b8c99594253bea47e73be956`, which `Cargo.lock` resolves as `ternary-types 0.2.0`.
  - License: `MIT OR Apache-2.0`.
  - Only dependency: optional `serde` (used here via the `serde` feature).

### `silo-core`

**Status: not published on crates.io.**

- Direct check: `https://crates.io/crates/silo-core` returns HTTP 404.
- Web search for `site:crates.io silo-core` did not surface a Rust crate by this name.
- Source repo: [`SuperInstance/silo-core`](https://github.com/SuperInstance/silo-core)
  - Described as a pure Z₃ math layer for the SuperInstance ternary fleet.
  - License: `MIT OR Apache-2.0` (per README).

---

## 2. Is `silo-core` Actually Used?

**No.**

A full-workspace grep for `silo_core::`, `silo-core`, and `silo_core` found only:

- The workspace declaration in `Cargo.toml:28`.
- Mentions in this audit / task documentation.
- No `use silo_core`, no module imports, no type references, no feature flags.

**Conclusion:** `silo-core` can be removed from `[workspace.dependencies]` in `Cargo.toml` with no code changes. This is the simplest fix and resolves half the blocker immediately.

---

## 3. How `ternary-types` Is Used

`ternary-types` is used in two workspace members:

### `pincher-core` (`pincher-core/src/route/mod.rs`)

- Types used: `ternary_types::Ternary`.
- Surface:
  - `TernaryGraph` stores edge weights as `Ternary`.
  - Graph algorithms (`shortest_paths`, `all_pairs_shortest_paths`, `label_propagation`, `connected_components`, `modularity`, etc.) convert `Ternary` to `f64` via `i8::from(t)` and compare against `Ternary::Positive` / `Ternary::Negative`.
  - `RoomGraph` exposes `add_trusted_route` / `add_blocked_route` which insert `Ternary::Positive` / `Ternary::Negative` edges.
  - Unit tests directly construct and assert on `Ternary` values.
- Depth: moderate. The enum is threaded through the public `TernaryGraph` and `RoomGraph` APIs, but the operations performed on it are simple (`i8::from`, equality, negation via `-Ternary`, addition via `Ternary + Ternary`).

### `hybrid-bridge` (`hybrid-bridge/src/ternary_bridge.rs`)

- Types used: `ternary_types::{Ternary, TernaryMatrix, TritVector}`.
- Surface:
  - `TritMapping` maps continuous conviction values to `Ternary`.
  - `PortfolioToTrits` encodes portfolio positions into `TritVector`.
  - `TritsToWeights` converts `TritVector` and `TernaryMatrix` back into float weights.
  - `ConsensusComputer` aggregates weighted `Ternary` votes into consensus `TritVector`s.
  - Convenience functions convert between `Ternary` and the local `TernaryGate` type.
  - Unit tests exercise the full bridge surface.
- Depth: deeper than `pincher-core`. The bridge relies on `TritVector` methods (`new`, `len`, `is_empty`, `get`, `as_slice`), `TernaryMatrix` methods (`new`, `rows`, `cols`, `row`, `as_slice`), and `Ternary` arithmetic (`+`, `-`, `*`, `Neg`).

### Overall integration

- `ternary-types` is not a tiny leaf type here; it appears in public types and test suites across two crates.
- Replacing it with a locally-defined enum is possible but would require re-implementing `TritVector` and `TernaryMatrix` (or changing `hybrid-bridge` to use its own containers).

---

## 4. Maturity of `SuperInstance/ternary-types`

Observations from the upstream repo:

- **README positioning:** Presents the crate as "the foundational type used across the SuperInstance ecosystem" and "the type-theoretic foundation stone of the entire ternary stack." This suggests it is intended to be stable and widely shared, not a one-off sketch.
- **Code structure:** The repo is organized as a real crate with `Cargo.toml`, `src/lib.rs`, modules (`ternary`, `trit_vector`, `matrix`, `packed`, `convert`, `iter`), tests, and dual licensing.
- **Version:** Already at `0.2.0`, with a changelog implied by the version bump from the `0.1.0` still present in `Cargo.lock` for an older unpinned git reference.
- **No explicit publish promise:** Neither the README nor the `Cargo.toml` states that the crate will be published to crates.io, only that it is the shared fleet type.
- **No crates.io badge / versioned install instructions:** The README shows `cargo build` / `cargo test`, not `cargo add ternary-types`.

**Interpretation:** The crate looks mature enough to publish, but there is no public commitment or timeline. Do not assume it will appear on crates.io without asking the upstream maintainers.

---

## 5. What Vendoring `ternary-types` Would Look Like

The upstream crate is small and self-contained, so vendoring is technically straightforward.

### Files to vendor

From [`SuperInstance/ternary-types`](https://github.com/SuperInstance/ternary-types) (`master`, version `0.2.0`):

```text
Cargo.toml
src/lib.rs
src/ternary.rs
src/trit_vector.rs
src/matrix.rs
src/packed.rs
src/convert.rs
src/iter.rs
LICENSE (MIT OR Apache-2.0)
```

### Size estimate

- ~7 source modules plus `lib.rs`.
- Roughly ~1,500–2,000 lines of Rust including doc comments and tests.
- No non-optional dependencies besides `serde`.

### Dependencies that would come along

- **Production:** only optional `serde` (version `1`, with `derive` feature). This workspace already depends on `serde`, so there is no new transitive dependency.
- **Dev-only (tests in the vendored crate):** `serde_json`, `serde_test`, `rand`. These only matter if you run the vendored crate's own tests; they would not affect downstream consumers.

### How to vendor

1. Add a new workspace member, e.g., `vendor/ternary-types/`.
2. Copy the files listed above into it.
3. Add `vendor/ternary-types` to the root `Cargo.toml` `[workspace] members` list.
4. Replace the git dependency in `[workspace.dependencies]` with `ternary-types = { path = "vendor/ternary-types", version = "0.2.0" }`.
5. Update `pincher-core/Cargo.toml` and `hybrid-bridge/Cargo.toml` to use the workspace dependency (or path) instead of the explicit git URL.
6. Preserve the upstream `LICENSE` file and note the provenance in the vendored directory.

### Caveats

- The `packed` feature uses `#![cfg_attr(feature = "packed", feature(core_intrinsics))]`, which requires nightly Rust. None of the current workspace usages enable the `packed` feature, so this is only a concern if someone later enables it.
- Vendoring creates a maintenance burden: upstream bug fixes and API changes must be manually synced.
- The vendored crate would need a crates.io-compatible name if it is ever published as part of this workspace; re-publishing under the same `ternary-types` name on crates.io is not possible unless the SuperInstance team either publishes it first or grants ownership.

---

## 6. Options for a Human Decision

### Option A — Wait for upstream publication

**Action:** Ask the `SuperInstance/ternary-types` maintainers to publish `ternary-types` to crates.io. Once published, replace the git dependency with a versioned registry dependency, e.g.:

```toml
ternary-types = { version = "0.2.0", features = ["serde"] }
```

**Pros:**

- Cleanest solution; no code or workspace structure changes beyond the dependency line.
- Lets the upstream team retain ownership of the canonical type definition.
- Avoids licensing/attribution duplication in this repo.

**Cons:**

- Blocked on an external team/timeline.
- No public commitment from upstream that this will happen.
- crates.io name must actually be available and released under a compatible version.

**Best if:** You can coordinate with SuperInstance and publication is likely in the near term.

---

### Option B — Vendor `ternary-types` into this workspace

**Action:** Copy the upstream crate into a workspace member (e.g., `vendor/ternary-types/`) and depend on it via path/workspace.

**Pros:**

- Removes the external git dependency entirely; publishing is no longer blocked by upstream.
- The crate is small and has no heavy transitive dependencies.
- Pinning the exact upstream revision means the behavior stays identical.

**Cons:**

- Adds ~1,500–2,000 lines of third-party code to maintain.
- Future upstream fixes must be manually ported.
- If the crate is later published to crates.io under the same name, this workspace would have a naming conflict if it tried to publish the vendored copy.

**Best if:** Upstream publication is uncertain or far off, and you need to unblock crates.io publishing soon.

---

### Option C — Replace `ternary-types` with a local implementation

**Action:** Define a minimal `Ternary` enum, `TritVector`, and `TernaryMatrix` inside this workspace (e.g., in `pincher-core` or a new `pincher-ternary` crate) and update the two call sites.

**Pros:**

- Full control over the API and publishing.
- No external source to track or license file to duplicate.
- Can be tailored to exactly the surface area this workspace uses.

**Cons:**

- Requires non-trivial code changes in `hybrid-bridge`, which uses `TritVector` and `TernaryMatrix` fairly extensively.
- Loses compatibility with the broader SuperInstance ternary ecosystem if interoperability matters later.
- Re-implementing tested primitives risks subtle bugs.

**Best if:** You want zero external dependencies and the SuperInstance ecosystem compatibility is not a long-term concern.

---

## 7. Recommended Next Step

1. **Remove `silo-core` immediately** — it is unused and safe to delete.
2. **Choose one of the three `ternary-types` options above.**
   - If you have a line of communication to SuperInstance, **Option A** is the cleanest.
   - If you need to publish soon and upstream is unresponsive, **Option B** is the pragmatic path.
   - If you want to minimize long-term coupling to SuperInstance, consider **Option C**, but budget for the larger refactor.

---

## References

- Workspace dependency declarations: `Cargo.toml:27-28`
- `pincher-core` usage: `pincher-core/src/route/mod.rs:30` and throughout
- `hybrid-bridge` usage: `hybrid-bridge/src/ternary_bridge.rs:41-42` and throughout
- Upstream repo: [`SuperInstance/ternary-types`](https://github.com/SuperInstance/ternary-types)
- crates.io pages checked (both 404):
  - `https://crates.io/crates/ternary-types`
  - `https://crates.io/crates/silo-core`
