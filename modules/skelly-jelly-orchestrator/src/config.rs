//! Configuration management for the orchestrator

use crate::error::{OrchestratorError, OrchestratorResult};
use dashmap::DashMap;
use skelly_jelly_event_bus::{EventBusTrait, ModuleId, BusMessage, MessagePayload};
use notify::RecommendedWatcher;
use serde::{Deserialize, Serialize};
use std::{
    path::Path,
    sync::Arc,
    time::Duration,
};
use tokio::sync::RwLock;
use tracing::{info, warn, error};

/// Configuration for the orchestrator module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorConfig {
    /// Startup configuration
    pub startup_timeout: Duration,
    pub module_start_delay: Duration,
    pub parallel_startup: bool,
    
    /// Health monitoring
    pub health_check_interval: Duration,
    pub health_check_timeout: Duration,
    pub unhealthy_threshold: u32,
    
    /// Recovery settings
    pub auto_recovery: bool,
    pub max_recovery_attempts: u32,
    pub recovery_backoff: Duration,
    
    /// Resource management
    pub resource_check_interval: Duration,
    pub throttle_threshold: f32,
}

impl Default for OrchestratorConfig {
    fn default() -> Self {
        Self {
            startup_timeout: Duration::from_secs(60),
            module_start_delay: Duration::from_secs(1),
            parallel_startup: false,
            health_check_interval: Duration::from_secs(30),
            health_check_timeout: Duration::from_secs(5),
            unhealthy_threshold: 3,
            auto_recovery: true,
            max_recovery_attempts: 3,
            recovery_backoff: Duration::from_secs(10),
            resource_check_interval: Duration::from_secs(10),
            throttle_threshold: 0.9,
        }
    }
}

/// Configuration storage
#[derive(Debug)]
pub struct ConfigStore {
    /// Global configuration
    pub global: OrchestratorConfig,
    
    /// Module-specific configs
    pub module_configs: DashMap<ModuleId, serde_json::Value>,
    
    /// Runtime overrides
    pub overrides: DashMap<String, serde_json::Value>,
}

impl ConfigStore {
    pub fn new(global: OrchestratorConfig) -> Self {
        Self {
            global,
            module_configs: DashMap::new(),
            overrides: DashMap::new(),
        }
    }

    pub fn update_module_config(&self, module_id: ModuleId, config: serde_json::Value) {
        self.module_configs.insert(module_id, config);
    }

    pub fn get_module_config(&self, module_id: ModuleId) -> Option<serde_json::Value> {
        self.module_configs.get(&module_id).map(|entry| entry.clone())
    }

    pub fn set_override(&self, key: String, value: serde_json::Value) {
        self.overrides.insert(key, value);
    }

    pub fn get_override(&self, key: &str) -> Option<serde_json::Value> {
        self.overrides.get(key).map(|entry| entry.clone())
    }
}

/// Configuration manager handles config distribution and hot-reloading
pub struct ConfigurationManager {
    /// Configuration storage
    config_store: Arc<RwLock<ConfigStore>>,
    
    /// Event bus for notifications
    event_bus: Arc<dyn EventBusTrait>,
    
    /// File watcher for hot-reload
    _watcher: Option<RecommendedWatcher>,
}

impl ConfigurationManager {
    pub fn new(
        config: OrchestratorConfig,
        event_bus: Arc<dyn EventBusTrait>,
    ) -> Self {
        let config_store = Arc::new(RwLock::new(ConfigStore::new(config)));
        
        Self {
            config_store,
            event_bus,
            _watcher: None,
        }
    }

    /// Update configuration for a module
    pub async fn update_config(
        &self,
        module_id: ModuleId,
        config: serde_json::Value,
    ) -> OrchestratorResult<()> {
        // Validate configuration (basic JSON validation for now)
        if !config.is_object() {
            return Err(OrchestratorError::ConfigurationError {
                module: module_id,
                reason: "Configuration must be a JSON object".to_string(),
            });
        }

        // Store new config
        {
            let store = self.config_store.read().await;
            store.update_module_config(module_id, config.clone());
        }

        // Notify module of config change
        let config_update = skelly_jelly_event_bus::message::ConfigUpdate {
            config_key: format!("{}_config", module_id),
            config_value: config,
            target_module: Some(module_id),
        };

        let message = BusMessage::new(
            ModuleId::Orchestrator,
            MessagePayload::ConfigUpdate(config_update),
        );

        self.event_bus.publish(message).await
            .map_err(OrchestratorError::EventBus)?;

        info!("Updated configuration for module: {}", module_id);
        Ok(())
    }

    /// Get configuration for a module
    pub async fn get_config(&self, module_id: ModuleId) -> Option<serde_json::Value> {
        let store = self.config_store.read().await;
        store.get_module_config(module_id)
    }

    /// Set a runtime override
    pub async fn set_override(&self, key: String, value: serde_json::Value) {
        let store = self.config_store.read().await;
        store.set_override(key, value);
    }

    /// Get a runtime override
    pub async fn get_override(&self, key: &str) -> Option<serde_json::Value> {
        let store = self.config_store.read().await;
        store.get_override(key)
    }

    /// Get global configuration
    pub async fn get_global_config(&self) -> OrchestratorConfig {
        let store = self.config_store.read().await;
        store.global.clone()
    }

    /// Watch configuration files for changes (placeholder implementation)
    pub fn watch_config_files<P: AsRef<Path>>(&mut self, _config_dir: P) -> OrchestratorResult<()> {
        // For now, we'll implement a basic file watcher
        // In a full implementation, this would watch the config directory
        // and trigger config reloads when files change
        
        info!("Configuration file watching is not yet implemented");
        Ok(())
    }

    /// Validate configuration for a module (basic implementation)
    fn _validate_config(&self, _config: &serde_json::Value) -> OrchestratorResult<()> {
        // Basic validation - in a real implementation, this would use
        // JSON schema validation or custom validators per module
        Ok(())
    }
}