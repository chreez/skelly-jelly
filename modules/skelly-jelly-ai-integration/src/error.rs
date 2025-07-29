//! Error types for the AI Integration module
//!
//! Provides comprehensive error handling with security-conscious error messages
//! that don't leak sensitive information.

use thiserror::Error;
use std::time::Duration;

/// Result type alias for AI Integration operations
pub type Result<T> = std::result::Result<T, AIIntegrationError>;

/// Comprehensive error types for AI Integration failures
#[derive(Error, Debug)]
pub enum AIIntegrationError {
    // Model loading and initialization errors
    #[error("Failed to load local model: {reason}")]
    ModelLoadFailed { reason: String },

    #[error("Model not found at path")]
    ModelNotFound,

    #[error("Insufficient memory: required {required_mb}MB, available {available_mb}MB")]
    InsufficientMemory { required_mb: usize, available_mb: usize },

    #[error("GPU support not available")]
    GPUNotAvailable,

    // Inference and generation errors
    #[error("Local inference failed")]
    InferenceFailed,

    #[error("Context too long: {tokens} tokens exceeds maximum {max_tokens}")]
    ContextTooLong { tokens: usize, max_tokens: usize },

    #[error("Generation timeout after {duration:?}")]
    GenerationTimeout { duration: Duration },

    #[error("Model output invalid")]
    InvalidOutput,

    // API fallback errors
    #[error("API key missing for service: {service}")]
    APIKeyMissing { service: String },

    #[error("API rate limited, retry after {retry_after:?}")]
    APIRateLimited { retry_after: Duration },

    #[error("API error from {service}: {status} - rate limit or service issue")]
    APIError { service: String, status: u16 },

    #[error("API request timeout")]
    APITimeout,

    #[error("Monthly API cost limit exceeded")]
    CostLimitExceeded,

    // Privacy and security errors
    #[error("Privacy violation detected: prompt contains sensitive data")]
    PrivacyViolation,

    #[error("User consent required for API usage")]
    ConsentRequired,

    #[error("PII detected in prompt, refusing to process")]
    PIIDetected,

    #[error("Prompt injection attempt detected")]
    PromptInjectionDetected,

    // Context processing errors
    #[error("Failed to analyze work context")]
    ContextAnalysisFailed,

    #[error("Invalid intervention type")]
    InvalidInterventionType,

    #[error("Context compression failed")]
    ContextCompressionFailed,

    // Personality and suggestion errors
    #[error("Personality application failed")]
    PersonalityApplicationFailed,

    #[error("Template not found for context")]
    TemplateNotFound,

    #[error("Suggestion validation failed")]
    SuggestionValidationFailed,

    // Configuration and setup errors
    #[error("Invalid configuration: {field}")]
    InvalidConfig { field: String },

    #[error("Module not initialized")]
    NotInitialized,

    #[error("Resource unavailable")]
    ResourceUnavailable,

    // I/O and system errors
    #[error("File system error")]
    FileSystemError,

    #[error("Network error")]
    NetworkError,

    #[error("Serialization error")]
    SerializationError,

    // Generic errors
    #[error("Internal error")]
    InternalError,

    #[error("Operation cancelled")]
    Cancelled,

    #[error("Feature not available: {feature}")]
    FeatureNotAvailable { feature: String },
}

impl AIIntegrationError {
    /// Check if the error is recoverable through retry
    pub fn is_recoverable(&self) -> bool {
        match self {
            // Permanent failures
            Self::ModelNotFound
            | Self::InsufficientMemory { .. }
            | Self::APIKeyMissing { .. }
            | Self::PrivacyViolation
            | Self::PIIDetected
            | Self::PromptInjectionDetected
            | Self::InvalidConfig { .. }
            | Self::FeatureNotAvailable { .. }
            | Self::CostLimitExceeded => false,

            // Temporary failures that might succeed on retry
            Self::ModelLoadFailed { .. }
            | Self::InferenceFailed
            | Self::GenerationTimeout { .. }
            | Self::APIRateLimited { .. }
            | Self::APIError { .. }
            | Self::APITimeout
            | Self::NetworkError
            | Self::ResourceUnavailable => true,

            // Context-dependent
            Self::ContextTooLong { .. } => false, // Need different approach
            Self::InvalidOutput => true, // Might work with different prompt
            Self::ConsentRequired => false, // Need user action

            // Processing errors might be recoverable with different approach
            Self::ContextAnalysisFailed
            | Self::ContextCompressionFailed
            | Self::PersonalityApplicationFailed => true,

            // Template and validation errors
            Self::TemplateNotFound => false, // Need different template
            Self::SuggestionValidationFailed => true,

            // System errors
            Self::NotInitialized => false, // Need initialization
            Self::FileSystemError => true,
            Self::SerializationError => true,

            // Generic
            Self::InternalError => true,
            Self::Cancelled => false,
            Self::InvalidInterventionType => false,
            Self::GPUNotAvailable => false,
        }
    }

    /// Get the severity level of the error
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            // Critical errors that prevent core functionality
            Self::ModelNotFound
            | Self::NotInitialized
            | Self::InvalidConfig { .. } => ErrorSeverity::Critical,

            // High severity errors that impact user experience
            Self::ModelLoadFailed { .. }
            | Self::InsufficientMemory { .. }
            | Self::PrivacyViolation
            | Self::PIIDetected
            | Self::PromptInjectionDetected
            | Self::CostLimitExceeded => ErrorSeverity::High,

            // Medium severity errors with fallback options
            Self::InferenceFailed
            | Self::APIKeyMissing { .. }
            | Self::APIError { .. }
            | Self::ContextTooLong { .. }
            | Self::ContextAnalysisFailed => ErrorSeverity::Medium,

            // Low severity errors that can be handled gracefully
            Self::GenerationTimeout { .. }
            | Self::APIRateLimited { .. }
            | Self::APITimeout
            | Self::TemplateNotFound
            | Self::InvalidOutput => ErrorSeverity::Low,

            // Informational errors
            Self::ConsentRequired
            | Self::FeatureNotAvailable { .. }
            | Self::Cancelled => ErrorSeverity::Info,

            // All others default to medium
            _ => ErrorSeverity::Medium,
        }
    }

    /// Get user-friendly error message that doesn't leak sensitive info
    pub fn user_message(&self) -> String {
        match self {
            Self::ModelNotFound => "AI model not available. Using template responses.".to_string(),
            Self::InsufficientMemory { .. } => "Not enough memory for AI model. Using template responses.".to_string(),
            Self::APIKeyMissing { .. } => "AI service not configured. Using local responses.".to_string(),
            Self::PrivacyViolation | Self::PIIDetected => "Cannot process request due to privacy settings.".to_string(),
            Self::ConsentRequired => "Permission needed to use cloud AI services.".to_string(),
            Self::CostLimitExceeded => "Monthly AI usage limit reached.".to_string(),
            Self::APIRateLimited { .. } => "AI service temporarily busy. Please try again.".to_string(),
            Self::GenerationTimeout { .. } => "AI response took too long. Using quick response.".to_string(),
            Self::ContextTooLong { .. } => "Request too complex. Using simplified response.".to_string(),
            _ => "Using backup response method.".to_string(),
        }
    }
}

/// Error severity levels for logging and handling decisions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorSeverity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

// Conversion from other error types
impl From<reqwest::Error> for AIIntegrationError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            Self::APITimeout
        } else if err.is_connect() {
            Self::NetworkError
        } else {
            Self::NetworkError
        }
    }
}

impl From<serde_json::Error> for AIIntegrationError {
    fn from(_: serde_json::Error) -> Self {
        Self::SerializationError
    }
}

impl From<std::io::Error> for AIIntegrationError {
    fn from(_: std::io::Error) -> Self {
        Self::FileSystemError
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_recoverability() {
        assert!(!AIIntegrationError::ModelNotFound.is_recoverable());
        assert!(AIIntegrationError::NetworkError.is_recoverable());
        assert!(!AIIntegrationError::PrivacyViolation.is_recoverable());
    }

    #[test]
    fn test_error_severity() {
        assert_eq!(AIIntegrationError::NotInitialized.severity(), ErrorSeverity::Critical);
        assert_eq!(AIIntegrationError::PrivacyViolation.severity(), ErrorSeverity::High);
        assert_eq!(AIIntegrationError::ConsentRequired.severity(), ErrorSeverity::Info);
    }

    #[test]
    fn test_user_messages_no_sensitive_info() {
        let errors = vec![
            AIIntegrationError::ModelNotFound,
            AIIntegrationError::PrivacyViolation,
            AIIntegrationError::APIKeyMissing { service: "openai".to_string() },
        ];

        for error in errors {
            let msg = error.user_message();
            // Ensure no sensitive information is leaked
            assert!(!msg.contains("key"));
            assert!(!msg.contains("token"));
            assert!(!msg.contains("secret"));
            assert!(!msg.contains("credential"));
        }
    }
}