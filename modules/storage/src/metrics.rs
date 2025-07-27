//! Performance metrics for the Storage module

use parking_lot::RwLock;
use prometheus::{
    register_counter_vec, register_gauge_vec, register_histogram_vec, CounterVec, GaugeVec,
    HistogramVec,
};
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use sysinfo::{CpuExt, System, SystemExt};

/// Performance metrics for monitoring storage operations
#[derive(Clone)]
pub struct PerformanceMetrics {
    // Event metrics
    pub events_received: Arc<AtomicU64>,
    pub events_per_second: Arc<RwLock<ExponentialMovingAverage>>,
    event_processing_latency: HistogramVec,

    // Batch metrics
    pub batches_created: Arc<AtomicU64>,
    pub events_per_batch: Arc<RwLock<RollingAverage>>,
    batch_creation_time: HistogramVec,

    // Screenshot metrics
    pub screenshots_received: Arc<AtomicU64>,
    pub screenshots_in_memory: Arc<AtomicU32>,
    pub screenshots_on_disk: Arc<AtomicU32>,
    pub screenshots_deleted: Arc<AtomicU64>,
    screenshot_processing_time: HistogramVec,

    // Database metrics
    db_write_latency: HistogramVec,
    pub db_write_batch_size: Arc<RwLock<RollingAverage>>,
    pub db_size_bytes: Arc<AtomicU64>,

    // Resource usage
    pub memory_usage_bytes: Arc<AtomicU64>,
    pub cpu_usage_percent: Arc<RwLock<ExponentialMovingAverage>>,

    // Prometheus metrics (optional)
    #[cfg(feature = "metrics")]
    prom_events_total: CounterVec,
    #[cfg(feature = "metrics")]
    prom_screenshots_gauge: GaugeVec,
    #[cfg(feature = "metrics")]
    prom_resource_gauge: GaugeVec,

    // System info for CPU monitoring
    system: Arc<RwLock<System>>,
    last_cpu_update: Arc<RwLock<Instant>>,
}

impl PerformanceMetrics {
    /// Create new metrics instance
    pub fn new() -> Self {
        // Initialize Prometheus metrics if enabled
        #[cfg(feature = "metrics")]
        let prom_events_total = register_counter_vec!(
            "storage_events_total",
            "Total number of events processed",
            &["event_type"]
        )
        .unwrap();

        #[cfg(feature = "metrics")]
        let prom_screenshots_gauge = register_gauge_vec!(
            "storage_screenshots",
            "Current number of screenshots",
            &["location"]
        )
        .unwrap();

        #[cfg(feature = "metrics")]
        let prom_resource_gauge = register_gauge_vec!(
            "storage_resources",
            "Resource usage metrics",
            &["resource"]
        )
        .unwrap();

        Self {
            // Event metrics
            events_received: Arc::new(AtomicU64::new(0)),
            events_per_second: Arc::new(RwLock::new(ExponentialMovingAverage::new(0.1))),
            event_processing_latency: HistogramVec::new(
                prometheus::HistogramOpts::new(
                    "storage_event_processing_latency",
                    "Event processing latency in seconds",
                ),
                &["event_type"],
            )
            .unwrap(),

            // Batch metrics
            batches_created: Arc::new(AtomicU64::new(0)),
            events_per_batch: Arc::new(RwLock::new(RollingAverage::new(100))),
            batch_creation_time: HistogramVec::new(
                prometheus::HistogramOpts::new(
                    "storage_batch_creation_time",
                    "Batch creation time in seconds",
                ),
                &[],
            )
            .unwrap(),

            // Screenshot metrics
            screenshots_received: Arc::new(AtomicU64::new(0)),
            screenshots_in_memory: Arc::new(AtomicU32::new(0)),
            screenshots_on_disk: Arc::new(AtomicU32::new(0)),
            screenshots_deleted: Arc::new(AtomicU64::new(0)),
            screenshot_processing_time: HistogramVec::new(
                prometheus::HistogramOpts::new(
                    "storage_screenshot_processing_time",
                    "Screenshot processing time in seconds",
                ),
                &[],
            )
            .unwrap(),

            // Database metrics
            db_write_latency: HistogramVec::new(
                prometheus::HistogramOpts::new(
                    "storage_db_write_latency",
                    "Database write latency in seconds",
                ),
                &["operation"],
            )
            .unwrap(),
            db_write_batch_size: Arc::new(RwLock::new(RollingAverage::new(100))),
            db_size_bytes: Arc::new(AtomicU64::new(0)),

            // Resource usage
            memory_usage_bytes: Arc::new(AtomicU64::new(0)),
            cpu_usage_percent: Arc::new(RwLock::new(ExponentialMovingAverage::new(0.1))),

            #[cfg(feature = "metrics")]
            prom_events_total,
            #[cfg(feature = "metrics")]
            prom_screenshots_gauge,
            #[cfg(feature = "metrics")]
            prom_resource_gauge,

            system: Arc::new(RwLock::new(System::new_all())),
            last_cpu_update: Arc::new(RwLock::new(Instant::now())),
        }
    }

    /// Record an event received
    pub fn record_event_received(&self, event_type: &str) {
        self.events_received.fetch_add(1, Ordering::Relaxed);
        
        #[cfg(feature = "metrics")]
        self.prom_events_total.with_label_values(&[event_type]).inc();
        
        // Update events per second
        let mut eps = self.events_per_second.write();
        eps.update(1.0);
    }

    /// Record event processing latency
    pub fn record_event_latency(&self, event_type: &str, duration: Duration) {
        self.event_processing_latency
            .with_label_values(&[event_type])
            .observe(duration.as_secs_f64());
    }

    /// Record batch creation
    pub fn record_batch_created(&self, event_count: usize, duration: Duration) {
        self.batches_created.fetch_add(1, Ordering::Relaxed);
        
        let mut avg = self.events_per_batch.write();
        avg.update(event_count as f64);
        
        self.batch_creation_time
            .with_label_values(&[])
            .observe(duration.as_secs_f64());
    }

    /// Record screenshot received
    pub fn record_screenshot_received(&self) {
        self.screenshots_received.fetch_add(1, Ordering::Relaxed);
    }

    /// Update screenshot counts
    pub fn update_screenshot_counts(&self, in_memory: u32, on_disk: u32) {
        self.screenshots_in_memory.store(in_memory, Ordering::Relaxed);
        self.screenshots_on_disk.store(on_disk, Ordering::Relaxed);
        
        #[cfg(feature = "metrics")]
        {
            self.prom_screenshots_gauge
                .with_label_values(&["memory"])
                .set(in_memory as f64);
            self.prom_screenshots_gauge
                .with_label_values(&["disk"])
                .set(on_disk as f64);
        }
    }

    /// Record screenshot deletion
    pub fn record_screenshot_deleted(&self) {
        self.screenshots_deleted.fetch_add(1, Ordering::Relaxed);
    }

    /// Record screenshot processing time
    pub fn record_screenshot_processing(&self, duration: Duration) {
        self.screenshot_processing_time
            .with_label_values(&[])
            .observe(duration.as_secs_f64());
    }

    /// Record database write latency
    pub fn record_db_write(&self, operation: &str, duration: Duration) {
        self.db_write_latency
            .with_label_values(&[operation])
            .observe(duration.as_secs_f64());
    }

    /// Record database write batch size
    pub fn record_db_batch_size(&self, size: usize) {
        let mut avg = self.db_write_batch_size.write();
        avg.update(size as f64);
    }

    /// Update database size
    pub fn update_db_size(&self, bytes: u64) {
        self.db_size_bytes.store(bytes, Ordering::Relaxed);
        
        #[cfg(feature = "metrics")]
        self.prom_resource_gauge
            .with_label_values(&["db_size_bytes"])
            .set(bytes as f64);
    }

    /// Update memory usage
    pub fn update_memory_usage(&self, bytes: u64) {
        self.memory_usage_bytes.store(bytes, Ordering::Relaxed);
        
        #[cfg(feature = "metrics")]
        self.prom_resource_gauge
            .with_label_values(&["memory_bytes"])
            .set(bytes as f64);
    }

    /// Update CPU usage
    pub fn update_cpu_usage(&self) {
        let mut last_update = self.last_cpu_update.write();
        let now = Instant::now();
        
        // Only update every second
        if now.duration_since(*last_update) >= Duration::from_secs(1) {
            let mut system = self.system.write();
            system.refresh_cpu();
            
            let cpu_usage = system.global_cpu_info().cpu_usage();
            let mut avg = self.cpu_usage_percent.write();
            avg.update(cpu_usage as f64);
            
            #[cfg(feature = "metrics")]
            self.prom_resource_gauge
                .with_label_values(&["cpu_percent"])
                .set(cpu_usage as f64);
            
            *last_update = now;
        }
    }

    /// Get current events per second
    pub fn events_per_second(&self) -> f64 {
        self.events_per_second.read().value()
    }

    /// Get average events per batch
    pub fn avg_events_per_batch(&self) -> f64 {
        self.events_per_batch.read().average()
    }

    /// Get average CPU usage
    pub fn avg_cpu_usage(&self) -> f64 {
        self.cpu_usage_percent.read().value()
    }

    /// Get current memory usage in MB
    pub fn memory_usage_mb(&self) -> f64 {
        self.memory_usage_bytes.load(Ordering::Relaxed) as f64 / (1024.0 * 1024.0)
    }
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Exponential moving average calculator
pub struct ExponentialMovingAverage {
    alpha: f64,
    value: f64,
    initialized: bool,
}

impl ExponentialMovingAverage {
    pub fn new(alpha: f64) -> Self {
        Self {
            alpha,
            value: 0.0,
            initialized: false,
        }
    }

    pub fn update(&mut self, new_value: f64) {
        if !self.initialized {
            self.value = new_value;
            self.initialized = true;
        } else {
            self.value = self.alpha * new_value + (1.0 - self.alpha) * self.value;
        }
    }

    pub fn value(&self) -> f64 {
        self.value
    }
}

/// Rolling average calculator
pub struct RollingAverage {
    values: Vec<f64>,
    capacity: usize,
    index: usize,
    count: usize,
}

impl RollingAverage {
    pub fn new(capacity: usize) -> Self {
        Self {
            values: vec![0.0; capacity],
            capacity,
            index: 0,
            count: 0,
        }
    }

    pub fn update(&mut self, value: f64) {
        self.values[self.index] = value;
        self.index = (self.index + 1) % self.capacity;
        if self.count < self.capacity {
            self.count += 1;
        }
    }

    pub fn average(&self) -> f64 {
        if self.count == 0 {
            0.0
        } else {
            self.values.iter().take(self.count).sum::<f64>() / self.count as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exponential_moving_average() {
        let mut ema = ExponentialMovingAverage::new(0.5);
        ema.update(10.0);
        assert_eq!(ema.value(), 10.0);
        ema.update(20.0);
        assert_eq!(ema.value(), 15.0);
    }

    #[test]
    fn test_rolling_average() {
        let mut avg = RollingAverage::new(3);
        avg.update(10.0);
        assert_eq!(avg.average(), 10.0);
        avg.update(20.0);
        assert_eq!(avg.average(), 15.0);
        avg.update(30.0);
        assert_eq!(avg.average(), 20.0);
        avg.update(40.0); // Overwrites first value
        assert_eq!(avg.average(), 30.0);
    }

    #[test]
    fn test_metrics_recording() {
        let metrics = PerformanceMetrics::new();
        
        metrics.record_event_received("keystroke");
        assert_eq!(metrics.events_received.load(Ordering::Relaxed), 1);
        
        metrics.record_batch_created(100, Duration::from_millis(10));
        assert_eq!(metrics.batches_created.load(Ordering::Relaxed), 1);
        assert_eq!(metrics.avg_events_per_batch(), 100.0);
    }
}