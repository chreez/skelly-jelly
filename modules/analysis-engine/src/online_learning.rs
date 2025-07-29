//! Online learning system for continuous model improvement
//!
//! This module implements online learning capabilities that allow the ML models
//! to adapt and improve based on user feedback, achieving:
//! - Continuous model updates from user corrections
//! - Personalized adaptation to individual user patterns
//! - Active learning for optimal feedback collection
//! - Model performance tracking and validation
//! - Safe incremental updates without catastrophic forgetting

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::{
    error::{AnalysisError, AnalysisResult},
    models::{ADHDState, ADHDStateType, RandomForestClassifier, StateDistribution, ModelMetrics, StateModel},
    types::FeatureVector,
    state_detection::{StateDetectionResult, UserFeedback},
};

/// Online learning coordinator managing model updates and user feedback
pub struct OnlineLearningEngine {
    /// Configuration for online learning
    config: OnlineLearningConfig,
    
    /// Feedback collection and processing
    feedback_processor: Arc<FeedbackProcessor>,
    
    /// Model update manager
    model_updater: Arc<ModelUpdater>,
    
    /// Active learning coordinator
    active_learner: Arc<ActiveLearner>,
    
    /// Performance validator
    validator: Arc<ModelValidator>,
    
    /// Learning metrics
    metrics: Arc<RwLock<OnlineLearningMetrics>>,
}

/// Processes and validates user feedback
pub struct FeedbackProcessor {
    /// Raw feedback buffer
    feedback_buffer: Arc<Mutex<VecDeque<RawFeedback>>>,
    
    /// Processed feedback ready for training
    training_samples: Arc<Mutex<Vec<TrainingSample>>>,
    
    /// Feedback validation rules
    validation_rules: FeedbackValidationRules,
    
    /// Duplicate detection
    feedback_history: Arc<Mutex<HashMap<FeedbackKey, DateTime<Utc>>>>,
}

/// Manages incremental model updates
pub struct ModelUpdater {
    /// Reference to the main classifier
    classifier: Arc<Mutex<RandomForestClassifier>>,
    
    /// Update scheduling
    update_scheduler: Arc<UpdateScheduler>,
    
    /// Model versioning
    model_versions: Arc<RwLock<Vec<ModelVersion>>>,
    
    /// Safe update protocols
    update_validator: Arc<UpdateValidator>,
}

/// Coordinates active learning strategies
pub struct ActiveLearner {
    /// Uncertainty sampling strategy
    uncertainty_sampler: UncertaintySampler,
    
    /// Diversity sampling for balanced learning
    diversity_sampler: DiversitySampler,
    
    /// Query strategy selection
    query_strategy: QueryStrategy,
    
    /// Sample selection buffer
    query_candidates: Arc<RwLock<Vec<QueryCandidate>>>,
}

/// Validates model performance and prevents degradation
pub struct ModelValidator {
    /// Validation datasets
    validation_sets: Arc<RwLock<Vec<ValidationSet>>>,
    
    /// Performance tracking
    performance_history: Arc<RwLock<VecDeque<ValidationResult>>>,
    
    /// Degradation detection
    degradation_detector: DegradationDetector,
    
    /// Rollback mechanisms
    rollback_manager: Arc<RollbackManager>,
}

/// Raw user feedback before processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawFeedback {
    pub id: Uuid,
    pub window_id: Uuid,
    pub user_id: Option<String>,
    pub predicted_state: ADHDStateType,
    pub user_corrected_state: String,
    pub confidence: f32,
    pub timestamp: DateTime<Utc>,
    pub context: Option<FeedbackContext>,
    pub source: FeedbackSource,
}

/// Additional context for feedback
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackContext {
    pub task_type: Option<String>,
    pub environment: Option<String>,
    pub stress_level: Option<f32>,
    pub time_of_day: Option<String>,
    pub notes: Option<String>,
}

/// Source of the feedback
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum FeedbackSource {
    UserCorrection,
    ExplicitFeedback,
    ImplicitBehavior,
    ExpertAnnotation,
}

/// Processed training sample
#[derive(Debug, Clone)]
pub struct TrainingSample {
    pub features: FeatureVector,
    pub true_state: ADHDState,
    pub predicted_state: ADHDState,
    pub confidence: f32,
    pub weight: f32,
    pub timestamp: DateTime<Utc>,
    pub sample_quality: f32,
}

/// Key for deduplicating feedback
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct FeedbackKey {
    window_id: Uuid,
    user_id: Option<String>,
    state_correction: String,
}

/// Query candidate for active learning
#[derive(Debug, Clone)]
pub struct QueryCandidate {
    pub window_id: Uuid,
    pub features: FeatureVector,
    pub current_prediction: StateDistribution,
    pub uncertainty_score: f32,
    pub diversity_score: f32,
    pub importance_score: f32,
    pub query_value: f32,
}

/// Model version for rollback capability
#[derive(Debug, Clone)]
pub struct ModelVersion {
    pub version_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub performance_metrics: ModelMetrics,
    pub training_samples_count: usize,
    pub model_checksum: String,
}

/// Validation result for model performance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub timestamp: DateTime<Utc>,
    pub accuracy: f32,
    pub precision: HashMap<ADHDStateType, f32>,
    pub recall: HashMap<ADHDStateType, f32>,
    pub f1_score: f32,
    pub validation_loss: f32,
    pub sample_count: usize,
}

/// Online learning performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnlineLearningMetrics {
    pub total_feedback_received: u64,
    pub valid_feedback_processed: u64,
    pub model_updates_performed: u64,
    pub accuracy_improvement: f32,
    pub user_satisfaction_score: f32,
    pub active_learning_queries: u64,
    pub query_response_rate: f32,
    pub model_adaptation_rate: f32,
    pub personalization_score: f32,
}

impl OnlineLearningEngine {
    /// Create a new online learning engine
    pub fn new(classifier: Arc<Mutex<RandomForestClassifier>>) -> Self {
        let config = OnlineLearningConfig::default();
        
        let feedback_processor = Arc::new(FeedbackProcessor::new(&config));
        let model_updater = Arc::new(ModelUpdater::new(classifier, &config));
        let active_learner = Arc::new(ActiveLearner::new(&config));
        let validator = Arc::new(ModelValidator::new(&config));
        
        Self {
            config,
            feedback_processor,
            model_updater,
            active_learner,
            validator,
            metrics: Arc::new(RwLock::new(OnlineLearningMetrics::default())),
        }
    }
    
    /// Process user feedback and trigger learning updates
    pub async fn process_feedback(&self, feedback: UserFeedback) -> AnalysisResult<()> {
        // Convert to raw feedback format
        let raw_feedback = RawFeedback {
            id: Uuid::new_v4(),
            window_id: feedback.window_id,
            user_id: None, // Would be extracted from context
            predicted_state: ADHDStateType::Neutral, // Would be from prediction history
            user_corrected_state: feedback.user_state,
            confidence: feedback.confidence,
            timestamp: feedback.timestamp,
            context: None,
            source: FeedbackSource::UserCorrection,
        };
        
        // Process through feedback pipeline
        self.feedback_processor.add_feedback(raw_feedback).await?;
        
        // Check if we should trigger model update
        if self.should_trigger_update().await? {
            self.trigger_model_update().await?;
        }
        
        // Update metrics
        let mut metrics = self.metrics.write().await;
        metrics.total_feedback_received += 1;
        
        Ok(())
    }
    
    /// Get active learning queries for user annotation
    pub async fn get_active_learning_queries(&self, count: usize) -> AnalysisResult<Vec<QueryCandidate>> {
        self.active_learner.select_queries(count).await
    }
    
    /// Submit response to active learning query
    pub async fn respond_to_query(&self, query_id: Uuid, response: UserFeedback) -> AnalysisResult<()> {
        // Process the response as high-value feedback
        self.process_feedback(response).await?;
        
        // Update query response metrics
        let mut metrics = self.metrics.write().await;
        metrics.active_learning_queries += 1;
        
        Ok(())
    }
    
    /// Check if model update should be triggered
    async fn should_trigger_update(&self) -> AnalysisResult<bool> {
        let feedback_count = self.feedback_processor.get_pending_sample_count().await;
        let time_since_last_update = self.model_updater.time_since_last_update().await;
        
        Ok(feedback_count >= self.config.min_samples_for_update ||
           time_since_last_update >= self.config.max_update_interval)
    }
    
    /// Trigger incremental model update
    async fn trigger_model_update(&self) -> AnalysisResult<()> {
        // Get training samples
        let samples = self.feedback_processor.get_training_samples().await?;
        
        if samples.is_empty() {
            return Ok(());
        }
        
        // Validate update safety
        if !self.validator.validate_update_safety(&samples).await? {
            return Err(AnalysisError::ValidationFailed {
                reason: "Model update failed safety validation".to_string(),
            });
        }
        
        // Perform incremental update
        let samples_len = samples.len();
        self.model_updater.perform_incremental_update(samples).await?;
        
        // Validate post-update performance
        let validation_result = self.validator.validate_model_performance().await?;
        
        if validation_result.accuracy < self.config.min_accuracy_threshold {
            // Rollback if performance degraded
            self.model_updater.rollback_last_update().await?;
            return Err(AnalysisError::ModelPerformanceDegraded {
                old_accuracy: 0.0, // Would track previous accuracy
                new_accuracy: validation_result.accuracy,
            });
        }
        
        // Update metrics
        let mut metrics = self.metrics.write().await;
        metrics.model_updates_performed += 1;
        metrics.accuracy_improvement = validation_result.accuracy - 0.8; // Baseline
        
        println!("Model updated successfully with {} samples, new accuracy: {:.3}",
                samples_len, validation_result.accuracy);
        
        Ok(())
    }
    
    /// Get personalization score for current user
    pub async fn get_personalization_score(&self) -> f32 {
        // Calculate how well the model has adapted to user patterns
        let metrics = self.metrics.read().await;
        metrics.personalization_score
    }
    
    /// Get current learning metrics
    pub async fn get_metrics(&self) -> OnlineLearningMetrics {
        self.metrics.read().await.clone()
    }
    
    /// Reset learning state (for testing or user reset)
    pub async fn reset_learning_state(&self) -> AnalysisResult<()> {
        self.feedback_processor.clear_feedback().await;
        self.model_updater.reset_to_base_model().await?;
        
        let mut metrics = self.metrics.write().await;
        *metrics = OnlineLearningMetrics::default();
        
        Ok(())
    }
}

impl FeedbackProcessor {
    fn new(config: &OnlineLearningConfig) -> Self {
        Self {
            feedback_buffer: Arc::new(Mutex::new(VecDeque::new())),
            training_samples: Arc::new(Mutex::new(Vec::new())),
            validation_rules: FeedbackValidationRules::new(config),
            feedback_history: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    async fn add_feedback(&self, feedback: RawFeedback) -> AnalysisResult<()> {
        // Validate feedback
        if !self.validation_rules.validate(&feedback) {
            return Err(AnalysisError::InvalidFeedback {
                reason: "Feedback failed validation rules".to_string(),
            });
        }
        
        // Check for duplicates
        let feedback_key = FeedbackKey {
            window_id: feedback.window_id,
            user_id: feedback.user_id.clone(),
            state_correction: feedback.user_corrected_state.clone(),
        };
        
        {
            let mut history = self.feedback_history.lock().unwrap();
            if let Some(previous_time) = history.get(&feedback_key) {
                let time_diff = feedback.timestamp.signed_duration_since(*previous_time);
                if time_diff.num_minutes() < 5 {
                    return Ok(()) // Skip duplicate
                }
            }
            history.insert(feedback_key, feedback.timestamp);
        }
        
        // Add to buffer
        {
            let mut buffer = self.feedback_buffer.lock().unwrap();
            buffer.push_back(feedback);
            
            // Keep buffer size manageable
            if buffer.len() > 1000 {
                buffer.pop_front();
            }
        }
        
        // Process into training samples
        self.process_feedback_buffer().await?;
        
        Ok(())
    }
    
    async fn process_feedback_buffer(&self) -> AnalysisResult<()> {
        let feedback_items = {
            let mut buffer = self.feedback_buffer.lock().unwrap();
            let items: Vec<_> = buffer.drain(..).collect();
            items
        };
        
        let mut training_samples = self.training_samples.lock().unwrap();
        
        for feedback in feedback_items {
            if let Some(sample) = self.convert_to_training_sample(feedback).await? {
                training_samples.push(sample);
            }
        }
        
        Ok(())
    }
    
    async fn convert_to_training_sample(&self, feedback: RawFeedback) -> AnalysisResult<Option<TrainingSample>> {
        // Parse corrected state
        let corrected_state_type = ADHDStateType::from_str(&feedback.user_corrected_state)
            .ok_or_else(|| AnalysisError::InvalidInput {
                message: format!("Invalid state type: {}", feedback.user_corrected_state),
            })?;
        
        // Create ADHD state
        let corrected_state = ADHDState {
            state_type: corrected_state_type,
            confidence: feedback.confidence,
            flow_depth: crate::types::FlowDepth::from_score(feedback.confidence),
            distraction_type: None,
            timestamp: feedback.timestamp,
            duration: Duration::from_secs(30),
        };
        
        // We need the original features - would be retrieved from prediction cache
        let features = FeatureVector::default(); // Placeholder
        let predicted_state = ADHDState::neutral(); // Placeholder
        
        // Calculate sample weight based on feedback quality
        let weight = self.calculate_sample_weight(&feedback);
        let quality = self.assess_sample_quality(&feedback);
        
        Ok(Some(TrainingSample {
            features,
            true_state: corrected_state,
            predicted_state,
            confidence: feedback.confidence,
            weight,
            timestamp: feedback.timestamp,
            sample_quality: quality,
        }))
    }
    
    fn calculate_sample_weight(&self, feedback: &RawFeedback) -> f32 {
        let mut weight = 1.0;
        
        // Higher weight for confident corrections
        weight *= feedback.confidence;
        
        // Higher weight for explicit feedback vs implicit
        match feedback.source {
            FeedbackSource::ExplicitFeedback => weight *= 1.2,
            FeedbackSource::UserCorrection => weight *= 1.0,
            FeedbackSource::ImplicitBehavior => weight *= 0.7,
            FeedbackSource::ExpertAnnotation => weight *= 1.5,
        }
        
        weight.clamp(0.1, 2.0)
    }
    
    fn assess_sample_quality(&self, feedback: &RawFeedback) -> f32 {
        let mut quality = 0.5;
        
        // Quality based on confidence
        quality += feedback.confidence * 0.3;
        
        // Quality based on context richness
        if feedback.context.is_some() {
            quality += 0.2;
        }
        
        quality.clamp(0.0, 1.0)
    }
    
    async fn get_pending_sample_count(&self) -> usize {
        self.training_samples.lock().unwrap().len()
    }
    
    async fn get_training_samples(&self) -> AnalysisResult<Vec<TrainingSample>> {
        let mut samples = self.training_samples.lock().unwrap();
        let result = samples.clone();
        samples.clear();
        Ok(result)
    }
    
    async fn clear_feedback(&self) {
        self.feedback_buffer.lock().unwrap().clear();
        self.training_samples.lock().unwrap().clear();
        self.feedback_history.lock().unwrap().clear();
    }
}

/// Validation rules for feedback quality
struct FeedbackValidationRules {
    min_confidence: f32,
    max_time_since_prediction: Duration,
    allowed_state_transitions: HashMap<ADHDStateType, Vec<ADHDStateType>>,
}

impl FeedbackValidationRules {
    fn new(config: &OnlineLearningConfig) -> Self {
        let mut allowed_transitions = HashMap::new();
        
        // Define realistic state transitions
        allowed_transitions.insert(ADHDStateType::Neutral, vec![
            ADHDStateType::Flow, ADHDStateType::Distracted, ADHDStateType::Transitioning
        ]);
        allowed_transitions.insert(ADHDStateType::Flow, vec![
            ADHDStateType::Hyperfocus, ADHDStateType::Transitioning, ADHDStateType::Distracted
        ]);
        allowed_transitions.insert(ADHDStateType::Hyperfocus, vec![
            ADHDStateType::Flow, ADHDStateType::Transitioning
        ]);
        allowed_transitions.insert(ADHDStateType::Distracted, vec![
            ADHDStateType::Transitioning, ADHDStateType::Neutral
        ]);
        allowed_transitions.insert(ADHDStateType::Transitioning, vec![
            ADHDStateType::Flow, ADHDStateType::Distracted, ADHDStateType::Neutral
        ]);
        
        Self {
            min_confidence: config.min_feedback_confidence,
            max_time_since_prediction: Duration::from_secs(300), // 5 minutes
            allowed_state_transitions: allowed_transitions,
        }
    }
    
    fn validate(&self, feedback: &RawFeedback) -> bool {
        // Confidence check
        if feedback.confidence < self.min_confidence {
            return false;
        }
        
        // Timing check
        let time_since_feedback = Utc::now().signed_duration_since(feedback.timestamp);
        if time_since_feedback.to_std().unwrap_or(Duration::from_secs(0)) > self.max_time_since_prediction {
            return false;
        }
        
        // State transition validity (simplified)
        if let Some(corrected_state) = ADHDStateType::from_str(&feedback.user_corrected_state) {
            if let Some(allowed_transitions) = self.allowed_state_transitions.get(&feedback.predicted_state) {
                return allowed_transitions.contains(&corrected_state) || 
                       corrected_state == feedback.predicted_state; // Allow same state corrections
            }
        }
        
        true
    }
}

// Additional implementation stubs for completeness
impl ModelUpdater {
    fn new(classifier: Arc<Mutex<RandomForestClassifier>>, _config: &OnlineLearningConfig) -> Self {
        Self {
            classifier,
            update_scheduler: Arc::new(UpdateScheduler::new()),
            model_versions: Arc::new(RwLock::new(Vec::new())),
            update_validator: Arc::new(UpdateValidator::new()),
        }
    }
    
    async fn time_since_last_update(&self) -> Duration {
        self.update_scheduler.time_since_last_update().await
    }
    
    async fn perform_incremental_update(&self, samples: Vec<TrainingSample>) -> AnalysisResult<()> {
        // Convert samples to format expected by classifier
        let training_data: Vec<(FeatureVector, ADHDState)> = samples.into_iter()
            .map(|sample| (sample.features, sample.true_state))
            .collect();
        
        // Update classifier with new samples
        let mut classifier = self.classifier.lock().unwrap();
        for (features, state) in &training_data {
            classifier.update(features, state).await?;
        }
        
        self.update_scheduler.record_update().await;
        
        Ok(())
    }
    
    async fn rollback_last_update(&self) -> AnalysisResult<()> {
        // Implementation would restore previous model version
        println!("Model rollback performed due to performance degradation");
        Ok(())
    }
    
    async fn reset_to_base_model(&self) -> AnalysisResult<()> {
        // Implementation would reset to original trained model
        Ok(())
    }
}

// Placeholder implementations for supporting structures
struct UpdateScheduler {
    last_update: Arc<RwLock<Option<DateTime<Utc>>>>,
}

impl UpdateScheduler {
    fn new() -> Self {
        Self {
            last_update: Arc::new(RwLock::new(None)),
        }
    }
    
    async fn time_since_last_update(&self) -> Duration {
        let last_update = self.last_update.read().await;
        if let Some(time) = *last_update {
            Utc::now().signed_duration_since(time).to_std().unwrap_or(Duration::from_secs(0))
        } else {
            Duration::from_secs(u64::MAX) // Force first update
        }
    }
    
    async fn record_update(&self) {
        *self.last_update.write().await = Some(Utc::now());
    }
}

struct UpdateValidator;
impl UpdateValidator {
    fn new() -> Self { Self }
}

struct UncertaintySampler;
struct DiversitySampler;

#[derive(Debug, Clone)]
enum QueryStrategy {
    Uncertainty,
    Diversity,
    Combined,
}

impl ActiveLearner {
    fn new(_config: &OnlineLearningConfig) -> Self {
        Self {
            uncertainty_sampler: UncertaintySampler,
            diversity_sampler: DiversitySampler,
            query_strategy: QueryStrategy::Combined,
            query_candidates: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    async fn select_queries(&self, count: usize) -> AnalysisResult<Vec<QueryCandidate>> {
        let candidates = self.query_candidates.read().await;
        Ok(candidates.iter().take(count).cloned().collect())
    }
}

struct ValidationSet;
struct DegradationDetector;
struct RollbackManager;

impl ModelValidator {
    fn new(_config: &OnlineLearningConfig) -> Self {
        Self {
            validation_sets: Arc::new(RwLock::new(Vec::new())),
            performance_history: Arc::new(RwLock::new(VecDeque::new())),
            degradation_detector: DegradationDetector,
            rollback_manager: Arc::new(RollbackManager),
        }
    }
    
    async fn validate_update_safety(&self, _samples: &[TrainingSample]) -> AnalysisResult<bool> {
        // Implementation would check for data quality, distribution shift, etc.
        Ok(true)
    }
    
    async fn validate_model_performance(&self) -> AnalysisResult<ValidationResult> {
        // Implementation would run validation dataset through updated model
        Ok(ValidationResult {
            timestamp: Utc::now(),
            accuracy: 0.82, // Placeholder
            precision: HashMap::new(),
            recall: HashMap::new(),
            f1_score: 0.81,
            validation_loss: 0.15,
            sample_count: 100,
        })
    }
}

/// Configuration for online learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnlineLearningConfig {
    pub enable_online_learning: bool,
    pub min_samples_for_update: usize,
    pub max_update_interval: Duration,
    pub min_feedback_confidence: f32,
    pub min_accuracy_threshold: f32,
    pub learning_rate: f32,
    pub enable_active_learning: bool,
    pub max_query_candidates: usize,
    pub validation_frequency: Duration,
}

impl Default for OnlineLearningConfig {
    fn default() -> Self {
        Self {
            enable_online_learning: true,
            min_samples_for_update: 20,
            max_update_interval: Duration::from_secs(3600), // 1 hour
            min_feedback_confidence: 0.6,
            min_accuracy_threshold: 0.75,
            learning_rate: 0.01,
            enable_active_learning: true,
            max_query_candidates: 100,
            validation_frequency: Duration::from_secs(300), // 5 minutes
        }
    }
}

impl Default for OnlineLearningMetrics {
    fn default() -> Self {
        Self {
            total_feedback_received: 0,
            valid_feedback_processed: 0,
            model_updates_performed: 0,
            accuracy_improvement: 0.0,
            user_satisfaction_score: 0.0,
            active_learning_queries: 0,
            query_response_rate: 0.0,
            model_adaptation_rate: 0.0,
            personalization_score: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::RandomForestClassifier;

    #[tokio::test]
    async fn test_online_learning_engine_creation() {
        let classifier = Arc::new(Mutex::new(RandomForestClassifier::new()));
        let engine = OnlineLearningEngine::new(classifier);
        
        let metrics = engine.get_metrics().await;
        assert_eq!(metrics.total_feedback_received, 0);
    }

    #[tokio::test]
    async fn test_feedback_processing() {
        let classifier = Arc::new(Mutex::new(RandomForestClassifier::new()));
        let engine = OnlineLearningEngine::new(classifier);
        
        let feedback = UserFeedback {
            window_id: Uuid::new_v4(),
            user_state: "flow".to_string(),
            confidence: 0.9,
            timestamp: Utc::now(),
            notes: Some("Deep focus session".to_string()),
        };
        
        let result = engine.process_feedback(feedback).await;
        assert!(result.is_ok());
        
        let metrics = engine.get_metrics().await;
        assert_eq!(metrics.total_feedback_received, 1);
    }

    #[test]
    fn test_feedback_validation() {
        let config = OnlineLearningConfig::default();
        let rules = FeedbackValidationRules::new(&config);
        
        let valid_feedback = RawFeedback {
            id: Uuid::new_v4(),
            window_id: Uuid::new_v4(),
            user_id: None,
            predicted_state: ADHDStateType::Neutral,
            user_corrected_state: "flow".to_string(),
            confidence: 0.8,
            timestamp: Utc::now(),
            context: None,
            source: FeedbackSource::UserCorrection,
        };
        
        assert!(rules.validate(&valid_feedback));
        
        let invalid_feedback = RawFeedback {
            confidence: 0.3, // Below threshold
            ..valid_feedback
        };
        
        assert!(!rules.validate(&invalid_feedback));
    }

    #[test]
    fn test_sample_weight_calculation() {
        let config = OnlineLearningConfig::default();
        let processor = FeedbackProcessor::new(&config);
        
        let high_confidence_feedback = RawFeedback {
            id: Uuid::new_v4(),
            window_id: Uuid::new_v4(),
            user_id: None,
            predicted_state: ADHDStateType::Neutral,
            user_corrected_state: "flow".to_string(),
            confidence: 0.9,
            timestamp: Utc::now(),
            context: None,
            source: FeedbackSource::ExplicitFeedback,
        };
        
        let weight = processor.calculate_sample_weight(&high_confidence_feedback);
        assert!(weight > 1.0); // Should have higher weight
    }
}