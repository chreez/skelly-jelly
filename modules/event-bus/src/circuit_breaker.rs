//! Circuit Breaker Pattern Implementation
//!
//! Provides circuit breaker functionality to handle failures in external dependencies
//! with intelligent failure detection and automatic recovery mechanisms.

use std::sync::Arc;
use std::time::{Duration, Instant};
use parking_lot::RwLock;
use tracing::{debug, warn, error};
use serde::{Deserialize, Serialize};

/// Configuration for circuit breaker behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    /// Number of failures required to open the circuit
    pub failure_threshold: u32,
    
    /// Time to wait in open state before attempting to close
    pub reset_timeout: Duration,
    
    /// Maximum time to wait for operations to complete
    pub operation_timeout: Duration,
    
    /// Percentage of successful operations needed to close circuit in half-open state
    pub success_threshold: f64,
    
    /// Number of test operations to perform in half-open state
    pub half_open_max_calls: u32,
    
    /// Time window for failure counting
    pub failure_count_window: Duration,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            reset_timeout: Duration::from_secs(60),
            operation_timeout: Duration::from_secs(10),
            success_threshold: 0.8, // 80% success rate required
            half_open_max_calls: 5,
            failure_count_window: Duration::from_secs(300), // 5 minutes
        }
    }
}

/// Circuit breaker states
#[derive(Debug, Clone, PartialEq)]
pub enum CircuitState {
    /// Circuit is closed, operations are allowed
    Closed,
    /// Circuit is open, operations are rejected
    Open { opened_at: Instant },
    /// Circuit is half-open, testing if service is recovered
    HalfOpen { test_count: u32, success_count: u32 },
}

/// Circuit breaker statistics
#[derive(Debug, Clone)]
pub struct CircuitBreakerStats {
    pub state: CircuitState,
    pub failure_count: u32,
    pub success_count: u32,
    pub total_operations: u64,
    pub last_failure_time: Option<Instant>,
    pub state_changed_at: Instant,
    pub consecutive_failures: u32,
    pub average_response_time: Duration,
}

/// Error types for circuit breaker operations
#[derive(Debug, thiserror::Error)]
pub enum CircuitBreakerError {
    #[error("Circuit breaker is open, operation rejected")]
    CircuitOpen,
    
    #[error("Operation timeout after {timeout:?}")]
    OperationTimeout { timeout: Duration },
    
    #[error("Operation failed: {reason}")]
    OperationFailed { reason: String },
    
    #[error("Circuit breaker configuration error: {reason}")]
    ConfigurationError { reason: String },
}

/// Result type for circuit breaker operations
pub type CircuitBreakerResult<T> = Result<T, CircuitBreakerError>;

/// Circuit breaker implementation with intelligent failure detection
pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    state: Arc<RwLock<CircuitBreakerInternalState>>,
    name: String,
}

#[derive(Debug)]
struct CircuitBreakerInternalState {
    state: CircuitState,
    failure_count: u32,
    success_count: u32,
    total_operations: u64,
    last_failure_time: Option<Instant>,
    state_changed_at: Instant,
    consecutive_failures: u32,
    response_times: Vec<Duration>,
    failure_times: Vec<Instant>,
}

impl CircuitBreaker {
    /// Create a new circuit breaker with the given configuration
    pub fn new(name: String, config: CircuitBreakerConfig) -> Self {
        let state = CircuitBreakerInternalState {
            state: CircuitState::Closed,
            failure_count: 0,
            success_count: 0,
            total_operations: 0,
            last_failure_time: None,
            state_changed_at: Instant::now(),
            consecutive_failures: 0,
            response_times: Vec::new(),
            failure_times: Vec::new(),
        };

        Self {
            config,
            state: Arc::new(RwLock::new(state)),
            name,
        }
    }

    /// Execute an operation through the circuit breaker
    pub async fn execute<F, T, E>(&self, operation: F) -> CircuitBreakerResult<T>
    where
        F: std::future::Future<Output = Result<T, E>>,
        E: std::fmt::Display,
    {
        // Check if operation is allowed
        if !self.is_operation_allowed() {
            return Err(CircuitBreakerError::CircuitOpen);
        }

        let start_time = Instant::now();
        
        // Execute the operation with timeout
        let result = tokio::time::timeout(self.config.operation_timeout, operation).await;
        
        let execution_time = start_time.elapsed();

        match result {
            Ok(Ok(value)) => {
                self.record_success(execution_time);
                Ok(value)
            }
            Ok(Err(error)) => {
                let error_msg = error.to_string();
                self.record_failure(execution_time);
                Err(CircuitBreakerError::OperationFailed { reason: error_msg })
            }
            Err(_) => {
                self.record_failure(execution_time);
                Err(CircuitBreakerError::OperationTimeout {
                    timeout: self.config.operation_timeout,
                })
            }
        }
    }

    /// Check if operation is allowed based on current circuit state
    fn is_operation_allowed(&self) -> bool {
        let mut state = self.state.write();
        
        match &state.state {
            CircuitState::Closed => true,
            CircuitState::Open { opened_at } => {
                // Check if reset timeout has passed
                if opened_at.elapsed() >= self.config.reset_timeout {
                    debug!("Circuit breaker {} transitioning to half-open", self.name);
                    state.state = CircuitState::HalfOpen {
                        test_count: 0,
                        success_count: 0,
                    };
                    state.state_changed_at = Instant::now();
                    true
                } else {
                    false
                }
            }
            CircuitState::HalfOpen { test_count, .. } => {
                *test_count < self.config.half_open_max_calls
            }
        }
    }

    /// Record a successful operation
    fn record_success(&self, execution_time: Duration) {
        let mut state = self.state.write();
        
        state.success_count += 1;
        state.total_operations += 1;
        state.consecutive_failures = 0;
        state.response_times.push(execution_time);
        
        // Keep only recent response times for average calculation
        if state.response_times.len() > 100 {
            state.response_times.remove(0);
        }

        match &state.state {
            CircuitState::HalfOpen { test_count, success_count } => {
                let new_test_count = test_count + 1;
                let new_success_count = success_count + 1;
                
                if new_test_count >= self.config.half_open_max_calls {
                    let success_rate = new_success_count as f64 / new_test_count as f64;
                    
                    if success_rate >= self.config.success_threshold {
                        debug!("Circuit breaker {} closing after successful recovery", self.name);
                        state.state = CircuitState::Closed;
                        state.failure_count = 0;
                        state.state_changed_at = Instant::now();
                    } else {
                        warn!("Circuit breaker {} reopening, success rate {:.2} below threshold {:.2}", 
                              self.name, success_rate, self.config.success_threshold);
                        state.state = CircuitState::Open { opened_at: Instant::now() };
                        state.state_changed_at = Instant::now();
                    }
                } else {
                    state.state = CircuitState::HalfOpen {
                        test_count: new_test_count,
                        success_count: new_success_count,
                    };
                }
            }
            _ => {} // No state change needed for closed circuit
        }
    }

    /// Record a failed operation
    fn record_failure(&self, execution_time: Duration) {
        let mut state = self.state.write();
        
        state.failure_count += 1;
        state.total_operations += 1;
        state.consecutive_failures += 1;
        state.last_failure_time = Some(Instant::now());
        state.failure_times.push(Instant::now());
        state.response_times.push(execution_time);
        
        // Clean up old failure times outside the window
        let cutoff_time = Instant::now() - self.config.failure_count_window;
        state.failure_times.retain(|&time| time >= cutoff_time);
        
        // Keep only recent response times
        if state.response_times.len() > 100 {
            state.response_times.remove(0);
        }

        match &state.state {
            CircuitState::Closed => {
                // Check if we should open the circuit
                let recent_failures = state.failure_times.len() as u32;
                if recent_failures >= self.config.failure_threshold {
                    warn!("Circuit breaker {} opening due to {} failures in window", 
                          self.name, recent_failures);
                    state.state = CircuitState::Open { opened_at: Instant::now() };
                    state.state_changed_at = Instant::now();
                }
            }
            CircuitState::HalfOpen { .. } => {
                // Any failure in half-open state reopens the circuit
                warn!("Circuit breaker {} reopening due to failure in half-open state", self.name);
                state.state = CircuitState::Open { opened_at: Instant::now() };
                state.state_changed_at = Instant::now();
            }
            CircuitState::Open { .. } => {
                // Already open, just record the failure
            }
        }
    }

    /// Get current circuit breaker statistics
    pub fn stats(&self) -> CircuitBreakerStats {
        let state = self.state.read();
        
        let average_response_time = if state.response_times.is_empty() {
            Duration::from_millis(0)
        } else {
            let total: Duration = state.response_times.iter().sum();
            total / state.response_times.len() as u32
        };

        CircuitBreakerStats {
            state: state.state.clone(),
            failure_count: state.failure_count,
            success_count: state.success_count,
            total_operations: state.total_operations,
            last_failure_time: state.last_failure_time,
            state_changed_at: state.state_changed_at,
            consecutive_failures: state.consecutive_failures,
            average_response_time,
        }
    }

    /// Get current circuit state
    pub fn current_state(&self) -> CircuitState {
        self.state.read().state.clone()
    }

    /// Check if the circuit is currently healthy
    pub fn is_healthy(&self) -> bool {
        matches!(self.current_state(), CircuitState::Closed)
    }

    /// Force the circuit to close (for testing/manual intervention)
    pub fn force_close(&self) {
        let mut state = self.state.write();
        state.state = CircuitState::Closed;
        state.failure_count = 0;
        state.consecutive_failures = 0;
        state.state_changed_at = Instant::now();
        debug!("Circuit breaker {} forcibly closed", self.name);
    }

    /// Force the circuit to open (for testing/manual intervention)
    pub fn force_open(&self) {
        let mut state = self.state.write();
        state.state = CircuitState::Open { opened_at: Instant::now() };
        state.state_changed_at = Instant::now();
        warn!("Circuit breaker {} forcibly opened", self.name);
    }

    /// Get the circuit breaker name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the circuit breaker configuration
    pub fn config(&self) -> &CircuitBreakerConfig {
        &self.config
    }
}

/// Circuit breaker registry for managing multiple circuit breakers
pub struct CircuitBreakerRegistry {
    breakers: Arc<RwLock<std::collections::HashMap<String, Arc<CircuitBreaker>>>>,
}

impl CircuitBreakerRegistry {
    /// Create a new circuit breaker registry
    pub fn new() -> Self {
        Self {
            breakers: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Register a new circuit breaker
    pub fn register(&self, name: String, config: CircuitBreakerConfig) -> Arc<CircuitBreaker> {
        let breaker = Arc::new(CircuitBreaker::new(name.clone(), config));
        self.breakers.write().insert(name, breaker.clone());
        breaker
    }

    /// Get a circuit breaker by name
    pub fn get(&self, name: &str) -> Option<Arc<CircuitBreaker>> {
        self.breakers.read().get(name).cloned()
    }

    /// Get all circuit breakers
    pub fn all(&self) -> Vec<Arc<CircuitBreaker>> {
        self.breakers.read().values().cloned().collect()
    }

    /// Get statistics for all circuit breakers
    pub fn all_stats(&self) -> std::collections::HashMap<String, CircuitBreakerStats> {
        self.breakers
            .read()
            .iter()
            .map(|(name, breaker)| (name.clone(), breaker.stats()))
            .collect()
    }

    /// Check if all circuit breakers are healthy
    pub fn all_healthy(&self) -> bool {
        self.breakers.read().values().all(|breaker| breaker.is_healthy())
    }

    /// Remove a circuit breaker
    pub fn remove(&self, name: &str) -> Option<Arc<CircuitBreaker>> {
        self.breakers.write().remove(name)
    }
}

impl Default for CircuitBreakerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_circuit_breaker_success() {
        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            reset_timeout: Duration::from_millis(100),
            operation_timeout: Duration::from_secs(1),
            success_threshold: 0.8,
            half_open_max_calls: 2,
            failure_count_window: Duration::from_secs(60),
        };

        let breaker = CircuitBreaker::new("test".to_string(), config);

        // Successful operation
        let result = breaker.execute(async { Ok::<i32, String>(42) }).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);

        let stats = breaker.stats();
        assert_eq!(stats.success_count, 1);
        assert_eq!(stats.failure_count, 0);
        assert!(matches!(stats.state, CircuitState::Closed));
    }

    #[tokio::test]
    async fn test_circuit_breaker_failure() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            reset_timeout: Duration::from_millis(100),
            operation_timeout: Duration::from_secs(1),
            success_threshold: 0.8,
            half_open_max_calls: 2,
            failure_count_window: Duration::from_secs(60),
        };

        let breaker = CircuitBreaker::new("test".to_string(), config);

        // First failure
        let result = breaker.execute(async { Err::<i32, String>("error".to_string()) }).await;
        assert!(result.is_err());

        // Second failure should open the circuit
        let result = breaker.execute(async { Err::<i32, String>("error".to_string()) }).await;
        assert!(result.is_err());

        let stats = breaker.stats();
        assert!(matches!(stats.state, CircuitState::Open { .. }));

        // Third operation should be rejected
        let result = breaker.execute(async { Ok::<i32, String>(42) }).await;
        assert!(matches!(result, Err(CircuitBreakerError::CircuitOpen)));
    }

    #[tokio::test]
    async fn test_circuit_breaker_recovery() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            reset_timeout: Duration::from_millis(50),
            operation_timeout: Duration::from_secs(1),
            success_threshold: 0.8,
            half_open_max_calls: 2,
            failure_count_window: Duration::from_secs(60),
        };

        let breaker = CircuitBreaker::new("test".to_string(), config);

        // Trigger failures to open circuit
        let _ = breaker.execute(async { Err::<i32, String>("error".to_string()) }).await;
        let _ = breaker.execute(async { Err::<i32, String>("error".to_string()) }).await;

        assert!(matches!(breaker.current_state(), CircuitState::Open { .. }));

        // Wait for reset timeout
        sleep(Duration::from_millis(60)).await;

        // Next operation should be allowed (half-open)
        let result = breaker.execute(async { Ok::<i32, String>(42) }).await;
        assert!(result.is_ok());

        // Another successful operation should close the circuit
        let result = breaker.execute(async { Ok::<i32, String>(43) }).await;
        assert!(result.is_ok());

        assert!(matches!(breaker.current_state(), CircuitState::Closed));
    }

    #[tokio::test]
    async fn test_circuit_breaker_timeout() {
        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            reset_timeout: Duration::from_millis(100),
            operation_timeout: Duration::from_millis(50),
            success_threshold: 0.8,
            half_open_max_calls: 2,
            failure_count_window: Duration::from_secs(60),
        };

        let breaker = CircuitBreaker::new("test".to_string(), config);

        // Operation that times out
        let result = breaker.execute(async {
            sleep(Duration::from_millis(100)).await;
            Ok::<i32, String>(42)
        }).await;

        assert!(matches!(result, Err(CircuitBreakerError::OperationTimeout { .. })));

        let stats = breaker.stats();
        assert_eq!(stats.failure_count, 1);
    }

    #[tokio::test]
    async fn test_circuit_breaker_registry() {
        let registry = CircuitBreakerRegistry::new();
        let config = CircuitBreakerConfig::default();

        // Register a circuit breaker
        let breaker = registry.register("test".to_string(), config);
        assert_eq!(breaker.name(), "test");

        // Get the circuit breaker
        let retrieved = registry.get("test").unwrap();
        assert_eq!(retrieved.name(), "test");

        // Check all stats
        let all_stats = registry.all_stats();
        assert!(all_stats.contains_key("test"));

        // Check health
        assert!(registry.all_healthy());

        // Remove the circuit breaker
        let removed = registry.remove("test");
        assert!(removed.is_some());
        assert!(registry.get("test").is_none());
    }
}