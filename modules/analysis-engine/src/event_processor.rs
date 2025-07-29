//! Event processing pipeline for behavioral analysis

use std::time::{Duration, Instant};
use tokio::sync::mpsc;

use crate::{
    error::{AnalysisError, AnalysisResult},
    feature_extraction::FeatureExtractionPipeline,
    metrics::{BehavioralMetrics, MetricEngine},
    models::StateClassifier,
    screenshot::ScreenshotAnalyzer,
    sliding_window::{AnalysisWindow, SlidingWindowManager},
    types::AnalysisResult as AnalysisResultType,
};
use skelly_jelly_storage::types::{EventBatch, RawEvent, ScreenshotId};

/// Main event processor that coordinates analysis pipeline
pub struct EventProcessor {
    /// Sliding window manager for event batching
    window_manager: SlidingWindowManager,
    
    /// Feature extraction pipeline
    feature_extractor: FeatureExtractionPipeline,
    
    /// Behavioral metrics calculator
    metric_engine: MetricEngine,
    
    /// Screenshot analyzer
    screenshot_analyzer: ScreenshotAnalyzer,
    
    /// ADHD state classifier
    state_classifier: StateClassifier,
    
    /// Configuration
    config: EventProcessorConfig,
    
    /// Performance metrics
    total_events_processed: u64,
    total_windows_analyzed: u64,
    avg_processing_time_ms: f32,
}

impl EventProcessor {
    /// Create a new event processor
    pub fn new() -> Self {
        Self {
            window_manager: SlidingWindowManager::new(
                Duration::from_secs(30),  // 30-second windows
                Duration::from_secs(5),   // 5-second overlap
                100,                      // Keep 100 windows in history
            ),
            feature_extractor: FeatureExtractionPipeline::new(),
            metric_engine: MetricEngine::new(),
            screenshot_analyzer: ScreenshotAnalyzer::new(),
            state_classifier: StateClassifier::new(),
            config: EventProcessorConfig::default(),
            total_events_processed: 0,
            total_windows_analyzed: 0,
            avg_processing_time_ms: 0.0,
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: EventProcessorConfig) -> Self {
        Self {
            window_manager: SlidingWindowManager::new(
                config.window_size,
                config.window_overlap,
                config.history_size,
            ),
            feature_extractor: FeatureExtractionPipeline::new(),
            metric_engine: MetricEngine::new(),
            screenshot_analyzer: ScreenshotAnalyzer::new(),
            state_classifier: StateClassifier::new(),
            config,
            total_events_processed: 0,
            total_windows_analyzed: 0,
            avg_processing_time_ms: 0.0,
        }
    }

    /// Process a batch of events from storage
    pub async fn process_event_batch(&mut self, batch: EventBatch) -> AnalysisResult<Option<AnalysisResultType>> {
        let start_time = Instant::now();
        
        // Add events to sliding window
        let mut completed_window = None;
        
        for event in batch.events {
            self.total_events_processed += 1;
            
            if let Some(window) = self.window_manager.add_event(event)? {
                completed_window = Some(window);
                break; // Process one window at a time
            }
        }
        
        // Add screenshot references
        for screenshot_id in batch.screenshot_refs {
            self.window_manager.add_screenshot(screenshot_id);
        }
        
        // Process completed window if available
        if let Some(window) = completed_window {
            let result = self.analyze_window(window).await?;
            
            // Update performance metrics
            let processing_time = start_time.elapsed().as_millis() as f32;
            self.update_performance_metrics(processing_time);
            
            return Ok(Some(result));
        }
        
        Ok(None)
    }

    /// Process individual events in real-time
    pub async fn process_event(&mut self, event: RawEvent) -> AnalysisResult<Option<AnalysisResultType>> {
        self.total_events_processed += 1;
        
        if let Some(window) = self.window_manager.add_event(event)? {
            let result = self.analyze_window(window).await?;
            return Ok(Some(result));
        }
        
        Ok(None)
    }

    /// Add a screenshot reference to current window
    pub fn add_screenshot(&mut self, screenshot_id: ScreenshotId) {
        self.window_manager.add_screenshot(screenshot_id);
    }

    /// Force analysis of current window
    pub async fn force_analysis(&mut self) -> AnalysisResult<Option<AnalysisResultType>> {
        if let Some(window) = self.window_manager.advance_window()? {
            let result = self.analyze_window(window).await?;
            return Ok(Some(result));
        }
        
        Ok(None)
    }

    /// Analyze a completed window
    async fn analyze_window(&mut self, mut window: AnalysisWindow) -> AnalysisResult<AnalysisResultType> {
        let start_time = Instant::now();
        self.total_windows_analyzed += 1;

        // Check if window has sufficient data
        if !window.has_sufficient_data() {
            return Err(AnalysisError::InsufficientData {
                required: 10,
                available: window.events.len(),
            });
        }

        // Extract features in parallel with metrics calculation
        let (features_result, metrics_result, screenshot_result) = tokio::join!(
            self.feature_extractor.extract_all_features(&window),
            tokio::spawn({
                let engine = self.metric_engine.clone();
                let window_clone = window.clone();
                async move { engine.calculate_all(&window_clone) }
            }),
            self.analyze_screenshots(&window)
        );

        let features = features_result?;
        let metrics = metrics_result.map_err(|e| AnalysisError::EventProcessingError {
            message: format!("Failed to join metrics calculation: {}", e),
        })?;
        let screenshot_context = screenshot_result?;

        // Update window with computed data
        window.extracted_features = features.clone();
        window.computed_metrics = metrics.clone();
        window.screenshot_context = screenshot_context.clone();

        // Classify ADHD state
        let state = self.state_classifier.classify(&features).await?;

        // Create analysis result
        let processing_time = start_time.elapsed().as_millis() as u32;
        let confidence = self.state_classifier.get_confidence();
        let feature_importance = self.state_classifier.get_feature_importance().await;

        let mut result = AnalysisResultType::new(window.window_id, state);
        result.confidence = confidence;
        result.metrics = metrics;
        result.work_context = screenshot_context.map(|ctx| crate::screenshot::WorkContext {
            primary_work_type: ctx.work_type,
            confidence: 0.8, // Would be calculated from analysis
            secondary_activities: Vec::new(),
            estimated_complexity: ctx.estimated_cognitive_load,
            focus_score: (1.0 - ctx.estimated_cognitive_load).max(0.0),
            distraction_indicators: ctx.activity_indicators,
        });
        result.intervention_readiness = result.state.intervention_urgency();
        result.processing_time_ms = processing_time;
        result.feature_importance = feature_importance.into_iter().collect();

        Ok(result)
    }

    /// Analyze screenshots in the window
    async fn analyze_screenshots(&self, window: &AnalysisWindow) -> AnalysisResult<Option<crate::screenshot::ScreenshotContext>> {
        if !self.config.enable_screenshot_analysis || window.screenshot_refs.is_empty() {
            return Ok(None);
        }

        // For now, return None as we don't have access to actual screenshot data
        // In a real implementation, this would:
        // 1. Load screenshot data from storage
        // 2. Analyze each screenshot
        // 3. Combine results for the window
        
        Ok(None)
    }

    /// Update performance metrics
    fn update_performance_metrics(&mut self, processing_time_ms: f32) {
        let n = self.total_windows_analyzed as f32;
        self.avg_processing_time_ms = (self.avg_processing_time_ms * (n - 1.0) + processing_time_ms) / n;
    }

    /// Get current window for inspection
    pub fn current_window(&self) -> &AnalysisWindow {
        self.window_manager.current_window()
    }

    /// Get recent analysis windows
    pub fn get_recent_windows(&self, count: usize) -> Vec<&AnalysisWindow> {
        self.window_manager.get_recent_windows(count)
    }

    /// Get processor performance statistics
    pub fn get_performance_stats(&self) -> ProcessorStats {
        ProcessorStats {
            total_events_processed: self.total_events_processed,
            total_windows_analyzed: self.total_windows_analyzed,
            avg_processing_time_ms: self.avg_processing_time_ms,
            window_manager_stats: self.window_manager.get_stats(),
            feature_count: self.feature_extractor.total_feature_count(),
        }
    }

    /// Update state classifier with user feedback
    pub async fn update_with_feedback(&mut self, window_id: uuid::Uuid, true_state: crate::models::ADHDState) -> AnalysisResult<()> {
        // Find the window and its features
        if let Some(window) = self.window_manager.get_window_by_id(window_id) {
            let features = &window.extracted_features;
            self.state_classifier.update_models(features, &true_state).await?;
        } else {
            return Err(AnalysisError::WindowError {
                reason: format!("Window {} not found in history", window_id),
            });
        }
        
        Ok(())
    }

    /// Get classifier metrics
    pub async fn get_classifier_metrics(&self) -> crate::models::ModelMetrics {
        self.state_classifier.get_ensemble_metrics()
    }

    /// Cleanup old data to manage memory
    pub fn cleanup(&mut self) {
        self.window_manager.cleanup_old_windows(self.config.history_size);
    }
}

impl Default for EventProcessor {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration for event processor
#[derive(Debug, Clone)]
pub struct EventProcessorConfig {
    /// Size of analysis windows
    pub window_size: Duration,
    
    /// Overlap between windows
    pub window_overlap: Duration,
    
    /// Number of windows to keep in history
    pub history_size: usize,
    
    /// Enable screenshot analysis
    pub enable_screenshot_analysis: bool,
    
    /// Minimum events required for analysis
    pub min_events_for_analysis: usize,
    
    /// Enable real-time processing
    pub enable_realtime_processing: bool,
    
    /// Processing timeout
    pub processing_timeout: Duration,
}

impl Default for EventProcessorConfig {
    fn default() -> Self {
        Self {
            window_size: Duration::from_secs(30),
            window_overlap: Duration::from_secs(5),
            history_size: 100,
            enable_screenshot_analysis: true,
            min_events_for_analysis: 10,
            enable_realtime_processing: true,
            processing_timeout: Duration::from_millis(50), // Target <50ms processing
        }
    }
}

/// Performance statistics for the event processor
#[derive(Debug, Clone)]
pub struct ProcessorStats {
    pub total_events_processed: u64,
    pub total_windows_analyzed: u64,
    pub avg_processing_time_ms: f32,
    pub window_manager_stats: crate::sliding_window::WindowManagerStats,
    pub feature_count: usize,
}

/// Event processor builder for easy configuration
pub struct EventProcessorBuilder {
    config: EventProcessorConfig,
}

impl EventProcessorBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            config: EventProcessorConfig::default(),
        }
    }

    /// Set window size
    pub fn window_size(mut self, size: Duration) -> Self {
        self.config.window_size = size;
        self
    }

    /// Set window overlap
    pub fn window_overlap(mut self, overlap: Duration) -> Self {
        self.config.window_overlap = overlap;
        self
    }

    /// Set history size
    pub fn history_size(mut self, size: usize) -> Self {
        self.config.history_size = size;
        self
    }

    /// Enable/disable screenshot analysis
    pub fn screenshot_analysis(mut self, enable: bool) -> Self {
        self.config.enable_screenshot_analysis = enable;
        self
    }

    /// Set minimum events for analysis
    pub fn min_events(mut self, min: usize) -> Self {
        self.config.min_events_for_analysis = min;
        self
    }

    /// Build the event processor
    pub fn build(self) -> EventProcessor {
        EventProcessor::with_config(self.config)
    }
}

impl Default for EventProcessorBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use skelly_jelly_storage::types::*;
    use std::time::SystemTime;

    #[tokio::test]
    async fn test_event_processor_creation() {
        let processor = EventProcessor::new();
        assert_eq!(processor.total_events_processed, 0);
        assert_eq!(processor.total_windows_analyzed, 0);
    }

    #[tokio::test]
    async fn test_event_processing() {
        let mut processor = EventProcessor::new();
        
        let event = RawEvent::Keystroke(KeystrokeEvent {
            timestamp: Utc::now(),
            key_code: 65,
            modifiers: KeyModifiers::default(),
            inter_key_interval_ms: Some(100),
        });
        
        let result = processor.process_event(event).await;
        assert!(result.is_ok());
        assert_eq!(processor.total_events_processed, 1);
    }

    #[tokio::test]
    async fn test_batch_processing() {
        let mut processor = EventProcessor::new();
        
        let batch = EventBatch {
            window_id: uuid::Uuid::new_v4(),
            start_time: Utc::now(),
            end_time: Utc::now(),
            events: vec![
                RawEvent::Keystroke(KeystrokeEvent {
                    timestamp: Utc::now(),
                    key_code: 65,
                    modifiers: KeyModifiers::default(),
                    inter_key_interval_ms: Some(100),
                })
            ],
            screenshot_refs: vec![],
        };
        
        let result = processor.process_event_batch(batch).await;
        assert!(result.is_ok());
        assert_eq!(processor.total_events_processed, 1);
    }

    #[test]
    fn test_processor_builder() {
        let processor = EventProcessorBuilder::new()
            .window_size(Duration::from_secs(60))
            .window_overlap(Duration::from_secs(10))
            .history_size(50)
            .screenshot_analysis(false)
            .min_events(5)
            .build();
        
        assert_eq!(processor.config.window_size, Duration::from_secs(60));
        assert_eq!(processor.config.window_overlap, Duration::from_secs(10));
        assert_eq!(processor.config.history_size, 50);
        assert!(!processor.config.enable_screenshot_analysis);
        assert_eq!(processor.config.min_events_for_analysis, 5);
    }

    #[test]
    fn test_performance_stats() {
        let processor = EventProcessor::new();
        let stats = processor.get_performance_stats();
        
        assert_eq!(stats.total_events_processed, 0);
        assert_eq!(stats.total_windows_analyzed, 0);
        assert!(stats.feature_count > 0);
    }

    #[tokio::test]
    async fn test_insufficient_data_handling() {
        let mut processor = EventProcessor::new();
        
        // Create a window with insufficient data
        let mut window = crate::sliding_window::AnalysisWindow::new(SystemTime::now());
        // Don't add enough events
        
        let result = processor.analyze_window(window).await;
        assert!(result.is_err());
        
        if let Err(AnalysisError::InsufficientData { required, available }) = result {
            assert_eq!(required, 10);
            assert_eq!(available, 0);
        } else {
            panic!("Expected InsufficientData error");
        }
    }
}