//! Error types for the Hybrid Manifold bridge.

use thiserror::Error;

/// Errors that can occur during hybrid bridge operations.
#[derive(Debug, Error)]
pub enum HybridError {
    #[error("Ticker {0} not found in matrix")]
    TickerNotFound(String),

    #[error("Feature tensor dimension mismatch: expected {expected}, got {actual}")]
    DimensionMismatch { expected: usize, actual: usize },

    #[error("TDA computation failed: {0}")]
    TdaError(String),

    #[error("Matrix snapshot too old: {elapsed}ms")]
    StaleSnapshot { elapsed: u64 },

    #[error("Veto freeze active: {reason}")]
    Frozen { reason: String },

    #[error("Bridge channel closed")]
    ChannelClosed,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Join error: {0}")]
    Join(#[from] tokio::task::JoinError),

    #[error("Internal error: {0}")]
    Internal(String),
}

/// Convenience result alias for hybrid bridge operations.
pub type HybridResult<T> = Result<T, HybridError>;
