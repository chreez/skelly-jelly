//! Recovery strategies and failure handling

use crate::error::{OrchestratorError, OrchestratorResult};
use crate::lifecycle::LifecycleController;
use skelly_jelly_event_bus::ModuleId;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::Mutex;
use tracing::{info, warn, error, debug};
use rand;

/// Recovery strategies for different types of failures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecoveryStrategy {
    /// Simple restart with exponential backoff
    Restart {
        max_attempts: u32,
        initial_delay: Duration,
        max_delay: Duration,
        backoff_multiplier: f32,
    },
    
    /// Restart with state reset
    RestartWithReset {
        max_attempts: u32,
        reset_state: bool,
        clear_queues: bool,
        initial_delay: Duration,
    },
    
    /// Fallback to degraded mode
    DegradedMode {
        disable_features: Vec<String>,
        reduce_load: f32,
        timeout: Duration,
    },
    
    /// Complete system restart
    SystemRestart {
        save_state: bool,
        notify_user: bool,
        grace_period: Duration,
    },
    
    /// No automatic recovery
    Manual {
        notify_admin: bool,
        block_system: bool,
    },
}

impl Default for RecoveryStrategy {
    fn default() -> Self {
        RecoveryStrategy::Restart {
            max_attempts: 3,
            initial_delay: Duration::from_secs(5),
            max_delay: Duration::from_secs(300),
            backoff_multiplier: 2.0,
        }
    }
}

/// Type of module failure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FailureType {
    Crash,
    Timeout,
    ResourceExhaustion,
    HealthCheckFailure,
    DependencyFailure,
    ConfigurationError,
    CommunicationFailure,
    UnknownError,
}

/// Module failure information
#[derive(Debug, Clone)]
pub struct ModuleFailure {
    pub module_id: ModuleId,
    pub failure_type: FailureType,
    pub error_message: String,
    pub timestamp: Instant,
    pub context: Option<String>,
    pub previous_state: Option<String>,
}

impl ModuleFailure {
    pub fn new(
        module_id: ModuleId,
        failure_type: FailureType,
        error_message: String,
    ) -> Self {
        Self {
            module_id,
            failure_type,
            error_message,
            timestamp: Instant::now(),
            context: None,
            previous_state: None,
        }
    }

    pub fn with_context(mut self, context: String) -> Self {
        self.context = Some(context);
        self
    }

    pub fn with_previous_state(mut self, state: String) -> Self {
        self.previous_state = Some(state);
        self
    }

    pub fn severity(&self) -> RecoverySeverity {
        match self.failure_type {
            FailureType::Crash | FailureType::ResourceExhaustion => RecoverySeverity::High,
            FailureType::Timeout | FailureType::HealthCheckFailure => RecoverySeverity::Medium,
            FailureType::DependencyFailure | FailureType::CommunicationFailure => RecoverySeverity::Medium,
            FailureType::ConfigurationError => RecoverySeverity::Low,
            FailureType::UnknownError => RecoverySeverity::Medium,
        }
    }
}

/// Severity level for recovery operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RecoverySeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Recovery attempt record
#[derive(Debug, Clone)]
pub struct RecoveryAttempt {
    pub attempt_number: u32,
    pub strategy: RecoveryStrategy,
    pub started_at: Instant,
    pub completed_at: Option<Instant>,
    pub success: Option<bool>,
    pub error: Option<String>,
}

impl RecoveryAttempt {
    pub fn new(attempt_number: u32, strategy: RecoveryStrategy) -> Self {
        Self {
            attempt_number,
            strategy,
            started_at: Instant::now(),
            completed_at: None,
            success: None,
            error: None,
        }
    }

    pub fn complete_success(&mut self) {
        self.completed_at = Some(Instant::now());
        self.success = Some(true);
    }

    pub fn complete_failure(&mut self, error: String) {
        self.completed_at = Some(Instant::now());
        self.success = Some(false);
        self.error = Some(error);
    }

    pub fn duration(&self) -> Option<Duration> {
        self.completed_at.map(|end| end.duration_since(self.started_at))
    }
}

/// Recovery history for a module
#[derive(Debug, Clone)]
pub struct RecoveryHistory {
    pub module_id: ModuleId,
    pub failures: Vec<ModuleFailure>,
    pub attempts: Vec<RecoveryAttempt>,
    pub last_success: Option<Instant>,
    pub consecutive_failures: u32,
    pub total_failures: u32,
}

impl RecoveryHistory {
    pub fn new(module_id: ModuleId) -> Self {
        Self {
            module_id,
            failures: Vec::new(),
            attempts: Vec::new(),
            last_success: None,
            consecutive_failures: 0,
            total_failures: 0,
        }
    }

    pub fn add_failure(&mut self, failure: ModuleFailure) {
        self.failures.push(failure);
        self.consecutive_failures += 1;
        self.total_failures += 1;
    }

    pub fn add_attempt(&mut self, attempt: RecoveryAttempt) {
        self.attempts.push(attempt);
    }

    pub fn mark_success(&mut self) {
        self.last_success = Some(Instant::now());
        self.consecutive_failures = 0;
    }

    pub fn get_recent_failures(&self, duration: Duration) -> Vec<&ModuleFailure> {
        let cutoff = Instant::now() - duration;
        self.failures
            .iter()
            .filter(|failure| failure.timestamp > cutoff)
            .collect()
    }

    pub fn get_attempts(&self) -> u32 {
        self.attempts.len() as u32
    }

    pub fn should_escalate(&self) -> bool {
        self.consecutive_failures > 5 || 
        self.get_recent_failures(Duration::from_secs(3600)).len() > 10
    }
}

/// Exponential backoff calculator
#[derive(Debug, Clone)]
pub struct ExponentialBackoff {
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub multiplier: f32,
    pub jitter: bool,
}

impl ExponentialBackoff {
    pub fn new(initial_delay: Duration, max_delay: Duration, multiplier: f32) -> Self {
        Self {
            initial_delay,
            max_delay,
            multiplier,
            jitter: true,
        }
    }

    pub fn calculate(&self, attempt: u32) -> Duration {
        let delay_ms = self.initial_delay.as_millis() as f32 * self.multiplier.powi(attempt as i32);
        let delay = Duration::from_millis(delay_ms.min(self.max_delay.as_millis() as f32) as u64);
        
        if self.jitter {
            // Add ±25% jitter to prevent thundering herd
            let jitter_factor = 0.75 + (rand::random::<f32>() * 0.5); // 0.75 to 1.25
            Duration::from_millis((delay.as_millis() as f32 * jitter_factor) as u64)
        } else {
            delay
        }
    }
}

/// Recovery manager handles module failure recovery
pub struct RecoveryManager {
    /// Recovery strategies per module
    strategies: HashMap<ModuleId, RecoveryStrategy>,
    
    /// Recovery history tracking
    recovery_history: Arc<Mutex<HashMap<ModuleId, RecoveryHistory>>>,
    
    /// Lifecycle controller for restarting modules
    lifecycle_controller: Arc<LifecycleController>,
    
    /// Global recovery settings
    global_settings: RecoverySettings,
}

#[derive(Debug, Clone)]
pub struct RecoverySettings {
    pub max_concurrent_recoveries: usize,
    pub recovery_timeout: Duration,
    pub escalation_threshold: u32,
    pub cooldown_period: Duration,
}

impl Default for RecoverySettings {
    fn default() -> Self {
        Self {
            max_concurrent_recoveries: 3,
            recovery_timeout: Duration::from_secs(300),
            escalation_threshold: 5,
            cooldown_period: Duration::from_secs(60),
        }
    }
}

impl RecoveryManager {
    pub fn new(lifecycle_controller: Arc<LifecycleController>) -> Self {
        let mut strategies = HashMap::new();
        
        // Set default strategies for each module type
        strategies.insert(ModuleId::DataCapture, RecoveryStrategy::Restart {
            max_attempts: 3,
            initial_delay: Duration::from_secs(5),
            max_delay: Duration::from_secs(60),
            backoff_multiplier: 2.0,
        });
        
        strategies.insert(ModuleId::Storage, RecoveryStrategy::RestartWithReset {
            max_attempts: 2,
            reset_state: false, // Don't reset database state
            clear_queues: true,
            initial_delay: Duration::from_secs(10),
        });
        
        strategies.insert(ModuleId::AnalysisEngine, RecoveryStrategy::Restart {
            max_attempts: 5,
            initial_delay: Duration::from_secs(3),
            max_delay: Duration::from_secs(120),
            backoff_multiplier: 1.5,
        });
        
        strategies.insert(ModuleId::CuteFigurine, RecoveryStrategy::DegradedMode {
            disable_features: vec!["animations".to_string(), "sounds".to_string()],
            reduce_load: 0.5,
            timeout: Duration::from_secs(300),
        });
        
        // Critical modules get more conservative strategies
        strategies.insert(ModuleId::EventBus, RecoveryStrategy::SystemRestart {
            save_state: true,
            notify_user: true,
            grace_period: Duration::from_secs(30),
        });

        Self {
            strategies,
            recovery_history: Arc::new(Mutex::new(HashMap::new())),
            lifecycle_controller,
            global_settings: RecoverySettings::default(),
        }
    }

    /// Initiate recovery for a failed module
    pub async fn recover_module(&self, failure: ModuleFailure) -> OrchestratorResult<()> {
        let module_id = failure.module_id;
        info!("Starting recovery for module {}: {:?}", module_id, failure.failure_type);

        // Update recovery history
        {
            let mut history = self.recovery_history.lock().await;
            let module_history = history.entry(module_id)
                .or_insert_with(|| RecoveryHistory::new(module_id));
            module_history.add_failure(failure.clone());

            // Check if we should escalate
            if module_history.should_escalate() {
                warn!("Escalating recovery for module {} due to repeated failures", module_id);
                return self.escalate_recovery(module_id, &failure).await;
            }
        }

        // Get recovery strategy
        let strategy = self.strategies.get(&module_id)
            .cloned()
            .unwrap_or_default();

        // Execute recovery strategy
        self.execute_recovery_strategy(module_id, strategy, failure).await
    }

    /// Execute a specific recovery strategy
    async fn execute_recovery_strategy(
        &self,
        module_id: ModuleId,
        strategy: RecoveryStrategy,
        _failure: ModuleFailure,
    ) -> OrchestratorResult<()> {
        let attempt_number = {
            let history = self.recovery_history.lock().await;
            history.get(&module_id)
                .map(|h| h.get_attempts() + 1)
                .unwrap_or(1)
        };

        let attempt = RecoveryAttempt::new(attempt_number, strategy.clone());
        
        // Add attempt to history
        {
            let mut history = self.recovery_history.lock().await;
            let module_history = history.entry(module_id)
                .or_insert_with(|| RecoveryHistory::new(module_id));
            module_history.add_attempt(attempt.clone());
        }

        let result = match strategy {
            RecoveryStrategy::Restart { max_attempts, initial_delay, max_delay, backoff_multiplier } => {
                if attempt_number > max_attempts {
                    return Err(OrchestratorError::RecoveryFailed {
                        module: module_id,
                        reason: format!("Exceeded maximum restart attempts ({})", max_attempts),
                    });
                }

                let backoff = ExponentialBackoff::new(initial_delay, max_delay, backoff_multiplier);
                let delay = backoff.calculate(attempt_number - 1);
                
                info!("Waiting {:?} before restarting module {}", delay, module_id);
                tokio::time::sleep(delay).await;
                
                self.lifecycle_controller.restart_module(module_id).await
            }
            
            RecoveryStrategy::RestartWithReset { max_attempts, reset_state, clear_queues, initial_delay } => {
                if attempt_number > max_attempts {
                    return Err(OrchestratorError::RecoveryFailed {
                        module: module_id,
                        reason: format!("Exceeded maximum restart attempts ({})", max_attempts),
                    });
                }

                tokio::time::sleep(initial_delay).await;
                
                if reset_state {
                    info!("Resetting state for module {}", module_id);
                    // In a real implementation, this would reset module state
                }
                
                if clear_queues {
                    info!("Clearing message queues for module {}", module_id);
                    // In a real implementation, this would clear message queues
                }
                
                self.lifecycle_controller.restart_module(module_id).await
            }
            
            RecoveryStrategy::DegradedMode { disable_features, reduce_load, timeout: _ } => {
                warn!("Switching module {} to degraded mode", module_id);
                info!("Disabling features: {:?}, reducing load by {:.0}%", 
                     disable_features, (1.0 - reduce_load) * 100.0);
                
                // In a real implementation, this would:
                // 1. Send degraded mode commands to the module
                // 2. Disable specified features
                // 3. Reduce processing load
                // 4. Set up monitoring to restore full functionality
                
                Ok(())
            }
            
            RecoveryStrategy::SystemRestart { save_state, notify_user, grace_period } => {
                error!("Critical module {} failed, initiating system restart", module_id);
                
                if notify_user {
                    // In a real implementation, notify user of impending restart
                    warn!("System will restart in {:?} due to critical module failure", grace_period);
                }
                
                if save_state {
                    info!("Saving system state before restart");
                    // In a real implementation, save current system state
                }
                
                tokio::time::sleep(grace_period).await;
                
                // In a real implementation, this would trigger a system restart
                warn!("System restart triggered (simulated)");
                Ok(())
            }
            
            RecoveryStrategy::Manual { notify_admin, block_system } => {
                error!("Manual recovery required for module {}", module_id);
                
                if notify_admin {
                    // In a real implementation, send notification to administrator
                    error!("Administrator notification sent for module {} failure", module_id);
                }
                
                if block_system {
                    warn!("System operations blocked pending manual recovery");
                    // In a real implementation, this would prevent new operations
                }
                
                Err(OrchestratorError::RecoveryFailed {
                    module: module_id,
                    reason: "Manual recovery required".to_string(),
                })
            }
        };

        // Update attempt result
        {
            let mut history = self.recovery_history.lock().await;
            if let Some(module_history) = history.get_mut(&module_id) {
                if let Some(last_attempt) = module_history.attempts.last_mut() {
                    match &result {
                        Ok(()) => {
                            last_attempt.complete_success();
                            module_history.mark_success();
                            info!("Recovery successful for module {}", module_id);
                        }
                        Err(e) => {
                            last_attempt.complete_failure(e.to_string());
                            error!("Recovery failed for module {}: {}", module_id, e);
                        }
                    }
                }
            }
        }

        result
    }

    /// Escalate recovery when normal strategies fail
    async fn escalate_recovery(
        &self,
        module_id: ModuleId,
        failure: &ModuleFailure,
    ) -> OrchestratorResult<()> {
        warn!("Escalating recovery for module {} due to repeated failures", module_id);
        
        // Escalation strategies based on module criticality
        match module_id {
            ModuleId::EventBus | ModuleId::Orchestrator => {
                // Critical infrastructure - system restart
                self.execute_recovery_strategy(
                    module_id,
                    RecoveryStrategy::SystemRestart {
                        save_state: true,
                        notify_user: true,
                        grace_period: Duration::from_secs(60),
                    },
                    failure.clone(),
                ).await
            }
            ModuleId::Storage => {
                // Critical data - manual intervention
                self.execute_recovery_strategy(
                    module_id,
                    RecoveryStrategy::Manual {
                        notify_admin: true,
                        block_system: false,
                    },
                    failure.clone(),
                ).await
            }
            _ => {
                // Non-critical modules - degraded mode
                self.execute_recovery_strategy(
                    module_id,
                    RecoveryStrategy::DegradedMode {
                        disable_features: vec!["all_optional".to_string()],
                        reduce_load: 0.1,
                        timeout: Duration::from_secs(3600),
                    },
                    failure.clone(),
                ).await
            }
        }
    }

    /// Set recovery strategy for a module
    pub fn set_strategy(&mut self, module_id: ModuleId, strategy: RecoveryStrategy) {
        self.strategies.insert(module_id, strategy);
        info!("Updated recovery strategy for module {}", module_id);
    }

    /// Get recovery history for a module
    pub async fn get_recovery_history(&self, module_id: ModuleId) -> Option<RecoveryHistory> {
        let history = self.recovery_history.lock().await;
        history.get(&module_id).cloned()
    }

    /// Get recovery statistics
    pub async fn get_recovery_stats(&self) -> HashMap<ModuleId, (u32, u32, Option<Instant>)> {
        let history = self.recovery_history.lock().await;
        history.iter()
            .map(|(module_id, h)| (*module_id, (h.total_failures, h.consecutive_failures, h.last_success)))
            .collect()
    }

    /// Check if a module is in cooldown period
    pub async fn is_in_cooldown(&self, module_id: ModuleId) -> bool {
        let history = self.recovery_history.lock().await;
        if let Some(module_history) = history.get(&module_id) {
            if let Some(last_success) = module_history.last_success {
                last_success.elapsed() < self.global_settings.cooldown_period
            } else {
                false
            }
        } else {
            false
        }
    }
}