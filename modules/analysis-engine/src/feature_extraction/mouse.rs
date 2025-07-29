//! Mouse feature extraction for behavioral analysis

use async_trait::async_trait;
use std::f32::consts::PI;

use crate::{
    error::{AnalysisError, AnalysisResult},
    feature_extraction::FeatureExtractor,
    sliding_window::AnalysisWindow,
};

/// Extracts behavioral features from mouse events
pub struct MouseFeatureExtractor {
    /// Configuration for feature extraction
    config: MouseConfig,
}

impl MouseFeatureExtractor {
    /// Create a new mouse feature extractor
    pub fn new() -> Self {
        Self {
            config: MouseConfig::default(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: MouseConfig) -> Self {
        Self { config }
    }

    /// Extract mouse movement features
    fn extract_movement_features(&self, window: &AnalysisWindow) -> AnalysisResult<[f32; 3]> {
        let mouse_events = window.get_mouse_move_events();
        
        if mouse_events.is_empty() {
            return Ok([0.0; 3]);
        }

        // Calculate velocity statistics
        let velocities: Vec<f32> = mouse_events.iter().map(|e| e.velocity).collect();
        let mean_velocity = velocities.iter().sum::<f32>() / velocities.len() as f32;
        
        let velocity_variance = velocities.iter()
            .map(|&v| (v - mean_velocity).powi(2))
            .sum::<f32>() / velocities.len() as f32;

        // Calculate movement smoothness using jerk (change in acceleration)
        let smoothness = self.calculate_movement_smoothness(window)?;

        Ok([
            (mean_velocity / 2000.0).min(1.0), // Normalize velocity (max 2000 px/s)
            (velocity_variance.sqrt() / 1000.0).min(1.0), // Normalize velocity variance
            smoothness,
        ])
    }

    /// Extract mouse click features
    fn extract_click_features(&self, window: &AnalysisWindow) -> AnalysisResult<[f32; 3]> {
        let click_events = window.get_mouse_click_events();
        let duration_secs = window.duration().as_secs_f32();
        
        if click_events.is_empty() || duration_secs == 0.0 {
            return Ok([0.0; 3]);
        }

        // Click frequency
        let click_frequency = click_events.len() as f32 / duration_secs;

        // Double click ratio
        let double_clicks = click_events.iter()
            .filter(|e| matches!(e.click_type, skelly_jelly_storage::types::ClickType::Double))
            .count();
        let double_click_ratio = double_clicks as f32 / click_events.len() as f32;

        // Click accuracy (based on click patterns and timing)
        let click_accuracy = self.calculate_click_accuracy(window)?;

        Ok([
            (click_frequency / 5.0).min(1.0), // Normalize (max 5 clicks/sec)
            double_click_ratio,
            click_accuracy,
        ])
    }

    /// Extract mouse behavior patterns
    fn extract_pattern_features(&self, window: &AnalysisWindow) -> AnalysisResult<[f32; 2]> {
        let mouse_events = window.get_mouse_move_events();
        
        if mouse_events.is_empty() {
            return Ok([0.0; 2]);
        }

        // Movement pattern regularity
        let movement_patterns = self.calculate_movement_regularity(window)?;

        // Idle time ratio
        let idle_ratio = self.calculate_idle_time_ratio(window)?;

        Ok([movement_patterns, idle_ratio])
    }

    /// Calculate movement smoothness using derivative analysis
    fn calculate_movement_smoothness(&self, window: &AnalysisWindow) -> AnalysisResult<f32> {
        let mouse_events = window.get_mouse_move_events();
        
        if mouse_events.len() < 3 {
            return Ok(0.0);
        }

        // Calculate direction changes and acceleration changes
        let mut direction_changes = 0;
        let mut total_movements = 0;

        for i in 2..mouse_events.len() {
            let p1 = (mouse_events[i-2].x, mouse_events[i-2].y);
            let p2 = (mouse_events[i-1].x, mouse_events[i-1].y);
            let p3 = (mouse_events[i].x, mouse_events[i].y);

            // Calculate vectors
            let v1 = (p2.0 - p1.0, p2.1 - p1.1);
            let v2 = (p3.0 - p2.0, p3.1 - p2.1);

            // Skip if no movement
            if (v1.0 == 0 && v1.1 == 0) || (v2.0 == 0 && v2.1 == 0) {
                continue;
            }

            // Calculate angle between vectors
            let dot_product = v1.0 * v2.0 + v1.1 * v2.1;
            let mag1 = ((v1.0 * v1.0 + v1.1 * v1.1) as f32).sqrt();
            let mag2 = ((v2.0 * v2.0 + v2.1 * v2.1) as f32).sqrt();

            if mag1 > 0.0 && mag2 > 0.0 {
                let cos_angle = (dot_product as f32) / (mag1 * mag2);
                let angle = cos_angle.clamp(-1.0, 1.0).acos();

                // Significant direction change threshold (45 degrees)
                if angle > PI / 4.0 {
                    direction_changes += 1;
                }
                total_movements += 1;
            }
        }

        if total_movements > 0 {
            // Convert to smoothness score (fewer direction changes = higher smoothness)
            let change_ratio = direction_changes as f32 / total_movements as f32;
            Ok((1.0 - change_ratio).max(0.0))
        } else {
            Ok(0.0)
        }
    }

    /// Calculate click accuracy based on timing and spatial patterns
    fn calculate_click_accuracy(&self, window: &AnalysisWindow) -> AnalysisResult<f32> {
        let click_events = window.get_mouse_click_events();
        
        if click_events.len() < 2 {
            return Ok(1.0);
        }

        let mut accurate_clicks = 0;
        let mut total_clicks = 0;

        for i in 1..click_events.len() {
            let prev_click = &click_events[i-1];
            let curr_click = &click_events[i];

            // Calculate time between clicks
            let time_diff = curr_click.timestamp
                .signed_duration_since(prev_click.timestamp)
                .num_milliseconds();

            // Calculate spatial distance
            let dx = curr_click.x - prev_click.x;
            let dy = curr_click.y - prev_click.y;
            let distance = ((dx * dx + dy * dy) as f32).sqrt();

            // Consider click accurate if it's not too rapid or too far apart
            let is_accurate = if time_diff < 200 {
                // Rapid clicks should be close together (double-click scenario)
                distance < 50.0
            } else if time_diff > 5000 {
                // Long delays are normal for different targets
                true
            } else {
                // Medium delays: reasonable movement distance
                distance < 500.0
            };

            if is_accurate {
                accurate_clicks += 1;
            }
            total_clicks += 1;
        }

        if total_clicks > 0 {
            Ok(accurate_clicks as f32 / total_clicks as f32)
        } else {
            Ok(1.0)
        }
    }

    /// Calculate movement pattern regularity
    fn calculate_movement_regularity(&self, window: &AnalysisWindow) -> AnalysisResult<f32> {
        let mouse_events = window.get_mouse_move_events();
        
        if mouse_events.len() < 5 {
            return Ok(0.0);
        }

        // Analyze movement patterns using velocity changes
        let mut velocity_changes = Vec::new();
        
        for i in 1..mouse_events.len() {
            let prev_velocity = mouse_events[i-1].velocity;
            let curr_velocity = mouse_events[i].velocity;
            
            if prev_velocity > 0.0 {
                let velocity_change = (curr_velocity - prev_velocity).abs() / prev_velocity;
                velocity_changes.push(velocity_change);
            }
        }

        if velocity_changes.is_empty() {
            return Ok(0.0);
        }

        // Calculate coefficient of variation for velocity changes
        let mean_change = velocity_changes.iter().sum::<f32>() / velocity_changes.len() as f32;
        let variance = velocity_changes.iter()
            .map(|&change| (change - mean_change).powi(2))
            .sum::<f32>() / velocity_changes.len() as f32;

        let cv = if mean_change > 0.0 {
            variance.sqrt() / mean_change
        } else {
            0.0
        };

        // Convert to regularity score (lower CV = higher regularity)
        Ok((1.0 / (1.0 + cv)).max(0.0).min(1.0))
    }

    /// Calculate idle time ratio
    fn calculate_idle_time_ratio(&self, window: &AnalysisWindow) -> AnalysisResult<f32> {
        let mouse_events = window.get_mouse_move_events();
        let total_duration = window.duration().as_millis() as i64;
        
        if mouse_events.is_empty() || total_duration == 0 {
            return Ok(1.0); // All time is idle if no mouse events
        }

        let mut idle_time = 0i64;
        let window_start = chrono::DateTime::<chrono::Utc>::from(window.start_time);

        for i in 1..mouse_events.len() {
            let time_gap = mouse_events[i].timestamp
                .signed_duration_since(mouse_events[i-1].timestamp)
                .num_milliseconds();

            // Consider gaps > 2 seconds as idle time
            if time_gap > self.config.idle_threshold_ms as i64 {
                idle_time += time_gap - self.config.idle_threshold_ms as i64;
            }
        }

        // Check idle time at the beginning and end
        if let Some(first_event) = mouse_events.first() {
            let initial_idle = first_event.timestamp
                .signed_duration_since(window_start)
                .num_milliseconds();
            if initial_idle > self.config.idle_threshold_ms as i64 {
                idle_time += initial_idle - self.config.idle_threshold_ms as i64;
            }
        }

        let idle_ratio = idle_time as f32 / total_duration as f32;
        Ok(idle_ratio.clamp(0.0, 1.0))
    }
}

#[async_trait]
impl FeatureExtractor for MouseFeatureExtractor {
    async fn extract(&self, window: &AnalysisWindow) -> AnalysisResult<Vec<f32>> {
        let mouse_move_events = window.get_mouse_move_events();
        let mouse_click_events = window.get_mouse_click_events();
        
        if mouse_move_events.is_empty() && mouse_click_events.is_empty() {
            return Ok(vec![0.0; 8]);
        }

        // Extract different feature groups
        let movement_features = self.extract_movement_features(window)?;
        let click_features = self.extract_click_features(window)?;
        let pattern_features = self.extract_pattern_features(window)?;

        // Combine all features into a single vector
        let mut features = Vec::with_capacity(8);
        features.extend_from_slice(&movement_features); // 3 features
        features.extend_from_slice(&click_features);    // 3 features
        features.extend_from_slice(&pattern_features);  // 2 features

        // Validate features
        for (i, &feature) in features.iter().enumerate() {
            if !feature.is_finite() {
                return Err(AnalysisError::FeatureExtractionError {
                    feature_type: "mouse".to_string(),
                    reason: format!("Invalid feature value at index {}: {}", i, feature),
                });
            }
        }

        Ok(features)
    }

    fn feature_count(&self) -> usize {
        8
    }

    fn feature_names(&self) -> Vec<String> {
        vec![
            "mouse_mean_velocity".to_string(),
            "mouse_velocity_variance".to_string(),
            "mouse_movement_smoothness".to_string(),
            "mouse_click_frequency".to_string(),
            "mouse_double_click_ratio".to_string(),
            "mouse_click_accuracy".to_string(),
            "mouse_movement_patterns".to_string(),
            "mouse_idle_time_ratio".to_string(),
        ]
    }

    fn extractor_type(&self) -> &'static str {
        "mouse"
    }
}

impl Default for MouseFeatureExtractor {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration for mouse feature extraction
#[derive(Debug, Clone)]
pub struct MouseConfig {
    /// Threshold for considering time as idle (ms)
    pub idle_threshold_ms: u32,
    /// Enable advanced pattern analysis
    pub enable_pattern_analysis: bool,
    /// Minimum movement distance for analysis
    pub min_movement_distance: f32,
}

impl Default for MouseConfig {
    fn default() -> Self {
        Self {
            idle_threshold_ms: 2000,
            enable_pattern_analysis: true,
            min_movement_distance: 5.0,
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
    async fn test_mouse_feature_extraction() {
        let extractor = MouseFeatureExtractor::new();
        let window = create_test_window_with_mouse_events();
        
        let result = extractor.extract(&window).await;
        assert!(result.is_ok());
        
        let features = result.unwrap();
        assert_eq!(features.len(), 8);
        assert!(features.iter().all(|&f| f.is_finite()));
    }

    #[test]
    fn test_movement_features() {
        let extractor = MouseFeatureExtractor::new();
        let window = create_test_window_with_mouse_events();
        
        let result = extractor.extract_movement_features(&window);
        assert!(result.is_ok());
        
        let features = result.unwrap();
        assert_eq!(features.len(), 3);
        assert!(features.iter().all(|&f| f.is_finite()));
    }

    #[test]
    fn test_click_features() {
        let extractor = MouseFeatureExtractor::new();
        let window = create_test_window_with_clicks();
        
        let result = extractor.extract_click_features(&window);
        assert!(result.is_ok());
        
        let features = result.unwrap();
        assert_eq!(features.len(), 3);
        assert!(features.iter().all(|&f| f.is_finite()));
    }

    #[test]
    fn test_empty_window() {
        let extractor = MouseFeatureExtractor::new();
        let window = AnalysisWindow::new(SystemTime::now());
        
        let result = extractor.extract_movement_features(&window);
        assert!(result.is_ok());
        
        let features = result.unwrap();
        assert_eq!(features, [0.0; 3]);
    }

    #[test]
    fn test_movement_smoothness() {
        let extractor = MouseFeatureExtractor::new();
        let window = create_smooth_movement_window();
        
        let smoothness = extractor.calculate_movement_smoothness(&window);
        assert!(smoothness.is_ok());
        assert!(smoothness.unwrap() >= 0.0);
    }

    fn create_test_window_with_mouse_events() -> AnalysisWindow {
        let mut window = AnalysisWindow::new(SystemTime::now());
        
        let base_time = Utc::now();
        
        // Add varied mouse movement events
        for i in 0..30 {
            let timestamp = base_time + chrono::Duration::milliseconds(i * 100);
            let event = RawEvent::MouseMove(MouseMoveEvent {
                timestamp,
                x: 100 + (i * 10) as i32,
                y: 100 + (i * 5) as i32,
                velocity: 200.0 + (i as f32 * 10.0),
            });
            window.add_event(event);
        }
        
        window
    }

    fn create_test_window_with_clicks() -> AnalysisWindow {
        let mut window = AnalysisWindow::new(SystemTime::now());
        
        let base_time = Utc::now();
        
        // Add mouse click events
        for i in 0..10 {
            let timestamp = base_time + chrono::Duration::milliseconds(i * 500);
            let event = RawEvent::MouseClick(MouseClickEvent {
                timestamp,
                x: 200 + (i * 20) as i32,
                y: 150 + (i * 10) as i32,
                button: MouseButton::Left,
                click_type: if i % 3 == 0 { ClickType::Double } else { ClickType::Single },
            });
            window.add_event(event);
        }
        
        window
    }

    fn create_smooth_movement_window() -> AnalysisWindow {
        let mut window = AnalysisWindow::new(SystemTime::now());
        
        let base_time = Utc::now();
        
        // Create smooth movement pattern (straight line)
        for i in 0..20 {
            let timestamp = base_time + chrono::Duration::milliseconds(i * 50);
            let event = RawEvent::MouseMove(MouseMoveEvent {
                timestamp,
                x: 100 + i as i32 * 5, // Straight line movement
                y: 100 + i as i32 * 2,
                velocity: 150.0, // Constant velocity
            });
            window.add_event(event);
        }
        
        window
    }
}