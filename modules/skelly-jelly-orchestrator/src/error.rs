//! Error types for the orchestrator module

use skelly_jelly_event_bus::{EventBusError, ModuleId};
use thiserror::Error;

/// Result type for orchestrator operations
pub type OrchestratorResult<T> = Result<T, OrchestratorError>;

/// Errors that can occur in the orchestrator
#[derive(Error, Debug)]
pub enum OrchestratorError {
    #[error("Event bus error: {0}")]
    EventBus(#[from] EventBusError),

    #[error("Module {module} failed to start: {reason}")]
    ModuleStartupFailed {
        module: ModuleId,
        reason: String,
    },

    #[error("Module {module} failed to stop: {reason}")]
    ModuleShutdownFailed {
        module: ModuleId,
        reason: String,
    },

    #[error("Dependency cycle detected: {cycle:?}")]
    DependencyCycle {
        cycle: Vec<ModuleId>,
    },

    #[error("Missing dependency: {module} requires {dependency}")]
    MissingDependency {
        module: ModuleId,
        dependency: ModuleId,
    },

    #[error("Configuration error for {module}: {reason}")]
    ConfigurationError {
        module: ModuleId,
        reason: String,
    },

    #[error("Health check failed for {module}: {reason}")]
    HealthCheckFailed {
        module: ModuleId,
        reason: String,
    },

    #[error("Resource limit exceeded for {module}: {resource}")]
    ResourceLimitExceeded {
        module: ModuleId,
        resource: String,
    },

    #[error("System startup timeout after {timeout_secs} seconds")]
    StartupTimeout {
        timeout_secs: u64,
    },

    #[error("System shutdown timeout after {timeout_secs} seconds")]
    ShutdownTimeout {
        timeout_secs: u64,
    },

    #[error("Recovery failed for {module}: {reason}")]
    RecoveryFailed {
        module: ModuleId,
        reason: String,
    },

    #[error("File system error: {0}")]
    FileSystem(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("System resource error: {0}")]
    SystemResource(String),

    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),
}