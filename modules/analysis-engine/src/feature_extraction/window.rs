//! Window focus feature extraction for behavioral analysis

use async_trait::async_trait;
use std::collections::HashMap;

use crate::{
    error::{AnalysisError, AnalysisResult},
    feature_extraction::FeatureExtractor,
    sliding_window::AnalysisWindow,
};

/// Extracts behavioral features from window focus events
pub struct WindowFeatureExtractor {
    /// Configuration for feature extraction
    config: WindowConfig,
}

impl WindowFeatureExtractor {
    /// Create a new window feature extractor
    pub fn new() -> Self {
        Self {
            config: WindowConfig::default(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: WindowConfig) -> Self {
        Self { config }
    }

    /// Extract focus duration features
    fn extract_focus_features(&self, window: &AnalysisWindow) -> AnalysisResult<[f32; 3]> {
        let window_events = window.get_window_focus_events();
        
        if window_events.is_empty() {
            return Ok([0.0; 3]);
        }

        // Collect focus durations
        let durations: Vec<f32> = window_events.iter()
            .filter_map(|event| event.duration_ms.map(|d| d as f32 / 1000.0)) // Convert to seconds
            .collect();

        if durations.is_empty() {
            return Ok([0.0; 3]);
        }

        // Calculate statistics
        let mean_duration = durations.iter().sum::<f32>() / durations.len() as f32;
        
        let variance = durations.iter()
            .map(|&d| (d - mean_duration).powi(2))
            .sum::<f32>() / durations.len() as f32;
        let std_duration = variance.sqrt();

        // Calculate focus stability (inverse of coefficient of variation)
        let focus_stability = if mean_duration > 0.0 {
            let cv = std_duration / mean_duration;
            (1.0 / (1.0 + cv)).max(0.0).min(1.0)
        } else {
            0.0
        };

        Ok([
            (mean_duration / 300.0).min(1.0), // Normalize to 5 minutes max
            (std_duration / 180.0).min(1.0),  // Normalize to 3 minutes max
            focus_stability,
        ])
    }

    /// Extract window switching features
    fn extract_switching_features(&self, window: &AnalysisWindow) -> AnalysisResult<[f32; 2]> {
        let window_events = window.get_window_focus_events();
        let duration_secs = window.duration().as_secs_f32();
        
        if window_events.is_empty() || duration_secs == 0.0 {
            return Ok([0.0; 2]);
        }

        // Calculate switch frequency
        let switch_count = window_events.len().saturating_sub(1); // N events = N-1 switches
        let switch_frequency = switch_count as f32 / (duration_secs / 60.0); // Switches per minute

        // Calculate rapid switch ratio
        let rapid_switches = window_events.windows(2)
            .filter(|pair| {
                if let (Some(prev_duration), Some(curr_duration)) = (pair[0].duration_ms, pair[1].duration_ms) {
                    prev_duration < self.config.rapid_switch_threshold_ms && 
                    curr_duration < self.config.rapid_switch_threshold_ms
                } else {
                    false
                }
            })
            .count();

        let rapid_switch_ratio = if switch_count > 0 {
            rapid_switches as f32 / switch_count as f32
        } else {
            0.0
        };

        Ok([
            (switch_frequency / 20.0).min(1.0), // Normalize to max 20 switches/minute
            rapid_switch_ratio,
        ])
    }

    /// Extract context coherence features
    fn extract_context_features(&self, window: &AnalysisWindow) -> AnalysisResult<[f32; 1]> {
        let window_events = window.get_window_focus_events();
        
        if window_events.len() < 2 {
            return Ok([1.0]); // Single or no windows = perfect coherence
        }

        let context_coherence = self.calculate_context_coherence(window)?;
        
        Ok([context_coherence])
    }

    /// Calculate context coherence based on application relationships
    fn calculate_context_coherence(&self, window: &AnalysisWindow) -> AnalysisResult<f32> {
        let window_events = window.get_window_focus_events();
        
        if window_events.len() < 2 {
            return Ok(1.0);
        }

        // Categorize applications and calculate coherence
        let mut app_categories = HashMap::new();
        let mut total_transitions = 0;
        let mut coherent_transitions = 0;

        // Categorize applications
        for event in &window_events {
            let category = self.categorize_application(&event.app_name);
            app_categories.insert(event.app_name.clone(), category);
        }

        // Analyze transitions
        for i in 1..window_events.len() {
            let prev_app = &window_events[i-1].app_name;
            let curr_app = &window_events[i].app_name;
            
            if prev_app != curr_app {
                total_transitions += 1;
                
                let prev_category = app_categories.get(prev_app).unwrap_or(&AppCategory::Unknown);
                let curr_category = app_categories.get(curr_app).unwrap_or(&AppCategory::Unknown);
                
                // Check if transition is coherent (related work contexts)
                if self.is_coherent_transition(prev_category, curr_category) {
                    coherent_transitions += 1;
                }
            }
        }

        if total_transitions > 0 {
            Ok(coherent_transitions as f32 / total_transitions as f32)
        } else {
            Ok(1.0)
        }
    }

    /// Categorize application by name
    fn categorize_application(&self, app_name: &str) -> AppCategory {
        let app_lower = app_name.to_lowercase();
        
        // Development tools
        if app_lower.contains("code") || app_lower.contains("xcode") || 
           app_lower.contains("intellij") || app_lower.contains("vim") ||
           app_lower.contains("emacs") || app_lower.contains("terminal") ||
           app_lower.contains("git") {
            return AppCategory::Development;
        }
        
        // Browsers
        if app_lower.contains("chrome") || app_lower.contains("firefox") ||
           app_lower.contains("safari") || app_lower.contains("edge") {
            return AppCategory::Browser;
        }
        
        // Communication
        if app_lower.contains("slack") || app_lower.contains("teams") ||
           app_lower.contains("zoom") || app_lower.contains("mail") ||
           app_lower.contains("message") || app_lower.contains("discord") {
            return AppCategory::Communication;
        }
        
        // Productivity
        if app_lower.contains("word") || app_lower.contains("excel") ||
           app_lower.contains("powerpoint") || app_lower.contains("notion") ||
           app_lower.contains("obsidian") || app_lower.contains("notes") {
            return AppCategory::Productivity;
        }
        
        // Design
        if app_lower.contains("photoshop") || app_lower.contains("illustrator") ||
           app_lower.contains("figma") || app_lower.contains("sketch") ||
           app_lower.contains("blender") {
            return AppCategory::Design;
        }
        
        // Entertainment
        if app_lower.contains("spotify") || app_lower.contains("youtube") ||
           app_lower.contains("netflix") || app_lower.contains("game") ||
           app_lower.contains("steam") {
            return AppCategory::Entertainment;
        }
        
        AppCategory::Unknown
    }

    /// Check if a transition between categories is coherent
    fn is_coherent_transition(&self, from: &AppCategory, to: &AppCategory) -> bool {
        use AppCategory::*;
        
        match (from, to) {
            // Same category is always coherent
            (a, b) if a == b => true,
            
            // Development workflow transitions
            (Development, Browser) | (Browser, Development) => true,
            (Development, Productivity) | (Productivity, Development) => true,
            
            // Creative workflow transitions
            (Design, Browser) | (Browser, Design) => true,
            (Design, Productivity) | (Productivity, Design) => true,
            
            // Communication breaks are sometimes coherent
            (Communication, Development) | (Development, Communication) => true,
            (Communication, Productivity) | (Productivity, Communication) => true,
            
            // Entertainment is usually incoherent with work
            (Entertainment, _) | (_, Entertainment) => false,
            
            // Unknown transitions are neutral
            (Unknown, _) | (_, Unknown) => true,
            
            // Default: coherent for productivity workflows
            _ => true,
        }
    }
}

#[async_trait]
impl FeatureExtractor for WindowFeatureExtractor {
    async fn extract(&self, window: &AnalysisWindow) -> AnalysisResult<Vec<f32>> {
        let window_events = window.get_window_focus_events();
        
        if window_events.is_empty() {
            return Ok(vec![0.0; 6]);
        }

        // Extract different feature groups
        let focus_features = self.extract_focus_features(window)?;
        let switching_features = self.extract_switching_features(window)?;
        let context_features = self.extract_context_features(window)?;

        // Combine all features into a single vector
        let mut features = Vec::with_capacity(6);
        features.extend_from_slice(&focus_features);    // 3 features
        features.extend_from_slice(&switching_features); // 2 features
        features.extend_from_slice(&context_features);   // 1 feature

        // Validate features
        for (i, &feature) in features.iter().enumerate() {
            if !feature.is_finite() {
                return Err(AnalysisError::FeatureExtractionError {
                    feature_type: "window".to_string(),
                    reason: format!("Invalid feature value at index {}: {}", i, feature),
                });
            }
        }

        Ok(features)
    }

    fn feature_count(&self) -> usize {
        6
    }

    fn feature_names(&self) -> Vec<String> {
        vec![
            "window_focus_duration_mean".to_string(),
            "window_focus_duration_std".to_string(),
            "window_focus_stability".to_string(),
            "window_switch_frequency".to_string(),
            "window_rapid_switch_ratio".to_string(),
            "window_context_coherence".to_string(),
        ]
    }

    fn extractor_type(&self) -> &'static str {
        "window"
    }
}

impl Default for WindowFeatureExtractor {
    fn default() -> Self {
        Self::new()
    }
}

/// Application categories for context analysis
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AppCategory {
    Development,
    Browser,
    Communication,
    Productivity,
    Design,
    Entertainment,
    Unknown,
}

/// Configuration for window feature extraction
#[derive(Debug, Clone)]
pub struct WindowConfig {
    /// Threshold for rapid switch detection (ms)
    pub rapid_switch_threshold_ms: u32,
    /// Enable context coherence analysis
    pub enable_context_analysis: bool,
    /// Minimum focus duration to consider (ms)
    pub min_focus_duration_ms: u32,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            rapid_switch_threshold_ms: 5000, // 5 seconds
            enable_context_analysis: true,
            min_focus_duration_ms: 500,      // 0.5 seconds
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
    async fn test_window_feature_extraction() {
        let extractor = WindowFeatureExtractor::new();
        let window = create_test_window_with_window_events();
        
        let result = extractor.extract(&window).await;
        assert!(result.is_ok());
        
        let features = result.unwrap();
        assert_eq!(features.len(), 6);
        assert!(features.iter().all(|&f| f.is_finite()));
    }

    #[test]
    fn test_focus_features() {
        let extractor = WindowFeatureExtractor::new();
        let window = create_test_window_with_window_events();
        
        let result = extractor.extract_focus_features(&window);
        assert!(result.is_ok());
        
        let features = result.unwrap();
        assert_eq!(features.len(), 3);
        assert!(features.iter().all(|&f| f.is_finite()));
    }

    #[test]
    fn test_switching_features() {
        let extractor = WindowFeatureExtractor::new();
        let window = create_test_window_with_rapid_switches();
        
        let result = extractor.extract_switching_features(&window);
        assert!(result.is_ok());
        
        let features = result.unwrap();
        assert_eq!(features.len(), 2);
        assert!(features[1] > 0.0); // Should detect rapid switches
    }

    #[test]
    fn test_app_categorization() {
        let extractor = WindowFeatureExtractor::new();
        
        assert_eq!(extractor.categorize_application("Visual Studio Code"), AppCategory::Development);
        assert_eq!(extractor.categorize_application("Google Chrome"), AppCategory::Browser);
        assert_eq!(extractor.categorize_application("Slack"), AppCategory::Communication);
        assert_eq!(extractor.categorize_application("Microsoft Word"), AppCategory::Productivity);
        assert_eq!(extractor.categorize_application("Adobe Photoshop"), AppCategory::Design);
        assert_eq!(extractor.categorize_application("Spotify"), AppCategory::Entertainment);
    }

    #[test]
    fn test_coherent_transitions() {
        let extractor = WindowFeatureExtractor::new();
        
        assert!(extractor.is_coherent_transition(&AppCategory::Development, &AppCategory::Browser));
        assert!(extractor.is_coherent_transition(&AppCategory::Productivity, &AppCategory::Communication));
        assert!(!extractor.is_coherent_transition(&AppCategory::Development, &AppCategory::Entertainment));
    }

    #[test]
    fn test_empty_window() {
        let extractor = WindowFeatureExtractor::new();
        let window = AnalysisWindow::new(SystemTime::now());
        
        let result = extractor.extract_focus_features(&window);
        assert!(result.is_ok());
        
        let features = result.unwrap();
        assert_eq!(features, [0.0; 3]);
    }

    fn create_test_window_with_window_events() -> AnalysisWindow {
        let mut window = AnalysisWindow::new(SystemTime::now());
        
        let base_time = Utc::now();
        let apps = vec![
            "Visual Studio Code",
            "Google Chrome", 
            "Terminal",
            "Slack",
            "Visual Studio Code",
        ];
        
        for (i, app_name) in apps.iter().enumerate() {
            let timestamp = base_time + chrono::Duration::seconds(i as i64 * 30);
            let event = RawEvent::WindowFocus(WindowFocusEvent {
                timestamp,
                window_title: format!("{} - Window {}", app_name, i),
                app_name: app_name.to_string(),
                process_id: 1000 + i as u32,
                duration_ms: Some(25000 + (i as u32 * 5000)), // Varied durations
            });
            window.add_event(event);
        }
        
        window
    }

    fn create_test_window_with_rapid_switches() -> AnalysisWindow {
        let mut window = AnalysisWindow::new(SystemTime::now());
        
        let base_time = Utc::now();
        let apps = vec!["App1", "App2", "App1", "App3", "App2"];
        
        for (i, app_name) in apps.iter().enumerate() {
            let timestamp = base_time + chrono::Duration::seconds(i as i64 * 2); // 2 second intervals
            let event = RawEvent::WindowFocus(WindowFocusEvent {
                timestamp,
                window_title: format!("{} Window", app_name),
                app_name: app_name.to_string(),
                process_id: 1000 + i as u32,
                duration_ms: Some(1500), // Short durations = rapid switches
            });
            window.add_event(event);
        }
        
        window
    }
}