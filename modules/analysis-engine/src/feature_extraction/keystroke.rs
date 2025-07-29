//! Keystroke feature extraction for behavioral analysis

use async_trait::async_trait;
use ndarray::Array1;
use statrs::statistics::OrderStatistics;

use crate::{
    error::{AnalysisError, AnalysisResult},
    feature_extraction::FeatureExtractor,
    sliding_window::AnalysisWindow,
};

/// Extracts behavioral features from keystroke events
pub struct KeystrokeFeatureExtractor {
    /// Configuration for feature extraction
    config: KeystrokeConfig,
}

impl KeystrokeFeatureExtractor {
    /// Create a new keystroke feature extractor
    pub fn new() -> Self {
        Self {
            config: KeystrokeConfig::default(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: KeystrokeConfig) -> Self {
        Self { config }
    }

    /// Extract inter-keystroke interval statistics
    fn extract_iki_features(&self, window: &AnalysisWindow) -> AnalysisResult<[f32; 3]> {
        let keystroke_events = window.get_keystroke_events();
        
        if keystroke_events.len() < 2 {
            return Ok([0.0; 3]);
        }

        // Calculate inter-keystroke intervals
        let mut intervals = Vec::new();
        
        for i in 1..keystroke_events.len() {
            let current_time = keystroke_events[i].timestamp;
            let prev_time = keystroke_events[i-1].timestamp;
            
            let interval_ms = current_time
                .signed_duration_since(prev_time)
                .num_milliseconds();
                
            // Filter reasonable intervals (10ms to 5000ms)
            if interval_ms >= 10 && interval_ms <= 5000 {
                intervals.push(interval_ms as f32);
            }
        }

        if intervals.is_empty() {
            return Ok([0.0; 3]);
        }

        let mean_iki = intervals.iter().sum::<f32>() / intervals.len() as f32;
        
        let variance = intervals.iter()
            .map(|&iki| (iki - mean_iki).powi(2))
            .sum::<f32>() / intervals.len() as f32;
        
        let iki_cv = if mean_iki > 0.0 {
            variance.sqrt() / mean_iki
        } else {
            0.0
        };

        Ok([
            mean_iki / 1000.0,  // Normalize to seconds
            variance / 1000000.0, // Normalize variance
            iki_cv.min(2.0) / 2.0, // Normalize CV to 0-1 range
        ])
    }

    /// Extract typing rhythm and pattern features
    fn extract_rhythm_features(&self, window: &AnalysisWindow) -> AnalysisResult<[f32; 2]> {
        let keystroke_events = window.get_keystroke_events();
        
        if keystroke_events.len() < 10 {
            return Ok([0.0; 2]);
        }

        // Calculate rhythm consistency
        let rhythm_score = self.calculate_rhythm_consistency(window)?;
        
        // Calculate pause patterns
        let pause_frequency = self.calculate_pause_frequency(window)?;

        Ok([rhythm_score, pause_frequency])
    }

    /// Extract burst pattern features
    fn extract_burst_features(&self, window: &AnalysisWindow) -> AnalysisResult<[f32; 3]> {
        let keystroke_events = window.get_keystroke_events();
        
        if keystroke_events.len() < 5 {
            return Ok([0.0; 3]);
        }

        let mut bursts = Vec::new();
        let mut current_burst_length = 1;
        let mut in_burst = false;

        for i in 1..keystroke_events.len() {
            let interval = keystroke_events[i].timestamp
                .signed_duration_since(keystroke_events[i-1].timestamp)
                .num_milliseconds();

            if interval < self.config.burst_threshold_ms as i64 {
                if !in_burst {
                    in_burst = true;
                    current_burst_length = 2;
                } else {
                    current_burst_length += 1;
                }
            } else {
                if in_burst && current_burst_length >= self.config.min_burst_length {
                    bursts.push(current_burst_length);
                }
                in_burst = false;
                current_burst_length = 1;
            }
        }

        // Add final burst if active
        if in_burst && current_burst_length >= self.config.min_burst_length {
            bursts.push(current_burst_length);
        }

        let burst_count = bursts.len() as f32;
        let mean_burst_length = if !bursts.is_empty() {
            bursts.iter().sum::<usize>() as f32 / bursts.len() as f32
        } else {
            0.0
        };
        
        // Calculate burst intensity (bursts per minute)
        let duration_minutes = window.duration().as_secs_f32() / 60.0;
        let burst_intensity = if duration_minutes > 0.0 {
            burst_count / duration_minutes
        } else {
            0.0
        };

        Ok([
            (burst_count / 20.0).min(1.0), // Normalize assuming max 20 bursts
            (mean_burst_length / 50.0).min(1.0), // Normalize assuming max 50 keystroke bursts
            (burst_intensity / 10.0).min(1.0), // Normalize assuming max 10 bursts per minute
        ])
    }

    /// Extract error and correction patterns
    fn extract_error_features(&self, window: &AnalysisWindow) -> AnalysisResult<[f32; 2]> {
        let keystroke_events = window.get_keystroke_events();
        
        if keystroke_events.is_empty() {
            return Ok([0.0; 2]);
        }

        let mut backspace_count = 0;
        let mut correction_patterns = 0;
        let mut last_was_backspace = false;

        for event in &keystroke_events {
            let is_backspace = event.key_code == 8 || event.key_code == 46; // Backspace or Delete
            
            if is_backspace {
                backspace_count += 1;
                
                // Check for correction patterns (backspace followed by typing)
                if !last_was_backspace {
                    correction_patterns += 1;
                }
                last_was_backspace = true;
            } else {
                last_was_backspace = false;
            }
        }

        let backspace_rate = backspace_count as f32 / keystroke_events.len() as f32;
        let correction_pattern_score = if backspace_count > 0 {
            correction_patterns as f32 / backspace_count as f32
        } else {
            0.0
        };

        Ok([
            backspace_rate.min(0.3) / 0.3, // Normalize assuming max 30% error rate
            correction_pattern_score,
        ])
    }

    /// Calculate rhythm consistency using autocorrelation
    fn calculate_rhythm_consistency(&self, window: &AnalysisWindow) -> AnalysisResult<f32> {
        let keystroke_events = window.get_keystroke_events();
        
        if keystroke_events.len() < 10 {
            return Ok(0.0);
        }

        // Extract intervals
        let mut intervals = Vec::new();
        for i in 1..keystroke_events.len() {
            let interval = keystroke_events[i].timestamp
                .signed_duration_since(keystroke_events[i-1].timestamp)
                .num_milliseconds() as f32;
            
            if interval >= 10.0 && interval <= 2000.0 {
                intervals.push(interval);
            }
        }

        if intervals.len() < 5 {
            return Ok(0.0);
        }

        // Calculate coefficient of variation
        let mean = intervals.iter().sum::<f32>() / intervals.len() as f32;
        let variance = intervals.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f32>() / intervals.len() as f32;
        
        let cv = if mean > 0.0 {
            variance.sqrt() / mean
        } else {
            return Ok(0.0);
        };

        // Convert CV to consistency score (lower CV = higher consistency)
        Ok((1.0 / (1.0 + cv)).max(0.0).min(1.0))
    }

    /// Calculate frequency of typing pauses
    fn calculate_pause_frequency(&self, window: &AnalysisWindow) -> AnalysisResult<f32> {
        let keystroke_events = window.get_keystroke_events();
        
        if keystroke_events.len() < 5 {
            return Ok(0.0);
        }

        let mut pause_count = 0;
        
        for i in 1..keystroke_events.len() {
            let interval = keystroke_events[i].timestamp
                .signed_duration_since(keystroke_events[i-1].timestamp)
                .num_milliseconds();
            
            if interval > self.config.pause_threshold_ms as i64 {
                pause_count += 1;
            }
        }

        let duration_minutes = window.duration().as_secs_f32() / 60.0;
        if duration_minutes > 0.0 {
            Ok((pause_count as f32 / duration_minutes / 10.0).min(1.0)) // Normalize to 0-1
        } else {
            Ok(0.0)
        }
    }
}

#[async_trait]
impl FeatureExtractor for KeystrokeFeatureExtractor {
    async fn extract(&self, window: &AnalysisWindow) -> AnalysisResult<Vec<f32>> {
        let keystroke_events = window.get_keystroke_events();
        
        if keystroke_events.is_empty() {
            return Ok(vec![0.0; 10]);
        }

        // Extract different feature groups
        let iki_features = self.extract_iki_features(window)?;
        let rhythm_features = self.extract_rhythm_features(window)?;
        let burst_features = self.extract_burst_features(window)?;
        let error_features = self.extract_error_features(window)?;

        // Combine all features into a single vector
        let mut features = Vec::with_capacity(10);
        features.extend_from_slice(&iki_features);     // 3 features
        features.extend_from_slice(&rhythm_features);  // 2 features
        features.extend_from_slice(&burst_features);   // 3 features
        features.extend_from_slice(&error_features);   // 2 features

        // Validate features
        for (i, &feature) in features.iter().enumerate() {
            if !feature.is_finite() {
                return Err(AnalysisError::FeatureExtractionError {
                    feature_type: "keystroke".to_string(),
                    reason: format!("Invalid feature value at index {}: {}", i, feature),
                });
            }
        }

        Ok(features)
    }

    fn feature_count(&self) -> usize {
        10
    }

    fn feature_names(&self) -> Vec<String> {
        vec![
            "keystroke_mean_iki".to_string(),
            "keystroke_iki_variance".to_string(),
            "keystroke_iki_cv".to_string(),
            "keystroke_rhythm_score".to_string(),
            "keystroke_pause_frequency".to_string(),
            "keystroke_burst_count".to_string(),
            "keystroke_mean_burst_length".to_string(),
            "keystroke_burst_intensity".to_string(),
            "keystroke_backspace_rate".to_string(),
            "keystroke_correction_patterns".to_string(),
        ]
    }

    fn extractor_type(&self) -> &'static str {
        "keystroke"
    }
}

impl Default for KeystrokeFeatureExtractor {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration for keystroke feature extraction
#[derive(Debug, Clone)]
pub struct KeystrokeConfig {
    /// Threshold for burst detection (ms)
    pub burst_threshold_ms: u32,
    /// Minimum length for a typing burst
    pub min_burst_length: usize,
    /// Threshold for pause detection (ms)
    pub pause_threshold_ms: u32,
    /// Enable advanced rhythm analysis
    pub enable_rhythm_analysis: bool,
}

impl Default for KeystrokeConfig {
    fn default() -> Self {
        Self {
            burst_threshold_ms: 200,
            min_burst_length: 3,
            pause_threshold_ms: 1000,
            enable_rhythm_analysis: true,
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
    async fn test_keystroke_feature_extraction() {
        let extractor = KeystrokeFeatureExtractor::new();
        let window = create_test_window_with_keystrokes();
        
        let result = extractor.extract(&window).await;
        assert!(result.is_ok());
        
        let features = result.unwrap();
        assert_eq!(features.len(), 10);
        assert!(features.iter().all(|&f| f.is_finite()));
    }

    #[test]
    fn test_iki_feature_extraction() {
        let extractor = KeystrokeFeatureExtractor::new();
        let window = create_test_window_with_keystrokes();
        
        let result = extractor.extract_iki_features(&window);
        assert!(result.is_ok());
        
        let features = result.unwrap();
        assert_eq!(features.len(), 3);
        assert!(features.iter().all(|&f| f.is_finite()));
    }

    #[test]
    fn test_empty_window() {
        let extractor = KeystrokeFeatureExtractor::new();
        let window = AnalysisWindow::new(SystemTime::now());
        
        let result = extractor.extract_iki_features(&window);
        assert!(result.is_ok());
        
        let features = result.unwrap();
        assert_eq!(features, [0.0; 3]);
    }

    #[test]
    fn test_burst_detection() {
        let extractor = KeystrokeFeatureExtractor::new();
        let window = create_burst_window();
        
        let result = extractor.extract_burst_features(&window);
        assert!(result.is_ok());
        
        let features = result.unwrap();
        assert!(features[0] > 0.0); // Should detect some bursts
    }

    fn create_test_window_with_keystrokes() -> AnalysisWindow {
        let mut window = AnalysisWindow::new(SystemTime::now());
        
        let base_time = Utc::now();
        
        // Add varied keystroke events
        for i in 0..50 {
            let timestamp = base_time + chrono::Duration::milliseconds(i * 150 + (i % 5) * 50);
            let event = RawEvent::Keystroke(KeystrokeEvent {
                timestamp,
                key_code: 65 + (i % 26),
                modifiers: KeyModifiers::default(),
                inter_key_interval_ms: Some(150 + (i % 5) * 50),
            });
            window.add_event(event);
        }
        
        window
    }

    fn create_burst_window() -> AnalysisWindow {
        let mut window = AnalysisWindow::new(SystemTime::now());
        
        let base_time = Utc::now();
        
        // Create a burst pattern: rapid typing followed by pause
        for i in 0..10 {
            let timestamp = base_time + chrono::Duration::milliseconds(i * 100);
            let event = RawEvent::Keystroke(KeystrokeEvent {
                timestamp,
                key_code: 65 + i,
                modifiers: KeyModifiers::default(),
                inter_key_interval_ms: Some(100),
            });
            window.add_event(event);
        }
        
        // Add pause and another burst
        for i in 10..20 {
            let timestamp = base_time + chrono::Duration::milliseconds(i * 100 + 2000);
            let event = RawEvent::Keystroke(KeystrokeEvent {
                timestamp,
                key_code: 65 + i,
                modifiers: KeyModifiers::default(),
                inter_key_interval_ms: Some(100),
            });
            window.add_event(event);
        }
        
        window
    }
}