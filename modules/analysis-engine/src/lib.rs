//! # Analysis Engine Module
//!
//! The intelligence core of Skelly-Jelly, processing behavioral data streams 
//! to detect ADHD states and work patterns in real-time using lightweight ML models.
//!
//! ## Core Capabilities
//! - **Real-time Analysis**: <50ms inference on behavioral patterns
//! - **State Detection**: Flow, hyperfocus, distracted, transitioning states
//! - **Feature Extraction**: Keystroke dynamics, mouse patterns, window behavior
//! - **Screenshot Analysis**: Work context extraction with privacy preservation
//! - **Behavioral Metrics**: Comprehensive productivity and focus metrics
//! - **Online Learning**: Continuous adaptation to user patterns

pub mod analysis_engine;
pub mod error;
pub mod event_bus_integration;
pub mod event_processor;
pub mod feature_extraction;
pub mod inference;
pub mod metrics;
pub mod models;
pub mod online_learning;
pub mod performance_validation;
pub mod privacy;
pub mod screenshot;
pub mod sliding_window;
pub mod state_detection;
pub mod training_pipeline;
pub mod types;

// Re-export public API
pub use analysis_engine::{AnalysisEngineImpl, AnalysisEngineConfig};
pub use error::{AnalysisError, AnalysisResult};
pub use event_bus_integration::{EventBusIntegration, EventBusConfig, EventProcessingMetrics, ProcessingStatus};
pub use event_processor::EventProcessor;
pub use feature_extraction::{FeatureExtractionPipeline, FeatureExtractor};
pub use inference::{InferenceEngine, InferenceConfig, InferencePriority};
pub use metrics::{BehavioralMetrics, MetricEngine};
pub use models::{ADHDState, StateClassifier, StateDistribution, RandomForestClassifier, ONNXClassifier, StateModel};
pub use online_learning::{OnlineLearningEngine, OnlineLearningConfig, UserFeedback as OnlineUserFeedback};
pub use performance_validation::{PerformanceValidator, ValidationConfig, ValidationResult, ValidationStatus};
pub use privacy::{LocalInferenceEngine, NetworkIsolationReport};
pub use screenshot::{ScreenshotAnalyzer, ScreenshotContext, WorkType};
pub use sliding_window::{AnalysisWindow, SlidingWindowManager};
pub use state_detection::{StateDetectionEngine, StateDetectionResult, StateDetectionConfig};
pub use training_pipeline::{TrainingPipeline, TrainingConfig, HyperparameterResults, TrainingStats};
pub use types::{AnalysisResult as AnalysisResultType, FeatureVector, FlowDepth, DistractionType};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use skelly_jelly_event_bus::{EventBusTrait, ModuleId};
use skelly_jelly_storage::types::EventBatch;
use std::{sync::Arc, time::Duration};
use uuid::Uuid;

/// Main trait for the analysis engine
#[async_trait]
pub trait AnalysisEngineTrait: Send + Sync {
    /// Process a batch of events from storage
    async fn analyze_batch(&self, batch: EventBatch) -> AnalysisResult<AnalysisResultType>;
    
    /// Get current ADHD state classification
    async fn get_current_state(&self) -> ADHDState;
    
    /// Get current behavioral metrics
    async fn get_metrics(&self) -> BehavioralMetrics;
    
    /// Process user feedback for online learning
    async fn process_feedback(&self, feedback: UserFeedback) -> AnalysisResult<()>;
    
    /// Update analysis configuration
    async fn update_config(&self, config: AnalysisEngineConfig) -> AnalysisResult<()>;
    
    /// Get analysis performance metrics
    async fn get_performance_metrics(&self) -> PerformanceMetrics;
}

/// User feedback for online learning
#[derive(Debug, Clone)]
pub struct UserFeedback {
    pub window_id: Uuid,
    pub user_state: String,
    pub confidence: f32,
    pub timestamp: DateTime<Utc>,
    pub notes: Option<String>,
}

/// Performance metrics for the analysis engine
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub avg_inference_time_ms: f32,
    pub total_analyses: u64,
    pub accuracy_score: f32,
    pub memory_usage_mb: f32,
    pub cache_hit_rate: f32,
}

/// Create a new analysis engine instance
pub async fn create_analysis_engine(
    config: AnalysisEngineConfig,
    event_bus: Arc<dyn EventBusTrait>,
) -> AnalysisResult<Arc<dyn AnalysisEngineTrait>> {
    let engine = AnalysisEngineImpl::new(config, event_bus).await?;
    Ok(Arc::new(engine))
}