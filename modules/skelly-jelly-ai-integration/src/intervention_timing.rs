//! Intervention Timing Engine
//! 
//! Manages when and how often to deliver contextual interventions based on:
//! - Focus state detection (hyperfocus, flow, distracted, transitioning)
//! - Cooldown management (15min minimum between interventions)
//! - Activity transitions and break points
//! - User preferences and intervention effectiveness

use crate::context_detection::WorkType;
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// ADHD focus states that affect intervention timing
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FocusState {
    /// Deep focus state - NO INTERRUPTIONS
    Hyperfocus {
        intensity: f32,  // 0.0-1.0
        duration: Duration,
    },
    /// Flow state - gentle interventions only
    Flow {
        depth: f32,      // 0.0-1.0
        stability: f32,  // how stable the flow is
    },
    /// Focused but not in flow - normal interventions OK
    Focused {
        concentration: f32,  // 0.0-1.0
    },
    /// Transitioning between tasks - good intervention point
    Transitioning {
        from_state: Option<Box<FocusState>>,
        confidence: f32,
    },
    /// Distracted state - interventions encouraged
    Distracted {
        severity: f32,   // 0.0-1.0
        duration: Duration,
    },
    /// Break time - gentle encouragement OK
    Break {
        planned: bool,   // Was this a planned break?
    },
    /// Unknown/uncertain state
    Unknown,
}

/// Intervention urgency levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum InterventionUrgency {
    Critical,   // Immediate attention needed
    High,       // Should intervene soon
    Normal,     // Standard intervention timing
    Low,        // Can wait for better timing
    Deferred,   // Skip for now
}

/// Types of interventions that can be delivered
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InterventionType {
    /// Coding-specific help (debugging tips, syntax help)
    CodingAssistance {
        language: Option<String>,
        issue_category: CodingIssueCategory,
    },
    /// Writing assistance (structure, clarity, flow)
    WritingSupport {
        document_type: String,
        issue_category: WritingIssueCategory,
    },
    /// Design feedback and suggestions
    DesignGuidance {
        design_type: String,
        issue_category: DesignIssueCategory,
    },
    /// General productivity and focus help
    FocusSupport {
        strategy: FocusStrategy,
    },
    /// Break and wellness reminders
    WellnessReminder {
        reminder_type: WellnessType,
    },
    /// Encouragement and motivation
    Encouragement {
        context: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CodingIssueCategory {
    DebuggingHelp,
    SyntaxError,
    ArchitectureAdvice,
    PerformanceOptimization,
    TestingGuidance,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WritingIssueCategory {
    StructureHelp,
    ClarityImprovement,
    GrammarCheck,
    FlowOptimization,
    IdeaGeneration,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DesignIssueCategory {
    LayoutSuggestion,
    ColorAdvice,
    TypographyHelp,
    AccessibilityCheck,
    UserExperience,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FocusStrategy {
    PomodoroSuggestion,
    BreakReminder,
    TaskPrioritization,
    DistractionElimination,
    EnergyManagement,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WellnessType {
    Hydration,
    PostureCheck,
    EyeBreak,
    MovementBreak,
    DeepBreathing,
}

/// Intervention timing decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterventionDecision {
    pub should_intervene: bool,
    pub urgency: InterventionUrgency,
    pub intervention_type: Option<InterventionType>,
    pub delay_seconds: u64,
    pub reason: String,
    pub confidence: f32,
}

/// History of past interventions for cooldown management
#[derive(Debug, Clone, Serialize, Deserialize)]
struct InterventionHistory {
    intervention_id: Uuid,
    timestamp: DateTime<Utc>,
    intervention_type: InterventionType,
    user_response: Option<UserResponse>,
    effectiveness_score: Option<f32>,  // 0.0-1.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserResponse {
    Helpful,
    NotHelpful,
    Dismissed,
    Ignored,
    ActionTaken,
}

/// Main intervention timing engine
pub struct InterventionTimingEngine {
    intervention_history: Vec<InterventionHistory>,
    last_intervention: Option<DateTime<Utc>>,
    user_preferences: InterventionPreferences,
    state_history: Vec<(FocusState, DateTime<Utc>)>,
    cooldown_overrides: HashMap<InterventionType, Duration>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterventionPreferences {
    pub min_cooldown_minutes: u64,  // Default: 15 minutes
    pub max_interventions_per_hour: u32,  // Default: 3
    pub respect_hyperfocus: bool,   // Default: true
    pub allow_break_reminders: bool, // Default: true
    pub preferred_intervention_types: Vec<InterventionType>,
    pub blocked_time_windows: Vec<(u32, u32)>, // (start_hour, end_hour) in 24h format  
}

impl Default for InterventionPreferences {
    fn default() -> Self {
        Self {
            min_cooldown_minutes: 15,
            max_interventions_per_hour: 3,
            respect_hyperfocus: true,
            allow_break_reminders: true,
            preferred_intervention_types: vec![],
            blocked_time_windows: vec![], // Empty = no blocked times
        }
    }
}

impl InterventionTimingEngine {
    pub fn new(preferences: InterventionPreferences) -> Self {
        Self {
            intervention_history: Vec::new(),
            last_intervention: None,
            user_preferences: preferences,
            state_history: Vec::new(),
            cooldown_overrides: HashMap::new(),
        }
    }

    /// Main decision function: should we intervene now?
    pub fn should_intervene(
        &mut self,
        current_state: FocusState,
        work_type: &WorkType,
        potential_intervention: InterventionType,
    ) -> InterventionDecision {
        let now = Utc::now();
        
        // Update state history
        self.state_history.push((current_state.clone(), now));
        self.cleanup_old_history();

        // Check absolute no-intervention conditions
        if let Some(reason) = self.check_blocking_conditions(&current_state, &now) {
            return InterventionDecision {
                should_intervene: false,
                urgency: InterventionUrgency::Deferred,
                intervention_type: None,
                delay_seconds: 300, // Try again in 5 minutes
                reason,
                confidence: 0.9,
            };
        }

        // Check cooldown period
        if let Some(reason) = self.check_cooldown(&now, &potential_intervention) {
            return InterventionDecision {
                should_intervene: false,
                urgency: InterventionUrgency::Deferred,
                intervention_type: None,
                delay_seconds: self.get_remaining_cooldown(&now, &potential_intervention),
                reason,
                confidence: 0.8,
            };
        }

        // Determine urgency based on state and context
        let urgency = self.calculate_urgency(&current_state, work_type, &potential_intervention);

        // Calculate intervention timing and confidence
        let (should_intervene, delay, confidence) = self.calculate_intervention_timing(
            &current_state,
            &urgency,
            &potential_intervention,
        );

        InterventionDecision {
            should_intervene,
            urgency,
            intervention_type: if should_intervene { Some(potential_intervention) } else { None },
            delay_seconds: delay,
            reason: self.get_decision_reason(&current_state, should_intervene),
            confidence,
        }
    }

    /// Record that an intervention was delivered
    pub fn record_intervention(
        &mut self,
        intervention_type: InterventionType,
        user_response: Option<UserResponse>,
    ) {
        let intervention = InterventionHistory {
            intervention_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            intervention_type,
            user_response,
            effectiveness_score: None, // Will be calculated later
        };

        self.intervention_history.push(intervention);
        self.last_intervention = Some(Utc::now());

        // Keep only last 100 interventions
        if self.intervention_history.len() > 100 {
            self.intervention_history.remove(0);
        }
    }

    /// Update effectiveness score for a previous intervention
    pub fn update_effectiveness(&mut self, intervention_id: Uuid, score: f32) {
        if let Some(intervention) = self.intervention_history
            .iter_mut()
            .find(|h| h.intervention_id == intervention_id) {
            intervention.effectiveness_score = Some(score.clamp(0.0, 1.0));
        }
    }

    /// Check absolute blocking conditions (hyperfocus, blocked times, etc.)
    fn check_blocking_conditions(&self, state: &FocusState, now: &DateTime<Utc>) -> Option<String> {
        // Respect hyperfocus state
        if self.user_preferences.respect_hyperfocus {
            if let FocusState::Hyperfocus { intensity, .. } = state {
                if *intensity > 0.7 {
                    return Some("User in hyperfocus state - no interruptions".to_string());
                }
            }
        }

        // Check blocked time windows
        let current_hour = now.hour();
        for (start_hour, end_hour) in &self.user_preferences.blocked_time_windows {
            if current_hour >= *start_hour && current_hour < *end_hour {
                return Some(format!("Blocked time window: {}:00-{}:00", start_hour, end_hour));
            }
        }

        // Check rate limiting
        let recent_interventions = self.count_recent_interventions(Duration::hours(1));
        if recent_interventions >= self.user_preferences.max_interventions_per_hour {
            return Some("Maximum interventions per hour reached".to_string());
        }

        None
    }

    /// Check if we're still in cooldown period
    fn check_cooldown(&self, now: &DateTime<Utc>, intervention_type: &InterventionType) -> Option<String> {
        if let Some(last_time) = self.last_intervention {
            let cooldown_duration = self.cooldown_overrides
                .get(intervention_type)
                .copied()
                .unwrap_or(Duration::minutes(self.user_preferences.min_cooldown_minutes as i64));

            let time_since_last = *now - last_time;
            if time_since_last < cooldown_duration {
                let remaining = cooldown_duration - time_since_last;
                return Some(format!("Cooldown period: {} minutes remaining", remaining.num_minutes()));
            }
        }
        None
    }

    /// Calculate intervention urgency based on current context
    fn calculate_urgency(
        &self,
        state: &FocusState,
        _work_type: &WorkType,
        intervention_type: &InterventionType,
    ) -> InterventionUrgency {
        match state {
            FocusState::Distracted { severity, duration } => {
                if *severity > 0.8 || duration.num_minutes() > 30 {
                    InterventionUrgency::High
                } else if *severity > 0.5 {
                    InterventionUrgency::Normal
                } else {
                    InterventionUrgency::Low
                }
            },
            FocusState::Transitioning { .. } => {
                // Good time for interventions
                InterventionUrgency::Normal
            },
            FocusState::Break { .. } => {
                match intervention_type {
                    InterventionType::WellnessReminder { .. } => InterventionUrgency::Normal,
                    InterventionType::Encouragement { .. } => InterventionUrgency::Low,
                    _ => InterventionUrgency::Deferred,
                }
            },
            FocusState::Flow { depth, .. } => {
                if *depth > 0.8 {
                    InterventionUrgency::Deferred  // Don't interrupt deep flow
                } else {
                    InterventionUrgency::Low
                }
            },
            FocusState::Focused { concentration } => {
                if *concentration > 0.7 {
                    InterventionUrgency::Low
                } else {
                    InterventionUrgency::Normal
                }
            },
            _ => InterventionUrgency::Normal,
        }
    }

    /// Calculate specific timing and confidence for intervention
    fn calculate_intervention_timing(
        &self,
        state: &FocusState,
        urgency: &InterventionUrgency,
        _intervention_type: &InterventionType,
    ) -> (bool, u64, f32) {
        match urgency {
            InterventionUrgency::Critical => (true, 0, 0.95),
            InterventionUrgency::High => {
                match state {
                    FocusState::Distracted { .. } => (true, 30, 0.85), // Wait 30 seconds
                    _ => (true, 60, 0.75), // Wait 1 minute
                }
            },
            InterventionUrgency::Normal => {
                match state {
                    FocusState::Transitioning { .. } => (true, 0, 0.8), // Good timing
                    FocusState::Focused { concentration } if *concentration < 0.5 => (true, 120, 0.7),
                    _ => (true, 300, 0.6), // Wait 5 minutes
                }
            },
            InterventionUrgency::Low => {
                match state {
                    FocusState::Break { .. } => (true, 0, 0.5),
                    _ => (false, 600, 0.3), // Wait 10 minutes, low priority
                }
            },
            InterventionUrgency::Deferred => (false, 900, 0.1), // Wait 15 minutes
        }
    }

    /// Get remaining cooldown time in seconds
    fn get_remaining_cooldown(&self, now: &DateTime<Utc>, intervention_type: &InterventionType) -> u64 {
        if let Some(last_time) = self.last_intervention {
            let cooldown_duration = self.cooldown_overrides
                .get(intervention_type)
                .copied()
                .unwrap_or(Duration::minutes(self.user_preferences.min_cooldown_minutes as i64));

            let time_since_last = *now - last_time;
            let remaining = cooldown_duration - time_since_last;
            remaining.num_seconds().max(0) as u64
        } else {
            0
        }
    }

    /// Count interventions in recent time period
    fn count_recent_interventions(&self, period: Duration) -> u32 {
        let cutoff = Utc::now() - period;
        self.intervention_history
            .iter()
            .filter(|h| h.timestamp > cutoff)
            .count() as u32
    }

    /// Clean up old state history (keep last 24 hours)
    fn cleanup_old_history(&mut self) {
        let cutoff = Utc::now() - Duration::hours(24);
        self.state_history.retain(|(_, timestamp)| *timestamp > cutoff);
    }

    /// Get human-readable decision reason
    fn get_decision_reason(&self, state: &FocusState, should_intervene: bool) -> String {
        if should_intervene {
            match state {
                FocusState::Distracted { severity, .. } => {
                    format!("User distracted (severity: {:.1}) - intervention helpful", severity)
                },
                FocusState::Transitioning { .. } => {
                    "Good timing during task transition".to_string()
                },
                FocusState::Break { .. } => {
                    "Break time - gentle intervention appropriate".to_string()
                },
                _ => "Normal intervention timing".to_string(),
            }
        } else {
            match state {
                FocusState::Hyperfocus { .. } => "Respecting hyperfocus state".to_string(),
                FocusState::Flow { .. } => "Preserving flow state".to_string(),
                _ => "Waiting for better timing".to_string(),
            }
        }
    }

    /// Get intervention effectiveness statistics
    pub fn get_effectiveness_stats(&self) -> InterventionStats {
        let total_interventions = self.intervention_history.len();
        let with_scores: Vec<f32> = self.intervention_history
            .iter()
            .filter_map(|h| h.effectiveness_score)
            .collect();

        let average_effectiveness = if !with_scores.is_empty() {
            with_scores.iter().sum::<f32>() / with_scores.len() as f32
        } else {
            0.0
        };

        let helpful_responses = self.intervention_history
            .iter()
            .filter(|h| matches!(h.user_response, Some(UserResponse::Helpful) | Some(UserResponse::ActionTaken)))
            .count();

        let dismissal_rate = if total_interventions > 0 {
            let dismissed = self.intervention_history
                .iter()
                .filter(|h| matches!(h.user_response, Some(UserResponse::Dismissed) | Some(UserResponse::Ignored)))
                .count();
            dismissed as f32 / total_interventions as f32
        } else {
            0.0
        };

        InterventionStats {
            total_interventions,
            average_effectiveness,
            helpful_responses,
            dismissal_rate,
            interventions_last_hour: self.count_recent_interventions(Duration::hours(1)),
            interventions_last_day: self.count_recent_interventions(Duration::hours(24)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterventionStats {
    pub total_interventions: usize,
    pub average_effectiveness: f32,
    pub helpful_responses: usize,
    pub dismissal_rate: f32,
    pub interventions_last_hour: u32,
    pub interventions_last_day: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hyperfocus_blocking() {
        let preferences = InterventionPreferences::default();
        let mut engine = InterventionTimingEngine::new(preferences);
        
        let hyperfocus_state = FocusState::Hyperfocus {
            intensity: 0.9,
            duration: Duration::minutes(30),
        };
        
        let decision = engine.should_intervene(
            hyperfocus_state,
            &WorkType::Unknown { confidence: 0.5 },
            InterventionType::FocusSupport { strategy: FocusStrategy::BreakReminder },
        );
        
        assert!(!decision.should_intervene);
        assert!(decision.reason.contains("hyperfocus"));
    }

    #[test]
    fn test_cooldown_management() {
        let preferences = InterventionPreferences {
            min_cooldown_minutes: 10,
            ..Default::default()
        };
        let mut engine = InterventionTimingEngine::new(preferences);
        
        // Record an intervention
        engine.record_intervention(
            InterventionType::FocusSupport { strategy: FocusStrategy::BreakReminder },
            Some(UserResponse::Helpful),
        );
        
        // Try to intervene again immediately
        let decision = engine.should_intervene(
            FocusState::Distracted { severity: 0.8, duration: Duration::minutes(5) },
            &WorkType::Unknown { confidence: 0.5 },
            InterventionType::FocusSupport { strategy: FocusStrategy::BreakReminder },
        );
        
        assert!(!decision.should_intervene);
        assert!(decision.reason.contains("Cooldown"));
    }

    #[test]
    fn test_transition_timing() {
        let preferences = InterventionPreferences::default();
        let mut engine = InterventionTimingEngine::new(preferences);
        
        let transition_state = FocusState::Transitioning {
            from_state: None,
            confidence: 0.8,
        };
        
        let decision = engine.should_intervene(
            transition_state,
            &WorkType::Unknown { confidence: 0.5 },
            InterventionType::FocusSupport { strategy: FocusStrategy::TaskPrioritization },
        );
        
        assert!(decision.should_intervene);
        assert_eq!(decision.delay_seconds, 0); // Should intervene immediately during transitions
    }

    #[test]
    fn test_distraction_urgency() {
        let preferences = InterventionPreferences::default();
        let mut engine = InterventionTimingEngine::new(preferences);
        
        let distracted_state = FocusState::Distracted {
            severity: 0.9,
            duration: Duration::minutes(35),
        };
        
        let decision = engine.should_intervene(
            distracted_state,
            &WorkType::Unknown { confidence: 0.5 },
            InterventionType::FocusSupport { strategy: FocusStrategy::DistractionElimination },
        );
        
        assert!(decision.should_intervene);
        assert_eq!(decision.urgency, InterventionUrgency::High);
    }
}