//! Resource management and monitoring

use crate::error::{OrchestratorError, OrchestratorResult};
use crate::module_registry::ModuleRegistry;
use dashmap::DashMap;
use skelly_jelly_event_bus::ModuleId;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
    collections::VecDeque,
};
use sysinfo::{System, Cpu, Process, Pid, ProcessExt, SystemExt};
use tokio::{
    task::JoinHandle,
    sync::{RwLock, mpsc, watch},
    time::{interval, timeout},
};
use tracing::{debug, warn, error, info};
use rand;
use serde::{Deserialize, Serialize};
use crossbeam_channel::{bounded, unbounded, Sender, Receiver};
use parking_lot::RwLock as ParkingLotRwLock;

/// Battery optimization settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatteryOptimization {
    pub enabled: bool,
    pub power_save_threshold: f32, // Battery percentage
    pub cpu_throttle_factor: f32,
    pub background_task_delay: Duration,
    pub reduced_monitoring_interval: Duration,
}

impl Default for BatteryOptimization {
    fn default() -> Self {
        Self {
            enabled: true,
            power_save_threshold: 20.0, // Below 20% battery
            cpu_throttle_factor: 0.5,   // Reduce CPU usage by 50%
            background_task_delay: Duration::from_millis(500),
            reduced_monitoring_interval: Duration::from_secs(5),
        }
    }
}

/// Resource priority levels for adaptive allocation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum ResourcePriority {
    Critical,
    High,
    Normal,
    Low,
    Background,
}

/// Resource limits for a module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub max_cpu_percent: f32,
    pub max_memory_mb: usize,
    pub max_file_handles: usize,
    pub max_threads: usize,
    pub max_network_kbps: f32,
    pub max_disk_io_kbps: f32,
    pub priority: ResourcePriority,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_cpu_percent: 2.0,   // Target: <2% CPU average
            max_memory_mb: 200,     // Target: <200MB total
            max_file_handles: 100,
            max_threads: 4,
            max_network_kbps: 1000.0,
            max_disk_io_kbps: 5000.0,
            priority: ResourcePriority::Normal,
        }
    }
}

impl ResourceLimits {
    pub fn new(cpu_percent: f32, memory_mb: usize) -> Self {
        Self {
            max_cpu_percent: cpu_percent,
            max_memory_mb: memory_mb,
            ..Default::default()
        }
    }

    pub fn with_file_handles(mut self, handles: usize) -> Self {
        self.max_file_handles = handles;
        self
    }

    pub fn with_threads(mut self, threads: usize) -> Self {
        self.max_threads = threads;
        self
    }
}

/// Current resource usage for a module
#[derive(Debug, Clone)]
pub struct ResourceUsage {
    pub cpu_percent: f32,
    pub memory_mb: usize,
    pub file_handles: usize,
    pub threads: usize,
    pub timestamp: Instant,
}

impl ResourceUsage {
    pub fn exceeds(&self, limits: &ResourceLimits) -> bool {
        self.cpu_percent > limits.max_cpu_percent ||
        self.memory_mb > limits.max_memory_mb ||
        self.file_handles > limits.max_file_handles ||
        self.threads > limits.max_threads
    }

    pub fn exceeds_any(&self, limits: &ResourceLimits) -> Vec<String> {
        let mut violations = Vec::new();

        if self.cpu_percent > limits.max_cpu_percent {
            violations.push(format!(
                "CPU: {:.1}% > {:.1}%",
                self.cpu_percent,
                limits.max_cpu_percent
            ));
        }

        if self.memory_mb > limits.max_memory_mb {
            violations.push(format!(
                "Memory: {}MB > {}MB",
                self.memory_mb,
                limits.max_memory_mb
            ));
        }

        if self.file_handles > limits.max_file_handles {
            violations.push(format!(
                "File handles: {} > {}",
                self.file_handles,
                limits.max_file_handles
            ));
        }

        if self.threads > limits.max_threads {
            violations.push(format!(
                "Threads: {} > {}",
                self.threads,
                limits.max_threads
            ));
        }

        violations
    }
}

/// Resource allocations across all modules
#[derive(Debug, Clone)]
pub struct ResourceAllocations {
    pub cpu_usage: HashMap<ModuleId, f32>,
    pub memory_usage: HashMap<ModuleId, usize>,
    pub thread_count: HashMap<ModuleId, usize>,
    pub file_handle_count: HashMap<ModuleId, usize>,
    pub last_updated: Instant,
}

impl ResourceAllocations {
    pub fn new() -> Self {
        Self {
            cpu_usage: HashMap::new(),
            memory_usage: HashMap::new(),
            thread_count: HashMap::new(),
            file_handle_count: HashMap::new(),
            last_updated: Instant::now(),
        }
    }

    pub fn total_cpu_usage(&self) -> f32 {
        self.cpu_usage.values().sum()
    }

    pub fn total_memory_usage(&self) -> usize {
        self.memory_usage.values().sum()
    }

    pub fn total_threads(&self) -> usize {
        self.thread_count.values().sum()
    }

    pub fn total_file_handles(&self) -> usize {
        self.file_handle_count.values().sum()
    }
}

/// System-wide resource information
#[derive(Debug, Clone)]
pub struct SystemResources {
    pub total_cpu_usage: f32,
    pub total_memory_mb: usize,
    pub available_memory_mb: usize,
    pub disk_usage_mb: usize,
    pub network_bandwidth_kbps: f32,
    pub load_average: (f32, f32, f32), // 1min, 5min, 15min
    pub timestamp: Instant,
}

impl SystemResources {
    pub fn memory_usage_percent(&self) -> f32 {
        if self.total_memory_mb > 0 {
            ((self.total_memory_mb - self.available_memory_mb) as f32 / self.total_memory_mb as f32) * 100.0
        } else {
            0.0
        }
    }
}

/// Throttle controller for managing resource usage
pub struct ThrottleController {
    /// Throttling actions per module
    throttle_actions: DashMap<ModuleId, ThrottleAction>,
    
    /// History of throttling events
    throttle_history: DashMap<ModuleId, Vec<ThrottleEvent>>,
}

#[derive(Debug, Clone)]
pub enum ThrottleAction {
    None,
    ReduceFrequency { factor: f32 },
    PauseProcessing { duration: Duration },
    LimitConcurrency { max_tasks: usize },
}

#[derive(Debug, Clone)]
pub struct ThrottleEvent {
    pub timestamp: Instant,
    pub action: ThrottleAction,
    pub reason: String,
    pub usage: ResourceUsage,
}

impl ThrottleController {
    pub fn new() -> Self {
        Self {
            throttle_actions: DashMap::new(),
            throttle_history: DashMap::new(),
        }
    }

    pub async fn throttle(
        &self,
        module_id: ModuleId,
        usage: &ResourceUsage,
        limits: &ResourceLimits,
    ) -> OrchestratorResult<()> {
        let violations = usage.exceeds_any(limits);
        if violations.is_empty() {
            return Ok(());
        }

        let action = self.determine_throttle_action(module_id, usage, limits);
        let reason = format!("Resource violations: {}", violations.join(", "));

        // Record throttle event
        let event = ThrottleEvent {
            timestamp: Instant::now(),
            action: action.clone(),
            reason: reason.clone(),
            usage: usage.clone(),
        };

        self.throttle_history
            .entry(module_id)
            .or_insert_with(Vec::new)
            .push(event);

        // Apply throttle action
        self.throttle_actions.insert(module_id, action.clone());

        warn!(
            "Throttling module {} due to resource violations: {} - Action: {:?}",
            module_id, reason, action
        );

        // In a real implementation, we would send throttling commands to the module
        Ok(())
    }

    fn determine_throttle_action(
        &self,
        _module_id: ModuleId,
        usage: &ResourceUsage,
        limits: &ResourceLimits,
    ) -> ThrottleAction {
        // Simple throttling strategy based on severity
        let cpu_violation = usage.cpu_percent / limits.max_cpu_percent;
        let memory_violation = usage.memory_mb as f32 / limits.max_memory_mb as f32;
        let max_violation = cpu_violation.max(memory_violation);

        if max_violation > 2.0 {
            // Severe violation - pause processing
            ThrottleAction::PauseProcessing {
                duration: Duration::from_secs(30),
            }
        } else if max_violation > 1.5 {
            // Moderate violation - limit concurrency
            ThrottleAction::LimitConcurrency { max_tasks: 2 }
        } else if max_violation > 1.2 {
            // Minor violation - reduce frequency
            ThrottleAction::ReduceFrequency { factor: 0.5 }
        } else {
            ThrottleAction::None
        }
    }

    pub fn get_current_throttle(&self, module_id: ModuleId) -> Option<ThrottleAction> {
        self.throttle_actions.get(&module_id).map(|entry| entry.clone())
    }

    pub fn clear_throttle(&self, module_id: ModuleId) {
        self.throttle_actions.remove(&module_id);
    }
}

/// Resource manager monitors and manages system resource allocation
pub struct ResourceManager {
    /// System resource monitoring
    system_monitor: Arc<tokio::sync::Mutex<System>>,
    
    /// Module registry for getting module info
    registry: Arc<ModuleRegistry>,
    
    /// Resource allocation tracking
    allocations: Arc<tokio::sync::RwLock<ResourceAllocations>>,
    
    /// Resource limits per module
    resource_limits: DashMap<ModuleId, ResourceLimits>,
    
    /// Throttling controller
    throttle_controller: ThrottleController,
    
    /// Current resource usage per module
    current_usage: DashMap<ModuleId, ResourceUsage>,
    
    /// System resource monitoring task
    monitor_task: Option<JoinHandle<()>>,
    
    /// Monitoring configuration
    check_interval: Duration,
    throttle_threshold: f32,
}

impl ResourceManager {
    pub fn new(
        registry: Arc<ModuleRegistry>,
        check_interval: Duration,
        throttle_threshold: f32,
    ) -> Self {
        let mut system = System::new_all();
        system.refresh_all();

        Self {
            system_monitor: Arc::new(tokio::sync::Mutex::new(system)),
            registry,
            allocations: Arc::new(tokio::sync::RwLock::new(ResourceAllocations::new())),
            resource_limits: DashMap::new(),
            throttle_controller: ThrottleController::new(),
            current_usage: DashMap::new(),
            monitor_task: None,
            check_interval,
            throttle_threshold,
        }
    }

    /// Start resource monitoring
    pub async fn start_monitoring(&mut self) -> OrchestratorResult<()> {
        info!("Starting resource monitoring");

        // Set default resource limits for all modules
        self.set_default_limits().await;

        // Start monitoring task
        let system_monitor = Arc::clone(&self.system_monitor);
        let allocations = Arc::clone(&self.allocations);
        let registry = Arc::clone(&self.registry);
        let current_usage = self.current_usage.clone();
        let check_interval = self.check_interval;

        let monitor_task = tokio::spawn(async move {
            let mut interval = tokio::time::interval(check_interval);

            loop {
                interval.tick().await;

                // Update system information
                {
                    let mut system = system_monitor.lock().await;
                    system.refresh_all();
                }

                // Update resource allocations
                if let Err(e) = Self::update_resource_allocations(
                    &system_monitor,
                    &allocations,
                    &Arc::new(current_usage.clone()),
                ).await {
                    error!("Failed to update resource allocations: {}", e);
                }
            }
        });

        self.monitor_task = Some(monitor_task);
        info!("Resource monitoring started");
        Ok(())
    }

    /// Stop resource monitoring
    pub async fn stop_monitoring(&mut self) {
        info!("Stopping resource monitoring");

        if let Some(task) = self.monitor_task.take() {
            task.abort();
        }

        info!("Resource monitoring stopped");
    }

    /// Set resource limits for a module
    pub fn set_resource_limits(&self, module_id: ModuleId, limits: ResourceLimits) {
        info!("Setting resource limits for {}: {:?}", module_id, limits);
        self.resource_limits.insert(module_id, limits);
    }

    /// Get resource limits for a module
    pub fn get_resource_limits(&self, module_id: ModuleId) -> ResourceLimits {
        self.resource_limits
            .get(&module_id)
            .map(|entry| entry.clone())
            .unwrap_or_default()
    }

    /// Enforce resource limits across all modules
    pub async fn enforce_limits(&self) -> OrchestratorResult<()> {
        debug!("Enforcing resource limits");

        for entry in self.current_usage.iter() {
            let module_id = *entry.key();
            let usage = entry.value();

            if let Some(limits) = self.resource_limits.get(&module_id) {
                if usage.exceeds(&limits) {
                    self.throttle_controller
                        .throttle(module_id, usage, &limits)
                        .await?;
                }
            }
        }

        Ok(())
    }

    /// Get current system resources
    pub async fn get_system_resources(&self) -> OrchestratorResult<SystemResources> {
        let mut system = self.system_monitor.lock().await;
        system.refresh_all();
        
        let total_memory = system.total_memory() as f32 / 1_024_000.0; // Convert to MB
        let available_memory = system.available_memory() as f32 / 1_024_000.0;
        let used_memory = total_memory - available_memory;

        // Calculate average CPU usage
        let cpu_usage = system.cpus()
            .iter()
            .map(|cpu| cpu.cpu_usage())
            .sum::<f32>() / system.cpus().len() as f32;

        Ok(SystemResources {
            total_cpu_usage: cpu_usage,
            total_memory_mb: used_memory as usize,
            available_memory_mb: available_memory as usize,
            disk_usage_mb: 0, // Simplified - would need disk monitoring
            network_bandwidth_kbps: 0.0, // Simplified - would need network monitoring
            load_average: (0.0, 0.0, 0.0), // Simplified - system.load_average() may not be available on all platforms
            timestamp: Instant::now(),
        })
    }

    /// Get current resource allocations
    pub async fn get_allocations(&self) -> ResourceAllocations {
        let allocations = self.allocations.read().await;
        allocations.clone()
    }

    /// Get resource usage for a specific module
    pub fn get_module_usage(&self, module_id: ModuleId) -> Option<ResourceUsage> {
        self.current_usage.get(&module_id).map(|entry| entry.clone())
    }

    /// Set default resource limits for all modules
    async fn set_default_limits(&self) {
        let modules = self.registry.get_all_modules();
        
        for descriptor in modules {
            let limits = match descriptor.id {
                ModuleId::DataCapture => ResourceLimits::new(15.0, 256), // Lower resource usage
                ModuleId::Storage => ResourceLimits::new(20.0, 512),
                ModuleId::AnalysisEngine => ResourceLimits::new(30.0, 1024), // Higher resource usage
                ModuleId::AiIntegration => ResourceLimits::new(25.0, 768),
                ModuleId::CuteFigurine => ResourceLimits::new(10.0, 256), // UI module
                ModuleId::Gamification => ResourceLimits::new(10.0, 256),
                ModuleId::EventBus => ResourceLimits::new(15.0, 256),
                ModuleId::Orchestrator => ResourceLimits::new(20.0, 512),
            };
            
            self.set_resource_limits(descriptor.id, limits);
        }
    }

    /// Update resource allocations (called by monitoring task)
    async fn update_resource_allocations(
        system_monitor: &Arc<tokio::sync::Mutex<System>>,
        allocations: &Arc<tokio::sync::RwLock<ResourceAllocations>>,
        current_usage: &Arc<DashMap<ModuleId, ResourceUsage>>,
    ) -> OrchestratorResult<()> {
        let _system = system_monitor.lock().await;
        let mut alloc = allocations.write().await;

        // For demonstration, we'll simulate resource usage
        // In a real implementation, this would track actual process resources
        
        alloc.cpu_usage.clear();
        alloc.memory_usage.clear();
        alloc.thread_count.clear();
        alloc.file_handle_count.clear();

        // Simulate resource usage for each module
        let modules = [
            ModuleId::DataCapture,
            ModuleId::Storage,
            ModuleId::AnalysisEngine,
            ModuleId::Gamification,
            ModuleId::AiIntegration,
            ModuleId::CuteFigurine,
            ModuleId::EventBus,
        ];

        for module_id in modules {
            // Simulate varying resource usage
            let cpu_usage = (rand::random::<f32>() * 20.0) + 5.0; // 5-25% CPU
            let memory_usage = (rand::random::<u32>() % 600 + 200) as usize; // 200-800MB
            let threads = rand::random::<usize>() % 6 + 2; // 2-8 threads
            let file_handles = rand::random::<usize>() % 90 + 10; // 10-100 file handles

            alloc.cpu_usage.insert(module_id, cpu_usage);
            alloc.memory_usage.insert(module_id, memory_usage);
            alloc.thread_count.insert(module_id, threads);
            alloc.file_handle_count.insert(module_id, file_handles);

            // Update current usage
            let usage = ResourceUsage {
                cpu_percent: cpu_usage,
                memory_mb: memory_usage,
                file_handles: file_handles,
                threads,
                timestamp: Instant::now(),
            };
            current_usage.insert(module_id, usage);
        }

        alloc.last_updated = Instant::now();
        debug!("Updated resource allocations for {} modules", modules.len());
        
        Ok(())
    }
}