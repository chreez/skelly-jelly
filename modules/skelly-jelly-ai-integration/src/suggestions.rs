//! Suggestion generation with template fallback
//!
//! Generates helpful, personality-driven suggestions using LLM or templates.

use crate::error::{AIIntegrationError, Result};
use crate::llm::{LLMManager, GenerationResult};
use crate::personality::{PersonalityEngine, PersonalityContext};
use crate::types::{
    LLMContext, GenerationParams, TemplateSuggestion, TemplateCategory,
    PersonalityModifier, ModifierType, ADHDState, CompanionMood, GenerationMethod
};
use rand::seq::SliceRandom;
use std::collections::HashMap;
use std::sync::Arc;

/// Generates suggestions using LLM or template fallback
pub struct SuggestionGenerator {
    template_manager: TemplateManager,
    llm_manager: Arc<LLMManager>,
    personality_engine: std::sync::Mutex<PersonalityEngine>,
    validator: SuggestionValidator,
}

impl SuggestionGenerator {
    pub fn new(
        llm_manager: Arc<LLMManager>,
        personality_engine: PersonalityEngine,
    ) -> Self {
        Self {
            template_manager: TemplateManager::new(),
            llm_manager,
            personality_engine: std::sync::Mutex::new(personality_engine),
            validator: SuggestionValidator::new(),
        }
    }

    /// Generate a suggestion based on context
    pub async fn generate(
        &self,
        context: LLMContext,
        urgency: SuggestionUrgency,
        allow_api: bool,
    ) -> Result<SuggestionResult> {
        // Decide whether to use template or LLM
        let use_template = self.should_use_template(&context, urgency);

        let raw_suggestion = if use_template {
            self.generate_template_suggestion(&context)?
        } else {
            self.generate_llm_suggestion(&context, allow_api).await?
        };

        // Apply personality modifications
        let personality_context = self.build_personality_context(&context);
        let personalized = self.personality_engine.lock().unwrap().apply(raw_suggestion.text, &personality_context)?;

        // Validate suggestion
        let validated = self.validator.validate(&personalized, &context)?;

        // Generate animation hints before moving validated.text
        let animation_hints = self.generate_animation_hints(&validated.text, &personality_context);

        Ok(SuggestionResult {
            text: validated.text,
            method: raw_suggestion.method,
            confidence: validated.confidence,
            animation_hints,
            follow_up_available: self.has_follow_up(&context),
            tokens_used: raw_suggestion.tokens_used,
        })
    }

    /// Update template library
    pub async fn update_templates(&mut self, templates: Vec<TemplateSuggestion>) -> Result<()> {
        self.template_manager.update_templates(templates)
    }

    fn should_use_template(&self, context: &LLMContext, urgency: SuggestionUrgency) -> bool {
        // Use templates for high urgency or simple contexts
        match urgency {
            SuggestionUrgency::High | SuggestionUrgency::Critical => true,
            SuggestionUrgency::Low => {
                // Use LLM if available for low urgency situations
                !self.llm_manager.has_local_model()
            }
            SuggestionUrgency::Normal => {
                // Balance between quality and speed
                context.max_tokens < 200 || !self.llm_manager.has_local_model()
            }
        }
    }

    fn generate_template_suggestion(&self, context: &LLMContext) -> Result<RawSuggestion> {
        let template = self.template_manager.get_suggestion_for_context(context)?;
        
        Ok(RawSuggestion {
            text: template.text,
            method: GenerationMethod::Template { template_id: template.template_id },
            tokens_used: None,
        })
    }

    async fn generate_llm_suggestion(
        &self,
        context: &LLMContext,
        allow_api: bool,
    ) -> Result<RawSuggestion> {
        let prompt = self.build_prompt(context)?;
        let params = GenerationParams {
            max_tokens: context.max_tokens.min(200), // Keep suggestions brief
            temperature: 0.7,
            top_p: 0.9,
            repeat_penalty: 1.1,
            stop_sequences: vec!["\n\n".to_string(), "Human:".to_string()],
            timeout_ms: 5000,
        };

        let result = self.llm_manager.generate(&prompt, params, allow_api).await?;

        Ok(RawSuggestion {
            text: result.text,
            method: GenerationMethod::LocalLLM { model_name: result.model_info },
            tokens_used: Some(result.tokens_used),
        })
    }

    fn build_prompt(&self, context: &LLMContext) -> Result<String> {
        let mut prompt = String::new();
        
        // Add system prompt
        prompt.push_str(&context.system_prompt);
        prompt.push_str("\n\n");

        // Add context information
        prompt.push_str("Current situation:\n");
        prompt.push_str(&format!("- Behavioral context: {}\n", context.behavioral_context));
        prompt.push_str(&format!("- Work context: {}\n", context.work_context));
        prompt.push_str(&format!("- Intervention type: {}\n", context.intervention_type));

        // Add user preferences
        prompt.push_str(&format!(
            "- Message style: {:?}\n",
            context.user_preferences.message_style
        ));

        // Add specific request
        prompt.push_str("\nProvide a helpful, brief suggestion (1-2 sentences max):\n");

        Ok(prompt)
    }

    fn build_personality_context(&self, context: &LLMContext) -> PersonalityContext {
        // This is simplified - in practice would need more sophisticated conversion
        let current_state = ADHDState {
            state_type: crate::types::ADHDStateType::Neutral, // Default
            confidence: 0.8,
            depth: None,
            duration: 0,
            metadata: HashMap::new(),
        };

        PersonalityContext {
            current_state,
            previous_state: None,
            metrics: crate::types::BehavioralMetrics {
                productive_time_ratio: 0.5,
                distraction_frequency: 0.3,
                focus_session_count: 1,
                average_session_length: 1000,
                recovery_time: 500,
                transition_smoothness: 0.7,
            },
            time_of_day: "unknown".to_string(),
            recent_interactions: Vec::new(),
        }
    }

    fn generate_animation_hints(&self, text: &str, context: &PersonalityContext) -> Vec<String> {
        let mut hints = Vec::new();

        // Generate hints based on text content and mood
        if text.contains("great") || text.contains("awesome") || text.contains("celebration") {
            hints.push("celebration".to_string());
        } else if text.contains("break") || text.contains("rest") {
            hints.push("sleepy".to_string());
        } else if text.contains("focus") || text.contains("concentrate") {
            hints.push("focused".to_string());
        } else {
            hints.push("supportive".to_string());
        }

        hints
    }

    fn has_follow_up(&self, context: &LLMContext) -> bool {
        // Determine if follow-up suggestions are available
        matches!(context.intervention_type.as_str(), "suggestion" | "gentle_nudge")
    }
}

/// Template manager for quick responses
pub struct TemplateManager {
    templates: HashMap<TemplateKey, Vec<TemplateSuggestion>>,
}

impl TemplateManager {
    pub fn new() -> Self {
        let mut manager = Self {
            templates: HashMap::new(),
        };
        manager.initialize_default_templates();
        manager
    }

    pub fn get_suggestion_for_context(&self, context: &LLMContext) -> Result<TemplateResponse> {
        let key = self.determine_template_key(context);
        
        let templates = self.templates.get(&key)
            .or_else(|| self.templates.get(&TemplateKey::default()))
            .ok_or(AIIntegrationError::TemplateNotFound)?;

        let template = templates.choose(&mut rand::thread_rng())
            .ok_or(AIIntegrationError::TemplateNotFound)?;

        let text = self.fill_template(template, context)?;

        Ok(TemplateResponse {
            text,
            template_id: template.template_id.clone(),
        })
    }

    pub fn update_templates(&mut self, templates: Vec<TemplateSuggestion>) -> Result<()> {
        // Group templates by key
        for template in templates {
            let key = TemplateKey {
                category: template.category.clone(),
                urgency: None, // Could be extended
            };
            
            self.templates.entry(key)
                .or_insert_with(Vec::new)
                .push(template);
        }
        
        Ok(())
    }

    fn initialize_default_templates(&mut self) {
        let default_templates = vec![
            // Encouragement templates
            TemplateSuggestion {
                template_id: "encourage_focus_1".to_string(),
                category: TemplateCategory::Encouragement,
                variations: vec![
                    "You've got this! Try focusing on just the next small step.".to_string(),
                    "Great job staying aware! Let's gently redirect that focus.".to_string(),
                    "Nice catch! Take a breath and ease back into your task.".to_string(),
                ],
                personality_modifiers: vec![
                    PersonalityModifier {
                        modifier_type: ModifierType::SupportivePhrase,
                        content: "I believe in you! ".to_string(),
                        conditions: vec!["supportiveness > 0.8".to_string()],
                    }
                ],
            },

            // Break reminders
            TemplateSuggestion {
                template_id: "break_reminder_1".to_string(),
                category: TemplateCategory::BreakReminder,
                variations: vec![
                    "Time for a quick break? Your brain has earned it!".to_string(),
                    "How about stretching those bones for a minute?".to_string(),
                    "Perfect time to hydrate and reset! 💧".to_string(),
                ],
                personality_modifiers: vec![
                    PersonalityModifier {
                        modifier_type: ModifierType::SkeletonPun,
                        content: "Time to give those bones a rest! ".to_string(),
                        conditions: vec!["pun_frequency > 0.05".to_string()],
                    }
                ],
            },

            // Gentle nudges
            TemplateSuggestion {
                template_id: "gentle_nudge_1".to_string(),
                category: TemplateCategory::GentleNudge,
                variations: vec![
                    "Hey, noticed you might be drifting. No worries - let's gently refocus.".to_string(),
                    "Wandering mind? Totally normal! Take a sec to reconnect with your task.".to_string(),
                    "Brain taking a little detour? Let's guide it back when you're ready.".to_string(),
                ],
                personality_modifiers: vec![
                    PersonalityModifier {
                        modifier_type: ModifierType::CasualPhrase,
                        content: "No biggie - ".to_string(),
                        conditions: vec!["casualness > 0.7".to_string()],
                    }
                ],
            },

            // Celebrations
            TemplateSuggestion {
                template_id: "celebration_1".to_string(),
                category: TemplateCategory::Celebration,
                variations: vec![
                    "Amazing work! You're absolutely crushing it! 🎉".to_string(),
                    "Look at you go! That focus is paying off beautifully.".to_string(),
                    "Fantastic! Your effort is really showing results.".to_string(),
                ],
                personality_modifiers: vec![
                    PersonalityModifier {
                        modifier_type: ModifierType::HumorousAddition,
                        content: " You're the marrow of success! ".to_string(),
                        conditions: vec!["humor > 0.7".to_string(), "pun_frequency > 0.1".to_string()],
                    }
                ],
            },

            // Suggestions
            TemplateSuggestion {
                template_id: "suggestion_1".to_string(),
                category: TemplateCategory::Suggestion,
                variations: vec![
                    "Try the 2-minute rule: just commit to 2 minutes on your task.".to_string(),
                    "Break this down into one tiny, concrete next step.".to_string(),
                    "Set a 15-minute timer and see how much you can tackle.".to_string(),
                ],
                personality_modifiers: vec![
                    PersonalityModifier {
                        modifier_type: ModifierType::SupportivePhrase,
                        content: "You've got the tools for this! ".to_string(),
                        conditions: vec!["supportiveness > 0.7".to_string()],
                    }
                ],
            },
        ];

        for template in default_templates {
            let key = TemplateKey {
                category: template.category.clone(),
                urgency: None,
            };
            
            self.templates.entry(key)
                .or_insert_with(Vec::new)
                .push(template);
        }
    }

    fn determine_template_key(&self, context: &LLMContext) -> TemplateKey {
        let category = match context.intervention_type.as_str() {
            "encouragement" => TemplateCategory::Encouragement,
            "gentle_nudge" => TemplateCategory::GentleNudge,
            "celebration" => TemplateCategory::Celebration,
            "suggestion" => TemplateCategory::Suggestion,
            "break_reminder" => TemplateCategory::BreakReminder,
            _ => TemplateCategory::Suggestion,
        };

        TemplateKey {
            category,
            urgency: None,
        }
    }

    fn fill_template(&self, template: &TemplateSuggestion, context: &LLMContext) -> Result<String> {
        // Choose a random variation
        let base_text = template.variations
            .choose(&mut rand::thread_rng())
            .ok_or(AIIntegrationError::TemplateNotFound)?
            .clone();

        // Apply personality modifiers based on conditions
        let mut result = base_text;
        for modifier in &template.personality_modifiers {
            if self.check_modifier_conditions(&modifier.conditions, context) {
                result = match modifier.modifier_type {
                    ModifierType::SupportivePhrase => format!("{}{}", modifier.content, result),
                    ModifierType::CasualPhrase => format!("{}{}", modifier.content, result),
                    ModifierType::HumorousAddition => format!("{}{}", result, modifier.content),
                    ModifierType::SkeletonPun => format!("{}{}", modifier.content, result),
                };
            }
        }

        Ok(result)
    }

    fn check_modifier_conditions(&self, conditions: &[String], context: &LLMContext) -> bool {
        // Simple condition checking - in practice would be more sophisticated
        for condition in conditions {
            if condition.contains("supportiveness > 0.8") && 
               context.user_preferences.personality_traits.supportiveness > 0.8 {
                return true;
            }
            if condition.contains("casualness > 0.7") && 
               context.user_preferences.personality_traits.casualness > 0.7 {
                return true;
            }
            if condition.contains("humor > 0.7") && 
               context.user_preferences.personality_traits.humor > 0.7 {
                return true;
            }
            if condition.contains("pun_frequency > 0.1") && 
               context.user_preferences.personality_traits.pun_frequency > 0.1 {
                return true;
            }
        }
        false
    }
}

/// Validates generated suggestions
pub struct SuggestionValidator;

impl SuggestionValidator {
    pub fn new() -> Self {
        Self
    }

    pub fn validate(&self, text: &str, context: &LLMContext) -> Result<ValidatedSuggestion> {
        let mut confidence: f32 = 1.0;
        let mut issues = Vec::new();

        // Check length
        if text.len() > 300 {
            confidence -= 0.2;
            issues.push("Text too long".to_string());
        }

        // Check for empty or very short text
        if text.trim().len() < 10 {
            confidence -= 0.5;
            issues.push("Text too short".to_string());
        }

        // Check for inappropriate content (basic)
        let inappropriate_words = vec!["stupid", "dumb", "failure", "worthless"];
        for word in inappropriate_words {
            if text.to_lowercase().contains(word) {
                confidence -= 0.8;
                issues.push("Inappropriate language detected".to_string());
            }
        }

        // Check relevance to intervention type
        if !self.is_relevant_to_intervention(text, &context.intervention_type) {
            confidence -= 0.3;
            issues.push("Low relevance to intervention type".to_string());
        }

        // Ensure minimum confidence
        confidence = confidence.max(0.1);

        Ok(ValidatedSuggestion {
            text: text.to_string(),
            confidence,
            issues,
        })
    }

    fn is_relevant_to_intervention(&self, text: &str, intervention_type: &str) -> bool {
        match intervention_type {
            "encouragement" => {
                text.to_lowercase().contains("you") || 
                text.to_lowercase().contains("great") ||
                text.to_lowercase().contains("good")
            }
            "break_reminder" => {
                text.to_lowercase().contains("break") ||
                text.to_lowercase().contains("rest") ||
                text.to_lowercase().contains("pause")
            }
            "celebration" => {
                text.to_lowercase().contains("amazing") ||
                text.to_lowercase().contains("fantastic") ||
                text.to_lowercase().contains("great")
            }
            _ => true, // Default to relevant
        }
    }
}

// Supporting types

#[derive(Debug, Clone)]
pub struct SuggestionResult {
    pub text: String,
    pub method: GenerationMethod,
    pub confidence: f32,
    pub animation_hints: Vec<String>,
    pub follow_up_available: bool,
    pub tokens_used: Option<u32>,
}

// GenerationMethod is imported from types module

#[derive(Debug, Clone)]
struct RawSuggestion {
    text: String,
    method: GenerationMethod,
    tokens_used: Option<u32>,
}

#[derive(Debug, Clone)]
struct ValidatedSuggestion {
    text: String,
    confidence: f32,
    issues: Vec<String>,
}

#[derive(Debug, Clone)]
struct TemplateResponse {
    text: String,
    template_id: String,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct TemplateKey {
    category: TemplateCategory,
    urgency: Option<SuggestionUrgency>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum SuggestionUrgency {
    Low,
    Normal,
    High,
    Critical,
}

impl Default for TemplateKey {
    fn default() -> Self {
        Self {
            category: TemplateCategory::Suggestion,
            urgency: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{PersonalityTraits, UserPreferences, MessageStyle, InterventionFrequency, UserPrivacyLevel, APIConsent};

    #[test]
    fn test_template_generation() {
        let manager = TemplateManager::new();
        
        let context = LLMContext {
            system_prompt: "test".to_string(),
            behavioral_context: "focused".to_string(),
            work_context: "coding".to_string(),
            intervention_type: "encouragement".to_string(),
            user_preferences: UserPreferences {
                intervention_frequency: InterventionFrequency::Moderate,
                message_style: MessageStyle::Encouraging,
                privacy_level: UserPrivacyLevel::LocalOnly,
                personality_traits: PersonalityTraits::default(),
                api_consent: APIConsent {
                    openai_allowed: false,
                    anthropic_allowed: false,
                    consent_timestamp: None,
                    monthly_limit_usd: None,
                },
            },
            max_tokens: 200,
        };

        let result = manager.get_suggestion_for_context(&context).unwrap();
        assert!(!result.text.is_empty());
        assert!(result.text.len() < 300);
    }

    #[test]
    fn test_suggestion_validation() {
        let validator = SuggestionValidator::new();
        
        let context = LLMContext {
            system_prompt: "test".to_string(),
            behavioral_context: "test".to_string(),
            work_context: "test".to_string(),
            intervention_type: "encouragement".to_string(),
            user_preferences: UserPreferences {
                intervention_frequency: InterventionFrequency::Moderate,
                message_style: MessageStyle::Encouraging,
                privacy_level: UserPrivacyLevel::LocalOnly,
                personality_traits: PersonalityTraits::default(),
                api_consent: APIConsent {
                    openai_allowed: false,
                    anthropic_allowed: false,
                    consent_timestamp: None,
                    monthly_limit_usd: None,
                },
            },
            max_tokens: 200,
        };

        // Test good suggestion
        let good_text = "You're doing great! Keep up the good work.";
        let result = validator.validate(good_text, &context).unwrap();
        assert!(result.confidence > 0.7);

        // Test problematic suggestion
        let bad_text = "You're stupid and will never succeed at this task.";
        let result = validator.validate(bad_text, &context).unwrap();
        assert!(result.confidence < 0.5);
    }

    #[test]
    fn test_template_key_determination() {
        let manager = TemplateManager::new();
        
        let context = LLMContext {
            system_prompt: "test".to_string(),
            behavioral_context: "test".to_string(),
            work_context: "test".to_string(),
            intervention_type: "celebration".to_string(),
            user_preferences: UserPreferences {
                intervention_frequency: InterventionFrequency::Moderate,
                message_style: MessageStyle::Encouraging,
                privacy_level: UserPrivacyLevel::LocalOnly,
                personality_traits: PersonalityTraits::default(),
                api_consent: APIConsent {
                    openai_allowed: false,
                    anthropic_allowed: false,
                    consent_timestamp: None,
                    monthly_limit_usd: None,
                },
            },
            max_tokens: 200,
        };

        let key = manager.determine_template_key(&context);
        assert!(matches!(key.category, TemplateCategory::Celebration));
    }
}