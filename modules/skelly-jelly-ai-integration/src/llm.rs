//! Local LLM management and API fallback
//!
//! Handles local model loading, inference, and secure API fallback when needed.

use crate::config::{LocalModelSettings, APIConfig};
use crate::error::{AIIntegrationError, Result};
use crate::privacy::PrivacyGuardian;
use crate::types::{GenerationParams, APIResponse, LocalModelConfig, ModelVariant};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use reqwest::Client;
use serde_json::{json, Value};

/// Manages local LLM and API fallback
pub struct LLMManager {
    local_model: Option<Arc<Mutex<LocalLLM>>>,
    api_fallback: APIFallbackManager,
    privacy_guardian: Arc<PrivacyGuardian>,
    config: LocalModelSettings,
    usage_stats: Arc<Mutex<LLMUsageStats>>,
}

impl LLMManager {
    pub fn new(
        local_config: LocalModelSettings,
        api_config: APIConfig,
        privacy_guardian: Arc<PrivacyGuardian>,
    ) -> Self {
        Self {
            local_model: None,
            api_fallback: APIFallbackManager::new(api_config, privacy_guardian.clone()),
            privacy_guardian,
            config: local_config,
            usage_stats: Arc::new(Mutex::new(LLMUsageStats::default())),
        }
    }

    /// Initialize the LLM manager
    pub async fn initialize(&mut self) -> Result<()> {
        // Try to load local model first
        match self.load_local_model().await {
            Ok(model) => {
                self.local_model = Some(Arc::new(Mutex::new(model)));
                log::info!("Local LLM loaded successfully");
            }
            Err(e) => {
                log::warn!("Failed to load local model: {}, will use API fallback", e);
            }
        }

        // Initialize API clients
        self.api_fallback.initialize().await?;

        Ok(())
    }

    /// Generate response using best available method
    pub async fn generate(
        &self,
        prompt: &str,
        params: GenerationParams,
        allow_api: bool,
    ) -> Result<GenerationResult> {
        let start_time = Instant::now();

        // Try local model first if available
        if let Some(ref local_model) = self.local_model {
            match self.generate_local(local_model.clone(), prompt, &params).await {
                Ok(mut result) => {
                    result.generation_time = start_time.elapsed();
                    self.record_usage(GenerationMethod::Local, &result).await;
                    return Ok(result);
                }
                Err(e) => {
                    log::warn!("Local generation failed: {}, trying API fallback", e);
                }
            }
        }

        // Fall back to API if allowed and available
        if allow_api {
            match self.api_fallback.generate(prompt, &params).await {
                Ok(mut result) => {
                    result.generation_time = start_time.elapsed();
                    self.record_usage(GenerationMethod::API, &result).await;
                    return Ok(result);
                }
                Err(e) => {
                    log::error!("API fallback failed: {}", e);
                }
            }
        }

        Err(AIIntegrationError::ResourceUnavailable)
    }

    /// Check if local model is available
    pub fn has_local_model(&self) -> bool {
        self.local_model.is_some()
    }

    /// Get usage statistics
    pub async fn get_usage_stats(&self) -> LLMUsageStats {
        self.usage_stats.lock().await.clone()
    }

    /// Health check for LLM services
    pub async fn health_check(&self) -> LLMHealthStatus {
        let mut status = LLMHealthStatus::default();

        // Check local model
        if let Some(ref local_model) = self.local_model {
            let model_guard = local_model.lock().await;
            status.local_model_available = true;
            status.local_model_memory_mb = model_guard.get_memory_usage();
        }

        // Check API services
        status.api_services = self.api_fallback.health_check().await;

        status
    }

    async fn load_local_model(&self) -> Result<LocalLLM> {
        // Check if model file exists
        let model_path = self.config.model_path
            .as_ref()
            .ok_or(AIIntegrationError::ModelNotFound)?;

        if !model_path.exists() {
            if self.config.auto_download {
                self.download_model(&self.config.model_variant, model_path).await?;
            } else {
                return Err(AIIntegrationError::ModelNotFound);
            }
        }

        // Detect system capabilities
        let system_info = self.detect_system_capabilities()?;
        
        // Configure model based on available resources
        let config = self.build_model_config(&system_info)?;

        // Load the model
        LocalLLM::load(config).await
    }

    async fn download_model(&self, variant: &ModelVariant, path: &PathBuf) -> Result<()> {
        // In a real implementation, this would download from HuggingFace or similar
        // For security, we only allow downloading from trusted sources
        log::info!("Auto-download not implemented for security. Please download model manually.");
        Err(AIIntegrationError::FeatureNotAvailable { 
            feature: "auto-download".to_string() 
        })
    }

    fn detect_system_capabilities(&self) -> Result<SystemCapabilities> {
        use sysinfo::System;
        
        let mut sys = System::new_all();
        sys.refresh_all();

        Ok(SystemCapabilities {
            total_memory_gb: sys.total_memory() as f32 / (1024.0 * 1024.0 * 1024.0),
            available_memory_gb: sys.available_memory() as f32 / (1024.0 * 1024.0 * 1024.0),
            cpu_cores: sys.cpus().len(),
            has_gpu: self.detect_gpu_support(),
        })
    }

    fn detect_gpu_support(&self) -> bool {
        // Simplified GPU detection - in practice would use proper GPU detection
        cfg!(target_os = "macos") || std::env::var("CUDA_VISIBLE_DEVICES").is_ok()
    }

    fn build_model_config(&self, system: &SystemCapabilities) -> Result<LocalModelConfig> {
        let memory_limit = self.config.max_memory_gb.min(system.available_memory_gb * 0.8);
        
        if memory_limit < 1.0 {
            return Err(AIIntegrationError::InsufficientMemory {
                required_mb: 1024,
                available_mb: (memory_limit * 1024.0) as usize,
            });
        }

        Ok(LocalModelConfig {
            model_path: self.config.model_path.clone().unwrap(),
            model_variant: self.config.model_variant.clone(),
            n_gpu_layers: if self.config.use_gpu && system.has_gpu {
                self.config.gpu_layers.unwrap_or(32)
            } else {
                0
            },
            context_length: self.config.context_length,
            batch_size: self.config.batch_size,
            threads: self.config.threads.unwrap_or(system.cpu_cores),
            use_mmap: self.config.use_mmap,
            use_mlock: self.config.use_mlock && memory_limit > 4.0,
            temperature: self.config.temperature,
            top_p: self.config.top_p,
            repeat_penalty: self.config.repeat_penalty,
        })
    }

    async fn generate_local(
        &self,
        model: Arc<Mutex<LocalLLM>>,
        prompt: &str,
        params: &GenerationParams,
    ) -> Result<GenerationResult> {
        let mut model_guard = model.lock().await;
        model_guard.generate(prompt, params).await
    }

    async fn record_usage(&self, method: GenerationMethod, result: &GenerationResult) {
        let mut stats = self.usage_stats.lock().await;
        stats.total_requests += 1;
        stats.total_tokens += result.tokens_used as u64;
        stats.total_generation_time += result.generation_time;

        match method {
            GenerationMethod::Local => stats.local_requests += 1,
            GenerationMethod::API => stats.api_requests += 1,
        }
    }
}

/// Local LLM implementation (stub for now, would integrate with actual LLM library)
pub struct LocalLLM {
    config: LocalModelConfig,
    model_loaded: bool,
    memory_usage_mb: usize,
}

impl LocalLLM {
    pub async fn load(config: LocalModelConfig) -> Result<Self> {
        // In a real implementation, this would load the actual model
        // For now, we'll simulate the loading process
        
        log::info!("Loading local model from {:?}", config.model_path);
        
        // Simulate memory usage based on model variant
        let memory_usage_mb = match config.model_variant {
            ModelVariant::TinyLlama => 1024,   // 1GB
            ModelVariant::Phi3Mini => 2048,   // 2GB
            ModelVariant::Mistral7B => 4096,  // 4GB
            ModelVariant::Custom(_) => 2048,  // Default
        };

        // Simulate loading time
        tokio::time::sleep(Duration::from_millis(100)).await;

        Ok(Self {
            config,
            model_loaded: true,
            memory_usage_mb,
        })
    }

    pub async fn generate(&mut self, prompt: &str, params: &GenerationParams) -> Result<GenerationResult> {
        if !self.model_loaded {
            return Err(AIIntegrationError::NotInitialized);
        }

        // Check token limits
        let estimated_prompt_tokens = prompt.len() / 4; // Rough estimate
        if estimated_prompt_tokens > self.config.context_length {
            return Err(AIIntegrationError::ContextTooLong {
                tokens: estimated_prompt_tokens,
                max_tokens: self.config.context_length,
            });
        }

        // Simulate generation with timeout
        let generation_start = Instant::now();
        
        tokio::select! {
            _ = tokio::time::sleep(Duration::from_millis(params.timeout_ms)) => {
                Err(AIIntegrationError::GenerationTimeout {
                    duration: Duration::from_millis(params.timeout_ms)
                })
            }
            result = self.simulate_generation(prompt, params) => {
                result
            }
        }
    }

    async fn simulate_generation(&self, prompt: &str, params: &GenerationParams) -> Result<GenerationResult> {
        // Simulate generation time based on complexity
        let base_time = 100 + (params.max_tokens as u64 * 10);
        tokio::time::sleep(Duration::from_millis(base_time)).await;

        // Generate a simple response (in real implementation, this would be actual LLM output)
        let response = self.generate_template_response(prompt);
        let tokens_used = response.len() / 4; // Rough estimate

        Ok(GenerationResult {
            text: response,
            tokens_used: tokens_used as u32,
            generation_time: Duration::from_millis(base_time),
            model_info: format!("{:?}", self.config.model_variant),
            finish_reason: "completed".to_string(),
        })
    }

    fn generate_template_response(&self, prompt: &str) -> String {
        // Simple template-based response for simulation
        if prompt.to_lowercase().contains("focus") {
            "Try breaking your task into smaller chunks! You've got this. 💪".to_string()
        } else if prompt.to_lowercase().contains("distracted") {
            "Take a deep breath and gently redirect your attention. No worries!".to_string()
        } else if prompt.to_lowercase().contains("break") {
            "A short break sounds good! Maybe stretch or grab some water.".to_string()
        } else {
            "You're doing great! Keep up the good work. ✨".to_string()
        }
    }

    pub fn get_memory_usage(&self) -> usize {
        self.memory_usage_mb
    }
}

/// API fallback manager for cloud services
pub struct APIFallbackManager {
    config: APIConfig,
    privacy_guardian: Arc<PrivacyGuardian>,
    client: Client,
    usage_tracker: Arc<Mutex<APIUsageTracker>>,
}

impl APIFallbackManager {
    pub fn new(config: APIConfig, privacy_guardian: Arc<PrivacyGuardian>) -> Self {
        Self {
            config,
            privacy_guardian,
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap(),
            usage_tracker: Arc::new(Mutex::new(APIUsageTracker::default())),
        }
    }

    pub async fn initialize(&self) -> Result<()> {
        // Validate API keys if present
        if self.config.openai_key.is_some() {
            log::info!("OpenAI API key configured");
        }
        if self.config.anthropic_key.is_some() {
            log::info!("Anthropic API key configured");
        }

        Ok(())
    }

    pub async fn generate(&self, prompt: &str, params: &GenerationParams) -> Result<GenerationResult> {
        // Check if we have API access
        if self.config.openai_key.is_none() && self.config.anthropic_key.is_none() {
            return Err(AIIntegrationError::APIKeyMissing {
                service: "any".to_string(),
            });
        }

        // Check monthly cost limit
        {
            let tracker = self.usage_tracker.lock().await;
            if let Some(limit) = self.config.max_monthly_cost {
                if tracker.monthly_cost_usd >= limit {
                    return Err(AIIntegrationError::CostLimitExceeded);
                }
            }
        }

        // Sanitize prompt for API
        let sanitized_prompt = self.privacy_guardian.sanitize_prompt(prompt)?;

        // Try OpenAI first if available
        if let Some(ref api_key) = self.config.openai_key {
            match self.call_openai(&sanitized_prompt, params, api_key).await {
                Ok(result) => {
                    self.record_usage("openai", result.tokens_used as f32 * 0.0001).await; // Rough cost estimate
                    return Ok(result);
                }
                Err(e) => {
                    log::warn!("OpenAI API failed: {}", e);
                }
            }
        }

        // Try Anthropic as fallback
        if let Some(ref api_key) = self.config.anthropic_key {
            match self.call_anthropic(&sanitized_prompt, params, api_key).await {
                Ok(result) => {
                    self.record_usage("anthropic", result.tokens_used as f32 * 0.0001).await;
                    return Ok(result);
                }
                Err(e) => {
                    log::warn!("Anthropic API failed: {}", e);
                }
            }
        }

        Err(AIIntegrationError::ResourceUnavailable)
    }

    async fn call_openai(
        &self,
        prompt: &str,
        params: &GenerationParams,
        api_key: &str,
    ) -> Result<GenerationResult> {
        let request_body = json!({
            "model": self.config.openai_model,
            "messages": [
                {
                    "role": "system",
                    "content": "You are Skelly, a supportive skeleton companion helping with ADHD focus."
                },
                {
                    "role": "user", 
                    "content": prompt
                }
            ],
            "max_tokens": params.max_tokens,
            "temperature": params.temperature,
            "top_p": params.top_p
        });

        let response = self.client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(AIIntegrationError::APIError {
                service: "openai".to_string(),
                status: response.status().as_u16(),
            });
        }

        let response_json: Value = response.json().await?;
        
        let text = response_json["choices"][0]["message"]["content"]
            .as_str()
            .ok_or(AIIntegrationError::InvalidOutput)?
            .to_string();

        let tokens_used = response_json["usage"]["total_tokens"]
            .as_u64()
            .unwrap_or(0) as u32;

        Ok(GenerationResult {
            text,
            tokens_used,
            generation_time: Duration::from_millis(0), // Will be set by caller
            model_info: self.config.openai_model.clone(),
            finish_reason: "completed".to_string(),
        })
    }

    async fn call_anthropic(
        &self,
        prompt: &str,
        params: &GenerationParams,
        api_key: &str,
    ) -> Result<GenerationResult> {
        let request_body = json!({
            "model": self.config.anthropic_model,
            "max_tokens": params.max_tokens,
            "temperature": params.temperature,
            "top_p": params.top_p,
            "messages": [
                {
                    "role": "user",
                    "content": prompt
                }
            ]
        });

        let response = self.client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", api_key)
            .header("Content-Type", "application/json")
            .header("anthropic-version", "2023-06-01")
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(AIIntegrationError::APIError {
                service: "anthropic".to_string(),
                status: response.status().as_u16(),
            });
        }

        let response_json: Value = response.json().await?;
        
        let text = response_json["content"][0]["text"]
            .as_str()
            .ok_or(AIIntegrationError::InvalidOutput)?
            .to_string();

        let tokens_used = response_json["usage"]["output_tokens"]
            .as_u64()
            .unwrap_or(0) as u32;

        Ok(GenerationResult {
            text,
            tokens_used,
            generation_time: Duration::from_millis(0),
            model_info: self.config.anthropic_model.clone(),
            finish_reason: "completed".to_string(),
        })
    }

    async fn record_usage(&self, service: &str, cost_usd: f32) {
        let mut tracker = self.usage_tracker.lock().await;
        tracker.monthly_cost_usd += cost_usd;
        tracker.requests_by_service.entry(service.to_string())
            .and_modify(|count| *count += 1)
            .or_insert(1);
    }

    pub async fn health_check(&self) -> std::collections::HashMap<String, bool> {
        let mut status = std::collections::HashMap::new();
        
        // Simple connectivity check (in practice would be more sophisticated)
        status.insert("openai".to_string(), self.config.openai_key.is_some());
        status.insert("anthropic".to_string(), self.config.anthropic_key.is_some());
        
        status
    }
}

// Supporting types and structures

#[derive(Debug, Clone)]
pub struct GenerationResult {
    pub text: String,
    pub tokens_used: u32,
    pub generation_time: Duration,
    pub model_info: String,
    pub finish_reason: String,
}

#[derive(Debug, Clone)]
pub enum GenerationMethod {
    Local,
    API,
}

#[derive(Debug, Clone, Default)]
pub struct LLMUsageStats {
    pub total_requests: u64,
    pub local_requests: u64,
    pub api_requests: u64,
    pub total_tokens: u64,
    pub total_generation_time: Duration,
}

#[derive(Debug, Clone, Default)]
pub struct LLMHealthStatus {
    pub local_model_available: bool,
    pub local_model_memory_mb: usize,
    pub api_services: std::collections::HashMap<String, bool>,
}

#[derive(Debug, Clone)]
pub struct SystemCapabilities {
    pub total_memory_gb: f32,
    pub available_memory_gb: f32,
    pub cpu_cores: usize,
    pub has_gpu: bool,
}

#[derive(Debug, Clone, Default)]
pub struct APIUsageTracker {
    pub monthly_cost_usd: f32,
    pub requests_by_service: std::collections::HashMap<String, u64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AIIntegrationConfig;

    #[tokio::test]
    async fn test_local_llm_simulation() {
        let config = LocalModelConfig {
            model_path: PathBuf::from("/tmp/test_model"),
            model_variant: ModelVariant::TinyLlama,
            n_gpu_layers: 0,
            context_length: 2048,
            batch_size: 256,
            threads: 4,
            use_mmap: true,
            use_mlock: false,
            temperature: 0.7,
            top_p: 0.9,
            repeat_penalty: 1.1,
        };

        // This test simulates loading without actual model file
        // In real implementation, would need actual model file
        let mut llm = LocalLLM {
            config,
            model_loaded: true,
            memory_usage_mb: 1024,
        };

        let params = GenerationParams::default();
        let result = llm.generate("Help me focus", &params).await.unwrap();
        
        assert!(!result.text.is_empty());
        assert!(result.tokens_used > 0);
    }

    #[test]
    fn test_system_capabilities_detection() {
        let manager = LLMManager::new(
            Default::default(),
            Default::default(),
            Arc::new(PrivacyGuardian::new()),
        );

        // This should not panic
        let capabilities = manager.detect_system_capabilities().unwrap();
        assert!(capabilities.total_memory_gb > 0.0);
        assert!(capabilities.cpu_cores > 0);
    }
}