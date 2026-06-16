# Claude Code Task: Fix pincher all-features build

## Context
The pincher repo at /tmp/pincher compiles fine with `cargo check` (default features) but fails with `cargo check --all-features`.

## Background
Working directory: /tmp/pincher  
pincher is the fleet's reflex engine. It has `wasmtime`, `landlock`, and `ort` as optional features. These have breaking API changes from their upstreams.

## What's broken
From `cargo check --all-features 2>&1`:

1. **wasmtime (v27)** → wasmtime v27 is fine but:
   - `config.cranelift()` removed in v28+ → just delete the call
   - `wasmtime::WASM_PAGE_SIZE` removed → replace with 65536u64

2. **landlock (v0.4)** → installed may be v0.5+:
   - `AccessFs::from_execute()` → use `AccessFs::from_all(Access::Execute)`
   - `PathBeneath::new(&path, access)` → needs fd: `PathBeneath::new(landlock::make_unchecked(fd), &path, access)`
   - `Ruleset` → now typed in v0.5+
   - Check: `pin landlock = "0.4"` if 0.5 breaks, or update code to 0.5 API

3. **ort (v2.0.0-rc.12)** → not verified but may need ndarray version pinning

## Task
1. Apply fixes for the 12 compilation errors
2. Make `cargo check --all-features` pass
3. Write the fix summary to /tmp/pincher-ci-fixes.md
4. Return the content of /tmp/pincher-ci-fixes.md

## Rules
- If upstream breaking changes are unavoidable, pin to the last compatible version
- Add `// #fix: reason` comments for each change
- Don't change any behavior, only fix compilation
- Keep changes minimal — single-line fixes preferred
