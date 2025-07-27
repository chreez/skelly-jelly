//! Subscription management for the event bus

use std::time::Duration;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::{MessageType, ModuleId, BusMessage};

/// Unique identifier for a subscription
pub type SubscriptionId = Uuid;

/// Message delivery modes with different guarantees
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeliveryMode {
    /// Guaranteed delivery with acknowledgment
    Reliable { timeout: Duration },
    
    /// Best effort, no acknowledgment required
    BestEffort,
    
    /// Latest value only (for status updates)
    LatestOnly,
}

impl Default for DeliveryMode {
    fn default() -> Self {
        DeliveryMode::BestEffort
    }
}

/// Filter for selecting which messages to receive
pub struct MessageFilter {
    /// Filter by message types
    pub types: Option<Vec<MessageType>>,
    
    /// Filter by source modules
    pub sources: Option<Vec<ModuleId>>,
    
    /// Custom predicate function for advanced filtering
    pub predicate: Option<Box<dyn Fn(&BusMessage) -> bool + Send + Sync>>,
}

impl std::fmt::Debug for MessageFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MessageFilter")
            .field("types", &self.types)
            .field("sources", &self.sources)
            .field("predicate", &if self.predicate.is_some() { &"Some(function)" } else { &"None" })
            .finish()
    }
}

impl MessageFilter {
    /// Create a filter that accepts all messages
    pub fn all() -> Self {
        Self {
            types: None,
            sources: None,
            predicate: None,
        }
    }

    /// Create a filter for specific message types
    pub fn types(types: Vec<MessageType>) -> Self {
        Self {
            types: Some(types),
            sources: None,
            predicate: None,
        }
    }

    /// Create a filter for specific source modules
    pub fn sources(sources: Vec<ModuleId>) -> Self {
        Self {
            types: None,
            sources: Some(sources),
            predicate: None,
        }
    }

    /// Create a filter combining types and sources
    pub fn types_and_sources(types: Vec<MessageType>, sources: Vec<ModuleId>) -> Self {
        Self {
            types: Some(types),
            sources: Some(sources),
            predicate: None,
        }
    }

    /// Add a custom predicate to the filter
    pub fn with_predicate<F>(mut self, predicate: F) -> Self 
    where
        F: Fn(&BusMessage) -> bool + Send + Sync + 'static,
    {
        self.predicate = Some(Box::new(predicate));
        self
    }

    /// Check if a message matches this filter
    pub fn matches(&self, message: &BusMessage) -> bool {
        // Check message type filter
        if let Some(ref types) = self.types {
            if !types.contains(&message.message_type()) {
                return false;
            }
        }

        // Check source module filter
        if let Some(ref sources) = self.sources {
            if !sources.contains(&message.source) {
                return false;
            }
        }

        // Check custom predicate
        if let Some(ref predicate) = self.predicate {
            if !predicate(message) {
                return false;
            }
        }

        true
    }
}

/// Represents an active subscription
#[derive(Debug)]
pub struct Subscription {
    /// Unique subscription identifier
    pub id: SubscriptionId,
    
    /// The module that owns this subscription
    pub subscriber: ModuleId,
    
    /// Filter for selecting messages
    pub filter: MessageFilter,
    
    /// Delivery mode and guarantees
    pub delivery_mode: DeliveryMode,
    
    /// Channel for sending messages to subscriber
    pub sender: crossbeam_channel::Sender<BusMessage>,
    
    /// When this subscription was created
    pub created_at: std::time::SystemTime,
    
    /// Statistics for this subscription
    pub stats: SubscriptionStats,
}

impl Subscription {
    /// Create a new subscription
    pub fn new(
        subscriber: ModuleId,
        filter: MessageFilter,
        delivery_mode: DeliveryMode,
        sender: crossbeam_channel::Sender<BusMessage>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            subscriber,
            filter,
            delivery_mode,
            sender,
            created_at: std::time::SystemTime::now(),
            stats: SubscriptionStats::default(),
        }
    }

    /// Check if this subscription is interested in a message
    pub fn wants_message(&self, message: &BusMessage) -> bool {
        self.filter.matches(message)
    }

    /// Try to deliver a message to this subscription
    pub fn try_deliver(&mut self, message: BusMessage) -> Result<(), DeliveryError> {
        self.stats.messages_attempted += 1;

        match self.sender.try_send(message) {
            Ok(_) => {
                self.stats.messages_delivered += 1;
                self.stats.last_delivery = Some(std::time::SystemTime::now());
                Ok(())
            }
            Err(crossbeam_channel::TrySendError::Full(_)) => {
                self.stats.messages_dropped += 1;
                Err(DeliveryError::QueueFull)
            }
            Err(crossbeam_channel::TrySendError::Disconnected(_)) => {
                self.stats.messages_dropped += 1;
                Err(DeliveryError::Disconnected)
            }
        }
    }
}

/// Statistics for a subscription
#[derive(Debug, Default)]
pub struct SubscriptionStats {
    pub messages_attempted: u64,
    pub messages_delivered: u64,
    pub messages_dropped: u64,
    pub last_delivery: Option<std::time::SystemTime>,
}

/// Errors that can occur during message delivery
#[derive(Debug)]
pub enum DeliveryError {
    QueueFull,
    Disconnected,
    Timeout,
}

/// Manager for all subscriptions in the system
#[derive(Debug)]
pub struct SubscriptionManager {
    subscriptions: parking_lot::RwLock<Vec<Subscription>>,
}

impl SubscriptionManager {
    /// Create a new subscription manager
    pub fn new() -> Self {
        Self {
            subscriptions: parking_lot::RwLock::new(Vec::new()),
        }
    }

    /// Add a new subscription
    pub fn add_subscription(&self, subscription: Subscription) -> SubscriptionId {
        let id = subscription.id;
        self.subscriptions.write().push(subscription);
        id
    }

    /// Remove a subscription by ID
    pub fn remove_subscription(&self, subscription_id: SubscriptionId) -> bool {
        let mut subscriptions = self.subscriptions.write();
        if let Some(pos) = subscriptions.iter().position(|s| s.id == subscription_id) {
            subscriptions.remove(pos);
            true
        } else {
            false
        }
    }

    /// Get all subscriptions for a specific module
    pub fn get_subscriptions_for_module(&self, module: ModuleId) -> Vec<SubscriptionId> {
        self.subscriptions
            .read()
            .iter()
            .filter(|s| s.subscriber == module)
            .map(|s| s.id)
            .collect()
    }

    /// Deliver a message to all interested subscriptions
    pub fn deliver_message(&self, message: BusMessage) -> DeliveryResults {
        let mut results = DeliveryResults::default();
        let mut subscriptions = self.subscriptions.write();

        for subscription in subscriptions.iter_mut() {
            if subscription.wants_message(&message) {
                match subscription.try_deliver(message.clone()) {
                    Ok(_) => results.successful += 1,
                    Err(DeliveryError::QueueFull) => results.queue_full += 1,
                    Err(DeliveryError::Disconnected) => {
                        results.disconnected += 1;
                        // Mark for removal - we'll clean up disconnected subscriptions
                    }
                    Err(DeliveryError::Timeout) => results.timeout += 1,
                }
            }
        }

        // Remove disconnected subscriptions
        // Note: crossbeam-channel doesn't have is_disconnected(), so we'll handle this differently
        // For now, we'll rely on the try_send errors to identify disconnected channels

        results
    }

    /// Get the total number of active subscriptions
    pub fn subscription_count(&self) -> usize {
        self.subscriptions.read().len()
    }

    /// Get statistics for all subscriptions
    pub fn get_stats(&self) -> Vec<(SubscriptionId, ModuleId, SubscriptionStats)> {
        self.subscriptions
            .read()
            .iter()
            .map(|s| (s.id, s.subscriber, SubscriptionStats {
                messages_attempted: s.stats.messages_attempted,
                messages_delivered: s.stats.messages_delivered,
                messages_dropped: s.stats.messages_dropped,
                last_delivery: s.stats.last_delivery,
            }))
            .collect()
    }
}

impl Default for SubscriptionManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Results from attempting to deliver a message
#[derive(Debug, Default)]
pub struct DeliveryResults {
    pub successful: u32,
    pub queue_full: u32,
    pub disconnected: u32,
    pub timeout: u32,
}

impl DeliveryResults {
    pub fn total_attempted(&self) -> u32 {
        self.successful + self.queue_full + self.disconnected + self.timeout
    }

    pub fn success_rate(&self) -> f64 {
        let total = self.total_attempted();
        if total == 0 {
            1.0
        } else {
            self.successful as f64 / total as f64
        }
    }
}