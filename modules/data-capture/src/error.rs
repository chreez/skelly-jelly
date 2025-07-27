//! Error types for the data capture module

use thiserror::Error;

pub type Result<T> = std::result::Result<T, DataCaptureError>;

#[derive(Error, Debug)]
pub enum DataCaptureError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Platform error: {0}")]
    Platform(String),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    #[error("Monitor error: {0}")]
    Monitor(String),
    
    #[error("Privacy filter error: {0}")]
    Privacy(String),
    
    #[error("Channel error: {0}")]
    ChannelError(String),
    
    #[error("Resource limit exceeded: {0}")]
    ResourceLimit(String),
    
    #[error("Screenshot error: {0}")]
    Screenshot(String),
    
    #[error("Not supported on this platform")]
    NotSupported,
    
    #[error("Module not initialized")]
    NotInitialized,
    
    #[error("Module already running")]
    AlreadyRunning,
    
    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl DataCaptureError {
    /// Check if this error is recoverable
    pub fn is_recoverable(&self) -> bool {
        matches!(self, 
            Self::Io(_) | 
            Self::Monitor(_) | 
            Self::Screenshot(_) |
            Self::ResourceLimit(_)
        )
    }
    
    /// Check if this error is due to missing permissions
    pub fn is_permission_error(&self) -> bool {
        matches!(self, Self::PermissionDenied(_))
    }
}