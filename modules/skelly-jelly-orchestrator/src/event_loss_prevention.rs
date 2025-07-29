//! Event loss prevention system for maintaining <0.1% event loss rate

use crate::error::{OrchestratorError, OrchestratorResult};
use dashmap::DashMap;
use skelly_jelly_event_bus::{ModuleId, MessageId, BusMessage};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, atomic::{AtomicU64, AtomicBool, Ordering}},
    time::{Duration, Instant},
};
use tokio::{
    sync::{RwLock, mpsc, watch, Semaphore},
    task::JoinHandle,
    time::{interval, timeout},
};
use tracing::{debug, warn, error, info};

/// Target event loss rate (<0.1%)
const TARGET_EVENT_LOSS_RATE: f32 = 0.001;
const QUEUE_HIGH_WATER_MARK: f32 = 0.8;
const QUEUE_CRITICAL_MARK: f32 = 0.95;
const BACKPRESSURE_RELIEF_TIME: Duration = Duration::from_millis(100);

/// Event loss prevention system
pub struct EventLossPreventionSystem {
    /// Queue monitors per module
    queue_monitors: DashMap<ModuleId, QueueMonitor>,
    
    /// Backpressure controllers
    backpressure_controllers: DashMap<ModuleId, BackpressureController>,
    
    /// Event loss tracking
    loss_tracker: Arc<EventLossTracker>,
    
    /// Circuit breaker for emergency stops
    emergency_circuit_breaker: Arc<EmergencyCircuitBreaker>,
    
    /// Graceful degradation manager
    degradation_manager: Arc<GracefulDegradationManager>,
    
    /// Background monitoring task
    monitoring_task: Option<JoinHandle<()>>,
    
    /// Configuration
    config: EventLossPreventionConfig,
}

#[derive(Debug, Clone)]
pub struct EventLossPreventionConfig {
    pub enabled: bool,
    pub monitoring_interval: Duration,
    pub max_queue_size: usize,
    pub high_water_mark: f32,
    pub critical_mark: f32,
    pub target_loss_rate: f32,
    pub emergency_threshold: f32,
}

impl Default for EventLossPreventionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            monitoring_interval: Duration::from_millis(100),
            max_queue_size: 10000,
            high_water_mark: QUEUE_HIGH_WATER_MARK,
            critical_mark: QUEUE_CRITICAL_MARK,
            target_loss_rate: TARGET_EVENT_LOSS_RATE,
            emergency_threshold: 0.01, // 1% loss rate triggers emergency
        }
    }
}

/// Queue monitor for a specific module
#[derive(Debug)]
pub struct QueueMonitor {
    pub module_id: ModuleId,
    pub max_depth: usize,
    pub current_depth: AtomicU64,
    pub high_water_mark: AtomicU64,
    pub total_enqueued: AtomicU64,
    pub total_dequeued: AtomicU64,
    pub drop_count: AtomicU64,
    pub last_check: RwLock<Instant>,
    pub status: RwLock<QueueStatus>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum QueueStatus {
    Healthy,
    Warning,
    Critical,
    Emergency,
}

impl QueueMonitor {
    pub fn new(module_id: ModuleId, max_depth: usize) -> Self {
        Self {
            module_id,
            max_depth,
            current_depth: AtomicU64::new(0),
            high_water_mark: AtomicU64::new(0),
            total_enqueued: AtomicU64::new(0),
            total_dequeued: AtomicU64::new(0),
            drop_count: AtomicU64::new(0),
            last_check: RwLock::new(Instant::now()),
            status: RwLock::new(QueueStatus::Healthy),
        }
    }

    pub async fn record_enqueue(&self) -> bool {
        let current = self.current_depth.fetch_add(1, Ordering::SeqCst) + 1;
        self.total_enqueued.fetch_add(1, Ordering::SeqCst);
        
        // Update high water mark
        self.high_water_mark.fetch_max(current, Ordering::SeqCst);
        
        // Check if we're approaching limits
        let utilization = current as f32 / self.max_depth as f32;
        let new_status = match utilization {
            u if u >= QUEUE_CRITICAL_MARK => QueueStatus::Emergency,
            u if u >= QUEUE_HIGH_WATER_MARK => QueueStatus::Critical,
            u if u >= 0.6 => QueueStatus::Warning,
            _ => QueueStatus::Healthy,
        };

        *self.status.write().await = new_status.clone();
        
        if matches!(new_status, QueueStatus::Emergency) {
            self.drop_count.fetch_add(1, Ordering::SeqCst);
            warn!("Queue for module {:?} is at emergency level, dropping event", self.module_id);
            return false;
        }

        true
    }

    pub fn record_dequeue(&self) {
        self.current_depth.fetch_sub(1, Ordering::SeqCst);
        self.total_dequeued.fetch_add(1, Ordering::SeqCst);
    }

    pub fn get_utilization(&self) -> f32 {
        let current = self.current_depth.load(Ordering::SeqCst);
        current as f32 / self.max_depth as f32
    }

    pub fn get_loss_rate(&self) -> f32 {
        let total_enqueued = self.total_enqueued.load(Ordering::SeqCst);
        let dropped = self.drop_count.load(Ordering::SeqCst);
        
        if total_enqueued > 0 {
            dropped as f32 / total_enqueued as f32
        } else {
            0.0
        }
    }

    pub async fn get_status(&self) -> QueueStatus {
        self.status.read().await.clone()
    }
}

/// Backpressure controller for managing flow control
#[derive(Debug)]
pub struct BackpressureController {
    pub module_id: ModuleId,
    pub enabled: AtomicBool,
    pub current_pressure: RwLock<f32>,
    pub relief_actions: RwLock<Vec<BackpressureAction>>,
    pub throttle_semaphore: Semaphore,
    pub last_relief: RwLock<Instant>,
}

#[derive(Debug, Clone)]
pub enum BackpressureAction {
    ReduceIngestion { factor: f32, duration: Duration },
    PauseNonCritical { duration: Duration },
    ActivateCircuitBreaker { timeout: Duration },
    IncreaseBufferSize { new_size: usize },
    ShedLoad { percentage: f32 },
}

impl BackpressureController {
    pub fn new(module_id: ModuleId, max_concurrent: usize) -> Self {
        Self {
            module_id,
            enabled: AtomicBool::new(true),
            current_pressure: RwLock::new(0.0),
            relief_actions: RwLock::new(Vec::new()),
            throttle_semaphore: Semaphore::new(max_concurrent),
            last_relief: RwLock::new(Instant::now()),
        }
    }

    pub async fn apply_backpressure(&self, pressure: f32) -> Vec<BackpressureAction> {
        if !self.enabled.load(Ordering::SeqCst) {
            return vec![];
        }

        *self.current_pressure.write().await = pressure;

        let actions = self.determine_relief_actions(pressure).await;
        *self.relief_actions.write().await = actions.clone();
        *self.last_relief.write().await = Instant::now();

        info!("Applied backpressure for module {:?} at {:.1}% pressure with {} actions", 
            self.module_id, pressure * 100.0, actions.len());

        actions
    }

    async fn determine_relief_actions(&self, pressure: f32) -> Vec<BackpressureAction> {
        match pressure {
            p if p >= 0.95 => vec![
                BackpressureAction::ShedLoad { percentage: 20.0 },
                BackpressureAction::PauseNonCritical { duration: Duration::from_millis(500) },
                BackpressureAction::ActivateCircuitBreaker { timeout: Duration::from_secs(10) },
            ],
            p if p >= 0.9 => vec![
                BackpressureAction::ReduceIngestion { factor: 0.5, duration: Duration::from_millis(200) },
                BackpressureAction::ShedLoad { percentage: 10.0 },
            ],
            p if p >= 0.8 => vec![
                BackpressureAction::ReduceIngestion { factor: 0.7, duration: Duration::from_millis(100) },
            ],
            p if p >= 0.6 => vec![
                BackpressureAction::ReduceIngestion { factor: 0.9, duration: Duration::from_millis(50) },
            ],
            _ => vec![],
        }
    }

    pub async fn get_pressure(&self) -> f32 {
        *self.current_pressure.read().await
    }

    pub async fn acquire_permit(&self) -> tokio::sync::SemaphorePermit<'_> {
        self.throttle_semaphore.acquire().await.unwrap()
    }
}

/// Event loss tracker for system-wide statistics
#[derive(Debug)]
pub struct EventLossTracker {
    /// Total events processed
    total_events: AtomicU64,
    
    /// Total events dropped
    dropped_events: AtomicU64,
    
    /// Per-module statistics
    module_stats: DashMap<ModuleId, ModuleEventStats>,
    
    /// Recent loss rate samples
    loss_rate_samples: RwLock<VecDeque<f32>>,
    
    /// Last calculation time
    last_calculation: RwLock<Instant>,
}

#[derive(Debug, Default)]
pub struct ModuleEventStats {
    pub total_events: AtomicU64,
    pub dropped_events: AtomicU64,
    pub last_updated: RwLock<Instant>,
}

impl EventLossTracker {
    pub fn new() -> Self {
        Self {
            total_events: AtomicU64::new(0),
            dropped_events: AtomicU64::new(0),
            module_stats: DashMap::new(),
            loss_rate_samples: RwLock::new(VecDeque::new()),
            last_calculation: RwLock::new(Instant::now()),
        }
    }

    pub fn record_event(&self, module_id: ModuleId) {
        self.total_events.fetch_add(1, Ordering::SeqCst);
        
        let stats = self.module_stats.entry(module_id).or_insert_with(ModuleEventStats::default);
        stats.total_events.fetch_add(1, Ordering::SeqCst);
        *stats.last_updated.blocking_write() = Instant::now();
    }

    pub fn record_drop(&self, module_id: ModuleId) {
        self.dropped_events.fetch_add(1, Ordering::SeqCst);
        
        let stats = self.module_stats.entry(module_id).or_insert_with(ModuleEventStats::default);
        stats.dropped_events.fetch_add(1, Ordering::SeqCst);
        *stats.last_updated.blocking_write() = Instant::now();
    }

    pub async fn calculate_loss_rate(&self) -> f32 {
        let total = self.total_events.load(Ordering::SeqCst);
        let dropped = self.dropped_events.load(Ordering::SeqCst);
        
        let current_rate = if total > 0 {
            dropped as f32 / total as f32
        } else {
            0.0
        };

        // Update samples for trend analysis
        {
            let mut samples = self.loss_rate_samples.write().await;
            samples.push_back(current_rate);
            
            // Keep only recent samples
            if samples.len() > 100 {
                samples.pop_front();
            }
        }

        *self.last_calculation.write().await = Instant::now();
        current_rate
    }

    pub async fn get_module_loss_rate(&self, module_id: ModuleId) -> f32 {
        if let Some(stats) = self.module_stats.get(&module_id) {
            let total = stats.total_events.load(Ordering::SeqCst);
            let dropped = stats.dropped_events.load(Ordering::SeqCst);
            
            if total > 0 {
                dropped as f32 / total as f32
            } else {
                0.0
            }
        } else {
            0.0
        }
    }

    pub async fn get_loss_rate_trend(&self) -> Vec<f32> {
        self.loss_rate_samples.read().await.iter().cloned().collect()
    }

    pub fn get_total_stats(&self) -> (u64, u64) {
        (
            self.total_events.load(Ordering::SeqCst),
            self.dropped_events.load(Ordering::SeqCst)
        )
    }
}

/// Emergency circuit breaker for critical situations
#[derive(Debug)]
pub struct EmergencyCircuitBreaker {
    pub state: RwLock<CircuitBreakerState>,
    pub failure_count: AtomicU64,
    pub last_failure: RwLock<Option<Instant>>,
    pub config: CircuitBreakerConfig,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CircuitBreakerState {
    Closed,
    Open,
    HalfOpen,
}

#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    pub failure_threshold: u64,
    pub timeout: Duration,
    pub half_open_max_calls: u64,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            timeout: Duration::from_secs(30),
            half_open_max_calls: 3,
        }
    }
}

impl EmergencyCircuitBreaker {
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            state: RwLock::new(CircuitBreakerState::Closed),
            failure_count: AtomicU64::new(0),
            last_failure: RwLock::new(None),
            config,
        }
    }

    pub async fn call<F, Fut, T>(&self, operation: F) -> Result<T, OrchestratorError>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, OrchestratorError>>,
    {
        let state = self.state.read().await.clone();
        
        match state {
            CircuitBreakerState::Open => {
                // Check if timeout has elapsed
                if let Some(last_failure) = *self.last_failure.read().await {
                    if last_failure.elapsed() >= self.config.timeout {
                        *self.state.write().await = CircuitBreakerState::HalfOpen;
                        self.failure_count.store(0, Ordering::SeqCst);
                    } else {
                        return Err(OrchestratorError::Internal("Circuit breaker is open".to_string()));
                    }
                }
            }
            CircuitBreakerState::HalfOpen => {
                // Limited calls in half-open state
                if self.failure_count.load(Ordering::SeqCst) >= self.config.half_open_max_calls {
                    return Err(OrchestratorError::Internal("Circuit breaker half-open limit exceeded".to_string()));
                }
            }
            CircuitBreakerState::Closed => {
                // Normal operation
            }
        }

        // Execute the operation
        match operation().await {
            Ok(result) => {
                // Success - reset failure count and close circuit if needed
                self.failure_count.store(0, Ordering::SeqCst);
                if matches!(state, CircuitBreakerState::HalfOpen) {
                    *self.state.write().await = CircuitBreakerState::Closed;
                }
                Ok(result)
            }
            Err(error) => {
                // Failure - increment count and potentially open circuit
                let failures = self.failure_count.fetch_add(1, Ordering::SeqCst) + 1;
                *self.last_failure.write().await = Some(Instant::now());
                
                if failures >= self.config.failure_threshold {
                    *self.state.write().await = CircuitBreakerState::Open;
                    error!("Emergency circuit breaker opened after {} failures", failures);
                }
                
                Err(error)
            }
        }
    }

    pub async fn get_state(&self) -> CircuitBreakerState {
        self.state.read().await.clone()
    }
}

/// Graceful degradation manager
#[derive(Debug)]
pub struct GracefulDegradationManager {
    pub degradation_level: RwLock<DegradationLevel>,
    pub active_strategies: RwLock<Vec<DegradationStrategy>>,
    pub last_adjustment: RwLock<Instant>,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum DegradationLevel {
    Normal,
    Light,
    Moderate,
    Heavy,
    Emergency,
}

#[derive(Debug, Clone)]
pub enum DegradationStrategy {
    ReduceSamplingRate { factor: f32 },
    DisableNonCriticalFeatures,
    IncreaseBufferSizes { factor: f32 },
    PrioritizeCriticalModules,
    EnableBatchProcessing { batch_size: usize },
}

impl GracefulDegradationManager {
    pub fn new() -> Self {
        Self {
            degradation_level: RwLock::new(DegradationLevel::Normal),
            active_strategies: RwLock::new(Vec::new()),
            last_adjustment: RwLock::new(Instant::now()),
        }
    }

    pub async fn adjust_degradation(&self, system_pressure: f32, loss_rate: f32) {
        let new_level = self.calculate_degradation_level(system_pressure, loss_rate);
        let current_level = self.degradation_level.read().await.clone();
        
        if new_level != current_level {
            *self.degradation_level.write().await = new_level.clone();
            
            let strategies = self.get_strategies_for_level(&new_level);
            *self.active_strategies.write().await = strategies.clone();
            *self.last_adjustment.write().await = Instant::now();
            
            info!("Degradation level adjusted to {:?} with {} strategies", new_level, strategies.len());
        }
    }

    fn calculate_degradation_level(&self, system_pressure: f32, loss_rate: f32) -> DegradationLevel {
        match (system_pressure, loss_rate) {
            (p, l) if l > 0.05 || p > 0.95 => DegradationLevel::Emergency,
            (p, l) if l > 0.01 || p > 0.9 => DegradationLevel::Heavy,
            (p, l) if l > 0.005 || p > 0.8 => DegradationLevel::Moderate,
            (p, l) if l > 0.001 || p > 0.7 => DegradationLevel::Light,
            _ => DegradationLevel::Normal,
        }
    }

    fn get_strategies_for_level(&self, level: &DegradationLevel) -> Vec<DegradationStrategy> {
        match level {
            DegradationLevel::Normal => vec![],
            DegradationLevel::Light => vec![
                DegradationStrategy::ReduceSamplingRate { factor: 0.9 },
            ],
            DegradationLevel::Moderate => vec![
                DegradationStrategy::ReduceSamplingRate { factor: 0.7 },
                DegradationStrategy::IncreaseBufferSizes { factor: 1.5 },
            ],
            DegradationLevel::Heavy => vec![
                DegradationStrategy::ReduceSamplingRate { factor: 0.5 },
                DegradationStrategy::DisableNonCriticalFeatures,
                DegradationStrategy::PrioritizeCriticalModules,
                DegradationStrategy::EnableBatchProcessing { batch_size: 100 },
            ],
            DegradationLevel::Emergency => vec![
                DegradationStrategy::ReduceSamplingRate { factor: 0.3 },
                DegradationStrategy::DisableNonCriticalFeatures,
                DegradationStrategy::PrioritizeCriticalModules,
                DegradationStrategy::EnableBatchProcessing { batch_size: 200 },
                DegradationStrategy::IncreaseBufferSizes { factor: 2.0 },
            ],
        }
    }

    pub async fn get_current_level(&self) -> DegradationLevel {
        self.degradation_level.read().await.clone()
    }

    pub async fn get_active_strategies(&self) -> Vec<DegradationStrategy> {
        self.active_strategies.read().await.clone()
    }
}

impl EventLossPreventionSystem {
    /// Create a new event loss prevention system
    pub fn new(config: EventLossPreventionConfig) -> Self {
        Self {
            queue_monitors: DashMap::new(),
            backpressure_controllers: DashMap::new(),
            loss_tracker: Arc::new(EventLossTracker::new()),
            emergency_circuit_breaker: Arc::new(EmergencyCircuitBreaker::new(CircuitBreakerConfig::default())),
            degradation_manager: Arc::new(GracefulDegradationManager::new()),
            monitoring_task: None,
            config,
        }
    }

    /// Start the event loss prevention system
    pub async fn start(&mut self) -> OrchestratorResult<()> {
        if !self.config.enabled {
            info!("Event loss prevention system disabled");
            return Ok(());
        }

        info!("Starting event loss prevention system");

        // Start monitoring task
        let queue_monitors = self.queue_monitors.clone();
        let loss_tracker = Arc::clone(&self.loss_tracker);
        let degradation_manager = Arc::clone(&self.degradation_manager);
        let monitoring_interval = self.config.monitoring_interval;
        let target_loss_rate = self.config.target_loss_rate;

        let monitoring_task = tokio::spawn(async move {
            let mut interval = interval(monitoring_interval);
            
            loop {
                interval.tick().await;
                
                // Calculate current loss rate
                let loss_rate = loss_tracker.calculate_loss_rate().await;
                
                // Calculate system pressure
                let mut total_pressure = 0.0f32;
                let mut monitor_count = 0;
                
                for monitor in queue_monitors.iter() {
                    total_pressure += monitor.get_utilization();
                    monitor_count += 1;
                }
                
                let avg_pressure = if monitor_count > 0 {
                    total_pressure / monitor_count as f32
                } else {
                    0.0
                };
                
                // Adjust degradation based on conditions
                degradation_manager.adjust_degradation(avg_pressure, loss_rate).await;
                
                // Log status if above targets
                if loss_rate > target_loss_rate {
                    warn!("Event loss rate ({:.3}%) exceeds target ({:.3}%)", 
                        loss_rate * 100.0, target_loss_rate * 100.0);
                }
                
                if avg_pressure > 0.7 {
                    warn!("Average queue pressure ({:.1}%) is high", avg_pressure * 100.0);
                }
            }
        });

        self.monitoring_task = Some(monitoring_task);
        info!("Event loss prevention system started");
        Ok(())
    }

    /// Stop the system
    pub async fn stop(&mut self) {
        info!("Stopping event loss prevention system");

        if let Some(task) = self.monitoring_task.take() {
            task.abort();
        }

        info!("Event loss prevention system stopped");
    }

    /// Register a queue monitor for a module
    pub fn register_queue_monitor(&self, module_id: ModuleId, max_queue_size: usize) {
        let monitor = QueueMonitor::new(module_id, max_queue_size);
        self.queue_monitors.insert(module_id, monitor);
        
        let controller = BackpressureController::new(module_id, 10); // Max 10 concurrent operations
        self.backpressure_controllers.insert(module_id, controller);
        
        info!("Registered queue monitor for module {:?} with max size {}", module_id, max_queue_size);
    }

    /// Check if an event can be enqueued
    pub async fn can_enqueue(&self, module_id: ModuleId) -> bool {
        self.loss_tracker.record_event(module_id);
        
        if let Some(monitor) = self.queue_monitors.get(&module_id) {
            if !monitor.record_enqueue().await {
                self.loss_tracker.record_drop(module_id);
                
                // Apply backpressure
                if let Some(controller) = self.backpressure_controllers.get(&module_id) {
                    let pressure = monitor.get_utilization();
                    controller.apply_backpressure(pressure).await;
                }
                
                return false;
            }
        }
        
        true
    }

    /// Record successful dequeue
    pub fn record_dequeue(&self, module_id: ModuleId) {
        if let Some(monitor) = self.queue_monitors.get(&module_id) {
            monitor.record_dequeue();
        }
    }

    /// Get current event loss statistics
    pub async fn get_loss_statistics(&self) -> EventLossStatistics {
        let overall_loss_rate = self.loss_tracker.calculate_loss_rate().await;
        let (total_events, dropped_events) = self.loss_tracker.get_total_stats();
        
        let mut module_stats = HashMap::new();
        for module_entry in self.queue_monitors.iter() {
            let module_id = *module_entry.key();
            let loss_rate = self.loss_tracker.get_module_loss_rate(module_id).await;
            let utilization = module_entry.get_utilization();
            let status = module_entry.get_status().await;
            
            module_stats.insert(module_id, ModuleStatistics {
                loss_rate,
                queue_utilization: utilization,
                queue_status: status,
                total_events: module_entry.total_enqueued.load(Ordering::SeqCst),
                dropped_events: module_entry.drop_count.load(Ordering::SeqCst),
            });
        }
        
        let degradation_level = self.degradation_manager.get_current_level().await;
        let circuit_breaker_state = self.emergency_circuit_breaker.get_state().await;
        
        EventLossStatistics {
            overall_loss_rate,
            total_events,
            dropped_events,
            meets_target: overall_loss_rate < self.config.target_loss_rate,
            module_statistics: module_stats,
            degradation_level,
            circuit_breaker_state,
            last_updated: Instant::now(),
        }
    }
}

/// Event loss statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventLossStatistics {
    pub overall_loss_rate: f32,
    pub total_events: u64,
    pub dropped_events: u64,
    pub meets_target: bool,
    pub module_statistics: HashMap<ModuleId, ModuleStatistics>,
    pub degradation_level: DegradationLevel,
    pub circuit_breaker_state: CircuitBreakerState,
    pub last_updated: Instant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleStatistics {
    pub loss_rate: f32,
    pub queue_utilization: f32,
    pub queue_status: QueueStatus,
    pub total_events: u64,
    pub dropped_events: u64,
}