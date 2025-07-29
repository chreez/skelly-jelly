//! # Orchestrator Module
//!
//! System lifecycle manager and health monitor for Skelly-Jelly.
//! Manages module startup order, health monitoring, configuration distribution,
//! and resource coordination.

pub mod config;
pub mod error;
pub mod health;
pub mod lifecycle;
pub mod module_registry;
pub mod orchestrator;
pub mod recovery;
pub mod resource;
pub mod startup;
pub mod enhanced_health;
pub mod config_watcher;
pub mod performance_telemetry;
pub mod event_loss_prevention;

#[cfg(test)]
pub mod resource_management_integration_test;

// Re-export public API
pub use config::{ConfigurationManager, OrchestratorConfig};
pub use error::{OrchestratorError, OrchestratorResult};
pub use health::{HealthMonitor, HealthReport, HealthStatus, HealthMetrics};
pub use lifecycle::{LifecycleController, ModuleState, StopReason};
pub use module_registry::{ModuleRegistry, ModuleDescriptor, DependencyGraph};
pub use orchestrator::{Orchestrator, OrchestratorImpl, SystemHealth, SystemStatus};
pub use recovery::{RecoveryManager, RecoveryStrategy};
pub use resource::{ResourceManager, ResourceLimits, ResourceAllocations, SystemResources, PerformanceStats, BatteryOptimization};
pub use performance_telemetry::{PerformanceTelemetrySystem, TelemetryConfig, DashboardData, PerformanceTrends};
pub use event_loss_prevention::{EventLossPreventionSystem, EventLossPreventionConfig, EventLossStatistics};
pub use startup::{StartupSequencer, StartupMetrics, StartupPhase, StartupBottleneck};
pub use enhanced_health::{EnhancedHealthMonitor, EnhancedHealthReport, EnhancedHealthStatus, EnhancedHealthMetrics, HealthConfig};
pub use config_watcher::{ConfigWatcher, ConfigChange, HotReloadConfig, ConfigValidation};

use async_trait::async_trait;
use skelly_jelly_event_bus::{EventBusTrait, ModuleId};
use std::{sync::Arc, time::Duration};

/// Main orchestrator trait defining the public API
#[async_trait]
pub trait OrchestratorTrait: Send + Sync {
    /// Start the entire system
    async fn start_system(&self) -> OrchestratorResult<()>;
    
    /// Stop the entire system gracefully
    async fn stop_system(&self, timeout: Duration) -> OrchestratorResult<()>;
    
    /// Get system health status
    async fn get_system_health(&self) -> SystemHealth;
    
    /// Update module configuration
    async fn update_config(&self, module_id: ModuleId, config: serde_json::Value) -> OrchestratorResult<()>;
    
    /// Restart a specific module
    async fn restart_module(&self, module_id: ModuleId) -> OrchestratorResult<()>;
    
    /// Register a new module
    async fn register_module(&self, descriptor: ModuleDescriptor) -> OrchestratorResult<()>;
    
    /// Get module state
    async fn get_module_state(&self, module_id: ModuleId) -> Option<ModuleState>;
}

/// Create a new orchestrator instance
pub async fn create_orchestrator(
    config: OrchestratorConfig,
    event_bus: Arc<dyn EventBusTrait>,
) -> OrchestratorResult<Arc<dyn OrchestratorTrait>> {
    let orchestrator = OrchestratorImpl::new(config, event_bus).await?;
    Ok(Arc::new(orchestrator))
}