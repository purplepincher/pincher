# Veto trait extraction — behavior-preservation verification trace

This document proves (by source-level reasoning, not by running tests — no
Rust toolchain is available in the environment that produced this change,
per `TASK.md`) that the refactor that extracted `VetoPolicy` /
`RuleBasedVetoPolicy` out of `VetoEngine` preserves every existing behavior
encoded by the `security::veto::tests` and the `test_veto_engine_*`
integration tests.

## What the refactor changed structurally

Before:

- `VetoEngine { pub rules: Vec<VetoRule> }` owned the rule list **and** the
  decision logic. `check()` iterated `self.rules` and called a private
  `self.evaluate_rule(...)`.

After:

- A new trait `VetoPolicy` (`pincher-core/src/security/veto.rs`) has one
  method: `fn evaluate(&self, action: &str, context: &ExecutionContext)
  -> VetoResult<VetoDecision>`.
- A new struct `RuleBasedVetoPolicy { pub rules: Vec<VetoRule> }` is the
  **default** `VetoPolicy`. It owns the rule list, the per-rule matching
  (`evaluate_rule`, byte-for-byte the old match), the ordered iteration with
  first-non-Allow-wins semantics, and all logging that used to live in
  `VetoEngine::check` / `evaluate_rule`.
- `VetoEngine` becomes a thin generic dispatcher:
  `VetoEngine<P: VetoPolicy = RuleBasedVetoPolicy> { policy: P }`. Its
  `check()` only emits the delegation debug line and forwards to
  `self.policy.evaluate(...)`.
- The default type parameter means a bare `VetoEngine` is still
  `VetoEngine<RuleBasedVetoPolicy>`, so `VetoEngine::with_defaults()`,
  `VetoEngine::default()`, `VetoEngine::new()`, `load_rules`, `save_rules`,
  `add_rule`, `rule_count` all still exist and delegate to the held
  `RuleBasedVetoPolicy`. The one structural API change is that the public
  `rules` field became a `rules()` accessor (documented on the type); no
  external code read `rules` directly (verified by grep — see
  "Call-site compatibility" below).

The decision semantics, rule set (`default_veto_rules()`), `VetoRule`
variants and their constructors, `VetoDecision` and its helpers,
`ExecutionContext` and its builders, `VetoError`, `VetoResult`, and the
`VetoConfig` TOML shape are all **unchanged** — only their physical home
moved from "the engine" to "the default policy".

## How `check` resolves before vs. after

| Step | Before (`VetoEngine::check`) | After (`VetoEngine::check` → `RuleBasedVetoPolicy::evaluate`) |
|------|------------------------------|----------------------------------------------------------------|
| 1 | `debug!("Checking action against veto rules")` | `debug!("Delegating action to veto policy")`, then policy emits `debug!("Checking action against veto rules")` |
| 2 | `for rule in &self.rules { ... }` | `for rule in &self.rules { ... }` (same `self.rules`, now on the policy) |
| 3 | `let d = self.evaluate_rule(rule, action, ctx)?;` | `let d = Self::evaluate_rule(rule, action, ctx)?;` — identical match arms, identical `debug!` lines, identical `Deny/RequireConfirmation` strings |
| 4 | first non-Allow ⇒ `info!("Action vetoed by rule"); return Ok(d)` | identical |
| 5 | loop ends ⇒ `debug!("Action allowed by all rules"); Ok(Allow)` | identical |

`evaluate_rule` is byte-identical to the pre-refactor body (the only edit
was turning `&self` + `rule: &VetoRule` into a `Self::evaluate_rule(rule,
...)` associated function). Its `match` arms, predicate order, returned
`VetoDecision` payloads, and `debug!` messages are all unchanged.

## Three traced test cases (the ones TASK.md named)

### 1. `test_veto_deny_rm_rf`

Source (`pincher-core/src/security/veto.rs`, `#[cfg(test)]`):

```rust
let engine = VetoEngine::with_defaults();
let context = ExecutionContext::for_command("rm -rf /");
let decision = engine.check("rm -rf /", &context).unwrap();
assert!(decision.is_denied());
```

Trace against the refactored code:

1. `VetoEngine::with_defaults()` → `impl VetoEngine<RuleBasedVetoPolicy>`
   constructor → `Self { policy: RuleBasedVetoPolicy::with_default_rules() }`.
   `with_default_rules()` seeds `rules` from `default_veto_rules()`, whose
   **first** element is
   `VetoRule::ForbiddenPattern { pattern: "rm -rf /", reason: "..." }`.
2. `engine.check("rm -rf /", &ctx)` (`impl<P: VetoPolicy> VetoEngine<P>`)
   → `self.policy.evaluate("rm -rf /", &ctx)`.
3. `RuleBasedVetoPolicy::evaluate` iterates `self.rules`. First rule is
   `ForbiddenPattern { pattern: "rm -rf /", reason: ... }`.
4. `evaluate_rule` hits the `ForbiddenPattern` arm:
   `"rm -rf /".contains("rm -rf /")` → `true` ⇒ returns
   `Ok(VetoDecision::Deny(reason.clone()))`.
5. Back in `evaluate`, `decision.is_allowed()` is `false` ⇒ `info!(...)`,
   `return Ok(Deny(...))`.
6. `engine.check` returns `Deny(...)`. `is_denied()` ⇒ `true`. **Pass.**

This matches the pre-refactor path exactly (same first rule, same arm,
same `Deny`).

### 2. `test_veto_allow_safe_command`

```rust
let engine = VetoEngine::with_defaults();
let context = ExecutionContext::for_command("ls -la");
let decision = engine.check("ls -la", &context).unwrap();
assert!(decision.is_allowed());
```

Trace:

1. Same default engine as above.
2. `check("ls -la", &ctx)` → `policy.evaluate("ls -la", &ctx)`.
3. `evaluate` iterates every rule in `default_veto_rules()`:
   - `ForbiddenPattern { "rm -rf /" }` → `"ls -la".contains(...)` false.
   - `ForbiddenPattern { "rm -rf /*" }`, `{ "rm -rf ~" }` → false.
   - `ForbiddenCommand { "mkfs" }`, `{ "dd if=" }` → `"ls -la".contains(...)`
     false.
   - `ForbiddenPath { "/etc" }`, `{ "/sys" }`, `{ "/proc" }`, `{ "/boot" }`,
     `{ "/dev" }`: the arm loops over `context.paths` (empty for
     `for_command`) and then checks `"ls -la".contains(path)` — none match.
   - `ForbiddenPattern { "curl " }`, `{ "wget " }`, `{ "ssh " }`, `{ "nc " }`
     → false.
   - `MaxFileSize { 100MiB }`: `context.data_size` is `None` ⇒ arm skipped.
   - `ForbiddenCommand { "apt-get install" }`, `{ "yum install" }`,
     `{ "pip install" }` → false.
   - `ForbiddenPattern { "base64 -d" }`, `{ "eval " }`, `{ "exec " }`,
     `{ "powershell -enc" }`, `{ "python -c" }`, `{ "perl -e" }` → false.
   Every rule returns `Allow`, so the loop completes.
4. `evaluate` emits `debug!("Action allowed by all rules")` and returns
   `Ok(VetoDecision::Allow)`.
5. `check` returns `Allow`. `is_allowed()` ⇒ `true`. **Pass.**

Identical to the pre-refactor iteration order and predicates (the rule
vector is produced by the same `default_veto_rules()` function, copied
verbatim).

### 3. `test_veto_capability_check`

```rust
let mut engine = VetoEngine::new();
engine.add_rule(VetoRule::RequireCapability { capability: "network".into() });
let context = ExecutionContext::for_command("curl http://example.com");
let decision = engine.check("curl http://example.com", &context).unwrap();
assert!(decision.requires_confirmation());
let context_with_cap =
    ExecutionContext::for_command("curl http://example.com").with_capability("network");
let decision2 = engine.check("curl http://example.com", &context_with_cap).unwrap();
assert!(decision2.is_allowed());
```

Trace:

1. `VetoEngine::new()` (`impl VetoEngine<RuleBasedVetoPolicy>`) →
   `Self { policy: RuleBasedVetoPolicy::new() }` → policy with empty `rules`.
2. `engine.add_rule(RequireCapability { capability: "network" })` →
   `self.policy.add_rule(...)` → pushes onto `policy.rules`. The engine now
   holds a policy with exactly one rule.
3. First `check` (context has **no** capabilities):
   - `policy.evaluate("curl http://example.com", &ctx)`.
   - One rule: `RequireCapability { capability: "network" }`.
   - `evaluate_rule` `RequireCapability` arm:
     `!context.capabilities.contains("network")` → capabilities is empty ⇒
     `true` ⇒ returns `Ok(RequireConfirmation("Action requires 'network' capability which is not granted"))`.
   - Loop: `is_allowed()` false ⇒ `info!`, `return Ok(RequireConfirmation(..))`.
   - `requires_confirmation()` ⇒ `true`. **Pass (first assert).**
4. Second `check` (context built with `.with_capability("network")`, so
   `capabilities == {"network"}`):
   - `evaluate_rule` `RequireCapability` arm:
     `!{"network"}.contains("network")` → `false` ⇒ arm falls through ⇒
     returns `Ok(Allow)`.
   - Loop completes ⇒ `Ok(Allow)`.
   - `is_allowed()` ⇒ `true`. **Pass (second assert).**

The `add_rule` indirection (`engine.add_rule` → `policy.add_rule`) is the
only new hop; the resulting `policy.rules` vector is the same as the
pre-refactor `engine.rules` vector would have been, so the iteration and the
`RequireCapability` arm evaluate identically.

## Other existing tests (quick confirmation)

- `test_veto_deny_etc_write` — `for_file_operation("write", "/etc/passwd", _)`
  puts `/etc/passwd` in `context.paths`; the `ForbiddenPath { "/etc" }` arm
  matches via `context_path.starts_with("/etc")` ⇒ `Deny`. Unchanged logic.
- `test_veto_forbidden_network_command` — `ForbiddenPattern { "curl " }`
  (note trailing space) matches `"curl http://example.com"` ⇒ `Deny`.
  Unchanged.
- `test_veto_deny_base64_pipe` / `_eval` / `_powershell_enc` / `_python_c` /
  `_perl_e` — all `ForbiddenPattern` substring matches, unchanged strings.
- `test_veto_max_file_size` — `MaxFileSize` arm, `200MiB > 100MiB` ⇒ `Deny`.
  Unchanged.
- `test_veto_decision_helpers` — pure `VetoDecision` method tests; the type
  is byte-for-byte unchanged.
- `test_load_save_rules` — `save_rules`→`policy.save_rules` writes
  `VetoConfig { rules }` via the identical TOML path; `load_rules` reads it
  back into a `RuleBasedVetoPolicy` and wraps it in an engine.
  `rule_count()`→`policy.rule_count()`⇒`self.rules.len()`. Counts match.
- `test_execution_context_builder` — `ExecutionContext` is unchanged.
- Integration tests in `pincher-core/tests/integration_tests.rs`
  (`test_veto_engine_rejects_blocked_patterns`,
  `test_veto_engine_allows_safe_commands`,
  `test_veto_engine_no_panic_on_empty_action`) all go through
  `VetoEngine::with_defaults()` + `check(...)`, traced above.

## New tests added to lock in the trait surface

Three tests were added to `veto.rs`'s `#[cfg(test)] mod tests` to prove the
trait is genuinely pluggable (not just that the old behavior survives):

- `test_custom_veto_policy_is_used_by_engine` — implements a hand-written
  `DenyX` policy, builds `VetoEngine::with_policy(DenyX)`, and asserts the
  engine's `check` returns exactly what that policy decides. This proves
  `check` delegates to the held policy rather than ignoring it.
- `test_boxed_policy_runtime_swap` — builds `VetoEngine<Box<dyn VetoPolicy>>`
  (enabled by the blanket `impl<P: ?Sized + VetoPolicy> VetoPolicy for
  Box<P>`), proving runtime polymorphism works for callers that want to swap
  policies at runtime.
- `test_rule_based_policy_implements_trait_and_matches_engine` — calls
  `RuleBasedVetoPolicy::evaluate` directly and `VetoEngine::check`, and
  asserts their allow/deny results agree across four actions, proving the
  engine is a pure delegation with no hidden behavior.

## Call-site compatibility (grep-verified)

- `pincher-core/src/reflex/engine.rs:374` (`check_veto`) uses
  `crate::security::veto::VetoEngine::default()` then `.check(...)` and
  matches on `VetoDecision::{Deny,RequireConfirmation,Allow}`. `default()`
  resolves to `VetoEngine::<RuleBasedVetoPolicy>::default()` (default type
  parameter + the retained `impl Default for VetoEngine<RuleBasedVetoPolicy>`);
  `check` lives on `impl<P: VetoPolicy> VetoEngine<P>`; the `VetoDecision`
  variants are unchanged. Compiles and behaves identically.
- `pincher-core/tests/integration_tests.rs` uses
  `pincher_core::security::veto::*` and `VetoEngine::with_defaults().check(...)`
  — both still resolve under the default type parameter.
- `pincher-core/src/lib.rs` and `pincher-core/src/security/mod.rs` re-export
  the veto types; both were updated to also re-export `VetoPolicy` and
  `RuleBasedVetoPolicy` (additive; no existing symbol removed).
- `dynamics::veto::VetoEngine` (`pincher-core/src/dynamics/veto.rs`) and
  `hybrid_bridge::engine::VetoEngine` (the portfolio/SAEP trait) are
  unrelated, differently-namespaced symbols and were intentionally **not**
  touched, per TASK.md's scope ("Don't touch unrelated modules").

## Documented API change

`VetoEngine` no longer derives `Serialize`/`Deserialize`. Rationale, recorded
on the type's doc comment: that derive was never exercised — rule
persistence has always gone through `save_rules`/`load_rules` using the
`VetoConfig` TOML shape, and the engine is now generic over `P`. The
serializable surface for the default policy's data is unchanged:
`RuleBasedVetoPolicy` still derives `Serialize`/`Deserialize`, and
`save_rules`/`load_rules` keep the same on-disk format.

## Verification limitation

No `cargo`/`rustc` is present in this environment (`which cargo` ⇒ 127),
so the above is a manual source-level trace, not a green test run. A
reviewer with a toolchain should run:

```
cargo test -p pincher-core security::veto
cargo test -p pincher-core --test integration_tests
cargo check -p pincher-core
```
