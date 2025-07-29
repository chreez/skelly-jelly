# Analysis Engine Module Design

## Module Purpose and Responsibilities

The Analysis Engine is the intelligence core of Skelly-Jelly, processing behavioral data streams to detect ADHD states and work patterns in real-time. It uses lightweight ML models optimized for local inference to identify flow states, distraction patterns, and optimal intervention moments.

### Core Responsibilities
- **Behavioral Analysis**: Process event batches to extract meaningful patterns
- **State Detection**: Classify ADHD states (flow, hyperfocus, distracted, transitioning)
- **Context Extraction**: Understand work context from screenshots and window data
- **Metric Calculation**: Compute behavioral metrics from raw events
- **Pattern Learning**: Continuously improve detection accuracy through online learning
- **Privacy Preservation**: All processing happens locally with no data leaving the device

## Key Components and Their Functions

### 1. Event Processor
```rust
pub struct EventProcessor {
    // Sliding window for analysis
    window_manager: SlidingWindowManager,
    
    // Feature extractors for different event types
    feature_extractors: HashMap<EventType, Box<dyn FeatureExtractor>>,
    
    // Metric calculators
    metric_engine: MetricEngine,
    
    // Screenshot analyzer
    screenshot_analyzer: ScreenshotAnalyzer,
}

pub struct SlidingWindowManager {
    // 30-second analysis windows
    current_window: AnalysisWindow,
    
    // Historical windows for trend analysis
    window_history: CircularBuffer<AnalysisWindow>,
    
    // Window overlap for smooth transitions
    overlap_duration: Duration, // 5 seconds
}

pub struct AnalysisWindow {
    pub window_id: Uuid,
    pub start_time: SystemTime,
    pub end_time: SystemTime,
    pub events: Vec<RawEvent>,
    pub extracted_features: FeatureVector,
    pub computed_metrics: BehavioralMetrics,
    pub screenshot_context: Option<ScreenshotContext>,
}
```

### 2. Feature Extraction Pipeline
```rust
pub trait FeatureExtractor: Send + Sync {
    fn extract(&self, events: &[RawEvent]) -> Result<Features>;
}

pub struct KeystrokeFeatureExtractor {
    // Inter-keystroke interval statistics
    iki_calculator: InterKeystrokeInterval,
    
    // Typing rhythm analysis
    rhythm_analyzer: RhythmAnalyzer,
    
    // Burst detection
    burst_detector: BurstDetector,
}

impl KeystrokeFeatureExtractor {
    pub fn extract_features(&self, keystrokes: &[KeystrokeEvent]) -> KeystrokeFeatures {
        KeystrokeFeatures {
            // Timing features
            mean_iki: self.iki_calculator.mean(keystrokes),
            iki_variance: self.iki_calculator.variance(keystrokes),
            iki_cv: self.iki_calculator.coefficient_of_variation(keystrokes),
            
            // Rhythm features
            typing_rhythm_score: self.rhythm_analyzer.analyze(keystrokes),
            pause_frequency: self.detect_pauses(keystrokes),
            
            // Burst features
            burst_count: self.burst_detector.count(keystrokes),
            mean_burst_length: self.burst_detector.mean_length(keystrokes),
            burst_intensity: self.burst_detector.intensity(keystrokes),
            
            // Error patterns
            backspace_rate: self.calculate_backspace_rate(keystrokes),
            correction_patterns: self.detect_corrections(keystrokes),
        }
    }
}

pub struct MouseFeatureExtractor {
    movement_analyzer: MouseMovementAnalyzer,
    click_pattern_detector: ClickPatternDetector,
}

pub struct WindowFeatureExtractor {
    switch_analyzer: WindowSwitchAnalyzer,
    focus_duration_tracker: FocusDurationTracker,
}
```

### 3. ML Model Architecture
```rust
pub struct ADHDStateClassifier {
    // Ensemble of models for robustness
    models: Vec<Box<dyn StateModel>>,
    
    // Model weights for ensemble
    model_weights: Vec<f32>,
    
    // Feature preprocessor
    preprocessor: FeaturePreprocessor,
    
    // Online learning component
    online_learner: OnlineLearner,
}

pub trait StateModel: Send + Sync {
    fn predict(&self, features: &FeatureVector) -> StateDistribution;
    fn update(&mut self, features: &FeatureVector, true_state: ADHDState);
}

pub struct RandomForestModel {
    forest: RandomForest,
    feature_importance: Vec<f32>,
}

pub struct LightweightNeuralNet {
    // Small MLP with 2 hidden layers
    network: Sequential,
    optimizer: AdamOptimizer,
}

pub enum ADHDState {
    Flow {
        confidence: f32,
        duration: Duration,
        depth: FlowDepth,
    },
    Hyperfocus {
        confidence: f32,
        target: String,
        intensity: f32,
    },
    Distracted {
        confidence: f32,
        distraction_type: DistractionType,
        severity: f32,
    },
    Transitioning {
        from_state: Box<ADHDState>,
        to_state: Box<ADHDState>,
        progress: f32,
    },
    Neutral {
        confidence: f32,
    },
}
```

### 4. Screenshot Context Analyzer
```rust
pub struct ScreenshotAnalyzer {
    // OCR for text extraction
    ocr_engine: OCREngine,
    
    // UI element detector
    ui_detector: UIElementDetector,
    
    // Work context classifier
    context_classifier: WorkContextClassifier,
    
    // Privacy mask applier
    privacy_filter: PrivacyFilter,
}

impl ScreenshotAnalyzer {
    pub async fn analyze(&self, screenshot: &Screenshot) -> Result<ScreenshotContext> {
        // Apply privacy filtering first
        let masked = self.privacy_filter.mask_sensitive_areas(screenshot)?;
        
        // Extract text with OCR
        let text_regions = self.ocr_engine.extract_text(&masked)?;
        
        // Detect UI elements
        let ui_elements = self.ui_detector.detect(&masked)?;
        
        // Classify work context
        let work_type = self.context_classifier.classify(&text_regions, &ui_elements)?;
        
        // Extract relevant features
        Ok(ScreenshotContext {
            work_type,
            text_density: self.calculate_text_density(&text_regions),
            ui_complexity: self.calculate_ui_complexity(&ui_elements),
            color_scheme: self.extract_color_scheme(&masked),
            activity_indicators: self.detect_activity_indicators(&ui_elements),
        })
    }
}

pub enum WorkType {
    Coding { language: String, framework: Option<String> },
    Writing { document_type: DocumentType },
    Design { tool: String, project_type: String },
    Research { topic_indicators: Vec<String> },
    Communication { platform: String },
    Entertainment { category: String },
    Unknown,
}
```

### 5. Behavioral Metrics Engine
```rust
pub struct MetricEngine {
    calculators: HashMap<MetricType, Box<dyn MetricCalculator>>,
}

pub struct BehavioralMetrics {
    // Activity metrics
    pub keystroke_rate: f32,
    pub mouse_activity_level: f32,
    pub window_switch_frequency: f32,
    
    // Focus metrics
    pub focus_duration: Duration,
    pub focus_depth_score: f32,
    pub distraction_frequency: f32,
    
    // Pattern metrics
    pub work_rhythm_consistency: f32,
    pub task_switching_index: f32,
    pub cognitive_load_estimate: f32,
    
    // Productivity indicators
    pub productive_time_ratio: f32,
    pub flow_state_probability: f32,
    pub intervention_receptivity: f32,
}

impl MetricEngine {
    pub fn calculate_all(&self, window: &AnalysisWindow) -> BehavioralMetrics {
        let mut metrics = BehavioralMetrics::default();
        
        // Parallel calculation of independent metrics
        rayon::scope(|s| {
            s.spawn(|_| {
                metrics.keystroke_rate = self.calculate_keystroke_rate(window);
            });
            s.spawn(|_| {
                metrics.mouse_activity_level = self.calculate_mouse_activity(window);
            });
            s.spawn(|_| {
                metrics.focus_duration = self.calculate_focus_duration(window);
            });
        });
        
        // Sequential calculation of dependent metrics
        metrics.cognitive_load_estimate = self.estimate_cognitive_load(&metrics);
        metrics.flow_state_probability = self.estimate_flow_probability(&metrics);
        
        metrics
    }
}
```

### 6. Online Learning System
```rust
pub struct OnlineLearner {
    // User feedback incorporation
    feedback_processor: FeedbackProcessor,
    
    // Model adaptation
    model_adapter: ModelAdapter,
    
    // Personalization engine
    personalizer: PersonalizationEngine,
    
    // Learning rate controller
    learning_controller: LearningRateController,
}

impl OnlineLearner {
    pub async fn incorporate_feedback(&mut self, feedback: UserFeedback) -> Result<()> {
        // Validate feedback
        let validated = self.feedback_processor.validate(feedback)?;
        
        // Update model weights
        self.model_adapter.update_from_feedback(validated)?;
        
        // Adjust personalization
        self.personalizer.refine_user_profile(validated)?;
        
        Ok(())
    }
    
    pub fn adapt_thresholds(&mut self, user_patterns: &UserPatterns) {
        // Adjust detection thresholds based on individual patterns
        self.personalizer.update_thresholds(user_patterns);
        
        // Update learning rate based on adaptation progress
        self.learning_controller.adjust_rate(user_patterns.stability_score);
    }
}
```

## Integration Points with Other Modules

### Input Sources
- **Storage Module**: Receives EventBatch every 30 seconds
- **Event Bus**: Subscribes to EventBatch messages

### Output Consumers
- **Gamification Module**: Sends StateClassification and metrics
- **AI Integration**: Provides work context for suggestions
- **Storage Module**: Triggers screenshot deletion after analysis

### Data Flow
```
Storage --EventBatch--> Analysis Engine
                          |
                          ├─> Extract Features
                          ├─> Analyze Screenshots
                          ├─> Calculate Metrics
                          ├─> Classify State
                          └─> Emit Results
                                |
                                v
                        Gamification Module
```

## Technology Choices

### Core Technology: Rust
- **Reasoning**: Performance-critical ML inference, memory safety, efficient data processing
- **Key Libraries**:
  - `candle`: Lightweight neural network inference
  - `smartcore`: Classical ML algorithms (Random Forest, SVM)
  - `ndarray`: Efficient numerical arrays
  - `rayon`: Data parallelism for feature extraction

### ML Runtime
- **Primary**: Custom Rust implementations for maximum performance
- **Secondary**: ONNX Runtime for complex models
- **Quantization**: 8-bit and 16-bit for reduced memory usage

### Computer Vision
- **OCR**: `tesseract-rs` with custom training for UI text
- **Image Processing**: `image` and `imageproc` crates
- **GPU Acceleration**: Optional Metal/CUDA via `wgpu`

## Data Structures and Interfaces

### Public API
```rust
#[async_trait]
pub trait AnalysisEngine: Send + Sync {
    /// Process a batch of events
    async fn analyze_batch(&self, batch: EventBatch) -> Result<AnalysisResult>;
    
    /// Get current state classification
    async fn get_current_state(&self) -> ADHDState;
    
    /// Incorporate user feedback
    async fn process_feedback(&self, feedback: UserFeedback) -> Result<()>;
    
    /// Get analysis metrics
    async fn get_metrics(&self) -> AnalysisMetrics;
}

pub struct AnalysisResult {
    pub window_id: Uuid,
    pub timestamp: SystemTime,
    pub state: ADHDState,
    pub confidence: f32,
    pub metrics: BehavioralMetrics,
    pub work_context: Option<WorkContext>,
    pub intervention_readiness: f32,
}
```

### Feature Vector Format
```rust
pub struct FeatureVector {
    // Keystroke features (10 dimensions)
    pub keystroke_features: [f32; 10],
    
    // Mouse features (8 dimensions)
    pub mouse_features: [f32; 8],
    
    // Window features (6 dimensions)
    pub window_features: [f32; 6],
    
    // Screenshot features (12 dimensions)
    pub screenshot_features: Option<[f32; 12]>,
    
    // Temporal features (5 dimensions)
    pub temporal_features: [f32; 5],
    
    // Resource usage features (4 dimensions)
    pub resource_features: [f32; 4],
}
```

### Configuration
```rust
pub struct AnalysisEngineConfig {
    // Model settings
    pub model_path: PathBuf,
    pub use_gpu: bool,
    pub batch_size: usize,              // Default: 1 (real-time)
    
    // Feature extraction
    pub window_size: Duration,          // Default: 30s
    pub window_overlap: Duration,       // Default: 5s
    pub feature_cache_size: usize,      // Default: 100 windows
    
    // State detection
    pub state_confidence_threshold: f32, // Default: 0.7
    pub state_transition_smoothing: f32, // Default: 0.3
    
    // Online learning
    pub enable_online_learning: bool,    // Default: true
    pub learning_rate: f32,              // Default: 0.01
    pub feedback_weight: f32,            // Default: 0.5
    
    // Privacy
    pub enable_screenshots: bool,        // Default: true
    pub ocr_confidence_threshold: f32,   // Default: 0.8
}
```

## Performance Considerations

### Inference Performance
- **Target Latency**: <50ms for full analysis
- **Feature Extraction**: <10ms using parallel processing
- **Model Inference**: <30ms for ensemble prediction
- **Screenshot Analysis**: <100ms (async, non-blocking)

### Memory Management
- **Feature Cache**: LRU cache for 100 windows (~10MB)
- **Model Size**: <20MB total for all models
- **Screenshot Buffer**: Process and discard immediately
- **Total Memory**: <100MB steady state

### Optimization Strategies
1. **Quantized Models**: 8-bit weights for 4x size reduction
2. **Feature Caching**: Reuse computed features across windows
3. **Lazy Evaluation**: Only compute features when needed
4. **SIMD Operations**: Vectorized feature calculations
5. **Async Screenshot**: Non-blocking image analysis

### Throughput
- **Events/second**: 1000+ event processing
- **Windows/minute**: 2 analysis windows
- **Concurrent Screenshots**: Up to 3 in parallel

## Error Handling Strategies

### Model Failures
```rust
pub enum AnalysisError {
    // Model inference failed
    InferenceError { model: String, error: String },
    
    // Feature extraction failed
    FeatureExtractionError { feature_type: String, reason: String },
    
    // Screenshot analysis failed
    ScreenshotError { reason: String },
    
    // Insufficient data for analysis
    InsufficientData { required: usize, available: usize },
}
```

### Graceful Degradation
1. **Model Ensemble Failure**: Fall back to single best model
2. **Screenshot Failure**: Continue without visual context
3. **OCR Failure**: Use UI element detection only
4. **GPU Failure**: Automatic CPU fallback
5. **Memory Pressure**: Reduce feature cache size

### Recovery Mechanisms
- **Model Reloading**: Hot-reload models on corruption
- **State Reset**: Clear state on persistent errors
- **Calibration Mode**: Re-calibrate on accuracy drop
- **Fallback Models**: Simpler models for edge cases

## Security Considerations

### Data Privacy
- **Local Processing**: No data leaves device
- **Screenshot Handling**: Immediate deletion after analysis
- **PII Detection**: Automatic masking of sensitive data
- **Memory Scrubbing**: Clear sensitive data after use

### Model Security
- **Model Validation**: Checksum verification on load
- **Input Validation**: Bounds checking on all features
- **Output Sanitization**: Ensure predictions in valid range
- **Update Security**: Signed model updates only

## Testing Strategy

### Unit Tests
- Feature extraction accuracy
- Metric calculation correctness
- Model prediction consistency
- Error handling paths

### Integration Tests
- End-to-end event processing
- Multi-model ensemble behavior
- Screenshot analysis pipeline
- Online learning updates

### Performance Tests
- Inference latency benchmarks
- Memory usage under load
- Throughput stress testing
- GPU vs CPU performance

### Accuracy Tests
- Model accuracy on test data
- State transition detection
- Feature importance analysis
- Personalization effectiveness