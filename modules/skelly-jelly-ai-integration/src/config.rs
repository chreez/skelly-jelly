//! Configuration management for AI Integration module
//!
//! Provides secure, privacy-focused configuration with sensible defaults.

use crate::types::{ModelVariant, UserPrivacyLevel, APIConsent};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

/// Main configuration for AI Integration module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIIntegrationConfig {
    /// Local model settings
    pub local_model: LocalModelSettings,
    
    /// API configuration for fallback
    pub api_config: APIConfig,
    
    /// Privacy settings and controls
    pub privacy: PrivacySettings,
    
    /// Performance and resource settings
    pub performance: PerformanceSettings,
    
    /// Personality configuration
    pub personality: PersonalityConfig,
    
    /// Template system settings
    pub templates: TemplateSettings,
}

impl Default for AIIntegrationConfig {
    fn default() -> Self {
        Self {
            local_model: LocalModelSettings::default(),
            api_config: APIConfig::default(),
            privacy: PrivacySettings::default(),
            performance: PerformanceSettings::default(),
            personality: PersonalityConfig::default(),
            templates: TemplateSettings::default(),
        }
    }
}

/// Local model configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalModelSettings {
    /// Path to the model file (GGUF format)
    pub model_path: Option<PathBuf>,
    
    /// Automatically download model if missing
    pub auto_download: bool,
    
    /// Which model variant to use
    pub model_variant: ModelVariant,
    
    /// Maximum memory to use for model (in GB)
    pub max_memory_gb: f32,
    
    /// Enable GPU acceleration if available
    pub use_gpu: bool,
    
    /// Number of layers to offload to GPU (None = auto-detect)
    pub gpu_layers: Option<i32>,
    
    /// Context window size
    pub context_length: usize,
    
    /// Batch size for processing
    pub batch_size: usize,
    
    /// Number of CPU threads to use (None = auto-detect)
    pub threads: Option<usize>,
    
    /// Use memory-mapped file loading
    pub use_mmap: bool,
    
    /// Lock model in RAM
    pub use_mlock: bool,
    
    /// Default generation temperature
    pub temperature: f32,
    
    /// Default top-p value
    pub top_p: f32,
    
    /// Repeat penalty
    pub repeat_penalty: f32,
}

impl Default for LocalModelSettings {
    fn default() -> Self {
        Self {
            model_path: None,
            auto_download: false, // Security: Don't auto-download by default
            model_variant: ModelVariant::Phi3Mini, // Smaller, faster model
            max_memory_gb: 4.0,
            use_gpu: true,
            gpu_layers: None, // Auto-detect
            context_length: 4096,
            batch_size: 512,
            threads: None, // Auto-detect
            use_mmap: true,
            use_mlock: false, // Don't lock by default
            temperature: 0.7,
            top_p: 0.9,
            repeat_penalty: 1.1,
        }
    }
}

/// API configuration for fallback services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct APIConfig {
    /// OpenAI API key (stored securely)
    pub openai_key: Option<String>,
    
    /// Anthropic API key (stored securely)
    pub anthropic_key: Option<String>,
    
    /// Maximum monthly cost limit in USD
    pub max_monthly_cost: Option<f32>,
    
    /// Always try local first, even if API is available
    pub prefer_local: bool,
    
    /// Timeout for API requests
    pub request_timeout: Duration,
    
    /// Maximum retries for failed requests
    pub max_retries: u32,
    
    /// Default model for OpenAI
    pub openai_model: String,
    
    /// Default model for Anthropic
    pub anthropic_model: String,
    
    /// Enable request caching
    pub enable_caching: bool,
    
    /// Cache TTL
    pub cache_ttl: Duration,
}

impl Default for APIConfig {
    fn default() -> Self {
        Self {
            openai_key: None,
            anthropic_key: None,
            max_monthly_cost: Some(10.0), // Conservative default
            prefer_local: true, // Privacy-first
            request_timeout: Duration::from_secs(30),
            max_retries: 3,
            openai_model: "gpt-3.5-turbo".to_string(),
            anthropic_model: "claude-3-haiku-20240307".to_string(),
            enable_caching: true,
            cache_ttl: Duration::from_secs(3600),
        }
    }
}

/// Privacy settings and controls
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacySettings {
    /// Default privacy level
    pub default_privacy_level: UserPrivacyLevel,
    
    /// Allow API fallback when local fails
    pub allow_api_fallback: bool,
    
    /// Prompt user for consent before first API use
    pub api_consent_prompt: bool,
    
    /// Always sanitize prompts before API
    pub sanitize_prompts: bool,
    
    /// Log prompts and responses for debugging (local only)
    pub log_prompts: bool,
    
    /// Enable PII detection
    pub pii_detection: bool,
    
    /// Prompt injection detection
    pub prompt_injection_detection: bool,
    
    /// Audit logging for compliance
    pub audit_logging: bool,
    
    /// Data retention period for logs
    pub log_retention_days: u32,
}

impl Default for PrivacySettings {
    fn default() -> Self {
        Self {
            default_privacy_level: UserPrivacyLevel::LocalOnly, // Most private
            allow_api_fallback: false, // Privacy-first
            api_consent_prompt: true, // Always ask for consent
            sanitize_prompts: true, // Always sanitize
            log_prompts: false, // Don't log by default for privacy
            pii_detection: true, // Enable PII detection
            prompt_injection_detection: true, // Security feature
            audit_logging: true, // For compliance
            log_retention_days: 30, // Limited retention
        }
    }
}

/// Performance and resource settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSettings {
    /// Maximum response time before timeout (ms)
    pub max_response_time_ms: u64,
    
    /// Enable response caching
    pub enable_caching: bool,
    
    /// Cache size (number of entries)
    pub cache_size: usize,
    
    /// Parallel processing for batch requests
    pub enable_parallel_processing: bool,
    
    /// Memory pool size for efficient allocation
    pub memory_pool_size_mb: usize,
    
    /// Context compression threshold
    pub context_compression_threshold: usize,
    
    /// Resource monitoring interval
    pub monitoring_interval: Duration,
    
    /// Automatic memory cleanup
    pub auto_cleanup: bool,
    
    /// Background model loading
    pub background_loading: bool,
}

impl Default for PerformanceSettings {
    fn default() -> Self {
        Self {
            max_response_time_ms: 5000,
            enable_caching: true,
            cache_size: 1000,
            enable_parallel_processing: true,
            memory_pool_size_mb: 100,
            context_compression_threshold: 3000, // tokens
            monitoring_interval: Duration::from_secs(60),
            auto_cleanup: true,
            background_loading: false, // Don't load in background by default
        }
    }
}

/// Personality configuration for Skelly
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalityConfig {
    /// Enable dynamic personality adjustment
    pub adaptive_personality: bool,
    
    /// Skeleton pun frequency (0.0-1.0)
    pub pun_frequency: f32,
    
    /// Enable context-aware responses
    pub context_awareness: bool,
    
    /// Message length preference
    pub preferred_message_length: MessageLength,
    
    /// Tone consistency checking
    pub tone_consistency: bool,
    
    /// Enable personality learning from user feedback
    pub personality_learning: bool,
}

impl Default for PersonalityConfig {
    fn default() -> Self {
        Self {
            adaptive_personality: true,
            pun_frequency: 0.1, // Occasional puns
            context_awareness: true,
            preferred_message_length: MessageLength::Brief,
            tone_consistency: true,
            personality_learning: false, // Privacy consideration
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageLength {
    VeryBrief,  // 1 sentence
    Brief,      // 1-2 sentences  
    Medium,     // 2-3 sentences
    Detailed,   // 3+ sentences
}

/// Template system settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateSettings {
    /// Enable template fallback
    pub enable_templates: bool,
    
    /// Template variety (number of variations per category)
    pub template_variety: usize,
    
    /// Personalize templates with user context
    pub personalize_templates: bool,
    
    /// Template refresh interval
    pub refresh_interval: Duration,
    
    /// Custom template directory
    pub custom_template_dir: Option<PathBuf>,
    
    /// Template priority over LLM
    pub prefer_templates: bool,
}

impl Default for TemplateSettings {
    fn default() -> Self {
        Self {
            enable_templates: true,
            template_variety: 5,
            personalize_templates: true,
            refresh_interval: Duration::from_secs(3600),
            custom_template_dir: None,
            prefer_templates: false, // Prefer LLM when available
        }
    }
}

impl AIIntegrationConfig {
    /// Create a privacy-focused configuration
    pub fn privacy_focused() -> Self {
        let mut config = Self::default();
        config.privacy.allow_api_fallback = false;
        config.privacy.log_prompts = false;
        config.api_config.prefer_local = true;
        config.local_model.auto_download = false;
        config
    }
    
    /// Create a performance-focused configuration
    pub fn performance_focused() -> Self {
        let mut config = Self::default();
        config.performance.enable_parallel_processing = true;
        config.performance.background_loading = true;
        config.performance.cache_size = 2000;
        config.local_model.use_gpu = true;
        config.local_model.use_mlock = true;
        config
    }
    
    /// Create a minimal resource configuration
    pub fn minimal_resources() -> Self {
        let mut config = Self::default();
        config.local_model.model_variant = ModelVariant::TinyLlama;
        config.local_model.max_memory_gb = 2.0;
        config.local_model.context_length = 2048;
        config.performance.cache_size = 100;
        config.performance.memory_pool_size_mb = 50;
        config
    }
    
    /// Validate the configuration
    pub fn validate(&self) -> Result<(), String> {
        // Memory validation
        if self.local_model.max_memory_gb < 1.0 {
            return Err("Maximum memory must be at least 1GB".to_string());
        }
        
        // Context length validation
        if self.local_model.context_length < 512 {
            return Err("Context length must be at least 512 tokens".to_string());
        }
        
        // Cost validation
        if let Some(cost) = self.api_config.max_monthly_cost {
            if cost < 0.0 {
                return Err("Monthly cost limit cannot be negative".to_string());
            }
        }
        
        // Privacy consistency validation
        if !self.privacy.allow_api_fallback && 
           (self.api_config.openai_key.is_some() || self.api_config.anthropic_key.is_some()) {
            return Err("API keys provided but API fallback disabled".to_string());
        }
        
        // Performance validation
        if self.performance.max_response_time_ms < 1000 {
            return Err("Response timeout too short (minimum 1000ms)".to_string());
        }
        
        Ok(())
    }
    
    /// Get effective memory limit considering system resources
    pub fn effective_memory_limit(&self) -> usize {
        let system_memory = self.get_system_memory_gb();
        let requested = self.local_model.max_memory_gb;
        let available = system_memory * 0.8; // Leave 20% for system
        
        (requested.min(available) * 1024.0 * 1024.0 * 1024.0) as usize
    }
    
    /// Detect system memory (simplified)
    fn get_system_memory_gb(&self) -> f32 {
        // In a real implementation, this would use sysinfo or similar
        8.0 // Default assumption
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AIIntegrationConfig::default();
        assert!(config.privacy.pii_detection);
        assert!(!config.privacy.allow_api_fallback);
        assert_eq!(config.local_model.model_variant, ModelVariant::Phi3Mini);
    }

    #[test]
    fn test_privacy_focused_config() {
        let config = AIIntegrationConfig::privacy_focused();
        assert!(!config.privacy.allow_api_fallback);
        assert!(!config.privacy.log_prompts);
        assert!(config.api_config.prefer_local);
    }

    #[test]
    fn test_config_validation() {
        let mut config = AIIntegrationConfig::default();
        assert!(config.validate().is_ok());
        
        config.local_model.max_memory_gb = 0.5;
        assert!(config.validate().is_err());
        
        config.local_model.max_memory_gb = 2.0;
        config.local_model.context_length = 100;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_message_length_serialization() {
        let length = MessageLength::Brief;
        let json = serde_json::to_string(&length).unwrap();
        let deserialized: MessageLength = serde_json::from_str(&json).unwrap();
        assert!(matches!(deserialized, MessageLength::Brief));
    }
}