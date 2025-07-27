# Data Capture Module Design

## Overview
The Data Capture module is the primary event producer in the Skelly-Jelly system, responsible for non-invasive system monitoring with <1% CPU overhead. It captures behavioral signals while maintaining system responsiveness and creates all RawEvent types that flow through the system.

## Module Responsibilities

### Primary Functions
- Create and emit all `RawEvent` types to the Storage module via Event Bus
- Monitor system activity with minimal performance impact
- Respect privacy and OS security restrictions
- Handle cross-platform differences gracefully

### Event Types Created
1. **KeystrokeEvent** - Keyboard activity patterns
2. **MouseEvent** - Mouse movements and clicks
3. **WindowEvent** - Window focus and switching
4. **ScreenshotEvent** - Screen captures with privacy masking
5. **ProcessEvent** - Running process information
6. **ResourceEvent** - System resource usage

## Architecture

### Component Structure
```
modules/data-capture/
├── src/
│   ├── lib.rs                    # Module entry point and exports
│   ├── config.rs                 # Configuration structures
│   ├── error.rs                  # Error types and handling
│   ├── event_bus.rs              # Event bus integration
│   ├── monitors/
│   │   ├── mod.rs               # Monitor trait and common code
│   │   ├── keystroke.rs         # Keystroke monitoring
│   │   ├── mouse.rs             # Mouse monitoring
│   │   ├── window.rs            # Window focus monitoring
│   │   ├── screenshot.rs        # Screenshot capture
│   │   ├── process.rs           # Process monitoring
│   │   └── resource.rs          # Resource usage monitoring
│   ├── platform/
│   │   ├── mod.rs               # Platform abstraction layer
│   │   ├── macos.rs             # macOS-specific implementations
│   │   ├── windows.rs           # Windows-specific implementations
│   │   └── linux.rs             # Linux-specific implementations
│   ├── privacy/
│   │   ├── mod.rs               # Privacy utilities
│   │   ├── masking.rs           # PII detection and masking
│   │   └── filters.rs           # Content filtering
│   └── types.rs                 # Shared types with core module
├── tests/
│   ├── unit/                    # Unit tests
│   └── integration/             # Integration tests
├── Cargo.toml                   # Rust dependencies
└── README.md                    # Module documentation
```

## Technical Design

### Performance Requirements
- **CPU Usage**: <1% average, <2% peak
- **Memory Usage**: <50MB resident
- **Latency**: <0.1ms added to system events
- **Event Loss**: <0.1% under normal load

### Platform Implementation

#### macOS
```rust
// Core APIs to use
- CGEventTap for keyboard/mouse events
- NSWorkspace for window monitoring
- CGWindowListCopyWindowInfo for screenshots
- NSProcessInfo for process information
- IOKit for resource monitoring
```

#### Windows
```rust
// Core APIs to use
- SetWindowsHookEx with WH_KEYBOARD_LL/WH_MOUSE_LL
- WinEvents for window focus
- Windows Graphics Capture API for screenshots
- Windows Management Instrumentation (WMI) for processes
- Performance Data Helper (PDH) for resources
```

#### Linux
```rust
// Core APIs to use
- X11 event hooks or evdev for input
- X11 window manager events
- XComposite for screenshots
- /proc filesystem for processes
- sysfs for resource monitoring
```

### Event Creation Pipeline

```rust
pub trait EventMonitor: Send + Sync {
    type Event: Into<RawEvent>;
    
    fn start(&mut self) -> Result<(), DataCaptureError>;
    fn stop(&mut self) -> Result<(), DataCaptureError>;
    fn events(&self) -> mpsc::Receiver<Self::Event>;
}
```

### Privacy Features

1. **PII Detection**
   - Regex patterns for common PII (SSN, credit cards, etc.)
   - ML-based text classification for sensitive content
   - Configurable sensitivity levels

2. **Screenshot Masking**
   - Password field detection
   - Browser incognito mode detection
   - Selective region blurring
   - OCR for text extraction (optional)

3. **Content Filtering**
   - Allowlist/blocklist for applications
   - URL filtering for browsers
   - Configurable privacy zones

### Resource Management

1. **Event Buffering**
   ```rust
   pub struct EventBuffer<T> {
       ring_buffer: VecDeque<T>,
       max_size: usize,
       overflow_strategy: OverflowStrategy,
   }
   ```

2. **Backpressure Handling**
   - Drop oldest events on buffer overflow
   - Emit warning metrics
   - Graceful degradation under load

3. **Screenshot Lifecycle**
   - Capture triggered by significant events
   - Immediate metadata extraction
   - Memory management (<5MB in memory, else temp file)
   - Automatic cleanup after processing

## Integration Points

### Event Bus Integration
```rust
pub struct DataCaptureModule {
    event_bus: Arc<EventBus>,
    monitors: Vec<Box<dyn EventMonitor>>,
    config: DataCaptureConfig,
}

impl DataCaptureModule {
    pub async fn emit_event(&self, event: RawEvent) -> Result<()> {
        self.event_bus.publish(BusMessage::RawEvent(event)).await
    }
}
```

### Configuration
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataCaptureConfig {
    pub monitors: MonitorConfig,
    pub privacy: PrivacyConfig,
    pub performance: PerformanceConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorConfig {
    pub keystroke: KeystrokeConfig,
    pub mouse: MouseConfig,
    pub window: WindowConfig,
    pub screenshot: ScreenshotConfig,
    pub process: ProcessConfig,
    pub resource: ResourceConfig,
}
```

## Dependencies

### Core Dependencies
```toml
[dependencies]
# Core
tokio = { version = "1.40", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
tracing = "0.1"

# Event handling
crossbeam-channel = "0.5"
parking_lot = "0.12"

# Cross-platform
[target.'cfg(target_os = "macos")'.dependencies]
core-foundation = "0.10"
core-graphics = "0.24"
cocoa = "0.26"

[target.'cfg(target_os = "windows")'.dependencies]
windows = { version = "0.58", features = ["Win32_UI_WindowsAndMessaging", "Win32_System_Performance"] }

[target.'cfg(target_os = "linux")'.dependencies]
x11 = "2.21"
xcb = "1.4"

# Privacy
regex = "1.11"
image = { version = "0.25", default-features = false, features = ["png", "jpeg"] }

# Shared with other modules
skelly-jelly-core = { path = "../core" }
```

## Testing Strategy

### Unit Tests
- Mock platform APIs
- Test event creation logic
- Verify privacy masking
- Performance benchmarks

### Integration Tests
- Test with real Event Bus
- Verify event flow to Storage
- Cross-platform compatibility
- Resource usage validation

### Performance Tests
```rust
#[bench]
fn bench_keystroke_capture(b: &mut Bencher) {
    // Measure overhead added to keystrokes
}

#[bench]
fn bench_screenshot_capture(b: &mut Bencher) {
    // Measure screenshot performance
}
```

## Development Phases

### Phase 1: Core Infrastructure (Week 1)
- [ ] Basic module structure
- [ ] Event bus integration
- [ ] Platform abstraction layer
- [ ] Configuration system

### Phase 2: Basic Monitors (Week 2)
- [ ] Keystroke monitor (macOS first)
- [ ] Mouse monitor
- [ ] Window monitor
- [ ] Resource monitor

### Phase 3: Advanced Features (Week 3)
- [ ] Screenshot capture with privacy
- [ ] Process monitoring
- [ ] Cross-platform support
- [ ] Performance optimization

### Phase 4: Testing & Polish (Week 4)
- [ ] Comprehensive testing
- [ ] Performance validation
- [ ] Documentation
- [ ] Integration with Storage module

## Security Considerations

1. **Permission Management**
   - Request minimal OS permissions
   - Graceful degradation if denied
   - Clear user communication

2. **Data Minimization**
   - Only capture necessary data
   - Immediate aggregation where possible
   - No storage of raw sensitive data

3. **Isolation**
   - Sandboxed execution where possible
   - No network access
   - Limited filesystem access

## Success Metrics

1. **Performance**
   - CPU usage <1% average
   - No noticeable system impact
   - <0.1% event loss

2. **Privacy**
   - Zero PII leakage
   - Effective masking accuracy >99%
   - User trust maintained

3. **Reliability**
   - >99.9% uptime
   - Graceful error recovery
   - Cross-platform consistency

## Future Enhancements

1. **Advanced Privacy**
   - ML-based context understanding
   - Adaptive masking strategies
   - User-trainable filters

2. **Performance**
   - GPU acceleration for screenshots
   - Predictive event batching
   - Dynamic resource allocation

3. **Features**
   - Audio activity detection
   - Peripheral device monitoring
   - Custom event types