//! Behavioral metrics calculation engine

// Removed unused rayon import
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

use crate::{
    error::{AnalysisError, AnalysisResult},
    sliding_window::AnalysisWindow,
};

/// Comprehensive behavioral metrics computed from event data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehavioralMetrics {
    // Activity metrics
    pub keystroke_rate: f32,            // Keystrokes per minute
    pub mouse_activity_level: f32,      // Mouse movement intensity (0-1)
    pub window_switch_frequency: f32,   // Window switches per minute
    
    // Focus metrics
    pub focus_duration: Duration,       // Longest continuous focus period
    pub focus_depth_score: f32,         // Depth of focus (0-1)
    pub distraction_frequency: f32,     // Distractions per hour
    
    // Pattern metrics
    pub work_rhythm_consistency: f32,   // Consistency of work patterns (0-1)
    pub task_switching_index: f32,      // Task switching behavior (0-1)
    pub cognitive_load_estimate: f32,   // Estimated cognitive load (0-1)
    
    // Productivity indicators
    pub productive_time_ratio: f32,     // Ratio of productive vs idle time
    pub flow_state_probability: f32,    // Likelihood of being in flow (0-1)
    pub intervention_receptivity: f32,  // Receptivity to interventions (0-1)
    
    // Error and correction patterns
    pub error_rate: f32,               // Rate of typing errors
    pub self_correction_rate: f32,     // Rate of self-corrections
    
    // Temporal patterns
    pub peak_activity_hour: Option<u8>, // Hour of peak activity (0-23)
    pub energy_level_trend: f32,       // Energy trend (-1 to 1)
    
    // Stress indicators
    pub stress_indicator: f32,         // Stress level indicator (0-1)
    pub fatigue_indicator: f32,        // Fatigue level indicator (0-1)
}

impl Default for BehavioralMetrics {
    fn default() -> Self {
        Self {
            keystroke_rate: 0.0,
            mouse_activity_level: 0.0,
            window_switch_frequency: 0.0,
            focus_duration: Duration::from_secs(0),
            focus_depth_score: 0.0,
            distraction_frequency: 0.0,
            work_rhythm_consistency: 0.0,
            task_switching_index: 0.0,
            cognitive_load_estimate: 0.0,
            productive_time_ratio: 0.0,
            flow_state_probability: 0.0,
            intervention_receptivity: 0.5,
            error_rate: 0.0,
            self_correction_rate: 0.0,
            peak_activity_hour: None,
            energy_level_trend: 0.0,
            stress_indicator: 0.0,
            fatigue_indicator: 0.0,
        }
    }
}

impl BehavioralMetrics {
    /// Calculate overall productivity score
    pub fn productivity_score(&self) -> f32 {
        let components = [
            self.productive_time_ratio * 0.3,
            self.focus_depth_score * 0.25,
            self.flow_state_probability * 0.2,
            (1.0 - self.distraction_frequency.min(1.0)) * 0.15,
            self.work_rhythm_consistency * 0.1,
        ];
        
        components.iter().sum::<f32>().min(1.0)
    }

    /// Calculate overall wellbeing score
    pub fn wellbeing_score(&self) -> f32 {
        let stress_component = 1.0 - self.stress_indicator;
        let fatigue_component = 1.0 - self.fatigue_indicator;
        let balance_component = 1.0 - (self.task_switching_index * 0.5);
        
        (stress_component * 0.4 + fatigue_component * 0.4 + balance_component * 0.2).min(1.0)
    }

    /// Get intervention priority (higher = more urgent)
    pub fn intervention_priority(&self) -> f32 {
        let stress_factor = self.stress_indicator * 0.4;
        let distraction_factor = self.distraction_frequency.min(1.0) * 0.3;
        let low_productivity_factor = (1.0 - self.productivity_score()) * 0.2;
        let fatigue_factor = self.fatigue_indicator * 0.1;
        
        (stress_factor + distraction_factor + low_productivity_factor + fatigue_factor).min(1.0)
    }
}

/// Trait for calculating specific metric types
pub trait MetricCalculator: Send + Sync {
    fn calculate(&self, window: &AnalysisWindow) -> AnalysisResult<f32>;
    fn metric_name(&self) -> &'static str;
}

/// Main metrics calculation engine
pub struct MetricEngine {
    calculators: HashMap<MetricType, Box<dyn MetricCalculator>>,
}

impl Clone for MetricEngine {
    fn clone(&self) -> Self {
        // Create a new engine instead of cloning the calculators
        Self::new()
    }
}

impl MetricEngine {
    /// Create a new metric engine with all standard calculators
    pub fn new() -> Self {
        let mut calculators: HashMap<MetricType, Box<dyn MetricCalculator>> = HashMap::new();
        
        calculators.insert(MetricType::KeystrokeRate, Box::new(KeystrokeRateCalculator));
        calculators.insert(MetricType::MouseActivity, Box::new(MouseActivityCalculator));
        calculators.insert(MetricType::WindowSwitchFreq, Box::new(WindowSwitchCalculator));
        calculators.insert(MetricType::FocusDepth, Box::new(FocusDepthCalculator));
        calculators.insert(MetricType::WorkRhythm, Box::new(WorkRhythmCalculator));
        calculators.insert(MetricType::CognitiveLoad, Box::new(CognitiveLoadCalculator));
        calculators.insert(MetricType::ProductiveTime, Box::new(ProductiveTimeCalculator));
        calculators.insert(MetricType::FlowProbability, Box::new(FlowProbabilityCalculator));
        calculators.insert(MetricType::StressIndicator, Box::new(StressIndicatorCalculator));
        calculators.insert(MetricType::FatigueIndicator, Box::new(FatigueIndicatorCalculator));
        
        Self { calculators }
    }

    /// Calculate all metrics for a window in parallel
    pub fn calculate_all(&self, window: &AnalysisWindow) -> BehavioralMetrics {
        let mut metrics = BehavioralMetrics::default();
        
        // Calculate metrics sequentially for simplicity
        metrics.keystroke_rate = self.calculate_keystroke_rate(window);
        metrics.mouse_activity_level = self.calculate_mouse_activity(window);
        metrics.window_switch_frequency = self.calculate_window_switch_frequency(window);
        metrics.focus_depth_score = self.calculate_focus_depth(window);
        metrics.work_rhythm_consistency = self.calculate_work_rhythm(window);
        metrics.productive_time_ratio = self.calculate_productive_time(window);
        metrics.stress_indicator = self.calculate_stress_indicator(window);
        metrics.fatigue_indicator = self.calculate_fatigue_indicator(window);
        
        // Sequential calculation of dependent metrics
        metrics.cognitive_load_estimate = self.estimate_cognitive_load(&metrics);
        metrics.flow_state_probability = self.estimate_flow_probability(&metrics);
        metrics.intervention_receptivity = self.calculate_intervention_receptivity(&metrics);
        
        // Additional calculations
        metrics.focus_duration = self.calculate_focus_duration(window);
        metrics.distraction_frequency = self.calculate_distraction_frequency(window);
        metrics.task_switching_index = self.calculate_task_switching_index(window);
        metrics.error_rate = self.calculate_error_rate(window);
        metrics.self_correction_rate = self.calculate_self_correction_rate(window);
        metrics.energy_level_trend = self.calculate_energy_trend(window);
        
        metrics
    }

    // Individual metric calculations
    fn calculate_keystroke_rate(&self, window: &AnalysisWindow) -> f32 {
        if let Some(calculator) = self.calculators.get(&MetricType::KeystrokeRate) {
            calculator.calculate(window).unwrap_or(0.0)
        } else {
            0.0
        }
    }

    fn calculate_mouse_activity(&self, window: &AnalysisWindow) -> f32 {
        if let Some(calculator) = self.calculators.get(&MetricType::MouseActivity) {
            calculator.calculate(window).unwrap_or(0.0)
        } else {
            0.0
        }
    }

    fn calculate_window_switch_frequency(&self, window: &AnalysisWindow) -> f32 {
        if let Some(calculator) = self.calculators.get(&MetricType::WindowSwitchFreq) {
            calculator.calculate(window).unwrap_or(0.0)
        } else {
            0.0
        }
    }

    fn calculate_focus_depth(&self, window: &AnalysisWindow) -> f32 {
        if let Some(calculator) = self.calculators.get(&MetricType::FocusDepth) {
            calculator.calculate(window).unwrap_or(0.0)
        } else {
            0.0
        }
    }

    fn calculate_work_rhythm(&self, window: &AnalysisWindow) -> f32 {
        if let Some(calculator) = self.calculators.get(&MetricType::WorkRhythm) {
            calculator.calculate(window).unwrap_or(0.0)
        } else {
            0.0
        }
    }

    fn calculate_productive_time(&self, window: &AnalysisWindow) -> f32 {
        if let Some(calculator) = self.calculators.get(&MetricType::ProductiveTime) {
            calculator.calculate(window).unwrap_or(0.0)
        } else {
            0.0
        }
    }

    fn calculate_stress_indicator(&self, window: &AnalysisWindow) -> f32 {
        if let Some(calculator) = self.calculators.get(&MetricType::StressIndicator) {
            calculator.calculate(window).unwrap_or(0.0)
        } else {
            0.0
        }
    }

    fn calculate_fatigue_indicator(&self, window: &AnalysisWindow) -> f32 {
        if let Some(calculator) = self.calculators.get(&MetricType::FatigueIndicator) {
            calculator.calculate(window).unwrap_or(0.0)
        } else {
            0.0
        }
    }

    // Complex metric calculations
    fn estimate_cognitive_load(&self, metrics: &BehavioralMetrics) -> f32 {
        let task_switching_load = metrics.task_switching_index * 0.3;
        let error_load = metrics.error_rate * 0.2;
        let activity_load = (metrics.keystroke_rate / 300.0).min(1.0) * 0.2;
        let stress_load = metrics.stress_indicator * 0.3;
        
        (task_switching_load + error_load + activity_load + stress_load).min(1.0)
    }

    fn estimate_flow_probability(&self, metrics: &BehavioralMetrics) -> f32 {
        let focus_component = metrics.focus_depth_score * 0.4;
        let rhythm_component = metrics.work_rhythm_consistency * 0.3;
        let low_distraction = (1.0 - metrics.distraction_frequency.min(1.0)) * 0.2;
        let low_stress = (1.0 - metrics.stress_indicator) * 0.1;
        
        (focus_component + rhythm_component + low_distraction + low_stress).min(1.0)
    }

    fn calculate_intervention_receptivity(&self, metrics: &BehavioralMetrics) -> f32 {
        // Higher when stress is high, flow is low, and distraction is high
        let stress_factor = metrics.stress_indicator * 0.4;
        let low_flow_factor = (1.0 - metrics.flow_state_probability) * 0.3;
        let distraction_factor = metrics.distraction_frequency.min(1.0) * 0.3;
        
        (stress_factor + low_flow_factor + distraction_factor).min(1.0)
    }

    // Additional simple calculations
    fn calculate_focus_duration(&self, window: &AnalysisWindow) -> Duration {
        let window_focus_events = window.get_window_focus_events();
        if window_focus_events.is_empty() {
            return Duration::from_secs(0);
        }

        // Find longest continuous focus period
        let mut max_duration = Duration::from_secs(0);
        
        for event in window_focus_events {
            if let Some(duration_ms) = event.duration_ms {
                let duration = Duration::from_millis(duration_ms as u64);
                max_duration = max_duration.max(duration);
            }
        }
        
        max_duration
    }

    fn calculate_distraction_frequency(&self, window: &AnalysisWindow) -> f32 {
        let window_switches = window.get_window_focus_events().len();
        let duration_hours = window.duration().as_secs_f32() / 3600.0;
        
        if duration_hours > 0.0 {
            window_switches as f32 / duration_hours
        } else {
            0.0
        }
    }

    fn calculate_task_switching_index(&self, window: &AnalysisWindow) -> f32 {
        let window_events = window.get_window_focus_events();
        if window_events.len() < 2 {
            return 0.0;
        }

        // Calculate switching pattern
        let rapid_switches = window_events.windows(2)
            .filter(|pair| {
                if let (Some(prev_duration), Some(curr_duration)) = (pair[0].duration_ms, pair[1].duration_ms) {
                    prev_duration < 5000 && curr_duration < 5000 // Less than 5 seconds
                } else {
                    false
                }
            })
            .count();

        (rapid_switches as f32 / (window_events.len() - 1) as f32).min(1.0)
    }

    fn calculate_error_rate(&self, window: &AnalysisWindow) -> f32 {
        let keystroke_events = window.get_keystroke_events();
        if keystroke_events.is_empty() {
            return 0.0;
        }

        // Count backspace/delete keys (key codes 8 and 46)
        let error_count = keystroke_events.iter()
            .filter(|event| event.key_code == 8 || event.key_code == 46)
            .count();

        error_count as f32 / keystroke_events.len() as f32
    }

    fn calculate_self_correction_rate(&self, window: &AnalysisWindow) -> f32 {
        // This is a simplified calculation - in practice, you'd analyze typing patterns
        self.calculate_error_rate(window) * 0.8 // Assume 80% of errors are self-corrected
    }

    fn calculate_energy_trend(&self, window: &AnalysisWindow) -> f32 {
        let events = &window.events;
        if events.len() < 10 {
            return 0.0;
        }

        // Analyze activity level over time
        let mid_point = events.len() / 2;
        let first_half_activity = events[..mid_point].len() as f32;
        let second_half_activity = events[mid_point..].len() as f32;

        if first_half_activity > 0.0 {
            ((second_half_activity - first_half_activity) / first_half_activity).clamp(-1.0, 1.0)
        } else {
            0.0
        }
    }
}

impl Default for MetricEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Types of metrics that can be calculated
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MetricType {
    KeystrokeRate,
    MouseActivity,
    WindowSwitchFreq,
    FocusDepth,
    WorkRhythm,
    CognitiveLoad,
    ProductiveTime,
    FlowProbability,
    StressIndicator,
    FatigueIndicator,
}

// Individual metric calculators
struct KeystrokeRateCalculator;
impl MetricCalculator for KeystrokeRateCalculator {
    fn calculate(&self, window: &AnalysisWindow) -> AnalysisResult<f32> {
        let keystroke_count = window.get_keystroke_events().len();
        let duration_minutes = window.duration().as_secs_f32() / 60.0;
        
        if duration_minutes > 0.0 {
            Ok(keystroke_count as f32 / duration_minutes)
        } else {
            Ok(0.0)
        }
    }
    
    fn metric_name(&self) -> &'static str { "keystroke_rate" }
}

struct MouseActivityCalculator;
impl MetricCalculator for MouseActivityCalculator {
    fn calculate(&self, window: &AnalysisWindow) -> AnalysisResult<f32> {
        let mouse_events = window.get_mouse_move_events();
        if mouse_events.is_empty() {
            return Ok(0.0);
        }

        let total_velocity: f32 = mouse_events.iter().map(|e| e.velocity).sum();
        let avg_velocity = total_velocity / mouse_events.len() as f32;
        
        // Normalize to 0-1 range (assuming max reasonable velocity is 2000 px/s)
        Ok((avg_velocity / 2000.0).min(1.0))
    }
    
    fn metric_name(&self) -> &'static str { "mouse_activity" }
}

struct WindowSwitchCalculator;
impl MetricCalculator for WindowSwitchCalculator {
    fn calculate(&self, window: &AnalysisWindow) -> AnalysisResult<f32> {
        let switch_count = window.get_window_focus_events().len();
        let duration_minutes = window.duration().as_secs_f32() / 60.0;
        
        if duration_minutes > 0.0 {
            Ok(switch_count as f32 / duration_minutes)
        } else {
            Ok(0.0)
        }
    }
    
    fn metric_name(&self) -> &'static str { "window_switch_frequency" }
}

struct FocusDepthCalculator;
impl MetricCalculator for FocusDepthCalculator {
    fn calculate(&self, window: &AnalysisWindow) -> AnalysisResult<f32> {
        let window_events = window.get_window_focus_events();
        if window_events.is_empty() {
            return Ok(0.0);
        }

        // Calculate average focus duration
        let total_duration: u32 = window_events.iter()
            .filter_map(|e| e.duration_ms)
            .sum();
        
        if window_events.is_empty() {
            return Ok(0.0);
        }

        let avg_duration = total_duration as f32 / window_events.len() as f32;
        
        // Normalize (assuming 5 minutes = deep focus)
        Ok((avg_duration / (5.0 * 60.0 * 1000.0)).min(1.0))
    }
    
    fn metric_name(&self) -> &'static str { "focus_depth" }
}

struct WorkRhythmCalculator;
impl MetricCalculator for WorkRhythmCalculator {
    fn calculate(&self, window: &AnalysisWindow) -> AnalysisResult<f32> {
        let keystroke_events = window.get_keystroke_events();
        if keystroke_events.len() < 10 {
            return Ok(0.0);
        }

        // Analyze rhythm consistency in inter-keystroke intervals
        let intervals: Vec<f32> = keystroke_events.windows(2)
            .filter_map(|pair| {
                let time1 = pair[0].timestamp;
                let time2 = pair[1].timestamp;
                
                let diff = time2.signed_duration_since(time1).num_milliseconds();
                if diff > 0 && diff < 2000 { // Reasonable typing interval
                    Some(diff as f32)
                } else {
                    None
                }
            })
            .collect();

        if intervals.len() < 5 {
            return Ok(0.0);
        }

        // Calculate coefficient of variation (lower = more consistent)
        let mean = intervals.iter().sum::<f32>() / intervals.len() as f32;
        let variance = intervals.iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f32>() / intervals.len() as f32;
        let std_dev = variance.sqrt();
        
        let cv = if mean > 0.0 { std_dev / mean } else { 1.0 };
        
        // Convert to consistency score (1 = very consistent, 0 = very inconsistent)
        Ok((1.0 - cv.min(1.0)).max(0.0))
    }
    
    fn metric_name(&self) -> &'static str { "work_rhythm" }
}

struct CognitiveLoadCalculator;
impl MetricCalculator for CognitiveLoadCalculator {
    fn calculate(&self, _window: &AnalysisWindow) -> AnalysisResult<f32> {
        // This is calculated in the main engine based on other metrics
        Ok(0.0)
    }
    
    fn metric_name(&self) -> &'static str { "cognitive_load" }
}

struct ProductiveTimeCalculator;
impl MetricCalculator for ProductiveTimeCalculator {
    fn calculate(&self, window: &AnalysisWindow) -> AnalysisResult<f32> {
        let total_events = window.events.len();
        if total_events == 0 {
            return Ok(0.0);
        }

        // Consider time with activity as productive
        let active_events = total_events; // All events indicate some activity
        let window_duration = window.duration().as_secs_f32();
        
        // Estimate productive time based on event density
        let events_per_second = active_events as f32 / window_duration.max(1.0);
        
        // Normalize (1 event per 3 seconds = fully productive)
        Ok((events_per_second * 3.0).min(1.0))
    }
    
    fn metric_name(&self) -> &'static str { "productive_time" }
}

struct FlowProbabilityCalculator;
impl MetricCalculator for FlowProbabilityCalculator {
    fn calculate(&self, _window: &AnalysisWindow) -> AnalysisResult<f32> {
        // This is calculated in the main engine based on other metrics
        Ok(0.0)
    }
    
    fn metric_name(&self) -> &'static str { "flow_probability" }
}

struct StressIndicatorCalculator;
impl MetricCalculator for StressIndicatorCalculator {
    fn calculate(&self, window: &AnalysisWindow) -> AnalysisResult<f32> {
        let keystroke_events = window.get_keystroke_events();
        let mouse_events = window.get_mouse_move_events();
        
        if keystroke_events.is_empty() && mouse_events.is_empty() {
            return Ok(0.0);
        }

        // Rapid, erratic behavior indicates stress
        let mut stress_indicators = 0.0;
        let mut total_indicators = 0.0;

        // Check for rapid keystroke patterns
        if keystroke_events.len() > 5 {
            let rapid_keystrokes = keystroke_events.windows(2)
                .filter(|pair| {
                    if let (Some(iki1), Some(iki2)) = (pair[0].inter_key_interval_ms, pair[1].inter_key_interval_ms) {
                        iki1 < 50 && iki2 < 50 // Very rapid typing
                    } else {
                        false
                    }
                })
                .count();
            
            stress_indicators += (rapid_keystrokes as f32 / keystroke_events.len() as f32) * 0.5;
            total_indicators += 0.5;
        }

        // Check for erratic mouse movement
        if mouse_events.len() > 5 {
            let high_velocity_moves = mouse_events.iter()
                .filter(|e| e.velocity > 1500.0) // High velocity movements
                .count();
            
            stress_indicators += (high_velocity_moves as f32 / mouse_events.len() as f32) * 0.3;
            total_indicators += 0.3;
        }

        // Check for excessive window switching
        let window_events = window.get_window_focus_events();
        if window_events.len() > 2 {
            let rapid_switches = window_events.windows(2)
                .filter(|pair| {
                    pair[0].duration_ms.map_or(false, |d| d < 3000) // Less than 3 seconds
                })
                .count();
            
            stress_indicators += (rapid_switches as f32 / window_events.len() as f32) * 0.2;
            total_indicators += 0.2;
        }

        if total_indicators > 0.0 {
            Ok((stress_indicators / total_indicators).min(1.0))
        } else {
            Ok(0.0)
        }
    }
    
    fn metric_name(&self) -> &'static str { "stress_indicator" }
}

struct FatigueIndicatorCalculator;
impl MetricCalculator for FatigueIndicatorCalculator {
    fn calculate(&self, window: &AnalysisWindow) -> AnalysisResult<f32> {
        let keystroke_events = window.get_keystroke_events();
        
        if keystroke_events.len() < 10 {
            return Ok(0.0);
        }

        // Gradually increasing inter-keystroke intervals indicate fatigue
        let intervals: Vec<f32> = keystroke_events.windows(2)
            .filter_map(|pair| {
                if let (Some(iki1), Some(iki2)) = (pair[0].inter_key_interval_ms, pair[1].inter_key_interval_ms) {
                    Some((iki1 + iki2) as f32 / 2.0)
                } else {
                    None
                }
            })
            .collect();

        if intervals.len() < 5 {
            return Ok(0.0);
        }

        // Calculate trend in intervals (increasing = fatigue)
        let mid_point = intervals.len() / 2;
        let early_avg = intervals[..mid_point].iter().sum::<f32>() / mid_point as f32;
        let late_avg = intervals[mid_point..].iter().sum::<f32>() / (intervals.len() - mid_point) as f32;

        if early_avg > 0.0 {
            let trend = (late_avg - early_avg) / early_avg;
            Ok(trend.max(0.0).min(1.0)) // Only positive trends indicate fatigue
        } else {
            Ok(0.0)
        }
    }
    
    fn metric_name(&self) -> &'static str { "fatigue_indicator" }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sliding_window::AnalysisWindow;
    use chrono::Utc;
    use skelly_jelly_storage::types::*;
    use std::time::SystemTime;

    #[test]
    fn test_behavioral_metrics_default() {
        let metrics = BehavioralMetrics::default();
        assert_eq!(metrics.keystroke_rate, 0.0);
        assert_eq!(metrics.intervention_receptivity, 0.5);
    }

    #[test]
    fn test_productivity_score_calculation() {
        let mut metrics = BehavioralMetrics::default();
        metrics.productive_time_ratio = 0.8;
        metrics.focus_depth_score = 0.7;
        metrics.flow_state_probability = 0.6;
        metrics.distraction_frequency = 0.2;
        metrics.work_rhythm_consistency = 0.9;
        
        let score = metrics.productivity_score();
        assert!(score > 0.0 && score <= 1.0);
    }

    #[test]
    fn test_keystroke_rate_calculator() {
        let calculator = KeystrokeRateCalculator;
        let mut window = AnalysisWindow::new(SystemTime::now());
        
        // Add some keystroke events
        for i in 0..60 {
            let event = RawEvent::Keystroke(KeystrokeEvent {
                timestamp: Utc::now(),
                key_code: 65 + (i % 26),
                modifiers: KeyModifiers::default(),
                inter_key_interval_ms: Some(100),
            });
            window.add_event(event);
        }
        
        let result = calculator.calculate(&window);
        assert!(result.is_ok());
        assert!(result.unwrap() > 0.0);
    }

    #[test]
    fn test_metric_engine_creation() {
        let engine = MetricEngine::new();
        assert!(engine.calculators.contains_key(&MetricType::KeystrokeRate));
        assert!(engine.calculators.contains_key(&MetricType::MouseActivity));
    }

    #[test]
    fn test_metrics_calculation() {
        let engine = MetricEngine::new();
        let window = AnalysisWindow::new(SystemTime::now());
        
        let metrics = engine.calculate_all(&window);
        
        // All metrics should be initialized
        assert!(metrics.keystroke_rate >= 0.0);
        assert!(metrics.intervention_receptivity >= 0.0 && metrics.intervention_receptivity <= 1.0);
    }
}