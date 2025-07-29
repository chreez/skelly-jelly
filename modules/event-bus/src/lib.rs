//! # Skelly-Jelly Event Bus
//!
//! High-performance message broker for inter-module communication.
//! Provides type-safe publish-subscribe messaging with configurable delivery guarantees.

pub mod error;
pub mod message;
pub mod router;
pub mod subscription;
pub mod bus;
pub mod metrics;
pub mod registry;
pub mod circuit_breaker;
pub mod retry;
pub mod dead_letter_queue;
pub mod error_logging;
pub mod recovery;
pub mod enhanced_bus;

// Re-export public API
pub use bus::{EventBus, EventBusImpl, create_event_bus, create_event_bus_with_config};
pub use error::{EventBusError, EventBusResult};
pub use message::{BusMessage, MessagePayload, MessagePriority, ModuleId, MessageType};
pub use subscription::{MessageFilter, SubscriptionId, DeliveryMode};
pub use metrics::BusMetrics;
pub use registry::{ModuleRegistry, ModuleInfo, ModuleStatus, HealthSummary, SystemHealth, RegistryConfig};

// Re-export error handling components
pub use circuit_breaker::{CircuitBreaker, CircuitBreakerConfig, CircuitBreakerRegistry, CircuitBreakerStats, CircuitState};
pub use retry::{RetryExecutor, RetryConfig, RetryStats, RetryPolicy, create_retry_executor};
pub use dead_letter_queue::{DeadLetterQueue, DeadLetterEntry, DeadLetterReason, DeadLetterStats, create_dead_letter_queue};
pub use error_logging::{ErrorLogger, ErrorContext, ErrorSeverity, ErrorCategory, CorrelationId, create_error_logger};
pub use recovery::{RecoverySystem, RecoveryAction, RecoveryStrategy, EscalationLevel, RecoveryIncident, IncidentStatus};
pub use enhanced_bus::{EnhancedEventBus, EnhancedEventBusArc, ErrorHandlingStats, create_enhanced_event_bus, create_enhanced_event_bus_with_config};

use async_trait::async_trait;
use uuid::Uuid;

/// Unique identifier for a published message
pub type MessageId = Uuid;

/// Unique identifier for a subscriber
pub type SubscriberId = Uuid;

/// Main event bus trait defining the public API
#[async_trait]
pub trait EventBusTrait: Send + Sync {
    /// Publish a message to the bus
    async fn publish(&self, message: BusMessage) -> EventBusResult<MessageId>;
    
    /// Subscribe to messages matching a filter
    async fn subscribe(
        &self,
        subscriber: ModuleId,
        filter: MessageFilter,
        delivery_mode: DeliveryMode,
    ) -> EventBusResult<SubscriptionId>;
    
    /// Unsubscribe from messages
    async fn unsubscribe(&self, subscription_id: SubscriptionId) -> EventBusResult<()>;
    
    /// Get current bus metrics
    async fn metrics(&self) -> EventBusResult<BusMetrics>;
    
    /// Shutdown the event bus gracefully
    async fn shutdown(&self) -> EventBusResult<()>;
}

/// Configuration for the event bus
#[derive(Debug, Clone)]
pub struct EventBusConfig {
    /// Maximum size of message queues
    pub max_queue_size: usize,
    
    /// Timeout for message delivery
    pub delivery_timeout: std::time::Duration,
    
    /// Maximum retry attempts for failed deliveries
    pub max_retry_attempts: u32,
    
    /// Size of dead letter queue
    pub dead_letter_queue_size: usize,
    
    /// Interval for collecting metrics
    pub metrics_interval: std::time::Duration,
    
    /// Threshold for detecting slow message handlers
    pub slow_handler_threshold: std::time::Duration,
    
    /// Configuration for circuit breakers
    pub circuit_breaker_config: Option<CircuitBreakerConfig>,
    
    /// Configuration for retry logic
    pub retry_config: Option<RetryConfig>,
    
    /// Configuration for error logging
    pub error_logging_config: Option<error_logging::ErrorLoggerConfig>,
    
    /// Configuration for recovery system
    pub recovery_config: Option<recovery::RecoveryConfig>,
    
    /// Whether to enable comprehensive error handling
    pub enable_error_handling: bool,
}

impl Default for EventBusConfig {
    fn default() -> Self {
        Self {
            max_queue_size: 10_000,
            delivery_timeout: std::time::Duration::from_secs(5),
            max_retry_attempts: 3,
            dead_letter_queue_size: 1_000,
            metrics_interval: std::time::Duration::from_secs(10),
            slow_handler_threshold: std::time::Duration::from_millis(100),
            circuit_breaker_config: Some(CircuitBreakerConfig::default()),
            retry_config: Some(RetryConfig::default()),
            error_logging_config: Some(error_logging::ErrorLoggerConfig::default()),
            recovery_config: Some(recovery::RecoveryConfig::default()),
            enable_error_handling: true,
        }
    }
}