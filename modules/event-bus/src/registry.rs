//! Module registry for tracking registered modules and their health status

use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use parking_lot::RwLock;

use crate::{ModuleId, EventBusError, EventBusResult};

/// Health status of a module
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ModuleStatus {
    /// Module is starting up
    Starting,
    /// Module is healthy and operational
    Healthy,
    /// Module is responding but with degraded performance
    Degraded,
    /// Module is not responding to health checks
    Unhealthy,
    /// Module is shutting down gracefully
    ShuttingDown,
    /// Module has stopped
    Stopped,
}

/// Information about a registered module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleInfo {
    /// Module identifier
    pub module_id: ModuleId,
    /// Current status
    pub status: ModuleStatus,
    /// When the module was registered
    pub registered_at: DateTime<Utc>,
    /// Last successful health check
    pub last_health_check: Option<DateTime<Utc>>,
    /// Last health check response time
    pub last_response_time: Option<Duration>,
    /// Module version (if provided)
    pub version: Option<String>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl ModuleInfo {
    /// Create new module info for registration
    pub fn new(module_id: ModuleId) -> Self {
        Self {
            module_id,
            status: ModuleStatus::Starting,
            registered_at: Utc::now(),
            last_health_check: None,
            last_response_time: None,
            version: None,
            metadata: HashMap::new(),
        }
    }

    /// Set module version
    pub fn with_version(mut self, version: String) -> Self {
        self.version = Some(version);
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Update health check status
    pub fn update_health(&mut self, status: ModuleStatus, response_time: Option<Duration>) {
        self.status = status;
        self.last_health_check = Some(Utc::now());
        self.last_response_time = response_time;
    }

    /// Check if module has been unresponsive for too long
    pub fn is_stale(&self, stale_threshold: Duration) -> bool {
        match self.last_health_check {
            Some(last_check) => {
                let elapsed = Utc::now().signed_duration_since(last_check);
                elapsed.to_std().unwrap_or(Duration::ZERO) > stale_threshold
            }
            None => {
                // If no health check yet, check registration time
                let elapsed =Utc::now().signed_duration_since(self.registered_at);
                elapsed.to_std().unwrap_or(Duration::ZERO) > stale_threshold
            }
        }
    }
}

/// Registry configuration
#[derive(Debug, Clone)]
pub struct RegistryConfig {
    /// How long to wait before marking a module as stale
    pub stale_threshold: Duration,
    /// How often to run background cleanup
    pub cleanup_interval: Duration,
    /// Maximum time to wait for health check response
    pub health_check_timeout: Duration,
}

impl Default for RegistryConfig {
    fn default() -> Self {
        Self {
            stale_threshold: Duration::from_secs(30),
            cleanup_interval: Duration::from_secs(10),
            health_check_timeout: Duration::from_secs(5),
        }
    }
}

/// Health check request details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckRequest {
    pub module_id: ModuleId,
    pub timestamp: DateTime<Utc>,
}

/// Health check response details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResponse {
    pub module_id: ModuleId,
    pub status: ModuleStatus,
    pub timestamp: DateTime<Utc>,
    pub response_time_ms: u64,
    pub details: Option<String>,
}

/// Module registry for tracking all registered modules
pub struct ModuleRegistry {
    /// Registered modules
    modules: RwLock<HashMap<ModuleId, ModuleInfo>>,
    /// Configuration
    config: RegistryConfig,
    /// Pending health check requests
    pending_health_checks: RwLock<HashMap<Uuid, (ModuleId, SystemTime)>>,
}

impl ModuleRegistry {
    /// Create a new module registry
    pub fn new(config: RegistryConfig) -> Self {
        Self {
            modules: RwLock::new(HashMap::new()),
            config,
            pending_health_checks: RwLock::new(HashMap::new()),
        }
    }

    /// Register a new module
    pub fn register_module(&self, module_info: ModuleInfo) -> EventBusResult<()> {
        let mut modules = self.modules.write();
        
        if modules.contains_key(&module_info.module_id) {
            return Err(EventBusError::ModuleAlreadyRegistered {
                module_id: module_info.module_id,
            });
        }

        modules.insert(module_info.module_id, module_info);
        Ok(())
    }

    /// Unregister a module
    pub fn unregister_module(&self, module_id: ModuleId) -> EventBusResult<ModuleInfo> {
        let mut modules = self.modules.write();
        
        modules.remove(&module_id).ok_or(EventBusError::ModuleNotFound { module_id })
    }

    /// Get information about a specific module
    pub fn get_module_info(&self, module_id: ModuleId) -> Option<ModuleInfo> {
        self.modules.read().get(&module_id).cloned()
    }

    /// Get all registered modules
    pub fn get_all_modules(&self) -> Vec<ModuleInfo> {
        self.modules.read().values().cloned().collect()
    }

    /// Get modules by status
    pub fn get_modules_by_status(&self, status: ModuleStatus) -> Vec<ModuleInfo> {
        self.modules
            .read()
            .values()
            .filter(|info| info.status == status)
            .cloned()
            .collect()
    }

    /// Update module status
    pub fn update_module_status(
        &self,
        module_id: ModuleId,
        status: ModuleStatus,
        response_time: Option<Duration>,
    ) -> EventBusResult<()> {
        let mut modules = self.modules.write();
        
        match modules.get_mut(&module_id) {
            Some(module_info) => {
                module_info.update_health(status, response_time);
                Ok(())
            }
            None => Err(EventBusError::ModuleNotFound { module_id }),
        }
    }

    /// Mark a module as ready (transition from Starting to Healthy)
    pub fn mark_module_ready(&self, module_id: ModuleId) -> EventBusResult<()> {
        self.update_module_status(module_id, ModuleStatus::Healthy, None)
    }

    /// Mark a module as shutting down
    pub fn mark_module_shutting_down(&self, module_id: ModuleId) -> EventBusResult<()> {
        self.update_module_status(module_id, ModuleStatus::ShuttingDown, None)
    }

    /// Get count of modules by status
    pub fn get_status_counts(&self) -> HashMap<ModuleStatus, usize> {
        let modules = self.modules.read();
        let mut counts = HashMap::new();
        
        for info in modules.values() {
            *counts.entry(info.status.clone()).or_insert(0) += 1;
        }
        
        counts
    }

    /// Check for stale modules that haven't responded to health checks
    pub fn find_stale_modules(&self) -> Vec<ModuleId> {
        self.modules
            .read()
            .values()
            .filter(|info| info.is_stale(self.config.stale_threshold))
            .map(|info| info.module_id)
            .collect()
    }

    /// Get overall system health summary
    pub fn get_health_summary(&self) -> HealthSummary {
        let modules = self.modules.read();
        let total_modules = modules.len();
        let mut healthy_count = 0;
        let mut degraded_count = 0;
        let mut unhealthy_count = 0;
        let mut starting_count = 0;
        let mut shutting_down_count = 0;
        let mut stopped_count = 0;

        for info in modules.values() {
            match info.status {
                ModuleStatus::Healthy => healthy_count += 1,
                ModuleStatus::Degraded => degraded_count += 1,
                ModuleStatus::Unhealthy => unhealthy_count += 1,
                ModuleStatus::Starting => starting_count += 1,
                ModuleStatus::ShuttingDown => shutting_down_count += 1,
                ModuleStatus::Stopped => stopped_count += 1,
            }
        }

        let overall_health = if unhealthy_count > 0 {
            SystemHealth::Critical
        } else if degraded_count > 0 {
            SystemHealth::Degraded
        } else if starting_count > 0 || shutting_down_count > 0 {
            SystemHealth::Transitioning
        } else if healthy_count == total_modules && total_modules > 0 {
            SystemHealth::Healthy
        } else {
            SystemHealth::Unknown
        };

        HealthSummary {
            overall_health,
            total_modules,
            healthy_count,
            degraded_count,
            unhealthy_count,
            starting_count,
            shutting_down_count,
            stopped_count,
            timestamp: Utc::now(),
        }
    }

    /// Record a pending health check request
    pub fn record_health_check_request(&self, request_id: Uuid, module_id: ModuleId) {
        self.pending_health_checks
            .write()
            .insert(request_id, (module_id, SystemTime::now()));
    }

    /// Process a health check response
    pub fn process_health_check_response(
        &self,
        request_id: Uuid,
        response: HealthCheckResponse,
    ) -> EventBusResult<()> {
        let (module_id, request_time) = {
            let mut pending = self.pending_health_checks.write();
            pending.remove(&request_id).ok_or(EventBusError::InvalidHealthCheckResponse)?
        };

        // Verify the response is for the correct module
        if module_id != response.module_id {
            return Err(EventBusError::InvalidHealthCheckResponse);
        }

        // Calculate response time
        let response_time = SystemTime::now()
            .duration_since(request_time)
            .unwrap_or(Duration::ZERO);

        // Update module status
        self.update_module_status(module_id, response.status, Some(response_time))
    }

    /// Clean up expired health check requests
    pub fn cleanup_expired_health_checks(&self) {
        let mut pending = self.pending_health_checks.write();
        let timeout = self.config.health_check_timeout;
        let now = SystemTime::now();

        pending.retain(|_, (_, request_time)| {
            now.duration_since(*request_time).unwrap_or(Duration::ZERO) < timeout
        });
    }
}

/// Overall system health status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SystemHealth {
    /// All modules are healthy
    Healthy,
    /// Some modules are degraded but functional
    Degraded,
    /// Some modules are unhealthy
    Critical,
    /// System is starting up or shutting down
    Transitioning,
    /// Unable to determine health status
    Unknown,
}

/// Summary of system health
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthSummary {
    pub overall_health: SystemHealth,
    pub total_modules: usize,
    pub healthy_count: usize,
    pub degraded_count: usize,
    pub unhealthy_count: usize,
    pub starting_count: usize,
    pub shutting_down_count: usize,
    pub stopped_count: usize,
    pub timestamp: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_registration() {
        let registry = ModuleRegistry::new(RegistryConfig::default());
        let module_info = ModuleInfo::new(ModuleId::DataCapture)
            .with_version("1.0.0".to_string());

        // Register module
        assert!(registry.register_module(module_info.clone()).is_ok());

        // Try to register again (should fail)
        assert!(registry.register_module(module_info).is_err());

        // Get module info
        let retrieved = registry.get_module_info(ModuleId::DataCapture);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().status, ModuleStatus::Starting);
    }

    #[test]
    fn test_module_status_updates() {
        let registry = ModuleRegistry::new(RegistryConfig::default());
        let module_info = ModuleInfo::new(ModuleId::Storage);

        registry.register_module(module_info).unwrap();

        // Mark as ready
        assert!(registry.mark_module_ready(ModuleId::Storage).is_ok());

        let info = registry.get_module_info(ModuleId::Storage).unwrap();
        assert_eq!(info.status, ModuleStatus::Healthy);
        assert!(info.last_health_check.is_some());
    }

    #[test]
    fn test_health_summary() {
        let registry = ModuleRegistry::new(RegistryConfig::default());

        // Register multiple modules with different statuses
        registry.register_module(ModuleInfo::new(ModuleId::DataCapture)).unwrap();
        registry.register_module(ModuleInfo::new(ModuleId::Storage)).unwrap();
        registry.register_module(ModuleInfo::new(ModuleId::AnalysisEngine)).unwrap();

        registry.mark_module_ready(ModuleId::DataCapture).unwrap();
        registry.update_module_status(
            ModuleId::Storage, 
            ModuleStatus::Degraded, 
            Some(Duration::from_millis(100))
        ).unwrap();

        let summary = registry.get_health_summary();
        assert_eq!(summary.overall_health, SystemHealth::Degraded);
        assert_eq!(summary.total_modules, 3);
        assert_eq!(summary.healthy_count, 1);
        assert_eq!(summary.degraded_count, 1);
        assert_eq!(summary.starting_count, 1);
    }

    #[test]
    fn test_stale_module_detection() {
        let config = RegistryConfig {
            stale_threshold: Duration::from_millis(1), // Very short for testing
            ..Default::default()
        };
        let registry = ModuleRegistry::new(config);

        let mut module_info = ModuleInfo::new(ModuleId::DataCapture);
        module_info.registered_at = Utc::now() - chrono::Duration::milliseconds(100);
        
        registry.register_module(module_info).unwrap();

        std::thread::sleep(Duration::from_millis(2));

        let stale_modules = registry.find_stale_modules();
        assert!(stale_modules.contains(&ModuleId::DataCapture));
    }
}