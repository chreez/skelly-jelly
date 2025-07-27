//! Metrics collection and monitoring for the event bus

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::{ModuleId, MessageType};

/// Comprehensive metrics for the event bus
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusMetrics {
    /// Total messages published since startup
    pub messages_published: u64,
    
    /// Total messages delivered since startup
    pub messages_delivered: u64,
    
    /// Total failed deliveries since startup
    pub messages_failed: u64,
    
    /// Current queue depth
    pub current_queue_depth: u64,
    
    /// Delivery latency statistics
    pub delivery_latency: LatencyStats,
    
    /// Per-module statistics
    pub module_stats: HashMap<ModuleId, ModuleMetrics>,
    
    /// Per-message-type statistics
    pub message_type_stats: HashMap<MessageType, MessageTypeMetrics>,
    
    /// Memory usage information
    pub memory_usage: MemoryMetrics,
    
    /// When these metrics were collected
    pub collected_at: DateTime<Utc>,
    
    /// System uptime since bus started
    pub uptime: Duration,
}

/// Latency statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyStats {
    pub min_ms: f64,
    pub max_ms: f64,
    pub mean_ms: f64,
    pub p50_ms: f64,
    pub p95_ms: f64,
    pub p99_ms: f64,
}

/// Per-module metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleMetrics {
    pub messages_published: u64,
    pub messages_received: u64,
    pub subscriptions_active: u32,
    pub last_activity: Option<DateTime<Utc>>,
}

/// Per-message-type metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageTypeMetrics {
    pub count: u64,
    pub avg_size_bytes: u64,
    pub avg_latency_ms: f64,
}

/// Memory usage metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryMetrics {
    pub total_allocated_bytes: u64,
    pub queue_memory_bytes: u64,
    pub subscription_memory_bytes: u64,
}

/// Internal metrics collector with atomic counters for performance
pub struct MetricsCollector {
    // Atomic counters for high-frequency operations
    messages_published: AtomicU64,
    messages_delivered: AtomicU64,
    messages_failed: AtomicU64,
    current_queue_depth: AtomicU64,
    
    // Latency tracking
    latency_samples: parking_lot::Mutex<Vec<Duration>>,
    max_latency_samples: usize,
    
    // Per-module counters
    module_published: dashmap::DashMap<ModuleId, AtomicU64>,
    module_received: dashmap::DashMap<ModuleId, AtomicU64>,
    module_last_activity: dashmap::DashMap<ModuleId, SystemTime>,
    
    // Per-message-type counters
    message_type_counts: dashmap::DashMap<MessageType, AtomicU64>,
    message_type_sizes: dashmap::DashMap<MessageType, AtomicU64>,
    message_type_latencies: dashmap::DashMap<MessageType, parking_lot::Mutex<Vec<Duration>>>,
    
    // System information
    start_time: SystemTime,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new() -> Self {
        Self {
            messages_published: AtomicU64::new(0),
            messages_delivered: AtomicU64::new(0),
            messages_failed: AtomicU64::new(0),
            current_queue_depth: AtomicU64::new(0),
            latency_samples: parking_lot::Mutex::new(Vec::new()),
            max_latency_samples: 10_000, // Keep last 10k samples
            module_published: dashmap::DashMap::new(),
            module_received: dashmap::DashMap::new(),
            module_last_activity: dashmap::DashMap::new(),
            message_type_counts: dashmap::DashMap::new(),
            message_type_sizes: dashmap::DashMap::new(),
            message_type_latencies: dashmap::DashMap::new(),
            start_time: SystemTime::now(),
        }
    }

    /// Record a message being published
    pub fn record_publish(&self, module: ModuleId, message_type: MessageType, size_bytes: usize) {
        self.messages_published.fetch_add(1, Ordering::Relaxed);
        
        // Update per-module stats
        self.module_published
            .entry(module)
            .or_insert_with(|| AtomicU64::new(0))
            .fetch_add(1, Ordering::Relaxed);
        
        self.module_last_activity.insert(module, SystemTime::now());
        
        // Update per-message-type stats
        self.message_type_counts
            .entry(message_type)
            .or_insert_with(|| AtomicU64::new(0))
            .fetch_add(1, Ordering::Relaxed);
        
        self.message_type_sizes
            .entry(message_type)
            .or_insert_with(|| AtomicU64::new(0))
            .fetch_add(size_bytes as u64, Ordering::Relaxed);
    }

    /// Record a message being delivered
    pub fn record_delivery(&self, module: ModuleId, message_type: MessageType, latency: Duration) {
        self.messages_delivered.fetch_add(1, Ordering::Relaxed);
        
        // Update per-module stats
        self.module_received
            .entry(module)
            .or_insert_with(|| AtomicU64::new(0))
            .fetch_add(1, Ordering::Relaxed);
        
        self.module_last_activity.insert(module, SystemTime::now());
        
        // Record latency
        {
            let mut samples = self.latency_samples.lock();
            samples.push(latency);
            
            // Keep only recent samples to prevent unbounded growth
            if samples.len() > self.max_latency_samples {
                samples.remove(0);
            }
        }
        
        // Record per-message-type latency
        self.message_type_latencies
            .entry(message_type)
            .or_insert_with(|| parking_lot::Mutex::new(Vec::new()))
            .lock()
            .push(latency);
    }

    /// Record a delivery failure
    pub fn record_failure(&self, _module: ModuleId, _message_type: MessageType) {
        self.messages_failed.fetch_add(1, Ordering::Relaxed);
    }

    /// Update current queue depth
    pub fn update_queue_depth(&self, depth: usize) {
        self.current_queue_depth.store(depth as u64, Ordering::Relaxed);
    }

    /// Record that a subscription was created
    pub fn record_subscription_created(&self, _module: ModuleId) {
        // This could be extended to track subscription-specific metrics
    }

    /// Record that a subscription was removed
    pub fn record_subscription_removed(&self, _module: ModuleId) {
        // This could be extended to track subscription-specific metrics
    }

    /// Generate a snapshot of current metrics
    pub fn snapshot(&self, subscription_counts: HashMap<ModuleId, u32>) -> BusMetrics {
        let now = SystemTime::now();
        let uptime = now.duration_since(self.start_time)
            .unwrap_or_default();

        // Calculate latency statistics
        let delivery_latency = {
            let samples = self.latency_samples.lock();
            calculate_latency_stats(&samples)
        };

        // Collect per-module statistics
        let mut module_stats = HashMap::new();
        for module in [
            ModuleId::DataCapture,
            ModuleId::Storage,
            ModuleId::AnalysisEngine,
            ModuleId::Gamification,
            ModuleId::AiIntegration,
            ModuleId::CuteFigurine,
            ModuleId::Orchestrator,
            ModuleId::EventBus,
        ] {
            let published = self.module_published
                .get(&module)
                .map(|v| v.load(Ordering::Relaxed))
                .unwrap_or(0);
            
            let received = self.module_received
                .get(&module)
                .map(|v| v.load(Ordering::Relaxed))
                .unwrap_or(0);
            
            let last_activity = self.module_last_activity
                .get(&module)
                .map(|entry| {
                    DateTime::<Utc>::from(*entry.value())
                });
            
            let subscriptions_active = subscription_counts.get(&module).copied().unwrap_or(0);

            module_stats.insert(module, ModuleMetrics {
                messages_published: published,
                messages_received: received,
                subscriptions_active,
                last_activity,
            });
        }

        // Collect per-message-type statistics
        let mut message_type_stats = HashMap::new();
        for entry in self.message_type_counts.iter() {
            let message_type = *entry.key();
            let count = entry.value().load(Ordering::Relaxed);
            let total_size = self.message_type_sizes
                .get(&message_type)
                .map(|v| v.load(Ordering::Relaxed))
                .unwrap_or(0);
            
            let avg_size_bytes = if count > 0 { total_size / count } else { 0 };
            
            let avg_latency_ms = self.message_type_latencies
                .get(&message_type)
                .map(|latencies| {
                    let latencies = latencies.lock();
                    if latencies.is_empty() {
                        0.0
                    } else {
                        latencies.iter()
                            .map(|d| d.as_millis() as f64)
                            .sum::<f64>() / latencies.len() as f64
                    }
                })
                .unwrap_or(0.0);
            
            message_type_stats.insert(message_type, MessageTypeMetrics {
                count,
                avg_size_bytes,
                avg_latency_ms,
            });
        }

        BusMetrics {
            messages_published: self.messages_published.load(Ordering::Relaxed),
            messages_delivered: self.messages_delivered.load(Ordering::Relaxed),
            messages_failed: self.messages_failed.load(Ordering::Relaxed),
            current_queue_depth: self.current_queue_depth.load(Ordering::Relaxed),
            delivery_latency,
            module_stats,
            message_type_stats,
            memory_usage: estimate_memory_usage(),
            collected_at: Utc::now(),
            uptime,
        }
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Calculate latency statistics from a collection of samples
fn calculate_latency_stats(samples: &[Duration]) -> LatencyStats {
    if samples.is_empty() {
        return LatencyStats {
            min_ms: 0.0,
            max_ms: 0.0,
            mean_ms: 0.0,
            p50_ms: 0.0,
            p95_ms: 0.0,
            p99_ms: 0.0,
        };
    }

    let mut sorted_ms: Vec<f64> = samples
        .iter()
        .map(|d| d.as_millis() as f64)
        .collect();
    sorted_ms.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let min_ms = sorted_ms[0];
    let max_ms = sorted_ms[sorted_ms.len() - 1];
    let mean_ms = sorted_ms.iter().sum::<f64>() / sorted_ms.len() as f64;

    let p50_ms = percentile(&sorted_ms, 0.5);
    let p95_ms = percentile(&sorted_ms, 0.95);
    let p99_ms = percentile(&sorted_ms, 0.99);

    LatencyStats {
        min_ms,
        max_ms,
        mean_ms,
        p50_ms,
        p95_ms,
        p99_ms,
    }
}

/// Calculate a percentile from sorted data
fn percentile(sorted_data: &[f64], p: f64) -> f64 {
    if sorted_data.is_empty() {
        return 0.0;
    }

    let index = (p * (sorted_data.len() - 1) as f64).round() as usize;
    sorted_data[index.min(sorted_data.len() - 1)]
}

/// Estimate current memory usage (simplified implementation)
fn estimate_memory_usage() -> MemoryMetrics {
    // This is a simplified implementation
    // In a real system, you might use platform-specific APIs to get actual memory usage
    MemoryMetrics {
        total_allocated_bytes: 0, // Would need platform-specific implementation
        queue_memory_bytes: 0,    // Estimated based on queue sizes
        subscription_memory_bytes: 0, // Estimated based on subscription count
    }
}