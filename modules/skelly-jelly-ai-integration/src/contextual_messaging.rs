//! Contextual Messaging System
//! 
//! Generates context-aware interventions based on:
//! - Detected work type (coding, writing, designing)
//! - Current focus state and behavioral patterns
//! - User preferences and feedback history
//! - Intervention effectiveness tracking

use crate::context_detection::{WorkType, DocumentType, DesignType};
use crate::intervention_timing::{
    FocusState, InterventionType, CodingIssueCategory, WritingIssueCategory, 
    DesignIssueCategory, FocusStrategy, WellnessType, UserResponse
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use rand::seq::SliceRandom;

/// A contextual message with animation hints and follow-up suggestions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextualMessage {
    pub message_id: Uuid,
    pub text: String,
    pub tone: MessageTone,
    pub animation_hints: Vec<AnimationHint>,
    pub follow_up_suggestions: Vec<String>,
    pub confidence: f32,
    pub personalization_score: f32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessageTone {
    Encouraging,    // Positive, supportive
    Gentle,         // Soft, non-intrusive  
    Informative,    // Helpful, educational
    Playful,        // Light, with skeleton humor
    Urgent,         // More direct for important issues
    Celebratory,    // For achievements and wins
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationHint {
    pub animation_type: String,
    pub emotion: String,
    pub duration_ms: u32,
    pub intensity: f32,
}

/// User preferences for message personalization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessagePersonalization {
    pub preferred_tone: MessageTone,
    pub humor_level: f32,           // 0.0-1.0, how much skeleton humor to use
    pub directness: f32,            // 0.0-1.0, how direct vs gentle
    pub technical_level: f32,       // 0.0-1.0, technical vs simple explanations
    pub encouragement_frequency: f32, // 0.0-1.0, how often to include encouragement
    pub blocked_phrases: Vec<String>, // Phrases user doesn't want to see
}

impl Default for MessagePersonalization {
    fn default() -> Self {
        Self {
            preferred_tone: MessageTone::Encouraging,
            humor_level: 0.3,
            directness: 0.6,
            technical_level: 0.7,
            encouragement_frequency: 0.8,
            blocked_phrases: vec![],
        }
    }
}

/// Template for different types of contextual messages
#[derive(Debug, Clone, Serialize, Deserialize)]
struct MessageTemplate {
    id: String,
    category: InterventionType,
    tone: MessageTone,
    templates: Vec<String>,
    placeholders: Vec<String>,
    min_confidence_threshold: f32,
}

/// Content database for contextual interventions
pub struct ContextualMessageGenerator {
    coding_templates: HashMap<CodingIssueCategory, Vec<MessageTemplate>>,
    writing_templates: HashMap<WritingIssueCategory, Vec<MessageTemplate>>,
    design_templates: HashMap<DesignIssueCategory, Vec<MessageTemplate>>,
    focus_templates: HashMap<FocusStrategy, Vec<MessageTemplate>>,
    wellness_templates: HashMap<WellnessType, Vec<MessageTemplate>>,
    encouragement_templates: Vec<MessageTemplate>,
    user_feedback_history: HashMap<String, Vec<UserFeedback>>,
    personalization: MessagePersonalization,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct UserFeedback {
    message_template_id: String,
    response: UserResponse,
    effectiveness_score: Option<f32>,
    timestamp: DateTime<Utc>,
    context: String,
}

impl ContextualMessageGenerator {
    pub fn new(personalization: MessagePersonalization) -> Self {
        let mut generator = Self {
            coding_templates: HashMap::new(),
            writing_templates: HashMap::new(),
            design_templates: HashMap::new(),
            focus_templates: HashMap::new(),
            wellness_templates: HashMap::new(),
            encouragement_templates: Vec::new(),
            user_feedback_history: HashMap::new(),
            personalization,
        };

        generator.initialize_templates();
        generator
    }

    /// Generate a contextual message based on work type, focus state, and intervention type
    pub fn generate_message(
        &mut self,
        work_type: &WorkType,
        focus_state: &FocusState,
        intervention_type: &InterventionType,
    ) -> Result<ContextualMessage, String> {
        let templates = self.get_relevant_templates(intervention_type)?;
        
        // Filter templates based on focus state and user preferences
        let suitable_templates = self.filter_templates(&templates, focus_state);
        
        if suitable_templates.is_empty() {
            return Err("No suitable templates found for this context".to_string());
        }

        // Select template based on effectiveness history
        let selected_template = self.select_best_template(&suitable_templates, work_type)?;
        
        // Generate message content
        let message_text = self.fill_template(&selected_template, work_type, focus_state)?;
        
        // Apply personalization
        let personalized_text = self.apply_personalization(message_text, &selected_template.tone);
        
        // Generate animation hints
        let animation_hints = self.generate_animation_hints(&selected_template.tone, focus_state);
        
        // Generate follow-up suggestions
        let follow_ups = self.generate_follow_ups(intervention_type, work_type);

        let confidence = self.calculate_message_confidence(&selected_template, work_type, focus_state);
        let personalization_score = self.calculate_personalization_score(&selected_template);

        Ok(ContextualMessage {
            message_id: Uuid::new_v4(),
            text: personalized_text,
            tone: selected_template.tone.clone(),
            animation_hints,
            follow_up_suggestions: follow_ups,
            confidence,
            personalization_score,
            created_at: Utc::now(),
        })
    }

    /// Record user feedback for a message to improve future selections
    pub fn record_feedback(
        &mut self,
        _message_id: Uuid,
        template_id: String,
        response: UserResponse,
        effectiveness_score: Option<f32>,
        context: String,
    ) {
        let feedback = UserFeedback {
            message_template_id: template_id.clone(),
            response,
            effectiveness_score,
            timestamp: Utc::now(),
            context,
        };

        self.user_feedback_history
            .entry(template_id.clone())
            .or_insert_with(Vec::new)
            .push(feedback);

        // Keep only last 50 feedback entries per template
        if let Some(history) = self.user_feedback_history.get_mut(&template_id) {
            if history.len() > 50 {
                history.remove(0);
            }
        }
    }

    /// Update user personalization preferences based on feedback patterns
    pub fn update_personalization(&mut self, new_preferences: MessagePersonalization) {
        self.personalization = new_preferences;
    }

    /// Initialize all message templates
    fn initialize_templates(&mut self) {
        self.initialize_coding_templates();
        self.initialize_writing_templates();
        self.initialize_design_templates();
        self.initialize_focus_templates();
        self.initialize_wellness_templates();
        self.initialize_encouragement_templates();
    }

    fn initialize_coding_templates(&mut self) {
        // Debugging help templates
        let debugging_templates = vec![
            MessageTemplate {
                id: "debug_gentle".to_string(),
                category: InterventionType::CodingAssistance {
                    language: None,
                    issue_category: CodingIssueCategory::DebuggingHelp,
                },
                tone: MessageTone::Gentle,
                templates: vec![
                    "Stuck on a bug? Try adding some console.log() statements to see what's happening step by step.".to_string(),
                    "Debugging can be tricky! Consider using your debugger to step through the code line by line.".to_string(),
                    "When code doesn't behave as expected, rubber duck debugging often helps - explain the problem out loud!".to_string(),
                ],
                placeholders: vec!["language".to_string()],
                min_confidence_threshold: 0.6,
            },
            MessageTemplate {
                id: "debug_technical".to_string(),
                category: InterventionType::CodingAssistance {
                    language: None,
                    issue_category: CodingIssueCategory::DebuggingHelp,
                },
                tone: MessageTone::Informative,
                templates: vec![
                    "Try isolating the issue: comment out code sections to narrow down where the problem starts.".to_string(),
                    "Check your error logs and stack traces - they often point directly to the issue location.".to_string(),
                    "Consider writing a minimal test case that reproduces the bug consistently.".to_string(),
                ],
                placeholders: vec!["language".to_string()],
                min_confidence_threshold: 0.7,
            },
        ];

        self.coding_templates.insert(CodingIssueCategory::DebuggingHelp, debugging_templates);

        // Syntax error templates
        let syntax_templates = vec![
            MessageTemplate {
                id: "syntax_gentle".to_string(),
                category: InterventionType::CodingAssistance {
                    language: None,
                    issue_category: CodingIssueCategory::SyntaxError,
                },
                tone: MessageTone::Gentle,
                templates: vec![
                    "Syntax errors happen to everyone! Check for missing brackets, semicolons, or quotes.".to_string(),
                    "Your IDE is highlighting something - it's usually trying to help with syntax issues.".to_string(),
                    "Take a quick break and look at the code with fresh eyes - syntax errors often jump out!".to_string(),
                ],
                placeholders: vec!["language".to_string()],
                min_confidence_threshold: 0.8,
            },
        ];

        self.coding_templates.insert(CodingIssueCategory::SyntaxError, syntax_templates);
    }

    fn initialize_writing_templates(&mut self) {
        // Structure help templates
        let structure_templates = vec![
            MessageTemplate {
                id: "structure_gentle".to_string(),
                category: InterventionType::WritingSupport {
                    document_type: "general".to_string(),
                    issue_category: WritingIssueCategory::StructureHelp,
                },
                tone: MessageTone::Gentle,
                templates: vec![
                    "Having trouble with structure? Try outlining your main points first.".to_string(),
                    "Consider using headings to break up your content - it helps both you and readers.".to_string(),
                    "One idea per paragraph usually works well - keeps things clear and focused.".to_string(),
                ],
                placeholders: vec!["document_type".to_string()],
                min_confidence_threshold: 0.6,
            },
        ];

        self.writing_templates.insert(WritingIssueCategory::StructureHelp, structure_templates);

        // Clarity improvement templates
        let clarity_templates = vec![
            MessageTemplate {
                id: "clarity_informative".to_string(),
                category: InterventionType::WritingSupport {
                    document_type: "general".to_string(),
                    issue_category: WritingIssueCategory::ClarityImprovement,
                },
                tone: MessageTone::Informative,
                templates: vec![
                    "Try reading your sentences out loud - if you stumble, your readers might too.".to_string(),
                    "Shorter sentences often communicate more clearly than longer ones.".to_string(),
                    "Consider replacing jargon with simpler terms unless writing for specialists.".to_string(),
                ],
                placeholders: vec!["document_type".to_string()],
                min_confidence_threshold: 0.7,
            },
        ];

        self.writing_templates.insert(WritingIssueCategory::ClarityImprovement, clarity_templates);
    }

    fn initialize_design_templates(&mut self) {
        // Layout suggestion templates
        let layout_templates = vec![
            MessageTemplate {
                id: "layout_informative".to_string(),
                category: InterventionType::DesignGuidance {
                    design_type: "ui".to_string(),
                    issue_category: DesignIssueCategory::LayoutSuggestion,
                },
                tone: MessageTone::Informative,
                templates: vec![
                    "Consider using whitespace to guide the viewer's eye through your design.".to_string(),
                    "Grid systems can help create visual consistency across your layout.".to_string(),
                    "Try the rule of thirds for more visually interesting compositions.".to_string(),
                ],
                placeholders: vec!["design_type".to_string()],
                min_confidence_threshold: 0.6,
            },
        ];

        self.design_templates.insert(DesignIssueCategory::LayoutSuggestion, layout_templates);
    }

    fn initialize_focus_templates(&mut self) {
        // Pomodoro suggestion templates
        let pomodoro_templates = vec![
            MessageTemplate {
                id: "pomodoro_encouraging".to_string(),
                category: InterventionType::FocusSupport {
                    strategy: FocusStrategy::PomodoroSuggestion,
                },
                tone: MessageTone::Encouraging,
                templates: vec![
                    "How about a focused 25-minute session? Set a timer and dive deep!".to_string(),
                    "Pomodoro time! Pick one task and give it your full attention for 25 minutes.".to_string(),
                    "Let's try a focused sprint - 25 minutes on one task, then a well-deserved break.".to_string(),
                ],
                placeholders: vec![],
                min_confidence_threshold: 0.5,
            },
        ];

        self.focus_templates.insert(FocusStrategy::PomodoroSuggestion, pomodoro_templates);

        // Break reminder templates
        let break_templates = vec![
            MessageTemplate {
                id: "break_gentle".to_string(),
                category: InterventionType::FocusSupport {
                    strategy: FocusStrategy::BreakReminder,
                },
                tone: MessageTone::Gentle,
                templates: vec![
                    "You've been at this for a while - how about a quick 5-minute break?".to_string(),
                    "Even skeletons need to rest their bones! Time for a short break.".to_string(),
                    "Your brain deserves a breather. Step away for a few minutes?".to_string(),
                ],
                placeholders: vec![],
                min_confidence_threshold: 0.7,
            },
        ];

        self.focus_templates.insert(FocusStrategy::BreakReminder, break_templates);
    }

    fn initialize_wellness_templates(&mut self) {
        // Hydration reminders
        let hydration_templates = vec![
            MessageTemplate {
                id: "hydration_playful".to_string(),
                category: InterventionType::WellnessReminder {
                    reminder_type: WellnessType::Hydration,
                },
                tone: MessageTone::Playful,
                templates: vec![
                    "Time to hydrate! Even skeleton bones need water to stay strong. 💧".to_string(),
                    "Grab some water - your brain is about 75% water and it's doing hard work!".to_string(),
                    "Hydration checkpoint! Your future self will thank you. 🥤".to_string(),
                ],
                placeholders: vec![],
                min_confidence_threshold: 0.4,
            },
        ];

        self.wellness_templates.insert(WellnessType::Hydration, hydration_templates);
    }

    fn initialize_encouragement_templates(&mut self) {
        self.encouragement_templates = vec![
            MessageTemplate {
                id: "general_encouragement".to_string(),
                category: InterventionType::Encouragement {
                    context: "general".to_string(),
                },
                tone: MessageTone::Encouraging,
                templates: vec![
                    "You're making great progress! Keep up the good work.".to_string(),
                    "Every line of code, every word written, every design iteration - it all adds up!".to_string(),
                    "Challenges are what make us grow. You've got this! 💪".to_string(),
                    "Small consistent steps lead to big achievements. You're on the right track!".to_string(),
                ],
                placeholders: vec![],
                min_confidence_threshold: 0.3,
            },
        ];
    }

    fn get_relevant_templates(&self, intervention_type: &InterventionType) -> Result<Vec<MessageTemplate>, String> {
        match intervention_type {
            InterventionType::CodingAssistance { issue_category, .. } => {
                self.coding_templates
                    .get(issue_category)
                    .cloned()
                    .ok_or_else(|| format!("No templates for coding issue: {:?}", issue_category))
            },
            InterventionType::WritingSupport { issue_category, .. } => {
                self.writing_templates
                    .get(issue_category)
                    .cloned()
                    .ok_or_else(|| format!("No templates for writing issue: {:?}", issue_category))
            },
            InterventionType::DesignGuidance { issue_category, .. } => {
                self.design_templates
                    .get(issue_category)
                    .cloned()
                    .ok_or_else(|| format!("No templates for design issue: {:?}", issue_category))
            },
            InterventionType::FocusSupport { strategy } => {
                self.focus_templates
                    .get(strategy)
                    .cloned()
                    .ok_or_else(|| format!("No templates for focus strategy: {:?}", strategy))
            },
            InterventionType::WellnessReminder { reminder_type } => {
                self.wellness_templates
                    .get(reminder_type)
                    .cloned()
                    .ok_or_else(|| format!("No templates for wellness type: {:?}", reminder_type))
            },
            InterventionType::Encouragement { .. } => {
                Ok(self.encouragement_templates.clone())
            },
        }
    }

    fn filter_templates(&self, templates: &[MessageTemplate], focus_state: &FocusState) -> Vec<MessageTemplate> {
        templates.iter()
            .filter(|template| self.is_template_suitable(template, focus_state))
            .cloned()
            .collect()
    }

    fn is_template_suitable(&self, template: &MessageTemplate, focus_state: &FocusState) -> bool {
        match focus_state {
            FocusState::Flow { depth, .. } => {
                // In flow state, prefer gentle, non-intrusive messages
                if *depth > 0.7 {
                    matches!(template.tone, MessageTone::Gentle)
                } else {
                    true
                }
            },
            FocusState::Distracted { severity, .. } => {
                // When distracted, prefer encouraging or informative messages
                if *severity > 0.7 {
                    matches!(template.tone, MessageTone::Encouraging | MessageTone::Informative)
                } else {
                    true
                }
            },
            FocusState::Break { .. } => {
                // During breaks, prefer gentle or playful messages
                matches!(template.tone, MessageTone::Gentle | MessageTone::Playful | MessageTone::Encouraging)
            },
            _ => true, // All templates suitable for other states
        }
    }

    fn select_best_template(&self, templates: &[MessageTemplate], work_type: &WorkType) -> Result<MessageTemplate, String> {
        if templates.is_empty() {
            return Err("No templates to select from".to_string());
        }

        // Score templates based on effectiveness history and context match
        let mut scored_templates: Vec<(f32, &MessageTemplate)> = templates.iter()
            .map(|template| {
                let effectiveness_score = self.get_template_effectiveness_score(&template.id);
                let context_score = self.get_context_match_score(template, work_type);
                let total_score = effectiveness_score * 0.6 + context_score * 0.4;
                (total_score, template)
            })
            .collect();

        // Sort by score (highest first)
        scored_templates.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

        // Select from top 3 templates (add some randomness)
        let top_templates: Vec<&MessageTemplate> = scored_templates
            .into_iter()
            .take(3)
            .map(|(_, template)| template)
            .collect();

        top_templates.choose(&mut rand::thread_rng())
            .cloned()
            .cloned()
            .ok_or_else(|| "Failed to select template".to_string())
    }

    fn get_template_effectiveness_score(&self, template_id: &str) -> f32 {
        if let Some(feedback_history) = self.user_feedback_history.get(template_id) {
            if feedback_history.is_empty() {
                return 0.5; // Neutral score for new templates
            }

            let total_score: f32 = feedback_history.iter()
                .map(|feedback| {
                    match feedback.response {
                        UserResponse::Helpful => 1.0,
                        UserResponse::ActionTaken => 1.0,
                        UserResponse::NotHelpful => 0.2,
                        UserResponse::Dismissed => 0.3,
                        UserResponse::Ignored => 0.4,
                    }
                })
                .sum();

            total_score / feedback_history.len() as f32
        } else {
            0.5 // Neutral score for templates with no history
        }
    }

    fn get_context_match_score(&self, template: &MessageTemplate, work_type: &WorkType) -> f32 {
        // Higher score for templates that match the current work context
        match (&template.category, work_type) {
            (InterventionType::CodingAssistance { .. }, WorkType::Coding { .. }) => 1.0,
            (InterventionType::WritingSupport { .. }, WorkType::Writing { .. }) => 1.0,
            (InterventionType::DesignGuidance { .. }, WorkType::Designing { .. }) => 1.0,
            (InterventionType::FocusSupport { .. }, _) => 0.8, // Focus support is generally applicable
            (InterventionType::WellnessReminder { .. }, _) => 0.7, // Wellness is always relevant
            (InterventionType::Encouragement { .. }, _) => 0.6, // Encouragement works for any context
            _ => 0.4, // Partial match or unknown context
        }
    }

    fn fill_template(&self, template: &MessageTemplate, work_type: &WorkType, _focus_state: &FocusState) -> Result<String, String> {
        let template_text = template.templates
            .choose(&mut rand::thread_rng())
            .ok_or_else(|| "No template text available".to_string())?;

        let mut filled_text = template_text.clone();

        // Replace placeholders with actual values
        if template.placeholders.contains(&"language".to_string()) {
            if let WorkType::Coding { language: Some(lang), .. } = work_type {
                filled_text = filled_text.replace("{language}", lang);
            } else {
                filled_text = filled_text.replace("{language}", "your code");
            }
        }

        if template.placeholders.contains(&"document_type".to_string()) {
            let doc_type = match work_type {
                WorkType::Writing { document_type, .. } => {
                    match document_type {
                        DocumentType::Technical => "technical document",
                        DocumentType::Creative => "creative writing",
                        DocumentType::Academic => "academic paper",
                        DocumentType::Business => "business document",
                        DocumentType::Personal => "personal notes",
                        DocumentType::Unknown => "document",
                    }
                },
                _ => "document",
            };
            filled_text = filled_text.replace("{document_type}", doc_type);
        }

        Ok(filled_text)
    }

    fn apply_personalization(&self, text: String, tone: &MessageTone) -> String {
        let mut personalized = text;

        // Apply humor level (skeleton puns)
        if self.personalization.humor_level > 0.5 && matches!(tone, MessageTone::Playful | MessageTone::Encouraging) {
            if rand::random::<f32>() < self.personalization.humor_level * 0.3 {
                personalized = self.add_skeleton_humor(personalized);
            }
        }

        // Apply directness level
        if self.personalization.directness < 0.4 {
            personalized = self.make_more_gentle(personalized);
        } else if self.personalization.directness > 0.8 {
            personalized = self.make_more_direct(personalized);
        }

        // Check for blocked phrases
        for blocked in &self.personalization.blocked_phrases {
            if personalized.contains(blocked) {
                personalized = personalized.replace(blocked, ""); // Remove blocked phrases
            }
        }

        personalized
    }

    fn add_skeleton_humor(&self, text: String) -> String {
        let skeleton_phrases = vec![
            " - that's bone-afide advice!",
            " I've got a funny bone about this!",
            " - no bones about it!",
            " Let's get to the backbone of it.",
        ];

        if let Some(phrase) = skeleton_phrases.choose(&mut rand::thread_rng()) {
            format!("{}{}", text, phrase)
        } else {
            text
        }
    }

    fn make_more_gentle(&self, text: String) -> String {
        text.replace("You should", "You might want to")
            .replace("Try", "Consider")
            .replace("!", ".")
            .replace("Check", "Maybe check")
    }

    fn make_more_direct(&self, text: String) -> String {
        text.replace("How about", "")
            .replace("Consider", "")
            .replace("might want to", "should")
            .replace("Maybe", "")
    }

    fn generate_animation_hints(&self, tone: &MessageTone, focus_state: &FocusState) -> Vec<AnimationHint> {
        let base_duration = match focus_state {
            FocusState::Flow { .. } => 1000,      // Short, gentle animations
            FocusState::Distracted { .. } => 2000, // Slightly longer to get attention
            FocusState::Break { .. } => 1500,     // Relaxed animations
            _ => 1500,
        };

        let (animation_type, emotion, intensity) = match tone {
            MessageTone::Encouraging => ("supportive", "happy", 0.7),
            MessageTone::Gentle => ("gentle", "caring", 0.4),
            MessageTone::Informative => ("thinking", "focused", 0.6),
            MessageTone::Playful => ("happy", "excited", 0.8),
            MessageTone::Urgent => ("alert", "concerned", 0.9),
            MessageTone::Celebratory => ("celebration", "excited", 1.0),
        };

        vec![AnimationHint {
            animation_type: animation_type.to_string(),
            emotion: emotion.to_string(),
            duration_ms: base_duration,
            intensity,
        }]
    }

    fn generate_follow_ups(&self, intervention_type: &InterventionType, _work_type: &WorkType) -> Vec<String> {
        match intervention_type {
            InterventionType::CodingAssistance { .. } => {
                vec![
                    "Need help with testing this code?".to_string(),
                    "Want suggestions for code organization?".to_string(),
                    "Interested in performance optimization tips?".to_string(),
                ]
            },
            InterventionType::WritingSupport { .. } => {
                vec![
                    "Need help with grammar or style?".to_string(),
                    "Want suggestions for better flow?".to_string(),
                    "Interested in readability improvements?".to_string(),
                ]
            },
            InterventionType::FocusSupport { .. } => {
                vec![
                    "Want to set a focus timer?".to_string(),
                    "Need help prioritizing tasks?".to_string(),
                    "Interested in productivity techniques?".to_string(),
                ]
            },
            _ => vec![],
        }
    }

    fn calculate_message_confidence(&self, template: &MessageTemplate, work_type: &WorkType, _focus_state: &FocusState) -> f32 {
        let base_confidence = template.min_confidence_threshold;
        let context_bonus = self.get_context_match_score(template, work_type) * 0.2;
        let effectiveness_bonus = self.get_template_effectiveness_score(&template.id) * 0.1;
        
        (base_confidence + context_bonus + effectiveness_bonus).min(1.0)
    }

    fn calculate_personalization_score(&self, template: &MessageTemplate) -> f32 {
        // Score based on how well the template matches user preferences
        let tone_match = if template.tone == self.personalization.preferred_tone { 0.3 } else { 0.1 };
        let blocked_phrase_penalty = if self.personalization.blocked_phrases.iter()
            .any(|phrase| template.templates.iter().any(|t| t.contains(phrase))) { -0.2 } else { 0.0 };
        
        (0.5 + tone_match + blocked_phrase_penalty).clamp(0.0, 1.0)
    }

    /// Get statistics about message effectiveness
    pub fn get_effectiveness_stats(&self) -> MessageEffectivenessStats {
        let total_templates = self.get_total_template_count();
        let templates_with_feedback = self.user_feedback_history.len();
        
        let total_feedback: usize = self.user_feedback_history.values()
            .map(|history| history.len())
            .sum();

        let avg_effectiveness = if !self.user_feedback_history.is_empty() {
            self.user_feedback_history.values()
                .map(|history| {
                    history.iter()
                        .map(|feedback| match feedback.response {
                            UserResponse::Helpful | UserResponse::ActionTaken => 1.0,
                            UserResponse::NotHelpful => 0.0,
                            UserResponse::Dismissed => 0.3,
                            UserResponse::Ignored => 0.4,
                        })
                        .sum::<f32>() / history.len() as f32
                })
                .sum::<f32>() / self.user_feedback_history.len() as f32
        } else {
            0.0
        };

        MessageEffectivenessStats {
            total_templates,
            templates_with_feedback,
            total_feedback_responses: total_feedback,
            average_effectiveness: avg_effectiveness,
            personalization_score: self.calculate_overall_personalization_score(),
        }
    }

    fn get_total_template_count(&self) -> usize {
        self.coding_templates.values().map(|v| v.len()).sum::<usize>()
            + self.writing_templates.values().map(|v| v.len()).sum::<usize>()
            + self.design_templates.values().map(|v| v.len()).sum::<usize>()
            + self.focus_templates.values().map(|v| v.len()).sum::<usize>()
            + self.wellness_templates.values().map(|v| v.len()).sum::<usize>()
            + self.encouragement_templates.len()
    }

    fn calculate_overall_personalization_score(&self) -> f32 {
        // This would be more sophisticated in a real implementation
        if self.user_feedback_history.is_empty() {
            0.5 // Default score
        } else {
            let helpful_responses = self.user_feedback_history.values()
                .flatten()
                .filter(|feedback| matches!(feedback.response, UserResponse::Helpful | UserResponse::ActionTaken))
                .count();
            
            let total_responses = self.user_feedback_history.values()
                .flatten()
                .count();

            if total_responses > 0 {
                helpful_responses as f32 / total_responses as f32
            } else {
                0.5
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageEffectivenessStats {
    pub total_templates: usize,
    pub templates_with_feedback: usize,
    pub total_feedback_responses: usize,
    pub average_effectiveness: f32,
    pub personalization_score: f32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context_detection::WorkType;
    use crate::intervention_timing::{FocusState, InterventionType, CodingIssueCategory};

    #[test]
    fn test_coding_message_generation() {
        let personalization = MessagePersonalization::default();
        let mut generator = ContextualMessageGenerator::new(personalization);

        let work_type = WorkType::Coding {
            language: Some("rust".to_string()),
            framework: None,
            confidence: 0.9,
        };

        let focus_state = FocusState::Distracted {
            severity: 0.7,
            duration: chrono::Duration::minutes(10),
        };

        let intervention_type = InterventionType::CodingAssistance {
            language: Some("rust".to_string()),
            issue_category: CodingIssueCategory::DebuggingHelp,
        };

        let result = generator.generate_message(&work_type, &focus_state, &intervention_type);
        assert!(result.is_ok());

        let message = result.unwrap();
        assert!(!message.text.is_empty());
        assert!(!message.animation_hints.is_empty());
        assert!(message.confidence > 0.0);
    }

    #[test]
    fn test_personalization_application() {
        let mut personalization = MessagePersonalization::default();
        personalization.humor_level = 0.8;
        personalization.directness = 0.2;

        let generator = ContextualMessageGenerator::new(personalization);

        let original = "You should try debugging this code!".to_string();
        let personalized = generator.apply_personalization(original, &MessageTone::Playful);

        // Should be less direct
        assert!(!personalized.contains("You should"));
        assert!(personalized.contains("might want to") || personalized.contains("Consider"));
    }

    #[test]
    fn test_template_selection_with_feedback() {
        let personalization = MessagePersonalization::default();
        let mut generator = ContextualMessageGenerator::new(personalization);

        // Record positive feedback for a template
        generator.record_feedback(
            Uuid::new_v4(),
            "debug_gentle".to_string(),
            UserResponse::Helpful,
            Some(0.9),
            "debugging context".to_string(),
        );

        let effectiveness = generator.get_template_effectiveness_score("debug_gentle");
        assert!(effectiveness > 0.8);
    }

    #[test]
    fn test_focus_state_template_filtering() {
        let personalization = MessagePersonalization::default();
        let generator = ContextualMessageGenerator::new(personalization);

        let templates = vec![
            MessageTemplate {
                id: "gentle_template".to_string(),
                category: InterventionType::Encouragement { context: "test".to_string() },
                tone: MessageTone::Gentle,
                templates: vec!["Test message".to_string()],
                placeholders: vec![],
                min_confidence_threshold: 0.5,
            },
            MessageTemplate {
                id: "urgent_template".to_string(),
                category: InterventionType::Encouragement { context: "test".to_string() },
                tone: MessageTone::Urgent,
                templates: vec!["Test message".to_string()],
                placeholders: vec![],
                min_confidence_threshold: 0.5,
            },
        ];

        let flow_state = FocusState::Flow { depth: 0.9, stability: 0.8 };
        let filtered = generator.filter_templates(&templates, &flow_state);

        // In deep flow, should prefer gentle messages
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].tone, MessageTone::Gentle);
    }
}