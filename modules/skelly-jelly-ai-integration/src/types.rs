//! Type definitions for the AI Integration module
//!
//! Provides comprehensive type safety and clear interfaces for AI operations.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;

// Re-export event bus types for convenience
pub use skelly_jelly_event_bus::message::{InterventionRequest, InterventionResponse, AnimationCommand};

/// Core trait for AI integration functionality
#[async_trait::async_trait]
pub trait AIIntegration: Send + Sync {
    /// Process an intervention request from the gamification module
    async fn process_intervention(
        &self,
        request: InterventionRequest,
    ) -> crate::Result<InterventionResponse>;

    /// Generate animation commands based on message content and mood
    async fn generate_animation(
        &self,
        text: &str,
        mood: CompanionMood,
    ) -> crate::Result<AnimationCommand>;

    /// Update personality settings
    async fn update_personality(
        &self,
        traits: PersonalityTraits,
    ) -> crate::Result<()>;

    /// Get usage statistics for monitoring and optimization
    async fn get_usage_stats(&self) -> UsageStatistics;

    /// Check if local model is available and healthy
    async fn health_check(&self) -> HealthStatus;
}

/// Enhanced intervention request with comprehensive context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtendedInterventionRequest {
    pub base: InterventionRequest,
    pub current_state: ADHDState,
    pub behavioral_metrics: BehavioralMetrics,
    pub work_context: WorkContext,
    pub state_history: Vec<ADHDState>,
    pub user_preferences: UserPreferences,
}

/// Enhanced intervention response with additional metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtendedInterventionResponse {
    pub base: InterventionResponse,
    pub generation_method: GenerationMethod,
    pub confidence: f32,
    pub tokens_used: Option<u32>,
    pub processing_time_ms: u64,
    pub privacy_level: PrivacyLevel,
}

/// How the response was generated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GenerationMethod {
    LocalLLM { model_name: String },
    APIFallback { service: String },
    Template { template_id: String },
    Cached { cache_key: String },
}

/// Privacy level of the response generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PrivacyLevel {
    Local,           // Fully local processing
    Sanitized,       // API with sanitized data
    ConsentBased,    // API with explicit consent
}

/// ADHD state classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ADHDState {
    pub state_type: ADHDStateType,
    pub confidence: f32,
    pub depth: Option<f32>,  // For flow states
    pub duration: u64,       // Duration in milliseconds
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ADHDStateType {
    Flow { depth: f32 },
    Hyperfocus { intensity: f32 },
    Distracted { severity: f32 },
    Transitioning,
    Neutral,
}

/// Behavioral metrics for context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehavioralMetrics {
    pub productive_time_ratio: f32,
    pub distraction_frequency: f32,
    pub focus_session_count: u32,
    pub average_session_length: u64,
    pub recovery_time: u64,
    pub transition_smoothness: f32,
}

/// Work context information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkContext {
    pub work_type: WorkType,
    pub application: String,
    pub window_title: String,
    pub screenshot_text: Option<String>,
    pub task_category: TaskCategory,
    pub urgency: UrgencyLevel,
    pub time_of_day: TimeOfDay,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkType {
    Coding { language: String, framework: Option<String> },
    Writing { document_type: String },
    Design { tool: String, project_type: String },
    Research { topic: String },
    Communication { platform: String },
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskCategory {
    Work,
    Creative,
    Research,
    Communication,
    Leisure,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UrgencyLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimeOfDay {
    Morning,
    Afternoon,
    Evening,
    Night,
}

/// User preferences for AI behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    pub intervention_frequency: InterventionFrequency,
    pub message_style: MessageStyle,
    pub privacy_level: UserPrivacyLevel,
    pub personality_traits: PersonalityTraits,
    pub api_consent: APIConsent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InterventionFrequency {
    Minimal,
    Moderate,
    Frequent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageStyle {
    Encouraging,
    Informative,
    Minimal,
    Humorous,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserPrivacyLevel {
    LocalOnly,
    SanitizedAPI,
    ConsentBased,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct APIConsent {
    pub openai_allowed: bool,
    pub anthropic_allowed: bool,
    pub consent_timestamp: Option<DateTime<Utc>>,
    pub monthly_limit_usd: Option<f32>,
}

/// Personality traits for Skelly companion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalityTraits {
    pub cheerfulness: f32,      // 0.0-1.0, default 0.7
    pub humor: f32,             // 0.0-1.0, default 0.5  
    pub supportiveness: f32,    // 0.0-1.0, default 0.9
    pub casualness: f32,        // 0.0-1.0, default 0.8
    pub pun_frequency: f32,     // 0.0-1.0, default 0.1 (rare)
    pub directness: f32,        // 0.0-1.0, default 0.6
}

impl Default for PersonalityTraits {
    fn default() -> Self {
        Self {
            cheerfulness: 0.7,
            humor: 0.5,
            supportiveness: 0.9,
            casualness: 0.8,
            pun_frequency: 0.1,
            directness: 0.6,
        }
    }
}

/// Companion mood for animation generation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum CompanionMood {
    Happy,
    Excited,
    Supportive,
    Concerned,
    Celebrating,
    Neutral,
    Sleepy,
}

/// LLM context for generation
#[derive(Debug, Clone)]
pub struct LLMContext {
    pub system_prompt: String,
    pub behavioral_context: String,
    pub work_context: String,
    pub intervention_type: String,
    pub user_preferences: UserPreferences,
    pub max_tokens: usize,
}

/// Template-based suggestion for quick responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateSuggestion {
    pub template_id: String,
    pub category: TemplateCategory,
    pub variations: Vec<String>,
    pub personality_modifiers: Vec<PersonalityModifier>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub enum TemplateCategory {
    Encouragement,
    GentleNudge,
    Celebration,
    Suggestion,
    BreakReminder,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalityModifier {
    pub modifier_type: ModifierType,
    pub content: String,
    pub conditions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModifierType {
    SkeletonPun,
    CasualPhrase,
    SupportivePhrase,
    HumorousAddition,
}

/// Local model configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalModelConfig {
    pub model_path: PathBuf,
    pub model_variant: ModelVariant,
    pub n_gpu_layers: i32,
    pub context_length: usize,
    pub batch_size: usize,
    pub threads: usize,
    pub use_mmap: bool,
    pub use_mlock: bool,
    pub temperature: f32,
    pub top_p: f32,
    pub repeat_penalty: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelVariant {
    Mistral7B,
    Phi3Mini,
    TinyLlama,
    Custom(String),
}

/// Generation parameters for LLM
#[derive(Debug, Clone)]
pub struct GenerationParams {
    pub max_tokens: usize,
    pub temperature: f32,
    pub top_p: f32,
    pub repeat_penalty: f32,
    pub stop_sequences: Vec<String>,
    pub timeout_ms: u64,
}

impl Default for GenerationParams {
    fn default() -> Self {
        Self {
            max_tokens: 150,
            temperature: 0.7,
            top_p: 0.9,
            repeat_penalty: 1.1,
            stop_sequences: vec!["\n\n".to_string()],
            timeout_ms: 5000,
        }
    }
}

/// API response wrapper
#[derive(Debug, Clone)]
pub struct APIResponse {
    pub text: String,
    pub tokens_used: u32,
    pub service: String,
    pub model: String,
    pub cost_usd: Option<f32>,
}

/// Usage statistics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UsageStatistics {
    pub requests_processed: u64,
    pub local_generations: u64,
    pub api_generations: u64,
    pub template_responses: u64,
    pub cached_responses: u64,
    pub average_response_time_ms: f64,
    pub total_tokens_used: u64,
    pub total_cost_usd: f32,
    pub privacy_violations_blocked: u64,
    pub uptime_percentage: f32,
    pub error_rate: f32,
}

/// Health status for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub overall_status: ServiceStatus,
    pub local_model_status: ServiceStatus,
    pub api_services: HashMap<String, ServiceStatus>,
    pub memory_usage_mb: usize,
    pub response_time_p95_ms: u64,
    pub error_rate_1h: f32,
    pub last_check: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

/// Privacy detection result
#[derive(Debug, Clone)]
pub struct PrivacyAnalysis {
    pub has_pii: bool,
    pub has_sensitive_data: bool,
    pub detected_patterns: Vec<SensitivePattern>,
    pub sanitized_text: String,
    pub safety_score: f32, // 0.0 = unsafe, 1.0 = safe
}

#[derive(Debug, Clone)]
pub struct SensitivePattern {
    pub pattern_type: SensitivePatternType,
    pub location: (usize, usize), // start, end positions
    pub confidence: f32,
    pub replacement: String,
}

#[derive(Debug, Clone)]
pub enum SensitivePatternType {
    Email,
    Phone,
    SSN,
    CreditCard,
    IPAddress,
    PersonalName,
    Organization,
    Custom(String),
}

/// Animation hint for cute figurine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationHint {
    pub animation_type: String,
    pub duration_ms: u32,
    pub intensity: f32,
    pub mood: CompanionMood,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_personality_traits_default() {
        let traits = PersonalityTraits::default();
        assert_eq!(traits.cheerfulness, 0.7);
        assert_eq!(traits.supportiveness, 0.9);
        assert_eq!(traits.pun_frequency, 0.1);
    }

    #[test]
    fn test_generation_params_default() {
        let params = GenerationParams::default();
        assert_eq!(params.max_tokens, 150);
        assert_eq!(params.temperature, 0.7);
        assert_eq!(params.top_p, 0.9);
    }

    #[test]
    fn test_serialization() {
        let traits = PersonalityTraits::default();
        let json = serde_json::to_string(&traits).unwrap();
        let deserialized: PersonalityTraits = serde_json::from_str(&json).unwrap();
        assert_eq!(traits.cheerfulness, deserialized.cheerfulness);
    }
}