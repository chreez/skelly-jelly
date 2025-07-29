//! User Feedback Collection System
//! 
//! Collects and analyzes user feedback on intervention effectiveness to improve:
//! - Message relevance and timing
//! - Personalization preferences
//! - Intervention type effectiveness
//! - User satisfaction and engagement

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;
use std::collections::HashMap;

/// Types of feedback users can provide
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FeedbackType {
    /// Quick 1-5 rating
    Rating {
        score: u8,  // 1-5 scale
        category: FeedbackCategory,
    },
    /// Binary helpful/not helpful
    Helpful {
        helpful: bool,
        reason: Option<String>,
    },
    /// Detailed feedback with text
    Detailed {
        rating: u8,
        helpfulness: bool,
        timing_appropriate: bool,
        message_clear: bool,
        would_see_again: bool,
        suggestions: Option<String>,
    },
    /// User action taken based on intervention
    ActionTaken {
        action: String,
        effectiveness: u8, // 1-5 how effective the action was
    },
    /// User dismissed or ignored the intervention
    Dismissed {
        reason: DismissalReason,
        too_frequent: bool,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FeedbackCategory {
    MessageContent,     // Was the message helpful/relevant?
    Timing,            // Was the timing appropriate?
    Frequency,         // Too many/few interventions?
    PersonalityFit,    // Did the tone/style match preferences?
    TechnicalAccuracy, // Was the advice technically sound?
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DismissalReason {
    NotRelevant,       // Message didn't apply to current situation
    BadTiming,         // Interrupted important work
    TooFrequent,       // Too many interventions recently
    NotHelpful,        // Content wasn't useful
    AlreadyKnow,       // User already knows this information
    WrongTone,         // Tone/personality didn't fit
    Other(String),     // User-specified reason
}

/// Feedback submission from the user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackSubmission {
    pub submission_id: Uuid,
    pub intervention_id: Uuid,
    pub user_id: Option<String>,  // Optional user identification
    pub feedback_type: FeedbackType,
    pub context: FeedbackContext,
    pub submitted_at: DateTime<Utc>,
    pub response_time_ms: u64,    // How long until user provided feedback
}

/// Context when feedback was provided
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackContext {
    pub work_type: String,
    pub focus_state: String,
    pub intervention_type: String,
    pub time_of_day: u8,          // Hour 0-23
    pub day_of_week: u8,          // 0=Sunday, 6=Saturday
    pub intervention_count_today: u32,
    pub user_session_duration_mins: u32,
}

/// Aggregated feedback analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackAnalytics {
    pub overall_satisfaction: f32,     // 0.0-1.0
    pub helpfulness_rate: f32,         // Percentage of interventions marked helpful
    pub dismissal_rate: f32,           // Percentage dismissed/ignored
    pub avg_response_time_ms: u64,     // How quickly users respond to interventions
    pub category_scores: HashMap<FeedbackCategory, f32>,
    pub temporal_patterns: TemporalPatterns,
    pub improvement_suggestions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalPatterns {
    pub best_times_of_day: Vec<u8>,    // Hours when interventions are most effective
    pub worst_times_of_day: Vec<u8>,   // Hours when interventions are least effective
    pub best_days_of_week: Vec<u8>,    // Days when interventions work best
    pub optimal_frequency_mins: u32,   // Optimal time between interventions
}

/// Personalization recommendations based on feedback patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalizationRecommendations {
    pub recommended_tone: Option<String>,
    pub optimal_frequency_mins: u32,
    pub preferred_intervention_types: Vec<String>,
    pub blocked_time_windows: Vec<(u8, u8)>, // (start_hour, end_hour)
    pub content_preferences: ContentPreferences,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentPreferences {
    pub technical_level: f32,      // 0.0=simple, 1.0=technical
    pub humor_level: f32,          // 0.0=none, 1.0=lots of skeleton humor
    pub encouragement_level: f32,  // 0.0=minimal, 1.0=very encouraging
    pub directness: f32,           // 0.0=gentle, 1.0=direct
}

/// Main feedback collection and analysis system
pub struct FeedbackCollector {
    feedback_history: Vec<FeedbackSubmission>,
    analytics_cache: Option<(FeedbackAnalytics, DateTime<Utc>)>, // Cache with timestamp
    intervention_metrics: HashMap<String, InterventionMetrics>,   // Per intervention type
    user_preferences: HashMap<String, PersonalizationRecommendations>, // Per user
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct InterventionMetrics {
    total_shown: u32,
    total_feedback: u32,
    avg_rating: f32,
    helpfulness_rate: f32,
    dismissal_rate: f32,
    avg_response_time_ms: u64,
    improvement_trend: f32, // -1.0 to 1.0, negative = getting worse
}

impl FeedbackCollector {
    pub fn new() -> Self {
        Self {
            feedback_history: Vec::new(),
            analytics_cache: None,
            intervention_metrics: HashMap::new(),
            user_preferences: HashMap::new(),
        }
    }

    /// Submit user feedback for an intervention
    pub fn submit_feedback(&mut self, feedback: FeedbackSubmission) -> Result<(), String> {
        // Validate feedback
        self.validate_feedback(&feedback)?;

        // Store feedback
        self.feedback_history.push(feedback.clone());

        // Update intervention metrics
        self.update_intervention_metrics(&feedback);

        // Update user preferences if this is detailed feedback
        if let Some(user_id) = &feedback.user_id {
            self.update_user_preferences(user_id, &feedback);
        }

        // Invalidate analytics cache
        self.analytics_cache = None;

        // Cleanup old feedback (keep last 1000 entries)
        if self.feedback_history.len() > 1000 {
            self.feedback_history.remove(0);
        }

        Ok(())
    }

    /// Get current feedback analytics (cached for performance)
    pub fn get_analytics(&mut self) -> FeedbackAnalytics {
        // Check cache (valid for 1 hour)
        if let Some((cached_analytics, cache_time)) = &self.analytics_cache {
            if Utc::now().signed_duration_since(*cache_time).num_hours() < 1 {
                return cached_analytics.clone();
            }
        }

        // Compute fresh analytics
        let analytics = self.compute_analytics();
        self.analytics_cache = Some((analytics.clone(), Utc::now()));
        analytics
    }

    /// Get personalization recommendations for a user
    pub fn get_personalization_recommendations(&self, user_id: &str) -> Option<PersonalizationRecommendations> {
        self.user_preferences.get(user_id).cloned()
    }

    /// Get feedback statistics for a specific intervention type
    pub fn get_intervention_metrics(&self, intervention_type: &str) -> Option<InterventionMetrics> {
        self.intervention_metrics.get(intervention_type).cloned()
    }

    /// Get recent feedback trends (last 7 days)
    pub fn get_recent_trends(&self) -> FeedbackTrends {
        let week_ago = Utc::now() - Duration::days(7);
        let recent_feedback: Vec<&FeedbackSubmission> = self.feedback_history
            .iter()
            .filter(|f| f.submitted_at > week_ago)
            .collect();

        if recent_feedback.is_empty() {
            return FeedbackTrends::default();
        }

        let total_count = recent_feedback.len();
        let helpful_count = recent_feedback.iter()
            .filter(|f| self.is_positive_feedback(&f.feedback_type))
            .count();

        let dismissed_count = recent_feedback.iter()
            .filter(|f| matches!(f.feedback_type, FeedbackType::Dismissed { .. }))
            .count();

        let avg_rating = self.calculate_average_rating(&recent_feedback);
        let trend_direction = self.calculate_trend_direction(&recent_feedback);

        FeedbackTrends {
            total_feedback_count: total_count,
            helpfulness_rate: helpful_count as f32 / total_count as f32,
            dismissal_rate: dismissed_count as f32 / total_count as f32,
            average_rating: avg_rating,
            trend_direction,
            most_common_dismissal_reason: self.get_most_common_dismissal_reason(&recent_feedback),
        }
    }

    /// Generate actionable improvement suggestions based on feedback patterns
    pub fn get_improvement_suggestions(&self) -> Vec<String> {
        let mut suggestions = Vec::new();
        let analytics = self.get_analytics();

        // Low overall satisfaction
        if analytics.overall_satisfaction < 0.6 {
            suggestions.push("Consider reducing intervention frequency - satisfaction is low".to_string());
        }

        // High dismissal rate
        if analytics.dismissal_rate > 0.4 {
            suggestions.push("Many interventions are being dismissed - review timing and relevance".to_string());
        }

        // Poor timing scores
        if let Some(timing_score) = analytics.category_scores.get(&FeedbackCategory::Timing) {
            if *timing_score < 0.5 {
                suggestions.push("Intervention timing needs improvement - avoid interrupting deep work".to_string());
            }
        }

        // Content relevance issues
        if let Some(content_score) = analytics.category_scores.get(&FeedbackCategory::MessageContent) {
            if *content_score < 0.6 {
                suggestions.push("Message content relevance could be improved".to_string());
            }
        }

        // Frequency issues
        if let Some(freq_score) = analytics.category_scores.get(&FeedbackCategory::Frequency) {
            if *freq_score < 0.5 {
                suggestions.push("Adjust intervention frequency based on user feedback".to_string());
            }
        }

        suggestions
    }

    fn validate_feedback(&self, feedback: &FeedbackSubmission) -> Result<(), String> {
        match &feedback.feedback_type {
            FeedbackType::Rating { score, .. } => {
                if *score < 1 || *score > 5 {
                    return Err("Rating must be between 1 and 5".to_string());
                }
            },
            FeedbackType::Detailed { rating, effectiveness, .. } => {
                if *rating < 1 || *rating > 5 {
                    return Err("Rating must be between 1 and 5".to_string());
                }
                if *effectiveness < 1 || *effectiveness > 5 {
                    return Err("Effectiveness must be between 1 and 5".to_string());
                }
            },
            FeedbackType::ActionTaken { effectiveness, .. } => {
                if *effectiveness < 1 || *effectiveness > 5 {
                    return Err("Effectiveness must be between 1 and 5".to_string());
                }
            },
            _ => {}, // Other types don't need validation
        }

        Ok(())
    }

    fn update_intervention_metrics(&mut self, feedback: &FeedbackSubmission) {
        let intervention_type = feedback.context.intervention_type.clone();
        let metrics = self.intervention_metrics
            .entry(intervention_type)
            .or_insert(InterventionMetrics {
                total_shown: 0,
                total_feedback: 0,
                avg_rating: 0.0,
                helpfulness_rate: 0.0,
                dismissal_rate: 0.0,
                avg_response_time_ms: 0,
                improvement_trend: 0.0,
            });

        metrics.total_feedback += 1;

        // Update response time
        let new_avg_response = (metrics.avg_response_time_ms * (metrics.total_feedback - 1) as u64 
            + feedback.response_time_ms) / metrics.total_feedback as u64;
        metrics.avg_response_time_ms = new_avg_response;

        // Update ratings and helpfulness
        match &feedback.feedback_type {
            FeedbackType::Rating { score, .. } => {
                let new_avg = (metrics.avg_rating * (metrics.total_feedback - 1) as f32 + *score as f32) 
                    / metrics.total_feedback as f32;
                metrics.avg_rating = new_avg;
            },
            FeedbackType::Helpful { helpful, .. } => {
                if *helpful {
                    metrics.helpfulness_rate = (metrics.helpfulness_rate * (metrics.total_feedback - 1) as f32 + 1.0) 
                        / metrics.total_feedback as f32;
                }
            },
            FeedbackType::Dismissed { .. } => {
                metrics.dismissal_rate = (metrics.dismissal_rate * (metrics.total_feedback - 1) as f32 + 1.0) 
                    / metrics.total_feedback as f32;
            },
            _ => {},
        }
    }

    fn update_user_preferences(&mut self, user_id: &str, feedback: &FeedbackSubmission) {
        // This would be more sophisticated in a real implementation
        // For now, just track basic preferences based on feedback patterns
        let recommendations = self.user_preferences
            .entry(user_id.to_string())
            .or_insert(PersonalizationRecommendations {
                recommended_tone: None,
                optimal_frequency_mins: 15,
                preferred_intervention_types: Vec::new(),
                blocked_time_windows: Vec::new(),
                content_preferences: ContentPreferences {
                    technical_level: 0.7,
                    humor_level: 0.3,
                    encouragement_level: 0.8,
                    directness: 0.6,
                },
            });

        // Adjust frequency based on dismissal patterns
        if matches!(feedback.feedback_type, FeedbackType::Dismissed { reason: DismissalReason::TooFrequent, .. }) {
            recommendations.optimal_frequency_mins = (recommendations.optimal_frequency_mins + 5).min(60);
        }

        // Add to blocked time windows if consistently dismissed at certain times
        if matches!(feedback.feedback_type, FeedbackType::Dismissed { reason: DismissalReason::BadTiming, .. }) {
            let hour = feedback.context.time_of_day;
            if !recommendations.blocked_time_windows.iter().any(|(start, end)| hour >= *start && hour < *end) {
                recommendations.blocked_time_windows.push((hour, hour + 1));
            }
        }
    }

    fn compute_analytics(&self) -> FeedbackAnalytics {
        if self.feedback_history.is_empty() {
            return FeedbackAnalytics {
                overall_satisfaction: 0.5,
                helpfulness_rate: 0.0,
                dismissal_rate: 0.0,
                avg_response_time_ms: 0,
                category_scores: HashMap::new(),
                temporal_patterns: TemporalPatterns {
                    best_times_of_day: vec![],
                    worst_times_of_day: vec![],
                    best_days_of_week: vec![],
                    optimal_frequency_mins: 15,
                },
                improvement_suggestions: vec![],
            };
        }

        let total_feedback = self.feedback_history.len();
        
        // Overall satisfaction
        let ratings: Vec<f32> = self.feedback_history.iter()
            .filter_map(|f| self.extract_rating(&f.feedback_type))
            .collect();
        let overall_satisfaction = if !ratings.is_empty() {
            ratings.iter().sum::<f32>() / ratings.len() as f32 / 5.0 // Normalize to 0-1
        } else {
            0.5
        };

        // Helpfulness rate
        let helpful_count = self.feedback_history.iter()
            .filter(|f| self.is_positive_feedback(&f.feedback_type))
            .count();
        let helpfulness_rate = helpful_count as f32 / total_feedback as f32;

        // Dismissal rate
        let dismissed_count = self.feedback_history.iter()
            .filter(|f| matches!(f.feedback_type, FeedbackType::Dismissed { .. }))
            .count();
        let dismissal_rate = dismissed_count as f32 / total_feedback as f32;

        // Average response time
        let avg_response_time_ms = if total_feedback > 0 {
            self.feedback_history.iter()
                .map(|f| f.response_time_ms)
                .sum::<u64>() / total_feedback as u64
        } else {
            0
        };

        // Category scores
        let category_scores = self.compute_category_scores();

        // Temporal patterns
        let temporal_patterns = self.compute_temporal_patterns();

        // Improvement suggestions
        let improvement_suggestions = self.get_improvement_suggestions();

        FeedbackAnalytics {
            overall_satisfaction,
            helpfulness_rate,
            dismissal_rate,
            avg_response_time_ms,
            category_scores,
            temporal_patterns,
            improvement_suggestions,
        }
    }

    fn extract_rating(&self, feedback_type: &FeedbackType) -> Option<f32> {
        match feedback_type {
            FeedbackType::Rating { score, .. } => Some(*score as f32),
            FeedbackType::Detailed { rating, .. } => Some(*rating as f32),
            FeedbackType::Helpful { helpful, .. } => Some(if *helpful { 5.0 } else { 1.0 }),
            FeedbackType::ActionTaken { effectiveness, .. } => Some(*effectiveness as f32),
            _ => None,
        }
    }

    fn is_positive_feedback(&self, feedback_type: &FeedbackType) -> bool {
        match feedback_type {
            FeedbackType::Rating { score, .. } => *score >= 4,
            FeedbackType::Helpful { helpful, .. } => *helpful,
            FeedbackType::Detailed { rating, helpfulness, .. } => *rating >= 4 && *helpfulness,
            FeedbackType::ActionTaken { effectiveness, .. } => *effectiveness >= 4,
            FeedbackType::Dismissed { .. } => false,
        }
    }

    fn compute_category_scores(&self) -> HashMap<FeedbackCategory, f32> {
        let mut scores = HashMap::new();

        // This would be more sophisticated in a real implementation
        // For now, derive scores from available feedback
        for feedback in &self.feedback_history {
            match &feedback.feedback_type {
                FeedbackType::Rating { score, category } => {
                    let current_score = scores.get(category).unwrap_or(&0.0);
                    scores.insert(category.clone(), (current_score + *score as f32) / 2.0);
                },
                FeedbackType::Detailed { 
                    rating, 
                    helpfulness, 
                    timing_appropriate, 
                    message_clear, 
                    .. 
                } => {
                    scores.insert(FeedbackCategory::MessageContent, if *helpfulness && *message_clear { *rating as f32 } else { 2.0 });
                    scores.insert(FeedbackCategory::Timing, if *timing_appropriate { *rating as f32 } else { 2.0 });
                },
                _ => {},
            }
        }

        // Normalize scores to 0-1 range
        for (_, score) in scores.iter_mut() {
            *score = (*score / 5.0).clamp(0.0, 1.0);
        }

        scores
    }

    fn compute_temporal_patterns(&self) -> TemporalPatterns {
        let mut hour_scores: HashMap<u8, Vec<f32>> = HashMap::new();
        let mut day_scores: HashMap<u8, Vec<f32>> = HashMap::new();

        for feedback in &self.feedback_history {
            if let Some(rating) = self.extract_rating(&feedback.feedback_type) {
                hour_scores.entry(feedback.context.time_of_day)
                    .or_insert_with(Vec::new)
                    .push(rating);
                
                day_scores.entry(feedback.context.day_of_week)
                    .or_insert_with(Vec::new)
                    .push(rating);
            }
        }

        // Find best and worst times
        let mut hour_averages: Vec<(u8, f32)> = hour_scores.iter()
            .map(|(hour, ratings)| (*hour, ratings.iter().sum::<f32>() / ratings.len() as f32))
            .collect();
        hour_averages.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        let best_times_of_day = hour_averages.iter().take(6).map(|(hour, _)| *hour).collect();
        let worst_times_of_day = hour_averages.iter().rev().take(3).map(|(hour, _)| *hour).collect();

        let mut day_averages: Vec<(u8, f32)> = day_scores.iter()
            .map(|(day, ratings)| (*day, ratings.iter().sum::<f32>() / ratings.len() as f32))
            .collect();
        day_averages.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        let best_days_of_week = day_averages.iter().take(3).map(|(day, _)| *day).collect();

        TemporalPatterns {
            best_times_of_day,
            worst_times_of_day,
            best_days_of_week,
            optimal_frequency_mins: 15, // This would be computed from dismissal patterns
        }
    }

    fn calculate_average_rating(&self, feedback: &[&FeedbackSubmission]) -> f32 {
        let ratings: Vec<f32> = feedback.iter()
            .filter_map(|f| self.extract_rating(&f.feedback_type))
            .collect();
        
        if ratings.is_empty() {
            3.0 // Default neutral rating
        } else {
            ratings.iter().sum::<f32>() / ratings.len() as f32
        }
    }

    fn calculate_trend_direction(&self, recent_feedback: &[&FeedbackSubmission]) -> f32 {
        if recent_feedback.len() < 10 {
            return 0.0; // Not enough data for trend
        }

        let half_point = recent_feedback.len() / 2;
        let first_half = &recent_feedback[..half_point];
        let second_half = &recent_feedback[half_point..];

        let first_avg = self.calculate_average_rating(first_half);
        let second_avg = self.calculate_average_rating(second_half);

        (second_avg - first_avg) / 5.0 // Normalize to -1.0 to 1.0
    }

    fn get_most_common_dismissal_reason(&self, feedback: &[&FeedbackSubmission]) -> Option<DismissalReason> {
        let mut reason_counts: HashMap<DismissalReason, u32> = HashMap::new();

        for f in feedback {
            if let FeedbackType::Dismissed { reason, .. } = &f.feedback_type {
                *reason_counts.entry(reason.clone()).or_insert(0) += 1;
            }
        }

        reason_counts.into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(reason, _)| reason)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackTrends {
    pub total_feedback_count: usize,
    pub helpfulness_rate: f32,
    pub dismissal_rate: f32,
    pub average_rating: f32,
    pub trend_direction: f32,        // -1.0 to 1.0, positive = improving
    pub most_common_dismissal_reason: Option<DismissalReason>,
}

impl Default for FeedbackTrends {
    fn default() -> Self {
        Self {
            total_feedback_count: 0,
            helpfulness_rate: 0.0,
            dismissal_rate: 0.0,
            average_rating: 3.0,
            trend_direction: 0.0,
            most_common_dismissal_reason: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feedback_submission() {
        let mut collector = FeedbackCollector::new();
        
        let feedback = FeedbackSubmission {
            submission_id: Uuid::new_v4(),
            intervention_id: Uuid::new_v4(),
            user_id: Some("test_user".to_string()),
            feedback_type: FeedbackType::Rating {
                score: 4,
                category: FeedbackCategory::MessageContent,
            },
            context: FeedbackContext {
                work_type: "coding".to_string(),
                focus_state: "focused".to_string(),
                intervention_type: "debug_help".to_string(),
                time_of_day: 14,
                day_of_week: 2,
                intervention_count_today: 3,
                user_session_duration_mins: 120,
            },
            submitted_at: Utc::now(),
            response_time_ms: 5000,
        };

        let result = collector.submit_feedback(feedback);
        assert!(result.is_ok());
        assert_eq!(collector.feedback_history.len(), 1);
    }

    #[test]
    fn test_analytics_computation() {
        let mut collector = FeedbackCollector::new();
        
        // Add some sample feedback
        for i in 0..5 {
            let feedback = FeedbackSubmission {
                submission_id: Uuid::new_v4(),
                intervention_id: Uuid::new_v4(),
                user_id: Some("test_user".to_string()),
                feedback_type: FeedbackType::Rating {
                    score: 4 + (i % 2), // Ratings of 4 and 5
                    category: FeedbackCategory::MessageContent,
                },
                context: FeedbackContext {
                    work_type: "coding".to_string(),
                    focus_state: "focused".to_string(),
                    intervention_type: "debug_help".to_string(),
                    time_of_day: 14,
                    day_of_week: 2,
                    intervention_count_today: i as u32,
                    user_session_duration_mins: 120,
                },
                submitted_at: Utc::now(),
                response_time_ms: 3000 + i as u64 * 1000,
            };
            collector.submit_feedback(feedback).unwrap();
        }

        let analytics = collector.get_analytics();
        assert!(analytics.overall_satisfaction > 0.8); // Should be high with 4-5 ratings
        assert!(analytics.helpfulness_rate > 0.0);
    }

    #[test]
    fn test_dismissal_tracking() {
        let mut collector = FeedbackCollector::new();
        
        let feedback = FeedbackSubmission {
            submission_id: Uuid::new_v4(),
            intervention_id: Uuid::new_v4(),
            user_id: Some("test_user".to_string()),
            feedback_type: FeedbackType::Dismissed {
                reason: DismissalReason::TooFrequent,
                too_frequent: true,
            },
            context: FeedbackContext {
                work_type: "coding".to_string(),
                focus_state: "focused".to_string(),
                intervention_type: "debug_help".to_string(),
                time_of_day: 14,
                day_of_week: 2,
                intervention_count_today: 5,
                user_session_duration_mins: 60,
            },
            submitted_at: Utc::now(),
            response_time_ms: 1000,
        };

        collector.submit_feedback(feedback).unwrap();
        
        let analytics = collector.get_analytics();
        assert!(analytics.dismissal_rate > 0.0);
        
        // Should update user preferences
        let recommendations = collector.get_personalization_recommendations("test_user").unwrap();
        assert!(recommendations.optimal_frequency_mins > 15);
    }

    #[test]
    fn test_validation() {
        let collector = FeedbackCollector::new();
        
        let invalid_feedback = FeedbackSubmission {
            submission_id: Uuid::new_v4(),
            intervention_id: Uuid::new_v4(),
            user_id: None,
            feedback_type: FeedbackType::Rating {
                score: 6, // Invalid score
                category: FeedbackCategory::MessageContent,
            },
            context: FeedbackContext {
                work_type: "coding".to_string(),
                focus_state: "focused".to_string(),
                intervention_type: "debug_help".to_string(),
                time_of_day: 14,
                day_of_week: 2,
                intervention_count_today: 1,
                user_session_duration_mins: 30,
            },
            submitted_at: Utc::now(),
            response_time_ms: 2000,
        };

        let result = collector.validate_feedback(&invalid_feedback);
        assert!(result.is_err());
    }
}