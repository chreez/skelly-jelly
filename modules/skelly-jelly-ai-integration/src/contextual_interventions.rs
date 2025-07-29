//! Contextual Interventions Integration
//! 
//! Orchestrates the complete Story 3.1 implementation by integrating:
//! - Work-type detection (coding, writing, designing)
//! - Intervention timing engine with flow state respect
//! - Context-aware messaging system  
//! - User feedback collection and learning

use crate::context_detection::{WorkTypeDetector, WorkType, WorkContext};
use crate::intervention_timing::{
    InterventionTimingEngine, FocusState, InterventionType, InterventionDecision,
    InterventionPreferences, UserResponse, CodingIssueCategory, WritingIssueCategory,
    DesignIssueCategory, FocusStrategy, WellnessType
};
use crate::contextual_messaging::{
    ContextualMessageGenerator, ContextualMessage, MessagePersonalization
};
use crate::user_feedback::{FeedbackCollector, FeedbackSubmission, FeedbackType, FeedbackContext};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Timelike, Datelike};
use uuid::Uuid;
use std::collections::HashMap;

/// Complete contextual intervention system
pub struct ContextualInterventionSystem {
    work_detector: WorkTypeDetector,
    timing_engine: InterventionTimingEngine,
    message_generator: ContextualMessageGenerator,
    feedback_collector: FeedbackCollector,
    current_work_context: Option<WorkContext>,
    intervention_history: Vec<InterventionRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct InterventionRecord {
    intervention_id: Uuid,
    timestamp: DateTime<Utc>,
    work_type: WorkType,
    focus_state: FocusState,
    intervention_type: InterventionType,
    message: ContextualMessage,
    user_response: Option<UserResponse>,
}

/// Configuration for the contextual intervention system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextualInterventionConfig {
    pub intervention_preferences: InterventionPreferences,
    pub message_personalization: MessagePersonalization,
    pub enable_work_detection: bool,
    pub enable_timing_engine: bool,
    pub enable_feedback_collection: bool,
}

impl Default for ContextualInterventionConfig {
    fn default() -> Self {
        Self {
            intervention_preferences: InterventionPreferences::default(),
            message_personalization: MessagePersonalization::default(),
            enable_work_detection: true,
            enable_timing_engine: true,
            enable_feedback_collection: true,
        }
    }
}

/// Input for generating a contextual intervention
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterventionContext {
    pub application_name: String,
    pub window_title: String,
    pub recent_text: Option<String>,
    pub current_focus_state: FocusState,
    pub session_duration_minutes: u32,
    pub interventions_today: u32,
}

/// Complete intervention response with all metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextualInterventionResponse {
    pub intervention_id: Uuid,
    pub should_show: bool,
    pub message: Option<ContextualMessage>,
    pub delay_seconds: u64,
    pub confidence: f32,
    pub work_context: WorkContext,
    pub intervention_type: Option<InterventionType>,
    pub reason: String,
    pub created_at: DateTime<Utc>,
}

impl ContextualInterventionSystem {
    pub fn new(config: ContextualInterventionConfig) -> Self {
        Self {
            work_detector: WorkTypeDetector::new(),
            timing_engine: InterventionTimingEngine::new(config.intervention_preferences),
            message_generator: ContextualMessageGenerator::new(config.message_personalization),
            feedback_collector: FeedbackCollector::new(),
            current_work_context: None,
            intervention_history: Vec::new(),
        }
    }

    /// Process a request for contextual intervention
    pub fn process_intervention_request(
        &mut self,
        context: InterventionContext,
    ) -> Result<ContextualInterventionResponse, String> {
        let intervention_id = Uuid::new_v4();
        let created_at = Utc::now();

        // Step 1: Detect work type and context
        let work_context = if self.message_generator.is_enabled() {
            self.work_detector.detect_work_type(
                &context.application_name,
                &context.window_title,
                context.recent_text.as_deref(),
            )
        } else {
            // Fallback to simple context
            WorkContext {
                work_type: WorkType::Unknown { confidence: 0.1 },
                application: context.application_name.clone(),
                window_title: context.window_title.clone(),
                detected_patterns: vec![],
                activity_duration: 0,
                last_updated: created_at,
            }
        };

        // Update current work context
        self.current_work_context = Some(work_context.clone());

        // Step 2: Determine intervention type based on work context and focus state
        let potential_intervention = self.determine_intervention_type(
            &work_context.work_type,
            &context.current_focus_state,
        );

        // Step 3: Check timing and decide whether to intervene
        let timing_decision = if self.timing_engine.is_enabled() {
            self.timing_engine.should_intervene(
                context.current_focus_state.clone(),
                &work_context.work_type,
                potential_intervention.clone(),
            )
        } else {
            // Simple fallback - always allow interventions with 5 minute delay
            InterventionDecision {
                should_intervene: true,
                urgency: crate::intervention_timing::InterventionUrgency::Normal,
                intervention_type: Some(potential_intervention.clone()),
                delay_seconds: 300,
                reason: "Timing engine disabled - default behavior".to_string(),
                confidence: 0.5,
            }
        };

        // Step 4: Generate contextual message if intervention is approved
        let message = if timing_decision.should_intervene {
            if let Some(ref intervention_type) = timing_decision.intervention_type {
                match self.message_generator.generate_message(
                    &work_context.work_type,
                    &context.current_focus_state,
                    intervention_type,
                ) {
                    Ok(msg) => Some(msg),
                    Err(err) => {
                        log::warn!("Failed to generate contextual message: {}", err);
                        None
                    }
                }
            } else {
                None
            }
        } else {
            None
        };

        // Step 5: Record intervention attempt
        if let Some(ref msg) = message {
            let record = InterventionRecord {
                intervention_id,
                timestamp: created_at,
                work_type: work_context.work_type.clone(),
                focus_state: context.current_focus_state.clone(),
                intervention_type: potential_intervention.clone(),
                message: msg.clone(),
                user_response: None,
            };
            self.intervention_history.push(record);

            // Keep only last 100 interventions
            if self.intervention_history.len() > 100 {
                self.intervention_history.remove(0);
            }
        }

        Ok(ContextualInterventionResponse {
            intervention_id,
            should_show: timing_decision.should_intervene && message.is_some(),
            message,
            delay_seconds: timing_decision.delay_seconds,
            confidence: timing_decision.confidence,
            work_context,
            intervention_type: timing_decision.intervention_type,
            reason: timing_decision.reason,
            created_at,
        })
    }

    /// Record user feedback for an intervention
    pub fn record_feedback(
        &mut self,
        intervention_id: Uuid,
        feedback_type: FeedbackType,
        response_time_ms: u64,
    ) -> Result<(), String> {
        // Find the intervention record
        let intervention_record = self.intervention_history
            .iter_mut()
            .find(|record| record.intervention_id == intervention_id)
            .ok_or_else(|| "Intervention not found".to_string())?;

        // Extract user response for timing engine
        let user_response = match &feedback_type {
            FeedbackType::Helpful { helpful, .. } => {
                if *helpful { UserResponse::Helpful } else { UserResponse::NotHelpful }
            },
            FeedbackType::Rating { score, .. } => {
                if *score >= 4 { UserResponse::Helpful } else { UserResponse::NotHelpful }
            },
            FeedbackType::ActionTaken { .. } => UserResponse::ActionTaken,
            FeedbackType::Dismissed { .. } => UserResponse::Dismissed,
            FeedbackType::Detailed { helpfulness, .. } => {
                if *helpfulness { UserResponse::Helpful } else { UserResponse::NotHelpful }
            },
        };

        // Update intervention record
        intervention_record.user_response = Some(user_response.clone());

        // Record in timing engine
        self.timing_engine.record_intervention(
            intervention_record.intervention_type.clone(),
            Some(user_response.clone()),
        );

        // Record in feedback collector
        if self.feedback_collector.is_enabled() {
            // Extract values before creating FeedbackContext to avoid borrowing issues
            let work_type_str = format!("{:?}", intervention_record.work_type);
            let focus_state_str = format!("{:?}", intervention_record.focus_state);
            let intervention_type_str = format!("{:?}", intervention_record.intervention_type);
            let timestamp_hour = intervention_record.timestamp.hour() as u8;
            let timestamp_day = intervention_record.timestamp.weekday().num_days_from_sunday() as u8;
            
            let intervention_count_today = self.count_interventions_today();
            let feedback_context = FeedbackContext {
                work_type: work_type_str,
                focus_state: focus_state_str,
                intervention_type: intervention_type_str,
                time_of_day: timestamp_hour,
                day_of_week: timestamp_day,
                intervention_count_today,
                user_session_duration_mins: 60, // This would be tracked separately
            };

            let feedback_submission = FeedbackSubmission {
                submission_id: Uuid::new_v4(),
                intervention_id,
                user_id: None, // This would come from user management system
                feedback_type,
                context: feedback_context,
                submitted_at: Utc::now(),
                response_time_ms,
            };

            self.feedback_collector.submit_feedback(feedback_submission)?;

            // Record message feedback for personalization
            let message_id_str = intervention_record.message.message_id.to_string();
            let work_type_context = format!("{:?}", intervention_record.work_type);
            self.message_generator.record_feedback(
                intervention_id,
                message_id_str,
                user_response,
                None, // Effectiveness score would be calculated separately
                work_type_context,
            );
        }

        Ok(())
    }

    /// Get system statistics and analytics
    pub fn get_analytics(&mut self) -> ContextualInterventionAnalytics {
        let total_interventions = self.intervention_history.len();
        let successful_interventions = self.intervention_history
            .iter()
            .filter(|record| matches!(
                record.user_response,
                Some(UserResponse::Helpful) | Some(UserResponse::ActionTaken)
            ))
            .count();

        let work_type_distribution = self.calculate_work_type_distribution();
        let timing_stats = self.timing_engine.get_effectiveness_stats();
        let feedback_analytics = if self.feedback_collector.is_enabled() {
            Some(self.feedback_collector.get_analytics())
        } else {
            None
        };
        let message_stats = self.message_generator.get_effectiveness_stats();

        ContextualInterventionAnalytics {
            total_interventions,
            successful_interventions,
            success_rate: if total_interventions > 0 {
                successful_interventions as f32 / total_interventions as f32
            } else {
                0.0
            },
            work_type_distribution,
            timing_stats,
            feedback_analytics,
            message_effectiveness: message_stats,
            last_updated: Utc::now(),
        }
    }

    /// Update system configuration
    pub fn update_config(&mut self, config: ContextualInterventionConfig) {
        // Update message personalization
        self.message_generator.update_personalization(config.message_personalization);
        
        // Note: Timing engine preferences would need to be updated via a new method
        // that we'd add to InterventionTimingEngine
    }

    /// Get current work context
    pub fn get_current_work_context(&self) -> Option<&WorkContext> {
        self.current_work_context.as_ref()
    }

    /// Get recent intervention history
    pub fn get_recent_interventions(&self, limit: usize) -> Vec<&InterventionRecord> {
        self.intervention_history
            .iter()
            .rev()
            .take(limit)
            .collect()
    }

    /// Determine appropriate intervention type based on context
    fn determine_intervention_type(
        &self,
        work_type: &WorkType,
        focus_state: &FocusState,
    ) -> InterventionType {
        match work_type {
            WorkType::Coding { language, .. } => {
                match focus_state {
                    FocusState::Distracted { severity, .. } if *severity > 0.7 => {
                        InterventionType::CodingAssistance {
                            language: language.clone(),
                            issue_category: CodingIssueCategory::DebuggingHelp,
                        }
                    },
                    FocusState::Transitioning { .. } => {
                        InterventionType::FocusSupport {
                            strategy: FocusStrategy::TaskPrioritization,
                        }
                    },
                    _ => {
                        InterventionType::CodingAssistance {
                            language: language.clone(),
                            issue_category: CodingIssueCategory::DebuggingHelp,
                        }
                    }
                }
            },
            WorkType::Writing { document_type, .. } => {
                InterventionType::WritingSupport {
                    document_type: format!("{:?}", document_type),
                    issue_category: WritingIssueCategory::StructureHelp,
                }
            },
            WorkType::Designing { design_type, .. } => {
                InterventionType::DesignGuidance {
                    design_type: format!("{:?}", design_type),
                    issue_category: DesignIssueCategory::LayoutSuggestion,
                }
            },
            WorkType::Communication { .. } => {
                InterventionType::FocusSupport {
                    strategy: FocusStrategy::DistractionElimination,
                }
            },
            WorkType::Unknown { .. } => {
                match focus_state {
                    FocusState::Distracted { .. } => {
                        InterventionType::FocusSupport {
                            strategy: FocusStrategy::PomodoroSuggestion,
                        }
                    },
                    FocusState::Break { .. } => {
                        InterventionType::WellnessReminder {
                            reminder_type: WellnessType::Hydration,
                        }
                    },
                    _ => {
                        InterventionType::Encouragement {
                            context: "general".to_string(),
                        }
                    }
                }
            },
        }
    }

    fn calculate_work_type_distribution(&self) -> HashMap<String, f32> {
        if self.intervention_history.is_empty() {
            return HashMap::new();
        }

        let mut counts: HashMap<String, u32> = HashMap::new();
        for record in &self.intervention_history {
            let work_type_name = match &record.work_type {
                WorkType::Coding { .. } => "Coding".to_string(),
                WorkType::Writing { .. } => "Writing".to_string(),
                WorkType::Designing { .. } => "Designing".to_string(),
                WorkType::Communication { .. } => "Communication".to_string(),
                WorkType::Unknown { .. } => "Unknown".to_string(),
            };
            *counts.entry(work_type_name).or_insert(0) += 1;
        }

        let total = self.intervention_history.len() as f32;
        counts.into_iter()
            .map(|(work_type, count)| (work_type, count as f32 / total))
            .collect()
    }

    fn count_interventions_today(&self) -> u32 {
        let today = Utc::now().date_naive();
        self.intervention_history
            .iter()
            .filter(|record| record.timestamp.date_naive() == today)
            .count() as u32
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextualInterventionAnalytics {
    pub total_interventions: usize,
    pub successful_interventions: usize,
    pub success_rate: f32,
    pub work_type_distribution: HashMap<String, f32>,
    pub timing_stats: crate::intervention_timing::InterventionStats,
    pub feedback_analytics: Option<crate::user_feedback::FeedbackAnalytics>,
    pub message_effectiveness: crate::contextual_messaging::MessageEffectivenessStats,
    pub last_updated: DateTime<Utc>,
}

// Helper trait extensions for enabled checks
trait ComponentEnabled {
    fn is_enabled(&self) -> bool;
}

impl ComponentEnabled for ContextualMessageGenerator {
    fn is_enabled(&self) -> bool {
        true // Always enabled for now
    }
}

impl ComponentEnabled for InterventionTimingEngine {
    fn is_enabled(&self) -> bool {
        true // Always enabled for now
    }
}

impl ComponentEnabled for FeedbackCollector {
    fn is_enabled(&self) -> bool {
        true // Always enabled for now
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::intervention_timing::FocusState;
    use chrono::Duration;

    #[test]
    fn test_contextual_intervention_system() {
        let config = ContextualInterventionConfig::default();
        let mut system = ContextualInterventionSystem::new(config);

        let context = InterventionContext {
            application_name: "Visual Studio Code".to_string(),
            window_title: "main.rs - skelly-jelly".to_string(),
            recent_text: Some("fn main() {\n    // TODO: implement\n}".to_string()),
            current_focus_state: FocusState::Distracted {
                severity: 0.8,
                duration: Duration::minutes(10),
            },
            session_duration_minutes: 45,
            interventions_today: 2,
        };

        let response = system.process_intervention_request(context).unwrap();
        
        assert!(response.confidence > 0.0);
        assert!(matches!(response.work_context.work_type, WorkType::Coding { .. }));
        
        if response.should_show {
            assert!(response.message.is_some());
        }
    }

    #[test]
    fn test_feedback_recording() {
        let config = ContextualInterventionConfig::default();
        let mut system = ContextualInterventionSystem::new(config);

        // Create an intervention first
        let context = InterventionContext {
            application_name: "Notion".to_string(),
            window_title: "Documentation".to_string(),
            recent_text: None,
            current_focus_state: FocusState::Focused { concentration: 0.7 },
            session_duration_minutes: 30,
            interventions_today: 1,
        };

        let response = system.process_intervention_request(context).unwrap();
        
        // Record feedback
        let feedback = FeedbackType::Helpful {
            helpful: true,
            reason: Some("Very helpful suggestion!".to_string()),
        };

        let result = system.record_feedback(
            response.intervention_id,
            feedback,
            3000, // 3 second response time
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_work_type_determination() {
        let config = ContextualInterventionConfig::default();
        let system = ContextualInterventionSystem::new(config);

        // Test coding context
        let coding_work_type = WorkType::Coding {
            language: Some("rust".to_string()),
            framework: None,
            confidence: 0.9,
        };
        let focus_state = FocusState::Distracted {
            severity: 0.8,
            duration: Duration::minutes(5),
        };

        let intervention_type = system.determine_intervention_type(&coding_work_type, &focus_state);
        
        assert!(matches!(
            intervention_type,
            InterventionType::CodingAssistance { .. }
        ));
    }

    #[test]
    fn test_analytics_computation() {
        let config = ContextualInterventionConfig::default();
        let mut system = ContextualInterventionSystem::new(config);

        // Add some test interventions
        for i in 0..5 {
            let context = InterventionContext {
                application_name: "Test App".to_string(),
                window_title: format!("Test Window {}", i),
                recent_text: None,
                current_focus_state: FocusState::Focused { concentration: 0.6 },
                session_duration_minutes: 20,
                interventions_today: i,
            };

            let response = system.process_intervention_request(context).unwrap();
            
            // Record positive feedback for half of them
            if i % 2 == 0 {
                let _ = system.record_feedback(
                    response.intervention_id,
                    FeedbackType::Helpful { helpful: true, reason: None },
                    2000,
                );
            }
        }

        let analytics = system.get_analytics();
        assert_eq!(analytics.total_interventions, 5);
        assert!(analytics.success_rate > 0.0);
    }
}