//! Mouse event monitoring

use async_trait::async_trait;
use tokio::sync::mpsc;
use tracing::{info, debug, error};

use crate::{
    monitors::{EventMonitor, MonitorStats},
    config::{DataCaptureConfig, MouseConfig},
    error::{DataCaptureError, Result},
};
use skelly_jelly_storage::{RawEvent, MouseMoveEvent, MouseClickEvent};

/// Generic mouse monitor interface
pub struct MouseMonitor {
    config: MouseConfig,
    event_sender: mpsc::Sender<RawEvent>,
    running: bool,
    stats: MonitorStats,
}

impl MouseMonitor {
    pub fn new(config: MouseConfig, event_sender: mpsc::Sender<RawEvent>) -> Self {
        Self {
            config,
            event_sender,
            running: false,
            stats: MonitorStats::default(),
        }
    }
}

#[async_trait]
impl EventMonitor for MouseMonitor {
    async fn start(&mut self) -> Result<()> {
        if self.running {
            return Err(DataCaptureError::AlreadyRunning);
        }
        
        info!("Starting mouse monitor");
        self.running = true;
        
        // Platform-specific implementation will be injected here
        Ok(())
    }
    
    async fn stop(&mut self) -> Result<()> {
        if !self.running {
            return Ok(());
        }
        
        info!("Stopping mouse monitor");
        self.running = false;
        Ok(())
    }
    
    fn is_running(&self) -> bool {
        self.running
    }
    
    fn name(&self) -> &'static str {
        "mouse"
    }
    
    fn stats(&self) -> MonitorStats {
        self.stats.clone()
    }
    
    async fn update_config(&mut self, config: &DataCaptureConfig) -> Result<()> {
        self.config = config.monitors.mouse.clone();
        Ok(())
    }
}