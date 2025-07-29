//! Event Bus integration for real-time behavioral data processing
//!
//! This module provides the bridge between the Event Bus and the Analysis Engine,
//! enabling real-time ADHD state detection as behavioral events flow in.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::{
    sync::{mpsc, RwLock},
    time::{interval, Instant},
};
use uuid::Uuid;

use skelly_jelly_event_bus::{
    EventBusTrait, EventHandler, EventHandlerResult, Message, MessageHandler, ModuleId,
};
use skelly_jelly_storage::types::{EventBatch, RawEvent};

use crate::{
    error::{AnalysisError, AnalysisResult},
    inference::{InferenceEngine, InferencePriority},
    sliding_window::{AnalysisWindow, SlidingWindowManager},
    state_detection::{StateDetectionEngine, StateDetectionResult},
    types::AnalysisResult as AnalysisResultType,
};

/// Real-time event bus integration for ADHD state detection
pub struct EventBusIntegration {
    /// Event bus instance
    event_bus: Arc<dyn EventBusTrait>,
    
    /// State detection engine
    state_detector: Arc<StateDetectionEngine>,
    
    /// Inference engine for high-performance processing
    inference_engine: Arc<InferenceEngine>,
    
    /// Sliding window manager for event aggregation
    window_manager: Arc<RwLock<SlidingWindowManager>>,
    
    /// Configuration
    config: EventBusConfig,
    
    /// Module identifier
    module_id: ModuleId,
    
    /// Event processing metrics
    metrics: Arc<RwLock<EventProcessingMetrics>>,
    
    /// Analysis result sender
    result_sender: Arc<Mutex<Option<mpsc::UnboundedSender<AnalysisResultType>>>>,
    
    /// Current processing status
    processing_status: Arc<RwLock<ProcessingStatus>>,
}

/// Configuration for event bus integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventBusConfig {
    /// Window size for analysis (seconds)
    pub window_size_secs: u64,
    
    /// Window overlap (seconds)
    pub window_overlap_secs: u64,
    
    /// Maximum events per window
    pub max_events_per_window: usize,
    
    /// Analysis frequency (windows per minute)
    pub analysis_frequency: u32,
    
    /// Event buffer size
    pub event_buffer_size: usize,
    
    /// Enable real-time processing
    pub enable_realtime_processing: bool,
    
    /// Processing priority
    pub processing_priority: InferencePriority,
    
    /// Event types to process
    pub event_types: Vec<String>,
    
    /// Performance monitoring
    pub enable_metrics: bool,
    pub metrics_update_interval_secs: u64,
}

impl Default for EventBusConfig {
    fn default() -> Self {
        Self {
            window_size_secs: 30,
            window_overlap_secs: 5,
            max_events_per_window: 1000,
            analysis_frequency: 2, // Every 30 seconds
            event_buffer_size: 10000,
            enable_realtime_processing: true,
            processing_priority: InferencePriority::Normal,
            event_types: vec![
                "keystroke".to_string(),
                "mouse_move".to_string(),
                "mouse_click".to_string(),
                "window_focus".to_string(),
                "resource_usage".to_string(),
            ],
            enable_metrics: true,
            metrics_update_interval_secs: 30,
        }
    }
}

/// Event processing metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventProcessingMetrics {
    pub total_events_processed: u64,
    pub total_windows_analyzed: u64,
    pub total_analysis_results: u64,
    pub avg_processing_time_ms: f32,
    pub avg_window_size: f32,
    pub event_type_counts: HashMap<String, u64>,
    pub processing_errors: u64,
    pub last_analysis_time: Option<DateTime<Utc>>,
    pub throughput_events_per_sec: f32,
    pub throughput_analyses_per_min: f32,
}

impl Default for EventProcessingMetrics {
    fn default() -> Self {
        Self {
            total_events_processed: 0,
            total_windows_analyzed: 0,
            total_analysis_results: 0,
            avg_processing_time_ms: 0.0,
            avg_window_size: 0.0,
            event_type_counts: HashMap::new(),
            processing_errors: 0,
            last_analysis_time: None,
            throughput_events_per_sec: 0.0,
            throughput_analyses_per_min: 0.0,
        }
    }
}

/// Current processing status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingStatus {
    pub is_active: bool,
    pub current_window_count: usize,
    pub buffer_utilization: f32,
    pub last_error: Option<String>,
    pub uptime_seconds: u64,
    pub current_analysis_load: f32,
}

impl Default for ProcessingStatus {
    fn default() -> Self {
        Self {
            is_active: false,
            current_window_count: 0,
            buffer_utilization: 0.0,
            last_error: None,
            uptime_seconds: 0,
            current_analysis_load: 0.0,
        }
    }
}

impl EventBusIntegration {
    /// Create a new event bus integration
    pub async fn new(
        event_bus: Arc<dyn EventBusTrait>,
        state_detector: Arc<StateDetectionEngine>,
        inference_engine: Arc<InferenceEngine>,
    ) -> AnalysisResult<Self> {
        Self::with_config(event_bus, state_detector, inference_engine, EventBusConfig::default()).await
    }

    /// Create with custom configuration
    pub async fn with_config(
        event_bus: Arc<dyn EventBusTrait>,
        state_detector: Arc<StateDetectionEngine>,
        inference_engine: Arc<InferenceEngine>,
        config: EventBusConfig,
    ) -> AnalysisResult<Self> {
        let module_id = ModuleId::new("analysis-engine");
        let window_duration = Duration::from_secs(config.window_size_secs);
        let window_overlap = Duration::from_secs(config.window_overlap_secs);

        let integration = Self {
            event_bus,
            state_detector,
            inference_engine,
            window_manager: Arc::new(RwLock::new(
                SlidingWindowManager::new(window_duration, window_overlap)
            )),
            config,
            module_id,
            metrics: Arc::new(RwLock::new(EventProcessingMetrics::default())),
            result_sender: Arc::new(Mutex::new(None)),
            processing_status: Arc::new(RwLock::new(ProcessingStatus::default())),
        };

        Ok(integration)
    }

    /// Start real-time event processing
    pub async fn start_processing(&self) -> AnalysisResult<mpsc::UnboundedReceiver<AnalysisResultType>> {
        println!("Starting real-time event processing...");

        // Create result channel
        let (result_tx, result_rx) = mpsc::unbounded_channel();
        {
            let mut sender = self.result_sender.lock().map_err(|_| {
                AnalysisError::ConcurrencyError {
                    operation: "result_sender_init".to_string(),
                }
            })?;
            *sender = Some(result_tx);
        }

        // Register event handlers
        self.register_event_handlers().await?;

        // Start analysis timer
        self.start_analysis_timer().await?;

        // Start metrics collection
        if self.config.enable_metrics {
            self.start_metrics_collection().await?;
        }

        // Update status
        {
            let mut status = self.processing_status.write().await;
            status.is_active = true;
        }

        println!("Real-time event processing started successfully!");
        Ok(result_rx)
    }

    /// Register event handlers with the event bus
    async fn register_event_handlers(&self) -> AnalysisResult<()> {
        println!("Registering event handlers...");

        // Create message handler for behavioral events
        let handler = Arc::new(BehavioralEventHandler::new(
            Arc::clone(&self.window_manager),
            Arc::clone(&self.metrics),
            self.config.clone(),
        ));

        // Register for each event type
        for event_type in &self.config.event_types {
            self.event_bus.subscribe(event_type, handler.clone()).await
                .map_err(|e| AnalysisError::EventBusError {
                    message: format!("Failed to subscribe to '{}': {}", event_type, e),
                })?;
        }

        println!("Event handlers registered for {} event types", self.config.event_types.len());
        Ok(())
    }

    /// Start periodic analysis timer
    async fn start_analysis_timer(&self) -> AnalysisResult<()> {
        let window_manager = Arc::clone(&self.window_manager);
        let state_detector = Arc::clone(&self.state_detector);
        let inference_engine = Arc::clone(&self.inference_engine);
        let result_sender = Arc::clone(&self.result_sender);
        let metrics = Arc::clone(&self.metrics);
        let processing_status = Arc::clone(&self.processing_status);
        let config = self.config.clone();

        tokio::spawn(async move {
            let analysis_interval = Duration::from_secs(60 / config.analysis_frequency as u64);
            let mut timer = interval(analysis_interval);

            loop {
                timer.tick().await;

                // Check if processing is active
                {
                    let status = processing_status.read().await;
                    if !status.is_active {
                        continue;
                    }
                }

                // Get current windows for analysis
                let windows = {
                    let mut manager = window_manager.write().await;
                    manager.get_completed_windows()
                };

                if windows.is_empty() {
                    continue;
                }

                println!("Analyzing {} windows...", windows.len());

                // Process each window
                for window in windows {
                    match Self::analyze_window_internal(
                        &window,
                        &state_detector,
                        &inference_engine,
                        config.processing_priority,
                    ).await {
                        Ok(result) => {
                            // Update metrics
                            Self::update_analysis_metrics(&metrics, &window, true).await;

                            // Send result
                            if let Ok(sender_guard) = result_sender.lock() {
                                if let Some(sender) = sender_guard.as_ref() {
                                    if let Err(e) = sender.send(result) {
                                        eprintln!("Failed to send analysis result: {}", e);
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Analysis failed for window {}: {}", window.window_id, e);
                            Self::update_analysis_metrics(&metrics, &window, false).await;
                        }
                    }
                }
            }
        });

        println!("Analysis timer started with {}s intervals", 60 / self.config.analysis_frequency);
        Ok(())
    }

    /// Analyze a single window
    async fn analyze_window_internal(
        window: &AnalysisWindow,
        state_detector: &StateDetectionEngine,
        inference_engine: &InferenceEngine,
        priority: InferencePriority,
    ) -> AnalysisResult<AnalysisResultType> {
        let start_time = Instant::now();

        // Perform state detection with priority
        let detection_result = inference_engine
            .infer_with_priority(window, priority)
            .await?;

        // Create analysis result
        let analysis_result = AnalysisResultType {
            window_id: window.window_id,
            timestamp: std::time::SystemTime::now(),
            state: detection_result.detected_state,
            confidence: detection_result.confidence,
            metrics: crate::metrics::BehavioralMetrics::default(), // Would be computed from features
            work_context: None, // Would come from screenshot analysis
            intervention_readiness: detection_result.intervention_readiness,
            processing_time_ms: start_time.elapsed().as_millis() as u32,
            feature_importance: detection_result.feature_importance,
        };

        Ok(analysis_result)
    }

    /// Update analysis metrics
    async fn update_analysis_metrics(
        metrics: &Arc<RwLock<EventProcessingMetrics>>,
        window: &AnalysisWindow,
        success: bool,
    ) {
        let mut metrics_guard = metrics.write().await;
        
        metrics_guard.total_windows_analyzed += 1;
        if success {
            metrics_guard.total_analysis_results += 1;
        } else {
            metrics_guard.processing_errors += 1;
        }

        // Update window size average
        let window_size = window.events.len() as f32;
        let n = metrics_guard.total_windows_analyzed as f32;
        metrics_guard.avg_window_size = (metrics_guard.avg_window_size * (n - 1.0) + window_size) / n;

        metrics_guard.last_analysis_time = Some(Utc::now());
    }

    /// Start metrics collection
    async fn start_metrics_collection(&self) -> AnalysisResult<()> {
        let metrics = Arc::clone(&self.metrics);
        let processing_status = Arc::clone(&self.processing_status);
        let update_interval = Duration::from_secs(self.config.metrics_update_interval_secs);

        tokio::spawn(async move {
            let mut timer = interval(update_interval);
            let start_time = Instant::now();

            loop {
                timer.tick().await;

                let uptime = start_time.elapsed().as_secs();
                
                // Update processing status
                {
                    let mut status = processing_status.write().await;
                    status.uptime_seconds = uptime;
                    
                    let metrics_guard = metrics.read().await;
                    if uptime > 0 {
                        status.current_analysis_load = metrics_guard.total_windows_analyzed as f32 / uptime as f32;
                    }
                }

                // Update throughput metrics
                {
                    let mut metrics_guard = metrics.write().await;
                    if uptime > 0 {
                        metrics_guard.throughput_events_per_sec = 
                            metrics_guard.total_events_processed as f32 / uptime as f32;
                        metrics_guard.throughput_analyses_per_min = 
                            metrics_guard.total_windows_analyzed as f32 / (uptime as f32 / 60.0);
                    }
                }
            }
        });

        println!("Metrics collection started with {}s intervals", self.config.metrics_update_interval_secs);
        Ok(())
    }

    /// Stop processing
    pub async fn stop_processing(&self) -> AnalysisResult<()> {
        println!("Stopping event processing...");

        {
            let mut status = self.processing_status.write().await;
            status.is_active = false;
        }

        // Unregister event handlers
        for event_type in &self.config.event_types {
            if let Err(e) = self.event_bus.unsubscribe(event_type, &self.module_id).await {
                eprintln!("Warning: Failed to unsubscribe from '{}': {}", event_type, e);
            }
        }

        println!("Event processing stopped successfully!");
        Ok(())
    }

    /// Get current metrics
    pub async fn get_metrics(&self) -> EventProcessingMetrics {
        self.metrics.read().await.clone()
    }

    /// Get current processing status
    pub async fn get_status(&self) -> ProcessingStatus {
        self.processing_status.read().await.clone()
    }

    /// Process a batch of events directly (for testing/offline processing)
    pub async fn process_event_batch(&self, batch: EventBatch) -> AnalysisResult<Vec<AnalysisResultType>> {
        let mut results = Vec::new();

        // Add events to window manager
        {
            let mut manager = self.window_manager.write().await;
            for event in batch.events {
                manager.add_event(event);
            }
        }

        // Get completed windows
        let windows = {
            let mut manager = self.window_manager.write().await;
            manager.get_completed_windows()
        };

        // Analyze each window
        for window in windows {
            match Self::analyze_window_internal(
                &window,
                &self.state_detector,
                &self.inference_engine,
                self.config.processing_priority,
            ).await {
                Ok(result) => {
                    Self::update_analysis_metrics(&self.metrics, &window, true).await;
                    results.push(result);
                }
                Err(e) => {
                    Self::update_analysis_metrics(&self.metrics, &window, false).await;
                    return Err(e);
                }
            }
        }

        Ok(results)
    }
}

/// Message handler for behavioral events
pub struct BehavioralEventHandler {
    window_manager: Arc<RwLock<SlidingWindowManager>>,
    metrics: Arc<RwLock<EventProcessingMetrics>>,
    config: EventBusConfig,
}

impl BehavioralEventHandler {
    pub fn new(
        window_manager: Arc<RwLock<SlidingWindowManager>>,
        metrics: Arc<RwLock<EventProcessingMetrics>>,
        config: EventBusConfig,
    ) -> Self {
        Self {
            window_manager,
            metrics,
            config,
        }
    }
}

#[async_trait]
impl MessageHandler for BehavioralEventHandler {
    async fn handle_message(&self, message: Message) -> EventHandlerResult {
        // Parse behavioral event from message
        match self.parse_behavioral_event(&message).await {
            Ok(event) => {
                // Add event to window manager
                {
                    let mut manager = self.window_manager.write().await;
                    manager.add_event(event.clone());
                }

                // Update metrics
                {
                    let mut metrics = self.metrics.write().await;
                    metrics.total_events_processed += 1;
                    
                    let event_type = self.get_event_type_name(&event);
                    *metrics.event_type_counts.entry(event_type).or_insert(0) += 1;
                }

                Ok(())
            }
            Err(e) => {
                // Update error metrics
                {
                    let mut metrics = self.metrics.write().await;
                    metrics.processing_errors += 1;
                }

                eprintln!("Failed to parse behavioral event: {}", e);
                Err(format!("Event parsing failed: {}", e))
            }
        }
    }
}

impl BehavioralEventHandler {
    /// Parse behavioral event from message
    async fn parse_behavioral_event(&self, message: &Message) -> AnalysisResult<RawEvent> {
        // Parse the message payload based on event type
        // This would depend on the actual message format from the data capture module
        
        let event_data: serde_json::Value = serde_json::from_slice(&message.payload)
            .map_err(|e| AnalysisError::InvalidInput {
                message: format!("Failed to parse event JSON: {}", e),
            })?;

        // Convert to RawEvent based on event type
        match message.event_type.as_str() {
            "keystroke" => {
                let keystroke_event: skelly_jelly_storage::types::KeystrokeEvent = 
                    serde_json::from_value(event_data)
                        .map_err(|e| AnalysisError::InvalidInput {
                            message: format!("Failed to parse keystroke event: {}", e),
                        })?;
                Ok(RawEvent::Keystroke(keystroke_event))
            }
            "mouse_move" => {
                let mouse_event: skelly_jelly_storage::types::MouseMoveEvent = 
                    serde_json::from_value(event_data)
                        .map_err(|e| AnalysisError::InvalidInput {
                            message: format!("Failed to parse mouse move event: {}", e),
                        })?;
                Ok(RawEvent::MouseMove(mouse_event))
            }
            "mouse_click" => {
                let mouse_event: skelly_jelly_storage::types::MouseClickEvent = 
                    serde_json::from_value(event_data)
                        .map_err(|e| AnalysisError::InvalidInput {
                            message: format!("Failed to parse mouse click event: {}", e),
                        })?;
                Ok(RawEvent::MouseClick(mouse_event))
            }
            "window_focus" => {
                let window_event: skelly_jelly_storage::types::WindowFocusEvent = 
                    serde_json::from_value(event_data)
                        .map_err(|e| AnalysisError::InvalidInput {
                            message: format!("Failed to parse window focus event: {}", e),
                        })?;
                Ok(RawEvent::WindowFocus(window_event))
            }
            "resource_usage" => {
                let resource_event: skelly_jelly_storage::types::ResourceUsageEvent = 
                    serde_json::from_value(event_data)
                        .map_err(|e| AnalysisError::InvalidInput {
                            message: format!("Failed to parse resource usage event: {}", e),
                        })?;
                Ok(RawEvent::ResourceUsage(resource_event))
            }
            _ => Err(AnalysisError::InvalidInput {
                message: format!("Unknown event type: {}", message.event_type),
            })
        }
    }

    /// Get event type name for metrics
    fn get_event_type_name(&self, event: &RawEvent) -> String {
        match event {
            RawEvent::Keystroke(_) => "keystroke".to_string(),
            RawEvent::MouseMove(_) => "mouse_move".to_string(),
            RawEvent::MouseClick(_) => "mouse_click".to_string(),
            RawEvent::WindowFocus(_) => "window_focus".to_string(),
            RawEvent::ResourceUsage(_) => "resource_usage".to_string(),
            _ => "unknown".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state_detection::StateDetectionEngine;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_event_bus_integration_creation() {
        // This test would require mock implementations of the event bus
        // For now, we'll test the configuration
        let config = EventBusConfig::default();
        
        assert_eq!(config.window_size_secs, 30);
        assert_eq!(config.analysis_frequency, 2);
        assert!(config.enable_realtime_processing);
        assert_eq!(config.event_types.len(), 5);
    }

    #[test]
    fn test_processing_metrics() {
        let metrics = EventProcessingMetrics::default();
        
        assert_eq!(metrics.total_events_processed, 0);
        assert_eq!(metrics.total_windows_analyzed, 0);
        assert_eq!(metrics.processing_errors, 0);
    }

    #[test]
    fn test_processing_status() {
        let status = ProcessingStatus::default();
        
        assert!(!status.is_active);
        assert_eq!(status.current_window_count, 0);
        assert_eq!(status.uptime_seconds, 0);
    }
}