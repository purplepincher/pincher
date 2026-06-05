//! Chord compression and .nail format extensions from PurplePincher/Pinch7 spec

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Partitioned CRDT cell (64-byte aligned for GPU safety)
#[repr(C, align(64))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrdtCell {
    pub cell_type: CrdtType,
    pub key: String,
    pub value: serde_json::Value,
    pub version_vector: VersionVector,
    pub last_updated: u64,
    pub access_count: u64,
}

/// CRDT cell types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CrdtType {
    LwwRegister,
    PnCounter,
    GCounter,
    OrSet,
    TrustScore,
}

/// Version vector for CRDT synchronization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionVector {
    pub peer_id: String,
    pub counter: u64,
}

/// Compression partition strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompressionPartition {
    Hot,    // >100 accesses/sec: reversible op-based CRDTs
    Cold,   // <1 access/sec: irreversible CRDTs
}

/// Shell thermal profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellThermalProfile {
    pub platform_id: String,
    pub tdp_watts: f32,
    pub idle_power_watts: f32,
    pub thermal_resistance: f32,
    pub throttle_temp_c: f32,
    pub shutdown_temp_c: f32,
    pub ram_total_gb: f32,
    pub cpu_freq_ghz: f32,
    pub peak_flops_tflops: f32,
    pub has_gpu: bool,
}
