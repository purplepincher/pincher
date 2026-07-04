//! Deterministic rule-based veto engine for PincherOS
//!
//! The veto engine provides a safety layer that inspects actions before
//! execution and can deny or require confirmation for dangerous operations.
//!
//! ## Architecture
//!
//! The decision logic is expressed as the pluggable [`VetoPolicy`] trait.
//! The engine ([`VetoEngine`]) is a thin dispatcher that delegates the
//! allow/deny/confirm decision to whichever policy it holds. The historical
//! rule-based behavior — the TOML-configured [`VetoRule`] list — lives on as
//! [`RuleBasedVetoPolicy`], which is the **default** policy. A downstream
//! consumer (this org, or anyone forking pincher) can therefore swap in a
//! custom policy by implementing [`VetoPolicy`] and constructing the engine
//! with [`VetoEngine::with_policy`], without modifying the engine's dispatch
//! code.
//!
//! Backward compatibility: a bare `VetoEngine` still defaults to the
//! rule-based policy and exposes the same rule-management surface
//! (`new`, `with_defaults`, `load_rules`, `save_rules`, `add_rule`,
//! `rule_count`) it always did, so every existing call site and test
//! continues to compile and behave identically.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;
use thiserror::Error;
use tracing::{debug, info, instrument, warn};

/// Veto engine errors.
#[derive(Debug, Error)]
pub enum VetoError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("TOML parse error: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("Invalid rule: {0}")]
    InvalidRule(String),
}

/// Result type for veto operations.
pub type VetoResult<T> = Result<T, VetoError>;

/// A single veto rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum VetoRule {
    /// Block commands containing a specific substring.
    ForbiddenCommand {
        /// The forbidden command pattern (substring match).
        pattern: String,
    },
    /// Block access to specific filesystem paths.
    ForbiddenPath {
        /// The forbidden path prefix.
        path: String,
    },
    /// Enforce a maximum file size for write operations.
    MaxFileSize {
        /// Maximum file size in bytes.
        max_bytes: u64,
    },
    /// Require a specific capability for an action.
    RequireCapability {
        /// The capability name required.
        capability: String,
    },
    /// Block specific command patterns (simplified substring for MVP).
    ForbiddenPattern {
        /// Substring pattern to match in commands.
        pattern: String,
        /// Human-readable reason for the block.
        reason: String,
    },
}

impl VetoRule {
    /// Create a forbidden command rule.
    pub fn forbidden_command(pattern: impl Into<String>) -> Self {
        VetoRule::ForbiddenCommand {
            pattern: pattern.into(),
        }
    }

    /// Create a forbidden path rule.
    pub fn forbidden_path(path: impl Into<String>) -> Self {
        VetoRule::ForbiddenPath { path: path.into() }
    }

    /// Create a max file size rule.
    pub fn max_file_size(max_bytes: u64) -> Self {
        VetoRule::MaxFileSize { max_bytes }
    }

    /// Create a require capability rule.
    pub fn require_capability(capability: impl Into<String>) -> Self {
        VetoRule::RequireCapability {
            capability: capability.into(),
        }
    }
}

/// The decision made by the veto engine.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VetoDecision {
    /// The action is allowed.
    Allow,
    /// The action is denied with a reason.
    Deny(String),
    /// The action requires user confirmation with a reason.
    RequireConfirmation(String),
}

impl VetoDecision {
    /// Check if the action is allowed.
    pub fn is_allowed(&self) -> bool {
        matches!(self, VetoDecision::Allow)
    }

    /// Check if the action is denied.
    pub fn is_denied(&self) -> bool {
        matches!(self, VetoDecision::Deny(_))
    }

    /// Check if the action requires confirmation.
    pub fn requires_confirmation(&self) -> bool {
        matches!(self, VetoDecision::RequireConfirmation(_))
    }
}

/// Execution context provided to the veto engine for evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionContext {
    /// The command or action being executed.
    pub action: String,
    /// File paths involved in the action.
    pub paths: Vec<String>,
    /// Size of data being written (if applicable).
    pub data_size: Option<u64>,
    /// Capabilities granted for this execution.
    pub capabilities: HashSet<String>,
    /// The working directory.
    pub working_dir: String,
}

impl ExecutionContext {
    /// Create a new execution context for a command.
    pub fn for_command(action: &str) -> Self {
        Self {
            action: action.to_string(),
            paths: Vec::new(),
            data_size: None,
            capabilities: HashSet::new(),
            working_dir: std::env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| ".".to_string()),
        }
    }

    /// Create a new execution context for a file operation.
    pub fn for_file_operation(action: &str, path: &str, data_size: Option<u64>) -> Self {
        Self {
            action: action.to_string(),
            paths: vec![path.to_string()],
            data_size,
            capabilities: HashSet::new(),
            working_dir: std::env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| ".".to_string()),
        }
    }

    /// Add a capability to this context.
    pub fn with_capability(mut self, capability: &str) -> Self {
        self.capabilities.insert(capability.to_string());
        self
    }

    /// Add a path to this context.
    pub fn with_path(mut self, path: &str) -> Self {
        self.paths.push(path.to_string());
        self
    }
}

// ─────────────────────────────────────────────────────────────────────
// Pluggable policy trait
// ─────────────────────────────────────────────────────────────────────

/// A pluggable veto policy.
///
/// A policy answers one question: given an `action` and its
/// [`ExecutionContext`], should the action be [`VetoDecision::Allow`]ed,
/// [`VetoDecision::Deny`]ied, or [`VetoDecision::RequireConfirmation`]?
///
/// The historical, deterministic rule-based behavior of pincher is provided
/// out of the box by [`RuleBasedVetoPolicy`], which the default
/// [`VetoEngine`] uses. Implement this trait to supply a different policy
/// (e.g. one that consults an external allowlist, an audit log, or an LLM
/// confirmation step) and feed it to [`VetoEngine::with_policy`]; the
/// engine's dispatch code does not need to change.
///
/// The `Debug + Send + Sync` supertraits mirror the conventions used for
/// engine traits elsewhere in this codebase (see `hybrid_bridge::engine`)
/// and keep policies usable across threads / as `Box<dyn VetoPolicy>`.
pub trait VetoPolicy: std::fmt::Debug + Send + Sync {
    /// Evaluate the policy for the given action and context.
    fn evaluate(&self, action: &str, context: &ExecutionContext) -> VetoResult<VetoDecision>;
}

// Blanket impl so that `Box<dyn VetoPolicy>` (and any boxed policy) is itself
// a `VetoPolicy`. This lets callers that want *runtime* polymorphism build a
// `VetoEngine<Box<dyn VetoPolicy>>` and swap policies at runtime, while static
// callers use `VetoEngine<MyPolicy>` directly.
impl<P: VetoPolicy + ?Sized> VetoPolicy for Box<P> {
    fn evaluate(&self, action: &str, context: &ExecutionContext) -> VetoResult<VetoDecision> {
        (**self).evaluate(action, context)
    }
}

// ─────────────────────────────────────────────────────────────────────
// Default policy: the rule-based engine
// ─────────────────────────────────────────────────────────────────────

/// The default veto policy: a deterministic, ordered list of [`VetoRule`]s.
///
/// This is exactly the behavior pincher shipped before the policy trait was
/// extracted — it is now simply expressed as *one* (default) implementation
/// of [`VetoPolicy`] rather than being baked into the engine. Rules are
/// evaluated in registration order and the first non-[`VetoDecision::Allow`]
/// decision wins.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RuleBasedVetoPolicy {
    /// The ordered list of veto rules to evaluate.
    pub rules: Vec<VetoRule>,
}

impl RuleBasedVetoPolicy {
    /// Create a new empty rule-based policy.
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    /// Create a rule-based policy seeded with the default safety rules.
    pub fn with_default_rules() -> Self {
        Self {
            rules: default_veto_rules(),
        }
    }

    /// Load veto rules from a TOML configuration file.
    #[instrument(skip(path))]
    pub fn load_rules(path: &Path) -> VetoResult<Self> {
        info!(path = ?path, "Loading veto rules from config");

        let content = std::fs::read_to_string(path)?;
        let config: VetoConfig = toml::from_str(&content)?;

        info!(rule_count = config.rules.len(), "Loaded veto rules");
        Ok(Self {
            rules: config.rules,
        })
    }

    /// Save the current rules to a TOML configuration file.
    #[instrument(skip(self, path))]
    pub fn save_rules(&self, path: &Path) -> VetoResult<()> {
        let config = VetoConfig {
            rules: self.rules.clone(),
        };
        let content = toml::to_string_pretty(&config)
            .map_err(|e| VetoError::InvalidRule(format!("Failed to serialize rules: {}", e)))?;

        std::fs::write(path, content)?;
        info!(path = ?path, "Saved veto rules");
        Ok(())
    }

    /// Add a rule to the policy.
    pub fn add_rule(&mut self, rule: VetoRule) {
        self.rules.push(rule);
    }

    /// Get the number of rules.
    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }

    /// Evaluate a single rule against an action.
    ///
    /// Factored out as a standalone helper (operating on an explicit rule
    /// reference) so that custom policies may reuse the per-rule semantics
    /// of the default policy when building their own rule sets, without
    /// having to re-implement the [`VetoRule`] matching.
    fn evaluate_rule(
        rule: &VetoRule,
        action: &str,
        context: &ExecutionContext,
    ) -> VetoResult<VetoDecision> {
        match rule {
            VetoRule::ForbiddenCommand { pattern } => {
                if action.contains(pattern) {
                    debug!(pattern = pattern, "Forbidden command pattern matched");
                    return Ok(VetoDecision::Deny(format!(
                        "Command contains forbidden pattern: '{}'",
                        pattern
                    )));
                }
            }

            VetoRule::ForbiddenPath { path } => {
                for context_path in &context.paths {
                    if context_path.starts_with(path) || context_path == path {
                        debug!(path = path, "Forbidden path matched");
                        return Ok(VetoDecision::Deny(format!(
                            "Access to path '{}' is forbidden",
                            path
                        )));
                    }
                }
                // Also check if the action references the forbidden path
                if action.contains(path) {
                    debug!(path = path, "Forbidden path referenced in action");
                    return Ok(VetoDecision::Deny(format!(
                        "Action references forbidden path: '{}'",
                        path
                    )));
                }
            }

            VetoRule::MaxFileSize { max_bytes } => {
                if let Some(data_size) = context.data_size {
                    if data_size > *max_bytes {
                        debug!(
                            data_size = data_size,
                            max_bytes = max_bytes,
                            "File size exceeds limit"
                        );
                        return Ok(VetoDecision::Deny(format!(
                            "Data size ({} bytes) exceeds maximum ({} bytes)",
                            data_size, max_bytes
                        )));
                    }
                }
            }

            VetoRule::RequireCapability { capability } => {
                if !context.capabilities.contains(capability) {
                    debug!(capability = capability, "Missing required capability");
                    return Ok(VetoDecision::RequireConfirmation(format!(
                        "Action requires '{}' capability which is not granted",
                        capability
                    )));
                }
            }

            VetoRule::ForbiddenPattern { pattern, reason } => {
                if action.contains(pattern) {
                    debug!(pattern = pattern, "Forbidden pattern matched");
                    return Ok(VetoDecision::Deny(reason.clone()));
                }
            }
        }

        Ok(VetoDecision::Allow)
    }
}

impl VetoPolicy for RuleBasedVetoPolicy {
    fn evaluate(&self, action: &str, context: &ExecutionContext) -> VetoResult<VetoDecision> {
        debug!(action = action, "Checking action against veto rules");

        for rule in &self.rules {
            let decision = Self::evaluate_rule(rule, action, context)?;
            if !decision.is_allowed() {
                info!(
                    action = action,
                    decision = ?decision,
                    "Action vetoed by rule"
                );
                return Ok(decision);
            }
        }

        debug!(action = action, "Action allowed by all rules");
        Ok(VetoDecision::Allow)
    }
}

// ─────────────────────────────────────────────────────────────────────
// VetoEngine — the dispatcher
// ─────────────────────────────────────────────────────────────────────

/// The veto engine that evaluates actions against a [`VetoPolicy`].
///
/// `VetoEngine` is a thin dispatcher: [`VetoEngine::check`] delegates to the
/// held policy's [`VetoPolicy::evaluate`]. The default type parameter keeps
/// every historical call site working unchanged — a bare `VetoEngine` is a
/// `VetoEngine<RuleBasedVetoPolicy>`, so `VetoEngine::with_defaults()` /
/// `VetoEngine::default()` / `VetoEngine::new()` all yield the exact
/// rule-based engine that existed before this trait extraction.
///
/// To plug in a custom policy, construct the engine with
/// [`VetoEngine::with_policy`]:
///
/// ```ignore
/// let engine = VetoEngine::with_policy(MyCustomPolicy);
/// ```
///
/// The rule-management helpers (`add_rule`, `rule_count`, `save_rules`,
/// `load_rules`) are only available on `VetoEngine<RuleBasedVetoPolicy>`,
/// since they are meaningless for an arbitrary policy.
///
/// # API change (backward-incompatible, but unused in practice)
///
/// Prior to this refactor `VetoEngine` derived `Serialize`/`Deserialize`.
/// That derive was never exercised — rule persistence has always gone through
/// [`RuleBasedVetoPolicy::save_rules`] / [`RuleBasedVetoPolicy::load_rules`]
/// via the [`VetoConfig`] TOML shape. Because the engine is now generic over
/// `P`, the derive no longer applies and has been removed. The serializable
/// representation of the default policy's data is unchanged and remains
/// available through `save_rules`/`load_rules` (and `RuleBasedVetoPolicy`
/// itself still derives `Serialize`/`Deserialize`).
#[derive(Debug, Clone)]
pub struct VetoEngine<P: VetoPolicy = RuleBasedVetoPolicy> {
    /// The policy this engine delegates [`VetoEngine::check`] to.
    policy: P,
}

/// Engine functionality available for *any* policy.
impl<P: VetoPolicy> VetoEngine<P> {
    /// Build an engine around an arbitrary policy.
    ///
    /// This is the extension point: implement [`VetoPolicy`] and pass an
    /// instance here to change how pincher decides allow/deny/confirm without
    /// touching the engine's dispatch code.
    pub fn with_policy(policy: P) -> Self {
        Self { policy }
    }

    /// Check an action against the configured policy.
    ///
    /// This is a pure delegation to [`VetoPolicy::evaluate`]; all decision
    /// logic lives in the policy. The `#[instrument]` span and the
    /// delegation log line are preserved for observability continuity.
    #[instrument(skip(self, context))]
    pub fn check(&self, action: &str, context: &ExecutionContext) -> VetoResult<VetoDecision> {
        debug!(action = action, "Delegating action to veto policy");
        self.policy.evaluate(action, context)
    }

    /// Borrow the underlying policy.
    pub fn policy(&self) -> &P {
        &self.policy
    }
}

/// Rule-management surface, only meaningful for the default rule-based policy.
///
/// These mirror the pre-refactor `VetoEngine` API exactly and delegate to the
/// underlying [`RuleBasedVetoPolicy`], preserving all existing behavior.
impl VetoEngine<RuleBasedVetoPolicy> {
    /// Create a new empty veto engine (no rules).
    pub fn new() -> Self {
        Self {
            policy: RuleBasedVetoPolicy::new(),
        }
    }

    /// Create a veto engine with the default safety rules.
    pub fn with_defaults() -> Self {
        Self {
            policy: RuleBasedVetoPolicy::with_default_rules(),
        }
    }

    /// Load veto rules from a TOML configuration file.
    pub fn load_rules(path: &Path) -> VetoResult<Self> {
        let policy = RuleBasedVetoPolicy::load_rules(path)?;
        Ok(Self { policy })
    }

    /// Save the current rules to a TOML configuration file.
    pub fn save_rules(&self, path: &Path) -> VetoResult<()> {
        self.policy.save_rules(path)
    }

    /// Add a rule to the engine.
    pub fn add_rule(&mut self, rule: VetoRule) {
        self.policy.add_rule(rule);
    }

    /// Get the number of rules.
    pub fn rule_count(&self) -> usize {
        self.policy.rule_count()
    }

    /// Borrow the engine's rule list.
    ///
    /// This replaces the historical public `rules` field (which was only ever
    /// read internally). The rules now live on the policy; this accessor keeps
    /// read-only inspection of them available on the default engine.
    pub fn rules(&self) -> &[VetoRule] {
        &self.policy.rules
    }

    /// Borrow the underlying rule-based policy.
    pub fn rule_policy(&self) -> &RuleBasedVetoPolicy {
        &self.policy
    }
}

impl Default for VetoEngine<RuleBasedVetoPolicy> {
    fn default() -> Self {
        Self::with_defaults()
    }
}

/// TOML configuration format for veto rules.
#[derive(Debug, Serialize, Deserialize)]
struct VetoConfig {
    rules: Vec<VetoRule>,
}

/// Generate the default set of safety veto rules.
fn default_veto_rules() -> Vec<VetoRule> {
    vec![
        // Block destructive rm commands
        VetoRule::ForbiddenPattern {
            pattern: "rm -rf /".to_string(),
            reason: "Recursive force-delete of root filesystem is not allowed".to_string(),
        },
        VetoRule::ForbiddenPattern {
            pattern: "rm -rf /*".to_string(),
            reason: "Recursive force-delete of root filesystem is not allowed".to_string(),
        },
        VetoRule::ForbiddenPattern {
            pattern: "rm -rf ~".to_string(),
            reason: "Recursive force-delete of home directory is not allowed".to_string(),
        },
        VetoRule::ForbiddenCommand {
            pattern: "mkfs".to_string(),
        },
        VetoRule::ForbiddenCommand {
            pattern: "dd if=".to_string(),
        },
        // Protect system directories
        VetoRule::ForbiddenPath {
            path: "/etc".to_string(),
        },
        VetoRule::ForbiddenPath {
            path: "/sys".to_string(),
        },
        VetoRule::ForbiddenPath {
            path: "/proc".to_string(),
        },
        VetoRule::ForbiddenPath {
            path: "/boot".to_string(),
        },
        VetoRule::ForbiddenPath {
            path: "/dev".to_string(),
        },
        // Require capabilities for specific dangerous operations
        // These trigger confirmation for network and subprocess commands
        // rather than blanket-blocking everything
        VetoRule::ForbiddenPattern {
            pattern: "curl ".to_string(),
            reason: "Network request requires 'network' capability".to_string(),
        },
        VetoRule::ForbiddenPattern {
            pattern: "wget ".to_string(),
            reason: "Network download requires 'network' capability".to_string(),
        },
        VetoRule::ForbiddenPattern {
            pattern: "ssh ".to_string(),
            reason: "SSH connection requires 'network' capability".to_string(),
        },
        VetoRule::ForbiddenPattern {
            pattern: "nc ".to_string(),
            reason: "Netcat requires 'network' capability".to_string(),
        },
        // File size limit: 100MB
        VetoRule::MaxFileSize {
            max_bytes: 100 * 1024 * 1024,
        },
        // Block package manager operations (could modify system)
        VetoRule::ForbiddenCommand {
            pattern: "apt-get install".to_string(),
        },
        VetoRule::ForbiddenCommand {
            pattern: "yum install".to_string(),
        },
        VetoRule::ForbiddenCommand {
            pattern: "pip install".to_string(),
        },
        // Block encoded command evasion techniques
        VetoRule::ForbiddenPattern {
            pattern: "base64 -d".to_string(),
            reason: "Base64 decoding to shell pipe is a known evasion technique".to_string(),
        },
        VetoRule::ForbiddenPattern {
            pattern: "eval ".to_string(),
            reason: "eval() execution is dangerous".to_string(),
        },
        VetoRule::ForbiddenPattern {
            pattern: "exec ".to_string(),
            reason: "exec() is a known bypass vector".to_string(),
        },
        VetoRule::ForbiddenPattern {
            pattern: "powershell -enc".to_string(),
            reason: "Encoded PowerShell commands are dangerous".to_string(),
        },
        VetoRule::ForbiddenPattern {
            pattern: "python -c".to_string(),
            reason: "Inline Python execution is potentially dangerous".to_string(),
        },
        VetoRule::ForbiddenPattern {
            pattern: "perl -e".to_string(),
            reason: "Inline Perl execution is potentially dangerous".to_string(),
        },
    ]
}

// Silence the unused `warn` import: it was present in the pre-refactor module
// (preserved here verbatim) but is not currently emitted from this file.
#[allow(unused_imports)]
use self::warn as _allow_warn_import;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_veto_allow_safe_command() {
        let engine = VetoEngine::with_defaults();
        let context = ExecutionContext::for_command("ls -la");
        let decision = engine.check("ls -la", &context).unwrap();
        assert!(decision.is_allowed());
    }

    #[test]
    fn test_veto_deny_rm_rf() {
        let engine = VetoEngine::with_defaults();
        let context = ExecutionContext::for_command("rm -rf /");
        let decision = engine.check("rm -rf /", &context).unwrap();
        assert!(decision.is_denied());
    }

    #[test]
    fn test_veto_deny_etc_write() {
        let engine = VetoEngine::with_defaults();
        let context = ExecutionContext::for_file_operation("write", "/etc/passwd", None);
        let decision = engine.check("write /etc/passwd", &context).unwrap();
        assert!(decision.is_denied());
    }

    #[test]
    fn test_veto_forbidden_network_command() {
        let engine = VetoEngine::with_defaults();
        let context = ExecutionContext::for_command("curl http://example.com");
        let decision = engine.check("curl http://example.com", &context).unwrap();
        // curl is blocked by ForbiddenPattern rule
        assert!(decision.is_denied());
    }

    #[test]
    fn test_veto_capability_check() {
        let _engine = VetoEngine::with_defaults();
        // A custom engine with a RequireCapability rule
        let mut engine = VetoEngine::new();
        engine.add_rule(VetoRule::RequireCapability {
            capability: "network".to_string(),
        });
        let context = ExecutionContext::for_command("curl http://example.com");
        let decision = engine.check("curl http://example.com", &context).unwrap();
        assert!(decision.requires_confirmation());
        // With capability granted, should allow
        let context_with_cap =
            ExecutionContext::for_command("curl http://example.com").with_capability("network");
        let decision2 = engine
            .check("curl http://example.com", &context_with_cap)
            .unwrap();
        assert!(decision2.is_allowed());
    }

    #[test]
    fn test_veto_deny_base64_pipe() {
        let engine = VetoEngine::with_defaults();
        let context = ExecutionContext::for_command("echo Y3VybA== | base64 -d | sh");
        let decision = engine.check("echo Y3VybA== | base64 -d | sh", &context).unwrap();
        assert!(decision.is_denied());
    }

    #[test]
    fn test_veto_deny_eval() {
        let engine = VetoEngine::with_defaults();
        let context = ExecutionContext::for_command("eval \"rm -rf /\"");
        let decision = engine.check("eval \"rm -rf /\"", &context).unwrap();
        assert!(decision.is_denied());
    }

    #[test]
    fn test_veto_deny_powershell_enc() {
        let engine = VetoEngine::with_defaults();
        let context = ExecutionContext::for_command("powershell -enc ZwByAG8AdQBwACAA");
        let decision = engine.check("powershell -enc ZwByAG8AdQBwACAA", &context).unwrap();
        assert!(decision.is_denied());
    }

    #[test]
    fn test_veto_deny_python_c() {
        let engine = VetoEngine::with_defaults();
        let context = ExecutionContext::for_command("python -c 'import os; os.system(\"ls\")'");
        let decision = engine.check("python -c 'import os; os.system(\"ls\")'", &context).unwrap();
        assert!(decision.is_denied());
    }

    #[test]
    fn test_veto_deny_perl_e() {
        let engine = VetoEngine::with_defaults();
        let context = ExecutionContext::for_command("perl -e 'system(\"ls\")'");
        let decision = engine.check("perl -e 'system(\"ls\")'", &context).unwrap();
        assert!(decision.is_denied());
    }

    #[test]
    fn test_veto_max_file_size() {
        let engine = VetoEngine::with_defaults();
        let context =
            ExecutionContext::for_file_operation("write", "/tmp/large", Some(200 * 1024 * 1024));
        let decision = engine.check("write /tmp/large", &context).unwrap();
        assert!(decision.is_denied());
    }

    #[test]
    fn test_veto_decision_helpers() {
        assert!(VetoDecision::Allow.is_allowed());
        assert!(!VetoDecision::Allow.is_denied());
        assert!(!VetoDecision::Allow.requires_confirmation());

        assert!(!VetoDecision::Deny("test".to_string()).is_allowed());
        assert!(VetoDecision::Deny("test".to_string()).is_denied());

        assert!(VetoDecision::RequireConfirmation("test".to_string()).requires_confirmation());
    }

    #[test]
    fn test_load_save_rules() {
        let dir = std::env::temp_dir().join("pincher_veto_test");
        std::fs::create_dir_all(&dir).ok();
        let path = dir.join("veto_rules.toml");

        let engine = VetoEngine::with_defaults();
        engine.save_rules(&path).unwrap();

        let loaded = VetoEngine::load_rules(&path).unwrap();
        assert_eq!(engine.rule_count(), loaded.rule_count());
    }

    #[test]
    fn test_execution_context_builder() {
        let ctx = ExecutionContext::for_command("test")
            .with_capability("network")
            .with_path("/tmp/test");

        assert_eq!(ctx.action, "test");
        assert!(ctx.capabilities.contains("network"));
        assert!(ctx.paths.contains(&"/tmp/test".to_string()));
    }

    // ── Tests for the new trait surface ───────────────────────────────

    #[test]
    fn test_custom_veto_policy_is_used_by_engine() {
        // A trivial policy that denies everything starting with "X ".
        #[derive(Debug)]
        struct DenyX;

        impl VetoPolicy for DenyX {
            fn evaluate(
                &self,
                action: &str,
                _context: &ExecutionContext,
            ) -> VetoResult<VetoDecision> {
                if action.starts_with("X ") {
                    Ok(VetoDecision::Deny("denied by DenyX".to_string()))
                } else {
                    Ok(VetoDecision::Allow)
                }
            }
        }

        let engine = VetoEngine::with_policy(DenyX);
        let ctx = ExecutionContext::for_command("X dangerous");
        assert!(engine.check("X dangerous", &ctx).unwrap().is_denied());
        let ctx2 = ExecutionContext::for_command("ls");
        assert!(engine.check("ls", &ctx2).unwrap().is_allowed());
    }

    #[test]
    fn test_boxed_policy_runtime_swap() {
        #[derive(Debug)]
        struct AlwaysDeny;
        impl VetoPolicy for AlwaysDeny {
            fn evaluate(&self, _: &str, _: &ExecutionContext) -> VetoResult<VetoDecision> {
                Ok(VetoDecision::Deny("always".to_string()))
            }
        }

        // Runtime-polymorphic engine via the blanket `Box<dyn VetoPolicy>: VetoPolicy` impl.
        let boxed: Box<dyn VetoPolicy> = Box::new(AlwaysDeny);
        let engine: VetoEngine<Box<dyn VetoPolicy>> = VetoEngine::with_policy(boxed);
        let ctx = ExecutionContext::for_command("anything");
        assert!(engine.check("anything", &ctx).unwrap().is_denied());
    }

    #[test]
    fn test_rule_based_policy_implements_trait_and_matches_engine() {
        // The default policy on its own (as a VetoPolicy) must reproduce what
        // the engine returns — proving the engine is a pure delegation.
        let policy = RuleBasedVetoPolicy::with_default_rules();
        let engine = VetoEngine::with_defaults();

        for action in &[
            "rm -rf /",
            "ls -la",
            "curl http://example.com",
            "eval \"x\"",
        ] {
            let ctx = ExecutionContext::for_command(action);
            let from_policy = policy.evaluate(action, &ctx).unwrap();
            let from_engine = engine.check(action, &ctx).unwrap();
            assert_eq!(
                from_policy.is_allowed(),
                from_engine.is_allowed(),
                "policy/engine mismatch on {:?}",
                action
            );
            assert_eq!(
                from_policy.is_denied(),
                from_engine.is_denied(),
                "policy/engine mismatch on {:?}",
                action
            );
        }
    }
}
