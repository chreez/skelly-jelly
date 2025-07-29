//! Comprehensive audit logging system for privacy operations
//! 
//! Provides centralized logging and tracking of all privacy-related activities
//! including screenshot lifecycle, PII detection, data access, and user actions.

use crate::error::{Result, StorageError};
use serde::{Serialize, Deserialize};
use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, RwLock},
    time::{SystemTime, UNIX_EPOCH},
};
use tracing::{debug, info, warn, error};

/// Centralized privacy audit logger
pub struct PrivacyAuditLogger {
    /// Audit log entries
    entries: Arc<RwLock<VecDeque<AuditEntry>>>,
    /// Configuration
    config: AuditConfig,
    /// Statistics tracking
    stats: Arc<RwLock<AuditStats>>,
    /// Real-time subscribers for audit events
    subscribers: Arc<RwLock<Vec<AuditSubscriber>>>,
}

/// Audit log entry with comprehensive metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    /// Unique entry identifier
    pub id: String,
    /// Timestamp (UTC)
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Operation category
    pub category: AuditCategory,
    /// Specific operation performed
    pub operation: String,
    /// User identifier (if applicable)
    pub user_id: Option<String>,
    /// Session identifier
    pub session_id: String,
    /// Resource affected
    pub resource: AuditResource,
    /// Operation outcome
    pub outcome: AuditOutcome,
    /// Privacy level of the operation
    pub privacy_level: PrivacyLevel,
    /// Data sensitivity classification
    pub data_sensitivity: DataSensitivity,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
    /// Compliance tags
    pub compliance_tags: Vec<String>,
}

/// Audit operation categories
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuditCategory {
    /// Screenshot lifecycle operations
    ScreenshotLifecycle,
    /// PII detection and masking
    PIIProcessing,
    /// Data encryption/decryption
    DataEncryption,
    /// User data access
    DataAccess,
    /// Data export operations
    DataExport,
    /// Data deletion operations
    DataDeletion,
    /// ML inference operations
    MLInference,
    /// User authentication
    Authentication,
    /// Configuration changes
    ConfigurationChange,
    /// Privacy settings modifications
    PrivacySettings,
}

/// Resource types affected by operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditResource {
    /// Screenshot file
    Screenshot {
        file_path: String,
        file_size: u64,
        screenshot_id: String,
    },
    /// User behavioral data
    BehavioralData {
        data_type: String,
        record_count: u64,
        time_range: String,
    },
    /// PII data
    PIIData {
        pii_type: String,
        detection_confidence: f32,
        masked_content_length: usize,
    },
    /// Encryption key
    EncryptionKey {
        key_id: String,
        algorithm: String,
        key_age_days: u64,
    },
    /// User profile data
    UserProfile {
        profile_id: String,
        data_fields: Vec<String>,
    },
    /// System configuration
    Configuration {
        config_type: String,
        setting_name: String,
    },
}

/// Operation outcome classification
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuditOutcome {
    /// Operation completed successfully
    Success,
    /// Operation failed
    Failed,
    /// Operation was blocked/denied
    Blocked,
    /// Operation completed with warnings
    Warning,
    /// Operation was unauthorized
    Unauthorized,
}

/// Privacy level classification
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum PrivacyLevel {
    /// No privacy implications
    None,
    /// Low privacy impact
    Low,
    /// Medium privacy impact
    Medium,
    /// High privacy impact
    High,
    /// Critical privacy impact
    Critical,
}

/// Data sensitivity classification
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum DataSensitivity {
    /// Public data
    Public,
    /// Internal data
    Internal,
    /// Confidential data
    Confidential,
    /// Restricted/PII data
    Restricted,
    /// Top secret/highly sensitive
    TopSecret,
}

/// Audit configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditConfig {
    /// Maximum number of entries to keep in memory
    pub max_entries: usize,
    /// Enable real-time audit notifications
    pub enable_notifications: bool,
    /// Minimum privacy level to log
    pub min_privacy_level: PrivacyLevel,
    /// Enable detailed metadata collection
    pub collect_detailed_metadata: bool,
    /// Retention period in days
    pub retention_days: u32,
    /// Enable compliance reporting
    pub enable_compliance_reporting: bool,
}

/// Audit statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditStats {
    /// Total entries logged
    pub total_entries: u64,
    /// Entries by category
    pub entries_by_category: HashMap<String, u64>,
    /// Entries by outcome
    pub entries_by_outcome: HashMap<String, u64>,
    /// Privacy violations detected
    pub privacy_violations: u64,
    /// Failed operations
    pub failed_operations: u64,
    /// Unauthorized access attempts
    pub unauthorized_attempts: u64,
    /// Data export operations
    pub data_exports: u64,
    /// Data deletion operations
    pub data_deletions: u64,
    /// PII detection events
    pub pii_detections: u64,
    /// Screenshot lifecycle events
    pub screenshot_events: u64,
}

/// Real-time audit event subscriber
pub type AuditSubscriber = Box<dyn Fn(&AuditEntry) + Send + Sync>;

/// Audit query parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditQuery {
    /// Filter by category
    pub category: Option<AuditCategory>,
    /// Filter by outcome
    pub outcome: Option<AuditOutcome>,
    /// Filter by privacy level (minimum)
    pub min_privacy_level: Option<PrivacyLevel>,
    /// Filter by time range
    pub time_range: Option<TimeRange>,
    /// Filter by user ID
    pub user_id: Option<String>,
    /// Filter by session ID
    pub session_id: Option<String>,
    /// Maximum number of results
    pub limit: Option<usize>,
    /// Search in operation or metadata
    pub search_term: Option<String>,
}

/// Time range for audit queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    pub start: chrono::DateTime<chrono::Utc>,
    pub end: chrono::DateTime<chrono::Utc>,
}

/// Compliance report summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceReport {
    /// Report generation timestamp
    pub generated_at: chrono::DateTime<chrono::Utc>,
    /// Report period
    pub period: TimeRange,
    /// Total privacy operations
    pub total_operations: u64,
    /// Privacy compliance score (0-100)
    pub compliance_score: f32,
    /// Operations by category
    pub operations_by_category: HashMap<String, u64>,
    /// Privacy violations summary
    pub violations_summary: Vec<ViolationSummary>,
    /// Data handling metrics
    pub data_handling_metrics: DataHandlingMetrics,
    /// Recommendations
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViolationSummary {
    pub violation_type: String,
    pub count: u64,
    pub severity: PrivacyLevel,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataHandlingMetrics {
    pub data_created: u64,
    pub data_accessed: u64,
    pub data_modified: u64,
    pub data_deleted: u64,
    pub data_exported: u64,
    pub pii_detected: u64,
    pub pii_masked: u64,
    pub screenshots_created: u64,
    pub screenshots_deleted: u64,
    pub encryption_operations: u64,
}

impl PrivacyAuditLogger {
    /// Create new privacy audit logger
    pub fn new(config: AuditConfig) -> Self {
        Self {
            entries: Arc::new(RwLock::new(VecDeque::new())),
            config,
            stats: Arc::new(RwLock::new(AuditStats::new())),
            subscribers: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Log a privacy operation
    pub fn log_operation(
        &self,
        category: AuditCategory,
        operation: &str,
        resource: AuditResource,
        outcome: AuditOutcome,
        privacy_level: PrivacyLevel,
        data_sensitivity: DataSensitivity,
        user_id: Option<String>,
        session_id: String,
        metadata: HashMap<String, String>,
    ) -> Result<String> {
        // Check if operation meets minimum privacy level
        if privacy_level < self.config.min_privacy_level {
            return Ok(String::new()); // Skip logging
        }

        let entry = AuditEntry {
            id: self.generate_entry_id(),
            timestamp: chrono::Utc::now(),
            category,
            operation: operation.to_string(),
            user_id,
            session_id,
            resource,
            outcome,
            privacy_level,
            data_sensitivity,
            metadata,
            compliance_tags: self.generate_compliance_tags(category, privacy_level),
        };

        let entry_id = entry.id.clone();

        // Add to entries
        if let Ok(mut entries) = self.entries.write() {
            entries.push_back(entry.clone());

            // Enforce max entries limit
            if entries.len() > self.config.max_entries {
                entries.pop_front();
            }
        }

        // Update statistics
        self.update_stats(&entry)?;

        // Notify subscribers
        if self.config.enable_notifications {
            self.notify_subscribers(&entry);
        }

        // Log to tracing system
        match entry.privacy_level {
            PrivacyLevel::Critical => error!("Critical privacy operation: {} - {}", category, operation),
            PrivacyLevel::High => warn!("High privacy operation: {} - {}", category, operation),
            PrivacyLevel::Medium => info!("Medium privacy operation: {} - {}", category, operation),
            _ => debug!("Privacy operation: {} - {}", category, operation),
        }

        Ok(entry_id)
    }

    /// Log screenshot lifecycle event
    pub fn log_screenshot_event(
        &self,
        operation: &str,
        screenshot_id: &str,
        file_path: &str,
        file_size: u64,
        outcome: AuditOutcome,
        session_id: String,
        metadata: HashMap<String, String>,
    ) -> Result<String> {
        let resource = AuditResource::Screenshot {
            file_path: file_path.to_string(),
            file_size,
            screenshot_id: screenshot_id.to_string(),
        };

        self.log_operation(
            AuditCategory::ScreenshotLifecycle,
            operation,
            resource,
            outcome,
            PrivacyLevel::High, // Screenshots are high privacy
            DataSensitivity::Restricted,
            None,
            session_id,
            metadata,
        )
    }

    /// Log PII detection event
    pub fn log_pii_detection(
        &self,
        pii_type: &str,
        detection_confidence: f32,
        masked_content_length: usize,
        outcome: AuditOutcome,
        session_id: String,
        metadata: HashMap<String, String>,
    ) -> Result<String> {
        let resource = AuditResource::PIIData {
            pii_type: pii_type.to_string(),
            detection_confidence,
            masked_content_length,
        };

        self.log_operation(
            AuditCategory::PIIProcessing,
            "pii_detection",
            resource,
            outcome,
            PrivacyLevel::Critical, // PII is critical privacy
            DataSensitivity::TopSecret,
            None,
            session_id,
            metadata,
        )
    }

    /// Log data access event
    pub fn log_data_access(
        &self,
        data_type: &str,
        record_count: u64,
        time_range: &str,
        user_id: Option<String>,
        session_id: String,
        metadata: HashMap<String, String>,
    ) -> Result<String> {
        let resource = AuditResource::BehavioralData {
            data_type: data_type.to_string(),
            record_count,
            time_range: time_range.to_string(),
        };

        self.log_operation(
            AuditCategory::DataAccess,
            "data_access",
            resource,
            AuditOutcome::Success,
            PrivacyLevel::Medium,
            DataSensitivity::Confidential,
            user_id,
            session_id,
            metadata,
        )
    }

    /// Log encryption operation
    pub fn log_encryption_operation(
        &self,
        operation: &str,
        key_id: &str,
        algorithm: &str,
        key_age_days: u64,
        outcome: AuditOutcome,
        session_id: String,
        metadata: HashMap<String, String>,
    ) -> Result<String> {
        let resource = AuditResource::EncryptionKey {
            key_id: key_id.to_string(),
            algorithm: algorithm.to_string(),
            key_age_days,
        };

        self.log_operation(
            AuditCategory::DataEncryption,
            operation,
            resource,
            outcome,
            PrivacyLevel::High,
            DataSensitivity::Restricted,
            None,
            session_id,
            metadata,
        )
    }

    /// Query audit log entries
    pub fn query_entries(&self, query: &AuditQuery) -> Result<Vec<AuditEntry>> {
        let entries = self.entries.read().map_err(|_| {
            StorageError::Other("Failed to read audit entries".to_string())
        })?;

        let mut results: Vec<AuditEntry> = entries
            .iter()
            .filter(|entry| self.matches_query(entry, query))
            .cloned()
            .collect();

        // Sort by timestamp (newest first)
        results.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        // Apply limit
        if let Some(limit) = query.limit {
            results.truncate(limit);
        }

        Ok(results)
    }

    /// Get audit statistics
    pub fn get_statistics(&self) -> Result<AuditStats> {
        self.stats.read()
            .map(|stats| stats.clone())
            .map_err(|_| StorageError::Other("Failed to read audit statistics".to_string()))
    }

    /// Generate compliance report
    pub fn generate_compliance_report(&self, period: TimeRange) -> Result<ComplianceReport> {
        let query = AuditQuery {
            time_range: Some(period.clone()),
            category: None,
            outcome: None,
            min_privacy_level: None,
            user_id: None,
            session_id: None,
            limit: None,
            search_term: None,
        };

        let entries = self.query_entries(&query)?;
        let total_operations = entries.len() as u64;

        // Calculate compliance score
        let compliance_score = self.calculate_compliance_score(&entries);

        // Group operations by category
        let mut operations_by_category = HashMap::new();
        for entry in &entries {
            let category_name = format!("{:?}", entry.category);
            *operations_by_category.entry(category_name).or_insert(0) += 1;
        }

        // Analyze violations
        let violations_summary = self.analyze_violations(&entries);

        // Calculate data handling metrics
        let data_handling_metrics = self.calculate_data_handling_metrics(&entries);

        // Generate recommendations
        let recommendations = self.generate_recommendations(&entries, compliance_score);

        Ok(ComplianceReport {
            generated_at: chrono::Utc::now(),
            period,
            total_operations,
            compliance_score,
            operations_by_category,
            violations_summary,
            data_handling_metrics,
            recommendations,
        })
    }

    /// Subscribe to real-time audit events
    pub fn subscribe(&self, subscriber: AuditSubscriber) -> Result<()> {
        self.subscribers.write()
            .map_err(|_| StorageError::Other("Failed to add subscriber".to_string()))?
            .push(subscriber);
        Ok(())
    }

    /// Export audit log for external analysis
    pub fn export_audit_log(&self, format: ExportFormat) -> Result<String> {
        let entries = self.entries.read().map_err(|_| {
            StorageError::Other("Failed to read audit entries".to_string())
        })?;

        match format {
            ExportFormat::Json => {
                serde_json::to_string_pretty(&*entries)
                    .map_err(|e| StorageError::Other(format!("JSON export failed: {}", e)))
            }
            ExportFormat::Csv => {
                let mut csv_content = String::from(
                    "timestamp,category,operation,outcome,privacy_level,data_sensitivity,user_id,session_id\n"
                );
                
                for entry in entries.iter() {
                    csv_content.push_str(&format!(
                        "{},{:?},{},{:?},{:?},{:?},{},{}\n",
                        entry.timestamp.format("%Y-%m-%d %H:%M:%S UTC"),
                        entry.category,
                        entry.operation,
                        entry.outcome,
                        entry.privacy_level,
                        entry.data_sensitivity,
                        entry.user_id.as_deref().unwrap_or(""),
                        entry.session_id
                    ));
                }
                
                Ok(csv_content)
            }
        }
    }

    /// Generate unique entry ID
    fn generate_entry_id(&self) -> String {
        format!("audit_{}", uuid::Uuid::new_v4().to_string())
    }

    /// Generate compliance tags based on operation
    fn generate_compliance_tags(&self, category: AuditCategory, privacy_level: PrivacyLevel) -> Vec<String> {
        let mut tags = Vec::new();

        // Add privacy level tag
        tags.push(format!("privacy_{:?}", privacy_level).to_lowercase());

        // Add category-specific tags
        match category {
            AuditCategory::PIIProcessing => {
                tags.push("gdpr_article_6".to_string());
                tags.push("ccpa_compliance".to_string());
            }
            AuditCategory::DataDeletion => {
                tags.push("right_to_erasure".to_string());
                tags.push("gdpr_article_17".to_string());
            }
            AuditCategory::DataExport => {
                tags.push("data_portability".to_string());
                tags.push("gdpr_article_20".to_string());
            }
            AuditCategory::DataAccess => {
                tags.push("right_to_access".to_string());
                tags.push("gdpr_article_15".to_string());
            }
            _ => {}
        }

        // Add high-risk tag for critical operations
        if privacy_level >= PrivacyLevel::High {
            tags.push("high_risk_operation".to_string());
        }

        tags
    }

    /// Update audit statistics
    fn update_stats(&self, entry: &AuditEntry) -> Result<()> {
        if let Ok(mut stats) = self.stats.write() {
            stats.total_entries += 1;

            // Update category counts
            let category_name = format!("{:?}", entry.category);
            *stats.entries_by_category.entry(category_name).or_insert(0) += 1;

            // Update outcome counts
            let outcome_name = format!("{:?}", entry.outcome);
            *stats.entries_by_outcome.entry(outcome_name).or_insert(0) += 1;

            // Update specific counters
            match entry.outcome {
                AuditOutcome::Failed => stats.failed_operations += 1,
                AuditOutcome::Unauthorized => stats.unauthorized_attempts += 1,
                _ => {}
            }

            match entry.category {
                AuditCategory::PIIProcessing => stats.pii_detections += 1,
                AuditCategory::DataExport => stats.data_exports += 1,
                AuditCategory::DataDeletion => stats.data_deletions += 1,
                AuditCategory::ScreenshotLifecycle => stats.screenshot_events += 1,
                _ => {}
            }

            // Check for privacy violations
            if entry.privacy_level >= PrivacyLevel::High && entry.outcome != AuditOutcome::Success {
                stats.privacy_violations += 1;
            }
        }

        Ok(())
    }

    /// Notify subscribers of new audit entry
    fn notify_subscribers(&self, entry: &AuditEntry) {
        if let Ok(subscribers) = self.subscribers.read() {
            for subscriber in subscribers.iter() {
                subscriber(entry);
            }
        }
    }

    /// Check if entry matches query criteria
    fn matches_query(&self, entry: &AuditEntry, query: &AuditQuery) -> bool {
        // Category filter
        if let Some(category) = &query.category {
            if entry.category != *category {
                return false;
            }
        }

        // Outcome filter
        if let Some(outcome) = &query.outcome {
            if entry.outcome != *outcome {
                return false;
            }
        }

        // Privacy level filter
        if let Some(min_privacy_level) = &query.min_privacy_level {
            if entry.privacy_level < *min_privacy_level {
                return false;
            }
        }

        // Time range filter
        if let Some(time_range) = &query.time_range {
            if entry.timestamp < time_range.start || entry.timestamp > time_range.end {
                return false;
            }
        }

        // User ID filter
        if let Some(user_id) = &query.user_id {
            if entry.user_id.as_ref() != Some(user_id) {
                return false;
            }
        }

        // Session ID filter
        if let Some(session_id) = &query.session_id {
            if entry.session_id != *session_id {
                return false;
            }
        }

        // Search term filter
        if let Some(search_term) = &query.search_term {
            let search_lower = search_term.to_lowercase();
            if !entry.operation.to_lowercase().contains(&search_lower) {
                // Check metadata for search term
                let metadata_match = entry.metadata.values()
                    .any(|value| value.to_lowercase().contains(&search_lower));
                if !metadata_match {
                    return false;
                }
            }
        }

        true
    }

    /// Calculate compliance score based on audit entries
    fn calculate_compliance_score(&self, entries: &[AuditEntry]) -> f32 {
        if entries.is_empty() {
            return 100.0;
        }

        let total_operations = entries.len() as f32;
        let failed_operations = entries.iter()
            .filter(|e| e.outcome == AuditOutcome::Failed || e.outcome == AuditOutcome::Unauthorized)
            .count() as f32;

        let success_rate = (total_operations - failed_operations) / total_operations;
        
        // Base score from success rate
        let mut score = success_rate * 80.0;

        // Bonus for high privacy operations handled correctly
        let high_privacy_ops = entries.iter()
            .filter(|e| e.privacy_level >= PrivacyLevel::High)
            .count() as f32;

        let high_privacy_success = entries.iter()
            .filter(|e| e.privacy_level >= PrivacyLevel::High && e.outcome == AuditOutcome::Success)
            .count() as f32;

        if high_privacy_ops > 0.0 {
            let high_privacy_rate = high_privacy_success / high_privacy_ops;
            score += high_privacy_rate * 20.0;
        } else {
            score += 20.0; // No high-risk operations is good
        }

        score.min(100.0).max(0.0)
    }

    /// Analyze privacy violations in audit entries
    fn analyze_violations(&self, entries: &[AuditEntry]) -> Vec<ViolationSummary> {
        let mut violations = HashMap::new();

        for entry in entries {
            if entry.outcome != AuditOutcome::Success && entry.privacy_level >= PrivacyLevel::Medium {
                let violation_type = format!("{:?}_{:?}", entry.category, entry.outcome);
                let summary = violations.entry(violation_type.clone()).or_insert(ViolationSummary {
                    violation_type,
                    count: 0,
                    severity: entry.privacy_level,
                    description: format!("Failed {} operations", entry.category),
                });
                summary.count += 1;
                summary.severity = summary.severity.max(entry.privacy_level);
            }
        }

        let mut result: Vec<_> = violations.into_values().collect();
        result.sort_by(|a, b| b.severity.cmp(&a.severity).then_with(|| b.count.cmp(&a.count)));
        result
    }

    /// Calculate data handling metrics
    fn calculate_data_handling_metrics(&self, entries: &[AuditEntry]) -> DataHandlingMetrics {
        let mut metrics = DataHandlingMetrics {
            data_created: 0,
            data_accessed: 0,
            data_modified: 0,
            data_deleted: 0,
            data_exported: 0,
            pii_detected: 0,
            pii_masked: 0,
            screenshots_created: 0,
            screenshots_deleted: 0,
            encryption_operations: 0,
        };

        for entry in entries {
            match entry.category {
                AuditCategory::DataAccess => metrics.data_accessed += 1,
                AuditCategory::DataExport => metrics.data_exported += 1,
                AuditCategory::DataDeletion => metrics.data_deleted += 1,
                AuditCategory::PIIProcessing => {
                    metrics.pii_detected += 1;
                    if entry.operation.contains("mask") {
                        metrics.pii_masked += 1;
                    }
                }
                AuditCategory::ScreenshotLifecycle => {
                    if entry.operation.contains("create") {
                        metrics.screenshots_created += 1;
                    } else if entry.operation.contains("delete") {
                        metrics.screenshots_deleted += 1;
                    }
                }
                AuditCategory::DataEncryption => metrics.encryption_operations += 1,
                _ => {}
            }
        }

        metrics
    }

    /// Generate recommendations based on audit analysis
    fn generate_recommendations(&self, entries: &[AuditEntry], compliance_score: f32) -> Vec<String> {
        let mut recommendations = Vec::new();

        if compliance_score < 80.0 {
            recommendations.push("Investigate and address failed privacy operations to improve compliance score".to_string());
        }

        let failed_count = entries.iter()
            .filter(|e| e.outcome == AuditOutcome::Failed)
            .count();

        if failed_count > 5 {
            recommendations.push("High number of failed operations detected. Review system reliability and error handling".to_string());
        }

        let unauthorized_count = entries.iter()
            .filter(|e| e.outcome == AuditOutcome::Unauthorized)
            .count();

        if unauthorized_count > 0 {
            recommendations.push("Unauthorized access attempts detected. Review access controls and authentication mechanisms".to_string());
        }

        let high_privacy_ops = entries.iter()
            .filter(|e| e.privacy_level >= PrivacyLevel::High)
            .count();

        if high_privacy_ops == 0 {
            recommendations.push("No high-privacy operations detected. Verify that privacy classification is working correctly".to_string());
        }

        if recommendations.is_empty() {
            recommendations.push("System is operating within acceptable privacy and compliance parameters".to_string());
        }

        recommendations
    }
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            max_entries: 10000,
            enable_notifications: true,
            min_privacy_level: PrivacyLevel::Low,
            collect_detailed_metadata: true,
            retention_days: 90,
            enable_compliance_reporting: true,
        }
    }
}

impl AuditStats {
    fn new() -> Self {
        Self {
            total_entries: 0,
            entries_by_category: HashMap::new(),
            entries_by_outcome: HashMap::new(),
            privacy_violations: 0,
            failed_operations: 0,
            unauthorized_attempts: 0,
            data_exports: 0,
            data_deletions: 0,
            pii_detections: 0,
            screenshot_events: 0,
        }
    }
}

/// Export format for audit logs
#[derive(Debug, Clone, Copy)]
pub enum ExportFormat {
    Json,
    Csv,
}

impl std::fmt::Display for AuditCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_logger_creation() {
        let config = AuditConfig::default();
        let logger = PrivacyAuditLogger::new(config);
        
        let stats = logger.get_statistics().unwrap();
        assert_eq!(stats.total_entries, 0);
    }

    #[test]
    fn test_log_screenshot_event() {
        let config = AuditConfig::default();
        let logger = PrivacyAuditLogger::new(config);

        let entry_id = logger.log_screenshot_event(
            "delete_screenshot",
            "screenshot_123",
            "/tmp/screenshot_123.png",
            1024,
            AuditOutcome::Success,
            "session_456".to_string(),
            HashMap::new(),
        ).unwrap();

        assert!(!entry_id.is_empty());

        let stats = logger.get_statistics().unwrap();
        assert_eq!(stats.total_entries, 1);
        assert_eq!(stats.screenshot_events, 1);
    }

    #[test]
    fn test_log_pii_detection() {
        let config = AuditConfig::default();
        let logger = PrivacyAuditLogger::new(config);

        let entry_id = logger.log_pii_detection(
            "credit_card",
            0.95,
            16,
            AuditOutcome::Success,
            "session_789".to_string(),
            HashMap::new(),
        ).unwrap();

        assert!(!entry_id.is_empty());

        let stats = logger.get_statistics().unwrap();
        assert_eq!(stats.total_entries, 1);
        assert_eq!(stats.pii_detections, 1);
    }

    #[test]
    fn test_audit_query() {
        let config = AuditConfig::default();
        let logger = PrivacyAuditLogger::new(config);

        // Log some test entries
        logger.log_screenshot_event(
            "create_screenshot",
            "screenshot_1",
            "/tmp/screenshot_1.png",
            512,
            AuditOutcome::Success,
            "session_1".to_string(),
            HashMap::new(),
        ).unwrap();

        logger.log_pii_detection(
            "ssn",
            0.98,
            9,
            AuditOutcome::Success,
            "session_1".to_string(),
            HashMap::new(),
        ).unwrap();

        // Query for screenshot events
        let query = AuditQuery {
            category: Some(AuditCategory::ScreenshotLifecycle),
            outcome: None,
            min_privacy_level: None,
            time_range: None,
            user_id: None,
            session_id: None,
            limit: None,
            search_term: None,
        };

        let results = logger.query_entries(&query).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].category, AuditCategory::ScreenshotLifecycle);
    }

    #[test]
    fn test_compliance_report() {
        let config = AuditConfig::default();
        let logger = PrivacyAuditLogger::new(config);

        // Log various operations
        logger.log_screenshot_event(
            "delete_screenshot",
            "screenshot_1",
            "/tmp/screenshot_1.png",
            1024,
            AuditOutcome::Success,
            "session_1".to_string(),
            HashMap::new(),
        ).unwrap();

        logger.log_pii_detection(
            "email",
            0.99,
            25,
            AuditOutcome::Success,
            "session_1".to_string(),
            HashMap::new(),
        ).unwrap();

        let time_range = TimeRange {
            start: chrono::Utc::now() - chrono::Duration::hours(1),
            end: chrono::Utc::now() + chrono::Duration::hours(1),
        };

        let report = logger.generate_compliance_report(time_range).unwrap();
        assert_eq!(report.total_operations, 2);
        assert!(report.compliance_score > 90.0);
        assert!(report.operations_by_category.contains_key("ScreenshotLifecycle"));
        assert!(report.operations_by_category.contains_key("PIIProcessing"));
    }

    #[test]
    fn test_export_audit_log() {
        let config = AuditConfig::default();
        let logger = PrivacyAuditLogger::new(config);

        logger.log_screenshot_event(
            "create_screenshot",
            "screenshot_1",
            "/tmp/screenshot_1.png",
            512,
            AuditOutcome::Success,
            "session_1".to_string(),
            HashMap::new(),
        ).unwrap();

        // Test JSON export
        let json_export = logger.export_audit_log(ExportFormat::Json).unwrap();
        assert!(json_export.contains("ScreenshotLifecycle"));
        assert!(json_export.contains("create_screenshot"));

        // Test CSV export
        let csv_export = logger.export_audit_log(ExportFormat::Csv).unwrap();
        assert!(csv_export.contains("timestamp,category,operation"));
        assert!(csv_export.contains("ScreenshotLifecycle"));
    }
}