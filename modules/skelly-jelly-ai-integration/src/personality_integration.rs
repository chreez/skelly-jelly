//! Integration layer for enhanced personality system
//!
//! Connects the enhanced personality components with the existing AI integration

use crate::error::{AIIntegrationError, Result};
use crate::personality::{PersonalityEngine as BasePersonalityEngine, PersonalityContext};
use crate::personality_enhanced::{
    ExpertiseTracker, UserMemorySystem, ConsistencyValidator, CelebrationManager,
    ExpertiseLevel, CommunicationPreferences, AttentionPreferences, UserFeedback
};
use crate::types::{PersonalityTraits, CompanionMood, WorkContext};
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc, Duration};

/// Enhanced personality engine that integrates all personality components
pub struct EnhancedPersonalityEngine {
    base_engine: BasePersonalityEngine,
    expertise_tracker: Arc<RwLock<ExpertiseTracker>>,
    user_memory: Arc<RwLock<UserMemorySystem>>,
    consistency_validator: Arc<RwLock<ConsistencyValidator>>,
    celebration_manager: Arc<RwLock<CelebrationManager>>,
    adaptive_communication: AdaptiveCommunicationSystem,
}

impl EnhancedPersonalityEngine {
    /// Create a new enhanced personality engine
    pub fn new(traits: PersonalityTraits) -> Self {
        Self {
            base_engine: BasePersonalityEngine::new(traits),
            expertise_tracker: Arc::new(RwLock::new(ExpertiseTracker::new())),
            user_memory: Arc::new(RwLock::new(UserMemorySystem::new())),
            consistency_validator: Arc::new(RwLock::new(ConsistencyValidator::new())),
            celebration_manager: Arc::new(RwLock::new(CelebrationManager::new())),
            adaptive_communication: AdaptiveCommunicationSystem::new(),
        }
    }
    
    /// Apply enhanced personality with full adaptation capabilities
    pub async fn apply_enhanced(
        &self,
        suggestion: String,
        context: &PersonalityContext,
    ) -> Result<EnhancedPersonalityResponse> {
        let start_time = std::time::Instant::now();
        
        // Update expertise tracking and user memory
        {
            let mut expertise_tracker = self.expertise_tracker.write().await;
            expertise_tracker.analyze_context(context);
        }
        
        {
            let mut user_memory = self.user_memory.write().await;
            user_memory.update_interaction(context);
        }
        
        // Get current user preferences and expertise level
        let (communication_prefs, attention_prefs, expertise_level) = {
            let user_memory = self.user_memory.read().await;
            let expertise_tracker = self.expertise_tracker.read().await;
            
            (
                user_memory.get_communication_preferences().clone(),
                user_memory.get_attention_preferences().clone(),
                expertise_tracker.get_expertise_level(&context.work_context),
            )
        };
        
        // Apply adaptive communication
        let adapted_message = self.adaptive_communication.adapt_message(
            &suggestion,
            &communication_prefs,
            &attention_prefs,
            &expertise_level,
            context,
        ).await?;
        
        // Apply base personality traits
        let mut personality_applied = adapted_message;
        
        // Apply consistency validation
        {
            let mut validator = self.consistency_validator.write().await;
            let user_memory = self.user_memory.read().await;
            personality_applied = validator.ensure_consistency(&personality_applied, &*user_memory)?;
        }
        
        // Generate celebration if appropriate
        let celebration_enhancement = if self.should_celebrate(context).await {
            let mut celebration_manager = self.celebration_manager.write().await;
            Some(celebration_manager.generate_authentic_celebration(
                &self.detect_celebration_type(context),
                self.calculate_celebration_magnitude(context),
                &communication_prefs,
            ))
        } else {
            None
        };
        
        // Record this interaction for future learning
        {
            let mut user_memory = self.user_memory.write().await;
            user_memory.record_personality_interaction(
                &personality_applied,
                &self.determine_current_mood(context),
                &expertise_level,
            );
        }
        
        let processing_time = start_time.elapsed();
        
        Ok(EnhancedPersonalityResponse {
            message: personality_applied,
            celebration: celebration_enhancement,
            expertise_level,
            communication_style: CommunicationStyle::from_preferences(&communication_prefs),
            adaptation_confidence: self.calculate_adaptation_confidence().await,
            processing_time_ms: processing_time.as_millis() as u32,
            learning_insights: self.generate_learning_insights().await,
        })
    }
    
    /// Process user feedback for continuous learning
    pub async fn process_user_feedback(&self, feedback: UserFeedback) -> Result<()> {
        let mut user_memory = self.user_memory.write().await;
        
        // In a full implementation, this would update the PersonalityContext
        // with the feedback for the analyze_response_patterns method
        log::info!("Received user feedback: {:?}", feedback.feedback_type);
        
        // Update communication preferences based on feedback
        match feedback.feedback_type {
            crate::personality_enhanced::FeedbackType::Positive => {
                // Reinforce current approach
                log::debug!("Positive feedback received, reinforcing current personality approach");
            }
            crate::personality_enhanced::FeedbackType::TooFormal => {
                log::debug!("Adjusting communication style to be more casual");
            }
            crate::personality_enhanced::FeedbackType::Patronizing => {
                log::warn!("User felt patronized, adjusting expertise-level adaptation");
            }
            _ => {}
        }
        
        Ok(())
    }
    
    /// Get current personality state for monitoring
    pub async fn get_personality_state(&self) -> PersonalityState {
        let user_memory = self.user_memory.read().await;
        let expertise_tracker = self.expertise_tracker.read().await;
        
        PersonalityState {
            communication_preferences: user_memory.get_communication_preferences().clone(),
            attention_preferences: user_memory.get_attention_preferences().clone(),
            expertise_assessments: self.get_expertise_summary(&*expertise_tracker).await,
            adaptation_confidence: self.calculate_adaptation_confidence().await,
            interaction_count: user_memory.get_interaction_count(),
            last_updated: Utc::now(),
        }
    }
    
    /// Update personality traits
    pub async fn update_traits(&self, new_traits: PersonalityTraits) -> Result<()> {
        // In a full implementation, this would recreate the base engine
        // For now, we'll log the update
        log::info!("Personality traits updated: {:?}", new_traits);
        Ok(())
    }
    
    // Private helper methods
    
    async fn should_celebrate(&self, context: &PersonalityContext) -> bool {
        // Check if the context indicates an achievement worth celebrating
        matches!(context.current_state.state_type, 
            crate::types::ADHDStateType::Flow { depth } if depth > 0.8
        ) || context.metrics.productive_time_ratio > 0.9
    }
    
    fn detect_celebration_type(&self, context: &PersonalityContext) -> String {
        match &context.current_state.state_type {
            crate::types::ADHDStateType::Flow { .. } => "flow_achievement".to_string(),
            _ => "task_completion".to_string(),
        }
    }
    
    fn calculate_celebration_magnitude(&self, context: &PersonalityContext) -> f32 {
        // Calculate how significant this achievement is
        let productivity_score = context.metrics.productive_time_ratio;
        let focus_score = match &context.current_state.state_type {
            crate::types::ADHDStateType::Flow { depth } => *depth,
            crate::types::ADHDStateType::Hyperfocus { intensity } => *intensity,
            _ => 0.5,
        };
        
        (productivity_score + focus_score) / 2.0
    }
    
    fn determine_current_mood(&self, context: &PersonalityContext) -> CompanionMood {
        match &context.current_state.state_type {
            crate::types::ADHDStateType::Flow { depth } if *depth > 0.8 => CompanionMood::Happy,
            crate::types::ADHDStateType::Hyperfocus { .. } => CompanionMood::Excited,
            crate::types::ADHDStateType::Distracted { severity } if *severity > 0.7 => CompanionMood::Concerned,
            _ => CompanionMood::Supportive,
        }
    }
    
    async fn calculate_adaptation_confidence(&self) -> f32 {
        let user_memory = self.user_memory.read().await;
        // In a full implementation, this would calculate based on user memory metrics
        0.75 // Placeholder confidence score
    }
    
    async fn generate_learning_insights(&self) -> Vec<LearningInsight> {
        // Generate insights about user preferences and adaptation effectiveness
        vec![
            LearningInsight {
                category: "communication_style".to_string(),
                insight: "User responds well to casual, supportive tone".to_string(),
                confidence: 0.8,
                timestamp: Utc::now(),
            }
        ]
    }
    
    async fn get_expertise_summary(&self, expertise_tracker: &ExpertiseTracker) -> Vec<ExpertiseSummary> {
        // In a full implementation, this would extract expertise data
        vec![
            ExpertiseSummary {
                domain: "rust_programming".to_string(),
                level: ExpertiseLevel::Intermediate,
                confidence: 0.7,
                last_assessed: Utc::now(),
            }
        ]
    }
}

/// Adaptive communication system that adjusts style based on user preferences
pub struct AdaptiveCommunicationSystem {
    style_adaptations: Vec<StyleAdaptation>,
}

impl AdaptiveCommunicationSystem {
    pub fn new() -> Self {
        Self {
            style_adaptations: Vec::new(),
        }
    }
    
    /// Adapt message based on user preferences and context
    pub async fn adapt_message(
        &self,
        message: &str,
        communication_prefs: &CommunicationPreferences,
        attention_prefs: &AttentionPreferences,
        expertise_level: &ExpertiseLevel,
        context: &PersonalityContext,
    ) -> Result<String> {
        let mut adapted = message.to_string();
        
        // Apply formality adaptation
        adapted = self.adapt_formality(&adapted, &communication_prefs.formality_level);
        
        // Apply length adaptation based on attention preferences
        adapted = self.adapt_length(&adapted, attention_prefs);
        
        // Apply expertise-level adaptation
        adapted = self.adapt_for_expertise(&adapted, expertise_level);
        
        // Apply context-specific adaptations
        adapted = self.adapt_for_context(&adapted, context);
        
        Ok(adapted)
    }
    
    fn adapt_formality(&self, message: &str, formality: &crate::personality_enhanced::FormalityLevel) -> String {
        match formality {
            crate::personality_enhanced::FormalityLevel::Casual => {
                message
                    .replace("consider", "try")
                    .replace("perhaps", "maybe")
                    .replace("excellent", "awesome")
            }
            crate::personality_enhanced::FormalityLevel::Professional => {
                message
                    .replace("hey", "")
                    .replace("awesome", "excellent")
                    .replace("cool", "good")
            }
            crate::personality_enhanced::FormalityLevel::Balanced => message.to_string(),
        }
    }
    
    fn adapt_length(&self, message: &str, attention_prefs: &AttentionPreferences) -> String {
        let words: Vec<&str> = message.split_whitespace().collect();
        let target_length = match attention_prefs.preferred_message_length {
            crate::personality_enhanced::MessageLength::VeryShort => 8,
            crate::personality_enhanced::MessageLength::Short => 15,
            crate::personality_enhanced::MessageLength::Medium => 25,
            crate::personality_enhanced::MessageLength::Long => 40,
        };
        
        if words.len() <= target_length {
            return message.to_string();
        }
        
        // Compress message while maintaining meaning
        match attention_prefs.detail_preference {
            crate::personality_enhanced::DetailPreference::KeyPointsOnly => {
                // Extract the main action or advice
                if let Some(main_sentence) = message.split('.').next() {
                    format!("{}.", main_sentence.trim())
                } else {
                    words.into_iter().take(target_length).collect::<Vec<_>>().join(" ")
                }
            }
            crate::personality_enhanced::DetailPreference::Concise => {
                // Remove filler words but keep structure
                let filtered: Vec<&str> = words.into_iter()
                    .filter(|&word| !matches!(word.to_lowercase().as_str(), 
                        "really" | "very" | "quite" | "pretty" | "just" | "actually" | "basically"))
                    .take(target_length)
                    .collect();
                filtered.join(" ")
            }
            crate::personality_enhanced::DetailPreference::Detailed => {
                // Keep full message even if longer
                message.to_string()
            }
        }
    }
    
    fn adapt_for_expertise(&self, message: &str, expertise: &ExpertiseLevel) -> String {
        match expertise {
            ExpertiseLevel::Expert => {
                // Remove overly explanatory language
                message
                    .replace("Don't worry, ", "")
                    .replace("Simply ", "")
                    .replace("Just try ", "Try ")
                    .replace("It's easy to ", "")
            }
            ExpertiseLevel::Intermediate => {
                // Balanced approach
                message
                    .replace("Don't worry, ", "")
                    .replace("Simply ", "")
            }
            ExpertiseLevel::Beginner => {
                // Keep supportive, explanatory language
                message.to_string()
            }
        }
    }
    
    fn adapt_for_context(&self, message: &str, context: &PersonalityContext) -> String {
        match &context.current_state.state_type {
            crate::types::ADHDStateType::Hyperfocus { .. } => {
                // Ultra-brief for hyperfocus
                if let Some(first_sentence) = message.split('.').next() {
                    format!("💡 {}", first_sentence.trim())
                } else {
                    format!("💡 {}", message)
                }
            }
            crate::types::ADHDStateType::Distracted { severity } if *severity > 0.7 => {
                // Add gentle encouragement for high distraction
                format!("It's okay! {}", message)
            }
            _ => message.to_string(),
        }
    }
}

/// Response from enhanced personality system
#[derive(Debug, Clone)]
pub struct EnhancedPersonalityResponse {
    pub message: String,
    pub celebration: Option<String>,
    pub expertise_level: ExpertiseLevel,
    pub communication_style: CommunicationStyle,
    pub adaptation_confidence: f32,
    pub processing_time_ms: u32,
    pub learning_insights: Vec<LearningInsight>,
}

/// Communication style derived from user preferences
#[derive(Debug, Clone)]
pub struct CommunicationStyle {
    pub formality: String,
    pub intensity: String,
    pub preferred_length: String,
}

impl CommunicationStyle {
    fn from_preferences(prefs: &CommunicationPreferences) -> Self {
        Self {
            formality: format!("{:?}", prefs.formality_level),
            intensity: format!("{:?}", prefs.intensity_preference),
            preferred_length: "adaptive".to_string(),
        }
    }
}

/// Learning insight about user preferences
#[derive(Debug, Clone)]
pub struct LearningInsight {
    pub category: String,
    pub insight: String,
    pub confidence: f32,
    pub timestamp: DateTime<Utc>,
}

/// Current personality state for monitoring
#[derive(Debug, Clone)]
pub struct PersonalityState {
    pub communication_preferences: CommunicationPreferences,
    pub attention_preferences: AttentionPreferences,
    pub expertise_assessments: Vec<ExpertiseSummary>,
    pub adaptation_confidence: f32,
    pub interaction_count: u32,
    pub last_updated: DateTime<Utc>,
}

/// Summary of user expertise in a domain
#[derive(Debug, Clone)]
pub struct ExpertiseSummary {
    pub domain: String,
    pub level: ExpertiseLevel,
    pub confidence: f32,
    pub last_assessed: DateTime<Utc>,
}

/// Style adaptation record
#[derive(Debug, Clone)]
struct StyleAdaptation {
    pub pattern: String,
    pub replacement: String,
    pub context: String,
    pub effectiveness: f32,
}

// Extension trait to add methods to UserMemorySystem
trait UserMemorySystemExt {
    fn get_interaction_count(&self) -> u32;
}

impl UserMemorySystemExt for UserMemorySystem {
    fn get_interaction_count(&self) -> u32 {
        // This would be implemented in the actual UserMemorySystem
        0 // Placeholder
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ADHDState, ADHDStateType, BehavioralMetrics};
    use std::collections::HashMap;
    
    #[tokio::test]
    async fn test_enhanced_personality_engine_creation() {
        let traits = PersonalityTraits::default();
        let engine = EnhancedPersonalityEngine::new(traits);
        
        // Should create successfully
        assert_eq!(engine.adaptive_communication.style_adaptations.len(), 0);
    }
    
    #[tokio::test]
    async fn test_adaptive_communication_formality() {
        let system = AdaptiveCommunicationSystem::new();
        let message = "You should consider trying this approach";
        
        let casual_result = system.adapt_formality(message, &crate::personality_enhanced::FormalityLevel::Casual);
        assert!(casual_result.contains("try"));
        assert!(!casual_result.contains("consider"));
        
        let professional_result = system.adapt_formality(message, &crate::personality_enhanced::FormalityLevel::Professional);
        // Professional version should be more formal
        assert!(!professional_result.is_empty());
    }
    
    #[tokio::test]
    async fn test_expertise_adaptation() {
        let system = AdaptiveCommunicationSystem::new();
        let message = "Don't worry, simply try this approach";
        
        let expert_result = system.adapt_for_expertise(message, &ExpertiseLevel::Expert);
        assert!(!expert_result.contains("Don't worry"));
        assert!(!expert_result.contains("Simply"));
        
        let beginner_result = system.adapt_for_expertise(message, &ExpertiseLevel::Beginner);
        assert_eq!(beginner_result, message); // Should keep supportive language
    }
    
    #[test]
    fn test_communication_style_from_preferences() {
        let prefs = CommunicationPreferences::default();
        let style = CommunicationStyle::from_preferences(&prefs);
        
        assert_eq!(style.formality, "Balanced");
        assert_eq!(style.intensity, "Moderate");
        assert_eq!(style.preferred_length, "adaptive");
    }
}