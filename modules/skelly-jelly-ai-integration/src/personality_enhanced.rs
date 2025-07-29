//! Enhanced personality system components
//! 
//! Provides expertise-level adaptation, user memory, and authentic celebration management

use crate::error::{AIIntegrationError, Result};
use crate::types::{PersonalityTraits, ADHDState, BehavioralMetrics, CompanionMood, WorkContext, WorkType};
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};

/// User expertise level in different domains
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExpertiseLevel {
    Beginner,
    Intermediate, 
    Expert,
}

/// User's communication preferences learned over time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunicationPreferences {
    pub intensity_preference: IntensityPreference,
    pub formality_level: FormalityLevel,
    pub humor_appreciation: f32, // 0.0-1.0 based on response to jokes/puns
    pub encouragement_preferred: bool,
    pub celebrations_preferred: bool,
    pub emojis_preferred: bool,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IntensityPreference {
    Subtle,
    Moderate,
    Energetic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FormalityLevel {
    Casual,
    Balanced,
    Professional,
}

/// User's attention and processing preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttentionPreferences {
    pub preferred_message_length: MessageLength,
    pub detail_preference: DetailPreference,
    pub processing_speed: ProcessingSpeed,
    pub context_switching_comfort: f32, // 0.0-1.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageLength {
    VeryShort, // 5-8 words
    Short,     // 10-15 words  
    Medium,    // 20-25 words
    Long,      // 30-40 words
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DetailPreference {
    KeyPointsOnly,
    Concise,
    Detailed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProcessingSpeed {
    Slow,
    Moderate,
    Fast,
}

/// User feedback on personality interactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserFeedback {
    pub interaction_id: String,
    pub feedback_type: FeedbackType,
    pub rating: Option<f32>, // 1.0-5.0 if provided
    pub timestamp: DateTime<Utc>,
    pub context: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FeedbackType {
    Positive,      // User appreciated the interaction
    Negative,      // User didn't like something
    TooFormal,     // User found language too professional
    TooInformal,   // User wanted more professional tone
    TooLong,       // Message was too lengthy
    TooShort,      // Message lacked detail
    Patronizing,   // User felt talked down to
    Helpful,       // User found it genuinely helpful
    Motivating,    // User felt motivated
    Ignored,       // User dismissed without reading
}

/// Tracks user expertise levels in different domains
pub struct ExpertiseTracker {
    domain_assessments: HashMap<String, ExpertiseAssessment>,
    confidence_threshold: f32,
}

#[derive(Debug, Clone)]
struct ExpertiseAssessment {
    level: ExpertiseLevel,
    confidence: f32,
    evidence_count: u32,
    last_updated: DateTime<Utc>,
}

impl ExpertiseTracker {
    pub fn new() -> Self {
        Self {
            domain_assessments: HashMap::new(),
            confidence_threshold: 0.7,
        }
    }
    
    /// Analyze context to assess user expertise
    pub fn analyze_context(&mut self, context: &PersonalityContext) {
        let domain = self.extract_domain(&context.work_context);
        let indicators = self.extract_expertise_indicators(context);
        
        let current_assessment = self.domain_assessments
            .entry(domain.clone())
            .or_insert_with(|| ExpertiseAssessment {
                level: ExpertiseLevel::Beginner,
                confidence: 0.1,
                evidence_count: 0,
                last_updated: Utc::now(),
            });
            
        // Update assessment based on new evidence
        self.update_assessment(current_assessment, &indicators);
    }
    
    /// Get current expertise level for a work context
    pub fn get_expertise_level(&self, work_context: &WorkContext) -> ExpertiseLevel {
        let domain = self.extract_domain(work_context);
        
        self.domain_assessments
            .get(&domain)
            .map(|assessment| {
                if assessment.confidence >= self.confidence_threshold {
                    assessment.level.clone()
                } else {
                    ExpertiseLevel::Beginner // Default to beginner if not confident
                }
            })
            .unwrap_or(ExpertiseLevel::Beginner)
    }
    
    fn extract_domain(&self, work_context: &WorkContext) -> String {
        match &work_context.work_type {
            WorkType::Coding { language, .. } => format!("coding_{}", language),
            WorkType::Writing { document_type } => format!("writing_{}", document_type),
            WorkType::Design { tool, .. } => format!("design_{}", tool),
            WorkType::Research { topic } => format!("research_{}", topic),
            WorkType::Communication { platform } => format!("communication_{}", platform),
            WorkType::Unknown => "general".to_string(),
        }
    }
    
    fn extract_expertise_indicators(&self, context: &PersonalityContext) -> ExpertiseIndicators {
        ExpertiseIndicators {
            terminology_sophistication: self.assess_terminology(&context.work_context),
            task_complexity: self.assess_task_complexity(&context.current_state, &context.metrics),
            efficiency_patterns: self.assess_efficiency(&context.metrics),
            error_recovery: self.assess_error_recovery(context),
            help_seeking_behavior: self.assess_help_seeking(&context.recent_interactions),
        }
    }
    
    fn assess_terminology(&self, work_context: &WorkContext) -> f32 {
        // Analyze screenshot text for domain-specific terminology
        if let Some(text) = &work_context.screenshot_text {
            let advanced_terms = match &work_context.work_type {
                WorkType::Coding { language, .. } => {
                    match language.as_str() {
                        "rust" => vec!["trait", "impl", "lifetime", "borrow", "async", "unsafe"],
                        "javascript" => vec!["async", "await", "closure", "prototype", "destructuring"],
                        "python" => vec!["decorator", "generator", "comprehension", "metaclass"],
                        _ => vec!["function", "variable", "loop", "condition"],
                    }
                }
                WorkType::Design { .. } => vec!["composition", "hierarchy", "typography", "grid"],
                _ => vec![],
            };
            
            let found_terms = advanced_terms.iter()
                .filter(|term| text.to_lowercase().contains(&term.to_lowercase()))
                .count();
                
            (found_terms as f32 / advanced_terms.len().max(1) as f32).min(1.0)
        } else {
            0.5 // Default neutral score
        }
    }
    
    fn assess_task_complexity(&self, state: &ADHDState, metrics: &BehavioralMetrics) -> f32 {
        // Higher complexity tasks suggest higher expertise
        let state_complexity = match &state.state_type {
            crate::types::ADHDStateType::Flow { depth } => *depth * 0.8,
            crate::types::ADHDStateType::Hyperfocus { intensity } => *intensity * 0.9,
            _ => 0.3,
        };
        
        let efficiency_score = metrics.productive_time_ratio * 0.7 + 
                              (1.0 - metrics.distraction_frequency) * 0.3;
                              
        (state_complexity + efficiency_score) / 2.0
    }
    
    fn assess_efficiency(&self, metrics: &BehavioralMetrics) -> f32 {
        // Fast task completion and low distraction suggests expertise
        let session_quality = metrics.productive_time_ratio;
        let focus_consistency = 1.0 - metrics.distraction_frequency;
        let recovery_speed = if metrics.recovery_time > 0 {
            (300.0 / metrics.recovery_time as f32).min(1.0)
        } else {
            1.0
        };
        
        (session_quality + focus_consistency + recovery_speed) / 3.0
    }
    
    fn assess_error_recovery(&self, context: &PersonalityContext) -> f32 {
        // Quick recovery from distractions suggests experience
        match &context.current_state.state_type {
            crate::types::ADHDStateType::Transitioning => {
                if context.current_state.duration < 30000 { // Quick transition
                    0.8
                } else {
                    0.4
                }
            }
            _ => 0.5,
        }
    }
    
    fn assess_help_seeking(&self, interactions: &[InteractionHistory]) -> f32 {
        if interactions.is_empty() {
            return 0.5;
        }
        
        // Experts typically ask more specific questions
        let specific_questions = interactions.iter()
            .filter(|interaction| {
                interaction.message.contains("how") || 
                interaction.message.contains("why") ||
                interaction.message.contains("specific")
            })
            .count();
            
        (specific_questions as f32 / interactions.len() as f32).min(1.0)
    }
    
    fn update_assessment(&mut self, assessment: &mut ExpertiseAssessment, indicators: &ExpertiseIndicators) {
        let new_evidence_score = (
            indicators.terminology_sophistication * 0.25 +
            indicators.task_complexity * 0.25 +
            indicators.efficiency_patterns * 0.25 +
            indicators.error_recovery * 0.15 +
            indicators.help_seeking_behavior * 0.10
        );
        
        // Update confidence and level
        assessment.evidence_count += 1;
        let learning_rate = 0.1; // How quickly to adapt
        assessment.confidence = assessment.confidence * (1.0 - learning_rate) + 
                               new_evidence_score * learning_rate;
        
        // Update expertise level based on accumulated evidence
        assessment.level = if assessment.confidence >= 0.8 {
            ExpertiseLevel::Expert
        } else if assessment.confidence >= 0.5 {
            ExpertiseLevel::Intermediate
        } else {
            ExpertiseLevel::Beginner
        };
        
        assessment.last_updated = Utc::now();
    }
}

#[derive(Debug)]
struct ExpertiseIndicators {
    terminology_sophistication: f32,
    task_complexity: f32,
    efficiency_patterns: f32,
    error_recovery: f32,
    help_seeking_behavior: f32,
}

/// Comprehensive user memory and preference learning system
pub struct UserMemorySystem {
    communication_preferences: CommunicationPreferences,
    attention_preferences: AttentionPreferences,
    interaction_history: Vec<PersonalityInteraction>,
    preference_confidence: HashMap<String, f32>,
    adaptation_metrics: AdaptationMetrics,
}

#[derive(Debug, Clone)]
struct PersonalityInteraction {
    message: String,
    mood: CompanionMood,
    expertise_level: ExpertiseLevel,
    user_response_time: Option<Duration>,
    user_engagement: EngagementLevel,
    timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone)]
enum EngagementLevel {
    Ignored,      // No response or quick dismissal
    Acknowledged, // Brief acknowledgment
    Engaged,      // Active response or follow-up
    Appreciated,  // Positive feedback or extended interaction
}

#[derive(Debug, Clone)]
struct AdaptationMetrics {
    successful_interactions: u32,
    total_interactions: u32,
    preference_stability: f32,
    last_major_adjustment: DateTime<Utc>,
}

impl UserMemorySystem {
    pub fn new() -> Self {
        Self {
            communication_preferences: CommunicationPreferences::default(),
            attention_preferences: AttentionPreferences::default(),
            interaction_history: Vec::new(),
            preference_confidence: HashMap::new(),
            adaptation_metrics: AdaptationMetrics {
                successful_interactions: 0,
                total_interactions: 0,
                preference_stability: 0.5,
                last_major_adjustment: Utc::now(),
            },
        }
    }
    
    /// Update memory based on new interaction context
    pub fn update_interaction(&mut self, context: &PersonalityContext) {
        // Analyze user behavior patterns from context
        self.analyze_attention_patterns(context);
        self.analyze_response_patterns(context);
        self.update_preference_confidence();
    }
    
    /// Record personality interaction for learning
    pub fn record_personality_interaction(
        &mut self,
        message: &str,
        mood: &CompanionMood,
        expertise_level: &ExpertiseLevel,
    ) {
        let interaction = PersonalityInteraction {
            message: message.to_string(),
            mood: mood.clone(),
            expertise_level: expertise_level.clone(),
            user_response_time: None, // Would be filled by UI feedback
            user_engagement: EngagementLevel::Acknowledged, // Default
            timestamp: Utc::now(),
        };
        
        self.interaction_history.push(interaction);
        self.adaptation_metrics.total_interactions += 1;
        
        // Keep history manageable
        if self.interaction_history.len() > 1000 {
            self.interaction_history.drain(0..200); // Remove oldest 200
        }
    }
    
    /// Get current communication preferences
    pub fn get_communication_preferences(&self) -> &CommunicationPreferences {
        &self.communication_preferences
    }
    
    /// Get current attention preferences
    pub fn get_attention_preferences(&self) -> &AttentionPreferences {
        &self.attention_preferences
    }
    
    /// Get user's preferred energy level based on response patterns
    pub fn get_preferred_energy_level(&self) -> f32 {
        // Analyze recent interactions to determine preferred energy level
        let recent_interactions: Vec<_> = self.interaction_history
            .iter()
            .rev()
            .take(20)
            .collect();
            
        if recent_interactions.is_empty() {
            return 0.6; // Default moderate energy
        }
        
        // Count positive responses to different energy levels
        let high_energy_success = recent_interactions.iter()
            .filter(|i| matches!(i.mood, CompanionMood::Excited | CompanionMood::Celebrating))
            .filter(|i| matches!(i.user_engagement, EngagementLevel::Engaged | EngagementLevel::Appreciated))
            .count();
            
        let calm_energy_success = recent_interactions.iter()
            .filter(|i| matches!(i.mood, CompanionMood::Supportive | CompanionMood::Neutral))
            .filter(|i| matches!(i.user_engagement, EngagementLevel::Engaged | EngagementLevel::Appreciated))
            .count();
            
        if high_energy_success > calm_energy_success {
            0.8 // User prefers higher energy
        } else if calm_energy_success > high_energy_success {
            0.4 // User prefers calmer interactions
        } else {
            0.6 // Balanced preference
        }
    }
    
    fn analyze_attention_patterns(&mut self, context: &PersonalityContext) {
        // Adjust message length preference based on session duration and state
        match &context.current_state.state_type {
            crate::types::ADHDStateType::Hyperfocus { .. } => {
                // In hyperfocus, user likely prefers very brief messages
                if matches!(self.attention_preferences.preferred_message_length, MessageLength::Medium | MessageLength::Long) {
                    self.attention_preferences.preferred_message_length = MessageLength::Short;
                }
            }
            crate::types::ADHDStateType::Distracted { severity } if *severity > 0.7 => {
                // When highly distracted, prefer even shorter messages
                self.attention_preferences.preferred_message_length = MessageLength::VeryShort;
            }
            crate::types::ADHDStateType::Flow { .. } => {
                // In flow state, slightly longer messages might be okay
                if matches!(self.attention_preferences.preferred_message_length, MessageLength::VeryShort) {
                    self.attention_preferences.preferred_message_length = MessageLength::Short;
                }
            }
            _ => {}
        }
        
        // Adjust processing speed based on metrics
        if context.metrics.transition_smoothness < 0.3 {
            self.attention_preferences.processing_speed = ProcessingSpeed::Slow;
        } else if context.metrics.transition_smoothness > 0.8 {
            self.attention_preferences.processing_speed = ProcessingSpeed::Fast;
        }
    }
    
    fn analyze_response_patterns(&mut self, context: &PersonalityContext) {
        // Analyze user feedback history to adjust communication preferences
        let recent_feedback: Vec<_> = context.user_feedback_history
            .iter()
            .rev()
            .take(10)
            .collect();
            
        for feedback in recent_feedback {
            match feedback.feedback_type {
                FeedbackType::TooFormal => {
                    if matches!(self.communication_preferences.formality_level, FormalityLevel::Professional | FormalityLevel::Balanced) {
                        self.communication_preferences.formality_level = FormalityLevel::Casual;
                    }
                }
                FeedbackType::TooInformal => {
                    if matches!(self.communication_preferences.formality_level, FormalityLevel::Casual) {
                        self.communication_preferences.formality_level = FormalityLevel::Balanced;
                    }
                }
                FeedbackType::TooLong => {
                    self.attention_preferences.preferred_message_length = match self.attention_preferences.preferred_message_length {
                        MessageLength::Long => MessageLength::Medium,
                        MessageLength::Medium => MessageLength::Short,
                        MessageLength::Short => MessageLength::VeryShort,
                        MessageLength::VeryShort => MessageLength::VeryShort,
                    };
                }
                FeedbackType::TooShort => {
                    self.attention_preferences.preferred_message_length = match self.attention_preferences.preferred_message_length {
                        MessageLength::VeryShort => MessageLength::Short,
                        MessageLength::Short => MessageLength::Medium,
                        MessageLength::Medium => MessageLength::Long,
                        MessageLength::Long => MessageLength::Long,
                    };
                }
                FeedbackType::Patronizing => {
                    // User feels talked down to - adjust formality upward
                    self.communication_preferences.formality_level = match self.communication_preferences.formality_level {
                        FormalityLevel::Casual => FormalityLevel::Balanced,
                        FormalityLevel::Balanced => FormalityLevel::Professional,
                        FormalityLevel::Professional => FormalityLevel::Professional,
                    };
                }
                FeedbackType::Positive | FeedbackType::Helpful | FeedbackType::Motivating => {
                    self.adaptation_metrics.successful_interactions += 1;
                }
                _ => {}
            }
        }
        
        self.communication_preferences.last_updated = Utc::now();
    }
    
    fn update_preference_confidence(&mut self) {
        // Calculate confidence in current preferences based on consistency
        let total = self.adaptation_metrics.total_interactions;
        let successful = self.adaptation_metrics.successful_interactions;
        
        if total > 0 {
            let success_rate = successful as f32 / total as f32;
            self.adaptation_metrics.preference_stability = success_rate;
        }
    }
}

/// Validates personality consistency across interactions
pub struct ConsistencyValidator {
    recent_responses: Vec<String>,
    personality_signature: PersonalitySignature,
}

#[derive(Debug, Clone)]
struct PersonalitySignature {
    typical_phrase_patterns: Vec<String>,
    tone_consistency: f32,
    formality_range: (f32, f32),
    humor_frequency: f32,
}

impl ConsistencyValidator {
    pub fn new() -> Self {
        Self {
            recent_responses: Vec::new(),
            personality_signature: PersonalitySignature {
                typical_phrase_patterns: vec![
                    "You've got this".to_string(),
                    "Nice work".to_string(),
                    "Keep going".to_string(),
                ],
                tone_consistency: 0.8,
                formality_range: (0.3, 0.7),
                humor_frequency: 0.1,
            },
        }
    }
    
    /// Ensure response is consistent with established personality
    pub fn ensure_consistency(&mut self, message: &str, memory: &UserMemorySystem) -> Result<String> {
        // Check against recent responses for variety
        if self.recent_responses.iter().any(|prev| self.too_similar(prev, message)) {
            // Generate slight variation to avoid repetition
            return Ok(self.add_variation(message));
        }
        
        // Validate tone consistency
        let validated = self.validate_tone_consistency(message, memory)?;
        
        // Store for future consistency checks
        self.recent_responses.push(validated.clone());
        if self.recent_responses.len() > 10 {
            self.recent_responses.remove(0);
        }
        
        Ok(validated)
    }
    
    fn too_similar(&self, prev: &str, current: &str) -> bool {
        // Simple similarity check - in practice would use more sophisticated NLP
        let prev_words: std::collections::HashSet<&str> = prev.split_whitespace().collect();
        let current_words: std::collections::HashSet<&str> = current.split_whitespace().collect();
        
        let intersection_size = prev_words.intersection(&current_words).count();
        let union_size = prev_words.union(&current_words).count();
        
        if union_size == 0 {
            return false;
        }
        
        let similarity = intersection_size as f32 / union_size as f32;
        similarity > 0.8 // 80% word overlap considered too similar
    }
    
    fn add_variation(&self, message: &str) -> String {
        let variations = vec![
            ("You've got this", "You can handle this"),
            ("Nice work", "Great job"),
            ("Keep going", "Keep it up"),
            ("Try", "Consider"),
            ("Good", "Excellent"),
        ];
        
        let mut varied = message.to_string();
        for (original, replacement) in variations {
            if varied.contains(original) {
                varied = varied.replace(original, replacement);
                break;
            }
        }
        
        varied
    }
    
    fn validate_tone_consistency(&self, message: &str, memory: &UserMemorySystem) -> Result<String> {
        // Ensure message matches user's expected tone based on their preferences
        let preferences = memory.get_communication_preferences();
        
        // Check formality level consistency
        let message_formality = self.assess_formality(message);
        let expected_formality = match preferences.formality_level {
            FormalityLevel::Casual => 0.3,
            FormalityLevel::Balanced => 0.5,
            FormalityLevel::Professional => 0.8,
        };
        
        if (message_formality - expected_formality).abs() > 0.3 {
            // Adjust tone to match expectations
            return Ok(self.adjust_formality(message, expected_formality));
        }
        
        Ok(message.to_string())
    }
    
    fn assess_formality(&self, message: &str) -> f32 {
        let formal_indicators = ["consider", "perhaps", "might", "would", "could"];
        let casual_indicators = ["hey", "cool", "awesome", "nice", "great"];
        
        let formal_count = formal_indicators.iter()
            .filter(|&&word| message.to_lowercase().contains(word))
            .count();
            
        let casual_count = casual_indicators.iter()
            .filter(|&&word| message.to_lowercase().contains(word))
            .count();
            
        if formal_count + casual_count == 0 {
            return 0.5; // Neutral
        }
        
        formal_count as f32 / (formal_count + casual_count) as f32
    }
    
    fn adjust_formality(&self, message: &str, target_formality: f32) -> String {
        if target_formality > 0.7 {
            // Make more formal
            message
                .replace("hey", "")
                .replace("awesome", "excellent")
                .replace("cool", "good")
                .replace("nice", "well done")
        } else if target_formality < 0.4 {
            // Make more casual
            message
                .replace("consider", "try")
                .replace("perhaps", "maybe")
                .replace("excellent", "awesome")
        } else {
            message.to_string()
        }
    }
}

/// Manages authentic celebrations without patronizing language
pub struct CelebrationManager {
    celebration_history: Vec<CelebrationEvent>,
    authenticity_patterns: Vec<AuthenticityPattern>,
}

#[derive(Debug, Clone)]
struct CelebrationEvent {
    achievement_type: String,
    celebration_intensity: f32,
    user_response: Option<EngagementLevel>,
    timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone)]
struct AuthenticityPattern {
    trigger: String,
    authentic_responses: Vec<String>,
    avoid_patterns: Vec<String>,
}

impl CelebrationManager {
    pub fn new() -> Self {
        let authenticity_patterns = vec![
            AuthenticityPattern {
                trigger: "flow_achievement".to_string(),
                authentic_responses: vec![
                    "That's solid focus right there".to_string(),
                    "You found your groove".to_string(),
                    "Nice flow state".to_string(),
                ],
                avoid_patterns: vec![
                    "Amazing job!".to_string(),
                    "You're incredible!".to_string(),
                    "Super duper!".to_string(),
                ],
            },
            AuthenticityPattern {
                trigger: "task_completion".to_string(),
                authentic_responses: vec![
                    "Task done".to_string(),
                    "One down".to_string(),
                    "Good work".to_string(),
                    "That's wrapped up".to_string(),
                ],
                avoid_patterns: vec![
                    "AMAZING WORK!!!".to_string(),
                    "You're the best!".to_string(),
                    "Perfect job!".to_string(),
                ],
            },
        ];
        
        Self {
            celebration_history: Vec::new(),
            authenticity_patterns,
        }
    }
    
    /// Generate authentic celebration that matches the achievement level
    pub fn generate_authentic_celebration(
        &mut self,
        achievement_type: &str,
        magnitude: f32,
        user_preferences: &CommunicationPreferences,
    ) -> String {
        // Find appropriate patterns for this achievement type
        let pattern = self.authenticity_patterns
            .iter()
            .find(|p| p.trigger == achievement_type)
            .or_else(|| self.authenticity_patterns.first());
            
        if let Some(pattern) = pattern {
            let mut suitable_responses: Vec<_> = pattern.authentic_responses
                .iter()
                .filter(|response| self.matches_user_style(response, user_preferences))
                .collect();
                
            // Adjust intensity based on magnitude
            if magnitude > 0.8 && user_preferences.celebrations_preferred {
                suitable_responses.extend(vec![
                    &"That's excellent work".to_string(),
                    &"Really solid".to_string(),
                ]);
            }
            
            if let Some(response) = suitable_responses.choose(&mut rand::thread_rng()) {
                let celebration = (*response).clone();
                
                // Record this celebration
                self.celebration_history.push(CelebrationEvent {
                    achievement_type: achievement_type.to_string(),
                    celebration_intensity: magnitude,
                    user_response: None, // Will be updated with user feedback
                    timestamp: Utc::now(),
                });
                
                return celebration;
            }
        }
        
        // Fallback to simple acknowledgment
        "Nice work".to_string()
    }
    
    fn matches_user_style(&self, response: &str, preferences: &CommunicationPreferences) -> bool {
        match preferences.formality_level {
            FormalityLevel::Professional => {
                !response.contains("nice") && !response.contains("cool")
            }
            FormalityLevel::Casual => {
                !response.contains("excellent") && !response.contains("accomplished")
            }
            FormalityLevel::Balanced => true, // Most responses work for balanced
        }
    }
}

/// Enhanced context for personality decisions with work context
#[derive(Debug, Clone)]
pub struct PersonalityContext {
    pub current_state: ADHDState,
    pub previous_state: Option<ADHDState>,
    pub metrics: BehavioralMetrics,
    pub work_context: WorkContext,
    pub time_of_day: String,
    pub recent_interactions: Vec<InteractionHistory>,
    pub user_feedback_history: Vec<UserFeedback>,
    pub session_duration: Duration,
}

#[derive(Debug, Clone)]
pub struct InteractionHistory {
    pub message: String,
    pub user_response: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl Default for CommunicationPreferences {
    fn default() -> Self {
        Self {
            intensity_preference: IntensityPreference::Moderate,
            formality_level: FormalityLevel::Balanced,
            humor_appreciation: 0.5,
            encouragement_preferred: true,
            celebrations_preferred: true,
            emojis_preferred: true,
            last_updated: Utc::now(),
        }
    }
}

impl Default for AttentionPreferences {
    fn default() -> Self {
        Self {
            preferred_message_length: MessageLength::Medium,
            detail_preference: DetailPreference::Concise,
            processing_speed: ProcessingSpeed::Moderate,
            context_switching_comfort: 0.5,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_expertise_tracker_new() {
        let tracker = ExpertiseTracker::new();
        assert_eq!(tracker.confidence_threshold, 0.7);
        assert!(tracker.domain_assessments.is_empty());
    }
    
    #[test]
    fn test_communication_preferences_default() {
        let prefs = CommunicationPreferences::default();
        assert!(matches!(prefs.intensity_preference, IntensityPreference::Moderate));
        assert!(matches!(prefs.formality_level, FormalityLevel::Balanced));
        assert_eq!(prefs.humor_appreciation, 0.5);
        assert!(prefs.encouragement_preferred);
    }
    
    #[test]
    fn test_celebration_manager_generates_authentic_responses() {
        let mut manager = CelebrationManager::new();
        let prefs = CommunicationPreferences::default();
        
        let celebration = manager.generate_authentic_celebration(
            "task_completion",
            0.7,
            &prefs,
        );
        
        // Should be a reasonable response
        assert!(!celebration.is_empty());
        assert!(!celebration.contains("AMAZING")); // Should avoid over-the-top responses
    }
    
    #[test]
    fn test_consistency_validator_detects_similarity() {
        let validator = ConsistencyValidator::new();
        
        let prev = "You've got this! Keep going with your task.";
        let current = "You've got this! Keep going with your work.";
        
        assert!(validator.too_similar(prev, current));
    }
    
    #[test]
    fn test_expertise_level_assessment() {
        let tracker = ExpertiseTracker::new();
        let work_context = WorkContext {
            work_type: WorkType::Coding { 
                language: "rust".to_string(), 
                framework: Some("tokio".to_string()) 
            },
            application: "vscode".to_string(),
            window_title: "main.rs".to_string(),
            screenshot_text: Some("impl trait for async fn".to_string()),
            task_category: crate::types::TaskCategory::Work,
            urgency: crate::types::UrgencyLevel::Medium,
            time_of_day: crate::types::TimeOfDay::Afternoon,
        };
        
        let level = tracker.get_expertise_level(&work_context);
        assert!(matches!(level, ExpertiseLevel::Beginner)); // Default for new domains
    }
}