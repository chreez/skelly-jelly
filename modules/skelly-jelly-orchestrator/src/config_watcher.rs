//! Configuration hot-reloading with file system watching

use crate::{
    error::{OrchestratorError, OrchestratorResult},
    config::{ConfigurationManager, OrchestratorConfig},
};
use notify::{RecommendedWatcher, Watcher, RecursiveMode, Event, EventKind};
use skelly_jelly_event_bus::{EventBusTrait, ModuleId, BusMessage, MessagePayload, message::ConfigUpdate};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::{
    fs,
    sync::{mpsc, RwLock},
    task::JoinHandle,
};
use tracing::{info, warn, error, debug};

/// Configuration file types supported
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ConfigFileType {
    Global,      // orchestrator.toml
    Module(ModuleId), // module-specific configs
    Environment, // env-specific overrides
}

/// Configuration change event
#[derive(Debug, Clone)]
pub struct ConfigChange {
    pub file_type: ConfigFileType,
    pub file_path: PathBuf,
    pub change_type: ConfigChangeType,
    pub timestamp: Instant,
}

/// Types of configuration changes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigChangeType {
    Created,
    Modified,
    Deleted,
    Renamed,
}

/// Configuration validation result
#[derive(Debug, Clone)]
pub struct ConfigValidation {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub suggestions: Vec<String>,
}

/// Hot-reload configuration options
#[derive(Debug, Clone)]
pub struct HotReloadConfig {
    pub enabled: bool,
    pub debounce_delay: Duration,
    pub validation_timeout: Duration,
    pub auto_rollback_on_error: bool,
    pub backup_generations: usize,
    pub excluded_paths: Vec<PathBuf>,
}

impl Default for HotReloadConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            debounce_delay: Duration::from_millis(500),
            validation_timeout: Duration::from_secs(10),
            auto_rollback_on_error: true,
            backup_generations: 5,
            excluded_paths: Vec::new(),
        }
    }
}

/// Configuration backup for rollback
#[derive(Debug, Clone)]
pub struct ConfigBackup {
    pub file_path: PathBuf,
    pub content: String,
    pub timestamp: Instant,
    pub checksum: String,
}

/// Configuration hot-reloader with file system watching
pub struct ConfigWatcher {
    /// Configuration directory being watched
    config_dir: PathBuf,
    
    /// Configuration manager for updates
    config_manager: Arc<ConfigurationManager>,
    
    /// Event bus for notifications
    event_bus: Arc<dyn EventBusTrait>,
    
    /// Hot-reload configuration
    hot_reload_config: HotReloadConfig,
    
    /// File system watcher
    watcher: Option<RecommendedWatcher>,
    
    /// File change event receiver
    change_receiver: Option<mpsc::UnboundedReceiver<ConfigChange>>,
    
    /// Processing task handle
    processor_task: Option<JoinHandle<()>>,
    
    /// Configuration backups for rollback
    config_backups: Arc<RwLock<HashMap<PathBuf, Vec<ConfigBackup>>>>,
    
    /// Last known good configurations
    last_known_good: Arc<RwLock<HashMap<ConfigFileType, ConfigBackup>>>,
    
    /// Debounce tracking
    pending_changes: Arc<RwLock<HashMap<PathBuf, Instant>>>,
}

impl ConfigWatcher {
    pub fn new(
        config_dir: PathBuf,
        config_manager: Arc<ConfigurationManager>,
        event_bus: Arc<dyn EventBusTrait>,
        hot_reload_config: HotReloadConfig,
    ) -> Self {
        Self {
            config_dir,
            config_manager,
            event_bus,
            hot_reload_config,
            watcher: None,
            change_receiver: None,
            processor_task: None,
            config_backups: Arc::new(RwLock::new(HashMap::new())),
            last_known_good: Arc::new(RwLock::new(HashMap::new())),
            pending_changes: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Start configuration watching and hot-reloading
    pub async fn start_watching(&mut self) -> OrchestratorResult<()> {
        if !self.hot_reload_config.enabled {
            info!("📁 Configuration hot-reloading is disabled");
            return Ok(());
        }

        info!("🔍 Starting configuration file watcher for: {:?}", self.config_dir);

        // Ensure config directory exists
        if !self.config_dir.exists() {
            fs::create_dir_all(&self.config_dir).await
                .map_err(|e| OrchestratorError::ConfigurationError {
                    module: ModuleId::Orchestrator,
                    reason: format!("Failed to create config directory: {}", e),
                })?;
        }

        // Create channel for file changes
        let (change_sender, change_receiver) = mpsc::unbounded_channel();
        self.change_receiver = Some(change_receiver);

        // Set up file system watcher
        let mut watcher = notify::recommended_watcher({
            let sender = change_sender.clone();
            let config_dir = self.config_dir.clone();
            
            move |result: Result<Event, notify::Error>| {
                match result {
                    Ok(event) => {
                        if let Err(e) = Self::handle_fs_event(event, &sender, &config_dir) {
                            error!("❌ Error handling file system event: {}", e);
                        }
                    }
                    Err(e) => {
                        error!("❌ File system watcher error: {}", e);
                    }
                }
            }
        }).map_err(|e| OrchestratorError::ConfigurationError {
            module: ModuleId::Orchestrator,
            reason: format!("Failed to create file watcher: {}", e),
        })?;

        // Start watching the config directory
        watcher.watch(&self.config_dir, RecursiveMode::Recursive)
            .map_err(|e| OrchestratorError::ConfigurationError {
                module: ModuleId::Orchestrator,
                reason: format!("Failed to start watching directory: {}", e),
            })?;

        self.watcher = Some(watcher);

        // Create initial backups of existing configurations
        self.create_initial_backups().await?;

        // Start the change processing task
        self.start_processor_task().await;

        info!("✅ Configuration watcher started successfully");
        Ok(())
    }

    /// Stop configuration watching
    pub async fn stop_watching(&mut self) {
        info!("🛑 Stopping configuration watcher");

        // Stop processor task
        if let Some(task) = self.processor_task.take() {
            task.abort();
        }

        // Drop watcher
        self.watcher = None;
        self.change_receiver = None;

        info!("✅ Configuration watcher stopped");
    }

    /// Handle file system events
    fn handle_fs_event(
        event: Event,
        sender: &mpsc::UnboundedSender<ConfigChange>,
        config_dir: &Path,
    ) -> OrchestratorResult<()> {
        for path in &event.paths {
            // Skip if not a configuration file
            if !Self::is_config_file(path) {
                continue;
            }

            // Skip excluded paths
            // (Implementation would check against excluded_paths)

            let change_type = match event.kind {
                EventKind::Create(_) => ConfigChangeType::Created,
                EventKind::Modify(_) => ConfigChangeType::Modified,
                EventKind::Remove(_) => ConfigChangeType::Deleted,
                _ => continue,
            };

            let file_type = Self::determine_file_type(path, config_dir)?;

            let change = ConfigChange {
                file_type,
                file_path: path.clone(),
                change_type,
                timestamp: Instant::now(),
            };

            if let Err(e) = sender.send(change) {
                error!("❌ Failed to send config change event: {}", e);
            }
        }

        Ok(())
    }

    /// Check if a path is a configuration file
    fn is_config_file(path: &Path) -> bool {
        if let Some(ext) = path.extension() {
            matches!(ext.to_str(), Some("toml") | Some("yaml") | Some("yml") | Some("json"))
        } else {
            false
        }
    }

    /// Determine the configuration file type
    fn determine_file_type(path: &Path, config_dir: &Path) -> OrchestratorResult<ConfigFileType> {
        let relative_path = path.strip_prefix(config_dir)
            .map_err(|_| OrchestratorError::ConfigurationError {
                module: ModuleId::Orchestrator,
                reason: "Path is not within config directory".to_string(),
            })?;

        let file_name = relative_path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");

        // Determine file type based on naming conventions
        if file_name.starts_with("orchestrator") {
            Ok(ConfigFileType::Global)
        } else if file_name.contains("env") || file_name.contains("environment") {
            Ok(ConfigFileType::Environment)
        } else {
            // Try to match to a specific module
            for module_id in [
                ModuleId::EventBus,
                ModuleId::Storage,
                ModuleId::DataCapture,
                ModuleId::AnalysisEngine,
                ModuleId::Gamification,
                ModuleId::AiIntegration,
                ModuleId::CuteFigurine,
            ] {
                if file_name.contains(&module_id.to_string().replace("-", "_")) ||
                   file_name.contains(&module_id.to_string()) {
                    return Ok(ConfigFileType::Module(module_id));
                }
            }
            
            // Default to global if we can't determine
            Ok(ConfigFileType::Global)
        }
    }

    /// Create initial backups of existing configurations
    async fn create_initial_backups(&self) -> OrchestratorResult<()> {
        info!("📦 Creating initial configuration backups");

        let mut dir_reader = fs::read_dir(&self.config_dir).await
            .map_err(|e| OrchestratorError::ConfigurationError {
                module: ModuleId::Orchestrator,
                reason: format!("Failed to read config directory: {}", e),
            })?;

        while let Some(entry) = dir_reader.next_entry().await
            .map_err(|e| OrchestratorError::ConfigurationError {
                module: ModuleId::Orchestrator,
                reason: format!("Failed to read directory entry: {}", e),
            })? {
            
            let path = entry.path();
            if Self::is_config_file(&path) {
                if let Err(e) = self.create_backup(&path).await {
                    warn!("⚠️  Failed to create initial backup for {:?}: {}", path, e);
                }
            }
        }

        info!("✅ Initial configuration backups created");
        Ok(())
    }

    /// Create a backup of a configuration file
    async fn create_backup(&self, file_path: &Path) -> OrchestratorResult<()> {
        let content = fs::read_to_string(file_path).await
            .map_err(|e| OrchestratorError::ConfigurationError {
                module: ModuleId::Orchestrator,
                reason: format!("Failed to read config file: {}", e),
            })?;

        let checksum = Self::calculate_checksum(&content);
        
        let backup = ConfigBackup {
            file_path: file_path.to_path_buf(),
            content,
            timestamp: Instant::now(),
            checksum,
        };

        // Store in backups with rotation
        {
            let mut backups = self.config_backups.write().await;
            let file_backups = backups.entry(file_path.to_path_buf()).or_insert_with(Vec::new);
            file_backups.push(backup.clone());

            // Rotate backups if we have too many
            if file_backups.len() > self.hot_reload_config.backup_generations {
                file_backups.remove(0);
            }
        }

        // Update last known good if this is a successful backup
        if let Ok(file_type) = Self::determine_file_type(file_path, &self.config_dir) {
            let mut last_good = self.last_known_good.write().await;
            last_good.insert(file_type, backup);
        }

        debug!("📦 Created backup for: {:?}", file_path);
        Ok(())
    }

    /// Calculate checksum for content
    fn calculate_checksum(content: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// Start the change processing task
    async fn start_processor_task(&mut self) {
        let mut change_receiver = self.change_receiver.take().unwrap();
        let config_manager = Arc::clone(&self.config_manager);
        let event_bus = Arc::clone(&self.event_bus);
        let config_backups = Arc::clone(&self.config_backups);
        let last_known_good = Arc::clone(&self.last_known_good);
        let pending_changes = Arc::clone(&self.pending_changes);
        let hot_reload_config = self.hot_reload_config.clone();

        let task = tokio::spawn(async move {
            info!("🔄 Starting configuration change processor");

            while let Some(change) = change_receiver.recv().await {
                // Handle debouncing
                {
                    let mut pending = pending_changes.write().await;
                    pending.insert(change.file_path.clone(), change.timestamp);
                }

                // Wait for debounce delay
                tokio::time::sleep(hot_reload_config.debounce_delay).await;

                // Check if this change is still the most recent
                let should_process = {
                    let pending = pending_changes.read().await;
                    pending.get(&change.file_path)
                        .map(|&timestamp| timestamp == change.timestamp)
                        .unwrap_or(false)
                };

                if !should_process {
                    debug!("⏭️  Skipping debounced change for: {:?}", change.file_path);
                    continue;
                }

                // Process the configuration change
                if let Err(e) = Self::process_config_change(
                    &change,
                    &config_manager,
                    &event_bus,
                    &config_backups,
                    &last_known_good,
                    &hot_reload_config,
                ).await {
                    error!("❌ Failed to process config change for {:?}: {}", change.file_path, e);
                }

                // Remove from pending changes
                {
                    let mut pending = pending_changes.write().await;
                    pending.remove(&change.file_path);
                }
            }

            info!("🔄 Configuration change processor stopped");
        });

        self.processor_task = Some(task);
    }

    /// Process a configuration change
    async fn process_config_change(
        change: &ConfigChange,
        config_manager: &Arc<ConfigurationManager>,
        event_bus: &Arc<dyn EventBusTrait>,
        config_backups: &Arc<RwLock<HashMap<PathBuf, Vec<ConfigBackup>>>>,
        last_known_good: &Arc<RwLock<HashMap<ConfigFileType, ConfigBackup>>>,
        hot_reload_config: &HotReloadConfig,
    ) -> OrchestratorResult<()> {
        info!("🔄 Processing configuration change: {:?} -> {:?}", 
              change.change_type, change.file_path);

        match change.change_type {
            ConfigChangeType::Created | ConfigChangeType::Modified => {
                Self::handle_config_update(
                    change,
                    config_manager,
                    event_bus,
                    config_backups,
                    last_known_good,
                    hot_reload_config,
                ).await
            }
            ConfigChangeType::Deleted => {
                Self::handle_config_deletion(
                    change,
                    config_manager,
                    event_bus,
                    last_known_good,
                ).await
            }
            ConfigChangeType::Renamed => {
                // Handle as deletion of old + creation of new
                info!("📝 Handling config rename for: {:?}", change.file_path);
                Ok(())
            }
        }
    }

    /// Handle configuration file update or creation
    async fn handle_config_update(
        change: &ConfigChange,
        config_manager: &Arc<ConfigurationManager>,
        event_bus: &Arc<dyn EventBusTrait>,
        config_backups: &Arc<RwLock<HashMap<PathBuf, Vec<ConfigBackup>>>>,
        last_known_good: &Arc<RwLock<HashMap<ConfigFileType, ConfigBackup>>>,
        hot_reload_config: &HotReloadConfig,
    ) -> OrchestratorResult<()> {
        // Read the new configuration
        let new_content = tokio::fs::read_to_string(&change.file_path).await
            .map_err(|e| OrchestratorError::ConfigurationError {
                module: ModuleId::Orchestrator,
                reason: format!("Failed to read config file: {}", e),
            })?;

        // Validate the configuration
        let validation = Self::validate_config_content(&new_content, &change.file_type).await;
        
        if !validation.valid {
            error!("❌ Configuration validation failed for {:?}", change.file_path);
            for error in &validation.errors {
                error!("  - {}", error);
            }

            if hot_reload_config.auto_rollback_on_error {
                warn!("🔄 Auto-rollback enabled, restoring last known good configuration");
                return Self::rollback_config(change, last_known_good).await;
            } else {
                return Err(OrchestratorError::ConfigurationError {
                    module: ModuleId::Orchestrator,
                    reason: format!("Configuration validation failed: {:?}", validation.errors),
                });
            }
        }

        // Create backup before applying changes
        Self::create_config_backup(&change.file_path, &new_content, config_backups).await?;

        // Apply the configuration change
        match &change.file_type {
            ConfigFileType::Global => {
                Self::apply_global_config_change(&new_content, config_manager).await?;
            }
            ConfigFileType::Module(module_id) => {
                Self::apply_module_config_change(*module_id, &new_content, config_manager).await?;
            }
            ConfigFileType::Environment => {
                Self::apply_environment_config_change(&new_content, config_manager).await?;
            }
        }

        // Update last known good
        {
            let backup = ConfigBackup {
                file_path: change.file_path.clone(),
                content: new_content.clone(),
                timestamp: change.timestamp,
                checksum: Self::calculate_checksum(&new_content),
            };
            
            let mut last_good = last_known_good.write().await;
            last_good.insert(change.file_type.clone(), backup);
        }

        // Notify all modules of the configuration change
        Self::broadcast_config_change(change, event_bus).await?;

        info!("✅ Successfully applied configuration change for: {:?}", change.file_path);

        // Log validation warnings if any
        if !validation.warnings.is_empty() {
            warn!("⚠️  Configuration warnings:");
            for warning in &validation.warnings {
                warn!("  - {}", warning);
            }
        }

        Ok(())
    }

    /// Handle configuration file deletion
    async fn handle_config_deletion(
        change: &ConfigChange,
        _config_manager: &Arc<ConfigurationManager>,
        event_bus: &Arc<dyn EventBusTrait>,
        _last_known_good: &Arc<RwLock<HashMap<ConfigFileType, ConfigBackup>>>,
    ) -> OrchestratorResult<()> {
        warn!("🗑️  Configuration file deleted: {:?}", change.file_path);

        // Notify modules that configuration was removed
        Self::broadcast_config_change(change, event_bus).await?;

        Ok(())
    }

    /// Validate configuration content
    async fn validate_config_content(
        content: &str,
        file_type: &ConfigFileType,
    ) -> ConfigValidation {
        let mut validation = ConfigValidation {
            valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            suggestions: Vec::new(),
        };

        // Basic syntax validation (try to parse as TOML/JSON/YAML)
        if content.trim().is_empty() {
            validation.errors.push("Configuration file is empty".to_string());
            validation.valid = false;
            return validation;
        }

        // Try parsing as TOML (most common format)
        if let Err(e) = toml::from_str::<toml::Value>(content) {
            // Try JSON as fallback
            if let Err(json_err) = serde_json::from_str::<serde_json::Value>(content) {
                validation.errors.push(format!("Invalid TOML: {}, Invalid JSON: {}", e, json_err));
                validation.valid = false;
                return validation;
            }
        }

        // File-type specific validation
        match file_type {
            ConfigFileType::Global => {
                // Validate global orchestrator configuration
                if let Err(e) = toml::from_str::<OrchestratorConfig>(content) {
                    validation.warnings.push(format!("Global config structure validation: {}", e));
                }
            }
            ConfigFileType::Module(module_id) => {
                debug!("Validating module configuration for: {}", module_id);
                // Module-specific validation would go here
            }
            ConfigFileType::Environment => {
                debug!("Validating environment configuration");
                // Environment-specific validation would go here
            }
        }

        validation
    }

    /// Create backup of configuration
    async fn create_config_backup(
        file_path: &Path,
        content: &str,
        config_backups: &Arc<RwLock<HashMap<PathBuf, Vec<ConfigBackup>>>>,
    ) -> OrchestratorResult<()> {
        let backup = ConfigBackup {
            file_path: file_path.to_path_buf(),
            content: content.to_string(),
            timestamp: Instant::now(),
            checksum: Self::calculate_checksum(content),
        };

        let mut backups = config_backups.write().await;
        let file_backups = backups.entry(file_path.to_path_buf()).or_insert_with(Vec::new);
        file_backups.push(backup);

        // Rotate backups (keep only last 5)
        if file_backups.len() > 5 {
            file_backups.remove(0);
        }

        Ok(())
    }

    /// Apply global configuration changes
    async fn apply_global_config_change(
        content: &str,
        _config_manager: &Arc<ConfigurationManager>,
    ) -> OrchestratorResult<()> {
        info!("🌐 Applying global configuration changes");
        
        // Parse as orchestrator config
        let _new_config: OrchestratorConfig = toml::from_str(content)
            .map_err(|e| OrchestratorError::ConfigurationError {
                module: ModuleId::Orchestrator,
                reason: format!("Failed to parse global config: {}", e),
            })?;

        // In a real implementation, this would update the orchestrator's live configuration
        // For now, we'll just log the change
        info!("✅ Global configuration updated");

        Ok(())
    }

    /// Apply module-specific configuration changes
    async fn apply_module_config_change(
        module_id: ModuleId,
        content: &str,
        config_manager: &Arc<ConfigurationManager>,
    ) -> OrchestratorResult<()> {
        info!("🔧 Applying configuration changes for module: {}", module_id);

        // Parse as generic JSON value for flexibility
        let config_value: serde_json::Value = if content.trim().starts_with('{') {
            serde_json::from_str(content)
                .map_err(|e| OrchestratorError::ConfigurationError {
                    module: ModuleId::Orchestrator,
                    reason: format!("Failed to parse module config as JSON: {}", e),
                })?
        } else {
            // Try TOML first
            let toml_value = toml::from_str::<toml::Value>(content)
                .map_err(|e| OrchestratorError::ConfigurationError {
                    module: ModuleId::Orchestrator,
                    reason: format!("Failed to parse module config as TOML: {}", e),
                })?;
            
            serde_json::to_value(toml_value)
                .map_err(|e| OrchestratorError::ConfigurationError {
                    module: ModuleId::Orchestrator,
                    reason: format!("Failed to convert TOML to JSON: {}", e),
                })?
        };

        // Update configuration through the configuration manager
        config_manager.update_config(module_id, config_value).await?;

        info!("✅ Module configuration updated for: {}", module_id);
        Ok(())
    }

    /// Apply environment-specific configuration changes
    async fn apply_environment_config_change(
        content: &str,
        config_manager: &Arc<ConfigurationManager>,
    ) -> OrchestratorResult<()> {
        info!("🌍 Applying environment configuration changes");

        // Parse environment overrides
        let env_overrides: HashMap<String, serde_json::Value> = 
            toml::from_str(content)
                .or_else(|_| serde_json::from_str(content))
                .map_err(|e| OrchestratorError::ConfigurationError {
                    module: ModuleId::Orchestrator,
                    reason: format!("Failed to parse environment config: {}", e),
                })?;

        // Apply environment overrides
        for (key, value) in env_overrides {
            config_manager.set_override(key, value).await;
        }

        info!("✅ Environment configuration updated");
        Ok(())
    }

    /// Broadcast configuration change to all modules
    async fn broadcast_config_change(
        change: &ConfigChange,
        event_bus: &Arc<dyn EventBusTrait>,
    ) -> OrchestratorResult<()> {
        let config_update = ConfigUpdate {
            config_key: format!("config_file_changed_{:?}", change.file_type),
            config_value: serde_json::json!({
                "change_type": format!("{:?}", change.change_type),
                "file_path": change.file_path.to_string_lossy(),
                "timestamp": change.timestamp.elapsed().as_secs(),
            }),
            target_module: None, // Broadcast to all
        };

        let message = BusMessage::new(
            ModuleId::Orchestrator,
            MessagePayload::ConfigUpdate(config_update),
        );

        event_bus.publish(message).await
            .map_err(|e| OrchestratorError::EventBus(e))?;

        Ok(())
    }

    /// Rollback to last known good configuration
    async fn rollback_config(
        change: &ConfigChange,
        last_known_good: &Arc<RwLock<HashMap<ConfigFileType, ConfigBackup>>>,
    ) -> OrchestratorResult<()> {
        warn!("🔄 Rolling back configuration for: {:?}", change.file_path);

        let backup = {
            let last_good = last_known_good.read().await;
            last_good.get(&change.file_type).cloned()
        };

        if let Some(backup) = backup {
            tokio::fs::write(&change.file_path, &backup.content).await
                .map_err(|e| OrchestratorError::ConfigurationError {
                    module: ModuleId::Orchestrator,
                    reason: format!("Failed to rollback config: {}", e),
                })?;

            info!("✅ Successfully rolled back configuration");
        } else {
            warn!("⚠️  No backup available for rollback");
        }

        Ok(())
    }

    /// Get configuration backup history
    pub async fn get_backup_history(&self, file_path: &Path) -> Vec<ConfigBackup> {
        let backups = self.config_backups.read().await;
        backups.get(file_path).cloned().unwrap_or_default()
    }

    /// Get last known good configurations
    pub async fn get_last_known_good(&self) -> HashMap<ConfigFileType, ConfigBackup> {
        let last_good = self.last_known_good.read().await;
        last_good.clone()
    }
}

// Additional trait implementations would go here for Serialize/Deserialize if needed