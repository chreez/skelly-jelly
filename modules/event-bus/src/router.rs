//! Message routing implementation for the event bus

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use crossbeam_channel::{Receiver, Sender};
use tracing::{debug, error, warn};

use crate::{
    BusMessage, EventBusError, EventBusResult, MessageId, ModuleId,
    subscription::SubscriptionManager,
    metrics::MetricsCollector,
};

/// High-performance message router
pub struct MessageRouter {
    /// Subscription manager for tracking all active subscriptions
    subscription_manager: Arc<SubscriptionManager>,
    
    /// Metrics collector for performance monitoring
    metrics: Arc<MetricsCollector>,
    
    /// Direct channels for high-frequency module-to-module communication
    direct_channels: Arc<parking_lot::RwLock<HashMap<(ModuleId, ModuleId), Sender<BusMessage>>>>,
    
    /// Main message queue for async delivery
    message_queue: (Sender<QueuedMessage>, Receiver<QueuedMessage>),
    
    /// Configuration
    config: RouterConfig,
    
    /// Shutdown signal
    shutdown_signal: Arc<tokio::sync::Notify>,
    
    /// Router state
    is_running: Arc<parking_lot::RwLock<bool>>,
}

/// Configuration for the message router
#[derive(Debug, Clone)]
pub struct RouterConfig {
    /// Maximum size of the message queue
    pub max_queue_size: usize,
    
    /// Timeout for message delivery
    pub delivery_timeout: Duration,
    
    /// Number of worker threads for message processing
    pub worker_threads: usize,
    
    /// Buffer size for direct channels
    pub direct_channel_buffer: usize,
}

impl Default for RouterConfig {
    fn default() -> Self {
        Self {
            max_queue_size: 10_000,
            delivery_timeout: Duration::from_secs(5),
            worker_threads: 4,
            direct_channel_buffer: 1_000,
        }
    }
}

/// A message in the routing queue with metadata
#[derive(Debug)]
struct QueuedMessage {
    message: BusMessage,
    queued_at: SystemTime,
    retry_count: u32,
}

impl MessageRouter {
    /// Create a new message router
    pub fn new(config: RouterConfig) -> Self {
        let (sender, receiver) = crossbeam_channel::bounded(config.max_queue_size);
        
        Self {
            subscription_manager: Arc::new(SubscriptionManager::new()),
            metrics: Arc::new(MetricsCollector::new()),
            direct_channels: Arc::new(parking_lot::RwLock::new(HashMap::new())),
            message_queue: (sender, receiver),
            config,
            shutdown_signal: Arc::new(tokio::sync::Notify::new()),
            is_running: Arc::new(parking_lot::RwLock::new(false)),
        }
    }

    /// Start the message router
    pub async fn start(&self) -> EventBusResult<()> {
        {
            let mut running = self.is_running.write();
            if *running {
                return Err(EventBusError::Internal("Router already running".to_string()));
            }
            *running = true;
        }

        debug!("Starting message router with {} worker threads", self.config.worker_threads);

        // Start worker tasks
        for worker_id in 0..self.config.worker_threads {
            let receiver = self.message_queue.1.clone();
            let subscription_manager = Arc::clone(&self.subscription_manager);
            let metrics = Arc::clone(&self.metrics);
            let shutdown_signal = Arc::clone(&self.shutdown_signal);
            let config = self.config.clone();

            tokio::spawn(async move {
                Self::worker_loop(
                    worker_id,
                    receiver,
                    subscription_manager,
                    metrics,
                    shutdown_signal,
                    config,
                ).await;
            });
        }

        debug!("Message router started successfully");
        Ok(())
    }

    /// Stop the message router
    pub async fn stop(&self) -> EventBusResult<()> {
        debug!("Stopping message router");
        
        {
            let mut running = self.is_running.write();
            *running = false;
        }

        self.shutdown_signal.notify_waiters();
        
        // Give workers time to finish processing current messages
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        debug!("Message router stopped");
        Ok(())
    }

    /// Publish a message through the router
    pub async fn publish(&self, message: BusMessage) -> EventBusResult<MessageId> {
        if !*self.is_running.read() {
            return Err(EventBusError::BusShuttingDown);
        }

        let message_id = message.id;
        let source = message.source;
        let message_type = message.message_type();
        
        // Estimate message size for metrics
        let message_size = estimate_message_size(&message);
        
        // Record publish metrics
        self.metrics.record_publish(source, message_type, message_size);

        // Route based on message type and optimization strategy
        match self.route_message(message).await {
            Ok(_) => {
                debug!("Message {} routed successfully", message_id);
                Ok(message_id)
            }
            Err(e) => {
                error!("Failed to route message {}: {}", message_id, e);
                self.metrics.record_failure(source, message_type);
                Err(e)
            }
        }
    }

    /// Route a message using the optimal strategy
    async fn route_message(&self, message: BusMessage) -> EventBusResult<()> {
        // Check for direct channel optimization
        if let Some(direct_route) = self.find_direct_route(&message) {
            return self.send_direct(direct_route, message).await;
        }

        // Use standard pub-sub routing
        self.queue_for_delivery(message).await
    }

    /// Find a direct channel route for high-frequency messages
    fn find_direct_route(&self, message: &BusMessage) -> Option<(ModuleId, ModuleId)> {
        // Define direct routes for high-frequency message patterns
        match (&message.source, &message.payload) {
            // Data Capture -> Storage for RawEvent messages
            (ModuleId::DataCapture, crate::MessagePayload::RawEvent(_)) => {
                Some((ModuleId::DataCapture, ModuleId::Storage))
            }
            // Add more direct routes as needed
            _ => None,
        }
    }

    /// Send message via direct channel
    async fn send_direct(&self, route: (ModuleId, ModuleId), message: BusMessage) -> EventBusResult<()> {
        {
            let direct_channels = self.direct_channels.read();
            
            if let Some(sender) = direct_channels.get(&route) {
                match sender.try_send(message.clone()) {
                    Ok(_) => return Ok(()),
                    Err(crossbeam_channel::TrySendError::Full(_)) => {
                        // Fall back to standard routing if direct channel is full
                    }
                    Err(crossbeam_channel::TrySendError::Disconnected(_)) => {
                        return Err(EventBusError::ChannelSend("Direct channel disconnected".to_string()));
                    }
                }
            }
        } // Drop the read lock here
        
        // Fall back to standard routing
        self.queue_for_delivery(message).await
    }

    /// Queue message for standard pub-sub delivery
    async fn queue_for_delivery(&self, message: BusMessage) -> EventBusResult<()> {
        let queued_message = QueuedMessage {
            message,
            queued_at: SystemTime::now(),
            retry_count: 0,
        };

        match self.message_queue.0.try_send(queued_message) {
            Ok(_) => {
                self.metrics.update_queue_depth(self.message_queue.1.len());
                Ok(())
            }
            Err(crossbeam_channel::TrySendError::Full(_)) => {
                Err(EventBusError::QueueFull {
                    current_size: self.message_queue.1.len(),
                    max_size: self.config.max_queue_size,
                })
            }
            Err(crossbeam_channel::TrySendError::Disconnected(_)) => {
                Err(EventBusError::ChannelSend("Message queue disconnected".to_string()))
            }
        }
    }

    /// Register a direct channel between two modules
    pub fn register_direct_channel(&self, from: ModuleId, to: ModuleId, sender: Sender<BusMessage>) {
        let mut channels = self.direct_channels.write();
        channels.insert((from, to), sender);
        debug!("Registered direct channel from {} to {}", from, to);
    }

    /// Get subscription manager
    pub fn subscription_manager(&self) -> &Arc<SubscriptionManager> {
        &self.subscription_manager
    }

    /// Get metrics collector
    pub fn metrics(&self) -> &Arc<MetricsCollector> {
        &self.metrics
    }

    /// Worker loop for processing messages
    async fn worker_loop(
        worker_id: usize,
        receiver: Receiver<QueuedMessage>,
        subscription_manager: Arc<SubscriptionManager>,
        metrics: Arc<MetricsCollector>,
        shutdown_signal: Arc<tokio::sync::Notify>,
        _config: RouterConfig,
    ) {
        debug!("Worker {} started", worker_id);

        loop {
            // Check for shutdown signal with timeout
            let recv_result = tokio::select! {
                _ = shutdown_signal.notified() => {
                    debug!("Worker {} received shutdown signal", worker_id);
                    break;
                }
                result = tokio::task::spawn_blocking({
                    let receiver = receiver.clone();
                    move || receiver.recv_timeout(Duration::from_millis(100))
                }) => {
                    match result {
                        Ok(recv_result) => recv_result,
                        Err(_) => continue, // Task was cancelled
                    }
                }
            };

            match recv_result {
                Ok(queued_message) => {
                    let start_time = SystemTime::now();
                    
                    // Deliver the message
                    let results = subscription_manager.deliver_message(queued_message.message.clone());
                    
                    // Record delivery metrics
                    let delivery_latency = start_time.elapsed().unwrap_or_default();
                    let message_type = queued_message.message.message_type();
                    
                    if results.successful > 0 {
                        // Record successful deliveries
                        for _ in 0..results.successful {
                            metrics.record_delivery(
                                queued_message.message.source,
                                message_type,
                                delivery_latency,
                            );
                        }
                    }
                    
                    // Record failures
                    let total_failures = results.queue_full + results.disconnected + results.timeout;
                    for _ in 0..total_failures {
                        metrics.record_failure(queued_message.message.source, message_type);
                    }
                    
                    if results.total_attempted() > 0 {
                        debug!(
                            "Worker {} delivered message {} to {}/{} subscribers (success rate: {:.1}%)",
                            worker_id,
                            queued_message.message.id,
                            results.successful,
                            results.total_attempted(),
                            results.success_rate() * 100.0
                        );
                    }
                    
                    // Handle retries for failed deliveries if needed
                    if results.queue_full > 0 && queued_message.retry_count < 3 {
                        // Could implement retry logic here
                        warn!("Message {} had {} queue full errors (retry {})", 
                              queued_message.message.id, results.queue_full, queued_message.retry_count);
                    }
                }
                Err(crossbeam_channel::RecvTimeoutError::Timeout) => {
                    // Normal timeout, continue loop
                    continue;
                }
                Err(crossbeam_channel::RecvTimeoutError::Disconnected) => {
                    debug!("Worker {} channel disconnected", worker_id);
                    break;
                }
            }
        }

        debug!("Worker {} stopped", worker_id);
    }
}

/// Estimate the size of a message for metrics purposes
fn estimate_message_size(message: &BusMessage) -> usize {
    // This is a rough estimate - in production you might use actual serialization
    use std::mem;
    
    let base_size = mem::size_of::<BusMessage>();
    
    // Add estimated payload size based on type
    let payload_size = match &message.payload {
        crate::MessagePayload::RawEvent(_) => 500,  // Typical event size
        crate::MessagePayload::EventBatch(_) => 5000, // Batch of events
        crate::MessagePayload::StorageStatus(_) => 200,
        crate::MessagePayload::AnalysisComplete(_) => 300,
        crate::MessagePayload::StateChange(_) => 150,
        crate::MessagePayload::InterventionRequest(_) => 400,
        crate::MessagePayload::RewardEvent(_) => 200,
        crate::MessagePayload::InterventionResponse(_) => 600,
        crate::MessagePayload::AnimationCommand(_) => 300,
        crate::MessagePayload::HealthCheck(_) => 100,
        crate::MessagePayload::ConfigUpdate(_) => 250,
        crate::MessagePayload::Shutdown(_) => 50,
        crate::MessagePayload::ModuleReady(_) => 50,
        crate::MessagePayload::Error(_) => 400,
    };
    
    base_size + payload_size
}