//! Feature extraction pipeline for behavioral analysis

pub mod keystroke;
pub mod mouse;
pub mod window;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{
    error::{AnalysisError, AnalysisResult},
    sliding_window::AnalysisWindow,
    types::{FeatureVector, KeystrokeFeatures, MouseFeatures, WindowFeatures},
};

pub use keystroke::KeystrokeFeatureExtractor;
pub use mouse::MouseFeatureExtractor;
pub use window::WindowFeatureExtractor;

/// Trait for extracting features from event data
#[async_trait]
pub trait FeatureExtractor: Send + Sync {
    /// Extract features from events in a window
    async fn extract(&self, window: &AnalysisWindow) -> AnalysisResult<Vec<f32>>;
    
    /// Get the number of features this extractor produces
    fn feature_count(&self) -> usize;
    
    /// Get feature names for explainability
    fn feature_names(&self) -> Vec<String>;
    
    /// Get the extractor type name
    fn extractor_type(&self) -> &'static str;
}

/// Main feature extraction coordinator
pub struct FeatureExtractionPipeline {
    keystroke_extractor: KeystrokeFeatureExtractor,
    mouse_extractor: MouseFeatureExtractor,
    window_extractor: WindowFeatureExtractor,
}

impl FeatureExtractionPipeline {
    /// Create a new feature extraction pipeline
    pub fn new() -> Self {
        Self {
            keystroke_extractor: KeystrokeFeatureExtractor::new(),
            mouse_extractor: MouseFeatureExtractor::new(),
            window_extractor: WindowFeatureExtractor::new(),
        }
    }

    /// Extract all features from a window
    pub async fn extract_all_features(&self, window: &AnalysisWindow) -> AnalysisResult<FeatureVector> {
        // Extract features in parallel
        let (keystroke_result, mouse_result, window_result) = tokio::join!(
            self.keystroke_extractor.extract(window),
            self.mouse_extractor.extract(window),
            self.window_extractor.extract(window)
        );

        let keystroke_features = keystroke_result?;
        let mouse_features = mouse_result?;
        let window_features = window_result?;

        // Extract temporal and resource features
        let temporal_features = self.extract_temporal_features(window)?;
        let resource_features = self.extract_resource_features(window)?;

        // Construct feature vector
        let mut feature_vector = FeatureVector::default();
        
        // Copy features to arrays (ensure we have the right number of features)
        if keystroke_features.len() >= 10 {
            feature_vector.keystroke_features.copy_from_slice(&keystroke_features[..10]);
        }
        
        if mouse_features.len() >= 8 {
            feature_vector.mouse_features.copy_from_slice(&mouse_features[..8]);
        }
        
        if window_features.len() >= 6 {
            feature_vector.window_features.copy_from_slice(&window_features[..6]);
        }
        
        if temporal_features.len() >= 5 {
            feature_vector.temporal_features.copy_from_slice(&temporal_features[..5]);
        }
        
        if resource_features.len() >= 4 {
            feature_vector.resource_features.copy_from_slice(&resource_features[..4]);
        }

        // Validate and normalize
        if !feature_vector.validate() {
            return Err(AnalysisError::InvalidFeatureVector {
                reason: "Feature vector contains invalid values".to_string(),
            });
        }

        feature_vector.normalize();
        
        Ok(feature_vector)
    }

    /// Extract temporal pattern features
    fn extract_temporal_features(&self, window: &AnalysisWindow) -> AnalysisResult<Vec<f32>> {
        let events = &window.events;
        
        if events.is_empty() {
            return Ok(vec![0.0; 5]);
        }

        let mut features = Vec::with_capacity(5);
        
        // Feature 1: Event density (events per second)
        let duration_secs = window.duration().as_secs_f32().max(1.0);
        let event_density = events.len() as f32 / duration_secs;
        features.push(event_density);

        // Feature 2: Activity rhythm consistency
        let activity_buckets = self.calculate_activity_buckets(window);
        let rhythm_consistency = self.calculate_rhythm_consistency(&activity_buckets);
        features.push(rhythm_consistency);

        // Feature 3: Peak activity intensity
        let peak_intensity = activity_buckets.iter()
            .map(|&count| count as f32)
            .fold(0.0f32, |a, b| a.max(b));
        features.push(peak_intensity / events.len() as f32);

        // Feature 4: Activity variance
        let mean_activity = activity_buckets.iter().sum::<usize>() as f32 / activity_buckets.len() as f32;
        let variance = activity_buckets.iter()
            .map(|&count| (count as f32 - mean_activity).powi(2))
            .sum::<f32>() / activity_buckets.len() as f32;
        features.push(variance.sqrt() / mean_activity.max(1.0));

        // Feature 5: Burst pattern score
        let burst_score = self.calculate_burst_pattern_score(window);
        features.push(burst_score);

        Ok(features)
    }

    /// Extract system resource usage features
    fn extract_resource_features(&self, window: &AnalysisWindow) -> AnalysisResult<Vec<f32>> {
        let resource_events = window.get_resource_events();
        
        if resource_events.is_empty() {
            return Ok(vec![0.0; 4]);
        }

        let mut features = Vec::with_capacity(4);

        // Feature 1: Average CPU usage
        let avg_cpu = resource_events.iter()
            .map(|e| e.cpu_percent)
            .sum::<f32>() / resource_events.len() as f32;
        features.push(avg_cpu / 100.0); // Normalize to 0-1

        // Feature 2: Average memory usage (normalized)
        let avg_memory = resource_events.iter()
            .map(|e| e.memory_mb)
            .sum::<u32>() as f32 / resource_events.len() as f32;
        features.push((avg_memory / 8192.0).min(1.0)); // Normalize assuming 8GB typical

        // Feature 3: Disk I/O intensity
        let avg_disk_io = resource_events.iter()
            .map(|e| e.disk_io_mb_per_sec)
            .sum::<f32>() / resource_events.len() as f32;
        features.push((avg_disk_io / 100.0).min(1.0)); // Normalize assuming 100MB/s max

        // Feature 4: Network I/O intensity
        let avg_network_io = resource_events.iter()
            .map(|e| e.network_io_mb_per_sec)
            .sum::<f32>() / resource_events.len() as f32;
        features.push((avg_network_io / 50.0).min(1.0)); // Normalize assuming 50MB/s max

        Ok(features)
    }

    /// Calculate activity in time buckets for rhythm analysis
    fn calculate_activity_buckets(&self, window: &AnalysisWindow) -> Vec<usize> {
        const BUCKET_COUNT: usize = 10;
        let mut buckets = vec![0; BUCKET_COUNT];
        
        let window_duration = window.duration().as_secs_f64();
        if window_duration <= 0.0 {
            return buckets;
        }

        let bucket_size = window_duration / BUCKET_COUNT as f64;

        for event in &window.events {
            let event_time = event.timestamp();
            let window_start = chrono::DateTime::<chrono::Utc>::from(window.start_time);
            
            if let Ok(time_since_start) = event_time.signed_duration_since(window_start).to_std() {
                let bucket_index = ((time_since_start.as_secs_f64() / bucket_size) as usize)
                    .min(BUCKET_COUNT - 1);
                buckets[bucket_index] += 1;
            }
        }

        buckets
    }

    /// Calculate rhythm consistency from activity buckets
    fn calculate_rhythm_consistency(&self, buckets: &[usize]) -> f32 {
        if buckets.len() < 2 {
            return 0.0;
        }

        let mean = buckets.iter().sum::<usize>() as f32 / buckets.len() as f32;
        if mean == 0.0 {
            return 0.0;
        }

        let variance = buckets.iter()
            .map(|&count| (count as f32 - mean).powi(2))
            .sum::<f32>() / buckets.len() as f32;
        
        let cv = variance.sqrt() / mean;
        
        // Convert coefficient of variation to consistency score
        (1.0 - cv.min(1.0)).max(0.0)
    }

    /// Calculate burst pattern score
    fn calculate_burst_pattern_score(&self, window: &AnalysisWindow) -> f32 {
        let events = &window.events;
        if events.len() < 5 {
            return 0.0;
        }

        // Define burst as rapid succession of events
        let mut burst_count = 0;
        let mut in_burst = false;
        let mut consecutive_rapid = 0;

        for i in 1..events.len() {
            let time_diff = events[i].timestamp()
                .signed_duration_since(events[i-1].timestamp())
                .num_milliseconds();

            if time_diff < 500 { // Less than 500ms = rapid
                consecutive_rapid += 1;
                if consecutive_rapid >= 3 && !in_burst {
                    burst_count += 1;
                    in_burst = true;
                }
            } else {
                consecutive_rapid = 0;
                in_burst = false;
            }
        }

        // Normalize burst count by total possible bursts
        let max_possible_bursts = events.len() / 5;
        if max_possible_bursts > 0 {
            (burst_count as f32 / max_possible_bursts as f32).min(1.0)
        } else {
            0.0
        }
    }

    /// Get all feature names for explainability
    pub fn get_all_feature_names(&self) -> Vec<String> {
        let mut names = Vec::new();
        
        names.extend(self.keystroke_extractor.feature_names());
        names.extend(self.mouse_extractor.feature_names());
        names.extend(self.window_extractor.feature_names());
        
        // Add temporal feature names
        names.extend(vec![
            "temporal_event_density".to_string(),
            "temporal_rhythm_consistency".to_string(),
            "temporal_peak_intensity".to_string(),
            "temporal_activity_variance".to_string(),
            "temporal_burst_score".to_string(),
        ]);
        
        // Add resource feature names
        names.extend(vec![
            "resource_cpu_usage".to_string(),
            "resource_memory_usage".to_string(),
            "resource_disk_io".to_string(),
            "resource_network_io".to_string(),
        ]);
        
        names
    }

    /// Get total feature count
    pub fn total_feature_count(&self) -> usize {
        self.keystroke_extractor.feature_count() +
        self.mouse_extractor.feature_count() +
        self.window_extractor.feature_count() +
        5 + // temporal features
        4   // resource features
    }
}

impl Default for FeatureExtractionPipeline {
    fn default() -> Self {
        Self::new()
    }
}

/// Feature extraction configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureExtractionConfig {
    pub enable_keystroke_features: bool,
    pub enable_mouse_features: bool,
    pub enable_window_features: bool,
    pub enable_temporal_features: bool,
    pub enable_resource_features: bool,
    pub normalize_features: bool,
    pub feature_cache_size: usize,
}

impl Default for FeatureExtractionConfig {
    fn default() -> Self {
        Self {
            enable_keystroke_features: true,
            enable_mouse_features: true,
            enable_window_features: true,
            enable_temporal_features: true,
            enable_resource_features: true,
            normalize_features: true,
            feature_cache_size: 100,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sliding_window::AnalysisWindow;
    use chrono::Utc;
    use skelly_jelly_storage::types::*;
    use std::time::SystemTime;

    #[tokio::test]
    async fn test_feature_extraction_pipeline() {
        let pipeline = FeatureExtractionPipeline::new();
        let window = create_test_window();
        
        let result = pipeline.extract_all_features(&window).await;
        assert!(result.is_ok());
        
        let features = result.unwrap();
        assert!(features.validate());
        assert!(features.feature_count() > 0);
    }

    #[test]
    fn test_temporal_features() {
        let pipeline = FeatureExtractionPipeline::new();
        let window = create_test_window();
        
        let result = pipeline.extract_temporal_features(&window);
        assert!(result.is_ok());
        
        let features = result.unwrap();
        assert_eq!(features.len(), 5);
        assert!(features.iter().all(|&f| f.is_finite()));
    }

    #[test]
    fn test_activity_buckets() {
        let pipeline = FeatureExtractionPipeline::new();
        let window = create_test_window();
        
        let buckets = pipeline.calculate_activity_buckets(&window);
        assert_eq!(buckets.len(), 10);
    }

    fn create_test_window() -> AnalysisWindow {
        let mut window = AnalysisWindow::new(SystemTime::now());
        
        // Add some test events
        for i in 0..20 {
            let event = RawEvent::Keystroke(KeystrokeEvent {
                timestamp: Utc::now(),
                key_code: 65 + (i % 26),
                modifiers: KeyModifiers::default(),
                inter_key_interval_ms: Some(100 + i * 10),
            });
            window.add_event(event);
        }
        
        window
    }
}