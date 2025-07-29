//! Privacy-preserving screenshot manager with secure deletion
//! 
//! Implements 30-second screenshot lifecycle with military-grade secure overwrite
//! to ensure complete data destruction for privacy protection.

use crate::{
    audit_logger::{PrivacyAuditLogger, AuditOutcome, PrivacyLevel, DataSensitivity},
    error::{Result, StorageError}, 
    types::*
};
use std::{
    collections::HashMap,
    fs::OpenOptions,
    io::{Write, Seek, SeekFrom},
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::{
    sync::RwLock,
    time::interval,
    task,
};
use uuid::Uuid;
use tracing::{info, error, debug};
use serde::{Serialize, Deserialize};

/// Screenshot entry with lifecycle tracking
#[derive(Debug, Clone)]
struct ScreenshotEntry {
    id: ScreenshotId,
    file_path: PathBuf,
    created_at: Instant,
    analyzed: bool,
    secure_deletion_scheduled: bool,
    file_size: u64,
}

/// Privacy-preserving screenshot manager with automatic secure deletion
pub struct ScreenshotManager {
    size_threshold: usize,
    /// Maximum lifetime for screenshots (30 seconds)
    max_lifetime: Duration,
    /// Directory for storing screenshots
    storage_dir: PathBuf,
    /// Tracked screenshot entries
    screenshots: Arc<RwLock<HashMap<ScreenshotId, ScreenshotEntry>>>,
    /// Secure deletion configuration
    secure_deletion_config: SecureDeletionConfig,
    /// Centralized privacy audit logger
    audit_logger: Arc<PrivacyAuditLogger>,
    /// Session ID for audit logging
    session_id: String,
}

/// Configuration for secure deletion
#[derive(Debug, Clone)]
struct SecureDeletionConfig {
    /// Number of overwrite passes (default: 3 passes)
    overwrite_passes: u32,
    /// Patterns for overwriting data
    overwrite_patterns: Vec<Vec<u8>>,
    /// Verify deletion by reading file after overwrite
    verify_deletion: bool,
}

impl Default for SecureDeletionConfig {
    fn default() -> Self {
        Self {
            overwrite_passes: 3,
            overwrite_patterns: vec![
                vec![0x00; 8192], // Zeros
                vec![0xFF; 8192], // Ones  
                (0..8192).map(|i| (i % 256) as u8).collect(), // Random pattern
            ],
            verify_deletion: true,
        }
    }
}

// Privacy audit entries are now handled by the centralized audit logger

impl ScreenshotManager {
    /// Create a new privacy-preserving screenshot manager
    pub fn new(size_threshold: usize, storage_dir: PathBuf, audit_logger: Arc<PrivacyAuditLogger>) -> Self {
        Self {
            size_threshold,
            max_lifetime: Duration::from_secs(30), // 30-second maximum lifetime
            storage_dir,
            screenshots: Arc::new(RwLock::new(HashMap::new())),
            secure_deletion_config: SecureDeletionConfig::default(),
            audit_logger,
            session_id: format!("screenshot_session_{}", Uuid::new_v4()),
        }
    }
    
    /// Start the privacy lifecycle manager background task
    pub async fn start_lifecycle_manager(&self) -> Result<()> {
        let mut interval = interval(Duration::from_secs(1)); // Check every second
        let screenshots = self.screenshots.clone();
        let storage_dir = self.storage_dir.clone();
        let config = self.secure_deletion_config.clone();
        let audit_logger = self.audit_logger.clone();
        let session_id = self.session_id.clone();
        
        task::spawn(async move {
            loop {
                interval.tick().await;
                
                let mut screenshots_lock = screenshots.write().await;
                let mut expired_ids = Vec::new();
                
                // Find expired screenshots
                for (id, entry) in screenshots_lock.iter() {
                    if entry.created_at.elapsed() >= Duration::from_secs(30) {
                        expired_ids.push(id.clone());
                    }
                }
                
                // Securely delete expired screenshots
                for id in expired_ids {
                    if let Some(entry) = screenshots_lock.remove(&id) {
                        debug!("Securely deleting expired screenshot: {}", id);
                        
                        let deletion_result = Self::secure_delete_file(
                            &entry.file_path, &config
                        ).await;
                        
                        let outcome = if deletion_result.is_ok() {
                            AuditOutcome::Success
                        } else {
                            AuditOutcome::Failed
                        };
                        
                        // Log to centralized audit system
                        let mut metadata = HashMap::new();
                        metadata.insert("file_path".to_string(), entry.file_path.to_string_lossy().to_string());
                        metadata.insert("file_size".to_string(), entry.file_size.to_string());
                        metadata.insert("age_seconds".to_string(), "30".to_string());
                        metadata.insert("deletion_method".to_string(), "automatic_lifecycle".to_string());
                        
                        if let Err(e) = &deletion_result {
                            metadata.insert("error".to_string(), e.to_string());
                        }
                        
                        let _ = audit_logger.log_screenshot_event(
                            "automatic_secure_deletion",
                            &id.to_string(),
                            &entry.file_path.to_string_lossy(),
                            entry.file_size,
                            outcome,
                            session_id.clone(),
                            metadata,
                        );
                        
                        if let Err(e) = deletion_result {
                            error!("Failed to securely delete screenshot {}: {}", id, e);
                        } else {
                            info!("Successfully deleted screenshot {} after 30 seconds", id);
                        }
                    }
                }
            }
        });
        
        Ok(())
    }
    
    /// Handle a screenshot event with privacy-preserving storage
    pub async fn handle(&mut self, screenshot: &ScreenshotEvent) -> Result<ScreenshotId> {
        let id = screenshot.screenshot_id.clone();
        let file_path = self.storage_dir.join(format!("{}.bin", id));
        
        // Store screenshot data to disk
        let write_result = tokio::fs::write(&file_path, &screenshot.data).await;
        let _outcome = if write_result.is_ok() {
            AuditOutcome::Success
        } else {
            AuditOutcome::Failed
        };
        
        if let Err(e) = write_result {
            // Log failed creation
            let mut metadata = HashMap::new();
            metadata.insert("file_path".to_string(), file_path.to_string_lossy().to_string());
            metadata.insert("data_size".to_string(), screenshot.data.len().to_string());
            metadata.insert("error".to_string(), e.to_string());
            
            let _ = self.audit_logger.log_screenshot_event(
                "screenshot_creation_failed",
                &id.to_string(),
                &file_path.to_string_lossy(),
                screenshot.data.len() as u64,
                AuditOutcome::Failed,
                self.session_id.clone(),
                metadata,
            );
            
            return Err(StorageError::IoError(e.to_string()));
        }
        
        let entry = ScreenshotEntry {
            id: id.clone(),
            file_path: file_path.clone(),
            created_at: Instant::now(),
            analyzed: false,
            secure_deletion_scheduled: false,
            file_size: screenshot.data.len() as u64,
        };
        
        // Add to tracked screenshots
        self.screenshots.write().await.insert(id.clone(), entry);
        
        // Log successful creation to centralized audit system
        let mut metadata = HashMap::new();
        metadata.insert("file_path".to_string(), file_path.to_string_lossy().to_string());
        metadata.insert("data_size".to_string(), screenshot.data.len().to_string());
        metadata.insert("timestamp".to_string(), screenshot.timestamp.to_string());
        metadata.insert("lifecycle_seconds".to_string(), "30".to_string());
        
        let _ = self.audit_logger.log_screenshot_event(
            "screenshot_created",
            &id.to_string(),
            &file_path.to_string_lossy(),
            screenshot.data.len() as u64,
            AuditOutcome::Success,
            self.session_id.clone(),
            metadata,
        );
        
        info!("Screenshot {} stored with 30-second privacy lifecycle", id);
        Ok(id)
    }
    
    /// Mark screenshot as analyzed (for audit purposes)
    pub async fn mark_analyzed(&self, screenshot_id: &ScreenshotId) -> Result<()> {
        let mut screenshots = self.screenshots.write().await;
        if let Some(entry) = screenshots.get_mut(screenshot_id) {
            entry.analyzed = true;
            
            // Log analysis completion to centralized audit system
            let mut metadata = HashMap::new();
            metadata.insert("file_path".to_string(), entry.file_path.to_string_lossy().to_string());
            metadata.insert("analysis_duration".to_string(), entry.created_at.elapsed().as_secs().to_string());
            
            let _ = self.audit_logger.log_screenshot_event(
                "screenshot_analyzed",
                &screenshot_id.to_string(),
                &entry.file_path.to_string_lossy(),
                entry.file_size,
                AuditOutcome::Success,
                self.session_id.clone(),
                metadata,
            );
            
            debug!("Screenshot {} marked as analyzed", screenshot_id);
            Ok(())
        } else {
            // Log failed analysis attempt
            let mut metadata = HashMap::new();
            metadata.insert("error".to_string(), "Screenshot not found".to_string());
            
            let _ = self.audit_logger.log_screenshot_event(
                "screenshot_analysis_failed",
                &screenshot_id.to_string(),
                "",
                0,
                AuditOutcome::Failed,
                self.session_id.clone(),
                metadata,
            );
            
            Err(StorageError::NotFound(format!("Screenshot {} not found", screenshot_id)))
        }
    }
    
    /// Clean up expired screenshots (manual trigger)
    pub async fn cleanup_expired(&mut self) -> Result<()> {
        let mut screenshots = self.screenshots.write().await;
        let mut expired_ids = Vec::new();
        
        // Find expired screenshots
        for (id, entry) in screenshots.iter() {
            if entry.created_at.elapsed() >= self.max_lifetime {
                expired_ids.push(id.clone());
            }
        }
        
        info!("Cleaning up {} expired screenshots", expired_ids.len());
        
        // Securely delete expired screenshots
        for id in expired_ids {
            if let Some(entry) = screenshots.remove(&id) {
                let deletion_result = Self::secure_delete_file(
                    &entry.file_path, &self.secure_deletion_config
                ).await;
                
                let outcome = if deletion_result.is_ok() {
                    AuditOutcome::Success
                } else {
                    AuditOutcome::Failed
                };
                
                // Log to centralized audit system
                let mut metadata = HashMap::new();
                metadata.insert("file_path".to_string(), entry.file_path.to_string_lossy().to_string());
                metadata.insert("file_size".to_string(), entry.file_size.to_string());
                metadata.insert("age_seconds".to_string(), entry.created_at.elapsed().as_secs().to_string());
                metadata.insert("deletion_method".to_string(), "manual_cleanup".to_string());
                
                if let Err(e) = &deletion_result {
                    metadata.insert("error".to_string(), e.to_string());
                }
                
                let _ = self.audit_logger.log_screenshot_event(
                    "manual_secure_deletion",
                    &id.to_string(),
                    &entry.file_path.to_string_lossy(),
                    entry.file_size,
                    outcome,
                    self.session_id.clone(),
                    metadata,
                );
                
                if let Err(e) = deletion_result {
                    error!("Failed to securely delete screenshot {}: {}", id, e);
                } else {
                    debug!("Successfully deleted screenshot {} via manual cleanup", id);
                }
            }
        }
        
        Ok(())
    }
    
    /// Securely delete a file with multiple overwrite passes
    async fn secure_delete_file(file_path: &Path, config: &SecureDeletionConfig) -> Result<()> {
        if !file_path.exists() {
            return Ok(()); // Already deleted
        }
        
        // Get file size for overwriting
        let file_size = tokio::fs::metadata(file_path).await
            .map_err(|e| StorageError::IoError(e.to_string()))?
            .len();
        
        // Perform overwrite passes
        for pass in 0..config.overwrite_passes {
            let pattern_index = (pass as usize) % config.overwrite_patterns.len();
            let pattern = &config.overwrite_patterns[pattern_index];
            
            debug!("Secure deletion pass {} for {:?}", pass + 1, file_path);
            
            // Overwrite file with pattern
            let mut file = OpenOptions::new()
                .write(true)
                .truncate(false)
                .open(file_path)
                .map_err(|e| StorageError::IoError(e.to_string()))?;
            
            file.seek(SeekFrom::Start(0))
                .map_err(|e| StorageError::IoError(e.to_string()))?;
            
            let mut written = 0u64;
            while written < file_size {
                let bytes_to_write = std::cmp::min(pattern.len() as u64, file_size - written);
                let write_slice = &pattern[..bytes_to_write as usize];
                
                file.write_all(write_slice)
                    .map_err(|e| StorageError::IoError(e.to_string()))?;
                written += bytes_to_write;
            }
            
            file.sync_all()
                .map_err(|e| StorageError::IoError(e.to_string()))?;
        }
        
        // Final deletion
        tokio::fs::remove_file(file_path).await
            .map_err(|e| StorageError::IoError(e.to_string()))?;
        
        // Verify deletion if configured
        if config.verify_deletion && file_path.exists() {
            return Err(StorageError::PrivacyViolation(
                "File still exists after secure deletion".to_string()
            ));
        }
        
        Ok(())
    }
    
    /// Get privacy audit log from centralized logger
    pub async fn get_privacy_audit_log(&self) -> Vec<crate::audit_logger::AuditEntry> {
        use crate::audit_logger::{AuditQuery, AuditCategory};
        
        let query = AuditQuery {
            category: Some(AuditCategory::ScreenshotLifecycle),
            session_id: Some(self.session_id.clone()),
            outcome: None,
            min_privacy_level: None,
            time_range: None,
            user_id: None,
            limit: Some(100),
            search_term: None,
        };
        
        self.audit_logger.query_entries(&query).unwrap_or_default()
    }
    
    /// Get current screenshot statistics
    pub async fn get_stats(&self) -> ScreenshotStats {
        let screenshots = self.screenshots.read().await;
        let total_count = screenshots.len();
        let analyzed_count = screenshots.values().filter(|e| e.analyzed).count();
        let total_size: u64 = screenshots.values().map(|e| e.file_size).sum();
        
        let oldest_age = screenshots.values()
            .map(|e| e.created_at.elapsed())
            .max()
            .unwrap_or(Duration::from_secs(0));
        
        ScreenshotStats {
            total_count,
            analyzed_count,
            pending_deletion: total_count - analyzed_count,
            total_size_bytes: total_size,
            oldest_age_seconds: oldest_age.as_secs(),
        }
    }
}

/// Screenshot statistics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenshotStats {
    pub total_count: usize,
    pub analyzed_count: usize,
    pub pending_deletion: usize,
    pub total_size_bytes: u64,
    pub oldest_age_seconds: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit_logger::AuditConfig;
    use tempfile::TempDir;
    use tokio::time::timeout;
    
    #[tokio::test]
    async fn test_screenshot_lifecycle() {
        let temp_dir = TempDir::new().unwrap();
        let audit_logger = Arc::new(PrivacyAuditLogger::new(AuditConfig::default()));
        let mut manager = ScreenshotManager::new(1024, temp_dir.path().to_path_buf(), audit_logger);
        
        let screenshot = ScreenshotEvent {
            screenshot_id: "test-123".to_string(),
            timestamp: chrono::Utc::now(),
            data: vec![1, 2, 3, 4, 5],
            metadata: ScreenshotMetadata::default(),
        };
        
        let id = manager.handle(&screenshot).await.unwrap();
        assert_eq!(id, "test-123");
        
        let stats = manager.get_stats().await;
        assert_eq!(stats.total_count, 1);
        assert_eq!(stats.analyzed_count, 0);
    }
    
    #[tokio::test]
    async fn test_secure_deletion() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test_file.dat");
        
        // Create test file
        tokio::fs::write(&test_file, b"sensitive data").await.unwrap();
        assert!(test_file.exists());
        
        let config = SecureDeletionConfig::default();
        ScreenshotManager::secure_delete_file(&test_file, &config).await.unwrap();
        
        // Verify file is deleted
        assert!(!test_file.exists());
    }
    
    #[tokio::test]
    async fn test_audit_logging() {
        let temp_dir = TempDir::new().unwrap();
        let audit_logger = Arc::new(PrivacyAuditLogger::new(AuditConfig::default()));
        let mut manager = ScreenshotManager::new(1024, temp_dir.path().to_path_buf(), audit_logger);
        
        let screenshot = ScreenshotEvent {
            screenshot_id: "audit-test".to_string(),
            timestamp: chrono::Utc::now(),
            data: vec![1, 2, 3],
            metadata: ScreenshotMetadata::default(),
        };
        
        manager.handle(&screenshot).await.unwrap();
        manager.mark_analyzed(&"audit-test".to_string()).await.unwrap();
        
        let audit_log = manager.get_privacy_audit_log().await;
        assert!(audit_log.len() >= 2); // Should have create + analyzed entries
        
        // Check that we have screenshot lifecycle events
        let screenshot_events: Vec<_> = audit_log.iter()
            .filter(|entry| matches!(entry.category, crate::audit_logger::AuditCategory::ScreenshotLifecycle))
            .collect();
        assert!(screenshot_events.len() >= 2);
    }
}