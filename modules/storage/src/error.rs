//! Error types for the Storage module

use thiserror::Error;

/// Result type for Storage operations
pub type Result<T> = std::result::Result<T, StorageError>;

/// Storage module errors
#[derive(Error, Debug)]
pub enum StorageError {
    /// Database operation failed
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    /// Screenshot storage operation failed
    #[error("Screenshot storage error: {0}")]
    ScreenshotStorage(String),

    /// Event bus communication error
    #[error("Event bus error: {0}")]
    EventBus(String),

    /// Resource exhaustion (memory, disk, etc.)
    #[error("Resource exhaustion: {resource}")]
    ResourceExhaustion {
        /// The exhausted resource
        resource: String,
    },

    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(#[from] config::ConfigError),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Compression error
    #[error("Compression error: {0}")]
    Compression(String),

    /// Channel send error
    #[error("Channel send error: {0}")]
    ChannelSend(String),

    /// Channel receive error
    #[error("Channel receive error: {0}")]
    ChannelRecv(String),

    /// Invalid state
    #[error("Invalid state: {0}")]
    InvalidState(String),

    /// Shutdown requested
    #[error("Shutdown requested: {0}")]
    Shutdown(String),

    /// Timeout error
    #[error("Operation timed out: {0}")]
    Timeout(String),

    /// Other errors
    #[error("{0}")]
    Other(String),
}

impl StorageError {
    /// Check if this error is recoverable
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            Self::Database(sqlx::Error::PoolTimedOut)
                | Self::Timeout(_)
                | Self::ChannelSend(_)
                | Self::ChannelRecv(_)
        )
    }

    /// Check if this error indicates resource exhaustion
    pub fn is_resource_exhaustion(&self) -> bool {
        matches!(self, Self::ResourceExhaustion { .. })
    }

    /// Check if this error is a shutdown signal
    pub fn is_shutdown(&self) -> bool {
        matches!(self, Self::Shutdown(_))
    }
}

// Implement From for common error conversions
impl From<bincode::Error> for StorageError {
    fn from(err: bincode::Error) -> Self {
        Self::Serialization(err.to_string())
    }
}

impl From<serde_json::Error> for StorageError {
    fn from(err: serde_json::Error) -> Self {
        Self::Serialization(err.to_string())
    }
}

impl<T> From<tokio::sync::mpsc::error::SendError<T>> for StorageError {
    fn from(err: tokio::sync::mpsc::error::SendError<T>) -> Self {
        Self::ChannelSend(err.to_string())
    }
}

impl From<tokio::sync::mpsc::error::RecvError> for StorageError {
    fn from(err: tokio::sync::mpsc::error::RecvError) -> Self {
        Self::ChannelRecv(err.to_string())
    }
}

impl From<lz4::liblz4::Error> for StorageError {
    fn from(err: lz4::liblz4::Error) -> Self {
        Self::Compression(format!("LZ4 error: {}", err))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_recovery_classification() {
        assert!(StorageError::Timeout("test".into()).is_recoverable());
        assert!(!StorageError::Config(config::ConfigError::Message("test".into())).is_recoverable());
    }

    #[test]
    fn test_error_resource_exhaustion() {
        assert!(StorageError::ResourceExhaustion {
            resource: "memory".into()
        }
        .is_resource_exhaustion());
        assert!(!StorageError::Database(sqlx::Error::RowNotFound).is_resource_exhaustion());
    }

    #[test]
    fn test_error_shutdown() {
        assert!(StorageError::Shutdown("test".into()).is_shutdown());
        assert!(!StorageError::Other("test".into()).is_shutdown());
    }
}