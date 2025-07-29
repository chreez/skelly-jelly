//! Personality Engine for Skelly Companion
//!
//! Maintains consistent skeleton companion personality across all interactions
//! with chill, supportive vibes and occasional skeleton puns.

use crate::error::{AIIntegrationError, Result};
use crate::types::{PersonalityTraits, ADHDState, BehavioralMetrics, CompanionMood, WorkContext};
use rand::seq::SliceRandom;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Personality engine that applies Skelly's character to responses
pub struct PersonalityEngine {
    traits: PersonalityTraits,
    mood_tracker: MoodTracker,
    expression_generator: ExpressionGenerator,
    pun_generator: SkeletonPunGenerator,
    tone_adjuster: ToneAdjuster,
}

impl PersonalityEngine {
    pub fn new(traits: PersonalityTraits) -> Self {
        Self {
            traits,
            mood_tracker: MoodTracker::new(),
            expression_generator: ExpressionGenerator::new(),
            pun_generator: SkeletonPunGenerator::new(),
            tone_adjuster: ToneAdjuster::new(),
        }
    }

    /// Apply personality to a suggestion message
    pub fn apply(&mut self, suggestion: String, context: &PersonalityContext) -> Result<String> {
        let mut modified = suggestion;

        // Update mood based on context
        let current_mood = self.mood_tracker.determine_mood(context);

        // Adjust tone based on user state
        modified = self.tone_adjuster.adjust_tone(&modified, context)?;

        // Add personality flair
        modified = self.add_personality_flair(&modified, &current_mood)?;

        // Maybe add a skeleton pun (sparingly)
        if self.should_add_pun() {
            modified = self.pun_generator.add_pun(&modified)?;
        }

        // Ensure appropriate length
        modified = self.ensure_brevity(&modified)?;

        // Add casual expressions
        modified = self.expression_generator.add_expression(&modified, &current_mood)?;

        Ok(modified)
    }

    /// Update personality traits
    pub fn update_traits(&mut self, new_traits: PersonalityTraits) -> Result<()> {
        self.traits = new_traits;
        Ok(())
    }

    /// Get current personality state
    pub fn get_current_state(&self) -> PersonalityState {
        PersonalityState {
            traits: self.traits.clone(),
            current_mood: self.mood_tracker.get_current_mood(),
            pun_streak: self.pun_generator.get_streak(),
            energy_level: self.calculate_energy_level(),
        }
    }

    fn should_add_pun(&self) -> bool {
        let mut rng = rand::thread_rng();
        rng.gen::<f32>() < self.traits.pun_frequency
    }

    fn add_personality_flair(&self, message: &str, mood: &CompanionMood) -> Result<String> {
        let mut modified = message.to_string();

        // Add mood-specific modifications
        match mood {
            CompanionMood::Happy => {
                if self.traits.cheerfulness > 0.7 {
                    modified = format!("✨ {}", modified);
                }
            }
            CompanionMood::Excited => {
                if self.traits.enthusiasm() > 0.8 {
                    modified = format!("🎉 {}", modified);
                }
            }
            CompanionMood::Supportive => {
                if self.traits.supportiveness > 0.8 {
                    modified = self.add_supportive_language(&modified);
                }
            }
            _ => {}
        }

        Ok(modified)
    }

    fn add_supportive_language(&self, message: &str) -> String {
        let supportive_phrases = vec![
            "You've got this! ",
            "I believe in you! ",
            "You're doing great! ",
            "Keep going! ",
        ];

        if let Some(phrase) = supportive_phrases.choose(&mut rand::thread_rng()) {
            format!("{}{}", phrase, message)
        } else {
            message.to_string()
        }
    }

    fn ensure_brevity(&self, message: &str) -> Result<String> {
        let words: Vec<&str> = message.split_whitespace().collect();
        
        if words.len() <= 20 {
            return Ok(message.to_string());
        }

        // Compress message while maintaining meaning
        let compressed = if words.len() > 30 {
            // Very long message - take first sentence
            message.split('.').next().unwrap_or(message).to_string()
        } else {
            // Moderately long - remove filler words
            words.into_iter()
                .filter(|&word| !self.is_filler_word(word))
                .collect::<Vec<_>>()
                .join(" ")
        };

        Ok(compressed)
    }

    fn is_filler_word(&self, word: &str) -> bool {
        matches!(word.to_lowercase().as_str(), 
            "really" | "very" | "quite" | "pretty" | "just" | "actually" | "basically")
    }

    fn calculate_energy_level(&self) -> f32 {
        (self.traits.cheerfulness + self.traits.humor) / 2.0
    }
}

impl PersonalityTraits {
    fn enthusiasm(&self) -> f32 {
        (self.cheerfulness + self.humor) / 2.0
    }
}

/// Context for personality decisions
#[derive(Debug, Clone)]
pub struct PersonalityContext {
    pub current_state: ADHDState,
    pub previous_state: Option<ADHDState>,
    pub metrics: BehavioralMetrics,
    pub time_of_day: String,
    pub recent_interactions: Vec<InteractionHistory>,
    pub work_context: WorkContext,
}

#[derive(Debug, Clone)]
pub struct InteractionHistory {
    pub message: String,
    pub user_response: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Current personality state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalityState {
    pub traits: PersonalityTraits,
    pub current_mood: CompanionMood,
    pub pun_streak: u32,
    pub energy_level: f32,
}

/// Tracks Skelly's current mood
pub struct MoodTracker {
    current_mood: CompanionMood,
    mood_history: Vec<(CompanionMood, chrono::DateTime<chrono::Utc>)>,
}

impl MoodTracker {
    pub fn new() -> Self {
        Self {
            current_mood: CompanionMood::Neutral,
            mood_history: Vec::new(),
        }
    }

    pub fn determine_mood(&mut self, context: &PersonalityContext) -> CompanionMood {
        let new_mood = match &context.current_state.state_type {
            crate::types::ADHDStateType::Flow { depth } => {
                if *depth > 0.8 {
                    CompanionMood::Happy
                } else {
                    CompanionMood::Supportive
                }
            }
            crate::types::ADHDStateType::Hyperfocus { .. } => CompanionMood::Excited,
            crate::types::ADHDStateType::Distracted { severity } => {
                if *severity > 0.7 {
                    CompanionMood::Concerned
                } else {
                    CompanionMood::Supportive
                }
            }
            crate::types::ADHDStateType::Transitioning => CompanionMood::Supportive,
            crate::types::ADHDStateType::Neutral => CompanionMood::Neutral,
        };

        self.update_mood(new_mood.clone());
        new_mood
    }

    pub fn get_current_mood(&self) -> CompanionMood {
        self.current_mood.clone()
    }

    fn update_mood(&mut self, new_mood: CompanionMood) {
        if self.current_mood != new_mood {
            self.mood_history.push((self.current_mood.clone(), chrono::Utc::now()));
            self.current_mood = new_mood;
        }
    }
}

/// Generates casual expressions for Skelly
pub struct ExpressionGenerator {
    expressions: HashMap<CompanionMood, Vec<String>>,
}

impl ExpressionGenerator {
    pub fn new() -> Self {
        let mut expressions = HashMap::new();

        expressions.insert(CompanionMood::Happy, vec![
            "😊".to_string(),
            "Nice!".to_string(),
            "Sweet!".to_string(),
            "Love it!".to_string(),
        ]);

        expressions.insert(CompanionMood::Excited, vec![
            "🎉".to_string(),
            "Awesome!".to_string(),
            "Woohoo!".to_string(),
            "That's the spirit!".to_string(),
        ]);

        expressions.insert(CompanionMood::Supportive, vec![
            "💪".to_string(),
            "You've got this".to_string(),
            "Hang in there".to_string(),
            "Take it easy".to_string(),
        ]);

        expressions.insert(CompanionMood::Concerned, vec![
            "🤗".to_string(),
            "No worries".to_string(),
            "It's okay".to_string(),
            "Let's regroup".to_string(),
        ]);

        expressions.insert(CompanionMood::Neutral, vec![
            "👍".to_string(),
            "Alright".to_string(),
            "Got it".to_string(),
            "Sure thing".to_string(),
        ]);

        expressions.insert(CompanionMood::Celebrating, vec![
            "🥳".to_string(),
            "Fantastic!".to_string(),
            "Amazing work!".to_string(),
            "You're on fire!".to_string(),
        ]);

        expressions.insert(CompanionMood::Sleepy, vec![
            "😴".to_string(),
            "Time for a break?".to_string(),
            "Rest up".to_string(),
            "Maybe some tea?".to_string(),
        ]);

        Self { expressions }
    }

    pub fn add_expression(&self, message: &str, mood: &CompanionMood) -> Result<String> {
        if let Some(expressions) = self.expressions.get(mood) {
            if let Some(expression) = expressions.choose(&mut rand::thread_rng()) {
                // Add expression at the end, separated by space
                return Ok(format!("{} {}", message, expression));
            }
        }
        
        Ok(message.to_string())
    }
}

/// Generates skeleton puns (used sparingly)
pub struct SkeletonPunGenerator {
    puns: Vec<SkeletonPun>,
    streak_count: u32,
    last_pun_time: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone)]
pub struct SkeletonPun {
    pub trigger_words: Vec<String>,
    pub pun_text: String,
    pub category: PunCategory,
}

#[derive(Debug, Clone)]
pub enum PunCategory {
    Bone,
    Skull,
    Spine,
    Funny,
    Supportive,
}

impl SkeletonPunGenerator {
    pub fn new() -> Self {
        let puns = vec![
            SkeletonPun {
                trigger_words: vec!["bone".to_string(), "problem".to_string()],
                pun_text: "I've got a bone to pick with distractions!".to_string(),
                category: PunCategory::Bone,
            },
            SkeletonPun {
                trigger_words: vec!["think".to_string(), "skull".to_string()],
                pun_text: "Let's put our skulls together on this".to_string(),
                category: PunCategory::Skull,
            },
            SkeletonPun {
                trigger_words: vec!["courage".to_string(), "spine".to_string()],
                pun_text: "You've got the spine for this!".to_string(),
                category: PunCategory::Spine,
            },
            SkeletonPun {
                trigger_words: vec!["core".to_string(), "marrow".to_string()],
                pun_text: "Getting to the marrow of it...".to_string(),
                category: PunCategory::Bone,
            },
            SkeletonPun {
                trigger_words: vec!["funny".to_string(), "humerus".to_string()],
                pun_text: "That's pretty humerus!".to_string(),
                category: PunCategory::Funny,
            },
            SkeletonPun {
                trigger_words: vec!["support".to_string(), "backbone".to_string()],
                pun_text: "I'll be your backbone through this!".to_string(),
                category: PunCategory::Supportive,
            },
        ];

        Self {
            puns,
            streak_count: 0,
            last_pun_time: None,
        }
    }

    pub fn add_pun(&mut self, message: &str) -> Result<String> {
        // Check if we should add a pun based on timing
        if let Some(last_time) = self.last_pun_time {
            let time_since = chrono::Utc::now() - last_time;
            if time_since.num_minutes() < 30 {
                // Don't pun too frequently
                return Ok(message.to_string());
            }
        }

        // Find a relevant pun
        let message_lower = message.to_lowercase();
        for pun in &self.puns {
            if pun.trigger_words.iter().any(|word| message_lower.contains(word)) {
                self.streak_count += 1;
                self.last_pun_time = Some(chrono::Utc::now());
                return Ok(format!("{} {}", message, pun.pun_text));
            }
        }

        // Random pun if no triggers found (very rarely)
        if rand::thread_rng().gen::<f32>() < 0.05 {
            if let Some(pun) = self.puns.choose(&mut rand::thread_rng()) {
                self.streak_count += 1;
                self.last_pun_time = Some(chrono::Utc::now());
                return Ok(format!("{} {}", message, pun.pun_text));
            }
        }

        Ok(message.to_string())
    }

    pub fn get_streak(&self) -> u32 {
        self.streak_count
    }
}

/// Adjusts tone based on user state
pub struct ToneAdjuster;

impl ToneAdjuster {
    pub fn new() -> Self {
        Self
    }

    pub fn adjust_tone(&self, message: &str, context: &PersonalityContext) -> Result<String> {
        match &context.current_state.state_type {
            crate::types::ADHDStateType::Flow { depth } if *depth > 0.7 => {
                // More subdued for deep flow states
                Ok(self.make_gentle(message))
            }
            crate::types::ADHDStateType::Distracted { severity } if *severity > 0.7 => {
                // More encouraging for high distraction
                Ok(self.make_encouraging(message))
            }
            crate::types::ADHDStateType::Hyperfocus { .. } => {
                // Brief and supportive for hyperfocus
                Ok(self.make_brief_supportive(message))
            }
            _ => Ok(message.to_string()),
        }
    }

    fn make_gentle(&self, message: &str) -> String {
        // Add gentle prefixes
        let gentle_prefixes = vec![
            "Just a gentle reminder: ",
            "When you're ready: ",
            "No rush, but ",
            "In your own time: ",
        ];

        if let Some(prefix) = gentle_prefixes.choose(&mut rand::thread_rng()) {
            format!("{}{}", prefix, message.to_lowercase())
        } else {
            message.to_string()
        }
    }

    fn make_encouraging(&self, message: &str) -> String {
        let encouraging_prefixes = vec![
            "Hey, you've got this! ",
            "Don't worry - ",
            "It's all good! ",
            "Take a breath: ",
        ];

        if let Some(prefix) = encouraging_prefixes.choose(&mut rand::thread_rng()) {
            format!("{}{}", prefix, message)
        } else {
            message.to_string()
        }
    }

    fn make_brief_supportive(&self, message: &str) -> String {
        // Make message more concise and supportive
        let words: Vec<&str> = message.split_whitespace().collect();
        if words.len() > 10 {
            // Shorten to key points
            format!("Quick tip: {}", words[..7].join(" "))
        } else {
            format!("💡 {}", message)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ADHDStateType;

    #[test]
    fn test_personality_application() {
        let traits = PersonalityTraits::default();
        let engine = PersonalityEngine::new(traits);
        
        let context = PersonalityContext {
            current_state: ADHDState {
                state_type: ADHDStateType::Flow { depth: 0.8 },
                confidence: 0.9,
                depth: Some(0.8),
                duration: 1000,
                metadata: std::collections::HashMap::new(),
            },
            previous_state: None,
            metrics: BehavioralMetrics {
                productive_time_ratio: 0.8,
                distraction_frequency: 0.2,
                focus_session_count: 5,
                average_session_length: 1800,
                recovery_time: 300,
                transition_smoothness: 0.7,
            },
            time_of_day: "afternoon".to_string(),
            recent_interactions: Vec::new(),
            work_context: WorkContext::default(),
        };

        let message = "Try taking a short break";
        let result = engine.apply(message.to_string(), &context).unwrap();
        
        // Should be modified in some way
        assert!(!result.is_empty());
    }

    #[test]
    fn test_mood_determination() {
        let mut tracker = MoodTracker::new();
        
        let flow_context = PersonalityContext {
            current_state: ADHDState {
                state_type: ADHDStateType::Flow { depth: 0.9 },
                confidence: 0.9,
                depth: Some(0.9),
                duration: 1000,
                metadata: std::collections::HashMap::new(),
            },
            previous_state: None,
            metrics: BehavioralMetrics {
                productive_time_ratio: 0.9,
                distraction_frequency: 0.1,
                focus_session_count: 3,
                average_session_length: 2000,
                recovery_time: 200,
                transition_smoothness: 0.8,
            },
            time_of_day: "morning".to_string(),
            recent_interactions: Vec::new(),
            work_context: WorkContext::default(),
        };

        let mood = tracker.determine_mood(&flow_context);
        assert!(matches!(mood, CompanionMood::Happy));
    }

    #[test]
    fn test_pun_generation() {
        let mut generator = SkeletonPunGenerator::new();
        
        let message_with_trigger = "I have a problem with focus";
        let result = generator.add_pun(message_with_trigger).unwrap();
        
        // Should contain the original message
        assert!(result.contains("problem with focus"));
    }

    #[test]
    fn test_brevity_enforcement() {
        let traits = PersonalityTraits::default();
        let engine = PersonalityEngine::new(traits);
        
        let long_message = "This is a very long message that contains way too many words and should be shortened to maintain the brief, helpful nature of Skelly's responses while still being useful and supportive to the user";
        
        let result = engine.ensure_brevity(long_message).unwrap();
        
        // Should be shorter than original
        assert!(result.len() < long_message.len());
    }

    #[test]
    fn test_tone_adjustment() {
        let adjuster = ToneAdjuster::new();
        
        let distracted_context = PersonalityContext {
            current_state: ADHDState {
                state_type: ADHDStateType::Distracted { severity: 0.8 },
                confidence: 0.9,
                depth: None,
                duration: 500,
                metadata: std::collections::HashMap::new(),
            },
            previous_state: None,
            metrics: BehavioralMetrics {
                productive_time_ratio: 0.3,
                distraction_frequency: 0.8,
                focus_session_count: 1,
                average_session_length: 500,
                recovery_time: 1000,
                transition_smoothness: 0.3,
            },
            time_of_day: "afternoon".to_string(),
            recent_interactions: Vec::new(),
            work_context: WorkContext::default(),
        };

        let message = "focus on your task";
        let result = adjuster.adjust_tone(message, &distracted_context).unwrap();
        
        // Should be more encouraging
        assert!(result.len() >= message.len());
    }
}