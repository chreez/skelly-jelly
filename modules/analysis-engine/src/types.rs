//! Core type definitions for the analysis engine

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::SystemTime;
use uuid::Uuid;

/// Feature vector containing all extracted behavioral features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureVector {
    /// Keystroke timing and pattern features (10 dimensions)
    pub keystroke_features: [f32; 10],
    
    /// Mouse movement and click features (8 dimensions)
    pub mouse_features: [f32; 8],
    
    /// Window switching and focus features (6 dimensions)
    pub window_features: [f32; 6],
    
    /// Screenshot-derived context features (12 dimensions, optional)
    pub screenshot_features: Option<[f32; 12]>,
    
    /// Temporal pattern features (5 dimensions)
    pub temporal_features: [f32; 5],
    
    /// System resource usage features (4 dimensions)
    pub resource_features: [f32; 4],
}

impl Default for FeatureVector {
    fn default() -> Self {
        Self {
            keystroke_features: [0.0; 10],
            mouse_features: [0.0; 8],
            window_features: [0.0; 6],
            screenshot_features: None,
            temporal_features: [0.0; 5],
            resource_features: [0.0; 4],
        }
    }
}

impl FeatureVector {
    /// Get the total number of features
    pub fn feature_count(&self) -> usize {
        let base = 10 + 8 + 6 + 5 + 4; // 33
        if self.screenshot_features.is_some() {
            base + 12 // 45 total
        } else {
            base
        }
    }

    /// Convert to a flat vector for ML models
    pub fn to_vec(&self) -> Vec<f32> {
        let mut features = Vec::with_capacity(45);
        
        features.extend_from_slice(&self.keystroke_features);
        features.extend_from_slice(&self.mouse_features);
        features.extend_from_slice(&self.window_features);
        features.extend_from_slice(&self.temporal_features);
        features.extend_from_slice(&self.resource_features);
        
        if let Some(screenshot_features) = &self.screenshot_features {
            features.extend_from_slice(screenshot_features);
        } else {
            // Fill with zeros if no screenshot features
            features.extend_from_slice(&[0.0; 12]);
        }
        
        features
    }

    /// Validate feature vector values
    pub fn validate(&self) -> bool {
        let all_features = self.to_vec();
        all_features.iter().all(|&f| f.is_finite() && !f.is_nan())
    }

    /// Normalize features to [0, 1] range
    pub fn normalize(&mut self) {
        // Apply min-max normalization to each feature group
        Self::normalize_array(&mut self.keystroke_features);
        Self::normalize_array(&mut self.mouse_features);
        Self::normalize_array(&mut self.window_features);
        Self::normalize_array(&mut self.temporal_features);
        Self::normalize_array(&mut self.resource_features);
        
        if let Some(ref mut screenshot_features) = self.screenshot_features {
            Self::normalize_array(screenshot_features);
        }
    }

    fn normalize_array<const N: usize>(arr: &mut [f32; N]) {
        if arr.is_empty() { return; }
        
        let min = arr.iter().fold(f32::INFINITY, |a, &b| a.min(b));
        let max = arr.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
        
        if (max - min).abs() < f32::EPSILON {
            // All values are the same, set to 0.5
            for item in arr.iter_mut() {
                *item = 0.5;
            }
        } else {
            for item in arr.iter_mut() {
                *item = (*item - min) / (max - min);
            }
        }
    }
}

/// Flow state depth levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FlowDepth {
    /// Light focus, easily interrupted
    Shallow,
    /// Moderate focus, some interruption resistance
    Medium,
    /// Deep focus, high interruption resistance
    Deep,
    /// Ultra-deep focus, complete task absorption
    UltraDeep,
}

impl FlowDepth {
    pub fn from_score(score: f32) -> Self {
        match score {
            s if s >= 0.9 => FlowDepth::UltraDeep,
            s if s >= 0.7 => FlowDepth::Deep,
            s if s >= 0.5 => FlowDepth::Medium,
            _ => FlowDepth::Shallow,
        }
    }

    pub fn score(&self) -> f32 {
        match self {
            FlowDepth::Shallow => 0.25,
            FlowDepth::Medium => 0.5,
            FlowDepth::Deep => 0.75,
            FlowDepth::UltraDeep => 0.95,
        }
    }
}

/// Types of distractions detected
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DistractionType {
    /// Task switching between applications
    TaskSwitching,
    /// Social media or entertainment
    SocialMedia,
    /// Communication interruptions
    Communication,
    /// Environmental disruptions
    Environmental,
    /// Internal mind wandering
    Internal,
    /// Information seeking (research rabbit holes)
    Research,
    /// Unknown distraction type
    Unknown,
}

impl DistractionType {
    pub fn severity(&self) -> f32 {
        match self {
            DistractionType::SocialMedia => 0.9,
            DistractionType::Communication => 0.7,
            DistractionType::TaskSwitching => 0.6,
            DistractionType::Research => 0.5,
            DistractionType::Environmental => 0.4,
            DistractionType::Internal => 0.3,
            DistractionType::Unknown => 0.5,
        }
    }
}

/// Result of analysis processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    /// Unique identifier for this analysis window
    pub window_id: Uuid,
    
    /// Timestamp when analysis was completed
    pub timestamp: SystemTime,
    
    /// Detected ADHD state
    pub state: crate::models::ADHDState,
    
    /// Overall confidence in the classification
    pub confidence: f32,
    
    /// Computed behavioral metrics
    pub metrics: crate::metrics::BehavioralMetrics,
    
    /// Work context from screenshot analysis
    pub work_context: Option<crate::screenshot::WorkContext>,
    
    /// How receptive the user is to interventions right now
    pub intervention_readiness: f32,
    
    /// Processing time for this analysis (ms)
    pub processing_time_ms: u32,
    
    /// Feature importance scores for explainability
    pub feature_importance: Vec<(String, f32)>,
}

impl AnalysisResult {
    /// Create a new analysis result
    pub fn new(window_id: Uuid, state: crate::models::ADHDState) -> Self {
        Self {
            window_id,
            timestamp: SystemTime::now(),
            state,
            confidence: 0.0,
            metrics: crate::metrics::BehavioralMetrics::default(),
            work_context: None,
            intervention_readiness: 0.5,
            processing_time_ms: 0,
            feature_importance: Vec::new(),
        }
    }
}

/// Features extracted from keystroke events
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct KeystrokeFeatures {
    // Timing features
    pub mean_iki: f32,              // Mean inter-keystroke interval
    pub iki_variance: f32,          // Variance in timing
    pub iki_cv: f32,               // Coefficient of variation
    
    // Rhythm features
    pub typing_rhythm_score: f32,   // Consistency of typing rhythm
    pub pause_frequency: f32,       // Frequency of typing pauses
    
    // Burst features
    pub burst_count: u32,           // Number of typing bursts
    pub mean_burst_length: f32,     // Average burst length
    pub burst_intensity: f32,       // Intensity of bursts
    
    // Error patterns
    pub backspace_rate: f32,        // Rate of corrections
    pub correction_patterns: f32,   // Pattern strength in corrections
}

/// Features extracted from mouse events
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MouseFeatures {
    // Movement features
    pub mean_velocity: f32,         // Average movement velocity
    pub velocity_variance: f32,     // Variance in velocity
    pub movement_smoothness: f32,   // Smoothness of movements
    
    // Click features
    pub click_frequency: f32,       // Clicks per second
    pub double_click_ratio: f32,    // Ratio of double clicks
    pub click_accuracy: f32,        // Accuracy of clicks
    
    // Pattern features
    pub movement_patterns: f32,     // Regularity of movement patterns
    pub idle_time_ratio: f32,       // Ratio of idle time
}

/// Features extracted from window events
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WindowFeatures {
    // Focus features
    pub focus_duration_mean: f32,   // Average focus duration
    pub focus_duration_std: f32,    // Standard deviation of focus
    pub focus_stability: f32,       // Stability of focus
    
    // Switching features
    pub switch_frequency: f32,      // Window switches per minute
    pub rapid_switch_ratio: f32,    // Ratio of rapid switches
    pub context_coherence: f32,     // Coherence between windows
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_vector_validation() {
        let mut fv = FeatureVector::default();
        assert!(fv.validate());
        
        fv.keystroke_features[0] = f32::NAN;
        assert!(!fv.validate());
    }

    #[test]
    fn test_feature_vector_normalization() {
        let mut fv = FeatureVector {
            keystroke_features: [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0],
            ..Default::default()
        };
        
        fv.normalize();
        
        // Check that values are now in [0, 1] range
        for &val in &fv.keystroke_features {
            assert!(val >= 0.0 && val <= 1.0);
        }
    }

    #[test]
    fn test_flow_depth_scoring() {
        assert_eq!(FlowDepth::from_score(0.95), FlowDepth::UltraDeep);
        assert_eq!(FlowDepth::from_score(0.75), FlowDepth::Deep);
        assert_eq!(FlowDepth::from_score(0.5), FlowDepth::Medium);
        assert_eq!(FlowDepth::from_score(0.2), FlowDepth::Shallow);
    }

    #[test]
    fn test_distraction_severity() {
        assert!(DistractionType::SocialMedia.severity() > DistractionType::Internal.severity());
        assert!(DistractionType::Communication.severity() > DistractionType::Environmental.severity());
    }
}