# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [0.2.0] - 2026-07-06

### Added

- Added `hybrid-bridge` workspace member with ternary bridge, CLI, datafeed, API/EXAMPLES docs, and integration tests.
- Added `TEMPLATES/` and `TUTORIALS.md` for onboarding and agent bundles.
- Added `ternary-kernel` Cargo feature with aarch64 NEON SIMD-optimized cosine/L2 kernels.
- Added devcontainer configuration for Codespaces / VS Code Rust development.
- Added GitHub Actions workflows: `ci.yml`, `release.yml`, `agent-workflow.yml`, `agent_activation.yml`, and `publish_nail.yml`.
- Added `AGENT.md` and `memory/JOURNAL.md` for repository-agent identity and duty log.
- Added fleet architecture docs, fleet-scout tooling, and GC tier scripts (`gc-fleet.sh`, `archive-gc.sh`, `dedup-reflexes.sh`).
- Added `.nail` bundle format specification (`docs/NAIL_FORMAT_SPEC.md`).
- Added `LICENSE-APACHE` to satisfy the declared `MIT OR Apache-2.0` dual license.
- Added per-crate `README.md` files for `pincher-core`, `pincher-cli`, and `hybrid-bridge`.
- Added `mocks` feature in `hybrid-bridge` so mock/chaos modules are not part of the default public API.
- Added `AGENTS.md` and `.gcconfig` for fleet GC protocol.
- Added `docs/trust-pipeline-integration.md` and `docs/CRATES_IO_READINESS.md`.
- Added prep-evaluation notes covering demo draft and sandbox test findings.

### Changed

- Rewrote `README.md` multiple times to provide honest, production-ready documentation and a shell-and-signal narrative.
- Updated repository URLs throughout the project to point at the `purplepincher/pincher` fork.
- Unified workspace package metadata (`authors`, `description`, `license`, `readme`, `keywords`) in root `Cargo.toml` and inherited it from each member.
- Wired the `pincher` CLI to the core reflex engine, implementing teach/do/reflexes/status/doctor/pack/unpack/run and related subcommands.
- Resolved the `ternary-types` git dependency blocker for `pincher-core` by inlining a minimal local `Ternary` enum.
- Refactored the veto engine into a pluggable `VetoPolicy` trait.
- Adapted the ONNX embedding module to `ort` 2.0.0-rc.12 and pinned `ndarray` to 0.17.
- Updated fleet-scout connectivity check to use the `gh` API instead of raw CDN access.

### Fixed

- Fixed all compiler warnings and deprecation warnings across the workspace.
- Fixed doctest type mismatch in `pincher-core/src/intent/schema.rs`.
- Fixed clippy `io-other-error` lints for Rust 1.96.
- Fixed optional feature builds (`wasmtime`, `landlock`) for Rust 1.96 compatibility.
- Restored the missing `Gastrolith` CLI command variant.
- Skipped sandbox backend assertion in tests when no sandbox backend is available.
- Resolved trait-method ambiguity in the veto engine that was blocking CI.
- Fixed borrow-checker and formatting issues in the new `hybrid-bridge` crate.

### Removed

- Removed the unused `silo-core` workspace dependency.
- Removed orphaned `pincher-core` source files containing a hardcoded secret (`daemon.rs`, `registry.rs`, `updater.rs`).
- Removed a second orphaned `src/` tree at the repo root containing the same secret files.
- Removed scratch task-brief and log files that should not have been tracked.
- Removed deprecated duplicate architecture files (`embedder.rs`, `engine.rs`, `types.rs`) from `pincher-core`.

### Security

- Removed the hardcoded `SUPER_INSTANCE_SHARED_SECRET_KEY_FOR_NAIL_INTEGRITY` secret from source control.
- Gated `hybrid-bridge` mock/chaos modules behind the non-default `mocks` feature so they are excluded from normal builds.

## [0.1.0] - 2026-06-03

### Added

- Initial PincherOS codebase: `pincher-core`, `pincher-cli`, inference tools, and supporting documentation.
- `pincher-core` runtime: reflex engine, vector store (SQLite + `sqlite-vec`), intent embedding, sandbox, migration, RPC, security/veto, and immunology modules.
- `pincher-cli` binary with core status, doctor, teach, and execution subcommands.
- `pincher-infer` Python package for ONNX-based embedding generation.
- Examples, benchmarks, install script, and initial README.
