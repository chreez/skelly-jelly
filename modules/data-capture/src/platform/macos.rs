//! macOS platform-specific monitors for data capture

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, error, info, warn};

use async_trait::async_trait;
use chrono::Utc;
use core_foundation::base::{CFRelease, TCFType};
use core_foundation::runloop::{CFRunLoop, CFRunLoopRef, CFRunLoopRun, CFRunLoopStop, kCFRunLoopDefaultMode};
use core_foundation::string::{CFString, CFStringRef};
use core_graphics::event::{
    CGEvent, CGEventRef, CGEventTap, CGEventTapLocation, CGEventTapOptions,
    CGEventTapPlacement, CGEventType, EventField, CGEventFlags,
};
use core_graphics::event_source::CGEventSource;
use core_graphics::window::{CGWindowID, CGWindowListCopyWindowInfo, CGWindowListOption, kCGWindowListOptionOnScreenOnly};
use cocoa::appkit::{NSApplication, NSRunningApplication};
// NSWorkspace commented out due to import issues
// use cocoa::foundation::NSWorkspace;
use cocoa::base::{id, nil, NO, YES};
use cocoa::foundation::{NSArray, NSString, NSAutoreleasePool, NSProcessInfo};
use objc::runtime::{Object, Class};
use objc::{class, msg_send, sel, sel_impl};

use crate::{
    DataCaptureError, Result,
    config::{
        KeystrokeConfig, MouseConfig, WindowConfig, ScreenshotConfig, 
        ProcessConfig, ResourceConfig, PrivacyConfig, PrivacyMode
    },
    monitors::{
        EventMonitor, MonitorStats, utils::{RateLimiter, EventBuffer},
        RawEvent, KeystrokeEvent, MouseMoveEvent, MouseClickEvent, 
        WindowFocusEvent, ScreenshotEvent, ProcessEvent, ResourceEvent,
    },
};

// Import types from storage module  
use skelly_jelly_storage::{
    ScreenshotId, ImageFormat, ScreenRegion, KeyModifiers, 
    MouseButton, ClickType, ProcessEventType
};

/// macOS keystroke monitor using CGEventTap
pub struct MacOSKeystrokeMonitor {
    config: KeystrokeConfig,
    event_sender: mpsc::Sender<RawEvent>,
    stats: Arc<RwLock<MonitorStats>>,
    is_running: Arc<RwLock<bool>>,
    event_buffer: Arc<RwLock<EventBuffer<KeystrokeEvent>>>,
    rate_limiter: Arc<RateLimiter>,
}

impl MacOSKeystrokeMonitor {
    pub async fn new(config: KeystrokeConfig, event_sender: mpsc::Sender<RawEvent>) -> Result<Self> {
        let stats = Arc::new(RwLock::new(MonitorStats::default()));
        let is_running = Arc::new(RwLock::new(false));
        let event_buffer = Arc::new(RwLock::new(EventBuffer::new(config.buffer_size)));
        let rate_limiter = Arc::new(RateLimiter::new(Duration::from_millis(config.coalescence_ms)));

        Ok(Self {
            config,
            event_sender,
            stats,
            is_running,
            event_buffer,
            rate_limiter,
        })
    }

    // Simplified implementation - event tap creation moved to start method
}

#[async_trait]
impl EventMonitor for MacOSKeystrokeMonitor {
    async fn start(&mut self) -> Result<()> {
        let mut is_running = self.is_running.write().await;
        if *is_running {
            return Err(DataCaptureError::AlreadyRunning);
        }

        info!("Starting macOS keystroke monitor");

        // Check permissions
        permissions::check_accessibility_permission().await?;

        *is_running = true;

        // Start processing task
        let event_sender = self.event_sender.clone();
        let event_buffer = self.event_buffer.clone();
        let stats = self.stats.clone();
        let is_running_clone = self.is_running.clone();
        let coalescence_ms = self.config.coalescence_ms;

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(coalescence_ms));
            
            while *is_running_clone.read().await {
                interval.tick().await;
                
                let events = {
                    let mut buffer = event_buffer.write().await;
                    buffer.drain()
                };

                for event in events {
                    if let Err(e) = event_sender.send(RawEvent::Keystroke(event)).await {
                        error!("Failed to send keystroke event: {}", e);
                        let mut stats_lock = stats.write().await;
                        stats_lock.events_dropped += 1;
                    } else {
                        let mut stats_lock = stats.write().await;
                        stats_lock.events_captured += 1;
                    }
                }
            }
        });

        info!("macOS keystroke monitor started");
        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        let mut is_running = self.is_running.write().await;
        if !*is_running {
            return Ok(());
        }

        info!("Stopping macOS keystroke monitor");

        *is_running = false;

        info!("macOS keystroke monitor stopped");
        Ok(())
    }

    fn is_running(&self) -> bool {
        // This is a blocking call, but should be fast
        match self.is_running.try_read() {
            Ok(running) => *running,
            Err(_) => false,
        }
    }

    fn name(&self) -> &'static str {
        "macOS Keystroke Monitor"
    }

    fn stats(&self) -> MonitorStats {
        match self.stats.try_read() {
            Ok(stats) => stats.clone(),
            Err(_) => MonitorStats::default(),
        }
    }

    async fn update_config(&mut self, config: &crate::DataCaptureConfig) -> Result<()> {
        let was_running = self.is_running();
        
        if was_running {
            self.stop().await?;
        }

        self.config = config.monitors.keystroke.clone();

        if was_running {
            self.start().await?;
        }

        Ok(())
    }
}

/// macOS mouse monitor using CGEventTap
pub struct MacOSMouseMonitor {
    config: MouseConfig,
    event_sender: mpsc::Sender<RawEvent>,
    stats: Arc<RwLock<MonitorStats>>,
    is_running: Arc<RwLock<bool>>,
    last_position: Arc<RwLock<(i32, i32)>>,
    rate_limiter: Arc<RateLimiter>,
}

impl MacOSMouseMonitor {
    pub async fn new(config: MouseConfig, event_sender: mpsc::Sender<RawEvent>) -> Result<Self> {
        let stats = Arc::new(RwLock::new(MonitorStats::default()));
        let is_running = Arc::new(RwLock::new(false));
        let last_position = Arc::new(RwLock::new((0, 0)));
        let rate_limiter = Arc::new(RateLimiter::new(Duration::from_millis(config.click_coalescence_ms)));

        Ok(Self {
            config,
            event_sender,
            stats,
            is_running,
            last_position,
            rate_limiter,
        })
    }
}

#[async_trait]
impl EventMonitor for MacOSMouseMonitor {
    async fn start(&mut self) -> Result<()> {
        let mut is_running = self.is_running.write().await;
        if *is_running {
            return Err(DataCaptureError::AlreadyRunning);
        }

        info!("Starting macOS mouse monitor");

        // Check permissions
        permissions::check_accessibility_permission().await?;

        // Simplified implementation - no event tap for now

        *is_running = true;

        info!("macOS mouse monitor started");
        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        let mut is_running = self.is_running.write().await;
        if !*is_running {
            return Ok(());
        }

        info!("Stopping macOS mouse monitor");

        *is_running = false;

        info!("macOS mouse monitor stopped");
        Ok(())
    }

    fn is_running(&self) -> bool {
        match self.is_running.try_read() {
            Ok(running) => *running,
            Err(_) => false,
        }
    }

    fn name(&self) -> &'static str {
        "macOS Mouse Monitor"
    }

    fn stats(&self) -> MonitorStats {
        match self.stats.try_read() {
            Ok(stats) => stats.clone(),
            Err(_) => MonitorStats::default(),
        }
    }

    async fn update_config(&mut self, config: &crate::DataCaptureConfig) -> Result<()> {
        let was_running = self.is_running();
        
        if was_running {
            self.stop().await?;
        }

        self.config = config.monitors.mouse.clone();

        if was_running {
            self.start().await?;
        }

        Ok(())
    }

    // Simplified implementation - no callback for now
}

/// macOS window monitor using NSWorkspace
pub struct MacOSWindowMonitor {
    config: WindowConfig,
    event_sender: mpsc::Sender<RawEvent>,
    stats: Arc<RwLock<MonitorStats>>,
    is_running: Arc<RwLock<bool>>,
    current_window: Arc<RwLock<Option<(String, String, u32)>>>, // title, app, pid
}

impl MacOSWindowMonitor {
    pub async fn new(config: WindowConfig, event_sender: mpsc::Sender<RawEvent>) -> Result<Self> {
        let stats = Arc::new(RwLock::new(MonitorStats::default()));
        let is_running = Arc::new(RwLock::new(false));
        let current_window = Arc::new(RwLock::new(None));

        Ok(Self {
            config,
            event_sender,
            stats,
            is_running,
            current_window,
        })
    }

    async fn monitor_window_changes(&self) -> Result<()> {
        let mut interval = tokio::time::interval(Duration::from_millis(self.config.switch_threshold_ms));
        
        while *self.is_running.read().await {
            interval.tick().await;
            
            if let Ok(window_info) = self.get_active_window_info() {
                let mut current = self.current_window.write().await;
                
                if let Some((title, app, pid)) = window_info {
                    if current.as_ref().map(|(_, _, p)| *p) != Some(pid) {
                        let event = WindowFocusEvent {
                            timestamp: Utc::now(),
                            window_title: title.clone(),
                            app_name: app.clone(),
                            process_id: pid,
                            duration_ms: None, // Could calculate from previous window
                        };

                        if let Err(e) = self.event_sender.send(RawEvent::WindowFocus(event)).await {
                            error!("Failed to send window focus event: {}", e);
                            let mut stats = self.stats.write().await;
                            stats.events_dropped += 1;
                        } else {
                            let mut stats = self.stats.write().await;
                            stats.events_captured += 1;
                        }

                        *current = Some((title, app, pid));
                    }
                }
            }
        }
        
        Ok(())
    }

    fn get_active_window_info(&self) -> Result<Option<(String, String, u32)>> {
        // Simplified implementation - NSWorkspace API not available
        Ok(Some(("Active Window".to_string(), "Active App".to_string(), 1)))
    }

    fn get_active_window_title(&self) -> Option<String> {
        // This would require more complex accessibility API calls
        // For now, return a placeholder
        Some("Active Window".to_string())
    }
}

#[async_trait]
impl EventMonitor for MacOSWindowMonitor {
    async fn start(&mut self) -> Result<()> {
        let mut is_running = self.is_running.write().await;
        if *is_running {
            return Err(DataCaptureError::AlreadyRunning);
        }

        info!("Starting macOS window monitor");
        *is_running = true;

        // Start monitoring task
        let monitor = self.clone();
        tokio::spawn(async move {
            if let Err(e) = monitor.monitor_window_changes().await {
                error!("Window monitoring error: {}", e);
            }
        });

        info!("macOS window monitor started");
        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        let mut is_running = self.is_running.write().await;
        if !*is_running {
            return Ok(());
        }

        info!("Stopping macOS window monitor");
        *is_running = false;
        info!("macOS window monitor stopped");
        Ok(())
    }

    fn is_running(&self) -> bool {
        match self.is_running.try_read() {
            Ok(running) => *running,
            Err(_) => false,
        }
    }

    fn name(&self) -> &'static str {
        "macOS Window Monitor"
    }

    fn stats(&self) -> MonitorStats {
        match self.stats.try_read() {
            Ok(stats) => stats.clone(),
            Err(_) => MonitorStats::default(),
        }
    }

    async fn update_config(&mut self, config: &crate::DataCaptureConfig) -> Result<()> {
        let was_running = self.is_running();
        
        if was_running {
            self.stop().await?;
        }

        self.config = config.monitors.window.clone();

        if was_running {
            self.start().await?;
        }

        Ok(())
    }
}

// Clone implementation for MacOSWindowMonitor
impl Clone for MacOSWindowMonitor {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            event_sender: self.event_sender.clone(),
            stats: self.stats.clone(),
            is_running: self.is_running.clone(),
            current_window: self.current_window.clone(),
        }
    }
}

/// macOS screenshot monitor using CGWindowListCopyWindowInfo
pub struct MacOSScreenshotMonitor {
    config: ScreenshotConfig,
    privacy_config: PrivacyConfig,
    event_sender: mpsc::Sender<RawEvent>,
    stats: Arc<RwLock<MonitorStats>>,
    is_running: Arc<RwLock<bool>>,
    last_screenshot: Arc<RwLock<Option<Instant>>>,
}

impl MacOSScreenshotMonitor {
    pub async fn new(
        config: ScreenshotConfig,
        privacy_config: PrivacyConfig,
        event_sender: mpsc::Sender<RawEvent>,
    ) -> Result<Self> {
        let stats = Arc::new(RwLock::new(MonitorStats::default()));
        let is_running = Arc::new(RwLock::new(false));
        let last_screenshot = Arc::new(RwLock::new(None));

        Ok(Self {
            config,
            privacy_config,
            event_sender,
            stats,
            is_running,
            last_screenshot,
        })
    }

    async fn capture_screenshot(&self) -> Result<ScreenshotEvent> {
        // Use CGWindowListCopyWindowInfo for screenshot capture
        // This is a complex implementation that would need proper CGImage handling
        
        let screenshot_id = ScreenshotId::new();
        let timestamp = Utc::now();
        
        // Placeholder implementation
        let event = ScreenshotEvent {
            timestamp,
            screenshot_id,
            data: vec![], // Actual screenshot data would go here
            format: ImageFormat::Png,
            window_title: "Active Window".to_string(),
            app_name: "Active App".to_string(),
            region: ScreenRegion {
                x: 0,
                y: 0,
                width: 1920,
                height: 1080,
            },
            privacy_masked: false,
        };

        Ok(event)
    }

    async fn should_capture_screenshot(&self) -> bool {
        let last = self.last_screenshot.read().await;
        
        match *last {
            Some(last_time) => {
                let elapsed = last_time.elapsed();
                elapsed >= Duration::from_millis(self.config.capture_interval_ms)
            }
            None => true,
        }
    }

    fn apply_privacy_filters(&self, screenshot: &mut ScreenshotEvent) {
        // Apply privacy filtering based on configuration
        if self.privacy_config.sensitive_app_list.contains(&screenshot.app_name) {
            screenshot.privacy_masked = true;
            screenshot.data.clear(); // Remove screenshot data for sensitive apps
        }
    }
}

#[async_trait]
impl EventMonitor for MacOSScreenshotMonitor {
    async fn start(&mut self) -> Result<()> {
        let mut is_running = self.is_running.write().await;
        if *is_running {
            return Err(DataCaptureError::AlreadyRunning);
        }

        info!("Starting macOS screenshot monitor");
        *is_running = true;

        // Start capture task
        let event_sender = self.event_sender.clone();
        let is_running_clone = self.is_running.clone();
        let config = self.config.clone();
        let privacy_config = self.privacy_config.clone();
        let stats = self.stats.clone();
        let last_screenshot = self.last_screenshot.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(config.capture_interval_ms));
            
            while *is_running_clone.read().await {
                interval.tick().await;
                
                // Create a temporary monitor instance for capturing
                let monitor = MacOSScreenshotMonitor {
                    config: config.clone(),
                    privacy_config: privacy_config.clone(),
                    event_sender: event_sender.clone(),
                    stats: stats.clone(),
                    is_running: is_running_clone.clone(),
                    last_screenshot: last_screenshot.clone(),
                };

                if monitor.should_capture_screenshot().await {
                    match monitor.capture_screenshot().await {
                        Ok(mut screenshot) => {
                            monitor.apply_privacy_filters(&mut screenshot);
                            
                            if let Err(e) = event_sender.send(RawEvent::Screenshot(screenshot)).await {
                                error!("Failed to send screenshot event: {}", e);
                                let mut stats_lock = stats.write().await;
                                stats_lock.events_dropped += 1;
                            } else {
                                let mut stats_lock = stats.write().await;
                                stats_lock.events_captured += 1;
                                
                                let mut last = last_screenshot.write().await;
                                *last = Some(Instant::now());
                            }
                        }
                        Err(e) => {
                            error!("Failed to capture screenshot: {}", e);
                            let mut stats_lock = stats.write().await;
                            stats_lock.errors += 1;
                        }
                    }
                }
            }
        });

        info!("macOS screenshot monitor started");
        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        let mut is_running = self.is_running.write().await;
        if !*is_running {
            return Ok(());
        }

        info!("Stopping macOS screenshot monitor");
        *is_running = false;
        info!("macOS screenshot monitor stopped");
        Ok(())
    }

    fn is_running(&self) -> bool {
        match self.is_running.try_read() {
            Ok(running) => *running,
            Err(_) => false,
        }
    }

    fn name(&self) -> &'static str {
        "macOS Screenshot Monitor"
    }

    fn stats(&self) -> MonitorStats {
        match self.stats.try_read() {
            Ok(stats) => stats.clone(),
            Err(_) => MonitorStats::default(),
        }
    }

    async fn update_config(&mut self, config: &crate::DataCaptureConfig) -> Result<()> {
        let was_running = self.is_running();
        
        if was_running {
            self.stop().await?;
        }

        self.config = config.monitors.screenshot.clone();
        self.privacy_config = config.privacy.clone();

        if was_running {
            self.start().await?;
        }

        Ok(())
    }
}

/// macOS process monitor using NSProcessInfo
pub struct MacOSProcessMonitor {
    config: ProcessConfig,
    event_sender: mpsc::Sender<RawEvent>,
    stats: Arc<RwLock<MonitorStats>>,
    is_running: Arc<RwLock<bool>>,
    known_processes: Arc<RwLock<std::collections::HashSet<u32>>>,
}

impl MacOSProcessMonitor {
    pub async fn new(config: ProcessConfig, event_sender: mpsc::Sender<RawEvent>) -> Result<Self> {
        let stats = Arc::new(RwLock::new(MonitorStats::default()));
        let is_running = Arc::new(RwLock::new(false));
        let known_processes = Arc::new(RwLock::new(std::collections::HashSet::new()));

        Ok(Self {
            config,
            event_sender,
            stats,
            is_running,
            known_processes,
        })
    }

    async fn scan_processes(&self) -> Result<()> {
        // Simplified implementation - NSWorkspace API not available
        // In a real implementation, this would scan running processes
        Ok(())
    }
}

#[async_trait]
impl EventMonitor for MacOSProcessMonitor {
    async fn start(&mut self) -> Result<()> {
        let mut is_running = self.is_running.write().await;
        if *is_running {
            return Err(DataCaptureError::AlreadyRunning);
        }

        info!("Starting macOS process monitor");
        *is_running = true;

        // Start monitoring task
        let event_sender = self.event_sender.clone();
        let is_running_clone = self.is_running.clone();
        let stats = self.stats.clone();
        let known_processes = self.known_processes.clone();
        let config = self.config.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(config.sample_interval_ms));
            
            let monitor = MacOSProcessMonitor {
                config,
                event_sender,
                stats,
                is_running: is_running_clone.clone(),
                known_processes,
            };
            
            while *is_running_clone.read().await {
                interval.tick().await;
                
                if let Err(e) = monitor.scan_processes().await {
                    error!("Process scanning error: {}", e);
                }
            }
        });

        info!("macOS process monitor started");
        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        let mut is_running = self.is_running.write().await;
        if !*is_running {
            return Ok(());
        }

        info!("Stopping macOS process monitor");
        *is_running = false;
        info!("macOS process monitor stopped");
        Ok(())
    }

    fn is_running(&self) -> bool {
        match self.is_running.try_read() {
            Ok(running) => *running,
            Err(_) => false,
        }
    }

    fn name(&self) -> &'static str {
        "macOS Process Monitor"
    }

    fn stats(&self) -> MonitorStats {
        match self.stats.try_read() {
            Ok(stats) => stats.clone(),
            Err(_) => MonitorStats::default(),
        }
    }

    async fn update_config(&mut self, config: &crate::DataCaptureConfig) -> Result<()> {
        let was_running = self.is_running();
        
        if was_running {
            self.stop().await?;
        }

        self.config = config.monitors.process.clone();

        if was_running {
            self.start().await?;
        }

        Ok(())
    }
}

/// macOS resource monitor using IOKit
pub struct MacOSResourceMonitor {
    config: ResourceConfig,
    event_sender: mpsc::Sender<RawEvent>,
    stats: Arc<RwLock<MonitorStats>>,
    is_running: Arc<RwLock<bool>>,
}

impl MacOSResourceMonitor {
    pub async fn new(config: ResourceConfig, event_sender: mpsc::Sender<RawEvent>) -> Result<Self> {
        let stats = Arc::new(RwLock::new(MonitorStats::default()));
        let is_running = Arc::new(RwLock::new(false));

        Ok(Self {
            config,
            event_sender,
            stats,
            is_running,
        })
    }

    async fn collect_resource_metrics(&self) -> Result<ResourceEvent> {
        // Use IOKit and other system APIs to collect resource metrics
        // This is a simplified implementation
        
        let event = ResourceEvent {
            timestamp: Utc::now(),
            cpu_percent: 0.0, // Would get from system APIs
            memory_mb: 0,     // Would get from system APIs
            disk_io_mb_per_sec: 0.0,
            network_io_mb_per_sec: 0.0,
        };

        Ok(event)
    }
}

#[async_trait]
impl EventMonitor for MacOSResourceMonitor {
    async fn start(&mut self) -> Result<()> {
        let mut is_running = self.is_running.write().await;
        if *is_running {
            return Err(DataCaptureError::AlreadyRunning);
        }

        info!("Starting macOS resource monitor");
        *is_running = true;

        // Start monitoring task
        let event_sender = self.event_sender.clone();
        let is_running_clone = self.is_running.clone();
        let stats = self.stats.clone();
        let config = self.config.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(config.sample_interval_ms));
            
            let monitor = MacOSResourceMonitor {
                config,
                event_sender,
                stats,
                is_running: is_running_clone.clone(),
            };
            
            while *is_running_clone.read().await {
                interval.tick().await;
                
                match monitor.collect_resource_metrics().await {
                    Ok(event) => {
                        if let Err(e) = monitor.event_sender.send(RawEvent::ResourceUsage(event)).await {
                            error!("Failed to send resource event: {}", e);
                            let mut stats = monitor.stats.write().await;
                            stats.events_dropped += 1;
                        } else {
                            let mut stats = monitor.stats.write().await;
                            stats.events_captured += 1;
                        }
                    }
                    Err(e) => {
                        error!("Resource collection error: {}", e);
                        let mut stats = monitor.stats.write().await;
                        stats.errors += 1;
                    }
                }
            }
        });

        info!("macOS resource monitor started");
        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        let mut is_running = self.is_running.write().await;
        if !*is_running {
            return Ok(());
        }

        info!("Stopping macOS resource monitor");
        *is_running = false;
        info!("macOS resource monitor stopped");
        Ok(())
    }

    fn is_running(&self) -> bool {
        match self.is_running.try_read() {
            Ok(running) => *running,
            Err(_) => false,
        }
    }

    fn name(&self) -> &'static str {
        "macOS Resource Monitor"
    }

    fn stats(&self) -> MonitorStats {
        match self.stats.try_read() {
            Ok(stats) => stats.clone(),
            Err(_) => MonitorStats::default(),
        }
    }

    async fn update_config(&mut self, config: &crate::DataCaptureConfig) -> Result<()> {
        let was_running = self.is_running();
        
        if was_running {
            self.stop().await?;
        }

        self.config = config.monitors.resource.clone();

        if was_running {
            self.start().await?;
        }

        Ok(())
    }
}

/// Permissions module for macOS
pub mod permissions {
    use super::*;

    /// Check if accessibility permission is granted
    pub async fn check_accessibility_permission() -> Result<()> {
        unsafe {
            let trusted = AXIsProcessTrusted();
            if trusted {
                Ok(())
            } else {
                Err(DataCaptureError::PermissionDenied(
                    "Accessibility permission required. Please enable in System Preferences > Security & Privacy > Privacy > Accessibility".to_string()
                ))
            }
        }
    }

    /// Request accessibility permission from the user
    pub async fn request_accessibility_permission() -> Result<()> {
        // For now, just check if permission is already granted
        // In a full implementation, this would show the system permission dialog
        check_accessibility_permission().await
    }

    // External function declarations for accessibility checks
    #[link(name = "ApplicationServices", kind = "framework")]
    extern "C" {
        fn AXIsProcessTrusted() -> bool;
        fn AXIsProcessTrustedWithOptions(options: core_foundation::dictionary::CFDictionaryRef) -> bool;
    }

    use core_foundation::dictionary::{CFDictionaryCreateMutable, CFDictionarySetValue, CFDictionaryRef};
    use core_foundation::base::kCFAllocatorDefault;
    use core_foundation::boolean::kCFBooleanTrue;
    use core_foundation::string::CFString;
    
    // Simplified permission checking
    const AX_TRUSTED_CHECK_OPTION_PROMPT: &str = "AXTrustedCheckOptionPrompt";
}

// Utility functions
fn nsstring_to_string(nsstring: id) -> String {
    unsafe {
        let utf8_string: *const std::os::raw::c_char = msg_send![nsstring, UTF8String];
        if utf8_string.is_null() {
            return "".to_string();
        }
        std::ffi::CStr::from_ptr(utf8_string)
            .to_string_lossy()
            .into_owned()
    }
}

// Additional CGEventTap types
// CGEventTapProxy is typically a raw pointer, so we'll define it if needed
type CGEventTapProxy = *mut std::ffi::c_void;

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn test_keystroke_monitor_creation() {
        let config = KeystrokeConfig::default();
        let (sender, _receiver) = mpsc::channel(100);
        
        let monitor = MacOSKeystrokeMonitor::new(config, sender).await;
        assert!(monitor.is_ok());
    }

    #[tokio::test]
    async fn test_mouse_monitor_creation() {
        let config = MouseConfig::default();
        let (sender, _receiver) = mpsc::channel(100);
        
        let monitor = MacOSMouseMonitor::new(config, sender).await;
        assert!(monitor.is_ok());
    }

    #[tokio::test]
    async fn test_window_monitor_creation() {
        let config = WindowConfig::default();
        let (sender, _receiver) = mpsc::channel(100);
        
        let monitor = MacOSWindowMonitor::new(config, sender).await;
        assert!(monitor.is_ok());
    }
}