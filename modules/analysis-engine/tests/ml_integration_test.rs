//! Integration tests for ML pipeline components
//!
//! Tests the complete ML workflow from feature extraction through
//! state detection, inference, and online learning.

use chrono::Utc;
use futures::future;
use skelly_jelly_analysis_engine::{
    AnalysisWindow, StateDetectionEngine, InferenceEngine, OnlineLearningEngine,
    FeatureExtractionPipeline, RandomForestClassifier, StateDetectionConfig,
    InferenceConfig, OnlineLearningConfig, FeatureVector, ADHDState, ADHDStateType,
    FlowDepth, DistractionType, InferencePriority, UserFeedback,
    models::get_adhd_state_type,
};
use skelly_jelly_storage::types::{RawEvent, KeystrokeEvent, KeyModifiers, MouseEvent, 
                                WindowEvent, ResourceEvent};
use std::{sync::{Arc, Mutex}, time::{SystemTime, Duration}};
use uuid::Uuid;

/// Test the complete ML pipeline end-to-end
#[tokio::test]
async fn test_complete_ml_pipeline() {
    // Create test training data
    let training_data = create_training_data();
    
    // Initialize state detection engine
    let mut state_detector = StateDetectionEngine::new();
    
    // Train the model
    let train_result = state_detector.train(&training_data).await;
    assert!(train_result.is_ok(), "Training should succeed with sufficient data");
    
    // Test inference
    let test_window = create_test_window_with_flow_pattern();
    let detection_result = state_detector.detect_state(&test_window).await;
    
    assert!(detection_result.is_ok(), "State detection should succeed after training");
    
    let result = detection_result.unwrap();
    assert!(result.confidence > 0.0, "Should have non-zero confidence");
    assert!(result.processing_time_ms < 50.0, "Should meet latency requirement");
    assert!(!result.feature_importance.is_empty(), "Should provide feature importance");
}

/// Test inference engine with caching and performance optimization
#[tokio::test]
async fn test_inference_engine_optimization() {
    let training_data = create_training_data();
    let state_detector = Arc::new(StateDetectionEngine::new());
    
    // Train the underlying model
    state_detector.train(&training_data).await.unwrap();
    
    // Create inference engine with caching enabled
    let config = InferenceConfig {
        enable_caching: true,
        cache_max_size: 100,
        max_concurrent_inferences: 5,
        ..Default::default()
    };
    
    let inference_engine = InferenceEngine::with_config(state_detector, config);
    
    // Test single inference
    let test_window = create_test_window_with_distracted_pattern();
    let result1 = inference_engine.infer(&test_window).await;
    assert!(result1.is_ok(), "First inference should succeed");
    
    // Test cached inference (should be faster)
    let start_time = std::time::Instant::now();
    let result2 = inference_engine.infer(&test_window).await;
    let cached_time = start_time.elapsed().as_millis() as f32;
    
    assert!(result2.is_ok(), "Cached inference should succeed");
    assert!(cached_time < 10.0, "Cached inference should be very fast");
    
    // Test priority inference
    let priority_result = inference_engine.infer_with_priority(&test_window, InferencePriority::High).await;
    assert!(priority_result.is_ok(), "Priority inference should succeed");
    
    // Check metrics
    let metrics = inference_engine.get_metrics().await;
    assert!(metrics.total_requests >= 3, "Should track all requests");
    assert!(metrics.cache_hits >= 1, "Should have cache hits");
}

/// Test online learning with user feedback
#[tokio::test]
async fn test_online_learning_system() {
    let classifier = Arc::new(Mutex::new(RandomForestClassifier::new()));
    let training_data = create_training_data();
    
    // Train initial model
    {
        let mut clf = classifier.lock().unwrap();
        clf.train(&training_data).await.unwrap();
    }
    
    let online_learner = OnlineLearningEngine::new(classifier);
    
    // Simulate user feedback
    let feedback = UserFeedback {
        window_id: Uuid::new_v4(),
        user_state: "flow".to_string(),
        confidence: 0.9,
        timestamp: Utc::now(),
        notes: Some("Deep focus coding session".to_string()),
    };
    
    let feedback_result = online_learner.process_feedback(feedback).await;
    assert!(feedback_result.is_ok(), "Feedback processing should succeed");
    
    // Check metrics
    let metrics = online_learner.get_metrics().await;
    assert_eq!(metrics.total_feedback_received, 1, "Should track feedback");
    
    // Test active learning queries
    let queries = online_learner.get_active_learning_queries(5).await;
    assert!(queries.is_ok(), "Should generate active learning queries");
}

/// Test feature extraction pipeline performance
#[tokio::test]
async fn test_feature_extraction_performance() {
    let pipeline = FeatureExtractionPipeline::new();
    
    // Create window with various event types
    let test_window = create_complex_test_window();
    
    let start_time = std::time::Instant::now();
    let features_result = pipeline.extract_all_features(&test_window).await;
    let extraction_time = start_time.elapsed().as_millis() as f32;
    
    assert!(features_result.is_ok(), "Feature extraction should succeed");
    assert!(extraction_time < 10.0, "Feature extraction should be fast");
    
    let features = features_result.unwrap();
    assert!(features.validate(), "Features should be valid");
    assert_eq!(features.feature_count(), 45, "Should extract 45 features total");
    
    // Test feature names consistency
    let feature_names = pipeline.get_all_feature_names();
    assert_eq!(feature_names.len(), pipeline.total_feature_count());
}

/// Test state detection accuracy with synthetic data
#[tokio::test]
async fn test_state_detection_accuracy() {
    let mut state_detector = StateDetectionEngine::new();
    
    // Create balanced training dataset
    let training_data = create_balanced_training_data();
    assert!(training_data.len() >= 500, "Need sufficient training data");
    
    // Train model
    state_detector.train(&training_data).await.unwrap();
    
    // Test accuracy on different state patterns
    let test_cases = vec![
        (create_test_window_with_flow_pattern(), ADHDStateType::Flow),
        (create_test_window_with_distracted_pattern(), ADHDStateType::Distracted),
        (create_test_window_with_hyperfocus_pattern(), ADHDStateType::Hyperfocus),
        (create_test_window_with_neutral_pattern(), ADHDStateType::Neutral),
    ];
    
    let mut correct_predictions = 0;
    for (window, expected_state) in test_cases {
        let result = state_detector.detect_state(&window).await.unwrap();
        if get_adhd_state_type(&result.detected_state) == expected_state {
            correct_predictions += 1;
        }
        
        // Check confidence and stability
        assert!(result.confidence > 0.0, "Should have confidence score");
        assert!(result.temporal_stability >= 0.0, "Should have stability score");
        assert!(result.intervention_readiness >= 0.0, "Should have readiness score");
    }
    
    let accuracy = correct_predictions as f32 / 4.0;
    println!("Test accuracy: {:.3}", accuracy);
    
    // Check model accuracy
    let model_accuracy = state_detector.get_accuracy().await;
    assert!(model_accuracy >= 0.8, "Model should achieve >80% accuracy requirement");
}

/// Test temporal smoothing and state transition stability
#[tokio::test]
async fn test_temporal_smoothing() {
    let config = StateDetectionConfig {
        temporal_smoothing_alpha: 0.3, // Heavy smoothing
        temporal_window_size: 5,
        ..Default::default()
    };
    
    let mut state_detector = StateDetectionEngine::with_config(config);
    let training_data = create_training_data();
    state_detector.train(&training_data).await.unwrap();
    
    // Simulate sequence of state detections
    let windows = vec![
        create_test_window_with_flow_pattern(),
        create_test_window_with_flow_pattern(),
        create_test_window_with_distracted_pattern(), // Transition
        create_test_window_with_flow_pattern(),
        create_test_window_with_flow_pattern(),
    ];
    
    let mut previous_state = None;
    let mut stable_transitions = 0;
    
    for window in windows {
        let result = state_detector.detect_state(&window).await.unwrap();
        
        if let Some(prev) = previous_state {
            // Check if transition is smooth (not too abrupt)
            if result.detected_state.state_type == prev || result.temporal_stability > 0.5 {
                stable_transitions += 1;
            }
        }
        
        previous_state = Some(get_adhd_state_type(&result.detected_state));
        
        // Temporal stability should improve with history
        assert!(result.temporal_stability >= 0.0);
    }
    
    assert!(stable_transitions >= 3, "Most transitions should be stable");
}

/// Test concurrent inference performance
#[tokio::test]
async fn test_concurrent_inference() {
    let training_data = create_training_data();
    let state_detector = Arc::new(StateDetectionEngine::new());
    state_detector.train(&training_data).await.unwrap();
    
    let inference_engine = Arc::new(InferenceEngine::new(state_detector));
    
    // Create multiple test windows
    let test_windows: Vec<_> = (0..20).map(|_| create_test_window_with_flow_pattern()).collect();
    
    // Test concurrent inference
    let start_time = std::time::Instant::now();
    let futures: Vec<_> = test_windows.iter()
        .map(|window| {
            let engine = Arc::clone(&inference_engine);
            let window = window.clone();
            tokio::spawn(async move {
                engine.infer(&window).await
            })
        })
        .collect();
    
    let results = futures::future::join_all(futures).await;
    let total_time = start_time.elapsed().as_millis() as f32;
    
    // Check that all inferences succeeded
    let successful_results = results.into_iter()
        .filter_map(|r| r.ok())
        .filter(|r| r.is_ok())
        .count();
    
    assert_eq!(successful_results, 20, "All concurrent inferences should succeed");
    
    // Average time per inference should be reasonable
    let avg_time_per_inference = total_time / 20.0;
    assert!(avg_time_per_inference < 100.0, "Average inference time should be reasonable with concurrency");
    
    // Check metrics
    let metrics = inference_engine.get_metrics().await;
    assert_eq!(metrics.total_requests, 20, "Should track all concurrent requests");
}

// Helper functions to create test data

fn create_training_data() -> Vec<(FeatureVector, ADHDState)> {
    let mut training_data = Vec::new();
    
    // Create 100 samples for each state (500 total)
    for _ in 0..100 {
        // Flow state samples
        let flow_features = create_flow_features();
        let flow_state = ADHDState::flow(FlowDepth::Deep, 0.9);
        training_data.push((flow_features, flow_state));
        
        // Distracted state samples
        let distracted_features = create_distracted_features();
        let distracted_state = ADHDState::distracted(DistractionType::TaskSwitching, 0.8, 0.8);
        training_data.push((distracted_features, distracted_state));
        
        // Neutral state samples
        let neutral_features = create_neutral_features();
        let neutral_state = ADHDState::neutral();
        training_data.push((neutral_features, neutral_state));
        
        // Hyperfocus samples (we need to create a helper function)
        let hyper_features = create_hyperfocus_features();
        let hyper_state = create_hyperfocus_state(0.95);
        training_data.push((hyper_features, hyper_state));
        
        // Transitioning samples (we need to create a helper function)
        let trans_features = create_transitioning_features();
        let trans_state = create_transitioning_state(0.7);
        training_data.push((trans_features, trans_state));
    }
    
    training_data
}

fn create_balanced_training_data() -> Vec<(FeatureVector, ADHDState)> {
    create_training_data() // Using same function for now
}

fn create_flow_features() -> FeatureVector {
    FeatureVector {
        keystroke_features: [0.15, 0.05, 0.3, 0.8, 0.2, 0.1, 0.7, 0.6, 0.05, 0.9], // Consistent rhythm
        mouse_features: [0.3, 0.1, 0.8, 0.2, 0.1, 0.9, 0.7, 0.1], // Smooth movements
        window_features: [0.9, 0.1, 0.95, 0.1, 0.05, 0.9], // Stable focus
        temporal_features: [0.6, 0.8, 0.7, 0.2, 0.3], // Consistent activity
        resource_features: [0.4, 0.3, 0.2, 0.1], // Moderate resource usage
        screenshot_features: None,
    }
}

fn create_distracted_features() -> FeatureVector {
    FeatureVector {
        keystroke_features: [0.3, 0.4, 0.8, 0.2, 0.7, 0.6, 0.3, 0.4, 0.3, 0.2], // Erratic typing
        mouse_features: [0.8, 0.6, 0.3, 0.9, 0.3, 0.4, 0.2, 0.6], // Frequent clicks, jerky movement
        window_features: [0.2, 0.8, 0.1, 0.9, 0.8, 0.2], // Frequent switching
        temporal_features: [0.9, 0.2, 0.8, 0.9, 0.7], // Burst activity
        resource_features: [0.3, 0.4, 0.3, 0.2], // Variable usage
        screenshot_features: None,
    }
}

fn create_neutral_features() -> FeatureVector {
    FeatureVector {
        keystroke_features: [0.5; 10], // Average patterns
        mouse_features: [0.5; 8], // Average patterns
        window_features: [0.5; 6], // Average patterns
        temporal_features: [0.5; 5], // Average patterns
        resource_features: [0.3, 0.3, 0.2, 0.1], // Low usage
        screenshot_features: None,
    }
}

fn create_hyperfocus_features() -> FeatureVector {
    FeatureVector {
        keystroke_features: [0.12, 0.02, 0.15, 0.95, 0.05, 0.02, 0.9, 0.95, 0.01, 0.98], // Ultra consistent
        mouse_features: [0.1, 0.02, 0.95, 0.05, 0.01, 0.98, 0.9, 0.02], // Very precise
        window_features: [0.98, 0.02, 0.99, 0.01, 0.01, 0.98], // Extremely stable
        temporal_features: [0.8, 0.95, 0.9, 0.05, 0.2], // Very consistent
        resource_features: [0.6, 0.5, 0.3, 0.2], // High sustained usage
        screenshot_features: None,
    }
}

fn create_transitioning_features() -> FeatureVector {
    FeatureVector {
        keystroke_features: [0.4, 0.3, 0.6, 0.4, 0.5, 0.4, 0.5, 0.4, 0.2, 0.4], // Variable patterns
        mouse_features: [0.6, 0.4, 0.5, 0.6, 0.3, 0.5, 0.4, 0.4], // Mixed behavior
        window_features: [0.4, 0.6, 0.3, 0.6, 0.5, 0.4], // Some switching
        temporal_features: [0.7, 0.4, 0.6, 0.6, 0.5], // Variable activity
        resource_features: [0.4, 0.4, 0.3, 0.2], // Changing usage
        screenshot_features: None,
    }
}

fn create_test_window_with_flow_pattern() -> AnalysisWindow {
    let mut window = AnalysisWindow::new(SystemTime::now());
    
    // Add consistent keystroke pattern
    let base_time = Utc::now();
    for i in 0..30 {
        let event = RawEvent::Keystroke(KeystrokeEvent {
            timestamp: base_time + chrono::Duration::milliseconds(i * 150),
            key_code: 65 + (i % 26),
            modifiers: KeyModifiers::default(),
            inter_key_interval_ms: Some(150), // Consistent timing
        });
        window.add_event(event);
    }
    
    // Add minimal window switching
    let window_event = RawEvent::Window(WindowEvent {
        timestamp: base_time,
        window_id: "editor".to_string(),
        window_title: "VS Code".to_string(),
        app_name: "code".to_string(),
        event_type: skelly_jelly_storage::types::WindowEventType::Focus,
        focus_duration_ms: Some(30000), // Long focus
    });
    window.add_event(window_event);
    
    window
}

fn create_test_window_with_distracted_pattern() -> AnalysisWindow {
    let mut window = AnalysisWindow::new(SystemTime::now());
    
    // Add erratic keystroke pattern
    let base_time = Utc::now();
    for i in 0..20 {
        let interval = if i % 3 == 0 { 50 } else { 400 }; // Erratic timing
        let event = RawEvent::Keystroke(KeystrokeEvent {
            timestamp: base_time + chrono::Duration::milliseconds(i * interval),
            key_code: 65 + (i % 26),
            modifiers: KeyModifiers::default(),
            inter_key_interval_ms: Some(interval),
        });
        window.add_event(event);
    }
    
    // Add frequent window switching
    let apps = ["browser", "slack", "email", "editor"];
    for (i, app) in apps.iter().enumerate() {
        let window_event = RawEvent::Window(WindowEvent {
            timestamp: base_time + chrono::Duration::seconds(i as i64 * 5),
            window_id: format!("{}_window", app),
            window_title: format!("{} Window", app),
            app_name: app.to_string(),
            event_type: skelly_jelly_storage::types::WindowEventType::Focus,
            focus_duration_ms: Some(2000), // Short focus durations
        });
        window.add_event(window_event);
    }
    
    window
}

fn create_test_window_with_hyperfocus_pattern() -> AnalysisWindow {
    let mut window = AnalysisWindow::new(SystemTime::now());
    
    // Add ultra-consistent keystroke pattern
    let base_time = Utc::now();
    for i in 0..50 {
        let event = RawEvent::Keystroke(KeystrokeEvent {
            timestamp: base_time + chrono::Duration::milliseconds(i * 120),
            key_code: 65 + (i % 26),
            modifiers: KeyModifiers::default(),
            inter_key_interval_ms: Some(120), // Very consistent
        });
        window.add_event(event);
    }
    
    // Single long-focus window
    let window_event = RawEvent::Window(WindowEvent {
        timestamp: base_time,
        window_id: "editor".to_string(),
        window_title: "Deep Work Session".to_string(),
        app_name: "code".to_string(),
        event_type: skelly_jelly_storage::types::WindowEventType::Focus,
        focus_duration_ms: Some(60000), // Very long focus
    });
    window.add_event(window_event);
    
    window
}

fn create_test_window_with_neutral_pattern() -> AnalysisWindow {
    let mut window = AnalysisWindow::new(SystemTime::now());
    
    // Add minimal activity
    let base_time = Utc::now();
    for i in 0..5 {
        let event = RawEvent::Keystroke(KeystrokeEvent {
            timestamp: base_time + chrono::Duration::seconds(i * 6),
            key_code: 65 + i,
            modifiers: KeyModifiers::default(),
            inter_key_interval_ms: Some(6000), // Sparse activity
        });
        window.add_event(event);
    }
    
    window
}

fn create_complex_test_window() -> AnalysisWindow {
    let mut window = AnalysisWindow::new(SystemTime::now());
    let base_time = Utc::now();
    
    // Add keystrokes
    for i in 0..20 {
        let event = RawEvent::Keystroke(KeystrokeEvent {
            timestamp: base_time + chrono::Duration::milliseconds(i * 200),
            key_code: 65 + (i % 26),
            modifiers: KeyModifiers::default(),
            inter_key_interval_ms: Some(200),
        });
        window.add_event(event);
    }
    
    // Add mouse events
    for i in 0..10 {
        let event = RawEvent::Mouse(MouseEvent {
            timestamp: base_time + chrono::Duration::milliseconds(i * 500),
            x: 100 + i * 10,
            y: 100 + i * 5,
            event_type: skelly_jelly_storage::types::MouseEventType::Move,
            button: None,
            scroll_delta: None,
            velocity: Some(i as f32 * 0.1),
        });
        window.add_event(event);
    }
    
    // Add window events
    let window_event = RawEvent::Window(WindowEvent {
        timestamp: base_time,
        window_id: "test_window".to_string(),
        window_title: "Test Application".to_string(),
        app_name: "test_app".to_string(),
        event_type: skelly_jelly_storage::types::WindowEventType::Focus,
        focus_duration_ms: Some(10000),
    });
    window.add_event(window_event);
    
    // Add resource events
    let resource_event = RawEvent::Resource(ResourceEvent {
        timestamp: base_time,
        process_name: "test_process".to_string(),
        cpu_percent: 25.5,
        memory_mb: 512,
        disk_io_mb_per_sec: 1.2,
        network_io_mb_per_sec: 0.5,
    });
    window.add_event(resource_event);
    
    window
}

// Helper functions for creating specific ADHD states

fn create_hyperfocus_state(confidence: f32) -> ADHDState {
    ADHDState::Hyperfocus {
        confidence,
        target: "Deep coding session".to_string(),
        intensity: 0.95,
        duration: Duration::from_secs(1800), // 30 minutes
        tunnel_vision_score: 0.9,
        external_awareness: 0.1,
    }
}

fn create_transitioning_state(confidence: f32) -> ADHDState {
    use skelly_jelly_analysis_engine::models::adhd_state::TransitionType;
    
    let from_state = Box::new(ADHDState::neutral());
    let to_state = Some(Box::new(ADHDState::flow(FlowDepth::Medium, 0.8)));
    
    ADHDState::Transitioning {
        from_state,
        to_state,
        progress: 0.5,
        transition_type: TransitionType::Natural,
        stability: confidence,
        estimated_completion: Duration::from_secs(60),
    }
}