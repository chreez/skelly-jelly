//! Error types for the analysis engine module

use thiserror::Error;

/// Analysis engine error types
#[derive(Error, Debug)]
pub enum AnalysisError {
    #[error("Model inference failed: {model} - {source}")]
    InferenceError {
        model: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Feature extraction failed for {feature_type}: {reason}")]
    FeatureExtractionError {
        feature_type: String,
        reason: String,
    },

    #[error("Screenshot analysis failed: {reason}")]
    ScreenshotError { reason: String },

    #[error("Insufficient data for analysis: required {required}, available {available}")]
    InsufficientData { required: usize, available: usize },

    #[error("Configuration error: {message}")]
    ConfigError { message: String },

    #[error("Event processing error: {message}")]
    EventProcessingError { message: String },

    #[error("Model not found: {model_name}")]
    ModelNotFound { model_name: String },

    #[error("Invalid feature vector: {reason}")]
    InvalidFeatureVector { reason: String },

    #[error("Window management error: {reason}")]
    WindowError { reason: String },

    #[error("Memory allocation error: {message}")]
    MemoryError { message: String },

    #[error("Serialization error: {source}")]
    SerializationError {
        #[from]
        source: serde_json::Error,
    },

    #[error("IO error: {source}")]
    IoError {
        #[from]
        source: std::io::Error,
    },

    #[error("Image processing error: {source}")]
    ImageError {
        #[from]
        source: image::ImageError,
    },

    #[error("Event bus error: {source}")]
    EventBusError {
        #[from]
        source: skelly_jelly_event_bus::EventBusError,
    },

    #[error("Math computation error: {message}")]
    MathError { message: String },

    #[error("Resource exhausted: {resource}")]
    ResourceExhausted { resource: String },

    #[error("Invalid input: {message}")]
    InvalidInput { message: String },

    #[error("Training failed: {message}")]
    TrainingFailed { message: String },

    #[error("Prediction failed: {message}")]
    PredictionFailed { message: String },

    #[error("Concurrency error in {operation}")]
    ConcurrencyError { operation: String },

    #[error("Validation failed: {reason}")]
    ValidationFailed { reason: String },

    #[error("Model performance degraded: old accuracy {old_accuracy:.3}, new accuracy {new_accuracy:.3}")]
    ModelPerformanceDegraded { old_accuracy: f32, new_accuracy: f32 },

    #[error("Invalid feedback: {reason}")]
    InvalidFeedback { reason: String },

    #[error("Operation '{operation}' timed out after {timeout_ms}ms")]
    TimeoutError { operation: String, timeout_ms: u64 },
}

/// Result type for analysis operations
pub type AnalysisResult<T> = Result<T, AnalysisError>;

impl AnalysisError {
    /// Check if the error is recoverable
    pub fn is_recoverable(&self) -> bool {
        match self {
            AnalysisError::InferenceError { .. } => true,
            AnalysisError::FeatureExtractionError { .. } => true,
            AnalysisError::ScreenshotError { .. } => true,
            AnalysisError::InsufficientData { .. } => false,
            AnalysisError::ConfigError { .. } => false,
            AnalysisError::EventProcessingError { .. } => true,
            AnalysisError::ModelNotFound { .. } => false,
            AnalysisError::InvalidFeatureVector { .. } => false,
            AnalysisError::WindowError { .. } => true,
            AnalysisError::MemoryError { .. } => true,
            AnalysisError::SerializationError { .. } => false,
            AnalysisError::IoError { .. } => true,
            AnalysisError::ImageError { .. } => true,
            AnalysisError::EventBusError { .. } => true,
            AnalysisError::MathError { .. } => false,
            AnalysisError::TimeoutError { .. } => true,
            AnalysisError::ConcurrencyError { .. } => true,
            AnalysisError::ValidationFailed { .. } => false,
            AnalysisError::ModelPerformanceDegraded { .. } => false,
            AnalysisError::InvalidFeedback { .. } => false,
            AnalysisError::ResourceExhausted { .. } => true,
            AnalysisError::InvalidInput { .. } => false,
            AnalysisError::TrainingFailed { .. } => false,
            AnalysisError::PredictionFailed { .. } => true,
        }
    }

    /// Get the error severity level
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            AnalysisError::ModelNotFound { .. } => ErrorSeverity::Critical,
            AnalysisError::ConfigError { .. } => ErrorSeverity::Critical,
            AnalysisError::MemoryError { .. } => ErrorSeverity::High,
            AnalysisError::ResourceExhausted { .. } => ErrorSeverity::High,
            AnalysisError::InferenceError { .. } => ErrorSeverity::Medium,
            AnalysisError::FeatureExtractionError { .. } => ErrorSeverity::Medium,
            AnalysisError::EventProcessingError { .. } => ErrorSeverity::Medium,
            AnalysisError::WindowError { .. } => ErrorSeverity::Medium,
            AnalysisError::TimeoutError { .. } => ErrorSeverity::Medium,
            AnalysisError::ConcurrencyError { .. } => ErrorSeverity::Medium,
            AnalysisError::ValidationFailed { .. } => ErrorSeverity::High,
            AnalysisError::ModelPerformanceDegraded { .. } => ErrorSeverity::High,
            AnalysisError::InvalidFeedback { .. } => ErrorSeverity::Low,
            AnalysisError::ScreenshotError { .. } => ErrorSeverity::Low,
            AnalysisError::InsufficientData { .. } => ErrorSeverity::Low,
            AnalysisError::InvalidFeatureVector { .. } => ErrorSeverity::Low,
            AnalysisError::SerializationError { .. } => ErrorSeverity::Low,
            AnalysisError::IoError { .. } => ErrorSeverity::Low,
            AnalysisError::ImageError { .. } => ErrorSeverity::Low,
            AnalysisError::EventBusError { .. } => ErrorSeverity::Medium,
            AnalysisError::MathError { .. } => ErrorSeverity::Low,
            AnalysisError::InvalidInput { .. } => ErrorSeverity::Low,
            AnalysisError::TrainingFailed { .. } => ErrorSeverity::High,
            AnalysisError::PredictionFailed { .. } => ErrorSeverity::Medium,
        }
    }
}

/// Error severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ErrorSeverity {
    Low = 0,
    Medium = 1,
    High = 2,
    Critical = 3,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_severity() {
        let error = AnalysisError::ModelNotFound {
            model_name: "test".to_string(),
        };
        assert_eq!(error.severity(), ErrorSeverity::Critical);
        assert!(!error.is_recoverable());
    }

    #[test]
    fn test_recoverable_errors() {
        let error = AnalysisError::InferenceError {
            model: "test".to_string(),
            source: Box::new(std::io::Error::new(std::io::ErrorKind::Other, "test")),
        };
        assert!(error.is_recoverable());
        assert_eq!(error.severity(), ErrorSeverity::Medium);
    }
}