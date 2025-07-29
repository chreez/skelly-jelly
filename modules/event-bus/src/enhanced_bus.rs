//! Enhanced Event Bus with Integrated Error Handling
//!
//! Provides a comprehensive event bus implementation that integrates all error handling
//! components including circuit breakers, retry logic, dead letter queue, structured
//! logging, and automatic recovery systems.

use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use crossbeam_channel::{bounded, Receiver};
use tracing::{debug, info, warn, error};

use crate::{
    BusMessage, EventBusConfig, EventBusError, EventBusResult, EventBusTrait,
    MessageId, ModuleId, SubscriptionId,
    subscription::{DeliveryMode, MessageFilter, Subscription},
    router::{MessageRouter, RouterConfig},
    metrics::BusMetrics,
    registry::{ModuleRegistry, ModuleInfo, RegistryConfig},
    circuit_breaker::{CircuitBreakerRegistry, CircuitBreakerConfig},
    retry::{RetryExecutor, RetryConfig},
    dead_letter_queue::{DeadLetterQueue, DeadLetterReason},
    error_logging::{ErrorLogger, ErrorContext, ErrorSeverity, ErrorCategory, CorrelationId},
    recovery::{RecoverySystem, DefaultRecoveryExecutor},
};

/// Enhanced event bus implementation with comprehensive error handling
pub struct EnhancedEventBus {
    /// Core event bus components
    router: Arc<MessageRouter>,
    registry: Arc<ModuleRegistry>,
    config: EventBusConfig,
    module_receivers: Arc<parking_lot::RwLock<HashMap<ModuleId, Receiver<BusMessage>>>>,
    is_shutdown: Arc<parking_lot::RwLock<bool>>,
    
    /// Error handling components
    circuit_breakers: Arc<CircuitBreakerRegistry>,
    retry_executor: Arc<RetryExecutor>,
    dead_letter_queue: Arc<DeadLetterQueue>,
    error_logger: Arc<ErrorLogger>,
    recovery_system: Arc<RecoverySystem>,
    
    /// Correlation tracking
    active_correlations: Arc<parking_lot::RwLock<HashMap<MessageId, CorrelationId>>>,
}

impl EnhancedEventBus {
    /// Create a new enhanced event bus instance
    pub fn new(config: EventBusConfig) -> EventBusResult<Self> {
        let router_config = RouterConfig {
            max_queue_size: config.max_queue_size,
            delivery_timeout: config.delivery_timeout,
            worker_threads: 4,
            direct_channel_buffer: 1_000,
        };

        let router = Arc::new(MessageRouter::new(router_config));
        
        let registry_config = RegistryConfig::default();
        let registry = Arc::new(ModuleRegistry::new(registry_config));

        // Initialize error handling components if enabled
        let (circuit_breakers, retry_executor, dead_letter_queue, error_logger, recovery_system) = 
            if config.enable_error_handling {
                let circuit_breakers = Arc::new(CircuitBreakerRegistry::new());
                
                let retry_config = config.retry_config.clone().unwrap_or_default();
                let retry_executor = Arc::new(RetryExecutor::new(retry_config)
                    .map_err(|e| EventBusError::Configuration(format!("Failed to create retry executor: {:?}", e)))?);
                
                let dead_letter_queue = Arc::new(DeadLetterQueue::new(
                    crate::dead_letter_queue::DeadLetterQueueConfig {
                        max_entries: config.dead_letter_queue_size,
                        ..Default::default()
                    }
                ));
                
                let error_logger_config = config.error_logging_config.clone().unwrap_or_default();
                let error_logger = Arc::new(ErrorLogger::new(error_logger_config));
                
                let recovery_config = config.recovery_config.clone().unwrap_or_default();
                let recovery_system = Arc::new(RecoverySystem::new(
                    recovery_config,
                    circuit_breakers.clone(),
                    retry_executor.clone(),
                    dead_letter_queue.clone(),
                    error_logger.clone(),
                ));
                
                // Register default recovery executor
                let default_executor = Arc::new(DefaultRecoveryExecutor::new(
                    circuit_breakers.clone(),
                    retry_executor.clone(),
                ));
                recovery_system.register_executor(default_executor);
                
                (circuit_breakers, retry_executor, dead_letter_queue, error_logger, recovery_system)
            } else {
                // Create minimal implementations when error handling is disabled
                let circuit_breakers = Arc::new(CircuitBreakerRegistry::new());
                let retry_executor = Arc::new(RetryExecutor::new(RetryConfig::default())
                    .map_err(|e| EventBusError::Configuration(format!("Failed to create retry executor: {:?}", e)))?);
                let dead_letter_queue = Arc::new(DeadLetterQueue::new(Default::default()));
                let error_logger = Arc::new(ErrorLogger::new(Default::default()));
                let recovery_system = Arc::new(RecoverySystem::new(
                    Default::default(),
                    circuit_breakers.clone(),
                    retry_executor.clone(),
                    dead_letter_queue.clone(),
                    error_logger.clone(),
                ));
                
                (circuit_breakers, retry_executor, dead_letter_queue, error_logger, recovery_system)
            };

        Ok(Self {
            router,
            registry,
            config,
            module_receivers: Arc::new(parking_lot::RwLock::new(HashMap::new())),
            is_shutdown: Arc::new(parking_lot::RwLock::new(false)),
            circuit_breakers,
            retry_executor,
            dead_letter_queue,
            error_logger,
            recovery_system,
            active_correlations: Arc::new(parking_lot::RwLock::new(HashMap::new())),
        })
    }

    /// Start the enhanced event bus
    pub async fn start(&self) -> EventBusResult<()> {
        if *self.is_shutdown.read() {
            return Err(EventBusError::BusShuttingDown);
        }

        info!("Starting enhanced event bus with error handling");
        
        // Start core router
        self.router.start().await?;
        
        // Load dead letter queue from disk if persistence is enabled
        if let Err(e) = self.dead_letter_queue.load_from_disk() {
            warn!("Failed to load dead letter queue from disk: {}", e);
        }
        
        info!("Enhanced event bus started successfully");
        Ok(())
    }

    /// Publish a message with enhanced error handling
    async fn publish_with_error_handling(&self, message: BusMessage) -> EventBusResult<MessageId> {
        let correlation_id = ErrorLogger::create_correlation_id();
        let operation_context = self.error_logger.start_operation(correlation_id, "publish_message");
        
        // Store correlation for tracking
        self.active_correlations.write().insert(message.id, correlation_id);
        
        debug!("Publishing message {} with correlation {}", message.id, correlation_id);

        // Check if any circuit breakers are open for this operation
        let circuit_name = format!("publish_{:?}", message.source);
        if let Some(breaker) = self.circuit_breakers.get(&circuit_name) {
            if !breaker.is_healthy() {
                let error = EventBusError::Internal("Circuit breaker is open".to_string());
                operation_context.complete_with_error(
                    message.source,
                    ErrorSeverity::Warning,
                    ErrorCategory::Network,
                    error.to_string(),
                );
                return Err(error);
            }
        }

        // Attempt to publish with retry logic
        let message_id = message.id;
        let result = self.retry_executor.execute_with_default(|attempt| {
            let router = self.router.clone();
            let msg = message.clone();
            Box::pin(async move {
                debug!("Publishing attempt {} for message {}", attempt.attempt_number, msg.id);
                router.publish(msg).await
            })
        }).await;

        match result {
            Ok(_) => {
                operation_context.complete();
                debug!("Successfully published message {}", message_id);
                Ok(message_id)
            }
            Err(retry_error) => {
                // Send to dead letter queue
                let dlq_reason = match &retry_error {
                    crate::retry::RetryError::MaxAttemptsExceeded { max_attempts } => {
                        DeadLetterReason::MaxRetriesExceeded { attempts: *max_attempts }
                    }
                    crate::retry::RetryError::TimeoutExceeded { timeout } => {
                        DeadLetterReason::DeliveryTimeout { timeout: *timeout }
                    }
                    _ => DeadLetterReason::SystemError { error: retry_error.to_string() }
                };

                let message_source = message.source;
                let _dlq_id = self.dead_letter_queue.add_message(
                    message,
                    dlq_reason,
                    self.config.max_retry_attempts,
                    vec![], // No specific recipients for publish failures
                    Some(retry_error.to_string()),
                    Some(correlation_id.to_string()),
                );

                let bus_error = EventBusError::Internal(format!("Message publish failed: {}", retry_error));
                
                // Trigger recovery if configured
                if self.config.enable_error_handling {
                    if let Err(e) = self.recovery_system.handle_incident(
                        correlation_id,
                        message_source,
                        &bus_error,
                        format!("Message publish failed after retries: {}", retry_error),
                    ).await {
                        error!("Failed to handle recovery incident: {}", e);
                    }
                }

                operation_context.complete_with_error(
                    message_source,
                    ErrorSeverity::Error,
                    ErrorCategory::Network,
                    bus_error.to_string(),
                );

                Err(bus_error)
            }
        }
    }

    /// Subscribe with enhanced error handling
    async fn subscribe_with_error_handling(
        &self,
        subscriber: ModuleId,
        filter: MessageFilter,
        delivery_mode: DeliveryMode,
    ) -> EventBusResult<SubscriptionId> {
        let correlation_id = ErrorLogger::create_correlation_id();
        let operation_context = self.error_logger.start_operation(correlation_id, "subscribe");

        debug!("Creating subscription for module {} with correlation {}", subscriber, correlation_id);

        // Create channel based on delivery mode
        let buffer_size = match delivery_mode {
            DeliveryMode::Reliable { .. } => self.config.max_queue_size / 4,
            DeliveryMode::BestEffort => self.config.max_queue_size / 8,
            DeliveryMode::LatestOnly => 1,
        };

        let (sender, receiver) = bounded(buffer_size);

        // Create subscription
        let subscription = Subscription::new(subscriber, filter, delivery_mode, sender);
        let subscription_id = subscription.id;

        // Register with router
        self.router.subscription_manager().add_subscription(subscription);
        
        // Record metrics
        self.router.metrics().record_subscription_created(subscriber);

        // Store receiver
        self.module_receivers.write().insert(subscriber, receiver);

        operation_context.complete();
        debug!("Created subscription {} for module {}", subscription_id, subscriber);

        Ok(subscription_id)
    }

    /// Get error handling statistics
    pub fn get_error_stats(&self) -> ErrorHandlingStats {
        ErrorHandlingStats {
            circuit_breaker_stats: self.circuit_breakers.all_stats(),
            retry_stats: self.retry_executor.stats(),
            dead_letter_stats: self.dead_letter_queue.stats(),
            error_logging_stats: self.error_logger.stats(),
            recovery_stats: self.recovery_system.stats(),
        }
    }

    /// Get circuit breaker registry
    pub fn circuit_breakers(&self) -> &Arc<CircuitBreakerRegistry> {
        &self.circuit_breakers
    }

    /// Get dead letter queue
    pub fn dead_letter_queue(&self) -> &Arc<DeadLetterQueue> {
        &self.dead_letter_queue
    }

    /// Get recovery system
    pub fn recovery_system(&self) -> &Arc<RecoverySystem> {
        &self.recovery_system
    }

    /// Get error logger
    pub fn error_logger(&self) -> &Arc<ErrorLogger> {
        &self.error_logger
    }

    /// Replay dead letter messages
    pub async fn replay_dead_letters(&self) -> EventBusResult<Vec<crate::dead_letter_queue::ReplayResult>> {
        info!("Starting dead letter message replay");
        
        let results = self.dead_letter_queue.replay_marked_messages(|message| {
            let router = self.router.clone();
            let msg = message.clone();
            Box::pin(async move {
                router.publish(msg).await.map(|_| ()).map_err(|e| e.into())
            })
        }).await;

        info!("Replayed {} dead letter messages", results.len());
        Ok(results)
    }

    /// Force cleanup of old dead letter entries
    pub fn cleanup_dead_letters(&self) -> usize {
        self.dead_letter_queue.cleanup_old_entries()
    }

    /// Create a receiver channel for a specific module
    pub fn create_module_receiver(&self, module: ModuleId, buffer_size: usize) -> Receiver<BusMessage> {
        let (sender, receiver) = bounded(buffer_size);
        
        // Register the sender with the router for direct channels if needed
        self.router.register_direct_channel(ModuleId::EventBus, module, sender);
        
        // Store the receiver for this module
        self.module_receivers.write().insert(module, receiver.clone());
        
        receiver
    }

    /// Register a module with the event bus
    pub fn register_module(&self, module_info: ModuleInfo) -> EventBusResult<()> {
        self.registry.register_module(module_info)
    }

    /// Unregister a module
    pub fn unregister_module(&self, module_id: ModuleId) -> EventBusResult<ModuleInfo> {
        self.registry.unregister_module(module_id)
    }

    /// Get module registry
    pub fn registry(&self) -> &Arc<ModuleRegistry> {
        &self.registry
    }

    /// Get a receiver for a subscription (mock implementation for tests)
    pub async fn get_receiver(&self, _subscription_id: SubscriptionId) -> EventBusResult<Receiver<BusMessage>> {
        // This is a placeholder implementation for testing
        let (_, receiver) = bounded(1000);
        Ok(receiver)
    }
}

#[async_trait]
impl EventBusTrait for EnhancedEventBus {
    async fn publish(&self, message: BusMessage) -> EventBusResult<MessageId> {
        if *self.is_shutdown.read() {
            return Err(EventBusError::BusShuttingDown);
        }

        if self.config.enable_error_handling {
            self.publish_with_error_handling(message).await
        } else {
            // Fallback to simple publish
            let message_id = message.id;
            self.router.publish(message).await?;
            Ok(message_id)
        }
    }

    async fn subscribe(
        &self,
        subscriber: ModuleId,
        filter: MessageFilter,
        delivery_mode: DeliveryMode,
    ) -> EventBusResult<SubscriptionId> {
        if *self.is_shutdown.read() {
            return Err(EventBusError::BusShuttingDown);
        }

        if self.config.enable_error_handling {
            self.subscribe_with_error_handling(subscriber, filter, delivery_mode).await
        } else {
            // Fallback to simple subscribe
            let buffer_size = match delivery_mode {
                DeliveryMode::Reliable { .. } => self.config.max_queue_size / 4,
                DeliveryMode::BestEffort => self.config.max_queue_size / 8,
                DeliveryMode::LatestOnly => 1,
            };

            let (sender, receiver) = bounded(buffer_size);
            let subscription = Subscription::new(subscriber, filter, delivery_mode, sender);
            let subscription_id = subscription.id;

            self.router.subscription_manager().add_subscription(subscription);
            self.router.metrics().record_subscription_created(subscriber);
            self.module_receivers.write().insert(subscriber, receiver);

            Ok(subscription_id)
        }
    }

    async fn unsubscribe(&self, subscription_id: SubscriptionId) -> EventBusResult<()> {
        debug!("Removing subscription {}", subscription_id);

        let removed = self.router.subscription_manager().remove_subscription(subscription_id);
        
        if removed {
            debug!("Successfully removed subscription {}", subscription_id);
            Ok(())
        } else {
            warn!("Subscription {} not found", subscription_id);
            Err(EventBusError::SubscriptionNotFound { subscription_id })
        }
    }

    async fn metrics(&self) -> EventBusResult<BusMetrics> {
        // Collect subscription counts per module
        let subscription_stats = self.router.subscription_manager().get_stats();
        let mut subscription_counts = HashMap::new();
        
        for (_, module, _) in subscription_stats {
            *subscription_counts.entry(module).or_insert(0) += 1;
        }

        Ok(self.router.metrics().snapshot(subscription_counts))
    }

    async fn shutdown(&self) -> EventBusResult<()> {
        info!("Shutting down enhanced event bus");

        {
            let mut shutdown = self.is_shutdown.write();
            if *shutdown {
                return Ok(()); // Already shut down
            }
            *shutdown = true;
        }

        // Cleanup dead letter queue before shutdown
        let cleanup_count = self.cleanup_dead_letters();
        if cleanup_count > 0 {
            info!("Cleaned up {} old dead letter entries during shutdown", cleanup_count);
        }

        // Stop the router
        self.router.stop().await?;

        // Clear all receivers
        self.module_receivers.write().clear();
        
        // Clear active correlations
        self.active_correlations.write().clear();

        info!("Enhanced event bus shutdown complete");
        Ok(())
    }
}

/// Statistics from all error handling components
#[derive(Debug, Clone)]
pub struct ErrorHandlingStats {
    pub circuit_breaker_stats: HashMap<String, crate::circuit_breaker::CircuitBreakerStats>,
    pub retry_stats: crate::retry::RetryStats,
    pub dead_letter_stats: crate::dead_letter_queue::DeadLetterStats,
    pub error_logging_stats: crate::error_logging::ErrorStats,
    pub recovery_stats: crate::recovery::RecoveryStats,
}

/// Type alias for enhanced event bus
pub type EnhancedEventBusArc = Arc<EnhancedEventBus>;

/// Create an enhanced event bus with default configuration
pub fn create_enhanced_event_bus() -> EventBusResult<EnhancedEventBusArc> {
    let config = EventBusConfig::default();
    create_enhanced_event_bus_with_config(config)
}

/// Create an enhanced event bus with custom configuration
pub fn create_enhanced_event_bus_with_config(config: EventBusConfig) -> EventBusResult<EnhancedEventBusArc> {
    let bus = EnhancedEventBus::new(config)?;
    Ok(Arc::new(bus))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::{MessagePayload, MessagePriority, RawEvent};
    use chrono::Utc;

    #[tokio::test]
    async fn test_enhanced_event_bus_creation() {
        let config = EventBusConfig::default();
        let bus = create_enhanced_event_bus_with_config(config).unwrap();
        assert!(!*bus.is_shutdown.read());
    }

    #[tokio::test]
    async fn test_enhanced_publish_with_error_handling() {
        let bus = create_enhanced_event_bus().unwrap();
        bus.start().await.unwrap();

        let raw_event = RawEvent {
            event_type: "test".to_string(),
            data: serde_json::json!({"key": "value"}),
            window_title: Some("Test Window".to_string()),
            timestamp: Utc::now(),
        };

        let message = BusMessage::with_priority(
            ModuleId::DataCapture,
            MessagePayload::RawEvent(raw_event),
            MessagePriority::Normal,
        );

        let result = bus.publish(message).await;
        assert!(result.is_ok());

        let stats = bus.get_error_stats();
        assert_eq!(stats.retry_stats.total_operations, 1);
        assert_eq!(stats.retry_stats.successful_operations, 1);

        bus.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn test_enhanced_subscribe_with_error_handling() {
        let bus = create_enhanced_event_bus().unwrap();
        bus.start().await.unwrap();

        let filter = MessageFilter::types(vec![crate::MessageType::RawEvent]);
        let subscription_id = bus.subscribe(
            ModuleId::Storage,
            filter,
            DeliveryMode::BestEffort,
        ).await.unwrap();

        let result = bus.unsubscribe(subscription_id).await;
        assert!(result.is_ok());

        bus.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn test_error_handling_disabled() {
        let config = EventBusConfig {
            enable_error_handling: false,
            ..EventBusConfig::default()
        };
        
        let bus = create_enhanced_event_bus_with_config(config).unwrap();
        bus.start().await.unwrap();

        let message = BusMessage::new(
            ModuleId::DataCapture,
            MessagePayload::ModuleReady(ModuleId::DataCapture),
        );

        let result = bus.publish(message).await;
        assert!(result.is_ok());

        bus.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn test_circuit_breaker_integration() {
        let bus = create_enhanced_event_bus().unwrap();
        bus.start().await.unwrap();

        // Register a circuit breaker for testing
        let breaker = bus.circuit_breakers().register(
            "test_circuit".to_string(),
            crate::circuit_breaker::CircuitBreakerConfig::default(),
        );

        // Force the circuit breaker open
        breaker.force_open();
        assert!(!breaker.is_healthy());

        // Close it again
        breaker.force_close();
        assert!(breaker.is_healthy());

        bus.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn test_dead_letter_queue_integration() {
        let bus = create_enhanced_event_bus().unwrap();
        bus.start().await.unwrap();

        // Add a message to the dead letter queue manually for testing
        let message = BusMessage::new(
            ModuleId::DataCapture,
            MessagePayload::ModuleReady(ModuleId::DataCapture),
        );

        let _dlq_id = bus.dead_letter_queue().add_message(
            message,
            DeadLetterReason::MaxRetriesExceeded { attempts: 3 },
            3,
            vec![ModuleId::Storage],
            Some("Test error".to_string()),
            None,
        );

        let stats = bus.dead_letter_queue().stats();
        assert_eq!(stats.total_entries, 1);

        bus.shutdown().await.unwrap();
    }
}