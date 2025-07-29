//! Enhanced health monitoring with auto-recovery and intelligent alerting

use crate::{
    error::{OrchestratorError, OrchestratorResult},
    health::{HealthMonitor, HealthReport, HealthStatus, HealthMetrics, IssueSeverity},
    recovery::{RecoveryManager, ModuleFailure, FailureType},
    module_registry::ModuleRegistry,
};
use dashmap::DashMap;
use skelly_jelly_event_bus::{EventBusTrait, ModuleId, BusMessage, MessagePayload, message::HealthCheckRequest};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::task::JoinHandle;
use tracing::{debug, warn, error, info};

/// Enhanced health status with more granular states
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EnhancedHealthStatus {
    Healthy { score: f32 },
    Degraded { score: f32, issues: Vec<String> },
    Unhealthy { score: f32, reason: String },
    Critical { reason: String, impact: CriticalImpact },
    Recovering { from_state: String, progress: f32 },
    Unknown,
}

/// Impact level for critical health issues
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CriticalImpact {
    LocalOnly,        // Affects only this module
    ServiceImpact,    // Affects dependent services
    SystemWide,       // Affects entire system
    DataIntegrity,    // Could cause data loss/corruption
}

/// Advanced health metrics with trend analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedHealthMetrics {
    pub cpu_usage: f32,
    pub memory_usage: usize,
    pub memory_trend: TrendDirection,
    pub message_queue_depth: usize,
    pub queue_trend: TrendDirection,
    pub error_rate: f32,
    pub error_trend: TrendDirection,
    pub response_time_ms: f32,
    pub response_trend: TrendDirection,
    pub health_score: f32,
    pub predictive_alerts: Vec<PredictiveAlert>,
}

/// Trend direction for metrics
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TrendDirection {
    Improving,
    Stable,
    Degrading,
    Critical,
}

/// Predictive alert for potential issues
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictiveAlert {
    pub alert_type: AlertType,
    pub confidence: f32,
    pub time_to_impact: Duration,
    pub severity: IssueSeverity,
    pub recommendation: String,
}

/// Types of predictive alerts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertType {
    MemoryExhaustion,
    QueueBacklog,
    ResponseTimeIncrease,
    ErrorRateSpike,
    CpuSaturation,
    DiskSpace,
    NetworkLatency,
}

/// Health monitoring configuration
#[derive(Debug, Clone)]
pub struct HealthConfig {
    pub check_interval: Duration,
    pub check_timeout: Duration,
    pub unhealthy_threshold: u32,
    pub recovery_enabled: bool,
    pub predictive_alerts_enabled: bool,
    pub trend_analysis_window: Duration,
    pub health_score_weights: HealthScoreWeights,
}

/// Weights for calculating health scores
#[derive(Debug, Clone)]
pub struct HealthScoreWeights {
    pub cpu_weight: f32,
    pub memory_weight: f32,
    pub response_time_weight: f32,
    pub error_rate_weight: f32,
    pub queue_depth_weight: f32,
}

impl Default for HealthConfig {
    fn default() -> Self {
        Self {
            check_interval: Duration::from_secs(15),
            check_timeout: Duration::from_secs(5),
            unhealthy_threshold: 3,
            recovery_enabled: true,
            predictive_alerts_enabled: true,
            trend_analysis_window: Duration::from_secs(300),
            health_score_weights: HealthScoreWeights {
                cpu_weight: 0.2,
                memory_weight: 0.3,
                response_time_weight: 0.25,
                error_rate_weight: 0.15,
                queue_depth_weight: 0.1,
            },
        }
    }
}

/// Historical health data point
#[derive(Debug, Clone)]
struct HealthDataPoint {
    timestamp: Instant,
    metrics: EnhancedHealthMetrics,
    status: EnhancedHealthStatus,
}

/// Enhanced health report with trend analysis
#[derive(Debug, Clone)]
pub struct EnhancedHealthReport {
    pub module_id: ModuleId,
    pub status: EnhancedHealthStatus,
    pub last_check: Instant,
    pub metrics: EnhancedHealthMetrics,
    pub uptime: Duration,
    pub check_count: u64,
    pub failure_count: u64,
    pub recovery_count: u64,
    pub last_recovery: Option<Instant>,
    pub trend_analysis: TrendAnalysis,
}

/// Trend analysis for a module
#[derive(Debug, Clone)]
pub struct TrendAnalysis {
    pub overall_trend: TrendDirection,
    pub stability_score: f32,
    pub risk_level: RiskLevel,
    pub predicted_failures: Vec<PredictiveAlert>,
    pub recommendations: Vec<String>,
}

/// Risk assessment levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Enhanced health monitor with intelligent monitoring and auto-recovery
pub struct EnhancedHealthMonitor {
    /// Base health monitor
    base_monitor: Arc<tokio::sync::RwLock<HealthMonitor>>,
    
    /// Module registry
    registry: Arc<ModuleRegistry>,
    
    /// Event bus
    event_bus: Arc<dyn EventBusTrait>,
    
    /// Recovery manager
    recovery_manager: Arc<RecoveryManager>,
    
    /// Enhanced health reports
    enhanced_reports: Arc<DashMap<ModuleId, EnhancedHealthReport>>,
    
    /// Historical data for trend analysis
    health_history: Arc<DashMap<ModuleId, Vec<HealthDataPoint>>>,
    
    /// Configuration
    config: HealthConfig,
    
    /// Monitoring tasks
    monitor_tasks: DashMap<ModuleId, JoinHandle<()>>,
    
    /// Main monitoring task
    main_task: Option<JoinHandle<()>>,
}

impl EnhancedHealthMonitor {
    pub fn new(
        base_monitor: Arc<tokio::sync::RwLock<HealthMonitor>>,
        registry: Arc<ModuleRegistry>,
        event_bus: Arc<dyn EventBusTrait>,
        recovery_manager: Arc<RecoveryManager>,
        config: HealthConfig,
    ) -> Self {
        Self {
            base_monitor,
            registry,
            event_bus,
            recovery_manager,
            enhanced_reports: Arc::new(DashMap::new()),
            health_history: Arc::new(DashMap::new()),
            config,
            monitor_tasks: DashMap::new(),
            main_task: None,
        }
    }

    /// Start enhanced health monitoring
    pub async fn start_monitoring(&mut self) -> OrchestratorResult<()> {
        info!("🏥 Starting enhanced health monitoring with auto-recovery");

        // Start base monitor
        {
            let mut base = self.base_monitor.write().await;
            base.start_monitoring().await?;
        }

        // Initialize enhanced reports for all modules
        let modules = self.registry.get_all_modules();
        for descriptor in modules {
            if descriptor.id == ModuleId::Orchestrator {
                continue;
            }

            self.initialize_enhanced_report(descriptor.id).await;
            self.start_enhanced_monitoring_task(descriptor.id).await?;
        }

        // Start main coordination task
        self.start_main_monitoring_task().await;

        info!("✅ Enhanced health monitoring started");
        Ok(())
    }

    /// Stop enhanced health monitoring
    pub async fn stop_monitoring(&mut self) {
        info!("🛑 Stopping enhanced health monitoring");

        // Stop main task
        if let Some(task) = self.main_task.take() {
            task.abort();
        }

        // Stop all module monitoring tasks
        for entry in self.monitor_tasks.iter() {
            entry.value().abort();
        }
        self.monitor_tasks.clear();

        // Stop base monitor
        {
            let mut base = self.base_monitor.write().await;
            base.stop_monitoring().await;
        }

        info!("✅ Enhanced health monitoring stopped");
    }

    /// Initialize enhanced health report for a module
    async fn initialize_enhanced_report(&self, module_id: ModuleId) {
        let report = EnhancedHealthReport {
            module_id,
            status: EnhancedHealthStatus::Unknown,
            last_check: Instant::now(),
            metrics: EnhancedHealthMetrics {
                cpu_usage: 0.0,
                memory_usage: 0,
                memory_trend: TrendDirection::Stable,
                message_queue_depth: 0,
                queue_trend: TrendDirection::Stable,
                error_rate: 0.0,
                error_trend: TrendDirection::Stable,
                response_time_ms: 0.0,
                response_trend: TrendDirection::Stable,
                health_score: 1.0,
                predictive_alerts: Vec::new(),
            },
            uptime: Duration::ZERO,
            check_count: 0,
            failure_count: 0,
            recovery_count: 0,
            last_recovery: None,
            trend_analysis: TrendAnalysis {
                overall_trend: TrendDirection::Stable,
                stability_score: 1.0,
                risk_level: RiskLevel::Low,
                predicted_failures: Vec::new(),
                recommendations: Vec::new(),
            },
        };

        self.enhanced_reports.insert(module_id, report);
        self.health_history.insert(module_id, Vec::new());
    }

    /// Start enhanced monitoring task for a specific module
    async fn start_enhanced_monitoring_task(&self, module_id: ModuleId) -> OrchestratorResult<()> {
        let event_bus = Arc::clone(&self.event_bus);
        let enhanced_reports = Arc::clone(&self.enhanced_reports);
        let health_history = Arc::clone(&self.health_history);
        let recovery_manager = Arc::clone(&self.recovery_manager);
        let check_interval = self.config.check_interval;
        let check_timeout = self.config.check_timeout;
        let config = self.config.clone();

        let task = tokio::spawn(async move {
            let mut interval = tokio::time::interval(check_interval);
            let mut consecutive_failures = 0u32;

            loop {
                interval.tick().await;

                // Perform enhanced health check
                let enhanced_metrics = Self::perform_enhanced_health_check(
                    module_id,
                    &event_bus,
                    check_timeout,
                ).await;

                // Update enhanced report
                if let Some(mut report_entry) = enhanced_reports.get_mut(&module_id) {
                    let report = report_entry.value_mut();
                    
                    match enhanced_metrics {
                        Ok(metrics) => {
                            // Calculate health score
                            let health_score = Self::calculate_health_score(&metrics, &config.health_score_weights);
                            
                            // Determine status based on health score
                            let status = match health_score {
                                score if score >= 0.8 => EnhancedHealthStatus::Healthy { score },
                                score if score >= 0.6 => EnhancedHealthStatus::Degraded { 
                                    score, 
                                    issues: Self::identify_issues(&metrics) 
                                },
                                score if score >= 0.3 => EnhancedHealthStatus::Unhealthy { 
                                    score, 
                                    reason: "Multiple performance issues detected".to_string() 
                                },
                                score => EnhancedHealthStatus::Critical { 
                                    reason: format!("Critical health score: {:.2}", score),
                                    impact: Self::assess_critical_impact(module_id),
                                },
                            };

                            // Update trend analysis
                            Self::update_trend_analysis(
                                &health_history, 
                                module_id, 
                                &metrics, 
                                &status,
                                config.trend_analysis_window
                            );

                            // Update report
                            report.status = status.clone();
                            report.metrics = metrics;
                            report.last_check = Instant::now();
                            report.check_count += 1;
                            consecutive_failures = 0;

                            // Check for auto-recovery triggers
                            if config.recovery_enabled && report.trend_analysis.risk_level >= RiskLevel::High {
                                Self::trigger_proactive_recovery(
                                    module_id,
                                    &recovery_manager,
                                    &report.trend_analysis
                                ).await;
                            }
                        }
                        Err(error) => {
                            consecutive_failures += 1;
                            report.failure_count += 1;
                            report.last_check = Instant::now();
                            report.check_count += 1;

                            if consecutive_failures >= config.unhealthy_threshold {
                                report.status = EnhancedHealthStatus::Critical {
                                    reason: error.to_string(),
                                    impact: Self::assess_critical_impact(module_id),
                                };

                                // Trigger immediate recovery
                                if config.recovery_enabled {
                                    let failure = ModuleFailure::new(
                                        module_id,
                                        FailureType::HealthCheckFailure,
                                        error.to_string(),
                                    );
                                    
                                    if let Err(e) = recovery_manager.recover_module(failure).await {
                                        error!("❌ Auto-recovery failed for {}: {}", module_id, e);
                                    } else {
                                        report.recovery_count += 1;
                                        report.last_recovery = Some(Instant::now());
                                        info!("🔄 Auto-recovery initiated for {}", module_id);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });

        self.monitor_tasks.insert(module_id, task);
        Ok(())
    }

    /// Start main monitoring coordination task
    async fn start_main_monitoring_task(&mut self) {
        let enhanced_reports = Arc::clone(&self.enhanced_reports);
        let event_bus = Arc::clone(&self.event_bus);
        let config = self.config.clone();

        let task = tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60)); // System-wide analysis every minute

            loop {
                interval.tick().await;

                // Perform system-wide health analysis
                Self::perform_system_analysis(&enhanced_reports, &event_bus, &config).await;

                // Clean up old historical data
                Self::cleanup_health_history(&enhanced_reports, config.trend_analysis_window * 2).await;
            }
        });

        self.main_task = Some(task);
    }

    /// Perform enhanced health check for a module
    async fn perform_enhanced_health_check(
        module_id: ModuleId,
        event_bus: &Arc<dyn EventBusTrait>,
        timeout: Duration,
    ) -> OrchestratorResult<EnhancedHealthMetrics> {
        debug!("🔍 Performing enhanced health check for module: {}", module_id);

        let request = HealthCheckRequest {
            module_id,
            timestamp: chrono::Utc::now(),
        };

        let message = BusMessage::new(
            ModuleId::Orchestrator,
            MessagePayload::HealthCheck(request),
        );

        // Send health check request with timeout
        match tokio::time::timeout(timeout, event_bus.publish(message)).await {
            Ok(Ok(_)) => {
                // For now, generate enhanced metrics based on simulated data
                // In a real implementation, this would wait for and parse the response
                Ok(Self::generate_enhanced_metrics(module_id))
            }
            Ok(Err(e)) => Err(OrchestratorError::HealthCheckFailed {
                module: module_id,
                reason: e.to_string(),
            }),
            Err(_) => Err(OrchestratorError::HealthCheckFailed {
                module: module_id,
                reason: "Health check timeout".to_string(),
            }),
        }
    }

    /// Generate enhanced metrics (simulation for now)
    fn generate_enhanced_metrics(module_id: ModuleId) -> EnhancedHealthMetrics {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        // Simulate different health patterns based on module type
        let (base_cpu, base_memory, base_response) = match module_id {
            ModuleId::EventBus => (15.0, 50_000_000, 10.0),
            ModuleId::Storage => (25.0, 100_000_000, 50.0),
            ModuleId::DataCapture => (20.0, 30_000_000, 20.0),
            ModuleId::AnalysisEngine => (40.0, 200_000_000, 100.0),
            ModuleId::Gamification => (10.0, 40_000_000, 30.0),
            ModuleId::AiIntegration => (35.0, 150_000_000, 200.0),
            ModuleId::CuteFigurine => (15.0, 60_000_000, 40.0),
            ModuleId::Orchestrator => (5.0, 20_000_000, 5.0),
        };

        let cpu_usage = base_cpu + rng.gen_range(-5.0..5.0);
        let memory_usage = base_memory + rng.gen_range(-10_000_000..10_000_000) as usize;
        let response_time = base_response + rng.gen_range(-10.0..20.0);
        let error_rate = rng.gen_range(0.0..0.05);
        let queue_depth = rng.gen_range(0..20);

        EnhancedHealthMetrics {
            cpu_usage,
            memory_usage,
            memory_trend: TrendDirection::Stable,
            message_queue_depth: queue_depth,
            queue_trend: TrendDirection::Stable,
            error_rate,
            error_trend: TrendDirection::Stable,
            response_time_ms: response_time,
            response_trend: TrendDirection::Stable,
            health_score: 0.9, // Will be recalculated
            predictive_alerts: Vec::new(),
        }
    }

    /// Calculate overall health score
    fn calculate_health_score(metrics: &EnhancedHealthMetrics, weights: &HealthScoreWeights) -> f32 {
        let cpu_score = (100.0 - metrics.cpu_usage).max(0.0) / 100.0;
        let memory_score = if metrics.memory_usage > 500_000_000 { 0.5 } else { 1.0 };
        let response_score = (200.0 - metrics.response_time_ms).max(0.0) / 200.0;
        let error_score = (1.0 - metrics.error_rate * 20.0).max(0.0);
        let queue_score = (50.0 - metrics.message_queue_depth as f32).max(0.0) / 50.0;

        (cpu_score * weights.cpu_weight +
         memory_score * weights.memory_weight +
         response_score * weights.response_time_weight +
         error_score * weights.error_rate_weight +
         queue_score * weights.queue_depth_weight).min(1.0).max(0.0)
    }

    /// Identify specific issues from metrics
    fn identify_issues(metrics: &EnhancedHealthMetrics) -> Vec<String> {
        let mut issues = Vec::new();

        if metrics.cpu_usage > 80.0 {
            issues.push("High CPU usage detected".to_string());
        }
        if metrics.memory_usage > 400_000_000 {
            issues.push("High memory usage detected".to_string());
        }
        if metrics.response_time_ms > 150.0 {
            issues.push("Slow response times detected".to_string());
        }
        if metrics.error_rate > 0.02 {
            issues.push("Elevated error rate detected".to_string());
        }
        if metrics.message_queue_depth > 30 {
            issues.push("Message queue backlog detected".to_string());
        }

        issues
    }

    /// Assess critical impact level
    fn assess_critical_impact(module_id: ModuleId) -> CriticalImpact {
        match module_id {
            ModuleId::EventBus | ModuleId::Orchestrator => CriticalImpact::SystemWide,
            ModuleId::Storage => CriticalImpact::DataIntegrity,
            ModuleId::AnalysisEngine | ModuleId::Gamification => CriticalImpact::ServiceImpact,
            _ => CriticalImpact::LocalOnly,
        }
    }

    /// Update trend analysis for a module
    fn update_trend_analysis(
        health_history: &Arc<DashMap<ModuleId, Vec<HealthDataPoint>>>,
        module_id: ModuleId,
        metrics: &EnhancedHealthMetrics,
        status: &EnhancedHealthStatus,
        window: Duration,
    ) {
        if let Some(mut history_entry) = health_history.get_mut(&module_id) {
            let history = history_entry.value_mut();
            
            // Add new data point
            history.push(HealthDataPoint {
                timestamp: Instant::now(),
                metrics: metrics.clone(),
                status: status.clone(),
            });

            // Remove old data points
            let cutoff = Instant::now() - window;
            history.retain(|point| point.timestamp > cutoff);

            // Analyze trends (simplified implementation)
            // In a real implementation, this would use more sophisticated trend analysis
        }
    }

    /// Trigger proactive recovery based on trend analysis
    async fn trigger_proactive_recovery(
        module_id: ModuleId,
        recovery_manager: &Arc<RecoveryManager>,
        trend_analysis: &TrendAnalysis,
    ) {
        warn!("🚨 Proactive recovery triggered for {} (risk: {:?})", module_id, trend_analysis.risk_level);

        // Create a predictive failure based on trend analysis
        let failure = ModuleFailure::new(
            module_id,
            FailureType::HealthCheckFailure,
            format!("Proactive recovery due to trend analysis (risk: {:?})", trend_analysis.risk_level),
        );

        if let Err(e) = recovery_manager.recover_module(failure).await {
            error!("❌ Proactive recovery failed for {}: {}", module_id, e);
        } else {
            info!("✅ Proactive recovery completed for {}", module_id);
        }
    }

    /// Perform system-wide health analysis
    async fn perform_system_analysis(
        enhanced_reports: &Arc<DashMap<ModuleId, EnhancedHealthReport>>,
        _event_bus: &Arc<dyn EventBusTrait>,
        _config: &HealthConfig,
    ) {
        let mut system_health_score = 0.0;
        let mut total_modules = 0;
        let mut critical_modules = 0;

        for entry in enhanced_reports.iter() {
            let report = entry.value();
            total_modules += 1;

            match &report.status {
                EnhancedHealthStatus::Healthy { score } => {
                    system_health_score += score;
                }
                EnhancedHealthStatus::Degraded { score, .. } => {
                    system_health_score += score;
                }
                EnhancedHealthStatus::Unhealthy { score, .. } => {
                    system_health_score += score * 0.5;
                }
                EnhancedHealthStatus::Critical { .. } => {
                    critical_modules += 1;
                }
                EnhancedHealthStatus::Recovering { .. } => {
                    system_health_score += 0.5;
                }
                EnhancedHealthStatus::Unknown => {}
            }
        }

        if total_modules > 0 {
            system_health_score /= total_modules as f32;
        }

        debug!("📊 System Health Analysis:");
        debug!("  - Overall Score: {:.2}", system_health_score);
        debug!("  - Total Modules: {}", total_modules);
        debug!("  - Critical Modules: {}", critical_modules);
    }

    /// Clean up old health history data
    async fn cleanup_health_history(
        _enhanced_reports: &Arc<DashMap<ModuleId, EnhancedHealthReport>>,
        _retention_period: Duration,
    ) {
        // Implementation would clean up old historical data
        debug!("🧹 Cleaning up old health history data");
    }

    /// Get enhanced health report for a module
    pub fn get_enhanced_report(&self, module_id: ModuleId) -> Option<EnhancedHealthReport> {
        self.enhanced_reports.get(&module_id).map(|entry| entry.clone())
    }

    /// Get all enhanced health reports
    pub fn get_all_enhanced_reports(&self) -> Vec<EnhancedHealthReport> {
        self.enhanced_reports.iter().map(|entry| entry.clone()).collect()
    }

    /// Get system-wide health status
    pub fn get_system_health_status(&self) -> EnhancedHealthStatus {
        let reports = self.get_all_enhanced_reports();
        
        if reports.is_empty() {
            return EnhancedHealthStatus::Unknown;
        }

        let mut total_score = 0.0;
        let mut critical_count = 0;
        let mut unhealthy_count = 0;

        for report in &reports {
            match &report.status {
                EnhancedHealthStatus::Healthy { score } => total_score += score,
                EnhancedHealthStatus::Degraded { score, .. } => total_score += score,
                EnhancedHealthStatus::Unhealthy { score, .. } => {
                    total_score += score;
                    unhealthy_count += 1;
                }
                EnhancedHealthStatus::Critical { .. } => {
                    critical_count += 1;
                }
                EnhancedHealthStatus::Recovering { .. } => total_score += 0.5,
                EnhancedHealthStatus::Unknown => {}
            }
        }

        let avg_score = total_score / reports.len() as f32;

        if critical_count > 0 {
            EnhancedHealthStatus::Critical {
                reason: format!("{} modules in critical state", critical_count),
                impact: CriticalImpact::SystemWide,
            }
        } else if unhealthy_count > 0 {
            EnhancedHealthStatus::Unhealthy {
                score: avg_score,
                reason: format!("{} modules unhealthy", unhealthy_count),
            }
        } else if avg_score < 0.8 {
            EnhancedHealthStatus::Degraded {
                score: avg_score,
                issues: vec![format!("System average health score: {:.2}", avg_score)],
            }
        } else {
            EnhancedHealthStatus::Healthy { score: avg_score }
        }
    }
}