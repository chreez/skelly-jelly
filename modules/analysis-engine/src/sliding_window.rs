//! Sliding window management for event analysis

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use skelly_jelly_storage::types::{RawEvent, ScreenshotId};
use std::time::{Duration, Instant, SystemTime};
use uuid::Uuid;

use crate::{
    error::{AnalysisError, AnalysisResult},
    metrics::BehavioralMetrics,
    screenshot::ScreenshotContext,
    types::FeatureVector,
};

/// Analysis window containing events and computed features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisWindow {
    /// Unique identifier for this window
    pub window_id: Uuid,
    
    /// Window start time
    pub start_time: SystemTime,
    
    /// Window end time
    pub end_time: SystemTime,
    
    /// Raw events in this window
    pub events: Vec<RawEvent>,
    
    /// Extracted feature vector
    pub extracted_features: FeatureVector,
    
    /// Computed behavioral metrics
    pub computed_metrics: BehavioralMetrics,
    
    /// Screenshot context analysis
    pub screenshot_context: Option<ScreenshotContext>,
    
    /// Screenshot references for this window
    pub screenshot_refs: Vec<ScreenshotId>,
    
    /// Quality score of this window (0.0-1.0)
    pub quality_score: f32,
    
    /// Whether this window has been fully processed
    pub is_complete: bool,
}

impl AnalysisWindow {
    /// Create a new analysis window
    pub fn new(start_time: SystemTime) -> Self {
        Self {
            window_id: Uuid::new_v4(),
            start_time,
            end_time: start_time,
            events: Vec::with_capacity(1000),
            extracted_features: FeatureVector::default(),
            computed_metrics: BehavioralMetrics::default(),
            screenshot_context: None,
            screenshot_refs: Vec::new(),
            quality_score: 0.0,
            is_complete: false,
        }
    }

    /// Add an event to this window
    pub fn add_event(&mut self, event: RawEvent) {
        self.events.push(event);
        self.update_end_time();
    }

    /// Add a screenshot reference
    pub fn add_screenshot(&mut self, screenshot_id: ScreenshotId) {
        self.screenshot_refs.push(screenshot_id);
    }

    /// Update the end time based on the latest event
    fn update_end_time(&mut self) {
        if let Some(latest_event) = self.events.last() {
            let latest_timestamp = latest_event.timestamp();
            if let Ok(duration) = latest_timestamp.signed_duration_since(DateTime::<Utc>::UNIX_EPOCH).to_std() {
                self.end_time = SystemTime::UNIX_EPOCH + duration;
            }
        }
    }

    /// Get the duration of this window
    pub fn duration(&self) -> Duration {
        self.end_time.duration_since(self.start_time).unwrap_or_default()
    }

    /// Calculate quality score based on event count and distribution
    pub fn calculate_quality_score(&mut self) {
        let event_count = self.events.len();
        let duration_secs = self.duration().as_secs() as f32;
        
        if duration_secs == 0.0 {
            self.quality_score = 0.0;
            return;
        }

        // Base score from event density
        let event_density = event_count as f32 / duration_secs;
        let density_score = (event_density / 10.0).min(1.0);

        // Score from event type diversity
        let keystroke_count = self.events.iter().filter(|e| matches!(e, RawEvent::Keystroke(_))).count();
        let mouse_count = self.events.iter().filter(|e| matches!(e, RawEvent::MouseMove(_) | RawEvent::MouseClick(_))).count();
        let window_count = self.events.iter().filter(|e| matches!(e, RawEvent::WindowFocus(_))).count();
        
        let diversity_score = if event_count > 0 {
            let types_present = [keystroke_count > 0, mouse_count > 0, window_count > 0]
                .iter()
                .map(|&present| if present { 1.0 } else { 0.0 })
                .sum::<f32>() / 3.0;
            types_present
        } else {
            0.0
        };

        // Screenshot bonus
        let screenshot_bonus = if !self.screenshot_refs.is_empty() { 0.1 } else { 0.0 };

        self.quality_score = (density_score * 0.5 + diversity_score * 0.4 + screenshot_bonus).min(1.0);
    }

    /// Check if window has sufficient data for analysis
    pub fn has_sufficient_data(&self) -> bool {
        self.events.len() >= 10 && self.quality_score >= 0.3
    }

    /// Get events of a specific type
    pub fn get_keystroke_events(&self) -> Vec<&skelly_jelly_storage::types::KeystrokeEvent> {
        self.events.iter().filter_map(|e| match e {
            RawEvent::Keystroke(ke) => Some(ke),
            _ => None,
        }).collect()
    }

    pub fn get_mouse_move_events(&self) -> Vec<&skelly_jelly_storage::types::MouseMoveEvent> {
        self.events.iter().filter_map(|e| match e {
            RawEvent::MouseMove(me) => Some(me),
            _ => None,
        }).collect()
    }

    pub fn get_mouse_click_events(&self) -> Vec<&skelly_jelly_storage::types::MouseClickEvent> {
        self.events.iter().filter_map(|e| match e {
            RawEvent::MouseClick(me) => Some(me),
            _ => None,
        }).collect()
    }

    pub fn get_window_focus_events(&self) -> Vec<&skelly_jelly_storage::types::WindowFocusEvent> {
        self.events.iter().filter_map(|e| match e {
            RawEvent::WindowFocus(we) => Some(we),
            _ => None,
        }).collect()
    }

    pub fn get_resource_events(&self) -> Vec<&skelly_jelly_storage::types::ResourceEvent> {
        self.events.iter().filter_map(|e| match e {
            RawEvent::ResourceUsage(re) => Some(re),
            _ => None,
        }).collect()
    }
}

/// Manages sliding windows for continuous analysis
pub struct SlidingWindowManager {
    /// Current active window
    current_window: AnalysisWindow,
    
    /// Historical windows for trend analysis  
    window_history: Vec<AnalysisWindow>,
    
    /// Maximum history size
    max_history: usize,
    
    /// Window size in seconds
    window_size: Duration,
    
    /// Overlap between windows in seconds
    overlap_duration: Duration,
    
    /// Last window creation time
    last_window_time: Instant,
    
    /// Performance metrics
    total_windows_processed: u64,
    avg_window_quality: f32,
}

impl SlidingWindowManager {
    /// Create a new sliding window manager
    pub fn new(window_size: Duration, overlap_duration: Duration, history_size: usize) -> Self {
        Self {
            current_window: AnalysisWindow::new(SystemTime::now()),
            window_history: Vec::with_capacity(history_size),
            max_history: history_size,
            window_size,
            overlap_duration,
            last_window_time: Instant::now(),
            total_windows_processed: 0,
            avg_window_quality: 0.0,
        }
    }

    /// Add an event to the current window
    pub fn add_event(&mut self, event: RawEvent) -> AnalysisResult<Option<AnalysisWindow>> {
        self.current_window.add_event(event);
        
        // Check if we need to create a new window
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_window_time);
        
        if elapsed >= (self.window_size - self.overlap_duration) {
            self.advance_window()
        } else {
            Ok(None)
        }
    }

    /// Add a screenshot reference to the current window
    pub fn add_screenshot(&mut self, screenshot_id: ScreenshotId) {
        self.current_window.add_screenshot(screenshot_id);
    }

    /// Force completion of the current window and start a new one
    pub fn advance_window(&mut self) -> AnalysisResult<Option<AnalysisWindow>> {
        // Finalize the current window
        self.current_window.calculate_quality_score();
        self.current_window.is_complete = true;

        // Update metrics
        self.total_windows_processed += 1;
        self.avg_window_quality = (self.avg_window_quality * (self.total_windows_processed - 1) as f32 
                                  + self.current_window.quality_score) / self.total_windows_processed as f32;

        // Return the completed window if it has sufficient data
        let completed_window = if self.current_window.has_sufficient_data() {
            Some(self.current_window.clone())
        } else {
            None
        };

        // Store in history
        self.window_history.push(self.current_window.clone());
        
        // Maintain history size limit
        if self.window_history.len() > self.max_history {
            self.window_history.remove(0);
        }

        // Create new window with overlap
        let overlap_start = self.current_window.end_time - self.overlap_duration;
        let new_window_start = overlap_start.max(self.current_window.start_time);
        
        self.current_window = AnalysisWindow::new(new_window_start);
        
        // Copy overlapping events to new window
        let overlap_cutoff = DateTime::<Utc>::from(overlap_start);
        let overlapping_events: Vec<RawEvent> = self.window_history
            .last()
            .map(|prev_window| {
                prev_window.events.iter()
                    .filter(|event| event.timestamp() >= overlap_cutoff)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default();

        for event in overlapping_events {
            self.current_window.add_event(event);
        }

        self.last_window_time = Instant::now();
        
        Ok(completed_window)
    }

    /// Get the current window (for inspection, not analysis)
    pub fn current_window(&self) -> &AnalysisWindow {
        &self.current_window
    }

    /// Get recent windows for trend analysis
    pub fn get_recent_windows(&self, count: usize) -> Vec<&AnalysisWindow> {
        let available = self.window_history.len().min(count);
        self.window_history.iter()
            .rev()
            .take(available)
            .collect()
    }

    /// Get window by ID from history
    pub fn get_window_by_id(&self, window_id: Uuid) -> Option<&AnalysisWindow> {
        self.window_history.iter()
            .find(|window| window.window_id == window_id)
    }

    /// Get performance statistics
    pub fn get_stats(&self) -> WindowManagerStats {
        WindowManagerStats {
            total_windows: self.total_windows_processed,
            avg_quality: self.avg_window_quality,
            current_window_events: self.current_window.events.len(),
            history_size: self.window_history.len(),
            window_size_secs: self.window_size.as_secs(),
            overlap_secs: self.overlap_duration.as_secs(),
        }
    }

    /// Clear old windows to free memory
    pub fn cleanup_old_windows(&mut self, keep_count: usize) {
        if self.window_history.len() > keep_count {
            let to_remove = self.window_history.len() - keep_count;
            self.window_history.drain(..to_remove);
        }
    }
}

/// Performance statistics for the window manager
#[derive(Debug, Clone)]
pub struct WindowManagerStats {
    pub total_windows: u64,
    pub avg_quality: f32,
    pub current_window_events: usize,
    pub history_size: usize,
    pub window_size_secs: u64,
    pub overlap_secs: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use skelly_jelly_storage::types::{KeystrokeEvent, KeyModifiers};

    #[test]
    fn test_analysis_window_creation() {
        let window = AnalysisWindow::new(SystemTime::now());
        assert_eq!(window.events.len(), 0);
        assert!(!window.is_complete);
        assert_eq!(window.quality_score, 0.0);
    }

    #[test]
    fn test_window_event_addition() {
        let mut window = AnalysisWindow::new(SystemTime::now());
        
        let event = RawEvent::Keystroke(KeystrokeEvent {
            timestamp: Utc::now(),
            key_code: 65,
            modifiers: KeyModifiers::default(),
            inter_key_interval_ms: Some(100),
        });
        
        window.add_event(event);
        assert_eq!(window.events.len(), 1);
    }

    #[test]
    fn test_quality_score_calculation() {
        let mut window = AnalysisWindow::new(SystemTime::now());
        
        // Add some events
        for i in 0..20 {
            let event = RawEvent::Keystroke(KeystrokeEvent {
                timestamp: Utc::now(),
                key_code: 65 + i,
                modifiers: KeyModifiers::default(),
                inter_key_interval_ms: Some(100),
            });
            window.add_event(event);
        }
        
        window.calculate_quality_score();
        assert!(window.quality_score > 0.0);
        assert!(window.has_sufficient_data());
    }

    #[test]
    fn test_sliding_window_manager() {
        let mut manager = SlidingWindowManager::new(
            Duration::from_secs(30),
            Duration::from_secs(5),
            10,
        );
        
        let event = RawEvent::Keystroke(KeystrokeEvent {
            timestamp: Utc::now(),
            key_code: 65,
            modifiers: KeyModifiers::default(),
            inter_key_interval_ms: Some(100),
        });
        
        let result = manager.add_event(event);
        assert!(result.is_ok());
        assert_eq!(manager.current_window().events.len(), 1);
    }

    #[test]
    fn test_window_stats() {
        let manager = SlidingWindowManager::new(
            Duration::from_secs(30),
            Duration::from_secs(5),
            10,
        );
        
        let stats = manager.get_stats();
        assert_eq!(stats.window_size_secs, 30);
        assert_eq!(stats.overlap_secs, 5);
        assert_eq!(stats.total_windows, 0);
    }
}