//! Main orchestrator implementation

use crate::{
    config::{ConfigurationManager, OrchestratorConfig},
    error::{OrchestratorError, OrchestratorResult},
    health::{HealthMonitor, HealthReport, HealthStatus},
    lifecycle::{LifecycleController, ModuleState},
    module_registry::{ModuleRegistry, ModuleDescriptor},
    recovery::{RecoveryManager, ModuleFailure, FailureType},
    resource::{ResourceManager, SystemResources, PerformanceStats},
    startup::{StartupSequencer, StartupMetrics},
    performance_telemetry::{PerformanceTelemetrySystem, TelemetryConfig},
    event_loss_prevention::{EventLossPreventionSystem, EventLossPreventionConfig},
    OrchestratorTrait,
};
use async_trait::async_trait;
use skelly_jelly_event_bus::{EventBusTrait, ModuleId, BusMessage, MessagePayload, message::ErrorReport};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::RwLock;
use tracing::{info, warn, error, debug};
use uuid::Uuid;

/// System-wide health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SystemStatus {
    Healthy,
    Degraded { reason: String },
    Critical { failing_modules: Vec<ModuleId> },
    Starting,
    Stopping,
    Stopped,
}

/// Comprehensive system health information
#[derive(Debug, Clone)]
pub struct SystemHealth {
    pub status: SystemStatus,
    pub uptime: Duration,
    pub module_health: HashMap<ModuleId, HealthReport>,
    pub resource_usage: SystemResources,
    pub active_issues: Vec<SystemIssue>,
    pub last_updated: Instant,
}

/// System-wide issue
#[derive(Debug, Clone)]
pub struct SystemIssue {
    pub id: Uuid,
    pub severity: IssueSeverity,
    pub description: String,
    pub affected_modules: Vec<ModuleId>,
    pub timestamp: Instant,
    pub resolved: bool,
}

/// Issue severity levels
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum IssueSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Main orchestrator implementation
pub struct OrchestratorImpl {
    /// Configuration manager
    config_manager: Arc<ConfigurationManager>,
    
    /// Module registry
    registry: Arc<ModuleRegistry>,
    
    /// Lifecycle controller
    lifecycle_controller: Arc<LifecycleController>,
    
    /// Health monitor
    health_monitor: Arc<RwLock<HealthMonitor>>,
    
    /// Resource manager
    resource_manager: Arc<RwLock<ResourceManager>>,
    
    /// Recovery manager
    recovery_manager: Arc<RecoveryManager>,
    
    /// Event bus for communication
    event_bus: Arc<dyn EventBusTrait>,
    
    /// System startup time
    startup_time: Instant,
    
    /// Current system status
    system_status: Arc<RwLock<SystemStatus>>,
    
    /// Active system issues
    active_issues: Arc<RwLock<Vec<SystemIssue>>>,
    
    /// Advanced startup sequencer
    startup_sequencer: Arc<RwLock<Option<StartupSequencer>>>,
    
    /// Performance telemetry system
    telemetry_system: Arc<RwLock<PerformanceTelemetrySystem>>,
    
    /// Event loss prevention system
    loss_prevention_system: Arc<RwLock<EventLossPreventionSystem>>,
}

impl OrchestratorImpl {
    pub async fn new(
        config: OrchestratorConfig,
        event_bus: Arc<dyn EventBusTrait>,
    ) -> OrchestratorResult<Self> {
        info!("Initializing orchestrator");

        // Create core components
        let registry = Arc::new(ModuleRegistry::new());
        let config_manager = Arc::new(ConfigurationManager::new(config.clone(), Arc::clone(&event_bus)));
        
        let lifecycle_controller = Arc::new(LifecycleController::new(
            Arc::clone(&registry),
            Arc::clone(&event_bus),
            Arc::clone(&config_manager),
        ));
        
        let health_monitor = Arc::new(RwLock::new(HealthMonitor::new(
            Arc::clone(&registry),
            Arc::clone(&event_bus),
            config.health_check_interval,
            config.health_check_timeout,
            config.unhealthy_threshold,
        )));
        
        let resource_manager = Arc::new(RwLock::new(ResourceManager::new(
            Arc::clone(&registry),
            config.resource_check_interval,
            config.throttle_threshold,
        )));
        
        let recovery_manager = Arc::new(RecoveryManager::new(Arc::clone(&lifecycle_controller)));
        
        // Create performance telemetry system
        let telemetry_config = TelemetryConfig::default();
        let telemetry_system = Arc::new(RwLock::new(PerformanceTelemetrySystem::new(telemetry_config)));
        
        // Create event loss prevention system
        let loss_prevention_config = EventLossPreventionConfig::default();
        let loss_prevention_system = Arc::new(RwLock::new(EventLossPreventionSystem::new(loss_prevention_config)));

        let orchestrator = Self {
            config_manager,
            registry,
            lifecycle_controller,
            health_monitor,
            resource_manager,
            recovery_manager,
            event_bus,
            startup_time: Instant::now(),
            system_status: Arc::new(RwLock::new(SystemStatus::Starting)),
            active_issues: Arc::new(RwLock::new(Vec::new())),
            startup_sequencer: Arc::new(RwLock::new(None)),
            telemetry_system,
            loss_prevention_system,
        };

        // Subscribe to system events
        orchestrator.setup_event_subscriptions().await?;

        info!("Orchestrator initialized successfully");
        Ok(orchestrator)
    }

    /// Setup event subscriptions for system monitoring
    async fn setup_event_subscriptions(&self) -> OrchestratorResult<()> {
        use skelly_jelly_event_bus::{MessageFilter, DeliveryMode, MessageType};

        debug!("Setting up orchestrator event subscriptions");

        // Subscribe to error reports
        let error_filter = MessageFilter::types(vec![MessageType::Error]);
        
        self.event_bus.subscribe(
            ModuleId::Orchestrator,
            error_filter,
            DeliveryMode::Reliable { timeout: Duration::from_secs(5) },
        ).await?;

        // Subscribe to module ready events
        let ready_filter = MessageFilter::types(vec![MessageType::ModuleReady]);
        
        self.event_bus.subscribe(
            ModuleId::Orchestrator,
            ready_filter,
            DeliveryMode::Reliable { timeout: Duration::from_secs(5) },
        ).await?;

        debug!("Event subscriptions setup complete");
        Ok(())
    }

    /// Handle incoming error reports
    async fn handle_error_report(&self, error_report: ErrorReport) -> OrchestratorResult<()> {
        warn!(
            "Received error report from {}: {} - {}",
            error_report.module,
            error_report.error_type,
            error_report.message
        );

        // Determine failure type
        let failure_type = match error_report.error_type.as_str() {
            "crash" => FailureType::Crash,
            "timeout" => FailureType::Timeout,
            "resource_exhaustion" => FailureType::ResourceExhaustion,
            "health_check" => FailureType::HealthCheckFailure,
            "dependency" => FailureType::DependencyFailure,
            "config" => FailureType::ConfigurationError,
            "communication" => FailureType::CommunicationFailure,
            _ => FailureType::UnknownError,
        };

        // Create failure record
        let failure = ModuleFailure::new(
            error_report.module,
            failure_type,
            error_report.message.clone(),
        ).with_context(
            error_report.context
                .map(|c| c.to_string())
                .unwrap_or_else(|| "No context provided".to_string())
        );

        // Add to system issues
        let issue = SystemIssue {
            id: Uuid::new_v4(),
            severity: self.map_failure_to_severity(&failure),
            description: format!("{}: {}", error_report.error_type, error_report.message),
            affected_modules: vec![error_report.module],
            timestamp: Instant::now(),
            resolved: false,
        };

        {
            let mut issues = self.active_issues.write().await;
            issues.push(issue);
        }

        // Trigger recovery if auto-recovery is enabled
        let global_config = self.config_manager.get_global_config().await;
        if global_config.auto_recovery {
            if let Err(e) = self.recovery_manager.recover_module(failure).await {
                error!("Recovery failed for module {}: {}", error_report.module, e);
            }
        }

        Ok(())
    }

    /// Map failure to issue severity
    fn map_failure_to_severity(&self, failure: &ModuleFailure) -> IssueSeverity {
        match failure.failure_type {
            FailureType::Crash => IssueSeverity::High,
            FailureType::ResourceExhaustion => IssueSeverity::High,
            FailureType::Timeout => IssueSeverity::Medium,
            FailureType::HealthCheckFailure => IssueSeverity::Medium,
            FailureType::DependencyFailure => IssueSeverity::High,
            FailureType::ConfigurationError => IssueSeverity::Low,
            FailureType::CommunicationFailure => IssueSeverity::Medium,
            FailureType::UnknownError => IssueSeverity::Medium,
        }
    }

    /// Start monitoring services
    async fn start_monitoring(&self) -> OrchestratorResult<()> {
        info!("Starting monitoring services");

        // Start health monitoring
        {
            let mut health_monitor = self.health_monitor.write().await;
            health_monitor.start_monitoring().await?;
        }

        // Start resource monitoring
        {
            let mut resource_manager = self.resource_manager.write().await;
            resource_manager.start_monitoring().await?;
        }

        info!("Monitoring services started");
        Ok(())
    }

    /// Stop monitoring services
    async fn stop_monitoring(&self) -> OrchestratorResult<()> {
        info!("Stopping monitoring services");

        // Stop health monitoring
        {
            let mut health_monitor = self.health_monitor.write().await;
            health_monitor.stop_monitoring().await;
        }

        // Stop resource monitoring
        {
            let mut resource_manager = self.resource_manager.write().await;
            resource_manager.stop_monitoring().await;
        }

        info!("Monitoring services stopped");
        Ok(())
    }

    /// Update system status based on current state
    async fn update_system_status(&self) -> SystemStatus {
        let health_monitor = self.health_monitor.read().await;
        let health_reports = health_monitor.get_all_health_reports();
        
        let unhealthy_modules: Vec<ModuleId> = health_reports
            .iter()
            .filter(|report| matches!(report.status, HealthStatus::Unhealthy { .. }))
            .map(|report| report.module_id)
            .collect();

        let degraded_modules: Vec<ModuleId> = health_reports
            .iter()
            .filter(|report| matches!(report.status, HealthStatus::Degraded { .. }))
            .map(|report| report.module_id)
            .collect();

        let status = if !unhealthy_modules.is_empty() {
            SystemStatus::Critical { failing_modules: unhealthy_modules }
        } else if !degraded_modules.is_empty() {
            SystemStatus::Degraded {
                reason: format!("Modules in degraded state: {:?}", degraded_modules),
            }
        } else {
            SystemStatus::Healthy
        };

        // Update stored status
        {
            let mut system_status = self.system_status.write().await;
            *system_status = status.clone();
        }

        status
    }
}

#[async_trait]
impl OrchestratorTrait for OrchestratorImpl {
    /// Start the entire system with advanced startup sequencing
    async fn start_system(&self) -> OrchestratorResult<()> {
        info!("🚀 Starting system with advanced orchestration");

        // Update system status
        {
            let mut status = self.system_status.write().await;
            *status = SystemStatus::Starting;
        }

        // Start monitoring services first
        self.start_monitoring().await?;
        
        // Start performance telemetry system
        {
            let mut telemetry = self.telemetry_system.write().await;
            telemetry.start().await?;
            info!("Performance telemetry system started");
        }
        
        // Start event loss prevention system
        {
            let mut loss_prevention = self.loss_prevention_system.write().await;
            loss_prevention.start().await?;
            
            // Register queue monitors for all modules
            loss_prevention.register_queue_monitor(ModuleId::DataCapture, 1000);
            loss_prevention.register_queue_monitor(ModuleId::Storage, 2000);
            loss_prevention.register_queue_monitor(ModuleId::AnalysisEngine, 500);
            loss_prevention.register_queue_monitor(ModuleId::AiIntegration, 500);
            loss_prevention.register_queue_monitor(ModuleId::CuteFigurine, 300);
            loss_prevention.register_queue_monitor(ModuleId::Gamification, 200);
            loss_prevention.register_queue_monitor(ModuleId::EventBus, 5000);
            
            info!("Event loss prevention system started with queue monitors");
        }

        // Initialize the startup sequencer
        {
            let mut sequencer_lock = self.startup_sequencer.write().await;
            *sequencer_lock = Some(StartupSequencer::new(
                Arc::clone(&self.registry),
                Arc::clone(&self.lifecycle_controller),
                Arc::clone(&self.health_monitor),
                Arc::clone(&self.config_manager),
                Arc::clone(&self.event_bus),
            ));
        }

        // Execute coordinated startup sequence
        let startup_metrics = {
            let mut sequencer_lock = self.startup_sequencer.write().await;
            if let Some(ref mut sequencer) = *sequencer_lock {
                sequencer.startup_system().await?
            } else {
                return Err(OrchestratorError::ConfigurationError {
                    module: ModuleId::Orchestrator,
                    reason: "Failed to initialize startup sequencer".to_string(),
                });
            }
        };

        // Update system status
        self.update_system_status().await;

        // Log comprehensive startup summary
        let total_time = self.startup_time.elapsed();
        info!("✅ System startup completed successfully!");
        info!("📊 Startup Summary:");
        info!("  - Total time: {:?} (target: <10s)", total_time);
        info!("  - Target met: {}", startup_metrics.target_met);
        info!("  - Modules started: {}", startup_metrics.module_startup_times.len());
        info!("  - Health validation: {:?}", startup_metrics.health_validation_time);
        info!("  - Dependency resolution: {:?}", startup_metrics.dependency_resolution_time);

        if !startup_metrics.bottlenecks.is_empty() {
            warn!("⚠️  Startup bottlenecks detected: {} issues", startup_metrics.bottlenecks.len());
            for bottleneck in &startup_metrics.bottlenecks {
                warn!("  - {}: {:?} - {}", bottleneck.module, bottleneck.duration, bottleneck.reason);
            }
        }

        Ok(())
    }

    /// Stop the entire system gracefully
    async fn stop_system(&self, timeout: Duration) -> OrchestratorResult<()> {
        info!("Stopping system with timeout: {:?}", timeout);

        // Update system status
        {
            let mut status = self.system_status.write().await;
            *status = SystemStatus::Stopping;
        }

        // Stop all modules
        if let Err(e) = self.lifecycle_controller.stop_system(timeout).await {
            warn!("Error during system shutdown: {}", e);
        }

        // Stop monitoring services
        self.stop_monitoring().await?;
        
        // Stop performance telemetry system
        {
            let mut telemetry = self.telemetry_system.write().await;
            telemetry.stop().await;
            info!("Performance telemetry system stopped");
        }
        
        // Stop event loss prevention system
        {
            let mut loss_prevention = self.loss_prevention_system.write().await;
            loss_prevention.stop().await;
            info!("Event loss prevention system stopped");
        }

        // Update system status
        {
            let mut status = self.system_status.write().await;
            *status = SystemStatus::Stopped;
        }

        info!("System shutdown completed");
        Ok(())
    }

    /// Get system health status
    async fn get_system_health(&self) -> SystemHealth {
        let status = self.update_system_status().await;
        
        let health_monitor = self.health_monitor.read().await;
        let health_reports = health_monitor.get_all_health_reports();
        
        let module_health: HashMap<ModuleId, HealthReport> = health_reports
            .into_iter()
            .map(|report| (report.module_id, report))
            .collect();

        let resource_manager = self.resource_manager.read().await;
        let resource_usage = resource_manager.get_system_resources().await
            .unwrap_or_else(|_| SystemResources {
                total_cpu_usage: 0.0,
                total_memory_mb: 0,
                available_memory_mb: 0,
                disk_usage_mb: 0,
                network_bandwidth_kbps: 0.0,
                load_average: (0.0, 0.0, 0.0),
                timestamp: Instant::now(),
            });

        let active_issues = {
            let issues = self.active_issues.read().await;
            issues.clone()
        };

        SystemHealth {
            status,
            uptime: self.startup_time.elapsed(),
            module_health,
            resource_usage,
            active_issues,
            last_updated: Instant::now(),
        }
    }

    /// Update module configuration
    async fn update_config(&self, module_id: ModuleId, config: serde_json::Value) -> OrchestratorResult<()> {
        info!("Updating configuration for module: {}", module_id);
        self.config_manager.update_config(module_id, config).await
    }

    /// Restart a specific module
    async fn restart_module(&self, module_id: ModuleId) -> OrchestratorResult<()> {
        info!("Restarting module: {}", module_id);
        self.lifecycle_controller.restart_module(module_id).await
    }

    /// Register a new module
    async fn register_module(&self, descriptor: ModuleDescriptor) -> OrchestratorResult<()> {
        info!("Registering module: {}", descriptor.id);
        self.registry.register_module(descriptor).await
    }

    /// Get module state
    async fn get_module_state(&self, module_id: ModuleId) -> Option<ModuleState> {
        self.registry.get_module_state(module_id)
    }
}

impl OrchestratorImpl {
    /// Get startup metrics (if available)
    pub async fn get_startup_metrics(&self) -> Option<StartupMetrics> {
        let sequencer_lock = self.startup_sequencer.read().await;
        sequencer_lock.as_ref().map(|sequencer| sequencer.get_metrics().clone())
    }
    
    /// Get performance statistics
    pub async fn get_performance_stats(&self) -> OrchestratorResult<PerformanceStats> {
        let resource_manager = self.resource_manager.read().await;
        resource_manager.get_performance_stats().await
    }
    
    /// Get real-time dashboard data
    pub async fn get_dashboard_data(&self) -> OrchestratorResult<crate::performance_telemetry::DashboardData> {
        let telemetry = self.telemetry_system.read().await;
        telemetry.get_dashboard_data().await
    }
    
    /// Get performance trends over time
    pub async fn get_performance_trends(&self, duration: Duration) -> OrchestratorResult<crate::performance_telemetry::PerformanceTrends> {
        let telemetry = self.telemetry_system.read().await;
        telemetry.get_performance_trends(duration).await
    }
    
    /// Get event loss statistics
    pub async fn get_event_loss_statistics(&self) -> OrchestratorResult<crate::event_loss_prevention::EventLossStatistics> {
        let loss_prevention = self.loss_prevention_system.read().await;
        loss_prevention.get_loss_statistics().await
    }
    
    /// Get resource optimization recommendations
    pub async fn get_optimization_recommendations(&self) -> OrchestratorResult<Vec<crate::resource::OptimizationRecommendation>> {
        let resource_manager = self.resource_manager.read().await;
        resource_manager.get_optimization_recommendations().await
    }
    
    /// Check if event can be enqueued (for event loss prevention)
    pub async fn can_enqueue_event(&self, module_id: ModuleId) -> bool {
        let loss_prevention = self.loss_prevention_system.read().await;
        loss_prevention.can_enqueue(module_id).await
    }
    
    /// Record successful event dequeue
    pub fn record_event_dequeue(&self, module_id: ModuleId) {
        if let Ok(loss_prevention) = self.loss_prevention_system.try_read() {
            loss_prevention.record_dequeue(module_id);
        }
    }
    
    /// Record resource usage for telemetry
    pub async fn record_resource_usage(&self, module_id: ModuleId, usage: crate::resource::ResourceUsage) -> OrchestratorResult<()> {
        let telemetry = self.telemetry_system.read().await;
        telemetry.record_resource_usage(module_id, usage).await
    }
    
    /// Record system resources for telemetry
    pub async fn record_system_resources(&self, resources: SystemResources) -> OrchestratorResult<()> {
        let telemetry = self.telemetry_system.read().await;
        telemetry.record_system_resources(resources).await
    }
}

/// Orchestrator type alias for convenience
pub type Orchestrator = Arc<dyn OrchestratorTrait>;