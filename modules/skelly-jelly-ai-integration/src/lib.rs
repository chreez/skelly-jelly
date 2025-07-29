//! AI Integration Module
//!
//! Provides privacy-first AI assistance with local LLM support and secure API fallback.
//! 
//! ## Key Features
//! - **Privacy First**: Local processing by default, opt-in API usage with sanitization
//! - **Security Focused**: PII detection, prompt sanitization, secure key storage
//! - **Personality Consistent**: Maintains Skelly's chill, supportive skeleton companion character
//! - **Context Aware**: Analyzes work context for relevant, helpful suggestions
//! - **Performance Optimized**: Efficient local inference with intelligent caching
//!
//! ## Usage
//! ```rust
//! use ai_integration::{AIIntegration, AIIntegrationConfig};
//! 
//! let config = AIIntegrationConfig::default();
//! let ai = AIIntegration::new(config).await?;
//! 
//! // Process intervention request from gamification module
//! let response = ai.process_intervention(intervention_request).await?;
//! ```

pub mod ai_integration;
pub mod anti_patronization;
pub mod config;
pub mod context;
pub mod context_detection;
pub mod contextual_interventions;
pub mod contextual_messaging;
pub mod error;
pub mod intervention_timing;
pub mod llm;
pub mod personality;
pub mod personality_enhanced;
pub mod personality_integration;
pub mod personality_testing;
pub mod personality_visual_bridge;
pub mod privacy;
pub mod suggestions;
pub mod types;
pub mod user_feedback;

pub use ai_integration::AIIntegrationImpl;
pub use types::AIIntegration;
pub use config::{AIIntegrationConfig, LocalModelSettings, APIConfig, PrivacySettings};
pub use error::{AIIntegrationError, Result};
pub use types::*;

// Export new contextual intervention components
pub use context_detection::{WorkTypeDetector, WorkType, WorkContext, DocumentType, DesignType};
pub use intervention_timing::{
    InterventionTimingEngine, FocusState, InterventionType, InterventionDecision,
    InterventionPreferences, InterventionStats, UserResponse
};
pub use contextual_messaging::{
    ContextualMessageGenerator, ContextualMessage, MessageTone, MessagePersonalization
};
pub use user_feedback::{
    FeedbackCollector, FeedbackSubmission, FeedbackType, FeedbackAnalytics,
    PersonalizationRecommendations, FeedbackTrends
};
pub use contextual_interventions::{
    ContextualInterventionSystem, ContextualInterventionConfig, InterventionContext,
    ContextualInterventionResponse, ContextualInterventionAnalytics
};

// Re-export event bus types for convenience
pub use skelly_jelly_event_bus::message::{
    InterventionRequest, InterventionResponse, AnimationCommand,
    MessagePayload, BusMessage, ModuleId
};

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_module_compilation() {
        // Ensure the module compiles correctly
        assert!(true);
    }
}