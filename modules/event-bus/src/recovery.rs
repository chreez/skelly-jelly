//! Automatic Recovery Strategies with Escalation Paths
//!
//! Provides comprehensive recovery mechanisms for handling system failures
//! with intelligent escalation and automated resolution strategies.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn, error};
use uuid::Uuid;

use crate::{
    ModuleId, EventBusError, EventBusResult,
    circuit_breaker::{CircuitBreaker, CircuitBreakerConfig, CircuitBreakerRegistry},
    retry::{RetryExecutor, RetryConfig, RetryResult},
    dead_letter_queue::{DeadLetterQueue, DeadLetterReason},
    error_logging::{ErrorLogger, ErrorContext, ErrorSeverity, ErrorCategory, CorrelationId},
};

/// Unique identifier for recovery incidents
pub type IncidentId = Uuid;

/// Recovery strategy types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RecoveryStrategy {
    /// Retry the operation with exponential backoff
    Retry { config: RetryConfig },
    
    /// Switch to a backup service or endpoint
    Failover { backup_target: String },
    
    /// Gracefully degrade functionality
    GracefulDegradation { fallback_mode: String },
    
    /// Reset circuit breaker to attempt recovery
    CircuitBreakerReset { circuit_name: String },
    
    /// Restart module or component
    ModuleRestart { module_id: ModuleId },
    
    /// Clear and rebuild caches
    CacheClear { cache_names: Vec<String> },
    
    /// Route traffic to alternative path
    TrafficRerouting { alternative_path: String },
    
    /// Scale resources up or down
    ResourceScaling { scale_factor: f64 },
    
    /// Custom recovery action
    Custom { action_name: String, parameters: HashMap<String, serde_json::Value> },
}

/// Escalation levels for recovery
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum EscalationLevel {
    /// Level 0: Automatic retry and simple recovery
    Automatic = 0,
    
    /// Level 1: Component-level recovery actions
    Component = 1,
    
    /// Level 2: Service-level recovery actions
    Service = 2,
    
    /// Level 3: System-level recovery actions
    System = 3,
    
    /// Level 4: Manual intervention required
    Manual = 4,
    
    /// Level 5: Emergency escalation
    Emergency = 5,
}

/// Recovery action configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryAction {
    /// Unique identifier for this recovery action
    pub id: Uuid,
    
    /// Name of the recovery action
    pub name: String,
    
    /// Description of what this action does
    pub description: String,
    
    /// Recovery strategy to execute
    pub strategy: RecoveryStrategy,
    
    /// Escalation level for this action
    pub escalation_level: EscalationLevel,
    
    /// Conditions that must be met to execute this action
    pub conditions: Vec<RecoveryCondition>,
    
    /// Maximum number of times this action can be executed
    pub max_executions: u32,
    
    /// Cooldown period between executions
    pub cooldown: Duration,
    
    /// Whether this action requires confirmation before execution
    pub requires_confirmation: bool,
    
    /// Expected recovery time for this action
    pub expected_recovery_time: Duration,
    
    /// Success rate threshold to consider this action successful
    pub success_threshold: f64,
}

/// Conditions for executing recovery actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecoveryCondition {
    /// Error rate exceeds threshold
    ErrorRateExceeds { threshold: f64, window: Duration },
    
    /// Response time exceeds threshold
    ResponseTimeExceeds { threshold: Duration },
    
    /// Circuit breaker is open
    CircuitBreakerOpen { circuit_name: String },
    
    /// Module is unhealthy
    ModuleUnhealthy { module_id: ModuleId },
    
    /// Resource utilization exceeds threshold
    ResourceUtilizationExceeds { resource: String, threshold: f64 },
    
    /// Specific error type occurred
    ErrorTypeMatches { error_types: Vec<String> },
    
    /// Time-based condition
    TimeWindow { start_hour: u8, end_hour: u8 },
    
    /// Dependency is unavailable
    DependencyUnavailable { dependency: String },
    
    /// Custom condition
    Custom { condition_name: String, parameters: HashMap<String, serde_json::Value> },
}

/// Recovery incident tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryIncident {
    /// Unique incident identifier
    pub id: IncidentId,
    
    /// Correlation ID for tracking related events
    pub correlation_id: CorrelationId,
    
    /// When the incident was detected
    pub detected_at: SystemTime,
    
    /// Module where the incident occurred
    pub module_id: ModuleId,
    
    /// Description of the incident
    pub description: String,
    
    /// Severity of the incident
    pub severity: ErrorSeverity,
    
    /// Current escalation level
    pub escalation_level: EscalationLevel,
    
    /// Recovery actions that have been attempted
    pub attempted_actions: Vec<RecoveryActionResult>,
    
    /// Current status of the incident
    pub status: IncidentStatus,
    
    /// When the incident was resolved (if applicable)
    pub resolved_at: Option<SystemTime>,
    
    /// Resolution details
    pub resolution: Option<String>,
    
    /// Metrics and metadata about the incident
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Status of a recovery incident
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IncidentStatus {
    /// Incident detected and being analyzed
    Detected,
    
    /// Recovery actions are being executed
    Recovering,
    
    /// Waiting for manual intervention
    AwaitingManualIntervention,
    
    /// Incident has been resolved
    Resolved,
    
    /// Recovery failed, incident escalated
    Escalated,
    
    /// Incident closed without resolution
    Closed,
}

/// Result of executing a recovery action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryActionResult {
    /// The action that was executed
    pub action_id: Uuid,
    
    /// When the action was executed
    pub executed_at: SystemTime,
    
    /// Whether the action was successful
    pub success: bool,
    
    /// Duration of the action execution
    pub execution_duration: Duration,
    
    /// Error message if the action failed
    pub error_message: Option<String>,
    
    /// Metrics from the action execution
    pub metrics: HashMap<String, f64>,
    
    /// Any side effects or notes from the action
    pub notes: Option<String>,
}

/// Configuration for the recovery system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryConfig {
    /// Whether automatic recovery is enabled
    pub enable_automatic_recovery: bool,
    
    /// Maximum escalation level to attempt automatically
    pub max_automatic_escalation_level: EscalationLevel,
    
    /// Default timeout for recovery actions
    pub default_action_timeout: Duration,
    
    /// Maximum number of concurrent recovery actions
    pub max_concurrent_actions: usize,
    
    /// Interval for checking recovery conditions
    pub condition_check_interval: Duration,
    
    /// Whether to enable recovery metrics
    pub enable_recovery_metrics: bool,
    
    /// Notification settings for escalation
    pub notification_config: NotificationConfig,
}

/// Configuration for notifications during recovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig {
    /// Whether to send notifications
    pub enabled: bool,
    
    /// Escalation levels that trigger notifications
    pub notification_levels: Vec<EscalationLevel>,
    
    /// Webhook URLs for notifications
    pub webhook_urls: Vec<String>,
    
    /// Email addresses for notifications
    pub email_addresses: Vec<String>,
    
    /// Slack channels for notifications
    pub slack_channels: Vec<String>,
}

impl Default for RecoveryConfig {
    fn default() -> Self {
        Self {
            enable_automatic_recovery: true,
            max_automatic_escalation_level: EscalationLevel::Service,
            default_action_timeout: Duration::from_secs(300), // 5 minutes
            max_concurrent_actions: 5,
            condition_check_interval: Duration::from_secs(30),
            enable_recovery_metrics: true,
            notification_config: NotificationConfig {
                enabled: true,
                notification_levels: vec![EscalationLevel::System, EscalationLevel::Manual, EscalationLevel::Emergency],
                webhook_urls: vec![],
                email_addresses: vec![],
                slack_channels: vec![],
            },
        }
    }
}

/// Trait for implementing custom recovery actions
#[async_trait]
pub trait RecoveryActionExecutor: Send + Sync {
    /// Execute a recovery action
    async fn execute_action(
        &self,
        action: &RecoveryAction,
        incident: &RecoveryIncident,
    ) -> Result<RecoveryActionResult, Box<dyn std::error::Error + Send + Sync>>;
    
    /// Check if this executor can handle the given action
    fn can_handle(&self, action: &RecoveryAction) -> bool;
    
    /// Get the name of this executor
    fn name(&self) -> &str;
}

/// Statistics about recovery operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryStats {
    pub total_incidents: u64,
    pub resolved_incidents: u64,
    pub escalated_incidents: u64,
    pub incidents_by_module: HashMap<ModuleId, u64>,
    pub incidents_by_escalation_level: HashMap<String, u64>,
    pub average_resolution_time: Duration,
    pub success_rate_by_action: HashMap<String, f64>,
    pub most_common_errors: Vec<(String, u64)>,
    pub recovery_actions_executed: u64,
    pub successful_recoveries: u64,
}

/// Main recovery system implementation
pub struct RecoverySystem {
    config: RecoveryConfig,
    actions: Arc<parking_lot::RwLock<Vec<RecoveryAction>>>,
    incidents: Arc<parking_lot::RwLock<HashMap<IncidentId, RecoveryIncident>>>,
    executors: Arc<parking_lot::RwLock<Vec<Arc<dyn RecoveryActionExecutor>>>>,
    circuit_breakers: Arc<CircuitBreakerRegistry>,
    retry_executor: Arc<RetryExecutor>,
    dead_letter_queue: Arc<DeadLetterQueue>,
    error_logger: Arc<ErrorLogger>,
    stats: Arc<parking_lot::RwLock<RecoveryStats>>,
}

impl RecoverySystem {
    /// Create a new recovery system
    pub fn new(
        config: RecoveryConfig,
        circuit_breakers: Arc<CircuitBreakerRegistry>,
        retry_executor: Arc<RetryExecutor>,
        dead_letter_queue: Arc<DeadLetterQueue>,
        error_logger: Arc<ErrorLogger>,
    ) -> Self {
        let stats = RecoveryStats {
            total_incidents: 0,
            resolved_incidents: 0,
            escalated_incidents: 0,
            incidents_by_module: HashMap::new(),
            incidents_by_escalation_level: HashMap::new(),
            average_resolution_time: Duration::from_secs(0),
            success_rate_by_action: HashMap::new(),
            most_common_errors: vec![],
            recovery_actions_executed: 0,
            successful_recoveries: 0,
        };

        Self {
            config,
            actions: Arc::new(parking_lot::RwLock::new(Vec::new())),
            incidents: Arc::new(parking_lot::RwLock::new(HashMap::new())),
            executors: Arc::new(parking_lot::RwLock::new(Vec::new())),
            circuit_breakers,
            retry_executor,
            dead_letter_queue,
            error_logger,
            stats: Arc::new(parking_lot::RwLock::new(stats)),
        }
    }

    /// Register a recovery action
    pub fn register_action(&self, action: RecoveryAction) {
        info!("Registering recovery action: {} (level: {:?})", action.name, action.escalation_level);
        self.actions.write().push(action);
    }

    /// Register a recovery action executor
    pub fn register_executor(&self, executor: Arc<dyn RecoveryActionExecutor>) {
        info!("Registering recovery executor: {}", executor.name());
        self.executors.write().push(executor);
    }

    /// Detect and handle an incident
    pub async fn handle_incident(
        &self,
        correlation_id: CorrelationId,
        module_id: ModuleId,
        error: &EventBusError,
        description: String,
    ) -> EventBusResult<IncidentId> {
        let severity = self.classify_error_severity(error);
        let incident_id = Uuid::new_v4();

        let incident = RecoveryIncident {
            id: incident_id,
            correlation_id,
            detected_at: SystemTime::now(),
            module_id,
            description: description.clone(),
            severity,
            escalation_level: EscalationLevel::Automatic,
            attempted_actions: vec![],
            status: IncidentStatus::Detected,
            resolved_at: None,
            resolution: None,
            metadata: HashMap::new(),
        };

        info!("Detected incident {} in module {:?}: {}", incident_id, module_id, description);

        // Store the incident
        self.incidents.write().insert(incident_id, incident.clone());

        // Update statistics
        self.update_stats_on_incident(&incident);

        // Log the incident
        let error_context = ErrorContext::new(
            correlation_id,
            module_id,
            "incident_detected".to_string(),
            severity,
            ErrorCategory::Unknown,
            description,
        );
        self.error_logger.log_error(&error_context);

        // Start recovery process if automatic recovery is enabled
        if self.config.enable_automatic_recovery {
            tokio::spawn({
                let recovery_system = self.clone();
                async move {
                    if let Err(e) = recovery_system.execute_recovery(incident_id).await {
                        error!("Recovery execution failed for incident {}: {}", incident_id, e);
                    }
                }
            });
        }

        Ok(incident_id)
    }

    /// Execute recovery actions for an incident
    pub async fn execute_recovery(&self, incident_id: IncidentId) -> EventBusResult<()> {
        let mut incident = {
            let incidents = self.incidents.read();
            match incidents.get(&incident_id) {
                Some(incident) => incident.clone(),
                None => return Err(EventBusError::Internal(format!("Incident {} not found", incident_id))),
            }
        };

        info!("Starting recovery for incident {}", incident_id);
        
        // Update incident status
        incident.status = IncidentStatus::Recovering;
        self.update_incident(incident_id, incident.clone());

        let mut current_escalation_level = EscalationLevel::Automatic;

        // Try recovery actions at each escalation level
        while current_escalation_level <= self.config.max_automatic_escalation_level {
            debug!("Attempting recovery at escalation level {:?} for incident {}", current_escalation_level, incident_id);

            let applicable_actions = self.get_applicable_actions(&incident, current_escalation_level);
            
            if applicable_actions.is_empty() {
                warn!("No applicable recovery actions found for escalation level {:?}", current_escalation_level);
                current_escalation_level = self.next_escalation_level(current_escalation_level);
                continue;
            }

            let mut recovery_successful = false;

            for action in applicable_actions {
                if !self.can_execute_action(&action, &incident) {
                    debug!("Skipping action {} due to execution constraints", action.name);
                    continue;
                }

                info!("Executing recovery action: {} for incident {}", action.name, incident_id);

                match self.execute_action(&action, &incident).await {
                    Ok(result) => {
                        incident.attempted_actions.push(result.clone());
                        
                        if result.success {
                            info!("Recovery action {} succeeded for incident {}", action.name, incident_id);
                            
                            // Check if the incident is now resolved
                            if self.is_incident_resolved(&incident).await {
                                incident.status = IncidentStatus::Resolved;
                                incident.resolved_at = Some(SystemTime::now());
                                incident.resolution = Some(format!("Resolved by action: {}", action.name));
                                
                                info!("Incident {} resolved successfully", incident_id);
                                self.update_stats_on_resolution(&incident);
                                self.update_incident(incident_id, incident);
                                return Ok(());
                            }
                            
                            recovery_successful = true;
                        } else {
                            warn!("Recovery action {} failed for incident {}: {}", 
                                  action.name, incident_id, result.error_message.unwrap_or_default());
                        }
                    }
                    Err(e) => {
                        error!("Failed to execute recovery action {} for incident {}: {}", action.name, incident_id, e);
                    }
                }

                self.update_incident(incident_id, incident.clone());
            }

            if recovery_successful {
                // Give some time for the system to stabilize
                tokio::time::sleep(Duration::from_secs(30)).await;
                
                // Check if incident is resolved after stabilization
                if self.is_incident_resolved(&incident).await {
                    incident.status = IncidentStatus::Resolved;
                    incident.resolved_at = Some(SystemTime::now());
                    incident.resolution = Some("Resolved after recovery actions".to_string());
                    
                    info!("Incident {} resolved after stabilization", incident_id);
                    self.update_stats_on_resolution(&incident);
                    self.update_incident(incident_id, incident);
                    return Ok(());
                }
            }

            // Escalate to the next level
            current_escalation_level = self.next_escalation_level(current_escalation_level);
        }

        // If we reach here, automatic recovery has failed
        warn!("Automatic recovery failed for incident {}, escalating to manual intervention", incident_id);
        
        incident.status = IncidentStatus::AwaitingManualIntervention;
        incident.escalation_level = EscalationLevel::Manual;
        self.update_incident(incident_id, incident.clone());
        self.update_stats_on_escalation(&incident);

        // Send notifications for manual intervention
        self.send_escalation_notification(&incident).await;

        Ok(())
    }

    /// Get applicable recovery actions for an incident at a specific escalation level
    fn get_applicable_actions(&self, incident: &RecoveryIncident, escalation_level: EscalationLevel) -> Vec<RecoveryAction> {
        let actions = self.actions.read();
        
        actions
            .iter()
            .filter(|action| {
                action.escalation_level == escalation_level &&
                self.conditions_met(action, incident)
            })
            .cloned()
            .collect()
    }

    /// Check if conditions are met for executing an action
    fn conditions_met(&self, action: &RecoveryAction, incident: &RecoveryIncident) -> bool {
        for condition in &action.conditions {
            if !self.evaluate_condition(condition, incident) {
                return false;
            }
        }
        true
    }

    /// Evaluate a specific recovery condition
    fn evaluate_condition(&self, condition: &RecoveryCondition, incident: &RecoveryIncident) -> bool {
        match condition {
            RecoveryCondition::ErrorRateExceeds { threshold, window } => {
                // TODO: Implement error rate checking logic
                true // Placeholder
            }
            RecoveryCondition::ResponseTimeExceeds { threshold } => {
                // TODO: Implement response time checking logic
                true // Placeholder
            }
            RecoveryCondition::CircuitBreakerOpen { circuit_name } => {
                if let Some(breaker) = self.circuit_breakers.get(circuit_name) {
                    !breaker.is_healthy()
                } else {
                    false
                }
            }
            RecoveryCondition::ModuleUnhealthy { module_id } => {
                // TODO: Implement module health checking logic
                incident.module_id == *module_id
            }
            RecoveryCondition::ErrorTypeMatches { error_types } => {
                // TODO: Implement error type matching logic
                true // Placeholder
            }
            _ => true, // Default to allowing execution for unimplemented conditions
        }
    }

    /// Check if an action can be executed considering constraints
    fn can_execute_action(&self, action: &RecoveryAction, incident: &RecoveryIncident) -> bool {
        // Check execution count
        let execution_count = incident.attempted_actions
            .iter()
            .filter(|result| result.action_id == action.id)
            .count() as u32;

        if execution_count >= action.max_executions {
            return false;
        }

        // Check cooldown period
        if let Some(last_execution) = incident.attempted_actions
            .iter()
            .filter(|result| result.action_id == action.id)
            .map(|result| result.executed_at)
            .max()
        {
            let elapsed = SystemTime::now().duration_since(last_execution).unwrap_or_default();
            if elapsed < action.cooldown {
                return false;
            }
        }

        true
    }

    /// Execute a specific recovery action
    async fn execute_action(
        &self,
        action: &RecoveryAction,
        incident: &RecoveryIncident,
    ) -> Result<RecoveryActionResult, Box<dyn std::error::Error + Send + Sync>> {
        let start_time = SystemTime::now();
        let execution_start = Instant::now();

        // Find an executor for this action
        let executor = {
            let executors = self.executors.read();
            executors
                .iter()
                .find(|e| e.can_handle(action))
                .cloned()
                .ok_or("No executor found for action")?
        };

        // Execute the action with timeout
        let result = tokio::time::timeout(
            self.config.default_action_timeout,
            executor.execute_action(action, incident)
        ).await;

        let execution_duration = execution_start.elapsed();

        match result {
            Ok(Ok(mut action_result)) => {
                action_result.executed_at = start_time;
                action_result.execution_duration = execution_duration;
                
                // Update statistics
                self.update_stats_on_action_execution(&action_result, true);
                
                Ok(action_result)
            }
            Ok(Err(e)) => {
                let action_result = RecoveryActionResult {
                    action_id: action.id,
                    executed_at: start_time,
                    success: false,
                    execution_duration,
                    error_message: Some(e.to_string()),
                    metrics: HashMap::new(),
                    notes: None,
                };
                
                // Update statistics
                self.update_stats_on_action_execution(&action_result, false);
                
                Ok(action_result)
            }
            Err(_) => {
                let action_result = RecoveryActionResult {
                    action_id: action.id,
                    executed_at: start_time,
                    success: false,
                    execution_duration,
                    error_message: Some("Action execution timed out".to_string()),
                    metrics: HashMap::new(),
                    notes: None,
                };
                
                // Update statistics
                self.update_stats_on_action_execution(&action_result, false);
                
                Ok(action_result)
            }
        }
    }

    /// Check if an incident is resolved
    async fn is_incident_resolved(&self, incident: &RecoveryIncident) -> bool {
        // TODO: Implement incident resolution checking logic
        // This could involve health checks, error rate monitoring, etc.
        false // Placeholder - for demo, incidents don't auto-resolve
    }

    /// Send escalation notification
    async fn send_escalation_notification(&self, incident: &RecoveryIncident) {
        if !self.config.notification_config.enabled {
            return;
        }

        if !self.config.notification_config.notification_levels.contains(&incident.escalation_level) {
            return;
        }

        info!("Sending escalation notification for incident {}", incident.id);
        
        // TODO: Implement actual notification sending (webhook, email, Slack, etc.)
        // For now, just log the notification
        warn!("ESCALATION NOTIFICATION: Incident {} requires manual intervention", incident.id);
    }

    /// Update incident in storage
    fn update_incident(&self, incident_id: IncidentId, incident: RecoveryIncident) {
        self.incidents.write().insert(incident_id, incident);
    }

    /// Get next escalation level
    fn next_escalation_level(&self, current: EscalationLevel) -> EscalationLevel {
        match current {
            EscalationLevel::Automatic => EscalationLevel::Component,
            EscalationLevel::Component => EscalationLevel::Service,
            EscalationLevel::Service => EscalationLevel::System,
            EscalationLevel::System => EscalationLevel::Manual,
            EscalationLevel::Manual => EscalationLevel::Emergency,
            EscalationLevel::Emergency => EscalationLevel::Emergency, // Max level
        }
    }

    /// Classify error severity
    fn classify_error_severity(&self, error: &EventBusError) -> ErrorSeverity {
        match error {
            EventBusError::QueueFull { .. } => ErrorSeverity::Critical,
            EventBusError::BusShuttingDown => ErrorSeverity::Warning,
            EventBusError::DeliveryTimeout { .. } => ErrorSeverity::Warning,
            EventBusError::Internal(_) => ErrorSeverity::Critical,
            _ => ErrorSeverity::Error,
        }
    }

    /// Update statistics on new incident
    fn update_stats_on_incident(&self, incident: &RecoveryIncident) {
        let mut stats = self.stats.write();
        stats.total_incidents += 1;
        *stats.incidents_by_module.entry(incident.module_id).or_insert(0) += 1;
        
        let level_key = format!("{:?}", incident.escalation_level);
        *stats.incidents_by_escalation_level.entry(level_key).or_insert(0) += 1;
    }

    /// Update statistics on incident resolution
    fn update_stats_on_resolution(&self, incident: &RecoveryIncident) {
        let mut stats = self.stats.write();
        stats.resolved_incidents += 1;
        stats.successful_recoveries += 1;
    }

    /// Update statistics on incident escalation
    fn update_stats_on_escalation(&self, incident: &RecoveryIncident) {
        let mut stats = self.stats.write();
        stats.escalated_incidents += 1;
    }

    /// Update statistics on action execution
    fn update_stats_on_action_execution(&self, result: &RecoveryActionResult, success: bool) {
        let mut stats = self.stats.write();
        stats.recovery_actions_executed += 1;
        
        if success {
            stats.successful_recoveries += 1;
        }
    }

    /// Get current statistics
    pub fn stats(&self) -> RecoveryStats {
        self.stats.read().clone()
    }

    /// Get incident by ID
    pub fn get_incident(&self, incident_id: IncidentId) -> Option<RecoveryIncident> {
        self.incidents.read().get(&incident_id).cloned()
    }

    /// Get all incidents
    pub fn get_all_incidents(&self) -> Vec<RecoveryIncident> {
        self.incidents.read().values().cloned().collect()
    }
}

impl Clone for RecoverySystem {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            actions: self.actions.clone(),
            incidents: self.incidents.clone(),
            executors: self.executors.clone(),
            circuit_breakers: self.circuit_breakers.clone(),
            retry_executor: self.retry_executor.clone(),
            dead_letter_queue: self.dead_letter_queue.clone(),
            error_logger: self.error_logger.clone(),
            stats: self.stats.clone(),
        }
    }
}

/// Built-in recovery action executor for common strategies
pub struct DefaultRecoveryExecutor {
    circuit_breakers: Arc<CircuitBreakerRegistry>,
    retry_executor: Arc<RetryExecutor>,
}

impl DefaultRecoveryExecutor {
    pub fn new(
        circuit_breakers: Arc<CircuitBreakerRegistry>,
        retry_executor: Arc<RetryExecutor>,
    ) -> Self {
        Self {
            circuit_breakers,
            retry_executor,
        }
    }
}

#[async_trait]
impl RecoveryActionExecutor for DefaultRecoveryExecutor {
    async fn execute_action(
        &self,
        action: &RecoveryAction,
        incident: &RecoveryIncident,
    ) -> Result<RecoveryActionResult, Box<dyn std::error::Error + Send + Sync>> {
        let start_time = SystemTime::now();
        let mut metrics = HashMap::new();

        let success = match &action.strategy {
            RecoveryStrategy::CircuitBreakerReset { circuit_name } => {
                if let Some(breaker) = self.circuit_breakers.get(circuit_name) {
                    breaker.force_close();
                    info!("Reset circuit breaker: {}", circuit_name);
                    true
                } else {
                    false
                }
            }
            RecoveryStrategy::GracefulDegradation { fallback_mode } => {
                info!("Activating graceful degradation mode: {}", fallback_mode);
                // TODO: Implement graceful degradation logic
                true
            }
            RecoveryStrategy::Custom { action_name, parameters } => {
                info!("Executing custom recovery action: {}", action_name);
                // TODO: Implement custom action execution
                true
            }
            _ => {
                warn!("Recovery strategy not implemented: {:?}", action.strategy);
                false
            }
        };

        Ok(RecoveryActionResult {
            action_id: action.id,
            executed_at: start_time,
            success,
            execution_duration: SystemTime::now().duration_since(start_time).unwrap_or_default(),
            error_message: if success { None } else { Some("Strategy not implemented".to_string()) },
            metrics,
            notes: Some(format!("Executed strategy: {:?}", action.strategy)),
        })
    }

    fn can_handle(&self, action: &RecoveryAction) -> bool {
        matches!(
            action.strategy,
            RecoveryStrategy::CircuitBreakerReset { .. } |
            RecoveryStrategy::GracefulDegradation { .. } |
            RecoveryStrategy::Custom { .. }
        )
    }

    fn name(&self) -> &str {
        "DefaultRecoveryExecutor"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        circuit_breaker::CircuitBreakerRegistry,
        retry::create_retry_executor,
        dead_letter_queue::create_dead_letter_queue,
        error_logging::create_error_logger,
    };

    fn create_test_recovery_system() -> RecoverySystem {
        let circuit_breakers = Arc::new(CircuitBreakerRegistry::new());
        let retry_executor = Arc::new(create_retry_executor().unwrap());
        let dead_letter_queue = Arc::new(create_dead_letter_queue());
        let error_logger = Arc::new(create_error_logger());

        RecoverySystem::new(
            RecoveryConfig::default(),
            circuit_breakers,
            retry_executor,
            dead_letter_queue,
            error_logger,
        )
    }

    #[tokio::test]
    async fn test_incident_detection() {
        let recovery_system = create_test_recovery_system();
        let correlation_id = Uuid::new_v4();
        let error = EventBusError::QueueFull { current_size: 100, max_size: 100 };

        let incident_id = recovery_system.handle_incident(
            correlation_id,
            ModuleId::EventBus,
            &error,
            "Test incident".to_string(),
        ).await.unwrap();

        let incident = recovery_system.get_incident(incident_id).unwrap();
        assert_eq!(incident.correlation_id, correlation_id);
        assert_eq!(incident.module_id, ModuleId::EventBus);
        assert_eq!(incident.description, "Test incident");
        assert_eq!(incident.severity, ErrorSeverity::Critical);
    }

    #[test]
    fn test_recovery_action_registration() {
        let recovery_system = create_test_recovery_system();

        let action = RecoveryAction {
            id: Uuid::new_v4(),
            name: "Test Action".to_string(),
            description: "Test recovery action".to_string(),
            strategy: RecoveryStrategy::CircuitBreakerReset { circuit_name: "test".to_string() },
            escalation_level: EscalationLevel::Automatic,
            conditions: vec![],
            max_executions: 3,
            cooldown: Duration::from_secs(60),
            requires_confirmation: false,
            expected_recovery_time: Duration::from_secs(30),
            success_threshold: 0.8,
        };

        recovery_system.register_action(action.clone());

        let actions = recovery_system.actions.read();
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].name, "Test Action");
    }

    #[tokio::test]
    async fn test_default_recovery_executor() {
        let circuit_breakers = Arc::new(CircuitBreakerRegistry::new());
        let retry_executor = Arc::new(create_retry_executor().unwrap());
        
        let executor = DefaultRecoveryExecutor::new(circuit_breakers.clone(), retry_executor);

        let action = RecoveryAction {
            id: Uuid::new_v4(),
            name: "Circuit Breaker Reset".to_string(),
            description: "Reset circuit breaker".to_string(),
            strategy: RecoveryStrategy::CircuitBreakerReset { circuit_name: "test".to_string() },
            escalation_level: EscalationLevel::Automatic,
            conditions: vec![],
            max_executions: 3,
            cooldown: Duration::from_secs(60),
            requires_confirmation: false,
            expected_recovery_time: Duration::from_secs(30),
            success_threshold: 0.8,
        };

        // Register a circuit breaker
        circuit_breakers.register("test".to_string(), CircuitBreakerConfig::default());

        let incident = RecoveryIncident {
            id: Uuid::new_v4(),
            correlation_id: Uuid::new_v4(),
            detected_at: SystemTime::now(),
            module_id: ModuleId::EventBus,
            description: "Test incident".to_string(),
            severity: ErrorSeverity::Error,
            escalation_level: EscalationLevel::Automatic,
            attempted_actions: vec![],
            status: IncidentStatus::Detected,
            resolved_at: None,
            resolution: None,
            metadata: HashMap::new(),
        };

        assert!(executor.can_handle(&action));
        
        let result = executor.execute_action(&action, &incident).await.unwrap();
        assert!(result.success);
    }

    #[test]
    fn test_escalation_levels() {
        let recovery_system = create_test_recovery_system();
        
        assert!(EscalationLevel::Automatic < EscalationLevel::Component);
        assert!(EscalationLevel::Component < EscalationLevel::Service);
        assert!(EscalationLevel::Service < EscalationLevel::System);
        assert!(EscalationLevel::System < EscalationLevel::Manual);
        assert!(EscalationLevel::Manual < EscalationLevel::Emergency);
        
        assert_eq!(recovery_system.next_escalation_level(EscalationLevel::Automatic), EscalationLevel::Component);
        assert_eq!(recovery_system.next_escalation_level(EscalationLevel::Emergency), EscalationLevel::Emergency);
    }
}