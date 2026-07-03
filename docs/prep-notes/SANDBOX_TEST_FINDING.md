# Investigation: `test_sandbox_sandbox_capability_manifest` CI failure

**Method:** static source reading only (no Rust toolchain available in
this environment to build/run locally; cross-verified against the real
CI log for run `27742187423`, 2026-06-18).

## Conclusion: a deterministic CI/build-config bug, not a logic defect

The failing assertion (`pincher-core/tests/integration_tests.rs:270`):

```rust
assert!(config.use_bwrap || config.use_landlock);
```

`config.use_bwrap` and `config.use_landlock` are set in
`pincher-core/src/security/sandbox.rs`:

```rust
let use_bwrap = which_bwrap().is_some();       // line 277 — runtime check
...
use_landlock: cfg!(feature = "landlock"),      // line 286 — COMPILE-TIME flag
```

- `use_bwrap` is a genuine runtime check (`which_bwrap()` in
  `pincher-core/src/sandbox/bwrap.rs:269` looks for the `bwrap` binary
  on `PATH`). GitHub's standard Ubuntu runners don't ship `bwrap`
  pre-installed, so this is `false` on CI as currently configured.
- `use_landlock` is **not** runtime-detected at all — it's a Cargo
  feature flag (`landlock = ["dep:landlock"]`, `pincher-core/Cargo.toml`
  line 9), and that feature is **not** in the crate's `default = []`
  feature set (line 7). CI's workflow (`.github/workflows/ci.yml:48`)
  runs plain `cargo test` with no `--features landlock` flag, so this
  is unconditionally `false` at compile time in CI, regardless of the
  runner's actual OS capabilities.

**Both conditions are false by CI configuration, every time.** This
isn't a flaky/environment-dependent test — it's guaranteed to fail on
every run of the current CI workflow. (Consistent with the 3 most
recent CI runs checked, all failing at the same point.)

## This is not a bug in the reflex/sandbox engine's actual logic

The detection code itself (`which_bwrap()`, the `cfg!` feature check) is
correct — it does what it says. The test's assertion is also reasonable
in intent (some sandboxing backend should be available). The actual
problem is narrower: the CI workflow never enables a build path where
either backend can be true, so the test can never pass as configured.

## For a future fix (not attempted here — no toolchain to verify against)

Any of these would resolve it, but require a maintainer with a working
toolchain to verify, not a source-only read:
1. Install `bwrap` (bubblewrap) in the CI runner before `cargo test`.
2. Add `--features landlock` to CI's `cargo test` invocation (Linux
   runners should support it).
3. If neither backend is expected to be available in a bare CI
   container, the test itself should be marked `#[ignore]` or gated
   behind an environment check, rather than asserting an invariant CI
   can't satisfy.

## Relevance to the broader evaluation

The rest of the test suite (171 of 172 tests) genuinely passes,
including substantial coverage of the reflex engine's core claims
(`security::veto::tests` — 12 tests covering command-injection
patterns; `route::tests` — graph/routing logic; `resource::controller`/
`resource::pid` — the resource-budget PID controller;
`migration::pack::tests` — the `.nail` bundle format). This one failure
is a CI-configuration gap, not evidence the engine itself is
unreliable — worth noting precisely rather than either dismissing the
CI failure or overstating what it means.
