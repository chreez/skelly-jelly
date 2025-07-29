//! Main event bus implementation

use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use crossbeam_channel::{bounded, Receiver};
use tracing::{debug, info, warn};

use crate::{
    BusMessage, EventBusConfig, EventBusError, EventBusResult, EventBusTrait,
    MessageId, ModuleId, SubscriptionId,
    subscription::{DeliveryMode, MessageFilter, Subscription},
    router::{MessageRouter, RouterConfig},
    metrics::BusMetrics,
    registry::{ModuleRegistry, ModuleInfo, RegistryConfig},
};

/// Main event bus implementation
pub struct EventBusImpl {
    /// Message router for handling delivery
    router: Arc<MessageRouter>,
    
    /// Module registry for tracking registered modules
    registry: Arc<ModuleRegistry>,
    
    /// Configuration
    config: EventBusConfig,
    
    /// Channels for modules to receive messages
    module_receivers: Arc<parking_lot::RwLock<HashMap<ModuleId, Receiver<BusMessage>>>>,
    
    /// Shutdown state
    is_shutdown: Arc<parking_lot::RwLock<bool>>,
}

impl EventBusImpl {
    /// Create a new event bus instance
    pub fn new(config: EventBusConfig) -> EventBusResult<Self> {
        let router_config = RouterConfig {
            max_queue_size: config.max_queue_size,
            delivery_timeout: config.delivery_timeout,
            worker_threads: 4, // Could be configurable
            direct_channel_buffer: 1_000,
        };

        let router = Arc::new(MessageRouter::new(router_config));
        
        let registry_config = RegistryConfig::default();
        let registry = Arc::new(ModuleRegistry::new(registry_config));

        Ok(Self {
            router,
            registry,
            config,
            module_receivers: Arc::new(parking_lot::RwLock::new(HashMap::new())),
            is_shutdown: Arc::new(parking_lot::RwLock::new(false)),
        })
    }

    /// Start the event bus
    pub async fn start(&self) -> EventBusResult<()> {
        if *self.is_shutdown.read() {
            return Err(EventBusError::BusShuttingDown);
        }

        info!("Starting event bus");
        self.router.start().await?;
        info!("Event bus started successfully");
        Ok(())
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
        // In a real implementation, you'd want to return the actual receiver for the subscription
        let (_, receiver) = bounded(1000);
        Ok(receiver)
    }
}

#[async_trait]
impl EventBusTrait for EventBusImpl {
    async fn publish(&self, message: BusMessage) -> EventBusResult<MessageId> {
        if *self.is_shutdown.read() {
            return Err(EventBusError::BusShuttingDown);
        }

        debug!("Publishing message {} from {}", message.id, message.source);
        
        let message_id = message.id;
        self.router.publish(message).await?;
        
        Ok(message_id)
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

        debug!("Creating subscription for module {}", subscriber);

        // Create a channel for this subscription
        let buffer_size = match delivery_mode {
            DeliveryMode::Reliable { .. } => self.config.max_queue_size / 4, // Larger buffer for reliable delivery
            DeliveryMode::BestEffort => self.config.max_queue_size / 8,       // Medium buffer
            DeliveryMode::LatestOnly => 1,                                    // Minimal buffer, only latest value
        };

        let (sender, receiver) = bounded(buffer_size);

        // Create the subscription
        let subscription = Subscription::new(subscriber, filter, delivery_mode, sender);
        let subscription_id = subscription.id;

        // Register with the subscription manager
        self.router.subscription_manager().add_subscription(subscription);
        
        // Record subscription creation in metrics
        self.router.metrics().record_subscription_created(subscriber);

        debug!("Created subscription {} for module {}", subscription_id, subscriber);

        // Note: In a real implementation, you'd want to return the receiver to the subscriber
        // This might involve storing it in a registry that modules can query
        // For now, we'll store it in our internal registry
        self.module_receivers.write().insert(subscriber, receiver);

        Ok(subscription_id)
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
        info!("Shutting down event bus");

        {
            let mut shutdown = self.is_shutdown.write();
            if *shutdown {
                return Ok(()); // Already shut down
            }
            *shutdown = true;
        }

        // Stop the router
        self.router.stop().await?;

        // Clear all receivers
        self.module_receivers.write().clear();

        info!("Event bus shutdown complete");
        Ok(())
    }
}

/// Convenient type alias for the main event bus implementation
pub type EventBus = Arc<EventBusImpl>;

/// Create a new event bus with default configuration
pub fn create_event_bus() -> EventBusResult<EventBus> {
    let config = EventBusConfig::default();
    create_event_bus_with_config(config)
}

/// Create a new event bus with custom configuration
pub fn create_event_bus_with_config(config: EventBusConfig) -> EventBusResult<EventBus> {
    let bus = EventBusImpl::new(config)?;
    Ok(Arc::new(bus))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::{MessagePayload, MessagePriority, RawEvent};
    use crate::subscription::MessageFilter;
    use chrono::Utc;

    #[tokio::test]
    async fn test_create_event_bus() {
        let bus = create_event_bus().unwrap();
        assert!(!*bus.is_shutdown.read());
    }

    #[tokio::test]
    async fn test_start_and_shutdown() {
        let bus = create_event_bus().unwrap();
        
        // Start the bus
        bus.start().await.unwrap();
        
        // Shutdown the bus
        bus.shutdown().await.unwrap();
        
        assert!(*bus.is_shutdown.read());
    }

    #[tokio::test]
    async fn test_publish_message() {
        let bus = create_event_bus().unwrap();
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

        bus.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn test_subscribe_and_unsubscribe() {
        let bus = create_event_bus().unwrap();
        bus.start().await.unwrap();

        // Create a subscription
        let filter = MessageFilter::types(vec![crate::MessageType::RawEvent]);
        let subscription_id = bus.subscribe(
            ModuleId::Storage,
            filter,
            DeliveryMode::BestEffort,
        ).await.unwrap();

        // Unsubscribe
        let result = bus.unsubscribe(subscription_id).await;
        assert!(result.is_ok());

        // Try to unsubscribe again (should fail)
        let result = bus.unsubscribe(subscription_id).await;
        assert!(matches!(result, Err(EventBusError::SubscriptionNotFound { .. })));

        bus.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn test_metrics() {
        let bus = create_event_bus().unwrap();
        bus.start().await.unwrap();

        let metrics = bus.metrics().await.unwrap();
        assert_eq!(metrics.messages_published, 0);
        assert_eq!(metrics.messages_delivered, 0);

        bus.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn test_shutdown_prevents_operations() {
        let bus = create_event_bus().unwrap();
        bus.start().await.unwrap();
        bus.shutdown().await.unwrap();

        // Try to publish after shutdown
        let message = BusMessage::new(
            ModuleId::DataCapture,
            MessagePayload::ModuleReady(ModuleId::DataCapture),
        );

        let result = bus.publish(message).await;
        assert!(matches!(result, Err(EventBusError::BusShuttingDown)));

        // Try to subscribe after shutdown
        let filter = MessageFilter::all();
        let result = bus.subscribe(ModuleId::Storage, filter, DeliveryMode::BestEffort).await;
        assert!(matches!(result, Err(EventBusError::BusShuttingDown)));
    }
}