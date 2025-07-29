# Analysis Engine Module

The intelligence core of Skelly-Jelly, processing behavioral data streams to detect ADHD states and work patterns in real-time using lightweight ML models.

## Overview

The Analysis Engine module provides:

- **Real-time ADHD State Detection**: Classifies user states (flow, hyperfocus, distracted, transitioning, neutral) with <50ms inference time
- **Behavioral Analytics**: Comprehensive metrics on productivity, focus, and work patterns
- **Feature Extraction**: Advanced analysis of keystroke dynamics, mouse patterns, and window behavior
- **Screenshot Analysis**: Work context extraction with privacy preservation
- **Online Learning**: Continuous adaptation to individual user patterns
- **Performance Optimization**: Efficient processing with memory-conscious design

## Architecture

### Core Components

- **EventProcessor**: Manages sliding windows and coordinates the analysis pipeline
- **FeatureExtraction**: Extracts behavioral features from raw event data
- **StateClassifier**: ML ensemble for ADHD state classification
- **MetricEngine**: Calculates comprehensive behavioral metrics
- **ScreenshotAnalyzer**: Extracts work context with privacy filtering

### Key Features

- **Sliding Window Analysis**: 30-second windows with 5-second overlap for smooth transitions
- **Multi-Model Ensemble**: Combines rule-based and ML models for robust classification
- **Privacy-First**: All processing happens locally with automatic PII filtering
- **Real-time Performance**: Target <50ms processing time per analysis window
- **Memory Efficient**: <100MB steady-state memory usage

## Usage

### Basic Usage

```rust
use skelly_jelly_analysis_engine::{create_analysis_engine, AnalysisEngineConfig};
use skelly_jelly_event_bus::EventBus;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create event bus
    let event_bus = Arc::new(EventBus::new());
    
    // Configure analysis engine
    let config = AnalysisEngineConfig::default();
    
    // Create analysis engine
    let engine = create_analysis_engine(config, event_bus).await?;
    
    // Get current state
    let state = engine.get_current_state().await;
    println!("Current ADHD state: {}", state.state_type());
    
    // Get behavioral metrics
    let metrics = engine.get_metrics().await;
    println!("Productivity score: {:.2}", metrics.productivity_score());
    
    Ok(())
}
```

### Processing Event Batches

```rust
use skelly_jelly_storage::types::EventBatch;

// Process a batch of events from storage
let result = engine.analyze_batch(event_batch).await?;

println!("State: {}", result.state.state_type());
println!("Confidence: {:.2}", result.confidence);
println!("Processing time: {}ms", result.processing_time_ms);
```

### User Feedback Integration

```rust
use skelly_jelly_analysis_engine::UserFeedback;
use chrono::Utc;

// Provide user feedback for online learning
let feedback = UserFeedback {
    window_id: result.window_id,
    user_state: "flow".to_string(),
    confidence: 0.9,
    timestamp: Utc::now(),
    notes: Some("Deep focus while coding".to_string()),
};

engine.process_feedback(feedback).await?;
```

## Configuration

### Analysis Engine Config

```rust
use std::time::Duration;

let config = AnalysisEngineConfig {
    window_size: Duration::from_secs(30),
    window_overlap: Duration::from_secs(5),
    state_confidence_threshold: 0.7,
    enable_online_learning: true,
    enable_screenshots: true,
    ..Default::default()
};
```

### Feature Extraction Config

```rust
use skelly_jelly_analysis_engine::feature_extraction::FeatureExtractionConfig;

let feature_config = FeatureExtractionConfig {
    enable_keystroke_features: true,
    enable_mouse_features: true,
    enable_window_features: true,
    normalize_features: true,
    ..Default::default()
};
```

## ADHD States

The engine classifies five primary ADHD states:

### Flow State
- **Characteristics**: Optimal focus and productivity
- **Indicators**: Consistent typing rhythm, stable window focus, low error rate
- **Intervention**: Minimal (preserve flow state)

### Hyperfocus State
- **Characteristics**: Intense concentration on specific task
- **Indicators**: High typing bursts, minimal window switching, reduced external awareness
- **Intervention**: Gentle reminders for breaks and awareness

### Distracted State
- **Characteristics**: Scattered attention, frequent interruptions
- **Indicators**: Rapid window switching, irregular typing patterns, high error rate
- **Intervention**: Blocking prompts, refocus assistance

### Transitioning State
- **Characteristics**: Moving between other states
- **Indicators**: Variable activity patterns, moderate window switching
- **Intervention**: Supportive guidance during transitions

### Neutral State
- **Characteristics**: Baseline cognitive state
- **Indicators**: Balanced activity levels, moderate focus stability
- **Intervention**: Focus initiation prompts when ready

## Behavioral Metrics

The engine calculates comprehensive behavioral metrics:

- **Activity Metrics**: Keystroke rate, mouse activity, window switching frequency
- **Focus Metrics**: Focus duration, depth score, distraction frequency
- **Pattern Metrics**: Work rhythm consistency, task switching index, cognitive load
- **Productivity Indicators**: Productive time ratio, flow state probability
- **Wellbeing Indicators**: Stress level, fatigue level, intervention receptivity

## Performance Characteristics

- **Inference Time**: <50ms target for real-time analysis
- **Memory Usage**: <100MB steady state
- **Feature Count**: 45 behavioral features across 6 categories
- **Model Size**: <20MB total for all models
- **Throughput**: 1000+ events/second processing capability

## Privacy and Security

- **Local Processing**: All analysis happens on-device
- **Screenshot Privacy**: Automatic PII detection and masking
- **Memory Scrubbing**: Sensitive data cleared after processing
- **No Data Transmission**: No behavioral data leaves the device

## Dependencies

- **Core**: tokio, async-trait, serde, uuid, chrono
- **ML**: smartcore, ndarray, statrs
- **Performance**: rayon, dashmap
- **Image Processing**: image, imageproc
- **Internal**: skelly-jelly-event-bus, skelly-jelly-storage

## Testing

Run the test suite:

```bash
cargo test
```

Run with features:

```bash
cargo test --features gpu,benchmark
```

## Development

### Adding New Features

1. Define feature extractor in `feature_extraction/`
2. Update `FeatureVector` with new dimensions
3. Modify classifiers to use new features
4. Add tests and documentation

### Adding New Models

1. Implement `StateModel` trait
2. Add to `StateClassifier` ensemble
3. Configure model weights
4. Test and validate performance

### Performance Tuning

1. Profile with `cargo bench`
2. Optimize hot paths in feature extraction
3. Tune model ensemble weights
4. Adjust window sizes for use case

## License

MIT