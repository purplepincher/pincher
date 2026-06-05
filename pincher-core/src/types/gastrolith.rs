//! Gastrolith checkpoint schema from PurplePincher/Pinch7 spec
//! Preserves core identity substance across migrations

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Core identity gastrolith checkpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gastrolith {
    /// Unique rigging ID (never changes for the lifetime of the agent)
    pub rigging_id: Uuid,
    /// Developmental stage: Zoea → Megalopa → Juvenile → Adult
    pub developmental_stage: String,
    /// Personality configuration
    pub personality: HashMap<String, String>,
    /// Top-K core reflexes (identity-critical, no truncation)
    pub core_reflexes: Vec<String>,
    /// Interoceptive snapshot: decision traces, state history
    pub interoceptive_snapshot: HashMap<String, serde_json::Value>,
    /// Consent records for migrations
    pub consent_records: Vec<ConsentEntry>,
    /// Shell fingerprint history
    pub shell_history: Vec<ShellFingerprint>,
    /// Timestamp of checkpoint
    pub timestamp: u64,
}

/// Consent entry for migration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsentEntry {
    pub initiator: String,
    pub timestamp: u64,
    pub constraints: HashMap<String, String>,
}

/// Shell hardware fingerprint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellFingerprint {
    pub cpu_model: String,
    pub ram_total: u64,
    pub gpu_model: Option<String>,
    pub kernel_version: String,
    pub architecture: String,
    pub fingerprint_hash: String,
}
