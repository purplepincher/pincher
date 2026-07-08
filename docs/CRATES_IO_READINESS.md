# crates.io Publish Readiness Audit

> **Re-audited live on 2026-07-08** with `cargo` / `rustc` **1.96.1**.
> The previous version of this doc was written in an environment with no Rust
> toolchain, so it could not run `cargo fmt`, `cargo build`, or
> `cargo publish --dry-run`. All of those have now been run for real; results
> below are from actual toolchain output, not reasoning traces.

---

## What remains for Casey (org owner) to do himself

This is the **complete** remaining list — there is nothing else:

```bash
cargo login                                 # paste Casey's crates.io API token
cargo publish -p pincher-core               # MUST go first — pincher depends on it
cargo publish -p pincher                    # only after pincher-core is live on crates.io
```

That is all. `pincher` cannot be dry-run-verified end-to-end locally until
`pincher-core` is actually on the registry (it's a real dependency, not a path
dep at publish time), so the two publishes must happen in that order. No real
publish was performed here — there is no crates.io token in this environment,
and publishing is reserved for Casey.

---

## Scope

Workspace members (root `Cargo.toml:3-7`):

- `pincher-core` — core runtime library. **Publishable.**
- `pincher-cli` directory → package renamed to **`pincher`** (see below). Builds
  the `pincher` binary. **Publishable.**
- `hybrid-bridge` — `publish = false` (`hybrid-bridge/Cargo.toml:3`). **Out of
  scope for this release**, unchanged.

Workspace version: **`0.2.0`** (`Cargo.toml:10`). License: `MIT OR Apache-2.0`,
both `LICENSE` and `LICENSE-APACHE` present at repo root.

---

## Verification run this session (cargo/rustc 1.96.1)

| Check | Command | Result |
|-------|---------|--------|
| Formatting | `cargo fmt --check` (whole workspace) | ✅ **pass**, exit 0 — no violations. The ~15-file `cargo fmt` debt the old audit flagged under `hybrid-bridge/` was already fixed on `main` by commit `355d33b` ("Fix cargo fmt violations found by CI's first real automatic run"). `cargo fmt` is now a confirmed no-op. |
| Build | `cargo build --workspace` | ✅ **pass**, exit 0 |
| Tests | `cargo test --workspace` | ✅ **pass**, exit 0 — **341 passed, 0 failed** (hybrid-bridge 104+18+1+24, pincher CLI 2, pincher-core 174+16, doc-tests 1+1; 9 ignored) |
| Dry run (core) | `cargo publish --dry-run -p pincher-core` | ✅ **pass**, exit 0, warning-free — packaged 57 files, 549.4 KiB (125.9 KiB compressed) |
| Dry run (cli) | `cargo publish --dry-run -p pincher` | ⚠️ **packages cleanly, then fails only at registry lookup** for `pincher-core` ("no matching package named `pincher-core` found / location searched: crates.io index"). This is expected: `pincher` depends on `pincher-core`, which is not on crates.io yet. The packaging step itself succeeds; the only blocker is publish order. |

### Crate-name availability (confirmed live, 2026-07-08)

All three candidate names are unclaimed. Verified against the live crates.io API
with a proper `User-Agent` header (crates.io blocks the default curl UA):

```
crate `pincher`       does not exist
crate `pincher-cli`   does not exist
crate `pincher-core`  does not exist
```

---

## Rename applied: `pincher-cli` package → `pincher`

**Decision (per the task brief):** rename the *package* to `pincher` rather than
rewrite the README. Rationale: `pincher` is confirmed available, and the README
already advertises `cargo install pincher` (`README.md:189`-region). The binary
target was already named `pincher` (`pincher-cli/Cargo.toml:14-16`,
`pincher-cli/src/main.rs`), so renaming the package aligns the published package
name with the documented install command and the binary — least churn.

**Core change:**
- `pincher-cli/Cargo.toml:2`: `name = "pincher-cli"` → `name = "pincher"`.

**The directory `pincher-cli/` and the `[[bin]] name = "pincher"` target were
intentionally NOT renamed** — only the `[package] name` field. Cargo resolves the
workspace member by path, so the member entry `"pincher-cli"` in root
`Cargo.toml:5` (a path) stays correct.

**Internal references updated** (package-name references that would otherwise
break `-p` selection or misstate the published name):

- `cargo build/release/test ... -p pincher-cli` → `-p pincher` in:
  `README.md` (×2), `GETTING_STARTED.md` (×3), `PLUG_AND_PLAY.md`,
  `TEMPLATES/ONBOARDING.md`, `install.sh`, `EDUCATIONAL_NOTES.md`,
  `docs/contributing.md`.
- Crates.io candidate-package lists pruned of `pincher-cli` (it is no longer a
  package at all): `README.md`, `EDUCATIONAL_NOTES.md`.
- `pincher-cli/README.md` heading `# pincher-cli` → `# pincher` (this file is
  the published crate README — see readme note below).

**Left alone (correctly):** directory-path references (`pincher-cli/`,
`pincher-cli/src/main.rs`, `./pincher-cli` in `.github/workflows/publish_nail.yml`,
etc.), component/historical mentions (`ARCHITECTURE.md`, `wiring-report.md`,
`CHANGELOG.md`, `docs/research/*`, `docs/prep-notes/*`), and the arbitrary temp
file-name prefix string in `pincher-cli/src/main.rs`. `Cargo.lock` was
auto-regenerated by cargo after the rename (`pincher-cli` → `pincher` at the
package entry).

---

## readme-path fix (warning silenced)

The inherited `readme.workspace = true` resolved to `../README.md` (workspace
root) from each member's perspective, producing this `cargo publish --dry-run`
warning:

> readme `../README.md` appears to be a path outside of the package, but there is
> already a file named `README.md` in the root of the package...

The packaged output was already correct (each crate shipped its own per-crate
`README.md` with `readme = "README.md"` in the packaged manifest — verified
byte-for-byte). To make intent explicit and silence the warning, the two
**publishable** crates now set the readme per-crate:

- `pincher-core/Cargo.toml`: `readme.workspace = true` → `readme = "README.md"`
- `pincher-cli/Cargo.toml`: `readme.workspace = true` → `readme = "README.md"`

`hybrid-bridge` is `publish = false` so it was left on workspace inheritance.
After this change both dry runs are **warning-free** (pincher-core end-to-end;
pincher up to the expected pincher-core registry lookup).

---

## Per-crate status

### `pincher-core` — ✅ publish-ready

| Item | Status | Notes |
|------|--------|-------|
| Metadata (description/license/repository/homepage/keywords/categories) | ✅ | workspace-inherited + per-crate `categories` (`pincher-core/Cargo.toml:12`) |
| `readme` | ✅ | `pincher-core/README.md` (see readme fix above) |
| License files | ✅ | `LICENSE` (MIT) + `LICENSE-APACHE` |
| Version | ✅ | `0.2.0` via `version.workspace = true` |
| No path-dep-on-member blockers | ✅ | no path dependencies on other workspace members |
| No git dependencies | ✅ | the old `ternary-types` git dep blocker does not apply to `pincher-core` — it has no git deps in its published dependency set |
| No hardcoded secrets | ✅ | the old `SUPER_INSTANCE_SHARED_SECRET_KEY_FOR_NAIL_INTEGRITY` surface (`daemon.rs`/`registry.rs`/`updater.rs` and the orphan root `src/`) was removed in prior waves and is absent |
| `cargo publish --dry-run` | ✅ | exit 0, warning-free, 57 files / 549.4 KiB |

### `pincher` (was `pincher-cli`) — ✅ publish-ready (pending `pincher-core`)

| Item | Status | Notes |
|------|--------|-------|
| Package name | ✅ | renamed `pincher-cli` → `pincher`; `pincher` confirmed unclaimed on crates.io |
| Binary name | ✅ | `[[bin]] name = "pincher"` (unchanged) — `cargo install pincher` yields the `pincher` binary |
| Metadata | ✅ | workspace-inherited + `categories = ["command-line-utilities"]` |
| `readme` | ✅ | `pincher-cli/README.md` (heading updated to `# pincher`) |
| License files | ✅ | `LICENSE` (MIT) + `LICENSE-APACHE` |
| Version | ✅ | `0.2.0` |
| Path dep has `version` | ✅ | `pincher-core = { path = "../pincher-core", version = "0.2.0" }` (`pincher-cli/Cargo.toml:19`) |
| No hardcoded secrets / unsafe | ✅ | registry token only via `--token` / `PINCHER_REGISTRY_TOKEN` (`pincher-cli/src/main.rs`) |
| `cargo publish --dry-run` | ⚠️ → ✅ | packages cleanly; the only dry-run failure is the not-yet-published `pincher-core` dependency, which resolves by publishing `pincher-core` first |

### `hybrid-bridge` — out of scope

`publish = false` (`hybrid-bridge/Cargo.toml:3`). Not published this release.
Its `cargo fmt` / build / test participation is green as part of the workspace
checks above.

---

## Minor non-blocking notes (not required for 0.2.0)

- `pincher-core/src/migration/pack.rs:178` carries a `TODO` doc comment for
  inline checksum verification. Cosmetic; the public API does not depend on it
  for 0.2.0.
