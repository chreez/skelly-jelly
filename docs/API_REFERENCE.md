# Skelly-Jelly API Reference

## Module APIs

### Data Capture Module

#### Purpose
Creates and emits all system monitoring events (RawEvent types).

#### Public API
```rust
pub trait DataCapture {
    /// Start monitoring system events
    fn start(&mut self) -> Result<(), CaptureError>;
    
    /// Stop monitoring and cleanup
    fn stop(&mut self) -> Result<(), CaptureError>;
    
    /// Get current capture statistics
    fn stats(&self) -> CaptureStats;
}

pub struct CaptureStats {
    pub events_captured: u64,
    pub events_dropped: u64,
    pub cpu_usage_percent: f32,
    pub memory_usage_mb: u32,
}
```

#### Events Emitted
- `KeystrokeEvent` - Typing patterns, inter-key intervals
- `MouseEvent` - Movement velocity, clicks, idle time
- `WindowEvent` - App focus, dwell time, context switches
- `ScreenshotEvent` - Visual context metadata (not raw images)
- `ProcessEvent` - Application launches
- `ResourceEvent` - CPU/memory usage

---

### Storage Module

#### Purpose
Receives RawEvents, manages screenshot lifecycle, creates EventBatches.

#### Public API
```rust
pub trait Storage {
    /// Store a raw event
    fn store_event(&mut self, event: RawEvent) -> Result<(), StorageError>;
    
    /// Get events for a time window
    fn get_events(&self, start: Timestamp, end: Timestamp) -> Result<Vec<RawEvent>, StorageError>;
    
    /// Create event batch for analysis
    fn create_batch(&mut self, window_duration: Duration) -> Result<EventBatch, StorageError>;
    
    /// Mark screenshot as processed (triggers deletion)
    fn mark_screenshot_processed(&mut self, id: ScreenshotId) -> Result<(), StorageError>;
}

pub struct EventBatch {
    pub window_start: Timestamp,
    pub window_end: Timestamp,
    pub events: Vec<RawEvent>,
    pub screenshot_refs: Vec<ScreenshotReference>,
}
```

#### Storage Rules
- Screenshots <5MB kept in memory
- Screenshots ≥5MB saved to temp files
- All screenshots deleted after 30-second analysis window
- Only metadata stored permanently

---

### Analysis Engine Module

#### Purpose
Receives EventBatches, performs ML inference, classifies ADHD states.

#### Public API
```rust
pub trait AnalysisEngine {
    /// Analyze a batch of events
    fn analyze_batch(&mut self, batch: EventBatch) -> Result<AnalysisWindow, AnalysisError>;
    
    /// Update ML model with user feedback
    fn update_model(&mut self, feedback: UserFeedback) -> Result<(), AnalysisError>;
    
    /// Get current model performance metrics
    fn model_metrics(&self) -> ModelMetrics;
}

pub struct AnalysisWindow {
    pub window_start: Timestamp,
    pub window_end: Timestamp,
    pub metrics: WindowMetrics,
    pub state_classification: StateClassification,
    pub confidence: f32, // 0.0 - 1.0
}

pub struct WindowMetrics {
    // Keystroke metrics
    pub mean_iki_ms: f32,
    pub iki_coefficient_variation: f32,
    pub burst_count: u32,
    pub backspace_rate: f32,
    pub typing_velocity_wpm: f32,
    
    // Mouse metrics
    pub mean_velocity: f32,
    pub click_rate_per_min: f32,
    pub idle_percentage: f32,
    
    // Window metrics
    pub switch_count: u32,
    pub mean_dwell_time_ms: f32,
    pub unique_apps: u32,
    
    // Resource metrics
    pub cpu_usage_percent: f32,
    pub memory_delta_mb: i32,
    pub process_spawn_rate: f32,
}
```

#### ADHD States
- `Flow` - Optimal productive state
- `Hyperfocus` - Extended deep focus
- `ProductiveSwitching` - Healthy task rotation
- `Distracted` - Attention fragmentation
- `Perseveration` - Stuck on unproductive task
- `Idle` - No activity
- `Break` - Rest period

---

### Gamification Module

#### Purpose
Receives StateClassifications, manages rewards, decides intervention timing.

#### Public API
```rust
pub trait Gamification {
    /// Process state change and decide if intervention needed
    fn process_state(&mut self, state: StateClassification) -> Result<Option<InterventionRequest>, GamificationError>;
    
    /// Update reward state based on user actions
    fn update_rewards(&mut self, action: UserAction) -> Result<RewardUpdate, GamificationError>;
    
    /// Get current user progress
    fn get_progress(&self) -> UserProgress;
    
    /// Set intervention cooldown preferences
    fn set_cooldown(&mut self, cooldown: Duration) -> Result<(), GamificationError>;
}

pub struct InterventionRequest {
    pub current_state: StateClassification,
    pub work_context: WorkContext,
    pub last_intervention_ms_ago: u64,
    pub user_energy_level: EnergyLevel,
    pub time_of_day_factor: f32, // 0.0 (worst) - 1.0 (best)
}

pub struct RewardUpdate {
    pub tokens_earned: u32,
    pub streak_count: u32,
    pub achievement_unlocked: Option<String>,
}
```

#### Intervention Rules
- Minimum 15 minutes between nudges
- Never interrupt detected flow states
- Adaptive to individual tolerance
- Positive reinforcement only

---

### AI Integration Module

#### Purpose
Receives InterventionRequests, generates contextual messages and animations.

#### Public API
```rust
pub trait AIIntegration {
    /// Generate intervention response with LLM
    fn generate_intervention(&mut self, request: InterventionRequest) -> Result<InterventionResponse, AIError>;
    
    /// Analyze work context from screenshots/events
    fn analyze_context(&mut self, context: RawContext) -> Result<WorkContext, AIError>;
    
    /// Set LLM configuration
    fn configure_llm(&mut self, config: LLMConfig) -> Result<(), AIError>;
}

pub struct InterventionResponse {
    pub should_intervene: bool,
    pub intervention_type: Option<InterventionType>,
    pub message: Option<String>,
    pub animation: Option<AnimationCommand>,
    pub delay_ms: Option<u64>,
}

pub struct WorkContext {
    pub detected_activity: ActivityType,
    pub current_task: Option<String>,
    pub error_state: Option<ErrorContext>,
    pub session_duration_ms: u64,
    pub productive_streak_count: u32,
}
```

#### LLM Integration
- Local inference via llama.cpp
- Mistral 7B or Phi-3 Mini models
- <500ms response time target
- Context-aware suggestions

---

### Cute Figurine Module

#### Purpose
Receives AnimationCommands, renders visual companion (read-only).

#### TypeScript API
```typescript
interface CuteFigurine {
    // Initialize the figurine at given position
    initialize(config: FigurineConfig): Promise<void>;
    
    // Play an animation command
    playAnimation(command: AnimationCommand): Promise<void>;
    
    // Update figurine position (draggable)
    setPosition(position: Position): void;
    
    // Show text bubble message
    showMessage(bubble: TextBubble): Promise<void>;
    
    // Get current state
    getState(): FigurineState;
}

interface FigurineConfig {
    position: Position;
    scale: number;
    opacity: number;
    clickThrough: boolean;
}

interface AnimationCommand {
    animation_id: string;
    duration_ms: number;
    expression?: FigurineExpression;
    movement?: FigurineMovement;
    effects?: VisualEffect[];
    message?: TextBubble;
}
```

#### Animation Types
- **Expressions**: neutral, happy, focused, sleepy, encouraging, celebrating, concerned
- **Movements**: bounce, sway, melt, solidify, breathe
- **Effects**: sparkle, glow, trail, bubble

---

## Event Bus API

### Central Communication Hub
```rust
pub trait EventBus {
    /// Subscribe to specific message types
    fn subscribe<T: BusMessage>(&mut self, handler: Box<dyn MessageHandler<T>>) -> SubscriptionId;
    
    /// Unsubscribe from messages
    fn unsubscribe(&mut self, id: SubscriptionId);
    
    /// Publish a message to all subscribers
    fn publish<T: BusMessage>(&mut self, message: T) -> Result<(), BusError>;
}

pub enum BusMessage {
    RawEvent(RawEvent),
    EventBatch(Vec<RawEvent>),
    AnalysisComplete(AnalysisWindow),
    StateChange(StateClassification),
    InterventionRequest(InterventionRequest),
    AnimationCommand(AnimationCommand),
    Shutdown(String),
}
```

---

## Configuration API

### System Configuration
```rust
pub trait Configuration {
    /// Load configuration from file
    fn load(path: &Path) -> Result<SystemConfig, ConfigError>;
    
    /// Save configuration to file
    fn save(&self, path: &Path) -> Result<(), ConfigError>;
    
    /// Update configuration at runtime
    fn update(&mut self, updates: ConfigUpdate) -> Result<(), ConfigError>;
    
    /// Validate configuration
    fn validate(&self) -> Result<(), ValidationError>;
}

pub struct SystemConfig {
    pub llm_provider: LLMProvider,
    pub capture_settings: CaptureSettings,
    pub intervention_settings: InterventionSettings,
    pub figurine_position: Position,
}
```

---

## Error Handling

### Common Error Types
```rust
#[derive(Debug, thiserror::Error)]
pub enum SkelllyJellyError {
    #[error("Capture error: {0}")]
    Capture(#[from] CaptureError),
    
    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),
    
    #[error("Analysis error: {0}")]
    Analysis(#[from] AnalysisError),
    
    #[error("Gamification error: {0}")]
    Gamification(#[from] GamificationError),
    
    #[error("AI error: {0}")]
    AI(#[from] AIError),
    
    #[error("Bus error: {0}")]
    Bus(#[from] BusError),
}
```

---

## Testing Utilities

### Mock Implementations
Each module provides mock implementations for testing:

```rust
#[cfg(test)]
pub mod mocks {
    pub struct MockDataCapture { /* ... */ }
    pub struct MockStorage { /* ... */ }
    pub struct MockAnalysisEngine { /* ... */ }
    pub struct MockGamification { /* ... */ }
    pub struct MockAIIntegration { /* ... */ }
}
```

### Test Helpers
```rust
pub mod testing {
    /// Generate synthetic event data
    pub fn generate_events(count: usize, pattern: EventPattern) -> Vec<RawEvent>;
    
    /// Create test configuration
    pub fn test_config() -> SystemConfig;
    
    /// Simulate user behavior patterns
    pub fn simulate_behavior(behavior: BehaviorType, duration: Duration) -> Vec<RawEvent>;
}
```

---

## Performance Monitoring

### Metrics API
```rust
pub trait Metrics {
    /// Record a timing measurement
    fn record_timing(&mut self, metric: &str, duration: Duration);
    
    /// Record a counter increment
    fn increment_counter(&mut self, metric: &str, value: u64);
    
    /// Record a gauge value
    fn record_gauge(&mut self, metric: &str, value: f64);
    
    /// Get current metrics snapshot
    fn snapshot(&self) -> MetricsSnapshot;
}
```

### Key Metrics
- `capture.events_per_second` - Event capture rate
- `storage.batch_creation_ms` - Batch creation time
- `analysis.inference_ms` - ML inference latency
- `intervention.response_ms` - AI response time
- `figurine.frame_rate` - Animation FPS

---

## Version Compatibility

### API Versioning
- All modules follow semantic versioning
- Breaking changes require major version bump
- Backward compatibility maintained within major versions
- Protocol Buffers ensure wire format compatibility

### Migration Support
```rust
pub trait Migration {
    /// Migrate data from old version
    fn migrate(from_version: Version, data: OldData) -> Result<NewData, MigrationError>;
    
    /// Check if migration needed
    fn needs_migration(version: Version) -> bool;
}
```