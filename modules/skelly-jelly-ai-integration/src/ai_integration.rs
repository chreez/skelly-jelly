//! Main AI Integration implementation
//!
//! Orchestrates all AI functionality with privacy-first, security-focused design.

use crate::config::AIIntegrationConfig;
use crate::context::ContextProcessor;
use crate::error::{AIIntegrationError, Result};
use crate::llm::LLMManager;
use crate::personality::PersonalityEngine;
use crate::privacy::PrivacyGuardian;
use crate::suggestions::{SuggestionGenerator, SuggestionUrgency};
use crate::types::{
    AIIntegration, ExtendedInterventionRequest, ExtendedInterventionResponse,
    PersonalityTraits, CompanionMood, UsageStatistics, HealthStatus,
    GenerationMethod, PrivacyLevel, ServiceStatus
};

use skelly_jelly_event_bus::message::{InterventionRequest, InterventionResponse, AnimationCommand, ModuleId};
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::Utc;
use uuid::Uuid;

/// Main AI Integration implementation
pub struct AIIntegrationImpl {
    config: AIIntegrationConfig,
    context_processor: ContextProcessor,
    llm_manager: Arc<LLMManager>,
    suggestion_generator: SuggestionGenerator,
    privacy_guardian: Arc<PrivacyGuardian>,
    personality_engine: Arc<RwLock<PersonalityEngine>>,
    usage_stats: Arc<RwLock<UsageStatistics>>,
    initialized: bool,
}

impl AIIntegrationImpl {
    /// Create a new AI Integration instance
    pub fn new(config: AIIntegrationConfig) -> Self {
        let privacy_guardian = Arc::new(PrivacyGuardian::new());
        let personality_engine = Arc::new(RwLock::new(
            PersonalityEngine::new(config.personality.traits())
        ));
        
        let llm_manager = Arc::new(LLMManager::new(
            config.local_model.clone(),
            config.api_config.clone(),
            privacy_guardian.clone(),
        ));

        let suggestion_generator = SuggestionGenerator::new(
            llm_manager.clone(),
            PersonalityEngine::new(config.personality.traits()),
        );

        Self {
            config,
            context_processor: ContextProcessor::new(),
            llm_manager,
            suggestion_generator,
            privacy_guardian,
            personality_engine,
            usage_stats: Arc::new(RwLock::new(UsageStatistics::default())),
            initialized: false,
        }
    }

    /// Initialize the AI Integration system
    pub async fn initialize(&mut self) -> Result<()> {
        if self.initialized {
            return Ok(());
        }

        log::info!("Initializing AI Integration module");

        // Validate configuration
        self.config.validate()
            .map_err(|e| AIIntegrationError::InvalidConfig { field: e })?;

        // Initialize LLM manager
        let llm_manager = Arc::get_mut(&mut self.llm_manager)
            .ok_or(AIIntegrationError::InternalError)?;
        llm_manager.initialize().await?;

        self.initialized = true;
        log::info!("AI Integration module initialized successfully");
        
        Ok(())
    }

    /// Convert basic InterventionRequest to extended format
    async fn extend_intervention_request(
        &self,
        request: InterventionRequest,
    ) -> Result<ExtendedInterventionRequest> {
        // In a real implementation, this would gather additional context
        // from other modules via the event bus
        
        // Parse context from the basic request
        let context_value = &request.context;
        
        // Extract work context, behavioral metrics, etc. from the JSON
        // This is simplified - in practice would have proper deserialization
        let work_context = self.extract_work_context(context_value)?;
        let behavioral_metrics = self.extract_behavioral_metrics(context_value)?;
        let current_state = self.extract_current_state(context_value)?;
        let user_preferences = self.extract_user_preferences(context_value)?;

        Ok(ExtendedInterventionRequest {
            base: request,
            current_state,
            behavioral_metrics,
            work_context,
            state_history: Vec::new(), // Would be populated from storage
            user_preferences,
        })
    }

    /// Determine suggestion urgency based on context
    fn determine_urgency(&self, request: &ExtendedInterventionRequest) -> SuggestionUrgency {
        match request.base.urgency.as_str() {
            "critical" => SuggestionUrgency::Critical,
            "high" => SuggestionUrgency::High,
            "low" => SuggestionUrgency::Low,
            _ => SuggestionUrgency::Normal,
        }
    }

    /// Check if API usage is allowed based on privacy settings
    fn allow_api_usage(&self, request: &ExtendedInterventionRequest) -> bool {
        match request.user_preferences.privacy_level {
            crate::types::UserPrivacyLevel::LocalOnly => false,
            crate::types::UserPrivacyLevel::SanitizedAPI => self.config.privacy.allow_api_fallback,
            crate::types::UserPrivacyLevel::ConsentBased => {
                self.config.privacy.allow_api_fallback && 
                (request.user_preferences.api_consent.openai_allowed || 
                 request.user_preferences.api_consent.anthropic_allowed)
            }
        }
    }

    /// Update usage statistics
    async fn update_usage_stats(&self, method: &GenerationMethod, tokens: Option<u32>) {
        let mut stats = self.usage_stats.write().await;
        stats.requests_processed += 1;
        
        if let Some(token_count) = tokens {
            stats.total_tokens_used += token_count as u64;
        }

        match method {
            GenerationMethod::LocalLLM { .. } => stats.local_generations += 1,
            GenerationMethod::APIFallback { .. } => stats.api_generations += 1,
            GenerationMethod::Template { .. } => stats.template_responses += 1,
            GenerationMethod::Cached { .. } => stats.cached_responses += 1,
        }
    }

    // Simplified extraction methods (in practice would be more sophisticated)
    fn extract_work_context(&self, context: &serde_json::Value) -> Result<crate::types::WorkContext> {
        Ok(crate::types::WorkContext {
            work_type: crate::types::WorkType::Unknown,
            application: context.get("application")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string(),
            window_title: context.get("window_title")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string(),
            screenshot_text: context.get("screenshot_text")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            task_category: crate::types::TaskCategory::Unknown,
            urgency: crate::types::UrgencyLevel::Medium,
            time_of_day: crate::types::TimeOfDay::Afternoon,
        })
    }

    fn extract_behavioral_metrics(&self, context: &serde_json::Value) -> Result<crate::types::BehavioralMetrics> {
        Ok(crate::types::BehavioralMetrics {
            productive_time_ratio: context.get("productive_time_ratio")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.5) as f32,
            distraction_frequency: context.get("distraction_frequency")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.3) as f32,
            focus_session_count: context.get("focus_session_count")
                .and_then(|v| v.as_u64())
                .unwrap_or(1) as u32,
            average_session_length: context.get("average_session_length")
                .and_then(|v| v.as_u64())
                .unwrap_or(1800),
            recovery_time: context.get("recovery_time")
                .and_then(|v| v.as_u64())
                .unwrap_or(300),
            transition_smoothness: context.get("transition_smoothness")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.7) as f32,
        })
    }

    fn extract_current_state(&self, context: &serde_json::Value) -> Result<crate::types::ADHDState> {
        let state_type = context.get("state")
            .and_then(|v| v.as_str())
            .unwrap_or("neutral");

        let state_type = match state_type {
            "flow" => crate::types::ADHDStateType::Flow { depth: 0.7 },
            "distracted" => crate::types::ADHDStateType::Distracted { severity: 0.5 },
            "hyperfocus" => crate::types::ADHDStateType::Hyperfocus { intensity: 0.8 },
            "transitioning" => crate::types::ADHDStateType::Transitioning,
            _ => crate::types::ADHDStateType::Neutral,
        };

        Ok(crate::types::ADHDState {
            state_type,
            confidence: context.get("confidence")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.8) as f32,
            depth: context.get("depth")
                .and_then(|v| v.as_f64())
                .map(|d| d as f32),
            duration: context.get("duration")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
            metadata: std::collections::HashMap::new(),
        })
    }

    fn extract_user_preferences(&self, context: &serde_json::Value) -> Result<crate::types::UserPreferences> {
        // Use defaults with some basic extraction
        let mut preferences = crate::types::UserPreferences {
            intervention_frequency: crate::types::InterventionFrequency::Moderate,
            message_style: crate::types::MessageStyle::Encouraging,
            privacy_level: self.config.privacy.default_privacy_level.clone(),
            personality_traits: PersonalityTraits::default(),
            api_consent: crate::types::APIConsent {
                openai_allowed: false,
                anthropic_allowed: false,
                consent_timestamp: None,
                monthly_limit_usd: self.config.api_config.max_monthly_cost,
            },
        };

        // Override with context values if available
        if let Some(privacy) = context.get("privacy_level").and_then(|v| v.as_str()) {
            preferences.privacy_level = match privacy {
                "local_only" => crate::types::UserPrivacyLevel::LocalOnly,
                "sanitized_api" => crate::types::UserPrivacyLevel::SanitizedAPI,
                "consent_based" => crate::types::UserPrivacyLevel::ConsentBased,
                _ => preferences.privacy_level,
            };
        }

        Ok(preferences)
    }
}

#[async_trait::async_trait]
impl AIIntegration for AIIntegrationImpl {
    /// Process an intervention request from the gamification module
    async fn process_intervention(
        &self,
        request: InterventionRequest,
    ) -> Result<InterventionResponse> {
        if !self.initialized {
            return Err(AIIntegrationError::NotInitialized);
        }

        let start_time = std::time::Instant::now();
        
        // Extend the basic request with full context
        let extended_request = self.extend_intervention_request(request).await?;
        
        // Determine urgency and privacy settings
        let urgency = self.determine_urgency(&extended_request);
        let allow_api = self.allow_api_usage(&extended_request);

        // Build context for AI generation
        let context = self.context_processor.build_context(
            &extended_request.base.intervention_type,
            &extended_request.current_state,
            &extended_request.state_history,
            &extended_request.behavioral_metrics,
            &extended_request.work_context,
            &extended_request.user_preferences,
        ).await?;

        // Generate suggestion
        let suggestion_result = self.suggestion_generator.generate(
            context,
            urgency,
            allow_api,
        ).await?;

        // Update usage statistics
        self.update_usage_stats(&suggestion_result.method, suggestion_result.tokens_used).await;

        // Create animation cues from hints
        let animation_cues: Vec<String> = suggestion_result.animation_hints;

        // Build response
        let response = InterventionResponse {
            request_id: extended_request.base.request_id,
            response_text: suggestion_result.text,
            animation_cues,
        };

        // Log successful processing
        let processing_time = start_time.elapsed();
        log::debug!(
            "Processed intervention request {} in {:?} using {:?}",
            extended_request.base.request_id,
            processing_time,
            suggestion_result.method
        );

        Ok(response)
    }

    /// Generate animation commands based on text and mood
    async fn generate_animation(
        &self,
        text: &str,
        mood: CompanionMood,
    ) -> Result<AnimationCommand> {
        // Analyze text for animation hints
        let animation_type = if text.contains("celebration") || text.contains("amazing") || text.contains("🎉") {
            "celebration"
        } else if text.contains("break") || text.contains("rest") || text.contains("💧") {
            "sleepy"
        } else if text.contains("focus") || text.contains("concentrate") {
            "focused"
        } else {
            match mood {
                CompanionMood::Happy => "happy",
                CompanionMood::Excited => "excited",
                CompanionMood::Celebrating => "celebration",
                CompanionMood::Concerned => "concerned",
                CompanionMood::Sleepy => "sleepy",
                _ => "supportive",
            }
        };

        // Determine duration based on text length and animation type
        let base_duration = match animation_type {
            "celebration" => 3000,
            "sleepy" => 2000,
            _ => 1500,
        };

        let duration_ms = base_duration + (text.len() * 50).min(2000);

        Ok(AnimationCommand {
            command_id: Uuid::new_v4(),
            animation_type: animation_type.to_string(),
            parameters: serde_json::json!({
                "mood": mood,
                "text_length": text.len(),
                "intensity": 0.7
            }),
            duration_ms: duration_ms as u32,
        })
    }

    /// Update personality settings
    async fn update_personality(
        &self,
        traits: PersonalityTraits,
    ) -> Result<()> {
        let mut personality_engine = self.personality_engine.write().await;
        personality_engine.update_traits(traits)?;
        
        log::info!("Updated personality traits");
        Ok(())
    }

    /// Get usage statistics
    async fn get_usage_stats(&self) -> UsageStatistics {
        let stats = self.usage_stats.read().await;
        let llm_stats = self.llm_manager.get_usage_stats().await;
        
        // Calculate averages and rates
        let total_requests = stats.requests_processed;
        let avg_response_time = if total_requests > 0 {
            llm_stats.total_generation_time.as_millis() as f64 / total_requests as f64
        } else {
            0.0
        };

        let error_rate = if total_requests > 0 {
            // This would be tracked separately in a real implementation
            0.05 // 5% estimated error rate
        } else {
            0.0
        };

        UsageStatistics {
            requests_processed: stats.requests_processed,
            local_generations: stats.local_generations,
            api_generations: stats.api_generations,
            template_responses: stats.template_responses,
            cached_responses: stats.cached_responses,
            average_response_time_ms: avg_response_time,
            total_tokens_used: stats.total_tokens_used,
            total_cost_usd: 0.0, // Would be calculated from API usage
            privacy_violations_blocked: 0, // Would be tracked by privacy guardian
            uptime_percentage: 99.5, // Would be calculated from health monitoring
            error_rate,
        }
    }

    /// Check health status
    async fn health_check(&self) -> HealthStatus {
        let llm_health = self.llm_manager.health_check().await;
        let stats = self.get_usage_stats().await;

        let overall_status = if self.initialized && 
                              (llm_health.local_model_available || !llm_health.api_services.is_empty()) {
            ServiceStatus::Healthy
        } else if !self.initialized {
            ServiceStatus::Unknown
        } else {
            ServiceStatus::Degraded
        };

        let mut api_services = std::collections::HashMap::new();
        for (service, available) in llm_health.api_services {
            api_services.insert(
                service, 
                if available { ServiceStatus::Healthy } else { ServiceStatus::Unhealthy }
            );
        }

        HealthStatus {
            overall_status,
            local_model_status: if llm_health.local_model_available {
                ServiceStatus::Healthy
            } else {
                ServiceStatus::Unhealthy
            },
            api_services,
            memory_usage_mb: llm_health.local_model_memory_mb,
            response_time_p95_ms: stats.average_response_time_ms as u64,
            error_rate_1h: stats.error_rate,
            last_check: Utc::now(),
        }
    }
}

// Helper trait to extend personality config
trait PersonalityConfigExt {
    fn traits(&self) -> PersonalityTraits;
}

impl PersonalityConfigExt for crate::config::PersonalityConfig {
    fn traits(&self) -> PersonalityTraits {
        PersonalityTraits {
            cheerfulness: 0.7,
            humor: if self.pun_frequency > 0.05 { 0.6 } else { 0.4 },
            supportiveness: 0.9,
            casualness: 0.8,
            pun_frequency: self.pun_frequency,
            directness: 0.6,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AIIntegrationConfig;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_ai_integration_initialization() {
        let config = AIIntegrationConfig::privacy_focused();
        let mut ai = AIIntegrationImpl::new(config);
        
        // Should succeed even without model file (uses simulation)
        let result = ai.initialize().await;
        assert!(result.is_ok() || matches!(result, Err(AIIntegrationError::ModelNotFound)));
    }

    #[tokio::test]
    async fn test_intervention_processing() {
        let config = AIIntegrationConfig::privacy_focused();
        let ai = AIIntegrationImpl::new(config);
        // Note: Not initializing to test error handling
        
        let request = InterventionRequest {
            request_id: Uuid::new_v4(),
            intervention_type: "encouragement".to_string(),
            urgency: "normal".to_string(),
            context: serde_json::json!({
                "state": "distracted",
                "application": "vscode",
                "window_title": "main.rs - test project"
            }),
        };

        let result = ai.process_intervention(request).await;
        assert!(matches!(result, Err(AIIntegrationError::NotInitialized)));
    }

    #[tokio::test]
    async fn test_animation_generation() {
        let config = AIIntegrationConfig::default();
        let ai = AIIntegrationImpl::new(config);
        
        let animation = ai.generate_animation(
            "Amazing work! You're crushing it! 🎉",
            CompanionMood::Celebrating,
        ).await.unwrap();
        
        assert_eq!(animation.animation_type, "celebration");
        assert!(animation.duration_ms > 3000);
    }

    #[tokio::test]
    async fn test_personality_update() {
        let config = AIIntegrationConfig::default();
        let ai = AIIntegrationImpl::new(config);
        
        let new_traits = PersonalityTraits {
            cheerfulness: 0.9,
            humor: 0.8,
            supportiveness: 1.0,
            casualness: 0.6,
            pun_frequency: 0.2,
            directness: 0.5,
        };
        
        let result = ai.update_personality(new_traits).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_usage_stats() {
        let config = AIIntegrationConfig::default();
        let ai = AIIntegrationImpl::new(config);
        
        let stats = ai.get_usage_stats().await;
        assert_eq!(stats.requests_processed, 0);
        assert_eq!(stats.total_tokens_used, 0);
    }

    #[tokio::test]
    async fn test_health_check() {
        let config = AIIntegrationConfig::default();
        let ai = AIIntegrationImpl::new(config);
        
        let health = ai.health_check().await;
        // Should indicate unhealthy when not initialized
        assert!(matches!(health.overall_status, ServiceStatus::Unknown | ServiceStatus::Degraded));
    }
}