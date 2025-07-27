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

// Re-export public API
pub use bus::{EventBus, EventBusImpl, create_event_bus, create_event_bus_with_config};
pub use error::{EventBusError, EventBusResult};
pub use message::{BusMessage, MessagePayload, MessagePriority, ModuleId, MessageType};
pub use subscription::{MessageFilter, SubscriptionId, DeliveryMode};
pub use metrics::BusMetrics;

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
        }
    }
}