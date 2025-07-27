//! Main storage module implementation

use crate::{
    config::StorageConfig,
    database::TimeSeriesDatabase,
    error::{Result, StorageError},
    metrics::PerformanceMetrics,
    types::*,
};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tracing::{error, info, warn};
use uuid::Uuid;

/// Main storage module that coordinates all storage operations
pub struct StorageModule {
    config: StorageConfig,
    database: Arc<TimeSeriesDatabase>,
    metrics: Arc<PerformanceMetrics>,
    event_receiver: mpsc::Receiver<BusMessage>,
    batch_sender: mpsc::Sender<BusMessage>,
    session_id: Uuid,
    shutdown_signal: Arc<Mutex<bool>>,
}

impl StorageModule {
    /// Create a new storage module instance
    pub async fn new(config: StorageConfig) -> Result<Self> {
        info!("Initializing Storage Module v{}", crate::VERSION);

        // Create database
        let database = Arc::new(TimeSeriesDatabase::new(config.database.clone()).await?);

        // Create metrics
        let metrics = Arc::new(PerformanceMetrics::new());

        // Create channels (these would normally connect to the Event Bus)
        let (event_sender, event_receiver) = mpsc::channel(config.performance.channel_capacity);
        let (batch_sender, batch_receiver) = mpsc::channel(100);

        // For now, drop the receivers we don't use
        drop(event_sender);
        drop(batch_receiver);

        let session_id = Uuid::new_v4();
        info!("Storage Module initialized with session {}", session_id);

        Ok(Self {
            config,
            database,
            metrics,
            event_receiver,
            batch_sender,
            session_id,
            shutdown_signal: Arc::new(Mutex::new(false)),
        })
    }

    /// Run the storage module
    pub async fn run(&mut self) -> Result<()> {
        info!("Storage Module starting...");

        // Spawn background tasks
        let metrics_handle = self.spawn_metrics_collector();
        let cleanup_handle = self.spawn_cleanup_task();

        // Main event processing loop
        loop {
            tokio::select! {
                // Receive events from Event Bus
                Some(msg) = self.event_receiver.recv() => {
                    if let Err(e) = self.handle_message(msg).await {
                        error!("Error handling message: {}", e);
                        if e.is_shutdown() {
                            break;
                        }
                    }
                }
                
                // Check shutdown signal
                _ = tokio::time::sleep(tokio::time::Duration::from_secs(1)) => {
                    if *self.shutdown_signal.lock().await {
                        info!("Shutdown signal received");
                        break;
                    }
                }
            }
        }

        // Wait for background tasks
        metrics_handle.abort();
        cleanup_handle.abort();

        info!("Storage Module stopped");
        Ok(())
    }

    /// Handle incoming bus messages
    async fn handle_message(&mut self, msg: BusMessage) -> Result<()> {
        match msg {
            BusMessage::RawEvent(event) => {
                self.handle_raw_event(event).await?;
            }
            BusMessage::Shutdown(reason) => {
                info!("Shutdown requested: {}", reason);
                *self.shutdown_signal.lock().await = true;
                return Err(StorageError::Shutdown(reason));
            }
            _ => {
                // Ignore messages not meant for us
            }
        }
        Ok(())
    }

    /// Handle a raw event
    async fn handle_raw_event(&self, event: RawEvent) -> Result<()> {
        let start = std::time::Instant::now();
        let event_type = event.event_type();

        // Record metrics
        self.metrics.record_event_received(event_type);

        // Store in database
        self.database.store_event(&self.session_id, &event).await?;

        // Record processing time
        self.metrics.record_event_latency(event_type, start.elapsed());

        Ok(())
    }

    /// Spawn metrics collection task
    fn spawn_metrics_collector(&self) -> tokio::task::JoinHandle<()> {
        let metrics = Arc::clone(&self.metrics);
        let database = Arc::clone(&self.database);
        let interval_secs = self.config.performance.metrics_interval_seconds;

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(interval_secs));
            
            loop {
                interval.tick().await;
                
                // Update CPU usage
                metrics.update_cpu_usage();
                
                // Update database size
                if let Ok(size) = database.get_size().await {
                    metrics.update_db_size(size);
                }
                
                // Log current metrics
                info!(
                    "Metrics: {} events/sec, {:.1} MB memory, {:.1}% CPU",
                    metrics.events_per_second() as u64,
                    metrics.memory_usage_mb(),
                    metrics.avg_cpu_usage()
                );
            }
        })
    }

    /// Spawn cleanup task for old data
    fn spawn_cleanup_task(&self) -> tokio::task::JoinHandle<()> {
        let database = Arc::clone(&self.database);
        let retention_days = self.config.retention.raw_events_days;
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_hours(24));
            
            loop {
                interval.tick().await;
                
                match database.cleanup_old_events(retention_days).await {
                    Ok(deleted) => {
                        if deleted > 0 {
                            info!("Cleaned up {} old events", deleted);
                        }
                    }
                    Err(e) => {
                        error!("Failed to cleanup old events: {}", e);
                    }
                }
                
                // Vacuum database
                if let Err(e) = database.vacuum().await {
                    error!("Failed to vacuum database: {}", e);
                }
            }
        })
    }

    /// Graceful shutdown
    pub async fn shutdown(&mut self) -> Result<()> {
        info!("Storage Module shutting down...");
        *self.shutdown_signal.lock().await = true;
        
        // Close database
        if let Ok(db) = Arc::try_unwrap(self.database.clone()) {
            db.close().await?;
        }
        
        Ok(())
    }

    /// Get current metrics
    pub fn metrics(&self) -> &PerformanceMetrics {
        &self.metrics
    }

    /// Get database handle
    pub fn database(&self) -> &TimeSeriesDatabase {
        &self.database
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn create_test_module() -> (StorageModule, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let mut config = StorageConfig::default();
        config.database.path = temp_dir.path().join("test.db");
        config.database.pool_size = 1;
        
        let module = StorageModule::new(config).await.unwrap();
        (module, temp_dir)
    }

    #[tokio::test]
    async fn test_module_creation() {
        let (_module, _temp_dir) = create_test_module().await;
    }

    #[tokio::test]
    async fn test_shutdown() {
        let (mut module, _temp_dir) = create_test_module().await;
        module.shutdown().await.unwrap();
    }
}