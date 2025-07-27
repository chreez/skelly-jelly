//! Integration tests for the Storage module

use skelly_jelly_storage::{StorageConfig, StorageModule};
use tempfile::TempDir;

#[tokio::test]
async fn test_storage_initialization() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create custom config with temp database
    let mut config = StorageConfig::default();
    config.database.path = temp_dir.path().join("test.db");
    config.database.pool_size = 1;
    
    
    // Initialize storage module
    let storage = StorageModule::new(config)
        .await
        .expect("Failed to initialize storage module");
    
    // Verify metrics are accessible
    let metrics = storage.metrics();
    assert_eq!(metrics.events_received.load(std::sync::atomic::Ordering::Relaxed), 0);
}

#[tokio::test]
async fn test_database_creation() {
    let temp_dir = TempDir::new().unwrap();
    
    let mut config = StorageConfig::default();
    config.database.path = temp_dir.path().join("test.db");
    
    let storage = StorageModule::new(config)
        .await
        .expect("Failed to initialize storage module");
    
    // Check database size  
    let db_size = storage.database().get_size().await.unwrap();
    assert!(db_size > 0, "Database should have been created");
}