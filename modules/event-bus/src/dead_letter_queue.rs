//! Dead Letter Queue Implementation
//!
//! Provides dead letter queue functionality for handling failed messages with
//! replay capability, analysis tools, and comprehensive failure tracking.

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use tracing::{debug, warn, error, info};
use uuid::Uuid;

use crate::{BusMessage, ModuleId, MessageId, EventBusError, EventBusResult};

/// Unique identifier for dead letter entries
pub type DeadLetterId = Uuid;

/// Reason why a message ended up in the dead letter queue
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DeadLetterReason {
    /// Maximum retry attempts exceeded
    MaxRetriesExceeded { attempts: u32 },
    
    /// Message delivery timeout
    DeliveryTimeout { timeout: Duration },
    
    /// Subscriber permanently unavailable
    SubscriberUnavailable { subscriber: ModuleId },
    
    /// Message rejected by subscriber
    MessageRejected { reason: String },
    
    /// Serialization/deserialization failure
    SerializationError { error: String },
    
    /// Circuit breaker open
    CircuitBreakerOpen { circuit: String },
    
    /// Queue overflow
    QueueOverflow { queue_size: usize },
    
    /// Permanent system error
    SystemError { error: String },
    
    /// Manual routing to dead letter queue
    ManualRouting { reason: String },
}

/// Dead letter entry containing the failed message and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadLetterEntry {
    /// Unique identifier for this dead letter entry
    pub id: DeadLetterId,
    
    /// The original message that failed
    pub message: BusMessage,
    
    /// Reason for failure
    pub reason: DeadLetterReason,
    
    /// When the message was added to the dead letter queue
    pub timestamp: SystemTime,
    
    /// How many times delivery was attempted
    pub retry_count: u32,
    
    /// Original intended recipient(s)
    pub intended_recipients: Vec<ModuleId>,
    
    /// Error details if available
    pub error_details: Option<String>,
    
    /// Correlation ID for tracking related messages
    pub correlation_id: Option<String>,
    
    /// How many times this message has been replayed
    pub replay_count: u32,
    
    /// Whether this message is marked for replay
    pub marked_for_replay: bool,
    
    /// Tags for categorization and filtering
    pub tags: Vec<String>,
}

/// Configuration for dead letter queue behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadLetterQueueConfig {
    /// Maximum number of entries to keep in memory
    pub max_entries: usize,
    
    /// Maximum age of entries before automatic cleanup
    pub max_age: Duration,
    
    /// Whether to persist entries to disk
    pub enable_persistence: bool,
    
    /// Path for persistence storage
    pub persistence_path: Option<String>,
    
    /// Automatic replay configuration
    pub auto_replay: Option<AutoReplayConfig>,
    
    /// Enable detailed metrics collection
    pub enable_metrics: bool,
    
    /// Batch size for replay operations
    pub replay_batch_size: usize,
}

/// Configuration for automatic replay of dead letter messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoReplayConfig {
    /// Interval between automatic replay attempts
    pub replay_interval: Duration,
    
    /// Maximum number of replay attempts per message
    pub max_replay_attempts: u32,
    
    /// Whether to use exponential backoff for replay timing
    pub use_exponential_backoff: bool,
    
    /// Initial delay for exponential backoff
    pub initial_replay_delay: Duration,
    
    /// Maximum delay for exponential backoff
    pub max_replay_delay: Duration,
    
    /// Conditions that must be met for replay
    pub replay_conditions: Vec<ReplayCondition>,
}

/// Conditions for automatic replay
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReplayCondition {
    /// System health must be above threshold
    SystemHealthy { min_health_score: f64 },
    
    /// Specific module must be available
    ModuleAvailable { module: ModuleId },
    
    /// Circuit breaker must be closed
    CircuitBreakerClosed { circuit: String },
    
    /// Minimum time must have passed since failure
    MinTimePassed { duration: Duration },
    
    /// Custom condition based on reason
    ReasonMatches { reasons: Vec<DeadLetterReason> },
}

impl Default for DeadLetterQueueConfig {
    fn default() -> Self {
        Self {
            max_entries: 10_000,
            max_age: Duration::from_secs(24 * 60 * 60), // 24 hours
            enable_persistence: true,
            persistence_path: Some("dead_letter_queue.json".to_string()),
            auto_replay: Some(AutoReplayConfig {
                replay_interval: Duration::from_secs(300), // 5 minutes
                max_replay_attempts: 5,
                use_exponential_backoff: true,
                initial_replay_delay: Duration::from_secs(60),
                max_replay_delay: Duration::from_secs(3600), // 1 hour
                replay_conditions: vec![
                    ReplayCondition::MinTimePassed { duration: Duration::from_secs(60) },
                ],
            }),
            enable_metrics: true,
            replay_batch_size: 10,
        }
    }
}

/// Statistics about dead letter queue operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadLetterStats {
    pub total_entries: usize,
    pub entries_by_reason: HashMap<String, usize>,
    pub entries_by_module: HashMap<ModuleId, usize>,
    pub total_replayed: u64,
    pub successful_replays: u64,
    pub failed_replays: u64,
    pub oldest_entry_age: Option<Duration>,
    pub newest_entry_age: Option<Duration>,
    pub average_retry_count: f64,
    pub replay_success_rate: f64,
}

/// Result of a replay operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayResult {
    pub entry_id: DeadLetterId,
    pub success: bool,
    pub error: Option<String>,
    pub retry_count: u32,
    pub replay_timestamp: SystemTime,
}

/// Filter for querying dead letter entries
#[derive(Debug, Clone, Default)]
pub struct DeadLetterFilter {
    pub reasons: Option<Vec<DeadLetterReason>>,
    pub modules: Option<Vec<ModuleId>>,
    pub tags: Option<Vec<String>>,
    pub correlation_id: Option<String>,
    pub min_age: Option<Duration>,
    pub max_age: Option<Duration>,
    pub marked_for_replay: Option<bool>,
    pub max_replay_count: Option<u32>,
}

/// Dead letter queue implementation
pub struct DeadLetterQueue {
    config: DeadLetterQueueConfig,
    entries: Arc<parking_lot::RwLock<VecDeque<DeadLetterEntry>>>,
    stats: Arc<parking_lot::RwLock<DeadLetterStats>>,
    entry_index: Arc<parking_lot::RwLock<HashMap<DeadLetterId, usize>>>,
}

impl DeadLetterQueue {
    /// Create a new dead letter queue with the given configuration
    pub fn new(config: DeadLetterQueueConfig) -> Self {
        let stats = DeadLetterStats {
            total_entries: 0,
            entries_by_reason: HashMap::new(),
            entries_by_module: HashMap::new(),
            total_replayed: 0,
            successful_replays: 0,
            failed_replays: 0,
            oldest_entry_age: None,
            newest_entry_age: None,
            average_retry_count: 0.0,
            replay_success_rate: 0.0,
        };

        Self {
            config,
            entries: Arc::new(parking_lot::RwLock::new(VecDeque::new())),
            stats: Arc::new(parking_lot::RwLock::new(stats)),
            entry_index: Arc::new(parking_lot::RwLock::new(HashMap::new())),
        }
    }

    /// Add a message to the dead letter queue
    pub fn add_message(
        &self,
        message: BusMessage,
        reason: DeadLetterReason,
        retry_count: u32,
        intended_recipients: Vec<ModuleId>,
        error_details: Option<String>,
        correlation_id: Option<String>,
    ) -> DeadLetterId {
        let entry = DeadLetterEntry {
            id: Uuid::new_v4(),
            message,
            reason: reason.clone(),
            timestamp: SystemTime::now(),
            retry_count,
            intended_recipients: intended_recipients.clone(),
            error_details,
            correlation_id,
            replay_count: 0,
            marked_for_replay: false,
            tags: vec![],
        };

        let entry_id = entry.id;
        
        debug!("Adding message {} to dead letter queue: {:?}", entry.message.id, reason);

        // Add to entries
        {
            let mut entries = self.entries.write();
            
            // Check if we need to remove old entries
            while entries.len() >= self.config.max_entries {
                if let Some(removed) = entries.pop_front() {
                    self.entry_index.write().remove(&removed.id);
                    warn!("Removed old entry {} due to queue size limit", removed.id);
                }
            }
            
            let index = entries.len();
            entries.push_back(entry);
            self.entry_index.write().insert(entry_id, index);
        }

        // Update statistics
        self.update_stats_on_add(&reason, &intended_recipients, retry_count);

        // Persist if enabled
        if self.config.enable_persistence {
            if let Err(e) = self.persist_to_disk() {
                error!("Failed to persist dead letter queue: {}", e);
            }
        }

        entry_id
    }

    /// Get all entries matching the filter
    pub fn get_entries(&self, filter: &DeadLetterFilter) -> Vec<DeadLetterEntry> {
        let entries = self.entries.read();
        
        entries
            .iter()
            .filter(|entry| self.matches_filter(entry, filter))
            .cloned()
            .collect()
    }

    /// Get a specific entry by ID
    pub fn get_entry(&self, id: DeadLetterId) -> Option<DeadLetterEntry> {
        let entries = self.entries.read();
        let index = self.entry_index.read().get(&id).copied()?;
        entries.get(index).cloned()
    }

    /// Mark entries for replay
    pub fn mark_for_replay(&self, filter: &DeadLetterFilter) -> usize {
        let mut entries = self.entries.write();
        let mut marked_count = 0;

        for entry in entries.iter_mut() {
            if self.matches_filter(entry, filter) && !entry.marked_for_replay {
                entry.marked_for_replay = true;
                marked_count += 1;
                debug!("Marked entry {} for replay", entry.id);
            }
        }

        info!("Marked {} entries for replay", marked_count);
        marked_count
    }

    /// Replay marked messages
    pub async fn replay_marked_messages<F>(&self, replay_fn: F) -> Vec<ReplayResult>
    where
        F: Fn(&BusMessage) -> std::pin::Pin<Box<dyn std::future::Future<Output = EventBusResult<()>> + Send>>,
    {
        let mut results = Vec::new();
        let mut entries_to_process = Vec::new();

        // Collect entries marked for replay
        {
            let entries = self.entries.read();
            entries_to_process = entries
                .iter()
                .filter(|entry| entry.marked_for_replay)
                .take(self.config.replay_batch_size)
                .cloned()
                .collect();
        }

        info!("Replaying {} marked messages", entries_to_process.len());

        for entry in entries_to_process {
            debug!("Replaying message {} (attempt {})", entry.message.id, entry.replay_count + 1);

            let replay_result = match replay_fn(&entry.message).await {
                Ok(_) => {
                    info!("Successfully replayed message {}", entry.message.id);
                    self.update_stats_on_successful_replay();
                    self.remove_entry(entry.id);
                    
                    ReplayResult {
                        entry_id: entry.id,
                        success: true,
                        error: None,
                        retry_count: entry.retry_count,
                        replay_timestamp: SystemTime::now(),
                    }
                }
                Err(error) => {
                    warn!("Failed to replay message {}: {}", entry.message.id, error);
                    self.update_stats_on_failed_replay();
                    self.increment_replay_count(entry.id);
                    
                    ReplayResult {
                        entry_id: entry.id,
                        success: false,
                        error: Some(error.to_string()),
                        retry_count: entry.retry_count,
                        replay_timestamp: SystemTime::now(),
                    }
                }
            };

            results.push(replay_result);
        }

        results
    }

    /// Remove an entry from the dead letter queue
    pub fn remove_entry(&self, id: DeadLetterId) -> bool {
        let mut entries = self.entries.write();
        let mut index_map = self.entry_index.write();

        if let Some(&index) = index_map.get(&id) {
            if index < entries.len() {
                entries.remove(index);
                index_map.remove(&id);
                
                // Update indices for remaining entries
                for (entry_id, entry_index) in index_map.iter_mut() {
                    if *entry_index > index {
                        *entry_index -= 1;
                    }
                }
                
                debug!("Removed entry {} from dead letter queue", id);
                return true;
            }
        }
        
        false
    }

    /// Clear all entries
    pub fn clear(&self) {
        self.entries.write().clear();
        self.entry_index.write().clear();
        info!("Cleared all entries from dead letter queue");
    }

    /// Clean up old entries based on age
    pub fn cleanup_old_entries(&self) -> usize {
        let cutoff_time = SystemTime::now() - self.config.max_age;
        let mut removed_count = 0;

        let mut entries = self.entries.write();
        let mut index_map = self.entry_index.write();

        entries.retain(|entry| {
            if entry.timestamp < cutoff_time {
                index_map.remove(&entry.id);
                removed_count += 1;
                false
            } else {
                true
            }
        });

        // Rebuild index map after removal
        index_map.clear();
        for (index, entry) in entries.iter().enumerate() {
            index_map.insert(entry.id, index);
        }

        if removed_count > 0 {
            info!("Cleaned up {} old entries from dead letter queue", removed_count);
        }

        removed_count
    }

    /// Get current statistics
    pub fn stats(&self) -> DeadLetterStats {
        self.update_stats();
        self.stats.read().clone()
    }

    /// Add tags to an entry
    pub fn add_tags(&self, id: DeadLetterId, tags: Vec<String>) -> bool {
        let mut entries = self.entries.write();
        let index = match self.entry_index.read().get(&id).copied() {
            Some(idx) => idx,
            None => return false,
        };

        if let Some(entry) = entries.get_mut(index) {
            for tag in tags {
                if !entry.tags.contains(&tag) {
                    entry.tags.push(tag);
                }
            }
            true
        } else {
            false
        }
    }

    /// Check if an entry matches the given filter
    fn matches_filter(&self, entry: &DeadLetterEntry, filter: &DeadLetterFilter) -> bool {
        // Check reason filter
        if let Some(ref reasons) = filter.reasons {
            if !reasons.contains(&entry.reason) {
                return false;
            }
        }

        // Check module filter
        if let Some(ref modules) = filter.modules {
            if !entry.intended_recipients.iter().any(|m| modules.contains(m)) {
                return false;
            }
        }

        // Check tags filter
        if let Some(ref tags) = filter.tags {
            if !tags.iter().any(|tag| entry.tags.contains(tag)) {
                return false;
            }
        }

        // Check correlation ID filter
        if let Some(ref correlation_id) = filter.correlation_id {
            if entry.correlation_id.as_ref() != Some(correlation_id) {
                return false;
            }
        }

        // Check age filters
        let age = SystemTime::now().duration_since(entry.timestamp).unwrap_or_default();
        
        if let Some(min_age) = filter.min_age {
            if age < min_age {
                return false;
            }
        }

        if let Some(max_age) = filter.max_age {
            if age > max_age {
                return false;
            }
        }

        // Check replay status filter
        if let Some(marked) = filter.marked_for_replay {
            if entry.marked_for_replay != marked {
                return false;
            }
        }

        // Check max replay count filter
        if let Some(max_replay_count) = filter.max_replay_count {
            if entry.replay_count > max_replay_count {
                return false;
            }
        }

        true
    }

    /// Update statistics on adding an entry
    fn update_stats_on_add(&self, reason: &DeadLetterReason, recipients: &[ModuleId], retry_count: u32) {
        let mut stats = self.stats.write();
        stats.total_entries += 1;

        // Update reason statistics
        let reason_key = format!("{:?}", reason);
        *stats.entries_by_reason.entry(reason_key).or_insert(0) += 1;

        // Update module statistics
        for module in recipients {
            *stats.entries_by_module.entry(*module).or_insert(0) += 1;
        }

        // Update average retry count
        let total_retries: u32 = stats.entries_by_reason.values().sum::<usize>() as u32 * retry_count;
        stats.average_retry_count = total_retries as f64 / stats.total_entries as f64;
    }

    /// Update statistics on successful replay
    fn update_stats_on_successful_replay(&self) {
        let mut stats = self.stats.write();
        stats.total_replayed += 1;
        stats.successful_replays += 1;
        stats.replay_success_rate = stats.successful_replays as f64 / stats.total_replayed as f64;
    }

    /// Update statistics on failed replay
    fn update_stats_on_failed_replay(&self) {
        let mut stats = self.stats.write();
        stats.total_replayed += 1;
        stats.failed_replays += 1;
        stats.replay_success_rate = stats.successful_replays as f64 / stats.total_replayed as f64;
    }

    /// Increment replay count for an entry
    fn increment_replay_count(&self, id: DeadLetterId) {
        let mut entries = self.entries.write();
        let index = match self.entry_index.read().get(&id).copied() {
            Some(idx) => idx,
            None => return,
        };

        if let Some(entry) = entries.get_mut(index) {
            entry.replay_count += 1;
            entry.marked_for_replay = false; // Unmark after replay attempt
        }
    }

    /// Update general statistics
    fn update_stats(&self) {
        let entries = self.entries.read();
        let mut stats = self.stats.write();

        stats.total_entries = entries.len();

        if !entries.is_empty() {
            let now = SystemTime::now();
            let ages: Vec<Duration> = entries
                .iter()
                .map(|entry| now.duration_since(entry.timestamp).unwrap_or_default())
                .collect();

            stats.oldest_entry_age = ages.iter().max().copied();
            stats.newest_entry_age = ages.iter().min().copied();
        } else {
            stats.oldest_entry_age = None;
            stats.newest_entry_age = None;
        }
    }

    /// Persist entries to disk
    fn persist_to_disk(&self) -> std::io::Result<()> {
        if let Some(ref path) = self.config.persistence_path {
            let entries = self.entries.read();
            let data = serde_json::to_string_pretty(&*entries)?;
            std::fs::write(path, data)?;
            debug!("Persisted {} entries to disk", entries.len());
        }
        Ok(())
    }

    /// Load entries from disk
    pub fn load_from_disk(&self) -> std::io::Result<usize> {
        if let Some(ref path) = self.config.persistence_path {
            if std::path::Path::new(path).exists() {
                let data = std::fs::read_to_string(path)?;
                let loaded_entries: VecDeque<DeadLetterEntry> = serde_json::from_str(&data)?;
                let count = loaded_entries.len();

                let mut entries = self.entries.write();
                let mut index_map = self.entry_index.write();

                *entries = loaded_entries;
                index_map.clear();

                for (index, entry) in entries.iter().enumerate() {
                    index_map.insert(entry.id, index);
                }

                info!("Loaded {} entries from disk", count);
                return Ok(count);
            }
        }
        Ok(0)
    }
}

/// Create a dead letter queue with default configuration
pub fn create_dead_letter_queue() -> DeadLetterQueue {
    DeadLetterQueue::new(DeadLetterQueueConfig::default())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::{MessagePayload, MessagePriority};

    fn create_test_message() -> BusMessage {
        BusMessage::with_priority(
            ModuleId::DataCapture,
            MessagePayload::ModuleReady(ModuleId::DataCapture),
            MessagePriority::Normal,
        )
    }

    #[test]
    fn test_add_and_get_message() {
        let dlq = create_dead_letter_queue();
        let message = create_test_message();
        let message_id = message.id;

        let entry_id = dlq.add_message(
            message,
            DeadLetterReason::MaxRetriesExceeded { attempts: 3 },
            3,
            vec![ModuleId::Storage],
            Some("Test error".to_string()),
            Some("correlation-123".to_string()),
        );

        let entry = dlq.get_entry(entry_id).unwrap();
        assert_eq!(entry.message.id, message_id);
        assert_eq!(entry.retry_count, 3);
        assert!(matches!(entry.reason, DeadLetterReason::MaxRetriesExceeded { attempts: 3 }));
        assert_eq!(entry.correlation_id, Some("correlation-123".to_string()));
    }

    #[test]
    fn test_filter_entries() {
        let dlq = create_dead_letter_queue();

        // Add test entries
        dlq.add_message(
            create_test_message(),
            DeadLetterReason::MaxRetriesExceeded { attempts: 3 },
            3,
            vec![ModuleId::Storage],
            None,
            Some("corr-1".to_string()),
        );

        dlq.add_message(
            create_test_message(),
            DeadLetterReason::DeliveryTimeout { timeout: Duration::from_secs(5) },
            1,
            vec![ModuleId::AnalysisEngine],
            None,
            Some("corr-2".to_string()),
        );

        // Filter by reason
        let filter = DeadLetterFilter {
            reasons: Some(vec![DeadLetterReason::MaxRetriesExceeded { attempts: 3 }]),
            ..Default::default()
        };
        let entries = dlq.get_entries(&filter);
        assert_eq!(entries.len(), 1);

        // Filter by correlation ID
        let filter = DeadLetterFilter {
            correlation_id: Some("corr-2".to_string()),
            ..Default::default()
        };
        let entries = dlq.get_entries(&filter);
        assert_eq!(entries.len(), 1);
    }

    #[test]
    fn test_mark_for_replay() {
        let dlq = create_dead_letter_queue();

        // Add test entries
        let entry_id = dlq.add_message(
            create_test_message(),
            DeadLetterReason::MaxRetriesExceeded { attempts: 3 },
            3,
            vec![ModuleId::Storage],
            None,
            None,
        );

        // Mark for replay
        let filter = DeadLetterFilter::default();
        let marked_count = dlq.mark_for_replay(&filter);
        assert_eq!(marked_count, 1);

        let entry = dlq.get_entry(entry_id).unwrap();
        assert!(entry.marked_for_replay);
    }

    #[tokio::test]
    async fn test_replay_messages() {
        let dlq = create_dead_letter_queue();

        // Add and mark entry for replay
        let entry_id = dlq.add_message(
            create_test_message(),
            DeadLetterReason::MaxRetriesExceeded { attempts: 3 },
            3,
            vec![ModuleId::Storage],
            None,
            None,
        );

        let filter = DeadLetterFilter::default();
        dlq.mark_for_replay(&filter);

        // Replay with successful function
        let results = dlq.replay_marked_messages(|_message| {
            Box::pin(async { Ok(()) })
        }).await;

        assert_eq!(results.len(), 1);
        assert!(results[0].success);
        assert_eq!(results[0].entry_id, entry_id);

        // Entry should be removed after successful replay
        assert!(dlq.get_entry(entry_id).is_none());
    }

    #[test]
    fn test_cleanup_old_entries() {
        let config = DeadLetterQueueConfig {
            max_age: Duration::from_millis(100),
            ..DeadLetterQueueConfig::default()
        };
        let dlq = DeadLetterQueue::new(config);

        // Add entry
        dlq.add_message(
            create_test_message(),
            DeadLetterReason::MaxRetriesExceeded { attempts: 3 },
            3,
            vec![ModuleId::Storage],
            None,
            None,
        );

        std::thread::sleep(Duration::from_millis(150));

        let removed_count = dlq.cleanup_old_entries();
        assert_eq!(removed_count, 1);

        let stats = dlq.stats();
        assert_eq!(stats.total_entries, 0);
    }

    #[test]
    fn test_add_tags() {
        let dlq = create_dead_letter_queue();

        let entry_id = dlq.add_message(
            create_test_message(),
            DeadLetterReason::MaxRetriesExceeded { attempts: 3 },
            3,
            vec![ModuleId::Storage],
            None,
            None,
        );

        let success = dlq.add_tags(entry_id, vec!["critical".to_string(), "retry".to_string()]);
        assert!(success);

        let entry = dlq.get_entry(entry_id).unwrap();
        assert!(entry.tags.contains(&"critical".to_string()));
        assert!(entry.tags.contains(&"retry".to_string()));
    }

    #[test]
    fn test_stats() {
        let dlq = create_dead_letter_queue();

        dlq.add_message(
            create_test_message(),
            DeadLetterReason::MaxRetriesExceeded { attempts: 3 },
            3,
            vec![ModuleId::Storage],
            None,
            None,
        );

        dlq.add_message(
            create_test_message(),
            DeadLetterReason::DeliveryTimeout { timeout: Duration::from_secs(5) },
            1,
            vec![ModuleId::AnalysisEngine],
            None,
            None,
        );

        let stats = dlq.stats();
        assert_eq!(stats.total_entries, 2);
        assert!(stats.entries_by_reason.len() > 0);
        assert!(stats.entries_by_module.len() > 0);
    }
}