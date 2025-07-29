//! Health monitoring for system modules

use crate::error::{OrchestratorError, OrchestratorResult};
use crate::module_registry::ModuleRegistry;
use dashmap::DashMap;
use skelly_jelly_event_bus::{EventBusTrait, ModuleId, BusMessage, MessagePayload, message::HealthCheckRequest};
use serde::{Deserialize, Serialize};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::task::JoinHandle;
use tracing::{debug, warn, error, info};
use uuid::Uuid;

/// Health status of a module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded { issues: Vec<String> },
    Unhealthy { reason: String },
    Unknown,
}

/// Health metrics for a module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthMetrics {
    pub cpu_usage: f32,
    pub memory_usage: usize,
    pub message_queue_depth: usize,
    pub error_rate: f32,
    pub response_time_ms: f32,
}

impl Default for HealthMetrics {
    fn default() -> Self {
        Self {
            cpu_usage: 0.0,
            memory_usage: 0,
            message_queue_depth: 0,
            error_rate: 0.0,
            response_time_ms: 0.0,
        }
    }
}

/// Health issue reported by a module
#[derive(Debug, Clone)]
pub struct HealthIssue {
    pub severity: IssueSeverity,
    pub description: String,
    pub timestamp: Instant,
}

/// Severity levels for health issues
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum IssueSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Comprehensive health report for a module
#[derive(Debug, Clone)]
pub struct HealthReport {
    pub module_id: ModuleId,
    pub status: HealthStatus,
    pub last_check: Instant,
    pub metrics: HealthMetrics,
    pub issues: Vec<HealthIssue>,
    pub uptime: Duration,
    pub check_count: u64,
    pub failure_count: u64,
}

impl HealthReport {
    pub fn new(module_id: ModuleId) -> Self {
        Self {
            module_id,
            status: HealthStatus::Unknown,
            last_check: Instant::now(),
            metrics: HealthMetrics::default(),
            issues: Vec::new(),
            uptime: Duration::ZERO,
            check_count: 0,
            failure_count: 0,
        }
    }

    pub fn healthy(module_id: ModuleId) -> Self {
        let mut report = Self::new(module_id);
        report.status = HealthStatus::Healthy;
        report
    }

    pub fn unhealthy(module_id: ModuleId, reason: &str) -> Self {
        let mut report = Self::new(module_id);
        report.status = HealthStatus::Unhealthy {
            reason: reason.to_string(),
        };
        report.failure_count = 1;
        report
    }

    pub fn degraded(module_id: ModuleId, issues: Vec<String>) -> Self {
        let mut report = Self::new(module_id);
        report.status = HealthStatus::Degraded { issues };
        report
    }

    pub fn is_healthy(&self) -> bool {
        matches!(self.status, HealthStatus::Healthy)
    }

    pub fn is_critical(&self) -> bool {
        matches!(self.status, HealthStatus::Unhealthy { .. }) ||
        self.issues.iter().any(|issue| matches!(issue.severity, IssueSeverity::Critical))
    }

    pub fn update_metrics(&mut self, metrics: HealthMetrics) {
        self.metrics = metrics;
        self.last_check = Instant::now();
        self.check_count += 1;
    }

    pub fn add_issue(&mut self, severity: IssueSeverity, description: String) {
        self.issues.push(HealthIssue {
            severity,
            description,
            timestamp: Instant::now(),
        });
    }
}

/// Health monitor manages continuous health checking of all modules
pub struct HealthMonitor {
    /// Module registry for getting module info
    registry: Arc<ModuleRegistry>,
    
    /// Event bus for health check communication
    event_bus: Arc<dyn EventBusTrait>,
    
    /// Health check tasks for each module
    health_checkers: DashMap<ModuleId, JoinHandle<()>>,
    
    /// Health status cache
    health_cache: Arc<DashMap<ModuleId, HealthReport>>,
    
    /// Health check configuration
    check_interval: Duration,
    check_timeout: Duration,
    unhealthy_threshold: u32,
    
    /// Monitoring task handle
    monitor_task: Option<JoinHandle<()>>,
}

impl HealthMonitor {
    pub fn new(
        registry: Arc<ModuleRegistry>,
        event_bus: Arc<dyn EventBusTrait>,
        check_interval: Duration,
        check_timeout: Duration,
        unhealthy_threshold: u32,
    ) -> Self {
        Self {
            registry,
            event_bus,
            health_checkers: DashMap::new(),
            health_cache: Arc::new(DashMap::new()),
            check_interval,
            check_timeout,
            unhealthy_threshold,
            monitor_task: None,
        }
    }

    /// Start health monitoring for all registered modules
    pub async fn start_monitoring(&mut self) -> OrchestratorResult<()> {
        info!("Starting health monitoring");

        let modules = self.registry.get_all_modules();
        
        for descriptor in modules {
            if descriptor.id == ModuleId::Orchestrator {
                continue; // Don't monitor ourselves
            }
            
            self.start_module_health_check(descriptor.id).await?;
        }

        // Start the main monitoring task
        let health_cache = Arc::clone(&self.health_cache);
        let registry = Arc::clone(&self.registry);
        let monitor_interval = self.check_interval;

        let monitor_task = tokio::spawn(async move {
            let mut interval = tokio::time::interval(monitor_interval);
            
            loop {
                interval.tick().await;
                
                // Cleanup old issues and update uptime
                for mut entry in health_cache.iter_mut() {
                    let report = entry.value_mut();
                    
                    // Remove issues older than 1 hour
                    let one_hour_ago = Instant::now() - Duration::from_secs(3600);
                    report.issues.retain(|issue| issue.timestamp > one_hour_ago);
                    
                    // Update uptime if module is running
                    if let Some(state) = registry.get_module_state(report.module_id) {
                        if let crate::lifecycle::ModuleState::Running { since } = state {
                            report.uptime = since.elapsed();
                        }
                    }
                }
            }
        });

        self.monitor_task = Some(monitor_task);
        info!("Health monitoring started");
        Ok(())
    }

    /// Stop health monitoring
    pub async fn stop_monitoring(&mut self) {
        info!("Stopping health monitoring");

        // Stop all health check tasks
        for entry in self.health_checkers.iter() {
            entry.value().abort();
        }
        self.health_checkers.clear();

        // Stop main monitoring task
        if let Some(task) = self.monitor_task.take() {
            task.abort();
        }

        info!("Health monitoring stopped");
    }

    /// Start health checking for a specific module
    async fn start_module_health_check(&self, module_id: ModuleId) -> OrchestratorResult<()> {
        debug!("Starting health check for module: {}", module_id);

        // Initialize health report
        let report = HealthReport::new(module_id);
        self.health_cache.insert(module_id, report);

        // Start health check task
        let event_bus = Arc::clone(&self.event_bus);
        let health_cache = Arc::clone(&self.health_cache);
        let check_interval = self.check_interval;
        let check_timeout = self.check_timeout;
        let unhealthy_threshold = self.unhealthy_threshold;

        let task = tokio::spawn(async move {
            let mut interval = tokio::time::interval(check_interval);
            let mut consecutive_failures = 0u32;

            loop {
                interval.tick().await;

                let check_result = Self::perform_health_check(
                    module_id,
                    &event_bus,
                    check_timeout,
                ).await;

                // Update health report
                if let Some(mut entry) = health_cache.get_mut(&module_id) {
                    let report = entry.value_mut();
                    
                    match check_result {
                        Ok(metrics) => {
                            report.update_metrics(metrics);
                            report.status = HealthStatus::Healthy;
                            consecutive_failures = 0;
                        }
                        Err(error) => {
                            consecutive_failures += 1;
                            report.failure_count += 1;
                            report.last_check = Instant::now();
                            report.check_count += 1;

                            if consecutive_failures >= unhealthy_threshold {
                                report.status = HealthStatus::Unhealthy {
                                    reason: error.to_string(),
                                };
                                warn!("Module {} marked as unhealthy: {}", module_id, error);
                            } else {
                                report.add_issue(
                                    IssueSeverity::Medium,
                                    format!("Health check failed: {}", error),
                                );
                            }
                        }
                    }
                }
            }
        });

        self.health_checkers.insert(module_id, task);
        Ok(())
    }

    /// Perform a health check for a module
    async fn perform_health_check(
        module_id: ModuleId,
        event_bus: &Arc<dyn EventBusTrait>,
        timeout: Duration,
    ) -> OrchestratorResult<HealthMetrics> {
        debug!("Performing health check for module: {}", module_id);

        // Create health check request
        let request = HealthCheckRequest {
            module_id,
            timestamp: chrono::Utc::now(),
        };

        let message = BusMessage::new(
            ModuleId::Orchestrator,
            MessagePayload::HealthCheck(request),
        );

        // Send health check request
        match tokio::time::timeout(timeout, event_bus.publish(message)).await {
            Ok(Ok(_)) => {
                // For now, assume success means healthy
                // In a real implementation, we'd wait for a response
                Ok(HealthMetrics {
                    cpu_usage: 5.0,  // Simulated values
                    memory_usage: 1024 * 1024 * 10, // 10MB
                    message_queue_depth: 5,
                    error_rate: 0.01,
                    response_time_ms: 50.0,
                })
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

    /// Get health report for a module
    pub fn get_module_health(&self, module_id: ModuleId) -> Option<HealthReport> {
        self.health_cache.get(&module_id).map(|entry| entry.clone())
    }

    /// Get health reports for all modules
    pub fn get_all_health_reports(&self) -> Vec<HealthReport> {
        self.health_cache.iter().map(|entry| entry.clone()).collect()
    }

    /// Get system-wide health status
    pub fn get_system_health_status(&self) -> HealthStatus {
        let reports = self.get_all_health_reports();
        
        if reports.is_empty() {
            return HealthStatus::Unknown;
        }

        let unhealthy_count = reports.iter()
            .filter(|report| matches!(report.status, HealthStatus::Unhealthy { .. }))
            .count();

        let degraded_count = reports.iter()
            .filter(|report| matches!(report.status, HealthStatus::Degraded { .. }))
            .count();

        if unhealthy_count > 0 {
            let issues: Vec<String> = reports.iter()
                .filter_map(|report| {
                    if let HealthStatus::Unhealthy { reason } = &report.status {
                        Some(format!("{}: {}", report.module_id, reason))
                    } else {
                        None
                    }
                })
                .collect();
            
            HealthStatus::Unhealthy {
                reason: format!("{} modules unhealthy: [{}]", unhealthy_count, issues.join(", ")),
            }
        } else if degraded_count > 0 {
            let issues: Vec<String> = reports.iter()
                .filter_map(|report| {
                    if let HealthStatus::Degraded { issues } = &report.status {
                        Some(format!("{}: [{}]", report.module_id, issues.join(", ")))
                    } else {
                        None
                    }
                })
                .collect();
            
            HealthStatus::Degraded { issues }
        } else {
            HealthStatus::Healthy
        }
    }

    /// Check if a module is healthy
    pub fn is_module_healthy(&self, module_id: ModuleId) -> bool {
        self.get_module_health(module_id)
            .map(|report| report.is_healthy())
            .unwrap_or(false)
    }

    /// Add a health issue for a module (called by other components)
    pub fn report_issue(&self, module_id: ModuleId, severity: IssueSeverity, description: String) {
        if let Some(mut entry) = self.health_cache.get_mut(&module_id) {
            let report = entry.value_mut();
            report.add_issue(severity, description);
            
            // Update status based on issue severity
            match severity {
                IssueSeverity::Critical => {
                    report.status = HealthStatus::Unhealthy {
                        reason: "Critical issue reported".to_string(),
                    };
                }
                IssueSeverity::High | IssueSeverity::Medium => {
                    if let HealthStatus::Healthy = report.status {
                        let issues: Vec<String> = report.issues.iter()
                            .map(|issue| issue.description.clone())
                            .collect();
                        report.status = HealthStatus::Degraded { issues };
                    }
                }
                IssueSeverity::Low => {
                    // Don't change status for low severity issues
                }
            }
        }
    }
}