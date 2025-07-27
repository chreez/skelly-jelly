//! Configuration structures for the data capture module

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Main configuration for the data capture module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataCaptureConfig {
    /// Monitor-specific configurations
    pub monitors: MonitorConfig,
    /// Privacy settings
    pub privacy: PrivacyConfig,
    /// Performance tuning
    pub performance: PerformanceConfig,
}

impl Default for DataCaptureConfig {
    fn default() -> Self {
        Self {
            monitors: MonitorConfig::default(),
            privacy: PrivacyConfig::default(),
            performance: PerformanceConfig::default(),
        }
    }
}

/// Configuration for individual monitors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorConfig {
    pub keystroke: KeystrokeConfig,
    pub mouse: MouseConfig,
    pub window: WindowConfig,
    pub screenshot: ScreenshotConfig,
    pub process: ProcessConfig,
    pub resource: ResourceConfig,
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            keystroke: KeystrokeConfig::default(),
            mouse: MouseConfig::default(),
            window: WindowConfig::default(),
            screenshot: ScreenshotConfig::default(),
            process: ProcessConfig::default(),
            resource: ResourceConfig::default(),
        }
    }
}

/// Keystroke monitor configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeystrokeConfig {
    pub enabled: bool,
    pub buffer_size: usize,
    pub coalescence_ms: u64,
    pub capture_modifiers: bool,
    pub capture_special_keys: bool,
}

impl Default for KeystrokeConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            buffer_size: 1000,
            coalescence_ms: 10,
            capture_modifiers: true,
            capture_special_keys: true,
        }
    }
}

/// Mouse monitor configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MouseConfig {
    pub enabled: bool,
    pub buffer_size: usize,
    pub movement_threshold: f64,
    pub click_coalescence_ms: u64,
    pub capture_movement: bool,
    pub capture_clicks: bool,
    pub capture_scroll: bool,
}

impl Default for MouseConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            buffer_size: 5000,
            movement_threshold: 5.0,
            click_coalescence_ms: 50,
            capture_movement: true,
            capture_clicks: true,
            capture_scroll: true,
        }
    }
}

/// Window monitor configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    pub enabled: bool,
    pub capture_title: bool,
    pub capture_app_name: bool,
    pub switch_threshold_ms: u64,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            capture_title: true,
            capture_app_name: true,
            switch_threshold_ms: 100,
        }
    }
}

/// Screenshot monitor configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenshotConfig {
    pub enabled: bool,
    pub capture_interval_ms: u64,
    pub max_size_mb: usize,
    pub compression_quality: u8,
    pub capture_on_significant_change: bool,
    pub change_threshold: f32,
    pub privacy_mode: PrivacyMode,
}

impl Default for ScreenshotConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            capture_interval_ms: 30000, // 30 seconds
            max_size_mb: 5,
            compression_quality: 85,
            capture_on_significant_change: true,
            change_threshold: 0.1, // 10% change
            privacy_mode: PrivacyMode::Balanced,
        }
    }
}

/// Process monitor configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessConfig {
    pub enabled: bool,
    pub sample_interval_ms: u64,
    pub capture_command_line: bool,
    pub capture_environment: bool,
}

impl Default for ProcessConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            sample_interval_ms: 1000, // 1 second
            capture_command_line: false,
            capture_environment: false,
        }
    }
}

/// Resource monitor configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceConfig {
    pub enabled: bool,
    pub sample_interval_ms: u64,
    pub capture_cpu: bool,
    pub capture_memory: bool,
    pub capture_disk: bool,
    pub capture_network: bool,
}

impl Default for ResourceConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            sample_interval_ms: 1000, // 1 second
            capture_cpu: true,
            capture_memory: true,
            capture_disk: false,
            capture_network: false,
        }
    }
}

/// Privacy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyConfig {
    pub pii_detection: bool,
    pub sensitive_app_list: Vec<String>,
    pub ignored_app_list: Vec<String>,
    pub mask_passwords: bool,
    pub mask_credit_cards: bool,
    pub mask_ssn: bool,
    pub mask_emails: bool,
    pub screenshot_privacy_zones: Vec<PrivacyZone>,
}

impl Default for PrivacyConfig {
    fn default() -> Self {
        Self {
            pii_detection: true,
            sensitive_app_list: vec![
                "1Password".to_string(),
                "KeePass".to_string(),
                "Bitwarden".to_string(),
                "Banking App".to_string(),
            ],
            ignored_app_list: vec![],
            mask_passwords: true,
            mask_credit_cards: true,
            mask_ssn: true,
            mask_emails: false,
            screenshot_privacy_zones: vec![],
        }
    }
}

/// Privacy zone for screenshots
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyZone {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub blur_radius: u32,
}

/// Privacy mode levels
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum PrivacyMode {
    /// Minimal privacy - faster but less secure
    Minimal,
    /// Balanced privacy - good performance with reasonable security
    Balanced,
    /// Strict privacy - maximum security, may impact performance
    Strict,
}

/// Performance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub max_cpu_percent: f32,
    pub max_memory_mb: usize,
    pub event_buffer_size: usize,
    pub event_batch_size: usize,
    pub backpressure_threshold: f32,
    pub drop_on_overflow: bool,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            max_cpu_percent: 1.0,
            max_memory_mb: 50,
            event_buffer_size: 10000,
            event_batch_size: 100,
            backpressure_threshold: 0.8,
            drop_on_overflow: true,
        }
    }
}