//! Event monitors for different types of system events

use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{info, warn, error};

use crate::{DataCaptureConfig, DataCaptureError, Result};

pub mod keystroke;
pub mod mouse;
pub mod window;
pub mod screenshot;
pub mod process;
pub mod resource;

// Import the generic monitor implementations
use keystroke::KeystrokeMonitor;
use mouse::MouseMonitor;
use window::WindowMonitor;
use screenshot::ScreenshotMonitor;
use process::ProcessMonitor;
use resource::ResourceMonitor;

// Re-export event types from storage module
pub use skelly_jelly_storage::{
    RawEvent, KeystrokeEvent, MouseMoveEvent, MouseClickEvent, 
    WindowFocusEvent, ScreenshotEvent, ProcessEvent, ResourceEvent
};

/// Common trait for all event monitors
#[async_trait]
pub trait EventMonitor: Send + Sync {
    /// Start the monitor
    async fn start(&mut self) -> Result<()>;
    
    /// Stop the monitor
    async fn stop(&mut self) -> Result<()>;
    
    /// Check if the monitor is currently running
    fn is_running(&self) -> bool;
    
    /// Get the name of this monitor
    fn name(&self) -> &'static str;
    
    /// Get current performance statistics
    fn stats(&self) -> MonitorStats;
    
    /// Update configuration
    async fn update_config(&mut self, config: &DataCaptureConfig) -> Result<()>;
}

/// Performance statistics for monitors
#[derive(Debug, Clone, Default)]
pub struct MonitorStats {
    pub events_captured: u64,
    pub events_dropped: u64,
    pub cpu_usage: f32,
    pub memory_usage: u64,
    pub errors: u64,
}

/// Enum for different monitor types to avoid object safety issues
#[cfg(target_os = "macos")]
pub enum Monitor {
    Keystroke(crate::platform::macos::MacOSKeystrokeMonitor),
    Mouse(crate::platform::macos::MacOSMouseMonitor), 
    Window(crate::platform::macos::MacOSWindowMonitor),
    Screenshot(crate::platform::macos::MacOSScreenshotMonitor),
    Process(crate::platform::macos::MacOSProcessMonitor),
    Resource(crate::platform::macos::MacOSResourceMonitor),
}

/// Fallback generic monitor enum for platforms without specific implementations
#[cfg(not(target_os = "macos"))]
pub enum Monitor {
    Keystroke(KeystrokeMonitor),
    Mouse(MouseMonitor),
    Window(WindowMonitor),
    Screenshot(ScreenshotMonitor),
    Process(ProcessMonitor),
    Resource(ResourceMonitor),
}

#[cfg(target_os = "macos")]
#[async_trait]
impl EventMonitor for Monitor {
    async fn start(&mut self) -> Result<()> {
        match self {
            Monitor::Keystroke(m) => m.start().await,
            Monitor::Mouse(m) => m.start().await,
            Monitor::Window(m) => m.start().await,
            Monitor::Screenshot(m) => m.start().await,
            Monitor::Process(m) => m.start().await,
            Monitor::Resource(m) => m.start().await,
        }
    }
    
    async fn stop(&mut self) -> Result<()> {
        match self {
            Monitor::Keystroke(m) => m.stop().await,
            Monitor::Mouse(m) => m.stop().await,
            Monitor::Window(m) => m.stop().await,
            Monitor::Screenshot(m) => m.stop().await,
            Monitor::Process(m) => m.stop().await,
            Monitor::Resource(m) => m.stop().await,
        }
    }
    
    fn is_running(&self) -> bool {
        match self {
            Monitor::Keystroke(m) => m.is_running(),
            Monitor::Mouse(m) => m.is_running(),
            Monitor::Window(m) => m.is_running(),
            Monitor::Screenshot(m) => m.is_running(),
            Monitor::Process(m) => m.is_running(),
            Monitor::Resource(m) => m.is_running(),
        }
    }
    
    fn name(&self) -> &'static str {
        match self {
            Monitor::Keystroke(m) => m.name(),
            Monitor::Mouse(m) => m.name(),
            Monitor::Window(m) => m.name(),
            Monitor::Screenshot(m) => m.name(),
            Monitor::Process(m) => m.name(),
            Monitor::Resource(m) => m.name(),
        }
    }
    
    fn stats(&self) -> MonitorStats {
        match self {
            Monitor::Keystroke(m) => m.stats(),
            Monitor::Mouse(m) => m.stats(),
            Monitor::Window(m) => m.stats(),
            Monitor::Screenshot(m) => m.stats(),
            Monitor::Process(m) => m.stats(),
            Monitor::Resource(m) => m.stats(),
        }
    }
    
    async fn update_config(&mut self, config: &DataCaptureConfig) -> Result<()> {
        match self {
            Monitor::Keystroke(m) => m.update_config(config).await,
            Monitor::Mouse(m) => m.update_config(config).await,
            Monitor::Window(m) => m.update_config(config).await,
            Monitor::Screenshot(m) => m.update_config(config).await,
            Monitor::Process(m) => m.update_config(config).await,
            Monitor::Resource(m) => m.update_config(config).await,
        }
    }
}

/// Implementation for generic monitors
#[cfg(not(target_os = "macos"))]
#[async_trait]
impl EventMonitor for Monitor {
    async fn start(&mut self) -> Result<()> {
        match self {
            Monitor::Keystroke(m) => m.start().await,
            Monitor::Mouse(m) => m.start().await,
            Monitor::Window(m) => m.start().await,
            Monitor::Screenshot(m) => m.start().await,
            Monitor::Process(m) => m.start().await,
            Monitor::Resource(m) => m.start().await,
        }
    }
    
    async fn stop(&mut self) -> Result<()> {
        match self {
            Monitor::Keystroke(m) => m.stop().await,
            Monitor::Mouse(m) => m.stop().await,
            Monitor::Window(m) => m.stop().await,
            Monitor::Screenshot(m) => m.stop().await,
            Monitor::Process(m) => m.stop().await,
            Monitor::Resource(m) => m.stop().await,
        }
    }
    
    fn is_running(&self) -> bool {
        match self {
            Monitor::Keystroke(m) => m.is_running(),
            Monitor::Mouse(m) => m.is_running(),
            Monitor::Window(m) => m.is_running(),
            Monitor::Screenshot(m) => m.is_running(),
            Monitor::Process(m) => m.is_running(),
            Monitor::Resource(m) => m.is_running(),
        }
    }
    
    fn name(&self) -> &'static str {
        match self {
            Monitor::Keystroke(m) => m.name(),
            Monitor::Mouse(m) => m.name(),
            Monitor::Window(m) => m.name(),
            Monitor::Screenshot(m) => m.name(),
            Monitor::Process(m) => m.name(),
            Monitor::Resource(m) => m.name(),
        }
    }
    
    fn stats(&self) -> MonitorStats {
        match self {
            Monitor::Keystroke(m) => m.stats(),
            Monitor::Mouse(m) => m.stats(),
            Monitor::Window(m) => m.stats(),
            Monitor::Screenshot(m) => m.stats(),
            Monitor::Process(m) => m.stats(),
            Monitor::Resource(m) => m.stats(),
        }
    }
    
    async fn update_config(&mut self, config: &DataCaptureConfig) -> Result<()> {
        match self {
            Monitor::Keystroke(m) => m.update_config(config).await,
            Monitor::Mouse(m) => m.update_config(config).await,
            Monitor::Window(m) => m.update_config(config).await,
            Monitor::Screenshot(m) => m.update_config(config).await,
            Monitor::Process(m) => m.update_config(config).await,
            Monitor::Resource(m) => m.update_config(config).await,
        }
    }
}

/// Manages all active monitors
pub struct MonitorManager {
    config: DataCaptureConfig,
    event_sender: mpsc::Sender<RawEvent>,
    monitors: Vec<Monitor>,
    stats: ManagerStats,
}

#[derive(Debug, Clone, Default)]
pub struct ManagerStats {
    pub total_events_captured: u64,
    pub total_events_dropped: u64,
    pub active_monitors: usize,
    pub total_cpu_usage: f32,
    pub total_memory_usage: u64,
}

impl MonitorManager {
    /// Create a new monitor manager
    pub async fn new(config: DataCaptureConfig, event_sender: mpsc::Sender<RawEvent>) -> Result<Self> {
        info!("Initializing monitor manager");
        
        let mut monitors: Vec<Monitor> = Vec::new();
        
        // Initialize platform-specific monitors
        #[cfg(target_os = "macos")]
        {
            use crate::platform::macos::*;
            
            if config.monitors.keystroke.enabled {
                let monitor = MacOSKeystrokeMonitor::new(
                    config.monitors.keystroke.clone(),
                    event_sender.clone()
                ).await?;
                monitors.push(Monitor::Keystroke(monitor));
            }
            
            if config.monitors.mouse.enabled {
                let monitor = MacOSMouseMonitor::new(
                    config.monitors.mouse.clone(),
                    event_sender.clone()
                ).await?;
                monitors.push(Monitor::Mouse(monitor));
            }
            
            if config.monitors.window.enabled {
                let monitor = MacOSWindowMonitor::new(
                    config.monitors.window.clone(),
                    event_sender.clone()
                ).await?;
                monitors.push(Monitor::Window(monitor));
            }
            
            if config.monitors.screenshot.enabled {
                let monitor = MacOSScreenshotMonitor::new(
                    config.monitors.screenshot.clone(),
                    config.privacy.clone(),
                    event_sender.clone()
                ).await?;
                monitors.push(Monitor::Screenshot(monitor));
            }
            
            if config.monitors.process.enabled {
                let monitor = MacOSProcessMonitor::new(
                    config.monitors.process.clone(),
                    event_sender.clone()
                ).await?;
                monitors.push(Monitor::Process(monitor));
            }
            
            if config.monitors.resource.enabled {
                let monitor = MacOSResourceMonitor::new(
                    config.monitors.resource.clone(),
                    event_sender.clone()
                ).await?;
                monitors.push(Monitor::Resource(monitor));
            }
        }
        
        #[cfg(not(target_os = "macos"))]
        {
            // Use generic monitor implementations for non-macOS platforms
            if config.monitors.keystroke.enabled {
                let monitor = KeystrokeMonitor::new(
                    config.monitors.keystroke.clone(),
                    event_sender.clone()
                );
                monitors.push(Monitor::Keystroke(monitor));
            }
            
            if config.monitors.mouse.enabled {
                let monitor = MouseMonitor::new(
                    config.monitors.mouse.clone(),
                    event_sender.clone()
                );
                monitors.push(Monitor::Mouse(monitor));
            }
            
            if config.monitors.window.enabled {
                let monitor = WindowMonitor::new(
                    config.monitors.window.clone(),
                    event_sender.clone()
                );
                monitors.push(Monitor::Window(monitor));
            }
            
            if config.monitors.screenshot.enabled {
                let monitor = ScreenshotMonitor::new(
                    config.monitors.screenshot.clone(),
                    event_sender.clone()
                );
                monitors.push(Monitor::Screenshot(monitor));
            }
            
            if config.monitors.process.enabled {
                let monitor = ProcessMonitor::new(
                    config.monitors.process.clone(),
                    event_sender.clone()
                );
                monitors.push(Monitor::Process(monitor));
            }
            
            if config.monitors.resource.enabled {
                let monitor = ResourceMonitor::new(
                    config.monitors.resource.clone(),
                    event_sender.clone()
                );
                monitors.push(Monitor::Resource(monitor));
            }
        }
        
        info!("Initialized {} monitors", monitors.len());
        
        Ok(Self {
            config,
            event_sender,
            monitors,
            stats: ManagerStats::default(),
        })
    }
    
    /// Start all monitors
    pub async fn start_all(&mut self) -> Result<()> {
        info!("Starting all monitors");
        
        let mut errors = Vec::new();
        
        for monitor in &mut self.monitors {
            if let Err(e) = monitor.start().await {
                error!("Failed to start monitor {}: {}", monitor.name(), e);
                errors.push(e);
            } else {
                info!("Started monitor: {}", monitor.name());
            }
        }
        
        self.update_stats();
        
        if !errors.is_empty() {
            warn!("Some monitors failed to start: {} errors", errors.len());
            // Continue with successfully started monitors
        }
        
        info!("Started {} out of {} monitors", 
            self.monitors.iter().filter(|m| m.is_running()).count(),
            self.monitors.len()
        );
        
        Ok(())
    }
    
    /// Stop all monitors
    pub async fn stop_all(&mut self) -> Result<()> {
        info!("Stopping all monitors");
        
        let mut errors = Vec::new();
        
        for monitor in &mut self.monitors {
            if let Err(e) = monitor.stop().await {
                error!("Failed to stop monitor {}: {}", monitor.name(), e);
                errors.push(e);
            } else {
                info!("Stopped monitor: {}", monitor.name());
            }
        }
        
        self.update_stats();
        
        if !errors.is_empty() {
            warn!("Some monitors failed to stop cleanly: {} errors", errors.len());
        }
        
        Ok(())
    }
    
    /// Update configuration for all monitors
    pub async fn update_config(&mut self, config: DataCaptureConfig) -> Result<()> {
        info!("Updating monitor configurations");
        
        for monitor in &mut self.monitors {
            if let Err(e) = monitor.update_config(&config).await {
                error!("Failed to update config for monitor {}: {}", monitor.name(), e);
            }
        }
        
        self.config = config;
        Ok(())
    }
    
    /// Get total events captured across all monitors
    pub fn total_events_captured(&self) -> u64 {
        self.stats.total_events_captured
    }
    
    /// Get total events dropped across all monitors
    pub fn total_events_dropped(&self) -> u64 {
        self.stats.total_events_dropped
    }
    
    /// Get number of active monitors
    pub fn active_monitor_count(&self) -> usize {
        self.monitors.iter().filter(|m| m.is_running()).count()
    }
    
    /// Get current CPU usage
    pub fn current_cpu_usage(&self) -> f32 {
        self.stats.total_cpu_usage
    }
    
    /// Get current memory usage
    pub fn current_memory_usage(&self) -> u64 {
        self.stats.total_memory_usage
    }
    
    /// Update aggregated statistics
    fn update_stats(&mut self) {
        let mut stats = ManagerStats::default();
        
        for monitor in &self.monitors {
            let monitor_stats = monitor.stats();
            stats.total_events_captured += monitor_stats.events_captured;
            stats.total_events_dropped += monitor_stats.events_dropped;
            stats.total_cpu_usage += monitor_stats.cpu_usage;
            stats.total_memory_usage += monitor_stats.memory_usage;
        }
        
        stats.active_monitors = self.active_monitor_count();
        self.stats = stats;
    }
}

/// Common monitor utilities
pub mod utils {
    use std::time::{Duration, Instant};
    use parking_lot::RwLock;
    
    /// Simple rate limiter for event throttling
    pub struct RateLimiter {
        last_event: RwLock<Instant>,
        min_interval: Duration,
    }
    
    impl RateLimiter {
        pub fn new(min_interval: Duration) -> Self {
            Self {
                last_event: RwLock::new(Instant::now() - min_interval),
                min_interval,
            }
        }
        
        pub fn should_allow(&self) -> bool {
            let now = Instant::now();
            let mut last = self.last_event.write();
            
            if now.duration_since(*last) >= self.min_interval {
                *last = now;
                true
            } else {
                false
            }
        }
    }
    
    /// Ring buffer for event batching
    pub struct EventBuffer<T> {
        buffer: Vec<Option<T>>,
        head: usize,
        tail: usize,
        size: usize,
        capacity: usize,
    }
    
    impl<T> EventBuffer<T> {
        pub fn new(capacity: usize) -> Self {
            Self {
                buffer: (0..capacity).map(|_| None).collect(),
                head: 0,
                tail: 0,
                size: 0,
                capacity,
            }
        }
        
        pub fn push(&mut self, item: T) -> Option<T> {
            let dropped = if self.size == self.capacity {
                self.buffer[self.tail].take()
            } else {
                None
            };
            
            self.buffer[self.head] = Some(item);
            self.head = (self.head + 1) % self.capacity;
            
            if self.size == self.capacity {
                self.tail = (self.tail + 1) % self.capacity;
            } else {
                self.size += 1;
            }
            
            dropped
        }
        
        pub fn drain(&mut self) -> Vec<T> {
            let mut result = Vec::with_capacity(self.size);
            
            while self.size > 0 {
                if let Some(item) = self.buffer[self.tail].take() {
                    result.push(item);
                }
                self.tail = (self.tail + 1) % self.capacity;
                self.size -= 1;
            }
            
            self.head = 0;
            self.tail = 0;
            
            result
        }
        
        pub fn len(&self) -> usize {
            self.size
        }
        
        pub fn is_empty(&self) -> bool {
            self.size == 0
        }
    }
}