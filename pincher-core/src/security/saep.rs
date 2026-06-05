//! State-Action Embedding Predictor veto system from pinch5 spec

use std::time::Duration;
use once_cell::sync::Lazy;

/// SAEP veto threshold constants
pub const SAEP_VETO_WARN: f32 = 0.15;
pub const SAEP_VETO_FAIL: f32 = 0.35;
pub const MIN_TRAINING_EXAMPLES: usize = 100;

/// Check if an embedding prediction anomaly should trigger a veto
pub fn check_saep_veto(anomaly_score: f32, training_count: usize) -> VetoResult {
    if training_count < MIN_TRAINING_EXAMPLES {
        VetoResult::Pass
    } else if anomaly_score > SAEP_VETO_FAIL {
        VetoResult::Fail
    } else if anomaly_score > SAEP_VETO_WARN {
        VetoResult::Warn
    } else {
        VetoResult::Pass
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum VetoResult {
    Pass,     // Allow execution
    Warn,     // Log warning but allow
    Fail,     // Block execution
}
