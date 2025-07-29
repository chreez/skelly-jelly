//! Error types for the event bus

use std::time::Duration;
use thiserror::Error;
use crate::{ModuleId, SubscriptionId};

/// Result type for event bus operations
pub type EventBusResult<T> = Result<T, EventBusError>;

/// Comprehensive error types for event bus operations
#[derive(Error, Debug, Clone)]
pub enum EventBusError {
    #[error("Subscriber {subscriber} is temporarily unavailable, retry after {retry_after:?}")]
    SubscriberUnavailable {
        subscriber: ModuleId,
        retry_after: Duration,
    },

    #[error("Message rejected: {reason}")]
    MessageRejected { reason: String },

    #[error("Delivery timeout: operation took {elapsed:?}")]
    DeliveryTimeout { elapsed: Duration },

    #[error("Queue full: current size {current_size}, max size {max_size}")]
    QueueFull { current_size: usize, max_size: usize },

    #[error("Subscription {subscription_id} not found")]
    SubscriptionNotFound { subscription_id: SubscriptionId },

    #[error("Invalid message filter: {reason}")]
    InvalidFilter { reason: String },

    #[error("Bus is shutting down")]
    BusShuttingDown,

    #[error("Channel send error: {0}")]
    ChannelSend(String),

    #[error("Channel receive error: {0}")]
    ChannelReceive(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Module {module_id} is already registered")]
    ModuleAlreadyRegistered { module_id: ModuleId },

    #[error("Module {module_id} not found")]
    ModuleNotFound { module_id: ModuleId },

    #[error("Invalid health check response")]
    InvalidHealthCheckResponse,

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("IO error: {0}")]
    Io(String),
}

impl EventBusError {
    /// Check if this error is recoverable with retry
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            EventBusError::SubscriberUnavailable { .. }
                | EventBusError::DeliveryTimeout { .. }
                | EventBusError::QueueFull { .. }
                | EventBusError::ChannelSend(_)
        )
    }

    /// Get the suggested retry delay for recoverable errors
    pub fn retry_delay(&self) -> Option<Duration> {
        match self {
            EventBusError::SubscriberUnavailable { retry_after, .. } => Some(*retry_after),
            EventBusError::DeliveryTimeout { .. } => Some(Duration::from_millis(100)),
            EventBusError::QueueFull { .. } => Some(Duration::from_millis(50)),
            _ => None,
        }
    }
}