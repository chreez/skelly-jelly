//! Configuration for the Storage module

use config::{Config, ConfigError, Environment, File};
use dirs::home_dir;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

/// Storage module configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Batching configuration
    #[serde(default)]
    pub batching: BatchingConfig,

    /// Screenshot management configuration
    #[serde(default)]
    pub screenshot: ScreenshotConfig,

    /// Database configuration
    #[serde(default)]
    pub database: DatabaseConfig,

    /// Performance configuration
    #[serde(default)]
    pub performance: PerformanceConfig,

    /// Retention policy configuration
    #[serde(default)]
    pub retention: RetentionConfig,

    /// Development mode settings
    #[serde(default)]
    pub dev_mode: DevModeConfig,
}

/// Batching configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchingConfig {
    /// Window duration in seconds
    #[serde(default = "default_batch_window_seconds")]
    pub window_seconds: u64,

    /// Maximum events per batch
    #[serde(default = "default_max_events_per_batch")]
    pub max_events_per_batch: usize,

    /// Batch buffer capacity
    #[serde(default = "default_batch_buffer_capacity")]
    pub buffer_capacity: usize,
}

/// Screenshot configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenshotConfig {
    /// Memory threshold in MB
    #[serde(default = "default_memory_threshold_mb")]
    pub memory_threshold_mb: usize,

    /// Retention time in seconds
    #[serde(default = "default_retention_seconds")]
    pub retention_seconds: u64,

    /// Memory cache size
    #[serde(default = "default_memory_cache_size")]
    pub memory_cache_size: usize,

    /// Temporary directory path
    #[serde(default = "default_temp_dir")]
    pub temp_dir: PathBuf,
}

/// Database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Database file path
    #[serde(default = "default_database_path")]
    pub path: PathBuf,

    /// Connection pool size
    #[serde(default = "default_pool_size")]
    pub pool_size: u32,

    /// Write buffer size in MB
    #[serde(default = "default_write_buffer_size_mb")]
    pub write_buffer_size_mb: usize,

    /// Compaction interval in hours
    #[serde(default = "default_compaction_interval_hours")]
    pub compaction_interval_hours: u64,

    /// Enable Write-Ahead Logging
    #[serde(default = "default_wal_enabled")]
    pub wal_enabled: bool,

    /// Synchronous mode
    #[serde(default = "default_synchronous_mode")]
    pub synchronous_mode: String,
}

/// Performance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Maximum memory usage in MB
    #[serde(default = "default_max_memory_mb")]
    pub max_memory_mb: usize,

    /// Target CPU usage percentage
    #[serde(default = "default_target_cpu_percent")]
    pub target_cpu_percent: f32,

    /// Event channel capacity
    #[serde(default = "default_channel_capacity")]
    pub channel_capacity: usize,

    /// Enable compression
    #[serde(default = "default_compression_enabled")]
    pub compression_enabled: bool,

    /// Metrics collection interval in seconds
    #[serde(default = "default_metrics_interval_seconds")]
    pub metrics_interval_seconds: u64,
}

/// Retention policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionConfig {
    /// Raw events retention in days
    #[serde(default = "default_raw_events_days")]
    pub raw_events_days: u32,

    /// Hourly aggregates retention in days
    #[serde(default = "default_hourly_aggregates_days")]
    pub hourly_aggregates_days: u32,

    /// Daily summaries retention in days
    #[serde(default = "default_daily_summaries_days")]
    pub daily_summaries_days: u32,
}

/// Development mode configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevModeConfig {
    /// Enable development mode
    #[serde(default)]
    pub enabled: bool,

    /// Number of screenshots to keep in dev mode
    #[serde(default = "default_dev_screenshot_count")]
    pub screenshot_count: usize,

    /// Enable verbose logging
    #[serde(default)]
    pub verbose_logging: bool,

    /// Enable debug endpoints
    #[serde(default)]
    pub debug_endpoints: bool,
}

// Default value functions
fn default_batch_window_seconds() -> u64 { 30 }
fn default_max_events_per_batch() -> usize { 10_000 }
fn default_batch_buffer_capacity() -> usize { 1_000 }
fn default_memory_threshold_mb() -> usize { 5 }
fn default_retention_seconds() -> u64 { 30 }
fn default_memory_cache_size() -> usize { 50 }
fn default_temp_dir() -> PathBuf {
    home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".skelly-jelly")
        .join("tmp")
}
fn default_database_path() -> PathBuf {
    home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".skelly-jelly")
        .join("events.db")
}
fn default_pool_size() -> u32 { 4 }
fn default_write_buffer_size_mb() -> usize { 10 }
fn default_compaction_interval_hours() -> u64 { 24 }
fn default_wal_enabled() -> bool { true }
fn default_synchronous_mode() -> String { "NORMAL".to_string() }
fn default_max_memory_mb() -> usize { 100 }
fn default_target_cpu_percent() -> f32 { 2.0 }
fn default_channel_capacity() -> usize { 10_000 }
fn default_compression_enabled() -> bool { true }
fn default_metrics_interval_seconds() -> u64 { 10 }
fn default_raw_events_days() -> u32 { 7 }
fn default_hourly_aggregates_days() -> u32 { 30 }
fn default_daily_summaries_days() -> u32 { 365 }
fn default_dev_screenshot_count() -> usize { 5 }

// Default implementations
impl Default for BatchingConfig {
    fn default() -> Self {
        Self {
            window_seconds: default_batch_window_seconds(),
            max_events_per_batch: default_max_events_per_batch(),
            buffer_capacity: default_batch_buffer_capacity(),
        }
    }
}

impl Default for ScreenshotConfig {
    fn default() -> Self {
        Self {
            memory_threshold_mb: default_memory_threshold_mb(),
            retention_seconds: default_retention_seconds(),
            memory_cache_size: default_memory_cache_size(),
            temp_dir: default_temp_dir(),
        }
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            path: default_database_path(),
            pool_size: default_pool_size(),
            write_buffer_size_mb: default_write_buffer_size_mb(),
            compaction_interval_hours: default_compaction_interval_hours(),
            wal_enabled: default_wal_enabled(),
            synchronous_mode: default_synchronous_mode(),
        }
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            max_memory_mb: default_max_memory_mb(),
            target_cpu_percent: default_target_cpu_percent(),
            channel_capacity: default_channel_capacity(),
            compression_enabled: default_compression_enabled(),
            metrics_interval_seconds: default_metrics_interval_seconds(),
        }
    }
}

impl Default for RetentionConfig {
    fn default() -> Self {
        Self {
            raw_events_days: default_raw_events_days(),
            hourly_aggregates_days: default_hourly_aggregates_days(),
            daily_summaries_days: default_daily_summaries_days(),
        }
    }
}

impl Default for DevModeConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            screenshot_count: default_dev_screenshot_count(),
            verbose_logging: false,
            debug_endpoints: false,
        }
    }
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            batching: BatchingConfig::default(),
            screenshot: ScreenshotConfig::default(),
            database: DatabaseConfig::default(),
            performance: PerformanceConfig::default(),
            retention: RetentionConfig::default(),
            dev_mode: DevModeConfig::default(),
        }
    }
}

impl StorageConfig {
    /// Load configuration from file
    pub fn from_file(path: &str) -> Result<Self, ConfigError> {
        let s = Config::builder()
            .add_source(File::with_name(path))
            .add_source(Environment::with_prefix("SKELLY_STORAGE"))
            .build()?;

        s.try_deserialize()
    }

    /// Load configuration from default locations
    pub fn load() -> Result<Self, ConfigError> {
        let mut builder = Config::builder()
            .set_default("batching.window_seconds", 30i64)?
            .set_default("screenshot.memory_threshold_mb", 5i64)?
            .set_default("database.pool_size", 4i64)?;

        // Try to load from config file if it exists
        let config_path = home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".skelly-jelly")
            .join("storage.toml");

        if config_path.exists() {
            builder = builder.add_source(File::from(config_path));
        }

        // Override with environment variables
        builder = builder.add_source(Environment::with_prefix("SKELLY_STORAGE"));

        let s = builder.build()?;
        s.try_deserialize()
    }

    /// Get batch window duration
    pub fn batch_window_duration(&self) -> Duration {
        Duration::from_secs(self.batching.window_seconds)
    }

    /// Get screenshot retention duration
    pub fn screenshot_retention_duration(&self) -> Duration {
        Duration::from_secs(self.screenshot.retention_seconds)
    }

    /// Get memory threshold in bytes
    pub fn memory_threshold_bytes(&self) -> usize {
        self.screenshot.memory_threshold_mb * 1024 * 1024
    }

    /// Get write buffer size in bytes
    pub fn write_buffer_bytes(&self) -> usize {
        self.database.write_buffer_size_mb * 1024 * 1024
    }

    /// Get max memory in bytes
    pub fn max_memory_bytes(&self) -> usize {
        self.performance.max_memory_mb * 1024 * 1024
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = StorageConfig::default();
        assert_eq!(config.batching.window_seconds, 30);
        assert_eq!(config.screenshot.memory_threshold_mb, 5);
        assert_eq!(config.database.pool_size, 4);
    }

    #[test]
    fn test_duration_conversions() {
        let config = StorageConfig::default();
        assert_eq!(config.batch_window_duration(), Duration::from_secs(30));
        assert_eq!(config.screenshot_retention_duration(), Duration::from_secs(30));
    }

    #[test]
    fn test_byte_conversions() {
        let config = StorageConfig::default();
        assert_eq!(config.memory_threshold_bytes(), 5 * 1024 * 1024);
        assert_eq!(config.write_buffer_bytes(), 10 * 1024 * 1024);
        assert_eq!(config.max_memory_bytes(), 100 * 1024 * 1024);
    }
}