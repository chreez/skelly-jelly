//! Performance telemetry system for real-time monitoring and regression detection

use crate::error::{OrchestratorError, OrchestratorResult};
use crate::resource::{ResourceUsage, SystemResources, PerformanceStats, OptimizationRecommendation};
use dashmap::DashMap;
use skelly_jelly_event_bus::ModuleId;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::{
    sync::{RwLock, mpsc, watch},
    task::JoinHandle,
    time::interval,
};
use tracing::{debug, warn, error, info};

/// Performance metrics aggregation period
const METRICS_AGGREGATION_PERIOD: Duration = Duration::from_secs(60);
const METRICS_RETENTION_PERIOD: Duration = Duration::from_secs(3600); // 1 hour
const REGRESSION_DETECTION_SAMPLES: usize = 10;

/// Performance telemetry system
pub struct PerformanceTelemetrySystem {
    /// Metrics storage
    metrics_store: Arc<RwLock<MetricsStore>>,
    
    /// Real-time metrics aggregator
    aggregator: Arc<MetricsAggregator>,
    
    /// Regression detector
    regression_detector: Arc<RegressionDetector>,
    
    /// Alert system
    alert_system: Arc<AlertSystem>,
    
    /// Background tasks
    aggregation_task: Option<JoinHandle<()>>,
    cleanup_task: Option<JoinHandle<()>>,
    
    /// Configuration
    config: TelemetryConfig,
}

#[derive(Debug, Clone)]
pub struct TelemetryConfig {
    pub enabled: bool,
    pub collection_interval: Duration,
    pub aggregation_interval: Duration,
    pub retention_period: Duration,
    pub regression_threshold: f32,
    pub alert_thresholds: AlertThresholds,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            collection_interval: Duration::from_secs(10),
            aggregation_interval: METRICS_AGGREGATION_PERIOD,
            retention_period: METRICS_RETENTION_PERIOD,
            regression_threshold: 0.2, // 20% degradation
            alert_thresholds: AlertThresholds::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AlertThresholds {
    pub cpu_usage_threshold: f32,
    pub memory_usage_threshold: usize,
    pub event_loss_threshold: f32,
    pub battery_drain_threshold: f32,
}

impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            cpu_usage_threshold: 2.0,  // 2% CPU
            memory_usage_threshold: 200, // 200MB
            event_loss_threshold: 0.001, // 0.1%
            battery_drain_threshold: 0.05, // 5%
        }
    }
}

/// Metrics storage with time-series data
#[derive(Debug)]
pub struct MetricsStore {
    /// Per-module resource metrics
    module_metrics: HashMap<ModuleId, VecDeque<TimestampedResourceUsage>>,
    
    /// System-wide metrics
    system_metrics: VecDeque<TimestampedSystemResources>,
    
    /// Performance statistics
    performance_stats: VecDeque<TimestampedPerformanceStats>,
    
    /// Alert history
    alert_history: VecDeque<AlertEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimestampedResourceUsage {
    pub usage: ResourceUsage,
    pub timestamp: Instant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimestampedSystemResources {
    pub resources: SystemResources,
    pub timestamp: Instant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimestampedPerformanceStats {
    pub stats: PerformanceStats,
    pub timestamp: Instant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertEvent {
    pub alert_type: AlertType,
    pub severity: AlertSeverity,
    pub message: String,
    pub module_id: Option<ModuleId>,
    pub timestamp: Instant,
    pub resolved: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertType {
    CpuThresholdExceeded,
    MemoryThresholdExceeded,
    EventLossDetected,
    BatteryDrainHigh,
    PerformanceRegression,
    SystemHealthLow,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum AlertSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

impl PerformanceTelemetrySystem {
    /// Create a new performance telemetry system
    pub fn new(config: TelemetryConfig) -> Self {
        let metrics_store = Arc::new(RwLock::new(MetricsStore {
            module_metrics: HashMap::new(),
            system_metrics: VecDeque::new(),
            performance_stats: VecDeque::new(),
            alert_history: VecDeque::new(),
        }));

        let aggregator = Arc::new(MetricsAggregator::new());
        let regression_detector = Arc::new(RegressionDetector::new(config.regression_threshold));
        let alert_system = Arc::new(AlertSystem::new(config.alert_thresholds.clone()));

        Self {
            metrics_store,
            aggregator,
            regression_detector,
            alert_system,
            aggregation_task: None,
            cleanup_task: None,
            config,
        }
    }

    /// Start the telemetry system
    pub async fn start(&mut self) -> OrchestratorResult<()> {
        if !self.config.enabled {
            info!("Performance telemetry disabled");
            return Ok(());
        }

        info!("Starting performance telemetry system");

        // Start metrics aggregation task
        let metrics_store = Arc::clone(&self.metrics_store);
        let aggregator = Arc::clone(&self.aggregator);
        let regression_detector = Arc::clone(&self.regression_detector);
        let alert_system = Arc::clone(&self.alert_system);
        let aggregation_interval = self.config.aggregation_interval;

        let aggregation_task = tokio::spawn(async move {
            let mut interval = interval(aggregation_interval);
            
            loop {
                interval.tick().await;
                
                if let Err(e) = Self::run_aggregation(
                    &metrics_store,
                    &aggregator,
                    &regression_detector,
                    &alert_system,
                ).await {
                    error!("Failed to run metrics aggregation: {}", e);
                }
            }
        });

        // Start cleanup task
        let metrics_store_cleanup = Arc::clone(&self.metrics_store);
        let retention_period = self.config.retention_period;

        let cleanup_task = tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(300)); // Every 5 minutes
            
            loop {
                interval.tick().await;
                
                if let Err(e) = Self::cleanup_old_metrics(&metrics_store_cleanup, retention_period).await {
                    error!("Failed to cleanup old metrics: {}", e);
                }
            }
        });

        self.aggregation_task = Some(aggregation_task);
        self.cleanup_task = Some(cleanup_task);

        info!("Performance telemetry system started");
        Ok(())
    }

    /// Stop the telemetry system
    pub async fn stop(&mut self) {
        info!("Stopping performance telemetry system");

        if let Some(task) = self.aggregation_task.take() {
            task.abort();
        }

        if let Some(task) = self.cleanup_task.take() {
            task.abort();
        }

        info!("Performance telemetry system stopped");
    }

    /// Record resource usage for a module
    pub async fn record_resource_usage(&self, module_id: ModuleId, usage: ResourceUsage) -> OrchestratorResult<()> {
        if !self.config.enabled {
            return Ok(());
        }

        let timestamped_usage = TimestampedResourceUsage {
            usage: usage.clone(),
            timestamp: Instant::now(),
        };

        {
            let mut store = self.metrics_store.write().await;
            let module_metrics = store.module_metrics.entry(module_id).or_insert_with(VecDeque::new);
            module_metrics.push_back(timestamped_usage);
            
            // Limit the size to prevent memory growth
            if module_metrics.len() > 1000 {
                module_metrics.pop_front();
            }
        }

        // Check for immediate alerts
        self.alert_system.check_resource_usage_alerts(module_id, &usage).await;

        Ok(())
    }

    /// Record system resources
    pub async fn record_system_resources(&self, resources: SystemResources) -> OrchestratorResult<()> {
        if !self.config.enabled {
            return Ok(());
        }

        let timestamped_resources = TimestampedSystemResources {
            resources: resources.clone(),
            timestamp: Instant::now(),
        };

        {
            let mut store = self.metrics_store.write().await;
            store.system_metrics.push_back(timestamped_resources);
            
            // Limit the size
            if store.system_metrics.len() > 1000 {
                store.system_metrics.pop_front();
            }
        }

        // Check for system-level alerts
        self.alert_system.check_system_alerts(&resources).await;

        Ok(())
    }

    /// Record performance statistics
    pub async fn record_performance_stats(&self, stats: PerformanceStats) -> OrchestratorResult<()> {
        if !self.config.enabled {
            return Ok(());
        }

        let timestamped_stats = TimestampedPerformanceStats {
            stats: stats.clone(),
            timestamp: Instant::now(),
        };

        {
            let mut store = self.metrics_store.write().await;
            store.performance_stats.push_back(timestamped_stats);
            
            // Limit the size
            if store.performance_stats.len() > 1000 {
                store.performance_stats.pop_front();
            }
        }

        // Check for performance regression
        self.regression_detector.check_regression(&stats).await;

        Ok(())
    }

    /// Get real-time dashboard data
    pub async fn get_dashboard_data(&self) -> OrchestratorResult<DashboardData> {
        let store = self.metrics_store.read().await;
        
        // Get latest metrics for each module
        let mut module_summaries = HashMap::new();
        for (module_id, metrics) in &store.module_metrics {
            if let Some(latest) = metrics.back() {
                module_summaries.insert(*module_id, latest.usage.clone());
            }
        }

        // Get latest system resources
        let latest_system = store.system_metrics.back().map(|ts| ts.resources.clone());

        // Get latest performance stats
        let latest_performance = store.performance_stats.back().map(|ts| ts.stats.clone());

        // Get recent alerts
        let recent_alerts: Vec<AlertEvent> = store.alert_history
            .iter()
            .rev()
            .take(10)
            .cloned()
            .collect();

        Ok(DashboardData {
            module_summaries,
            system_resources: latest_system,
            performance_stats: latest_performance,
            recent_alerts,
            last_updated: Instant::now(),
        })
    }

    /// Get performance trends
    pub async fn get_performance_trends(&self, duration: Duration) -> OrchestratorResult<PerformanceTrends> {
        let store = self.metrics_store.read().await;
        let cutoff_time = Instant::now() - duration;

        // Collect CPU usage trend
        let cpu_trend: Vec<(Instant, f32)> = store.performance_stats
            .iter()
            .filter(|ts_stats| ts_stats.timestamp >= cutoff_time)
            .map(|ts_stats| (ts_stats.timestamp, ts_stats.stats.total_cpu_usage))
            .collect();

        // Collect memory usage trend
        let memory_trend: Vec<(Instant, usize)> = store.performance_stats
            .iter()
            .filter(|ts_stats| ts_stats.timestamp >= cutoff_time)
            .map(|ts_stats| (ts_stats.timestamp, ts_stats.stats.total_memory_usage))
            .collect();

        // Calculate efficiency trends
        let efficiency_trend: Vec<(Instant, f32)> = store.performance_stats
            .iter()
            .filter(|ts_stats| ts_stats.timestamp >= cutoff_time)
            .map(|ts_stats| (ts_stats.timestamp, ts_stats.stats.system_health_score))
            .collect();

        Ok(PerformanceTrends {
            cpu_usage_trend: cpu_trend,
            memory_usage_trend: memory_trend,
            efficiency_trend,
            duration,
            samples: store.performance_stats.len(),
        })
    }

    /// Run metrics aggregation
    async fn run_aggregation(
        metrics_store: &Arc<RwLock<MetricsStore>>,
        _aggregator: &Arc<MetricsAggregator>,
        regression_detector: &Arc<RegressionDetector>,
        alert_system: &Arc<AlertSystem>,
    ) -> OrchestratorResult<()> {
        debug!("Running metrics aggregation");

        let store = metrics_store.read().await;
        
        // Check for performance regressions
        if let Some(latest_stats) = store.performance_stats.back() {
            regression_detector.check_regression(&latest_stats.stats).await;
        }

        // Update alert system with current state
        alert_system.process_aggregated_metrics(&*store).await;

        debug!("Metrics aggregation completed");
        Ok(())
    }

    /// Clean up old metrics
    async fn cleanup_old_metrics(
        metrics_store: &Arc<RwLock<MetricsStore>>,
        retention_period: Duration,
    ) -> OrchestratorResult<()> {
        debug!("Cleaning up old metrics");

        let cutoff_time = Instant::now() - retention_period;
        let mut store = metrics_store.write().await;

        // Clean up module metrics
        for metrics in store.module_metrics.values_mut() {
            metrics.retain(|ts_usage| ts_usage.timestamp >= cutoff_time);
        }

        // Clean up system metrics
        store.system_metrics.retain(|ts_resources| ts_resources.timestamp >= cutoff_time);

        // Clean up performance stats
        store.performance_stats.retain(|ts_stats| ts_stats.timestamp >= cutoff_time);

        // Clean up old alerts
        store.alert_history.retain(|alert| alert.timestamp >= cutoff_time);

        debug!("Old metrics cleanup completed");
        Ok(())
    }
}

/// Dashboard data structure
#[derive(Debug, Clone)]
pub struct DashboardData {
    pub module_summaries: HashMap<ModuleId, ResourceUsage>,
    pub system_resources: Option<SystemResources>,
    pub performance_stats: Option<PerformanceStats>,
    pub recent_alerts: Vec<AlertEvent>,
    pub last_updated: Instant,
}

/// Performance trends over time
#[derive(Debug, Clone)]
pub struct PerformanceTrends {
    pub cpu_usage_trend: Vec<(Instant, f32)>,
    pub memory_usage_trend: Vec<(Instant, usize)>,
    pub efficiency_trend: Vec<(Instant, f32)>,
    pub duration: Duration,
    pub samples: usize,
}

/// Metrics aggregator for statistical analysis
pub struct MetricsAggregator {
    // Future implementation for statistical aggregation
}

impl MetricsAggregator {
    pub fn new() -> Self {
        Self {}
    }
}

/// Regression detector for performance degradation
pub struct RegressionDetector {
    regression_threshold: f32,
    baseline_metrics: Arc<RwLock<Option<PerformanceBaseline>>>,
}

#[derive(Debug, Clone)]
pub struct PerformanceBaseline {
    pub cpu_usage_baseline: f32,
    pub memory_usage_baseline: usize,
    pub efficiency_baseline: f32,
    pub established_at: Instant,
}

impl RegressionDetector {
    pub fn new(threshold: f32) -> Self {
        Self {
            regression_threshold: threshold,
            baseline_metrics: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn check_regression(&self, current_stats: &PerformanceStats) {
        let baseline = self.baseline_metrics.read().await;
        
        if let Some(baseline) = baseline.as_ref() {
            // Check for CPU regression
            let cpu_degradation = (current_stats.total_cpu_usage - baseline.cpu_usage_baseline) / baseline.cpu_usage_baseline;
            if cpu_degradation > self.regression_threshold {
                warn!("CPU performance regression detected: {:.1}% increase", cpu_degradation * 100.0);
            }

            // Check for memory regression
            let memory_degradation = (current_stats.total_memory_usage as f32 - baseline.memory_usage_baseline as f32) / baseline.memory_usage_baseline as f32;
            if memory_degradation > self.regression_threshold {
                warn!("Memory performance regression detected: {:.1}% increase", memory_degradation * 100.0);
            }

            // Check for efficiency regression
            let efficiency_degradation = (baseline.efficiency_baseline - current_stats.system_health_score) / baseline.efficiency_baseline;
            if efficiency_degradation > self.regression_threshold {
                warn!("System efficiency regression detected: {:.1}% decrease", efficiency_degradation * 100.0);
            }
        } else {
            // Establish baseline if not set
            let mut baseline_mut = self.baseline_metrics.write().await;
            *baseline_mut = Some(PerformanceBaseline {
                cpu_usage_baseline: current_stats.total_cpu_usage,
                memory_usage_baseline: current_stats.total_memory_usage,
                efficiency_baseline: current_stats.system_health_score,
                established_at: Instant::now(),
            });
            info!("Performance baseline established");
        }
    }
}

/// Alert system for proactive monitoring
pub struct AlertSystem {
    thresholds: AlertThresholds,
    active_alerts: Arc<RwLock<HashMap<String, AlertEvent>>>,
}

impl AlertSystem {
    pub fn new(thresholds: AlertThresholds) -> Self {
        Self {
            thresholds,
            active_alerts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn check_resource_usage_alerts(&self, module_id: ModuleId, usage: &ResourceUsage) {
        // Check CPU threshold
        if usage.cpu_percent > self.thresholds.cpu_usage_threshold {
            self.trigger_alert(AlertEvent {
                alert_type: AlertType::CpuThresholdExceeded,
                severity: AlertSeverity::Warning,
                message: format!("Module {:?} CPU usage ({:.2}%) exceeds threshold ({:.2}%)", 
                    module_id, usage.cpu_percent, self.thresholds.cpu_usage_threshold),
                module_id: Some(module_id),
                timestamp: Instant::now(),
                resolved: false,
            }).await;
        }

        // Check memory threshold
        if usage.memory_mb > self.thresholds.memory_usage_threshold {
            self.trigger_alert(AlertEvent {
                alert_type: AlertType::MemoryThresholdExceeded,
                severity: AlertSeverity::Warning,
                message: format!("Module {:?} memory usage ({}MB) exceeds threshold ({}MB)", 
                    module_id, usage.memory_mb, self.thresholds.memory_usage_threshold),
                module_id: Some(module_id),
                timestamp: Instant::now(),
                resolved: false,
            }).await;
        }

        // Check battery drain
        if usage.battery_impact > self.thresholds.battery_drain_threshold {
            self.trigger_alert(AlertEvent {
                alert_type: AlertType::BatteryDrainHigh,
                severity: AlertSeverity::Error,
                message: format!("Module {:?} battery drain ({:.1}%) exceeds threshold ({:.1}%)", 
                    module_id, usage.battery_impact * 100.0, self.thresholds.battery_drain_threshold * 100.0),
                module_id: Some(module_id),
                timestamp: Instant::now(),
                resolved: false,
            }).await;
        }
    }

    pub async fn check_system_alerts(&self, resources: &SystemResources) {
        // Check system health
        let health_score = resources.system_health_score();
        if health_score < 0.5 {
            self.trigger_alert(AlertEvent {
                alert_type: AlertType::SystemHealthLow,
                severity: AlertSeverity::Error,
                message: format!("System health score ({:.2}) is critically low", health_score),
                module_id: None,
                timestamp: Instant::now(),
                resolved: false,
            }).await;
        }
    }

    pub async fn process_aggregated_metrics(&self, _store: &MetricsStore) {
        // Future implementation for processing aggregated metrics
        // This would analyze trends and trigger predictive alerts
    }

    async fn trigger_alert(&self, alert: AlertEvent) {
        let alert_key = format!("{:?}_{:?}", alert.alert_type, alert.module_id);
        
        {
            let mut active_alerts = self.active_alerts.write().await;
            active_alerts.insert(alert_key.clone(), alert.clone());
        }

        // Log the alert
        match alert.severity {
            AlertSeverity::Info => info!("{}", alert.message),
            AlertSeverity::Warning => warn!("{}", alert.message),
            AlertSeverity::Error => error!("{}", alert.message),
            AlertSeverity::Critical => error!("CRITICAL: {}", alert.message),
        }
    }
}