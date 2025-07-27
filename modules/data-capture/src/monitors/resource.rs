//! Resource usage monitoring

use async_trait::async_trait;
use tokio::sync::mpsc;
use tracing::{info, debug, error};

use crate::{
    monitors::{EventMonitor, MonitorStats},
    config::{DataCaptureConfig, ResourceConfig},
    error::{DataCaptureError, Result},
};
use skelly_jelly_storage::{RawEvent, ResourceEvent};

/// Generic resource monitor interface
pub struct ResourceMonitor {
    config: ResourceConfig,
    event_sender: mpsc::Sender<RawEvent>,
    running: bool,
    stats: MonitorStats,
}

impl ResourceMonitor {
    pub fn new(config: ResourceConfig, event_sender: mpsc::Sender<RawEvent>) -> Self {
        Self {
            config,
            event_sender,
            running: false,
            stats: MonitorStats::default(),
        }
    }
}

#[async_trait]
impl EventMonitor for ResourceMonitor {
    async fn start(&mut self) -> Result<()> {
        if self.running {
            return Err(DataCaptureError::AlreadyRunning);
        }
        
        info!("Starting resource monitor");
        self.running = true;
        
        // Platform-specific implementation will be injected here
        Ok(())
    }
    
    async fn stop(&mut self) -> Result<()> {
        if !self.running {
            return Ok(());
        }
        
        info!("Stopping resource monitor");
        self.running = false;
        Ok(())
    }
    
    fn is_running(&self) -> bool {
        self.running
    }
    
    fn name(&self) -> &'static str {
        "resource"
    }
    
    fn stats(&self) -> MonitorStats {
        self.stats.clone()
    }
    
    async fn update_config(&mut self, config: &DataCaptureConfig) -> Result<()> {
        self.config = config.monitors.resource.clone();
        Ok(())
    }
}