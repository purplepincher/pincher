# crates.io Publish Readiness Audit

> **Prep-only audit.** No `cargo publish`, `cargo package`, or network calls to crates.io were performed. The environment has no Rust toolchain, so local `cargo publish --dry-run` verification was not possible.

## Scope

Workspace members are defined in the root `Cargo.toml`:

- `pincher-core` — core runtime library (`Cargo.toml:3-4`).
- `pincher-cli` — command-line package that builds the `pincher` binary (`Cargo.toml:5`).
- `hybrid-bridge` — Hybrid Manifold communication backbone (`Cargo.toml:6`).

The root `README.md` describes the workspace as "two crates" (`pincher-core` and `pincher-cli`) and does not mention `hybrid-bridge` as a user-facing published library (`README.md:113-117`). Whether `hybrid-bridge` should be published at all is noted as an open question below.

---

## `pincher-core`

| Item | Status | Notes / Citation |
|------|--------|------------------|
| `description` | ✅ ready | inherited from workspace (`Cargo.toml:14`, `pincher-core/Cargo.toml:7`) |
| `license` | ✅ ready | inherited from workspace (`Cargo.toml:13`, `pincher-core/Cargo.toml:6`) |
| `repository` | ✅ ready | inherited from workspace (`Cargo.toml:15`, `pincher-core/Cargo.toml:8`) |
| `homepage` | ✅ ready | inherited from workspace (`Cargo.toml:16`, `pincher-core/Cargo.toml:9`) |
| `keywords` | ✅ ready | inherited from workspace (`Cargo.toml:18`, `pincher-core/Cargo.toml:11`) |
| `categories` | ✅ ready | set per-crate (`pincher-core/Cargo.toml:12`) |
| `readme` | ✅ ready | inherited from workspace, file created (`pincher-core/Cargo.toml:10`, `pincher-core/README.md`) |
| License files exist | ✅ ready | `LICENSE` (MIT) exists; `LICENSE-APACHE` added |
| Path deps to members have `version` | ✅ ready | no path dependencies on other workspace members |
| Version number | ✅ ready | `version.workspace = true` → `0.1.0` (`pincher-core/Cargo.toml:3`) |
| Name availability | ⚠️ needs human verification | `pincher-core` must be checked on crates.io |
| Git dependencies | 🔴 needs fix | `ternary-types` is a git dependency (`pincher-core/Cargo.toml:30`). crates.io rejects packages that depend on unpublished/git crates. `ternary-types` (and `silo-core`, declared at `Cargo.toml:28` but unused) must be published to crates.io or vendored before `pincher-core` can publish. |
| Hardcoded secrets / dead code | 🔴 needs fix | `pincher-core/src/updater.rs:129` contains the hardcoded shared secret `SUPER_INSTANCE_SHARED_SECRET_KEY_FOR_NAIL_INTEGRITY`. `pincher-core/src/daemon.rs`, `pincher-core/src/registry.rs`, and `pincher-core/src/updater.rs` are **not** referenced by `pincher-core/src/lib.rs` (which declares modules at lines 6-21), but they would still be included in the published tarball. They contain registry-token handling and an undeclared `reqwest` dependency. These files should be removed, moved behind a feature gate, or promoted to real compiled modules with proper dependencies before publishing. |
| TODO/FIXME markers | ⚠️ minor | `pincher-core/src/migration/pack.rs:178` has a `TODO` doc comment for inline checksum verification. Not a blocker on its own, but the public API surface should be complete before a `1.0` publish. |

---

## `pincher-cli`

| Item | Status | Notes / Citation |
|------|--------|------------------|
| `description` | ✅ ready | inherited from workspace (`pincher-cli/Cargo.toml:7`) |
| `license` | ✅ ready | inherited from workspace (`pincher-cli/Cargo.toml:6`) |
| `repository` | ✅ ready | inherited from workspace (`pincher-cli/Cargo.toml:8`) |
| `homepage` | ✅ ready | inherited from workspace (`pincher-cli/Cargo.toml:9`) |
| `keywords` | ✅ ready | inherited from workspace (`pincher-cli/Cargo.toml:11`) |
| `categories` | ✅ ready | set per-crate (`pincher-cli/Cargo.toml:12`) |
| `readme` | ✅ ready | inherited from workspace, file created (`pincher-cli/Cargo.toml:10`, `pincher-cli/README.md`) |
| License files exist | ✅ ready | `LICENSE` (MIT) and `LICENSE-APACHE` exist |
| Path deps to members have `version` | ✅ ready | `pincher-core = { path = "../pincher-core", version = "0.1.0" }` (`pincher-cli/Cargo.toml:19`) |
| Version number | ✅ ready | `version.workspace = true` → `0.1.0` (`pincher-cli/Cargo.toml:3`) |
| Name availability / naming | ⚠️ needs human decision | The package name is `pincher-cli`, but the binary name is `pincher` (`pincher-cli/src/main.rs:7-9`, `pincher-cli/Cargo.toml:15-16`). The README currently advertises `cargo install pincher` (`README.md:189`), which would require a package named `pincher`. Verify that `pincher-cli` (and possibly `pincher`) are available on crates.io and decide whether to rename the package or update the README. |
| Hardcoded secrets / unsafe code | ✅ ready | No hardcoded credentials found in CLI source. The `publish` subcommand takes a registry token via `--token` / `PINCHER_REGISTRY_TOKEN` (`pincher-cli/src/main.rs:106-107`). |
| Transitive blockers | 🔴 blocked | `pincher-cli` depends on `pincher-core`, which is blocked by the `ternary-types` git dependency. |

---

## `hybrid-bridge`

| Item | Status | Notes / Citation |
|------|--------|------------------|
| `description` | ✅ ready | set per-crate (`hybrid-bridge/Cargo.toml:7`) |
| `license` | ✅ ready | inherited from workspace (`hybrid-bridge/Cargo.toml:6`) |
| `repository` | ✅ ready | inherited from workspace (`hybrid-bridge/Cargo.toml:8`) |
| `homepage` | ✅ ready | inherited from workspace (`hybrid-bridge/Cargo.toml:9`) |
| `keywords` | ✅ ready | set per-crate (`hybrid-bridge/Cargo.toml:11`) |
| `categories` | ✅ ready | set per-crate (`hybrid-bridge/Cargo.toml:12`) |
| `readme` | ✅ ready | inherited from workspace, file created (`hybrid-bridge/Cargo.toml:10`, `hybrid-bridge/README.md`) |
| License files exist | ✅ ready | `LICENSE` (MIT) and `LICENSE-APACHE` exist |
| Path deps to members have `version` | ✅ ready | `pincher-core = { path = "../pincher-core", version = "0.1.0" }` (`hybrid-bridge/Cargo.toml:15`) |
| Version number | ✅ ready | `version.workspace = true` → `0.1.0` (`hybrid-bridge/Cargo.toml:3`) |
| Name availability | ⚠️ needs human verification | `hybrid-bridge` must be checked on crates.io |
| Publish intent | ⚠️ open question | The root README describes the workspace as two crates and does not list `hybrid-bridge` as a published library (`README.md:113-117`). Its API docs (`hybrid-bridge/API.md`) present it as a reusable crate, but it may be intended as an internal component. Decide whether to publish it or leave it workspace-only. |
| Git dependencies | 🔴 needs fix | `ternary-types` is inherited from workspace (`hybrid-bridge/Cargo.toml:16` → `Cargo.toml:27`). Same blocker as `pincher-core`: must be on crates.io before publishing. |
| Dev-only code in public API | ✅ resolved | Mock/chaos modules (`chaos`, `mock_matrix`, `mock_room`, `mock_veto`) and their `MockRoomAgent`/`MockVetoEngine` re-exports are now gated behind `#[cfg(any(test, feature = "mocks"))]` in `hybrid-bridge/src/lib.rs`. The non-default `mocks` feature is auto-enabled only for this crate's own test suite via a `[dev-dependencies]` self-reference, so a plain `cargo add hybrid-bridge` no longer pulls the testing modules into the public API. See `docs/prep-notes/mock-feature-gate-verification-trace.md`. |

---

## Cross-cutting findings

- **Version consistency:** All publishable members are at `0.1.0` (`Cargo.toml:10`).
- **License files:** The workspace claimed `license = "MIT OR Apache-2.0"` but only an MIT `LICENSE` file existed. An Apache-2.0 `LICENSE-APACHE` file was added to match the declared license.
- **Workspace metadata:** Added `authors`, `description`, `readme`, and `keywords` to `[workspace.package]` in the root `Cargo.toml` so all members can inherit them consistently.
- **No Rust toolchain available:** `cargo publish --dry-run` / `cargo package` could not be run to verify packaging, dependency resolution, or crate size. A human with a Rust toolchain must run these checks before publishing.

---

## Open questions / required human sign-off

1. **Name availability:** Verify on crates.io that `pincher-core`, `pincher-cli`, and (if desired) `pincher` and `hybrid-bridge` are available.
2. **CLI package naming:** Decide whether `pincher-cli` is the intended package name or whether the package should be renamed to `pincher` to match `cargo install pincher` in the README.
3. **External dependencies:** Publish or vendor `ternary-types` and `silo-core` to crates.io, or replace the git dependencies with versioned registry dependencies.
4. **Dead code / secrets:** Remove or refactor `pincher-core/src/daemon.rs`, `registry.rs`, and `updater.rs`; in particular replace the hardcoded `SUPER_INSTANCE_SHARED_SECRET_KEY_FOR_NAIL_INTEGRITY` with runtime configuration.
5. **`hybrid-bridge` scope:** Decide whether `hybrid-bridge` is meant to be a public crate. The mock/chaos dev-only modules are now feature-gated behind the non-default `mocks` feature (see the `hybrid-bridge` table above), so that specific blocker is resolved; the remaining open question is purely whether to publish the crate at all.

---

## Safe metadata fixes applied

- `Cargo.toml`: added `authors`, `description`, `readme`, and `keywords` to `[workspace.package]`.
- `pincher-core/Cargo.toml`: added workspace inheritance for `authors`, `license`, `description`, `repository`, `homepage`, `readme`, `keywords`, plus per-crate `categories`.
- `pincher-cli/Cargo.toml`: added workspace inheritance for the same metadata fields, per-crate `categories`, and added `version = "0.1.0"` to the `pincher-core` path dependency.
- `hybrid-bridge/Cargo.toml`: switched `version` to `version.workspace = true`, added workspace inheritance for metadata, added `readme`, `keywords`, `categories`, and added `version = "0.1.0"` to the `pincher-core` path dependency.
- Created `pincher-core/README.md`, `pincher-cli/README.md`, and `hybrid-bridge/README.md` so each crate's `readme` field points to a real file.
- Added `LICENSE-APACHE` to satisfy the declared `MIT OR Apache-2.0` dual license.
