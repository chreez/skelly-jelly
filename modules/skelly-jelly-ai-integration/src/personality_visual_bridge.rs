//! Bridge between personality system and visual companion expressions
//!
//! Translates personality states and communications into visual expressions

use crate::error::{AIIntegrationError, Result};
use crate::personality_enhanced::{ExpertiseLevel, CommunicationPreferences, AttentionPreferences};
use crate::personality_integration::{EnhancedPersonalityResponse, CommunicationStyle};
use crate::types::{CompanionMood, ADHDState};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// Bridge that connects personality system with visual companion
pub struct PersonalityVisualBridge {
    expression_mappings: HashMap<PersonalityExpression, VisualExpression>,
    mood_translations: HashMap<CompanionMood, VisualMood>,
    gesture_generator: GestureGenerator,
    animation_coordinator: AnimationCoordinator,
    visual_preferences: VisualPreferences,
}

impl PersonalityVisualBridge {
    /// Create a new personality-visual bridge
    pub fn new() -> Self {
        Self {
            expression_mappings: Self::build_expression_mappings(),
            mood_translations: Self::build_mood_translations(),
            gesture_generator: GestureGenerator::new(),
            animation_coordinator: AnimationCoordinator::new(),
            visual_preferences: VisualPreferences::default(),
        }
    }
    
    /// Convert personality response into visual commands
    pub fn translate_to_visual(
        &self,
        personality_response: &EnhancedPersonalityResponse,
        user_state: &ADHDState,
        communication_prefs: &CommunicationPreferences,
    ) -> Result<VisualExpressionCommand> {
        // Determine base visual mood from personality mood
        let visual_mood = self.determine_visual_mood(personality_response, user_state)?;
        
        // Generate appropriate gestures and animations
        let gestures = self.gesture_generator.generate_gestures(
            &personality_response.message,
            &personality_response.expertise_level,
            communication_prefs,
        )?;
        
        // Coordinate animations with message timing
        let animation_sequence = self.animation_coordinator.coordinate_with_message(
            &personality_response.message,
            &visual_mood,
            &gestures,
            communication_prefs,
        )?;
        
        // Apply celebration effects if present
        let celebration_effects = if let Some(ref celebration) = personality_response.celebration {
            self.generate_celebration_effects(celebration, &personality_response.expertise_level)?
        } else {
            Vec::new()
        };
        
        Ok(VisualExpressionCommand {
            primary_mood: visual_mood,
            animation_sequence,
            gestures,
            celebration_effects,
            message_sync: MessageSyncConfig {
                display_duration: self.calculate_display_duration(&personality_response.message),
                fade_in_duration: 500,
                fade_out_duration: 300,
                sync_with_speech: false, // Could be enabled for voice synthesis
            },
            adaptation_hints: self.generate_adaptation_hints(personality_response),
            timestamp: Utc::now(),
        })
    }
    
    /// Update visual preferences based on user feedback
    pub fn update_visual_preferences(&mut self, feedback: &VisualFeedback) -> Result<()> {
        match feedback.feedback_type {
            VisualFeedbackType::TooDistracting => {
                self.visual_preferences.animation_intensity *= 0.8;
                self.visual_preferences.gesture_frequency *= 0.7;
            }
            VisualFeedbackType::TooSubtle => {
                self.visual_preferences.animation_intensity *= 1.2;
                self.visual_preferences.gesture_frequency *= 1.3;
            }
            VisualFeedbackType::PerfectTiming => {
                // Reinforce current timing settings
                self.visual_preferences.timing_confidence *= 1.1;
            }
            VisualFeedbackType::BadTiming => {
                self.visual_preferences.timing_confidence *= 0.9;
            }
            VisualFeedbackType::LoveIt => {
                // Reinforce all current settings
                self.visual_preferences.overall_satisfaction += 0.1;
            }
        }
        
        Ok(())
    }
    
    /// Get current visual expression for continuous display
    pub fn get_ambient_expression(
        &self,
        user_state: &ADHDState,
        expertise_level: &ExpertiseLevel,
        communication_prefs: &CommunicationPreferences,
    ) -> Result<AmbientExpression> {
        let base_mood = self.translate_adhd_state_to_mood(user_state);
        
        // Adjust expression intensity based on expertise level
        let intensity_modifier = match expertise_level {
            ExpertiseLevel::Expert => 0.7,    // More subtle for experts
            ExpertiseLevel::Intermediate => 1.0, // Standard intensity
            ExpertiseLevel::Beginner => 1.2,     // Slightly more expressive
        };
        
        // Generate subtle ambient gestures
        let ambient_gestures = self.gesture_generator.generate_ambient_gestures(
            user_state,
            &self.visual_preferences,
        )?;
        
        Ok(AmbientExpression {
            mood: base_mood,
            intensity: self.visual_preferences.animation_intensity * intensity_modifier,
            gestures: ambient_gestures,
            glow_intensity: self.calculate_glow_intensity(user_state, communication_prefs),
            melt_level: self.calculate_melt_level(user_state),
            particle_effects: self.generate_ambient_particles(user_state),
        })
    }
    
    // Private helper methods
    
    fn determine_visual_mood(
        &self,
        personality_response: &EnhancedPersonalityResponse,
        user_state: &ADHDState,
    ) -> Result<VisualMood> {
        // Start with base mood from user state
        let base_mood = self.translate_adhd_state_to_mood(user_state);
        
        // Modify based on personality response characteristics
        let message_sentiment = self.analyze_message_sentiment(&personality_response.message);
        
        // Combine factors to determine final visual mood
        Ok(match (base_mood, message_sentiment) {
            (VisualMood::Focused, MessageSentiment::Encouraging) => VisualMood::Supportive,
            (VisualMood::Distracted, MessageSentiment::Celebratory) => VisualMood::Uplifting,
            (VisualMood::Tired, MessageSentiment::Gentle) => VisualMood::Caring,
            (mood, MessageSentiment::Celebratory) => VisualMood::Celebrating,
            (mood, _) => mood, // Keep base mood for other cases
        })
    }
    
    fn translate_adhd_state_to_mood(&self, user_state: &ADHDState) -> VisualMood {
        match &user_state.state_type {
            crate::types::ADHDStateType::Flow { depth } => {
                if *depth > 0.8 {
                    VisualMood::Focused
                } else {
                    VisualMood::Supportive
                }
            }
            crate::types::ADHDStateType::Hyperfocus { intensity } => {
                if *intensity > 0.9 {
                    VisualMood::Intense
                } else {
                    VisualMood::Focused
                }
            }
            crate::types::ADHDStateType::Distracted { severity } => {
                if *severity > 0.7 {
                    VisualMood::Concerned
                } else {
                    VisualMood::Distracted
                }
            }
            crate::types::ADHDStateType::Transitioning => VisualMood::Supportive,
            crate::types::ADHDStateType::Neutral => VisualMood::Neutral,
        }
    }
    
    fn analyze_message_sentiment(&self, message: &str) -> MessageSentiment {
        let message_lower = message.to_lowercase();
        
        // Simple sentiment analysis based on keywords
        if message_lower.contains("celebration") || message_lower.contains("amazing") || 
           message_lower.contains("excellent") || message_lower.contains("great work") {
            MessageSentiment::Celebratory
        } else if message_lower.contains("gently") || message_lower.contains("when ready") ||
                 message_lower.contains("take your time") {
            MessageSentiment::Gentle
        } else if message_lower.contains("you've got this") || message_lower.contains("keep going") ||
                 message_lower.contains("you can") {
            MessageSentiment::Encouraging
        } else {
            MessageSentiment::Neutral
        }
    }
    
    fn calculate_display_duration(&self, message: &str) -> u32 {
        // Base duration on message length and complexity
        let word_count = message.split_whitespace().count();
        let base_duration = (word_count as f32 * 300.0) as u32; // ~300ms per word
        
        // Adjust based on visual preferences
        let adjusted = (base_duration as f32 * self.visual_preferences.timing_multiplier) as u32;
        
        // Clamp to reasonable bounds
        adjusted.clamp(2000, 8000)
    }
    
    fn generate_celebration_effects(
        &self,
        celebration: &str,
        expertise_level: &ExpertiseLevel,
    ) -> Result<Vec<CelebrationEffect>> {
        let intensity = match expertise_level {
            ExpertiseLevel::Expert => 0.6,    // Subtle for experts
            ExpertiseLevel::Intermediate => 0.8, // Moderate
            ExpertiseLevel::Beginner => 1.0,     // Full celebration
        };
        
        Ok(vec![
            CelebrationEffect {
                effect_type: CelebrationEffectType::Particles,
                duration: (2000.0 * intensity) as u32,
                intensity,
                color_scheme: self.get_celebration_colors(celebration),
                delay: 0,
            },
            CelebrationEffect {
                effect_type: CelebrationEffectType::Glow,
                duration: (1500.0 * intensity) as u32,
                intensity: intensity * 0.8,
                color_scheme: self.get_celebration_colors(celebration),
                delay: 200,
            },
        ])
    }
    
    fn get_celebration_colors(&self, celebration: &str) -> Vec<String> {
        // Different color schemes based on celebration type
        if celebration.contains("flow") || celebration.contains("focus") {
            vec!["#4A90E2".to_string(), "#5CB85C".to_string()] // Blue-green for focus
        } else if celebration.contains("task") || celebration.contains("done") {
            vec!["#FFD700".to_string(), "#FFA500".to_string()] // Gold for completion
        } else {
            vec!["#FF69B4".to_string(), "#DDA0DD".to_string()] // Pink-purple for general celebration
        }
    }
    
    fn calculate_glow_intensity(
        &self,
        user_state: &ADHDState,
        communication_prefs: &CommunicationPreferences,
    ) -> f32 {
        let base_intensity = match &user_state.state_type {
            crate::types::ADHDStateType::Flow { depth } => *depth * 0.6,
            crate::types::ADHDStateType::Hyperfocus { intensity } => *intensity * 0.4, // Dimmer for hyperfocus
            crate::types::ADHDStateType::Distracted { .. } => 0.3,
            crate::types::ADHDStateType::Transitioning => 0.4,
            _ => 0.3,
        };
        
        // Adjust based on user's intensity preference
        let preference_modifier = match communication_prefs.intensity_preference {
            crate::personality_enhanced::IntensityPreference::Subtle => 0.7,
            crate::personality_enhanced::IntensityPreference::Moderate => 1.0,
            crate::personality_enhanced::IntensityPreference::Energetic => 1.3,
        };
        
        (base_intensity * preference_modifier).clamp(0.1, 1.0)
    }
    
    fn calculate_melt_level(&self, user_state: &ADHDState) -> f32 {
        match &user_state.state_type {
            crate::types::ADHDStateType::Distracted { severity } => {
                // Higher distraction = more melting
                (*severity * 60.0).clamp(0.0, 80.0)
            }
            crate::types::ADHDStateType::Flow { depth } => {
                // Deep flow reduces melting
                (30.0 - (*depth * 30.0)).clamp(0.0, 30.0)
            }
            crate::types::ADHDStateType::Hyperfocus { .. } => 0.0, // No melting in hyperfocus
            _ => 20.0, // Neutral melt level
        }
    }
    
    fn generate_ambient_particles(&self, user_state: &ADHDState) -> ParticleConfig {
        match &user_state.state_type {
            crate::types::ADHDStateType::Flow { depth } => ParticleConfig {
                count: (*depth * 20.0) as u32,
                speed: 0.3,
                color: "#4A90E2".to_string(),
                pattern: ParticlePattern::Gentle,
            },
            crate::types::ADHDStateType::Hyperfocus { intensity } => ParticleConfig {
                count: (*intensity * 10.0) as u32, // Fewer particles in hyperfocus
                speed: 0.1,
                color: "#E27B4A".to_string(),
                pattern: ParticlePattern::Focused,
            },
            _ => ParticleConfig {
                count: 5,
                speed: 0.2,
                color: "#FFFFFF".to_string(),
                pattern: ParticlePattern::Ambient,
            },
        }
    }
    
    fn generate_adaptation_hints(&self, personality_response: &EnhancedPersonalityResponse) -> Vec<AdaptationHint> {
        let mut hints = Vec::new();
        
        // Add hint about expertise level adaptation
        hints.push(AdaptationHint {
            category: "expertise".to_string(),
            suggestion: format!("Adapted for {:?} level user", personality_response.expertise_level),
            confidence: personality_response.adaptation_confidence,
        });
        
        // Add hint about communication style
        hints.push(AdaptationHint {
            category: "communication".to_string(),
            suggestion: format!("Using {} communication style", personality_response.communication_style.formality),
            confidence: 0.8,
        });
        
        hints
    }
    
    fn build_expression_mappings() -> HashMap<PersonalityExpression, VisualExpression> {
        let mut mappings = HashMap::new();
        
        mappings.insert(
            PersonalityExpression::Encouraging,
            VisualExpression {
                gesture: Some(GestureType::NodApproval),
                glow_color: "#32CD32".to_string(),
                particle_effect: Some(ParticleEffectType::Sparkle),
                animation: "supportive_nod".to_string(),
                duration: 2000,
            }
        );
        
        mappings.insert(
            PersonalityExpression::Celebrating,
            VisualExpression {
                gesture: Some(GestureType::Celebration),
                glow_color: "#FFD700".to_string(),
                particle_effect: Some(ParticleEffectType::Burst),
                animation: "celebration_dance".to_string(),
                duration: 3000,
            }
        );
        
        mappings.insert(
            PersonalityExpression::Gentle,
            VisualExpression {
                gesture: Some(GestureType::GentleWave),
                glow_color: "#87CEEB".to_string(),
                particle_effect: Some(ParticleEffectType::Gentle),
                animation: "gentle_attention".to_string(),
                duration: 1500,
            }
        );
        
        mappings
    }
    
    fn build_mood_translations() -> HashMap<CompanionMood, VisualMood> {
        let mut translations = HashMap::new();
        
        translations.insert(CompanionMood::Happy, VisualMood::Supportive);
        translations.insert(CompanionMood::Excited, VisualMood::Celebrating);
        translations.insert(CompanionMood::Supportive, VisualMood::Supportive);
        translations.insert(CompanionMood::Concerned, VisualMood::Concerned);
        translations.insert(CompanionMood::Celebrating, VisualMood::Celebrating);
        translations.insert(CompanionMood::Neutral, VisualMood::Neutral);
        translations.insert(CompanionMood::Sleepy, VisualMood::Tired);
        
        translations
    }
}

/// Generates appropriate gestures based on personality context
pub struct GestureGenerator {
    gesture_library: HashMap<GestureContext, Vec<GestureSequence>>,
}

impl GestureGenerator {
    pub fn new() -> Self {
        Self {
            gesture_library: Self::build_gesture_library(),
        }
    }
    
    /// Generate gestures for a specific message and context
    pub fn generate_gestures(
        &self,
        message: &str,
        expertise_level: &ExpertiseLevel,
        communication_prefs: &CommunicationPreferences,
    ) -> Result<Vec<Gesture>> {
        let context = self.determine_gesture_context(message, expertise_level, communication_prefs);
        
        if let Some(sequences) = self.gesture_library.get(&context) {
            // Select appropriate sequence (in practice would be more sophisticated)
            if let Some(sequence) = sequences.first() {
                return Ok(sequence.gestures.clone());
            }
        }
        
        // Default gesture for unknown contexts
        Ok(vec![Gesture {
            gesture_type: GestureType::Subtle,
            duration: 1000,
            intensity: 0.5,
            delay: 0,
        }])
    }
    
    /// Generate ambient gestures for background expression
    pub fn generate_ambient_gestures(
        &self,
        user_state: &ADHDState,
        visual_prefs: &VisualPreferences,
    ) -> Result<Vec<Gesture>> {
        let base_gestures = match &user_state.state_type {
            crate::types::ADHDStateType::Flow { .. } => vec![
                Gesture {
                    gesture_type: GestureType::GentleBreathe,
                    duration: 3000,
                    intensity: 0.4,
                    delay: 0,
                }
            ],
            crate::types::ADHDStateType::Hyperfocus { .. } => vec![
                Gesture {
                    gesture_type: GestureType::StillPresence,
                    duration: 5000,
                    intensity: 0.2,
                    delay: 0,
                }
            ],
            crate::types::ADHDStateType::Distracted { .. } => vec![
                Gesture {
                    gesture_type: GestureType::GentleFloat,
                    duration: 2000,
                    intensity: 0.6,
                    delay: 0,
                }
            ],
            _ => vec![
                Gesture {
                    gesture_type: GestureType::IdleBob,
                    duration: 4000,
                    intensity: 0.3,
                    delay: 0,
                }
            ],
        };
        
        Ok(base_gestures)
    }
    
    fn determine_gesture_context(
        &self,
        message: &str,
        expertise_level: &ExpertiseLevel,
        communication_prefs: &CommunicationPreferences,
    ) -> GestureContext {
        // Analyze message content
        let message_lower = message.to_lowercase();
        
        if message_lower.contains("celebration") || message_lower.contains("great") {
            match expertise_level {
                ExpertiseLevel::Expert => GestureContext::SubtleCelebration,
                _ => GestureContext::Celebration,
            }
        } else if message_lower.contains("focus") || message_lower.contains("concentrate") {
            GestureContext::Guidance
        } else if message_lower.contains("break") || message_lower.contains("rest") {
            GestureContext::Gentle
        } else {
            match communication_prefs.intensity_preference {
                crate::personality_enhanced::IntensityPreference::Subtle => GestureContext::Minimal,
                crate::personality_enhanced::IntensityPreference::Energetic => GestureContext::Expressive,
                _ => GestureContext::Supportive,
            }
        }
    }
    
    fn build_gesture_library() -> HashMap<GestureContext, Vec<GestureSequence>> {
        let mut library = HashMap::new();
        
        // Celebration gestures
        library.insert(GestureContext::Celebration, vec![
            GestureSequence {
                name: "celebration_sequence".to_string(),
                gestures: vec![
                    Gesture {
                        gesture_type: GestureType::Celebration,
                        duration: 1500,
                        intensity: 0.9,
                        delay: 0,
                    },
                    Gesture {
                        gesture_type: GestureType::JoyfulBounce,
                        duration: 2000,
                        intensity: 0.7,
                        delay: 500,
                    },
                ],
            }
        ]);
        
        // Subtle celebration for experts
        library.insert(GestureContext::SubtleCelebration, vec![
            GestureSequence {
                name: "subtle_approval".to_string(),
                gestures: vec![
                    Gesture {
                        gesture_type: GestureType::NodApproval,
                        duration: 1000,
                        intensity: 0.5,
                        delay: 0,
                    },
                ],
            }
        ]);
        
        // Supportive gestures
        library.insert(GestureContext::Supportive, vec![
            GestureSequence {
                name: "supportive_presence".to_string(),
                gestures: vec![
                    Gesture {
                        gesture_type: GestureType::GentleWave,
                        duration: 1200,
                        intensity: 0.6,
                        delay: 0,
                    },
                ],
            }
        ]);
        
        library
    }
}

/// Coordinates animations with message display timing
pub struct AnimationCoordinator {
    timing_profiles: HashMap<AnimationProfile, TimingConfig>,
}

impl AnimationCoordinator {
    pub fn new() -> Self {
        Self {
            timing_profiles: Self::build_timing_profiles(),
        }
    }
    
    /// Coordinate animation sequence with message display
    pub fn coordinate_with_message(
        &self,
        message: &str,
        mood: &VisualMood,
        gestures: &[Gesture],
        communication_prefs: &CommunicationPreferences,
    ) -> Result<AnimationSequence> {
        let profile = self.determine_animation_profile(mood, communication_prefs);
        let timing_config = self.timing_profiles.get(&profile)
            .unwrap_or(&TimingConfig::default());
        
        let message_duration = self.calculate_message_duration(message);
        
        Ok(AnimationSequence {
            phases: vec![
                AnimationPhase {
                    name: "intro".to_string(),
                    duration: timing_config.intro_duration,
                    animations: vec!["fade_in".to_string()],
                    sync_point: Some(SyncPoint::MessageStart),
                },
                AnimationPhase {
                    name: "main".to_string(),
                    duration: message_duration,
                    animations: self.select_main_animations(mood, gestures),
                    sync_point: Some(SyncPoint::MessageDisplay),
                },
                AnimationPhase {
                    name: "outro".to_string(),
                    duration: timing_config.outro_duration,
                    animations: vec!["fade_out".to_string()],
                    sync_point: Some(SyncPoint::MessageEnd),
                },
            ],
            total_duration: timing_config.intro_duration + message_duration + timing_config.outro_duration,
        })
    }
    
    fn determine_animation_profile(
        &self,
        mood: &VisualMood,
        communication_prefs: &CommunicationPreferences,
    ) -> AnimationProfile {
        match (mood, &communication_prefs.intensity_preference) {
            (VisualMood::Celebrating, _) => AnimationProfile::Celebratory,
            (_, crate::personality_enhanced::IntensityPreference::Subtle) => AnimationProfile::Subtle,
            (_, crate::personality_enhanced::IntensityPreference::Energetic) => AnimationProfile::Energetic,
            _ => AnimationProfile::Standard,
        }
    }
    
    fn calculate_message_duration(&self, message: &str) -> u32 {
        // Estimate reading time based on message length
        let word_count = message.split_whitespace().count();
        (word_count as f32 * 250.0) as u32 // ~250ms per word for reading
    }
    
    fn select_main_animations(&self, mood: &VisualMood, gestures: &[Gesture]) -> Vec<String> {
        let mut animations = Vec::new();
        
        // Add mood-based base animation
        animations.push(match mood {
            VisualMood::Celebrating => "celebration_base".to_string(),
            VisualMood::Supportive => "supportive_base".to_string(),
            VisualMood::Focused => "focused_base".to_string(),
            VisualMood::Gentle => "gentle_base".to_string(),
            _ => "neutral_base".to_string(),
        });
        
        // Add gesture-specific animations
        for gesture in gestures {
            animations.push(format!("gesture_{:?}", gesture.gesture_type));
        }
        
        animations
    }
    
    fn build_timing_profiles() -> HashMap<AnimationProfile, TimingConfig> {
        let mut profiles = HashMap::new();
        
        profiles.insert(AnimationProfile::Celebratory, TimingConfig {
            intro_duration: 800,
            outro_duration: 1200,
            transition_speed: 1.5,
        });
        
        profiles.insert(AnimationProfile::Subtle, TimingConfig {
            intro_duration: 300,
            outro_duration: 400,
            transition_speed: 0.8,
        });
        
        profiles.insert(AnimationProfile::Energetic, TimingConfig {
            intro_duration: 600,
            outro_duration: 800,
            transition_speed: 1.3,
        });
        
        profiles.insert(AnimationProfile::Standard, TimingConfig {
            intro_duration: 500,
            outro_duration: 600,
            transition_speed: 1.0,
        });
        
        profiles
    }
}

// Type definitions for visual expression system

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualExpressionCommand {
    pub primary_mood: VisualMood,
    pub animation_sequence: AnimationSequence,
    pub gestures: Vec<Gesture>,
    pub celebration_effects: Vec<CelebrationEffect>,
    pub message_sync: MessageSyncConfig,
    pub adaptation_hints: Vec<AdaptationHint>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmbientExpression {
    pub mood: VisualMood,
    pub intensity: f32,
    pub gestures: Vec<Gesture>,
    pub glow_intensity: f32,
    pub melt_level: f32,
    pub particle_effects: ParticleConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VisualMood {
    Neutral,
    Supportive,
    Celebrating,
    Focused,
    Intense,
    Distracted,
    Concerned,
    Uplifting,
    Caring,
    Tired,
    Gentle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationSequence {
    pub phases: Vec<AnimationPhase>,
    pub total_duration: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationPhase {
    pub name: String,
    pub duration: u32,
    pub animations: Vec<String>,
    pub sync_point: Option<SyncPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncPoint {
    MessageStart,
    MessageDisplay,
    MessageEnd,
    UserResponse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gesture {
    pub gesture_type: GestureType,
    pub duration: u32,
    pub intensity: f32,
    pub delay: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GestureType {
    NodApproval,
    Celebration,
    GentleWave,
    JoyfulBounce,
    Subtle,
    GentleBreathe,
    StillPresence,
    GentleFloat,
    IdleBob,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CelebrationEffect {
    pub effect_type: CelebrationEffectType,
    pub duration: u32,
    pub intensity: f32,
    pub color_scheme: Vec<String>,
    pub delay: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CelebrationEffectType {
    Particles,
    Glow,
    Rainbow,
    Sparkle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageSyncConfig {
    pub display_duration: u32,
    pub fade_in_duration: u32,
    pub fade_out_duration: u32,
    pub sync_with_speech: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticleConfig {
    pub count: u32,
    pub speed: f32,
    pub color: String,
    pub pattern: ParticlePattern,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParticlePattern {
    Gentle,
    Focused,
    Ambient,
    Celebration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptationHint {
    pub category: String,
    pub suggestion: String,
    pub confidence: f32,
}

#[derive(Debug, Clone)]
pub struct VisualPreferences {
    pub animation_intensity: f32,
    pub gesture_frequency: f32,
    pub timing_multiplier: f32,
    pub timing_confidence: f32,
    pub overall_satisfaction: f32,
}

impl Default for VisualPreferences {
    fn default() -> Self {
        Self {
            animation_intensity: 1.0,
            gesture_frequency: 1.0,
            timing_multiplier: 1.0,
            timing_confidence: 0.8,
            overall_satisfaction: 0.5,
        }
    }
}

#[derive(Debug, Clone)]
pub struct VisualFeedback {
    pub feedback_type: VisualFeedbackType,
    pub context: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub enum VisualFeedbackType {
    TooDistracting,
    TooSubtle,
    PerfectTiming,
    BadTiming,
    LoveIt,
}

// Internal enums and structs

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
enum PersonalityExpression {
    Encouraging,
    Celebrating,
    Gentle,
    Guidance,
    Supportive,
}

#[derive(Debug, Clone)]
struct VisualExpression {
    gesture: Option<GestureType>,
    glow_color: String,
    particle_effect: Option<ParticleEffectType>,
    animation: String,
    duration: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum ParticleEffectType {
    Sparkle,
    Burst,
    Gentle,
    Trail,
}

#[derive(Debug, Clone)]
enum MessageSentiment {
    Celebratory,
    Encouraging,
    Gentle,
    Neutral,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
enum GestureContext {
    Celebration,
    SubtleCelebration,
    Supportive,
    Guidance,
    Gentle,
    Minimal,
    Expressive,
}

#[derive(Debug, Clone)]
struct GestureSequence {
    name: String,
    gestures: Vec<Gesture>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
enum AnimationProfile {
    Celebratory,
    Subtle,
    Energetic,
    Standard,
}

#[derive(Debug, Clone)]
struct TimingConfig {
    intro_duration: u32,
    outro_duration: u32,
    transition_speed: f32,
}

impl Default for TimingConfig {
    fn default() -> Self {
        Self {
            intro_duration: 500,
            outro_duration: 600,
            transition_speed: 1.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::personality_enhanced::CommunicationPreferences;
    use crate::personality_integration::EnhancedPersonalityResponse;
    use crate::types::{ADHDState, ADHDStateType};
    use std::collections::HashMap;
    
    #[test]
    fn test_personality_visual_bridge_creation() {
        let bridge = PersonalityVisualBridge::new();
        assert!(!bridge.expression_mappings.is_empty());
        assert!(!bridge.mood_translations.is_empty());
    }
    
    #[test]
    fn test_adhd_state_to_visual_mood_translation() {
        let bridge = PersonalityVisualBridge::new();
        
        let flow_state = ADHDState {
            state_type: ADHDStateType::Flow { depth: 0.9 },
            confidence: 0.9,
            depth: Some(0.9),
            duration: 5000,
            metadata: HashMap::new(),
        };
        
        let visual_mood = bridge.translate_adhd_state_to_mood(&flow_state);
        assert!(matches!(visual_mood, VisualMood::Focused));
    }
    
    #[test]
    fn test_ambient_expression_generation() {
        let bridge = PersonalityVisualBridge::new();
        let communication_prefs = CommunicationPreferences::default();
        
        let user_state = ADHDState {
            state_type: ADHDStateType::Neutral,
            confidence: 0.8,
            depth: None,
            duration: 1000,
            metadata: HashMap::new(),
        };
        
        let ambient = bridge.get_ambient_expression(
            &user_state,
            &ExpertiseLevel::Intermediate,
            &communication_prefs,
        ).unwrap();
        
        assert!(matches!(ambient.mood, VisualMood::Neutral));
        assert!(ambient.intensity > 0.0);
        assert!(ambient.glow_intensity > 0.0);
    }
    
    #[test]
    fn test_gesture_generation() {
        let generator = GestureGenerator::new();
        let communication_prefs = CommunicationPreferences::default();
        
        let gestures = generator.generate_gestures(
            "Great work on that task!",
            &ExpertiseLevel::Beginner,
            &communication_prefs,
        ).unwrap();
        
        assert!(!gestures.is_empty());
        assert!(gestures[0].duration > 0);
    }
    
    #[test]
    fn test_celebration_effect_intensity_by_expertise() {
        let bridge = PersonalityVisualBridge::new();
        
        let expert_effects = bridge.generate_celebration_effects(
            "Nice work",
            &ExpertiseLevel::Expert,
        ).unwrap();
        
        let beginner_effects = bridge.generate_celebration_effects(
            "Nice work",
            &ExpertiseLevel::Beginner,
        ).unwrap();
        
        // Expert celebrations should be more subtle
        assert!(expert_effects[0].intensity < beginner_effects[0].intensity);
    }
    
    #[test]
    fn test_animation_coordination() {
        let coordinator = AnimationCoordinator::new();
        let communication_prefs = CommunicationPreferences::default();
        let gestures = vec![Gesture {
            gesture_type: GestureType::NodApproval,
            duration: 1000,
            intensity: 0.5,
            delay: 0,
        }];
        
        let sequence = coordinator.coordinate_with_message(
            "This is a test message",
            &VisualMood::Supportive,
            &gestures,
            &communication_prefs,
        ).unwrap();
        
        assert!(!sequence.phases.is_empty());
        assert!(sequence.total_duration > 0);
    }
}