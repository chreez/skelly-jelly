# Data Capture Module

The Data Capture module is responsible for non-invasive system monitoring and creating all `RawEvent` types in the Skelly-Jelly system. It maintains <1% CPU overhead while capturing behavioral signals for ADHD state detection.

## Overview

This module serves as the primary event producer, creating and emitting events to the Storage module via the Event Bus. It handles all system-level monitoring including keystrokes, mouse activity, window focus, screenshots, process information, and resource usage.

## Architecture

### Event Flow
```
System Activity → Platform APIs → Event Monitors → Privacy Filters → Event Bus → Storage Module
```

### Key Components

1. **Event Monitors** - Platform-specific implementations for each event type
2. **Privacy System** - PII detection, masking, and content filtering  
3. **Platform Layer** - Abstractions for cross-platform compatibility
4. **Event Bus Integration** - Publishing events to the system

## Event Types

```rust
pub enum RawEvent {
    Keystroke(KeystrokeEvent),
    MouseClick(MouseClickEvent),
    MouseMove(MouseMoveEvent),
    WindowFocus(WindowFocusEvent),
    WindowSwitch(WindowSwitchEvent),
    Screenshot(ScreenshotEvent),
    Process(ProcessEvent),
    Resource(ResourceEvent),
}
```

## Usage

```rust
use skelly_jelly_data_capture::{DataCaptureModule, DataCaptureConfig};
use skelly_jelly_core::EventBus;

// Initialize with event bus
let event_bus = Arc::new(EventBus::new());
let config = DataCaptureConfig::default();
let mut capture = DataCaptureModule::new(config, event_bus).await?;

// Start monitoring
capture.start().await?;

// Events are automatically emitted to the event bus
// Stop monitoring
capture.stop().await?;
```

## Configuration

```rust
pub struct DataCaptureConfig {
    pub monitors: MonitorConfig,
    pub privacy: PrivacyConfig,
    pub performance: PerformanceConfig,
}

pub struct MonitorConfig {
    pub keystroke: KeystrokeConfig,
    pub mouse: MouseConfig,
    pub window: WindowConfig,
    pub screenshot: ScreenshotConfig,
    pub process: ProcessConfig,
    pub resource: ResourceConfig,
}
```

### Example Configuration

```toml
[monitors.keystroke]
enabled = true
buffer_size = 1000
coalescence_ms = 10

[monitors.screenshot]
enabled = true
capture_interval_ms = 30000
max_size_mb = 5
privacy_mode = "strict"

[privacy]
pii_detection = true
sensitive_app_list = ["1Password", "KeePass", "Banking App"]
mask_passwords = true

[performance]
max_cpu_percent = 1.0
max_memory_mb = 50
event_buffer_size = 10000
```

## Platform Support

### macOS (Primary Platform)
- Uses `CGEventTap` for keyboard/mouse events
- `NSWorkspace` notifications for window events
- `CGWindowListCopyWindowInfo` for screenshots
- Full Metal acceleration support

### Windows
- `SetWindowsHookEx` for input monitoring
- Windows Graphics Capture API for screenshots
- WMI for process information

### Linux
- X11 event hooks or evdev
- XComposite for screenshots
- `/proc` filesystem parsing

## Privacy Features

1. **PII Detection & Masking**
   - Regex patterns for SSN, credit cards, emails
   - ML-based sensitive content classification
   - Real-time masking before event creation

2. **Screenshot Privacy**
   - Password field detection and blurring
   - Incognito/private mode awareness
   - Configurable privacy zones
   - Immediate metadata extraction

3. **Application Filtering**
   - Allowlist/blocklist support
   - Automatic filtering of sensitive apps
   - URL filtering for browsers

## Performance Characteristics

- **CPU Usage**: <1% average, <2% peak
- **Memory**: <50MB resident
- **Latency**: <0.1ms added to system events  
- **Event Loss**: <0.1% under normal load

## Development

### Building
```bash
cd modules/data-capture
cargo build --release
```

### Testing
```bash
# Unit tests
cargo test

# Integration tests
cargo test --test '*' -- --test-threads=1

# Benchmarks
cargo bench
```

### Platform-Specific Development

For macOS development, you'll need:
- Xcode Command Line Tools
- Accessibility permissions for the app
- Screen Recording permissions for screenshots

## Integration with Other Modules

### Storage Module
All events are sent to Storage via the Event Bus:
```rust
self.event_bus.publish(BusMessage::RawEvent(event)).await
```

### Analysis Engine
The Analysis Engine receives batched events from Storage, not directly from Data Capture.

### Expected Event Volume
- Keystrokes: 100-1000 events/minute (typing)
- Mouse: 500-5000 events/minute (active use)
- Window: 5-50 events/minute
- Screenshots: 1-2 per minute
- Process: 1 event/second
- Resource: 1 event/second

## Error Handling

The module is designed to fail gracefully:
- Missing permissions: Disable affected monitor
- High load: Drop oldest events (ring buffer)
- Platform errors: Log and continue
- Privacy violations: Block event creation

## Security Considerations

1. **Minimal Permissions** - Request only necessary OS permissions
2. **No Network Access** - Completely offline operation
3. **Memory Safety** - Rust's ownership system prevents leaks
4. **Sandboxing** - Runs in restricted environment where possible

## Troubleshooting

### Common Issues

1. **No events captured**
   - Check OS permissions (Accessibility, Screen Recording)
   - Verify event bus connection
   - Check monitor enable flags

2. **High CPU usage**
   - Reduce screenshot frequency
   - Increase event coalescence time
   - Check for event buffer overflow

3. **Missing events**
   - Increase buffer sizes
   - Check system load
   - Verify platform API limits