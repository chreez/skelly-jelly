//! Window focus monitoring

use async_trait::async_trait;
use tokio::sync::mpsc;
use tracing::{info, debug, error};

use crate::{
    monitors::{EventMonitor, MonitorStats},
    config::{DataCaptureConfig, WindowConfig},
    error::{DataCaptureError, Result},
};
use skelly_jelly_storage::{RawEvent, WindowFocusEvent};

/// Generic window monitor interface
pub struct WindowMonitor {
    config: WindowConfig,
    event_sender: mpsc::Sender<RawEvent>,
    running: bool,
    stats: MonitorStats,
}

impl WindowMonitor {
    pub fn new(config: WindowConfig, event_sender: mpsc::Sender<RawEvent>) -> Self {
        Self {
            config,
            event_sender,
            running: false,
            stats: MonitorStats::default(),
        }
    }
}

#[async_trait]
impl EventMonitor for WindowMonitor {
    async fn start(&mut self) -> Result<()> {
        if self.running {
            return Err(DataCaptureError::AlreadyRunning);
        }
        
        info!("Starting window monitor");
        self.running = true;
        
        // Platform-specific implementation will be injected here
        Ok(())
    }
    
    async fn stop(&mut self) -> Result<()> {
        if !self.running {
            return Ok(());
        }
        
        info!("Stopping window monitor");
        self.running = false;
        Ok(())
    }
    
    fn is_running(&self) -> bool {
        self.running
    }
    
    fn name(&self) -> &'static str {
        "window"
    }
    
    fn stats(&self) -> MonitorStats {
        self.stats.clone()
    }
    
    async fn update_config(&mut self, config: &DataCaptureConfig) -> Result<()> {
        self.config = config.monitors.window.clone();
        Ok(())
    }
}