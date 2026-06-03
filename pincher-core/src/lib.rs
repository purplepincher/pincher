//! # PincherOS Core Library
//!
//! `pincher-core` is the core library for PincherOS — a "post-model operating
//! system" using the hermit crab metaphor.

pub mod capability;
pub mod carapace;
pub mod db;
pub mod dynamics;
pub mod embed;
pub mod immunology;
pub mod intent;
pub mod migration;
pub mod reflex;
pub mod resource;
pub mod rpc;
pub mod sandbox;
pub mod security;
pub mod shell;

// ── Crate-level re-exports ──────────────────────────────────────────

pub use embed::{
    EmbedError, EmbedResult,
    cosine_similarity,
    EMBEDDING_DIM,
    download_model,
};

pub use reflex::{
    EngineError, EngineResult, EngineStatus, Execution, MatchType,
    MatchError, MatchThresholds,
    Reflex, ReflexEngine,
};

pub use db::{
    Database, DbError, DbResult,
    schema::{
        ActionLogRow, ReflexRow, SessionRow, ShellRow,
        EMBEDDING_DIM as DB_EMBEDDING_DIM,
        embed_to_bytes, bytes_to_embed,
    },
};

pub use resource::{
    PidController, ResourceBudget, ResourceController as ResourceCtrl, ResourceError, ResourceMetrics,
    ResourceResult, ResourceState, ResourceThresholds,
};

pub use security::{
    Capability as SecCapability, LandlockRule, SandboxConfig, SandboxError as SecSandboxError,
    SandboxResult as SecSandboxResult, SignedToken,
    veto::{VetoDecision, VetoEngine as SecVetoEngine, VetoError, VetoResult, VetoRule, ExecutionContext},
};

pub use capability::{
    manifest::{CapabilityManifest, Permission},
    token::CapabilityToken,
};

pub use migration::{
    compatibility_score, fingerprint, fingerprint_hash,
    FingerprintError, FingerprintResult,
    pack_nail, unpack_nail, verify_nail, read_manifest, read_identity,
    AgentConfig, AgentIdentity, AgentPreferences, NailChecksums, NailManifest,
    PackError, PackResult,
    ShellFingerprint,
};

pub use rpc::{
    start_rpc_server, EngineCommand, JsonRpcRequest, JsonRpcResponse, RpcError, RpcErrorValue,
    RpcRequest, RpcResponse,
};
