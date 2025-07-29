//! Exponential Backoff Retry Logic
//!
//! Provides robust retry mechanisms with exponential backoff, jitter, and configurable
//! maximum attempts for handling transient failures in distributed systems.

use std::time::{Duration, Instant};
use rand::Rng;
use tracing::{debug, warn, error};
use serde::{Deserialize, Serialize};

/// Configuration for retry behavior
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    
    /// Initial delay between retries
    pub initial_delay: Duration,
    
    /// Maximum delay between retries
    pub max_delay: Duration,
    
    /// Multiplier for exponential backoff
    pub backoff_multiplier: f64,
    
    /// Maximum jitter percentage (0.0-1.0)
    pub jitter_factor: f64,
    
    /// Overall timeout for all retry attempts
    pub total_timeout: Option<Duration>,
    
    /// Whether to reset delay on success
    pub reset_on_success: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
            jitter_factor: 0.1, // 10% jitter
            total_timeout: Some(Duration::from_secs(300)), // 5 minutes
            reset_on_success: true,
        }
    }
}

/// Information about a retry attempt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryAttempt {
    pub attempt_number: u32,
    pub delay_before_attempt: Duration,
    pub total_elapsed: Duration,
    pub is_final_attempt: bool,
}

/// Statistics about retry operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryStats {
    pub total_operations: u64,
    pub successful_operations: u64,
    pub failed_operations: u64,
    pub total_retry_attempts: u64,
    pub average_attempts_per_operation: f64,
    pub average_success_time: Duration,
    pub max_attempts_reached: u64,
    pub timeout_exceeded: u64,
}

/// Error types for retry operations
#[derive(Debug, thiserror::Error)]
pub enum RetryError<E> {
    #[error("Maximum retry attempts ({max_attempts}) exceeded")]
    MaxAttemptsExceeded { max_attempts: u32 },
    
    #[error("Total timeout ({timeout:?}) exceeded")]
    TimeoutExceeded { timeout: Duration },
    
    #[error("Operation failed permanently: {error}")]
    PermanentFailure { error: E },
    
    #[error("Retry configuration error: {reason}")]
    ConfigurationError { reason: String },
}

/// Result type for retry operations
pub type RetryResult<T, E> = Result<T, RetryError<E>>;

/// Retry policy that determines if an error should be retried
pub trait RetryPolicy<E> {
    /// Determine if an error should be retried
    fn should_retry(&self, error: &E, attempt: u32) -> bool;
    
    /// Get a custom delay for this specific error (optional)
    fn custom_delay(&self, error: &E, attempt: u32) -> Option<Duration> {
        None
    }
}

/// Default retry policy that retries all errors
pub struct DefaultRetryPolicy;

impl<E> RetryPolicy<E> for DefaultRetryPolicy {
    fn should_retry(&self, _error: &E, _attempt: u32) -> bool {
        true
    }
}

/// Retry policy that never retries (fail-fast)
pub struct NoRetryPolicy;

impl<E> RetryPolicy<E> for NoRetryPolicy {
    fn should_retry(&self, _error: &E, _attempt: u32) -> bool {
        false
    }
}

/// Predicate-based retry policy
pub struct PredicateRetryPolicy<F> {
    predicate: F,
}

impl<F> PredicateRetryPolicy<F> {
    pub fn new(predicate: F) -> Self {
        Self { predicate }
    }
}

impl<F, E> RetryPolicy<E> for PredicateRetryPolicy<F>
where
    F: Fn(&E, u32) -> bool,
{
    fn should_retry(&self, error: &E, attempt: u32) -> bool {
        (self.predicate)(error, attempt)
    }
}

/// Exponential backoff retry implementation
pub struct RetryExecutor {
    config: RetryConfig,
    stats: parking_lot::RwLock<RetryStats>,
}

impl RetryExecutor {
    /// Create a new retry executor with the given configuration
    pub fn new(config: RetryConfig) -> Result<Self, RetryError<()>> {
        // Validate configuration
        if config.max_attempts == 0 {
            return Err(RetryError::ConfigurationError {
                reason: "max_attempts must be greater than 0".to_string(),
            });
        }
        
        if config.backoff_multiplier <= 0.0 {
            return Err(RetryError::ConfigurationError {
                reason: "backoff_multiplier must be positive".to_string(),
            });
        }
        
        if config.jitter_factor < 0.0 || config.jitter_factor > 1.0 {
            return Err(RetryError::ConfigurationError {
                reason: "jitter_factor must be between 0.0 and 1.0".to_string(),
            });
        }

        let stats = RetryStats {
            total_operations: 0,
            successful_operations: 0,
            failed_operations: 0,
            total_retry_attempts: 0,
            average_attempts_per_operation: 0.0,
            average_success_time: Duration::from_millis(0),
            max_attempts_reached: 0,
            timeout_exceeded: 0,
        };

        Ok(Self {
            config,
            stats: parking_lot::RwLock::new(stats),
        })
    }

    /// Execute an operation with retry logic
    pub async fn execute<F, T, E, P>(&self, mut operation: F, policy: P) -> RetryResult<T, E>
    where
        F: FnMut(RetryAttempt) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, E>> + Send>>,
        E: std::fmt::Debug + Clone,
        P: RetryPolicy<E>,
    {
        let start_time = Instant::now();
        let mut current_delay = self.config.initial_delay;
        let mut attempt = 1;

        // Update total operations count
        {
            let mut stats = self.stats.write();
            stats.total_operations += 1;
        }

        loop {
            let attempt_start = Instant::now();
            let total_elapsed = start_time.elapsed();
            
            // Check total timeout
            if let Some(timeout) = self.config.total_timeout {
                if total_elapsed >= timeout {
                    self.record_timeout_exceeded();
                    return Err(RetryError::TimeoutExceeded { timeout });
                }
            }

            let is_final_attempt = attempt >= self.config.max_attempts;
            let retry_attempt = RetryAttempt {
                attempt_number: attempt,
                delay_before_attempt: if attempt > 1 { current_delay } else { Duration::from_millis(0) },
                total_elapsed,
                is_final_attempt,
            };

            debug!("Executing operation attempt {} of {}", attempt, self.config.max_attempts);

            // Execute the operation
            let result = operation(retry_attempt).await;

            match result {
                Ok(value) => {
                    let success_time = attempt_start.elapsed();
                    self.record_success(attempt, success_time);
                    debug!("Operation succeeded on attempt {} after {:?}", attempt, success_time);
                    return Ok(value);
                }
                Err(error) => {
                    let failure_time = attempt_start.elapsed();
                    debug!("Operation failed on attempt {} after {:?}: {:?}", attempt, failure_time, error);

                    // Check if this is the final attempt
                    if is_final_attempt {
                        self.record_max_attempts_reached();
                        warn!("Operation failed after {} attempts", attempt);
                        return Err(RetryError::MaxAttemptsExceeded {
                            max_attempts: self.config.max_attempts,
                        });
                    }

                    // Check if we should retry this error
                    if !policy.should_retry(&error, attempt) {
                        self.record_permanent_failure();
                        return Err(RetryError::PermanentFailure { error });
                    }

                    // Calculate delay for next attempt
                    let base_delay = policy.custom_delay(&error, attempt).unwrap_or(current_delay);
                    let jittered_delay = self.apply_jitter(base_delay);
                    
                    debug!("Retrying after {:?} delay", jittered_delay);
                    tokio::time::sleep(jittered_delay).await;

                    // Update delay for next iteration
                    current_delay = self.calculate_next_delay(current_delay);
                    
                    // Record retry attempt
                    self.record_retry_attempt();
                    
                    attempt += 1;
                }
            }
        }
    }

    /// Execute with default retry policy
    pub async fn execute_with_default<F, T, E>(&self, operation: F) -> RetryResult<T, E>
    where
        F: FnMut(RetryAttempt) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, E>> + Send>>,
        E: std::fmt::Debug + Clone,
    {
        self.execute(operation, DefaultRetryPolicy).await
    }

    /// Calculate the next delay using exponential backoff
    fn calculate_next_delay(&self, current_delay: Duration) -> Duration {
        let next_delay_ms = (current_delay.as_millis() as f64 * self.config.backoff_multiplier) as u64;
        let next_delay = Duration::from_millis(next_delay_ms);
        
        // Cap at max delay
        if next_delay > self.config.max_delay {
            self.config.max_delay
        } else {
            next_delay
        }
    }

    /// Apply jitter to the delay to avoid thundering herd
    fn apply_jitter(&self, delay: Duration) -> Duration {
        if self.config.jitter_factor == 0.0 {
            return delay;
        }

        let mut rng = rand::thread_rng();
        let jitter_range = delay.as_millis() as f64 * self.config.jitter_factor;
        let jitter = rng.gen_range(-jitter_range..=jitter_range) as i64;
        
        let jittered_ms = delay.as_millis() as i64 + jitter;
        Duration::from_millis(jittered_ms.max(0) as u64)
    }

    /// Record a successful operation
    fn record_success(&self, attempts: u32, success_time: Duration) {
        let mut stats = self.stats.write();
        stats.successful_operations += 1;
        stats.total_retry_attempts += (attempts - 1) as u64; // Don't count the first attempt as a retry
        
        // Update average attempts per operation
        let total_ops = stats.total_operations as f64;
        let total_attempts = stats.total_retry_attempts as f64 + stats.total_operations as f64;
        stats.average_attempts_per_operation = total_attempts / total_ops;
        
        // Update average success time (exponential moving average)
        if stats.successful_operations == 1 {
            stats.average_success_time = success_time;
        } else {
            let alpha = 0.1; // Smoothing factor
            let current_avg_ms = stats.average_success_time.as_millis() as f64;
            let new_time_ms = success_time.as_millis() as f64;
            let new_avg_ms = alpha * new_time_ms + (1.0 - alpha) * current_avg_ms;
            stats.average_success_time = Duration::from_millis(new_avg_ms as u64);
        }
    }

    /// Record a retry attempt
    fn record_retry_attempt(&self) {
        let mut stats = self.stats.write();
        stats.total_retry_attempts += 1;
    }

    /// Record max attempts reached
    fn record_max_attempts_reached(&self) {
        let mut stats = self.stats.write();
        stats.failed_operations += 1;
        stats.max_attempts_reached += 1;
    }

    /// Record timeout exceeded
    fn record_timeout_exceeded(&self) {
        let mut stats = self.stats.write();
        stats.failed_operations += 1;
        stats.timeout_exceeded += 1;
    }

    /// Record permanent failure
    fn record_permanent_failure(&self) {
        let mut stats = self.stats.write();
        stats.failed_operations += 1;
    }

    /// Get current retry statistics
    pub fn stats(&self) -> RetryStats {
        self.stats.read().clone()
    }

    /// Reset statistics
    pub fn reset_stats(&self) {
        let mut stats = self.stats.write();
        *stats = RetryStats {
            total_operations: 0,
            successful_operations: 0,
            failed_operations: 0,
            total_retry_attempts: 0,
            average_attempts_per_operation: 0.0,
            average_success_time: Duration::from_millis(0),
            max_attempts_reached: 0,
            timeout_exceeded: 0,
        };
    }

    /// Get configuration
    pub fn config(&self) -> &RetryConfig {
        &self.config
    }
}

/// Convenience function to create a retry executor with default configuration
pub fn create_retry_executor() -> Result<RetryExecutor, RetryError<()>> {
    RetryExecutor::new(RetryConfig::default())
}

/// Convenience function to create a retry executor with custom max attempts
pub fn create_retry_executor_with_attempts(max_attempts: u32) -> Result<RetryExecutor, RetryError<()>> {
    let config = RetryConfig {
        max_attempts,
        ..RetryConfig::default()
    };
    RetryExecutor::new(config)
}

/// Convenience function to retry an operation with default configuration
pub async fn retry_operation<F, T, E>(operation: F) -> RetryResult<T, E>
where
    F: FnMut(RetryAttempt) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, E>> + Send>>,
    E: std::fmt::Debug + Clone,
{
    let executor = create_retry_executor().map_err(|_| RetryError::ConfigurationError {
        reason: "Failed to create default retry executor".to_string(),
    })?;
    
    executor.execute_with_default(operation).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, atomic::{AtomicU32, Ordering}};
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_successful_operation_no_retry() {
        let executor = create_retry_executor().unwrap();
        
        let result = executor.execute_with_default(|_attempt| {
            Box::pin(async { Ok::<i32, String>(42) })
        }).await;
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        
        let stats = executor.stats();
        assert_eq!(stats.successful_operations, 1);
        assert_eq!(stats.total_retry_attempts, 0);
    }

    #[tokio::test]
    async fn test_retry_until_success() {
        let executor = create_retry_executor().unwrap();
        let attempt_count = Arc::new(AtomicU32::new(0));
        
        let attempt_count_clone = attempt_count.clone();
        let result = executor.execute_with_default(move |_retry_attempt| {
            let count = attempt_count_clone.clone();
            Box::pin(async move {
                let current = count.fetch_add(1, Ordering::SeqCst) + 1;
                if current < 3 {
                    Err("not ready yet".to_string())
                } else {
                    Ok(42)
                }
            })
        }).await;
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(attempt_count.load(Ordering::SeqCst), 3);
        
        let stats = executor.stats();
        assert_eq!(stats.successful_operations, 1);
        assert_eq!(stats.total_retry_attempts, 2); // 3 attempts = 2 retries
    }

    #[tokio::test]
    async fn test_max_attempts_exceeded() {
        let config = RetryConfig {
            max_attempts: 2,
            initial_delay: Duration::from_millis(10),
            ..RetryConfig::default()
        };
        let executor = RetryExecutor::new(config).unwrap();
        
        let result = executor.execute_with_default(|_attempt| {
            Box::pin(async { Err::<i32, String>("always fails".to_string()) })
        }).await;
        
        assert!(matches!(result, Err(RetryError::MaxAttemptsExceeded { max_attempts: 2 })));
        
        let stats = executor.stats();
        assert_eq!(stats.failed_operations, 1);
        assert_eq!(stats.max_attempts_reached, 1);
    }

    #[tokio::test]
    async fn test_timeout_exceeded() {
        let config = RetryConfig {
            max_attempts: 10,
            initial_delay: Duration::from_millis(50),
            total_timeout: Some(Duration::from_millis(100)),
            ..RetryConfig::default()
        };
        let executor = RetryExecutor::new(config).unwrap();
        
        let result = executor.execute_with_default(|_attempt| {
            Box::pin(async {
                sleep(Duration::from_millis(30)).await;
                Err::<i32, String>("slow failure".to_string())
            })
        }).await;
        
        assert!(matches!(result, Err(RetryError::TimeoutExceeded { .. })));
        
        let stats = executor.stats();
        assert_eq!(stats.timeout_exceeded, 1);
    }

    #[tokio::test]
    async fn test_custom_retry_policy() {
        let executor = create_retry_executor().unwrap();
        
        // Policy that only retries "retryable" errors
        let policy = PredicateRetryPolicy::new(|error: &String, _attempt| {
            error.contains("retryable")
        });
        
        let result = executor.execute(|_attempt| {
            Box::pin(async { Err::<i32, String>("permanent error".to_string()) })
        }, policy).await;
        
        assert!(matches!(result, Err(RetryError::PermanentFailure { .. })));
    }

    #[tokio::test]
    async fn test_exponential_backoff() {
        let config = RetryConfig {
            max_attempts: 4,
            initial_delay: Duration::from_millis(10),
            backoff_multiplier: 2.0,
            jitter_factor: 0.0, // No jitter for predictable testing
            ..RetryConfig::default()
        };
        let executor = RetryExecutor::new(config).unwrap();
        
        let delays = Arc::new(parking_lot::Mutex::new(Vec::new()));
        let delays_clone = delays.clone();
        
        let start_time = Instant::now();
        let result = executor.execute_with_default(move |attempt| {
            let delays = delays_clone.clone();
            Box::pin(async move {
                if attempt.attempt_number > 1 {
                    delays.lock().push(attempt.delay_before_attempt);
                }
                Err::<i32, String>("always fails".to_string())
            })
        }).await;
        
        assert!(result.is_err());
        
        let recorded_delays = delays.lock();
        assert_eq!(recorded_delays.len(), 3); // 4 attempts = 3 delays
        
        // Check exponential backoff (approximately, allowing for small timing variations)
        assert!(recorded_delays[0] >= Duration::from_millis(10));
        assert!(recorded_delays[1] >= Duration::from_millis(20));
        assert!(recorded_delays[2] >= Duration::from_millis(40));
    }

    #[tokio::test]
    async fn test_jitter() {
        let config = RetryConfig {
            max_attempts: 10,
            initial_delay: Duration::from_millis(100),
            backoff_multiplier: 1.0, // No backoff, constant delay
            jitter_factor: 0.5, // 50% jitter
            ..RetryConfig::default()
        };
        let executor = RetryExecutor::new(config).unwrap();
        
        let delays = Arc::new(parking_lot::Mutex::new(Vec::new()));
        let delays_clone = delays.clone();
        let attempt_count = Arc::new(AtomicU32::new(0));
        let attempt_count_clone = attempt_count.clone();
        
        let result = executor.execute_with_default(move |attempt| {
            let delays = delays_clone.clone();
            let count = attempt_count_clone.clone();
            Box::pin(async move {
                let current = count.fetch_add(1, Ordering::SeqCst) + 1;
                if attempt.attempt_number > 1 {
                    delays.lock().push(attempt.delay_before_attempt);
                }
                if current < 5 {
                    Err("not ready yet".to_string())
                } else {
                    Ok(42)
                }
            })
        }).await;
        
        assert!(result.is_ok());
        
        let recorded_delays = delays.lock();
        assert_eq!(recorded_delays.len(), 4); // 5 attempts = 4 delays
        
        // Check that delays vary due to jitter (not all exactly 100ms)
        let base_delay = Duration::from_millis(100);
        let min_delay = Duration::from_millis(50); // 50% jitter down
        let max_delay = Duration::from_millis(150); // 50% jitter up
        
        for delay in recorded_delays.iter() {
            assert!(*delay >= min_delay && *delay <= max_delay);
        }
    }

    #[tokio::test]
    async fn test_retry_stats() {
        let executor = create_retry_executor().unwrap();
        
        // Successful operation
        let _ = executor.execute_with_default(|_attempt| {
            Box::pin(async { Ok::<i32, String>(42) })
        }).await;
        
        // Failed operation with retries
        let attempt_count = Arc::new(AtomicU32::new(0));
        let attempt_count_clone = attempt_count.clone();
        let _ = executor.execute_with_default(move |_attempt| {
            let count = attempt_count_clone.clone();
            Box::pin(async move {
                let current = count.fetch_add(1, Ordering::SeqCst) + 1;
                if current < 3 {
                    Err("not ready yet".to_string())
                } else {
                    Ok(99)
                }
            })
        }).await;
        
        let stats = executor.stats();
        assert_eq!(stats.total_operations, 2);
        assert_eq!(stats.successful_operations, 2);
        assert_eq!(stats.failed_operations, 0);
        assert_eq!(stats.total_retry_attempts, 2); // Second operation had 2 retries
        assert!(stats.average_attempts_per_operation > 1.0);
    }

    #[tokio::test]
    async fn test_configuration_validation() {
        // Test invalid max_attempts
        let config = RetryConfig {
            max_attempts: 0,
            ..RetryConfig::default()
        };
        assert!(RetryExecutor::new(config).is_err());
        
        // Test invalid backoff_multiplier
        let config = RetryConfig {
            backoff_multiplier: -1.0,
            ..RetryConfig::default()
        };
        assert!(RetryExecutor::new(config).is_err());
        
        // Test invalid jitter_factor
        let config = RetryConfig {
            jitter_factor: 1.5,
            ..RetryConfig::default()
        };
        assert!(RetryExecutor::new(config).is_err());
    }
}