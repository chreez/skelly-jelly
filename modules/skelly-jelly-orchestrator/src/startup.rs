//! Advanced startup sequencing with dependency-ordered initialization

use crate::{
    error::{OrchestratorError, OrchestratorResult},
    lifecycle::LifecycleController,
    health::HealthMonitor,
    module_registry::ModuleRegistry,
    config::ConfigurationManager,
};
use skelly_jelly_event_bus::{EventBusTrait, ModuleId, BusMessage, MessagePayload};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::time::timeout;
use tracing::{info, warn, error, debug};

/// Startup phase tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StartupPhase {
    Initializing,
    PreparingDependencies,
    StartingCore,
    StartingServices,
    StartingUI,
    ValidatingSystem,
    Ready,
    Failed(StartupError),
}

/// Startup error types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StartupError {
    DependencyTimeout,
    HealthCheckFailed,
    CriticalServiceFailed,
    ValidationFailed,
    SystemTimeout,
}

/// Startup metrics and performance tracking
#[derive(Debug, Clone)]
pub struct StartupMetrics {
    pub total_duration: Duration,
    pub phase_durations: HashMap<StartupPhase, Duration>,
    pub module_startup_times: HashMap<ModuleId, Duration>,
    pub dependency_resolution_time: Duration,
    pub health_validation_time: Duration,
    pub target_met: bool,
    pub bottlenecks: Vec<StartupBottleneck>,
}

/// Identified startup bottlenecks
#[derive(Debug, Clone)]
pub struct StartupBottleneck {
    pub module: ModuleId,
    pub duration: Duration,
    pub reason: String,
    pub impact: BottleneckImpact,
}

#[derive(Debug, Clone, Copy)]
pub enum BottleneckImpact {
    Low,
    Medium,
    High,
    Critical,
}

/// Module startup group for parallel initialization
#[derive(Debug, Clone)]
pub struct StartupGroup {
    pub phase: StartupPhase,
    pub modules: Vec<ModuleId>,
    pub dependencies_satisfied: bool,
    pub max_parallel: usize,
    pub timeout: Duration,
}

/// Advanced startup sequencer with performance optimization
pub struct StartupSequencer {
    registry: Arc<ModuleRegistry>,
    lifecycle_controller: Arc<LifecycleController>,
    health_monitor: Arc<tokio::sync::RwLock<HealthMonitor>>,
    config_manager: Arc<ConfigurationManager>,
    event_bus: Arc<dyn EventBusTrait>,
    
    /// Performance targets
    total_startup_target: Duration,
    health_check_target: Duration,
    
    /// Current state
    current_phase: StartupPhase,
    startup_start_time: Option<Instant>,
    phase_start_times: HashMap<StartupPhase, Instant>,
    
    /// Metrics collection
    metrics: StartupMetrics,
}

impl StartupSequencer {
    pub fn new(
        registry: Arc<ModuleRegistry>,
        lifecycle_controller: Arc<LifecycleController>,
        health_monitor: Arc<tokio::sync::RwLock<HealthMonitor>>,
        config_manager: Arc<ConfigurationManager>,
        event_bus: Arc<dyn EventBusTrait>,
    ) -> Self {
        Self {
            registry,
            lifecycle_controller,
            health_monitor,
            config_manager,
            event_bus,
            total_startup_target: Duration::from_secs(10), // Target: <10 seconds
            health_check_target: Duration::from_secs(2),
            current_phase: StartupPhase::Initializing,
            startup_start_time: None,
            phase_start_times: HashMap::new(),
            metrics: StartupMetrics {
                total_duration: Duration::ZERO,
                phase_durations: HashMap::new(),
                module_startup_times: HashMap::new(),
                dependency_resolution_time: Duration::ZERO,
                health_validation_time: Duration::ZERO,
                target_met: false,
                bottlenecks: Vec::new(),
            },
        }
    }

    /// Execute coordinated system startup with performance monitoring
    pub async fn startup_system(&mut self) -> OrchestratorResult<StartupMetrics> {
        info!("🚀 Starting coordinated system startup sequence");
        
        let startup_start = Instant::now();
        self.startup_start_time = Some(startup_start);
        self.record_phase_start(StartupPhase::Initializing);

        // Phase 1: Initialize dependency graph and validate
        self.advance_phase(StartupPhase::PreparingDependencies).await?;
        let startup_order = self.compute_optimized_startup_order().await?;
        info!("📋 Computed startup order: {:?}", startup_order);

        // Phase 2: Start core infrastructure (EventBus first)
        self.advance_phase(StartupPhase::StartingCore).await?;
        self.start_core_modules(&startup_order).await?;

        // Phase 3: Start service modules with parallelization
        self.advance_phase(StartupPhase::StartingServices).await?;
        self.start_service_modules_parallel(&startup_order).await?;

        // Phase 4: Start UI modules
        self.advance_phase(StartupPhase::StartingUI).await?;
        self.start_ui_modules(&startup_order).await?;

        // Phase 5: System validation and health checks
        self.advance_phase(StartupPhase::ValidatingSystem).await?;
        self.validate_system_health().await?;

        // Phase 6: Mark system as ready
        self.advance_phase(StartupPhase::Ready).await?;
        
        // Calculate final metrics
        self.finalize_metrics(startup_start);
        
        let total_time = startup_start.elapsed();
        if total_time <= self.total_startup_target {
            info!("✅ System startup completed successfully in {:?} (target: {:?})", 
                  total_time, self.total_startup_target);
            self.metrics.target_met = true;
        } else {
            warn!("⚠️  System startup took {:?}, exceeding target of {:?}", 
                  total_time, self.total_startup_target);
            self.analyze_startup_bottlenecks();
        }

        // Publish startup completion event
        self.publish_startup_complete().await?;

        Ok(self.metrics.clone())
    }

    /// Compute optimized startup order with parallel groups
    async fn compute_optimized_startup_order(&mut self) -> OrchestratorResult<Vec<StartupGroup>> {
        let dependency_start = Instant::now();
        
        // Get basic dependency order
        let _basic_order = self.registry.compute_startup_order().await?;
        
        // Group modules by startup phase and dependencies
        let groups = vec![
            StartupGroup {
                phase: StartupPhase::StartingCore,
                modules: vec![ModuleId::EventBus, ModuleId::Orchestrator],
                dependencies_satisfied: true,
                max_parallel: 1, // Core modules start sequentially
                timeout: Duration::from_secs(5),
            },
            StartupGroup {
                phase: StartupPhase::StartingServices,
                modules: vec![
                    ModuleId::Storage,
                    ModuleId::DataCapture,
                    ModuleId::AnalysisEngine,
                ],
                dependencies_satisfied: false,
                max_parallel: 2, // Can start some in parallel
                timeout: Duration::from_secs(15),
            },
            StartupGroup {
                phase: StartupPhase::StartingUI,
                modules: vec![
                    ModuleId::Gamification,
                    ModuleId::AiIntegration,
                    ModuleId::CuteFigurine,
                ],
                dependencies_satisfied: false,
                max_parallel: 3, // UI modules can start in parallel
                timeout: Duration::from_secs(10),
            },
        ];

        self.metrics.dependency_resolution_time = dependency_start.elapsed();
        debug!("🔗 Dependency resolution completed in {:?}", self.metrics.dependency_resolution_time);
        
        Ok(groups)
    }

    /// Start core infrastructure modules sequentially
    async fn start_core_modules(&mut self, groups: &[StartupGroup]) -> OrchestratorResult<()> {
        let core_group = groups.iter()
            .find(|g| matches!(g.phase, StartupPhase::StartingCore))
            .ok_or_else(|| OrchestratorError::ConfigurationError {
                module: ModuleId::Orchestrator,
                reason: "Core startup group not found".to_string(),
            })?;

        info!("🏗️  Starting core modules: {:?}", core_group.modules);

        for &module_id in &core_group.modules {
            if module_id == ModuleId::Orchestrator {
                continue; // Already running
            }

            let module_start = Instant::now();
            
            // Start module with timeout
            match timeout(core_group.timeout, self.lifecycle_controller.start_module(module_id)).await {
                Ok(Ok(())) => {
                    let duration = module_start.elapsed();
                    self.metrics.module_startup_times.insert(module_id, duration);
                    info!("✅ Core module {} started in {:?}", module_id, duration);
                    
                    // Brief delay between core modules for stability
                    tokio::time::sleep(Duration::from_millis(500)).await;
                }
                Ok(Err(e)) => {
                    error!("❌ Failed to start core module {}: {}", module_id, e);
                    return Err(e);
                }
                Err(_) => {
                    let error = OrchestratorError::ModuleStartupFailed {
                        module: module_id,
                        reason: format!("Timeout after {:?}", core_group.timeout),
                    };
                    error!("❌ Core module {} startup timeout", module_id);
                    return Err(error);
                }
            }
        }

        Ok(())
    }

    /// Start service modules with controlled parallelization
    async fn start_service_modules_parallel(&mut self, groups: &[StartupGroup]) -> OrchestratorResult<()> {
        let service_group = groups.iter()
            .find(|g| matches!(g.phase, StartupPhase::StartingServices))
            .ok_or_else(|| OrchestratorError::ConfigurationError {
                module: ModuleId::Orchestrator,
                reason: "Service startup group not found".to_string(),
            })?;

        info!("⚙️  Starting service modules with parallelization: {:?}", service_group.modules);

        // Start modules in parallel batches
        let mut remaining_modules = service_group.modules.clone();
        
        while !remaining_modules.is_empty() {
            // Take up to max_parallel modules that have dependencies satisfied
            let mut current_batch = Vec::new();
            let mut i = 0;
            
            while i < remaining_modules.len() && current_batch.len() < service_group.max_parallel {
                let module_id = remaining_modules[i];
                
                // Check if dependencies are satisfied
                if self.are_dependencies_satisfied(module_id).await {
                    current_batch.push(module_id);
                    remaining_modules.remove(i);
                } else {
                    i += 1;
                }
            }

            if current_batch.is_empty() {
                // Dependency deadlock - try to start one module anyway
                if let Some(module_id) = remaining_modules.first() {
                    warn!("⚠️  Potential dependency deadlock, force-starting {}", module_id);
                    current_batch.push(*module_id);
                    remaining_modules.remove(0);
                }
            }

            // Start current batch in parallel
            if !current_batch.is_empty() {
                self.start_module_batch(current_batch, service_group.timeout).await?;
                
                // Brief pause between batches
                tokio::time::sleep(Duration::from_millis(200)).await;
            }
        }

        Ok(())
    }

    /// Start UI modules in parallel
    async fn start_ui_modules(&mut self, groups: &[StartupGroup]) -> OrchestratorResult<()> {
        let ui_group = groups.iter()
            .find(|g| matches!(g.phase, StartupPhase::StartingUI))
            .ok_or_else(|| OrchestratorError::ConfigurationError {
                module: ModuleId::Orchestrator,
                reason: "UI startup group not found".to_string(),
            })?;

        info!("🎨 Starting UI modules: {:?}", ui_group.modules);

        // UI modules can typically start in parallel
        self.start_module_batch(ui_group.modules.clone(), ui_group.timeout).await?;

        Ok(())
    }

    /// Start a batch of modules in parallel
    async fn start_module_batch(&mut self, modules: Vec<ModuleId>, timeout_duration: Duration) -> OrchestratorResult<()> {
        let batch_start = Instant::now();
        
        // Create futures for all modules in the batch
        let mut tasks = Vec::new();
        
        for module_id in &modules {
            let lifecycle_controller = Arc::clone(&self.lifecycle_controller);
            let module_id = *module_id;
            
            let task = tokio::spawn(async move {
                let start_time = Instant::now();
                let result = lifecycle_controller.start_module(module_id).await;
                (module_id, result, start_time.elapsed())
            });
            
            tasks.push(task);
        }

        // Wait for all tasks to complete or timeout
        let results = match timeout(timeout_duration, futures::future::join_all(tasks)).await {
            Ok(results) => results,
            Err(_) => {
                error!("❌ Module batch startup timeout after {:?}", timeout_duration);
                return Err(OrchestratorError::ModuleStartupFailed {
                    module: ModuleId::Orchestrator,
                    reason: format!("Batch timeout after {:?}", timeout_duration),
                });
            }
        };

        // Process results
        for result in results {
            match result {
                Ok((module_id, Ok(()), duration)) => {
                    self.metrics.module_startup_times.insert(module_id, duration);
                    info!("✅ Module {} started in {:?}", module_id, duration);
                }
                Ok((module_id, Err(e), duration)) => {
                    error!("❌ Module {} failed to start after {:?}: {}", module_id, duration, e);
                    
                    // Record as bottleneck
                    self.metrics.bottlenecks.push(StartupBottleneck {
                        module: module_id,
                        duration,
                        reason: e.to_string(),
                        impact: BottleneckImpact::High,
                    });
                    
                    return Err(e);
                }
                Err(e) => {
                    error!("❌ Task execution error: {}", e);
                    return Err(OrchestratorError::ModuleStartupFailed {
                        module: ModuleId::Orchestrator,
                        reason: format!("Task execution error: {}", e),
                    });
                }
            }
        }

        debug!("📦 Batch of {} modules started in {:?}", modules.len(), batch_start.elapsed());
        Ok(())
    }

    /// Check if all dependencies for a module are satisfied
    async fn are_dependencies_satisfied(&self, module_id: ModuleId) -> bool {
        let dependencies = self.registry.get_dependencies(module_id).await;
        
        for dependency in dependencies {
            if let Some(state) = self.registry.get_module_state(dependency) {
                if !matches!(state, crate::lifecycle::ModuleState::Running { .. }) {
                    return false;
                }
            } else {
                return false;
            }
        }
        
        true
    }

    /// Validate system health after startup
    async fn validate_system_health(&mut self) -> OrchestratorResult<()> {
        info!("🔍 Validating system health");
        let health_start = Instant::now();

        // Wait a moment for modules to fully initialize
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Get health status from all modules
        let health_monitor = self.health_monitor.read().await;
        let health_reports = health_monitor.get_all_health_reports();
        
        let mut unhealthy_modules = Vec::new();
        let mut degraded_modules = Vec::new();

        for report in &health_reports {
            match &report.status {
                crate::health::HealthStatus::Unhealthy { reason } => {
                    unhealthy_modules.push((report.module_id, reason.clone()));
                }
                crate::health::HealthStatus::Degraded { issues } => {
                    degraded_modules.push((report.module_id, issues.clone()));
                }
                _ => {}
            }
        }

        self.metrics.health_validation_time = health_start.elapsed();

        if !unhealthy_modules.is_empty() {
            error!("❌ System validation failed - unhealthy modules: {:?}", unhealthy_modules);
            return Err(OrchestratorError::HealthCheckFailed {
                module: ModuleId::Orchestrator,
                reason: format!("Unhealthy modules: {:?}", unhealthy_modules),
            });
        }

        if !degraded_modules.is_empty() {
            warn!("⚠️  System has degraded modules: {:?}", degraded_modules);
        }

        let health_duration = self.metrics.health_validation_time;
        if health_duration <= self.health_check_target {
            info!("✅ System health validation completed in {:?} (target: {:?})", 
                  health_duration, self.health_check_target);
        } else {
            warn!("⚠️  Health validation took {:?}, exceeding target of {:?}", 
                  health_duration, self.health_check_target);
        }

        Ok(())
    }

    /// Advance to the next startup phase
    async fn advance_phase(&mut self, new_phase: StartupPhase) -> OrchestratorResult<()> {
        // Record duration of previous phase
        if let Some(start_time) = self.phase_start_times.get(&self.current_phase) {
            let duration = start_time.elapsed();
            self.metrics.phase_durations.insert(self.current_phase, duration);
            debug!("📊 Phase {:?} completed in {:?}", self.current_phase, duration);
        }

        // Advance to new phase
        self.current_phase = new_phase;
        self.record_phase_start(new_phase);
        
        info!("🔄 Advanced to startup phase: {:?}", new_phase);
        Ok(())
    }

    fn record_phase_start(&mut self, phase: StartupPhase) {
        self.phase_start_times.insert(phase, Instant::now());
    }

    /// Analyze startup bottlenecks and suggest improvements
    fn analyze_startup_bottlenecks(&mut self) {
        info!("🔍 Analyzing startup bottlenecks");

        // Find modules that took longer than expected
        for (module_id, duration) in &self.metrics.module_startup_times {
            let expected_duration = match module_id {
                ModuleId::EventBus => Duration::from_secs(2),
                ModuleId::Storage => Duration::from_secs(3),
                ModuleId::DataCapture => Duration::from_secs(2),
                ModuleId::AnalysisEngine => Duration::from_secs(4),
                ModuleId::Gamification => Duration::from_secs(2),
                ModuleId::AiIntegration => Duration::from_secs(3),
                ModuleId::CuteFigurine => Duration::from_secs(2),
                ModuleId::Orchestrator => Duration::from_secs(1),
            };

            if *duration > expected_duration * 2 {
                let impact = if *duration > expected_duration * 4 {
                    BottleneckImpact::Critical
                } else if *duration > expected_duration * 3 {
                    BottleneckImpact::High
                } else {
                    BottleneckImpact::Medium
                };

                self.metrics.bottlenecks.push(StartupBottleneck {
                    module: *module_id,
                    duration: *duration,
                    reason: format!("Exceeded expected startup time of {:?}", expected_duration),
                    impact,
                });
            }
        }

        // Log bottleneck analysis
        if !self.metrics.bottlenecks.is_empty() {
            warn!("⚠️  Identified {} startup bottlenecks:", self.metrics.bottlenecks.len());
            for bottleneck in &self.metrics.bottlenecks {
                warn!("  - {}: {:?} ({:?}) - {}", 
                      bottleneck.module, 
                      bottleneck.duration, 
                      bottleneck.impact,
                      bottleneck.reason);
            }
        }
    }

    /// Finalize startup metrics calculation
    fn finalize_metrics(&mut self, startup_start: Instant) {
        self.metrics.total_duration = startup_start.elapsed();
        
        // Record final phase duration
        if let Some(start_time) = self.phase_start_times.get(&self.current_phase) {
            let duration = start_time.elapsed();
            self.metrics.phase_durations.insert(self.current_phase, duration);
        }
    }

    /// Publish startup completion event
    async fn publish_startup_complete(&self) -> OrchestratorResult<()> {
        let completion_event = serde_json::json!({
            "event_type": "system_startup_complete",
            "total_duration_ms": self.metrics.total_duration.as_millis(),
            "target_met": self.metrics.target_met,
            "modules_started": self.metrics.module_startup_times.len(),
            "bottlenecks_count": self.metrics.bottlenecks.len(),
        });

        let config_update = skelly_jelly_event_bus::message::ConfigUpdate {
            config_key: "system_startup_complete".to_string(),
            config_value: completion_event,
            target_module: None, // Broadcast to all modules
        };

        let message = BusMessage::new(
            ModuleId::Orchestrator,
            MessagePayload::ConfigUpdate(config_update),
        );

        self.event_bus.publish(message).await
            .map_err(|e| OrchestratorError::EventBus(e))?;

        Ok(())
    }

    /// Get current startup metrics
    pub fn get_metrics(&self) -> &StartupMetrics {
        &self.metrics
    }

    /// Get current startup phase
    pub fn get_current_phase(&self) -> StartupPhase {
        self.current_phase
    }
}