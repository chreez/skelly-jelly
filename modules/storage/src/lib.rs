//! Skelly-Jelly Storage Module
//! 
//! High-performance event storage and batching system handling 1000+ events/second
//! with automatic screenshot lifecycle management.

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod config;
pub mod database;
pub mod error;
pub mod metrics;
pub mod types;

mod batch_manager;
mod event_receiver;
mod screenshot_manager;
mod storage_module;

pub use config::StorageConfig;
pub use error::{Result, StorageError};
pub use metrics::PerformanceMetrics;
pub use storage_module::StorageModule;

// Re-export commonly used types
pub use types::{
    BusMessage, EventBatch, RawEvent, ScreenshotEvent, ScreenshotId, ScreenshotMetadata,
};

/// Module version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Initialize storage module with default configuration
pub async fn init() -> Result<StorageModule> {
    let config = StorageConfig::default();
    StorageModule::new(config).await
}

/// Initialize storage module with custom configuration path
pub async fn init_with_config_path(path: &str) -> Result<StorageModule> {
    let config = StorageConfig::from_file(path)?;
    StorageModule::new(config).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }
}