//! Module lifecycle management

use crate::config::ConfigurationManager;
use crate::error::{OrchestratorError, OrchestratorResult};
use crate::module_registry::ModuleRegistry;
use skelly_jelly_event_bus::{EventBusTrait, ModuleId, BusMessage, MessagePayload};
use serde::{Deserialize, Serialize};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::time::timeout;
use tracing::{info, warn, error, debug};

/// State of a module in the system
#[derive(Debug, Clone)]
pub enum ModuleState {
    NotStarted,
    Starting { since: Instant },
    Running { since: Instant },
    Stopping { since: Instant },
    Stopped { reason: StopReason },
    Failed { error: String, attempts: u32 },
}

/// Reason for module stop
#[derive(Debug, Clone)]
pub enum StopReason {
    Requested,
    Shutdown,
    Error(String),
    Dependency(ModuleId),
}

/// Lifecycle controller manages module startup and shutdown
pub struct LifecycleController {
    registry: Arc<ModuleRegistry>,
    event_bus: Arc<dyn EventBusTrait>,
    config_manager: Arc<ConfigurationManager>,
}

impl LifecycleController {
    pub fn new(
        registry: Arc<ModuleRegistry>,
        event_bus: Arc<dyn EventBusTrait>,
        config_manager: Arc<ConfigurationManager>,
    ) -> Self {
        Self {
            registry,
            event_bus,
            config_manager,
        }
    }

    /// Start the entire system
    pub async fn start_system(&self) -> OrchestratorResult<()> {
        info!("Starting system...");

        // Get global configuration
        let global_config = self.config_manager.get_global_config().await;

        // Compute startup order from dependency graph
        let startup_order = self.registry.compute_startup_order().await?;
        info!("Computed startup order: {:?}", startup_order);

        // Start modules in dependency order
        for module_id in startup_order {
            if module_id == ModuleId::Orchestrator {
                // Orchestrator is already running
                continue;
            }

            self.start_module(module_id).await?;
            
            // Wait between module starts if configured
            if global_config.module_start_delay > Duration::ZERO {
                tokio::time::sleep(global_config.module_start_delay).await;
            }
        }

        info!("System startup completed successfully");
        Ok(())
    }

    /// Stop the entire system gracefully
    pub async fn stop_system(&self, timeout_duration: Duration) -> OrchestratorResult<()> {
        info!("Stopping system with timeout: {:?}", timeout_duration);

        // Get all modules in reverse dependency order
        let startup_order = self.registry.compute_startup_order().await?;
        let mut shutdown_order = startup_order;
        shutdown_order.reverse();

        // Stop modules in reverse order
        for module_id in shutdown_order {
            if module_id == ModuleId::Orchestrator {
                // Orchestrator stops last
                continue;
            }

            if let Err(e) = self.stop_module(module_id, timeout_duration).await {
                warn!("Failed to stop module {}: {}", module_id, e);
                // Continue stopping other modules
            }
        }

        info!("System shutdown completed");
        Ok(())
    }

    /// Start a specific module
    pub async fn start_module(&self, module_id: ModuleId) -> OrchestratorResult<()> {
        info!("Starting module: {}", module_id);

        // Check if module is already running
        if let Some(state) = self.registry.get_module_state(module_id) {
            match state {
                ModuleState::Running { .. } => {
                    info!("Module {} is already running", module_id);
                    return Ok(());
                }
                ModuleState::Starting { .. } => {
                    info!("Module {} is already starting", module_id);
                    return Ok(());
                }
                _ => {}
            }
        }

        // Check dependencies are running
        let dependencies = self.registry.get_dependencies(module_id).await;
        for dependency in dependencies {
            if let Some(state) = self.registry.get_module_state(dependency) {
                match state {
                    ModuleState::Running { .. } => continue,
                    _ => {
                        return Err(OrchestratorError::MissingDependency {
                            module: module_id,
                            dependency,
                        });
                    }
                }
            }
        }

        // Set module state to starting
        self.registry.set_module_state(
            module_id,
            ModuleState::Starting { since: Instant::now() },
        );

        // Get module descriptor for timeout
        let descriptor = self.registry.get_module(module_id)
            .ok_or_else(|| OrchestratorError::ConfigurationError {
                module: module_id,
                reason: "Module not registered".to_string(),
            })?;

        // Start the module based on its type
        let result = timeout(
            descriptor.startup_timeout,
            self.start_module_impl(module_id),
        ).await;

        match result {
            Ok(Ok(())) => {
                self.registry.set_module_state(
                    module_id,
                    ModuleState::Running { since: Instant::now() },
                );
                
                // Publish module ready event
                let message = BusMessage::new(
                    ModuleId::Orchestrator,
                    MessagePayload::ModuleReady(module_id),
                );
                let _ = self.event_bus.publish(message).await;

                info!("Module {} started successfully", module_id);
                Ok(())
            }
            Ok(Err(e)) => {
                self.registry.set_module_state(
                    module_id,
                    ModuleState::Failed {
                        error: e.to_string(),
                        attempts: 1,
                    },
                );
                Err(e)
            }
            Err(_) => {
                let error = OrchestratorError::ModuleStartupFailed {
                    module: module_id,
                    reason: "Startup timeout".to_string(),
                };
                self.registry.set_module_state(
                    module_id,
                    ModuleState::Failed {
                        error: error.to_string(),
                        attempts: 1,
                    },
                );
                Err(error)
            }
        }
    }

    /// Stop a specific module
    pub async fn stop_module(&self, module_id: ModuleId, timeout_duration: Duration) -> OrchestratorResult<()> {
        info!("Stopping module: {}", module_id);

        // Check if module is already stopped
        if let Some(state) = self.registry.get_module_state(module_id) {
            match state {
                ModuleState::Stopped { .. } | ModuleState::NotStarted => {
                    info!("Module {} is already stopped", module_id);
                    return Ok(());
                }
                ModuleState::Stopping { .. } => {
                    info!("Module {} is already stopping", module_id);
                    return Ok(());
                }
                _ => {}
            }
        }

        // Set module state to stopping
        self.registry.set_module_state(
            module_id,
            ModuleState::Stopping { since: Instant::now() },
        );

        // Stop dependent modules first
        let dependents = self.registry.get_dependents(module_id).await;
        for dependent in dependents {
            if let Some(state) = self.registry.get_module_state(dependent) {
                match state {
                    ModuleState::Running { .. } | ModuleState::Starting { .. } => {
                        warn!("Stopping dependent module {} before {}", dependent, module_id);
                        let _ = Box::pin(self.stop_module(dependent, timeout_duration)).await;
                    }
                    _ => {}
                }
            }
        }

        // Stop the module
        let result = timeout(timeout_duration, self.stop_module_impl(module_id)).await;

        match result {
            Ok(Ok(())) => {
                self.registry.set_module_state(
                    module_id,
                    ModuleState::Stopped { reason: StopReason::Requested },
                );
                info!("Module {} stopped successfully", module_id);
                Ok(())
            }
            Ok(Err(e)) => {
                self.registry.set_module_state(
                    module_id,
                    ModuleState::Failed {
                        error: e.to_string(),
                        attempts: 1,
                    },
                );
                Err(e)
            }
            Err(_) => {
                let error = OrchestratorError::ModuleShutdownFailed {
                    module: module_id,
                    reason: "Shutdown timeout".to_string(),
                };
                self.registry.set_module_state(
                    module_id,
                    ModuleState::Failed {
                        error: error.to_string(),
                        attempts: 1,
                    },
                );
                Err(error)
            }
        }
    }

    /// Restart a specific module
    pub async fn restart_module(&self, module_id: ModuleId) -> OrchestratorResult<()> {
        info!("Restarting module: {}", module_id);

        // Get timeout from descriptor
        let descriptor = self.registry.get_module(module_id)
            .ok_or_else(|| OrchestratorError::ConfigurationError {
                module: module_id,
                reason: "Module not registered".to_string(),
            })?;

        // Stop the module first
        self.stop_module(module_id, descriptor.shutdown_timeout).await?;

        // Wait a moment before restart
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Start the module
        self.start_module(module_id).await?;

        info!("Module {} restarted successfully", module_id);
        Ok(())
    }

    /// Implementation of module starting (placeholder)
    async fn start_module_impl(&self, module_id: ModuleId) -> OrchestratorResult<()> {
        match module_id {
            ModuleId::EventBus => {
                // Event bus should already be running since we need it for communication
                debug!("Event bus module start - assuming already running");
                Ok(())
            }
            ModuleId::Storage => {
                // Load storage config and start storage module
                debug!("Starting storage module");
                // In a real implementation, this would spawn the storage module task
                self.simulate_module_start(module_id).await
            }
            ModuleId::DataCapture => {
                debug!("Starting data capture module");
                self.simulate_module_start(module_id).await
            }
            ModuleId::AnalysisEngine => {
                debug!("Starting analysis engine module");
                self.simulate_module_start(module_id).await
            }
            ModuleId::Gamification => {
                debug!("Starting gamification module");
                self.simulate_module_start(module_id).await
            }
            ModuleId::AiIntegration => {
                debug!("Starting AI integration module");
                self.simulate_module_start(module_id).await
            }
            ModuleId::CuteFigurine => {
                debug!("Starting cute figurine module");
                self.simulate_module_start(module_id).await
            }
            ModuleId::Orchestrator => {
                // Orchestrator is already running
                Ok(())
            }
        }
    }

    /// Implementation of module stopping (placeholder)
    async fn stop_module_impl(&self, module_id: ModuleId) -> OrchestratorResult<()> {
        debug!("Stopping module: {}", module_id);
        
        // Send shutdown message
        let shutdown_request = skelly_jelly_event_bus::message::ShutdownRequest {
            module_id,
            timeout: Duration::from_secs(10),
            save_state: true,
        };
        
        let message = BusMessage::new(
            ModuleId::Orchestrator,
            MessagePayload::Shutdown(shutdown_request),
        );
        
        // In a real implementation, this would be sent specifically to the target module
        let _ = self.event_bus.publish(message).await;

        // Get module handle and stop it
        if let Some(mut handle) = self.registry.get_module_handle_mut(module_id) {
            handle.stop().await;
        }

        // Simulate stop time
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        Ok(())
    }

    /// Simulate module start for demonstration (placeholder)
    async fn simulate_module_start(&self, module_id: ModuleId) -> OrchestratorResult<()> {
        debug!("Simulating start for module: {}", module_id);
        
        // Simulate startup time
        tokio::time::sleep(Duration::from_millis(500)).await;
        
        // In a real implementation, this would:
        // 1. Load module configuration
        // 2. Spawn the module's main task
        // 3. Store the task handle
        // 4. Wait for module to signal ready
        
        Ok(())
    }
}