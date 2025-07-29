//! Anti-patronization filters and authentic celebration system
//!
//! Prevents condescending language and ensures authentic, respectful interactions

use crate::error::{AIIntegrationError, Result};
use crate::personality_enhanced::{ExpertiseLevel, FormalityLevel, CommunicationPreferences};
use crate::types::{ADHDState, BehavioralMetrics, CompanionMood};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use regex::Regex;

/// Comprehensive anti-patronization filter system
pub struct AntiPatronizationFilter {
    patronizing_patterns: Vec<PatronizingPattern>,
    expertise_filters: HashMap<ExpertiseLevel, Vec<LanguageFilter>>,
    context_sensitive_replacements: Vec<ContextualReplacement>,
    authenticity_validator: AuthenticityValidator,
}

impl AntiPatronizationFilter {
    /// Create a new anti-patronization filter
    pub fn new() -> Self {
        let patronizing_patterns = Self::build_patronizing_patterns();
        let expertise_filters = Self::build_expertise_filters();
        let context_sensitive_replacements = Self::build_contextual_replacements();
        
        Self {
            patronizing_patterns,
            expertise_filters,
            context_sensitive_replacements,
            authenticity_validator: AuthenticityValidator::new(),
        }
    }
    
    /// Filter message to remove patronizing language
    pub fn filter_message(
        &self,
        message: &str,
        expertise_level: &ExpertiseLevel,
        user_state: &ADHDState,
        communication_prefs: &CommunicationPreferences,
    ) -> Result<String> {
        let mut filtered = message.to_string();
        
        // Apply general patronizing pattern filters
        filtered = self.apply_general_filters(&filtered)?;
        
        // Apply expertise-specific filters
        filtered = self.apply_expertise_filters(&filtered, expertise_level)?;
        
        // Apply context-sensitive replacements
        filtered = self.apply_contextual_replacements(&filtered, user_state, communication_prefs)?;
        
        // Validate authenticity
        filtered = self.authenticity_validator.ensure_authentic(&filtered, expertise_level)?;
        
        Ok(filtered)
    }
    
    /// Check if a message contains patronizing language
    pub fn contains_patronizing_language(&self, message: &str, expertise_level: &ExpertiseLevel) -> bool {
        // Check general patronizing patterns
        for pattern in &self.patronizing_patterns {
            if pattern.matches(message, expertise_level) {
                return true;
            }
        }
        
        // Check expertise-specific patterns
        if let Some(filters) = self.expertise_filters.get(expertise_level) {
            for filter in filters {
                if filter.triggers_filter(message) {
                    return true;
                }
            }
        }
        
        false
    }
    
    /// Generate authenticity score for a message
    pub fn calculate_authenticity_score(&self, message: &str, expertise_level: &ExpertiseLevel) -> f32 {
        self.authenticity_validator.calculate_score(message, expertise_level)
    }
    
    fn apply_general_filters(&self, message: &str) -> Result<String> {
        let mut filtered = message.to_string();
        
        for pattern in &self.patronizing_patterns {
            filtered = pattern.apply_filter(&filtered);
        }
        
        Ok(filtered)
    }
    
    fn apply_expertise_filters(&self, message: &str, expertise_level: &ExpertiseLevel) -> Result<String> {
        let mut filtered = message.to_string();
        
        if let Some(filters) = self.expertise_filters.get(expertise_level) {
            for filter in filters {
                filtered = filter.apply(&filtered);
            }
        }
        
        Ok(filtered)
    }
    
    fn apply_contextual_replacements(
        &self,
        message: &str,
        user_state: &ADHDState,
        communication_prefs: &CommunicationPreferences,
    ) -> Result<String> {
        let mut filtered = message.to_string();
        
        for replacement in &self.context_sensitive_replacements {
            if replacement.applies_to_context(user_state, communication_prefs) {
                filtered = replacement.apply(&filtered);
            }
        }
        
        Ok(filtered)
    }
    
    fn build_patronizing_patterns() -> Vec<PatronizingPattern> {
        vec![
            // Overly simplifying language
            PatronizingPattern {
                name: "oversimplification".to_string(),
                triggers: vec![
                    "simply".to_string(),
                    "just".to_string(),
                    "easily".to_string(),
                    "obviously".to_string(),
                    "basic".to_string(),
                    "elementary".to_string(),
                ],
                replacements: HashMap::from([
                    ("simply".to_string(), "".to_string()),
                    ("just try".to_string(), "try".to_string()),
                    ("easily".to_string(), "".to_string()),
                    ("obviously".to_string(), "".to_string()),
                    ("basic".to_string(), "".to_string()),
                    ("elementary".to_string(), "".to_string()),
                ]),
                expertise_sensitivity: HashMap::from([
                    (ExpertiseLevel::Expert, 1.0),
                    (ExpertiseLevel::Intermediate, 0.7),
                    (ExpertiseLevel::Beginner, 0.3),
                ]),
            },
            
            // Condescending reassurance
            PatronizingPattern {
                name: "condescending_reassurance".to_string(),
                triggers: vec![
                    "don't worry".to_string(),
                    "no need to worry".to_string(),
                    "it's okay if you don't understand".to_string(),
                    "that's perfectly normal".to_string(),
                    "everyone struggles with this".to_string(),
                ],
                replacements: HashMap::from([
                    ("don't worry,".to_string(), "".to_string()),
                    ("don't worry".to_string(), "".to_string()),
                    ("no need to worry,".to_string(), "".to_string()),
                    ("it's okay if you don't understand".to_string(), "".to_string()),
                    ("that's perfectly normal".to_string(), "".to_string()),
                    ("everyone struggles with this".to_string(), "this can be challenging".to_string()),
                ]),
                expertise_sensitivity: HashMap::from([
                    (ExpertiseLevel::Expert, 1.0),
                    (ExpertiseLevel::Intermediate, 0.8),
                    (ExpertiseLevel::Beginner, 0.2),
                ]),
            },
            
            // Over-explaining
            PatronizingPattern {
                name: "over_explaining".to_string(),
                triggers: vec![
                    "let me explain".to_string(),
                    "what this means is".to_string(),
                    "in other words".to_string(),
                    "to put it simply".to_string(),
                    "basically what happens is".to_string(),
                ],
                replacements: HashMap::from([
                    ("let me explain".to_string(), "".to_string()),
                    ("what this means is".to_string(), "".to_string()),
                    ("in other words".to_string(), "".to_string()),
                    ("to put it simply".to_string(), "".to_string()),
                    ("basically what happens is".to_string(), "".to_string()),
                ]),
                expertise_sensitivity: HashMap::from([
                    (ExpertiseLevel::Expert, 1.0),
                    (ExpertiseLevel::Intermediate, 0.6),
                    (ExpertiseLevel::Beginner, 0.1),
                ]),
            },
            
            // False encouragement
            PatronizingPattern {
                name: "false_encouragement".to_string(),
                triggers: vec![
                    "you're doing great!".to_string(),
                    "amazing job!".to_string(),
                    "you're so smart!".to_string(),
                    "perfect!".to_string(),
                    "wonderful!".to_string(),
                ],
                replacements: HashMap::from([
                    ("you're doing great!".to_string(), "good work".to_string()),
                    ("amazing job!".to_string(), "nice work".to_string()),
                    ("you're so smart!".to_string(), "".to_string()),
                    ("perfect!".to_string(), "good".to_string()),
                    ("wonderful!".to_string(), "good".to_string()),
                ]),
                expertise_sensitivity: HashMap::from([
                    (ExpertiseLevel::Expert, 0.9),
                    (ExpertiseLevel::Intermediate, 0.7),
                    (ExpertiseLevel::Beginner, 0.3),
                ]),
            },
        ]
    }
    
    fn build_expertise_filters() -> HashMap<ExpertiseLevel, Vec<LanguageFilter>> {
        let mut filters = HashMap::new();
        
        // Expert-level filters (remove almost all simplification)
        filters.insert(ExpertiseLevel::Expert, vec![
            LanguageFilter {
                name: "expert_oversimplification".to_string(),
                pattern: Regex::new(r"\b(try to|attempt to|see if you can)\b").unwrap(),
                replacement: "".to_string(),
                confidence: 0.9,
            },
            LanguageFilter {
                name: "expert_unnecessary_explanation".to_string(),
                pattern: Regex::new(r"\b(this means that|what happens is|the reason is)\b").unwrap(),
                replacement: "".to_string(),
                confidence: 0.8,
            },
        ]);
        
        // Intermediate-level filters (moderate filtering)
        filters.insert(ExpertiseLevel::Intermediate, vec![
            LanguageFilter {
                name: "intermediate_oversimplification".to_string(),
                pattern: Regex::new(r"\b(very easy|super simple|piece of cake)\b").unwrap(),
                replacement: "straightforward".to_string(),
                confidence: 0.7,
            },
        ]);
        
        // Beginner-level filters (minimal filtering, keep supportive language)
        filters.insert(ExpertiseLevel::Beginner, vec![
            LanguageFilter {
                name: "beginner_condescension".to_string(),
                pattern: Regex::new(r"\b(of course you don't know|naturally you wouldn't understand)\b").unwrap(),
                replacement: "".to_string(),
                confidence: 1.0,
            },
        ]);
        
        filters
    }
    
    fn build_contextual_replacements() -> Vec<ContextualReplacement> {
        vec![
            // Flow state adaptations
            ContextualReplacement {
                name: "flow_state_respect".to_string(),
                trigger_states: vec![StatePattern::Flow { min_depth: 0.7 }],
                pattern: Regex::new(r"\b(take a break|step away|stop what you're doing)\b").unwrap(),
                replacement: "when ready, consider".to_string(),
                formality_levels: vec![FormalityLevel::Balanced, FormalityLevel::Professional],
            },
            
            // Hyperfocus adaptations
            ContextualReplacement {
                name: "hyperfocus_brevity".to_string(),
                trigger_states: vec![StatePattern::Hyperfocus { min_intensity: 0.6 }],
                pattern: Regex::new(r"^(.{50,})$").unwrap(), // Long messages
                replacement: "💡 Quick tip".to_string(),
                formality_levels: vec![FormalityLevel::Casual, FormalityLevel::Balanced],
            },
            
            // Distraction-sensitive language
            ContextualReplacement {
                name: "distraction_sensitivity".to_string(),
                trigger_states: vec![StatePattern::Distracted { min_severity: 0.6 }],
                pattern: Regex::new(r"\b(focus|concentrate|pay attention)\b").unwrap(),
                replacement: "gently redirect to".to_string(),
                formality_levels: vec![FormalityLevel::Casual, FormalityLevel::Balanced],
            },
        ]
    }
}

/// Pattern that identifies patronizing language
#[derive(Debug, Clone)]
struct PatronizingPattern {
    name: String,
    triggers: Vec<String>,
    replacements: HashMap<String, String>,
    expertise_sensitivity: HashMap<ExpertiseLevel, f32>, // 0.0-1.0, higher = more filtering
}

impl PatronizingPattern {
    fn matches(&self, message: &str, expertise_level: &ExpertiseLevel) -> bool {
        let sensitivity = self.expertise_sensitivity.get(expertise_level).unwrap_or(&0.5);
        
        if *sensitivity < 0.3 {
            return false; // Low sensitivity, don't filter
        }
        
        let message_lower = message.to_lowercase();
        self.triggers.iter().any(|trigger| message_lower.contains(trigger))
    }
    
    fn apply_filter(&self, message: &str) -> String {
        let mut filtered = message.to_string();
        
        for (trigger, replacement) in &self.replacements {
            // Case-insensitive replacement
            let pattern = regex::Regex::new(&format!(r"(?i)\b{}\b", regex::escape(trigger))).unwrap();
            filtered = pattern.replace_all(&filtered, replacement.as_str()).to_string();
        }
        
        // Clean up extra spaces
        let space_pattern = regex::Regex::new(r"\s+").unwrap();
        filtered = space_pattern.replace_all(&filtered, " ").trim().to_string();
        
        filtered
    }
}

/// Language filter for expertise-specific patterns
#[derive(Debug, Clone)]
struct LanguageFilter {
    name: String,
    pattern: Regex,
    replacement: String,
    confidence: f32,
}

impl LanguageFilter {
    fn triggers_filter(&self, message: &str) -> bool {
        self.pattern.is_match(message)
    }
    
    fn apply(&self, message: &str) -> String {
        self.pattern.replace_all(message, self.replacement.as_str()).to_string()
    }
}

/// Context-sensitive replacement based on user state
#[derive(Debug, Clone)]
struct ContextualReplacement {
    name: String,
    trigger_states: Vec<StatePattern>,
    pattern: Regex,
    replacement: String,
    formality_levels: Vec<FormalityLevel>,
}

impl ContextualReplacement {
    fn applies_to_context(
        &self,
        user_state: &ADHDState,
        communication_prefs: &CommunicationPreferences,
    ) -> bool {
        // Check if formality level matches
        if !self.formality_levels.contains(&communication_prefs.formality_level) {
            return false;
        }
        
        // Check if user state matches any trigger pattern
        self.trigger_states.iter().any(|pattern| pattern.matches(user_state))
    }
    
    fn apply(&self, message: &str) -> String {
        self.pattern.replace_all(message, self.replacement.as_str()).to_string()
    }
}

/// State pattern for matching ADHD states
#[derive(Debug, Clone)]
enum StatePattern {
    Flow { min_depth: f32 },
    Hyperfocus { min_intensity: f32 },
    Distracted { min_severity: f32 },
}

impl StatePattern {
    fn matches(&self, state: &ADHDState) -> bool {
        match (self, &state.state_type) {
            (StatePattern::Flow { min_depth }, crate::types::ADHDStateType::Flow { depth }) => {
                depth >= min_depth
            }
            (StatePattern::Hyperfocus { min_intensity }, crate::types::ADHDStateType::Hyperfocus { intensity }) => {
                intensity >= min_intensity
            }
            (StatePattern::Distracted { min_severity }, crate::types::ADHDStateType::Distracted { severity }) => {
                severity >= min_severity
            }
            _ => false,
        }
    }
}

/// Validates message authenticity and prevents artificial language
pub struct AuthenticityValidator {
    artificial_patterns: Vec<ArtificialPattern>,
    authenticity_indicators: Vec<AuthenticityIndicator>,
}

impl AuthenticityValidator {
    pub fn new() -> Self {
        Self {
            artificial_patterns: Self::build_artificial_patterns(),
            authenticity_indicators: Self::build_authenticity_indicators(),
        }
    }
    
    /// Ensure message feels authentic and natural
    pub fn ensure_authentic(&self, message: &str, expertise_level: &ExpertiseLevel) -> Result<String> {
        let mut authentic = message.to_string();
        
        // Remove artificial patterns
        for pattern in &self.artificial_patterns {
            if pattern.applies_to_expertise(expertise_level) {
                authentic = pattern.filter(&authentic);
            }
        }
        
        // Enhance with authenticity indicators if needed
        authentic = self.enhance_authenticity(&authentic, expertise_level);
        
        Ok(authentic)
    }
    
    /// Calculate authenticity score (0.0-1.0, higher is more authentic)
    pub fn calculate_score(&self, message: &str, expertise_level: &ExpertiseLevel) -> f32 {
        let mut score = 1.0;
        
        // Subtract points for artificial patterns
        for pattern in &self.artificial_patterns {
            if pattern.applies_to_expertise(expertise_level) && pattern.matches(message) {
                score -= pattern.penalty;
            }
        }
        
        // Add points for authenticity indicators
        for indicator in &self.authenticity_indicators {
            if indicator.matches(message) {
                score += indicator.bonus;
            }
        }
        
        score.clamp(0.0, 1.0)
    }
    
    fn enhance_authenticity(&self, message: &str, expertise_level: &ExpertiseLevel) -> String {
        // If authenticity score is low, try to improve it
        let score = self.calculate_score(message, expertise_level);
        
        if score < 0.6 {
            // Message feels artificial, make it more natural
            self.naturalize_language(message, expertise_level)
        } else {
            message.to_string()
        }
    }
    
    fn naturalize_language(&self, message: &str, expertise_level: &ExpertiseLevel) -> String {
        let mut natural = message.to_string();
        
        // Replace overly formal constructions with natural ones
        let naturalizations = match expertise_level {
            ExpertiseLevel::Expert => vec![
                ("I would suggest that you", "You might"),
                ("It would be beneficial to", "Consider"),
                ("You have the capability to", "You can"),
            ],
            ExpertiseLevel::Intermediate => vec![
                ("I would recommend", "Try"),
                ("It might be helpful to", "You could"),
                ("Consider the possibility of", "Maybe"),
            ],
            ExpertiseLevel::Beginner => vec![
                ("You should attempt to", "Try"),
                ("It would be wise to", "You might want to"),
            ],
        };
        
        for (formal, natural_replacement) in naturalizations {
            natural = natural.replace(formal, natural_replacement);
        }
        
        natural
    }
    
    fn build_artificial_patterns() -> Vec<ArtificialPattern> {
        vec![
            ArtificialPattern {
                name: "robotic_language".to_string(),
                patterns: vec![
                    "I am here to assist you".to_string(),
                    "As an AI assistant".to_string(),
                    "I understand your concern".to_string(),
                    "Thank you for your question".to_string(),
                ],
                penalty: 0.3,
                expertise_applicability: vec![ExpertiseLevel::Expert, ExpertiseLevel::Intermediate, ExpertiseLevel::Beginner],
            },
            ArtificialPattern {
                name: "excessive_politeness".to_string(),
                patterns: vec![
                    "I would be delighted to".to_string(),
                    "It would be my pleasure to".to_string(),
                    "I sincerely hope".to_string(),
                ],
                penalty: 0.2,
                expertise_applicability: vec![ExpertiseLevel::Expert, ExpertiseLevel::Intermediate],
            },
            ArtificialPattern {
                name: "corporate_speak".to_string(),
                patterns: vec![
                    "moving forward".to_string(),
                    "best practices".to_string(),
                    "optimize your workflow".to_string(),
                    "leverage your capabilities".to_string(),
                ],
                penalty: 0.25,
                expertise_applicability: vec![ExpertiseLevel::Expert],
            },
        ]
    }
    
    fn build_authenticity_indicators() -> Vec<AuthenticityIndicator> {
        vec![
            AuthenticityIndicator {
                name: "natural_conversational".to_string(),
                patterns: vec![
                    "you might".to_string(),
                    "maybe try".to_string(),
                    "could work".to_string(),
                    "worth a shot".to_string(),
                ],
                bonus: 0.1,
            },
            AuthenticityIndicator {
                name: "specific_actionable".to_string(),
                patterns: vec![
                    "try this:".to_string(),
                    "here's what".to_string(),
                    "next step".to_string(),
                ],
                bonus: 0.15,
            },
            AuthenticityIndicator {
                name: "empathetic_realistic".to_string(),
                patterns: vec![
                    "can be tricky".to_string(),
                    "happens to everyone".to_string(),
                    "totally get that".to_string(),
                ],
                bonus: 0.1,
            },
        ]
    }
}

/// Pattern that identifies artificial/robotic language
#[derive(Debug, Clone)]
struct ArtificialPattern {
    name: String,
    patterns: Vec<String>,
    penalty: f32,
    expertise_applicability: Vec<ExpertiseLevel>,
}

impl ArtificialPattern {
    fn matches(&self, message: &str) -> bool {
        let message_lower = message.to_lowercase();
        self.patterns.iter().any(|pattern| message_lower.contains(&pattern.to_lowercase()))
    }
    
    fn applies_to_expertise(&self, expertise: &ExpertiseLevel) -> bool {
        self.expertise_applicability.contains(expertise)
    }
    
    fn filter(&self, message: &str) -> String {
        let mut filtered = message.to_string();
        
        for pattern in &self.patterns {
            // Remove the artificial pattern
            filtered = filtered.replace(pattern, "");
        }
        
        // Clean up spacing
        let space_pattern = regex::Regex::new(r"\s+").unwrap();
        filtered = space_pattern.replace_all(&filtered, " ").trim().to_string();
        
        filtered
    }
}

/// Indicator of authentic, natural language
#[derive(Debug, Clone)]
struct AuthenticityIndicator {
    name: String,
    patterns: Vec<String>,
    bonus: f32,
}

impl AuthenticityIndicator {
    fn matches(&self, message: &str) -> bool {
        let message_lower = message.to_lowercase();
        self.patterns.iter().any(|pattern| message_lower.contains(&pattern.to_lowercase()))
    }
}

/// Authentic celebration system that avoids over-the-top responses
pub struct AuthenticCelebrationSystem {
    celebration_patterns: HashMap<CelebrationType, Vec<AuthenticCelebration>>,
    intensity_modifiers: HashMap<CelebrationIntensity, IntensityModifier>,
    anti_patterns: Vec<String>, // Patterns to avoid
}

impl AuthenticCelebrationSystem {
    pub fn new() -> Self {
        Self {
            celebration_patterns: Self::build_celebration_patterns(),
            intensity_modifiers: Self::build_intensity_modifiers(),
            anti_patterns: Self::build_anti_patterns(),
        }
    }
    
    /// Generate authentic celebration message
    pub fn generate_celebration(
        &self,
        celebration_type: CelebrationType,
        intensity: CelebrationIntensity,
        expertise_level: &ExpertiseLevel,
        communication_prefs: &CommunicationPreferences,
    ) -> Result<String> {
        // Get appropriate celebration patterns
        let patterns = self.celebration_patterns.get(&celebration_type)
            .ok_or_else(|| AIIntegrationError::InternalError)?;
        
        // Filter patterns by expertise level and communication preferences
        let suitable_patterns: Vec<_> = patterns.iter()
            .filter(|pattern| pattern.suitable_for_expertise(expertise_level))
            .filter(|pattern| pattern.matches_formality(&communication_prefs.formality_level))
            .collect();
        
        if suitable_patterns.is_empty() {
            return Ok("Nice work".to_string()); // Safe fallback
        }
        
        // Select a pattern
        let selected_pattern = suitable_patterns[0]; // In practice, would use random selection
        
        // Apply intensity modification
        let intensity_modifier = self.intensity_modifiers.get(&intensity)
            .ok_or_else(|| AIIntegrationError::InternalError)?;
        
        let celebration = intensity_modifier.apply(selected_pattern, communication_prefs);
        
        // Ensure it doesn't contain anti-patterns
        if self.contains_anti_patterns(&celebration) {
            return Ok("Good work".to_string()); // Safe fallback
        }
        
        Ok(celebration)
    }
    
    fn contains_anti_patterns(&self, message: &str) -> bool {
        let message_lower = message.to_lowercase();
        self.anti_patterns.iter().any(|pattern| message_lower.contains(pattern))
    }
    
    fn build_celebration_patterns() -> HashMap<CelebrationType, Vec<AuthenticCelebration>> {
        let mut patterns = HashMap::new();
        
        patterns.insert(CelebrationType::TaskCompletion, vec![
            AuthenticCelebration {
                message: "Task done".to_string(),
                formality_levels: vec![FormalityLevel::Casual, FormalityLevel::Balanced],
                expertise_levels: vec![ExpertiseLevel::Beginner, ExpertiseLevel::Intermediate, ExpertiseLevel::Expert],
                base_intensity: 0.3,
            },
            AuthenticCelebration {
                message: "One down".to_string(),
                formality_levels: vec![FormalityLevel::Casual],
                expertise_levels: vec![ExpertiseLevel::Intermediate, ExpertiseLevel::Expert],
                base_intensity: 0.4,
            },
            AuthenticCelebration {
                message: "That's wrapped up".to_string(),
                formality_levels: vec![FormalityLevel::Balanced, FormalityLevel::Professional],
                expertise_levels: vec![ExpertiseLevel::Expert],
                base_intensity: 0.3,
            },
        ]);
        
        patterns.insert(CelebrationType::FlowState, vec![
            AuthenticCelebration {
                message: "Nice flow state".to_string(),
                formality_levels: vec![FormalityLevel::Casual, FormalityLevel::Balanced],
                expertise_levels: vec![ExpertiseLevel::Intermediate, ExpertiseLevel::Expert],
                base_intensity: 0.5,
            },
            AuthenticCelebration {
                message: "You found your groove".to_string(),
                formality_levels: vec![FormalityLevel::Casual],
                expertise_levels: vec![ExpertiseLevel::Beginner, ExpertiseLevel::Intermediate],
                base_intensity: 0.6,
            },
            AuthenticCelebration {
                message: "Solid focus right there".to_string(),
                formality_levels: vec![FormalityLevel::Casual, FormalityLevel::Balanced],
                expertise_levels: vec![ExpertiseLevel::Expert],
                base_intensity: 0.7,
            },
        ]);
        
        patterns.insert(CelebrationType::Milestone, vec![
            AuthenticCelebration {
                message: "That's a milestone".to_string(),
                formality_levels: vec![FormalityLevel::Balanced, FormalityLevel::Professional],
                expertise_levels: vec![ExpertiseLevel::Expert],
                base_intensity: 0.6,
            },
            AuthenticCelebration {
                message: "Nice progress".to_string(),
                formality_levels: vec![FormalityLevel::Casual, FormalityLevel::Balanced],
                expertise_levels: vec![ExpertiseLevel::Beginner, ExpertiseLevel::Intermediate],
                base_intensity: 0.5,
            },
        ]);
        
        patterns
    }
    
    fn build_intensity_modifiers() -> HashMap<CelebrationIntensity, IntensityModifier> {
        let mut modifiers = HashMap::new();
        
        modifiers.insert(CelebrationIntensity::Subtle, IntensityModifier {
            prefix: None,
            suffix: None,
            emoji_allowed: false,
            enthusiasm_multiplier: 0.7,
        });
        
        modifiers.insert(CelebrationIntensity::Moderate, IntensityModifier {
            prefix: None,
            suffix: None,
            emoji_allowed: true,
            enthusiasm_multiplier: 1.0,
        });
        
        modifiers.insert(CelebrationIntensity::Enthusiastic, IntensityModifier {
            prefix: Some("Really ".to_string()),
            suffix: Some("!".to_string()),
            emoji_allowed: true,
            enthusiasm_multiplier: 1.3,
        });
        
        modifiers
    }
    
    fn build_anti_patterns() -> Vec<String> {
        vec![
            "amazing!!!".to_string(),
            "super duper".to_string(),
            "you're the best".to_string(),
            "incredible job".to_string(),
            "perfect work".to_string(),
            "outstanding".to_string(),
            "phenomenal".to_string(),
            "spectacular".to_string(),
        ]
    }
}

/// Type of celebration
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum CelebrationType {
    TaskCompletion,
    FlowState,
    Milestone,
    Recovery, // Recovering from distraction
}

/// Intensity of celebration
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum CelebrationIntensity {
    Subtle,
    Moderate,
    Enthusiastic,
}

/// Authentic celebration pattern
#[derive(Debug, Clone)]
struct AuthenticCelebration {
    message: String,
    formality_levels: Vec<FormalityLevel>,
    expertise_levels: Vec<ExpertiseLevel>,
    base_intensity: f32,
}

impl AuthenticCelebration {
    fn suitable_for_expertise(&self, expertise: &ExpertiseLevel) -> bool {
        self.expertise_levels.contains(expertise)
    }
    
    fn matches_formality(&self, formality: &FormalityLevel) -> bool {
        self.formality_levels.contains(formality)
    }
}

/// Modifies celebration intensity
#[derive(Debug, Clone)]
struct IntensityModifier {
    prefix: Option<String>,
    suffix: Option<String>,
    emoji_allowed: bool,
    enthusiasm_multiplier: f32,
}

impl IntensityModifier {
    fn apply(&self, celebration: &AuthenticCelebration, prefs: &CommunicationPreferences) -> String {
        let mut result = celebration.message.clone();
        
        // Add prefix if appropriate
        if let Some(ref prefix) = self.prefix {
            if celebration.base_intensity * self.enthusiasm_multiplier <= 1.0 {
                result = format!("{}{}", prefix, result);
            }
        }
        
        // Add suffix if appropriate
        if let Some(ref suffix) = self.suffix {
            result = format!("{}{}", result, suffix);
        }
        
        // Add emoji if allowed and user prefers them
        if self.emoji_allowed && prefs.emojis_preferred {
            if celebration.base_intensity > 0.6 {
                result = format!("👍 {}", result);
            }
        }
        
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ADHDState, ADHDStateType};
    use std::collections::HashMap;
    
    #[test]
    fn test_anti_patronization_filter_creation() {
        let filter = AntiPatronizationFilter::new();
        assert!(!filter.patronizing_patterns.is_empty());
        assert!(!filter.expertise_filters.is_empty());
    }
    
    #[test]
    fn test_patronizing_detection() {
        let filter = AntiPatronizationFilter::new();
        
        let patronizing_message = "Don't worry, simply try this basic approach";
        assert!(filter.contains_patronizing_language(patronizing_message, &ExpertiseLevel::Expert));
        
        let respectful_message = "Consider trying this approach";
        assert!(!filter.contains_patronizing_language(respectful_message, &ExpertiseLevel::Expert));
    }
    
    #[test]
    fn test_authenticity_validator() {
        let validator = AuthenticityValidator::new();
        
        let artificial_message = "I am here to assist you with your concern";
        let score = validator.calculate_score(artificial_message, &ExpertiseLevel::Expert);
        assert!(score < 0.8); // Should have low authenticity score
        
        let natural_message = "You might try this approach";
        let score = validator.calculate_score(natural_message, &ExpertiseLevel::Expert);
        assert!(score > 0.8); // Should have high authenticity score
    }
    
    #[test]
    fn test_authentic_celebration_system() {
        let system = AuthenticCelebrationSystem::new();
        let prefs = CommunicationPreferences::default();
        
        let celebration = system.generate_celebration(
            CelebrationType::TaskCompletion,
            CelebrationIntensity::Moderate,
            &ExpertiseLevel::Expert,
            &prefs,
        ).unwrap();
        
        assert!(!celebration.is_empty());
        assert!(!celebration.contains("AMAZING")); // Should avoid over-the-top language
    }
    
    #[test]
    fn test_expertise_specific_filtering() {
        let filter = AntiPatronizationFilter::new();
        let prefs = CommunicationPreferences::default();
        let state = ADHDState {
            state_type: ADHDStateType::Neutral,
            confidence: 0.8,
            depth: None,
            duration: 1000,
            metadata: HashMap::new(),
        };
        
        let message = "Don't worry, simply try this basic approach";
        
        // Expert should get more filtering
        let expert_filtered = filter.filter_message(message, &ExpertiseLevel::Expert, &state, &prefs).unwrap();
        assert!(!expert_filtered.contains("Don't worry"));
        assert!(!expert_filtered.contains("simply"));
        
        // Beginner should get less filtering
        let beginner_filtered = filter.filter_message(message, &ExpertiseLevel::Beginner, &state, &prefs).unwrap();
        // Some supportive language might be preserved for beginners
        assert!(!beginner_filtered.is_empty());
    }
    
    #[test]
    fn test_contextual_replacements() {
        let filter = AntiPatronizationFilter::new();
        let prefs = CommunicationPreferences::default();
        
        // Test flow state adaptation
        let flow_state = ADHDState {
            state_type: ADHDStateType::Flow { depth: 0.8 },
            confidence: 0.9,
            depth: Some(0.8),
            duration: 5000,
            metadata: HashMap::new(),
        };
        
        let message = "Take a break now";
        let filtered = filter.filter_message(message, &ExpertiseLevel::Intermediate, &flow_state, &prefs).unwrap();
        
        // Should be more respectful of flow state
        assert!(filtered.contains("when ready") || !filtered.contains("Take a break now"));
    }
}