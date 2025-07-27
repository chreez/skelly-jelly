//! Keystroke event monitoring

use async_trait::async_trait;
use tokio::sync::mpsc;
use tracing::{info, debug, error};

use crate::{
    monitors::{EventMonitor, MonitorStats},
    config::{DataCaptureConfig, KeystrokeConfig},
    error::{DataCaptureError, Result},
};
use skelly_jelly_storage::{RawEvent, KeystrokeEvent};

/// Generic keystroke monitor interface
pub struct KeystrokeMonitor {
    config: KeystrokeConfig,
    event_sender: mpsc::Sender<RawEvent>,
    running: bool,
    stats: MonitorStats,
}

impl KeystrokeMonitor {
    pub fn new(config: KeystrokeConfig, event_sender: mpsc::Sender<RawEvent>) -> Self {
        Self {
            config,
            event_sender,
            running: false,
            stats: MonitorStats::default(),
        }
    }
}

#[async_trait]
impl EventMonitor for KeystrokeMonitor {
    async fn start(&mut self) -> Result<()> {
        if self.running {
            return Err(DataCaptureError::AlreadyRunning);
        }
        
        info!("Starting keystroke monitor");
        self.running = true;
        
        // Platform-specific implementation will be injected here
        Ok(())
    }
    
    async fn stop(&mut self) -> Result<()> {
        if !self.running {
            return Ok(());
        }
        
        info!("Stopping keystroke monitor");
        self.running = false;
        Ok(())
    }
    
    fn is_running(&self) -> bool {
        self.running
    }
    
    fn name(&self) -> &'static str {
        "keystroke"
    }
    
    fn stats(&self) -> MonitorStats {
        self.stats.clone()
    }
    
    async fn update_config(&mut self, config: &DataCaptureConfig) -> Result<()> {
        self.config = config.monitors.keystroke.clone();
        Ok(())
    }
}