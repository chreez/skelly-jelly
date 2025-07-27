# Data Capture Module - Manual Testing Guide

This guide shows you how to manually test the data-capture module to verify it's working correctly.

## 🚀 Quick Start Testing

### 1. Basic Compilation Test
```bash
# Navigate to the data-capture module
cd modules/data-capture

# Check that everything compiles
cargo check
```

### 2. Run Unit Tests
```bash
# Run all tests
cargo test

# Run specific test suites
cargo test privacy::tests
cargo test platform::macos::tests
```

### 3. Run the Manual Test Application
```bash
# Run the comprehensive manual test
cargo run --example manual_test

# Run with debug logging
RUST_LOG=debug cargo run --example manual_test
```

## 📋 Manual Testing Checklist

### ✅ Phase 1: Module Lifecycle
- [ ] Module creates without errors
- [ ] Module starts successfully  
- [ ] Module stops cleanly
- [ ] Configuration updates work
- [ ] Statistics are reported correctly

### ✅ Phase 2: Monitor Testing
- [ ] **Keystroke Monitor**: Type on keyboard, verify events
- [ ] **Mouse Monitor**: Move mouse, click, verify events
- [ ] **Window Monitor**: Switch apps, verify focus events
- [ ] **Process Monitor**: Launch apps, verify process events
- [ ] **Resource Monitor**: Check CPU/memory monitoring

### ✅ Phase 3: Privacy Features
- [ ] **PII Detection**: Test email, SSN, credit card masking
- [ ] **App Filtering**: Test sensitive vs ignored apps
- [ ] **Privacy Modes**: Test Minimal, Balanced, Strict modes
- [ ] **Content Classification**: Test various content types

### ✅ Phase 4: Performance Validation
- [ ] **CPU Usage**: Should be <1% average, <2% peak
- [ ] **Memory Usage**: Should be <50MB resident
- [ ] **Event Latency**: Should feel responsive (<0.1ms)
- [ ] **Event Loss**: Should be <0.1% under normal load

### ✅ Phase 5: Platform Integration
- [ ] **macOS**: Test accessibility permissions
- [ ] **Error Handling**: Test permission denials gracefully
- [ ] **Resource Limits**: Test behavior under memory pressure

## 🧪 Interactive Testing Scenarios

### Scenario 1: Basic Event Capture
1. Run the manual test application
2. During the interactive phase:
   - Type in different applications
   - Move the mouse around
   - Click on various UI elements
   - Switch between applications
3. Verify events are being captured in the logs

### Scenario 2: Privacy Testing
1. Open sensitive applications (Terminal, 1Password simulator)
2. Type passwords or sensitive information
3. Verify that PII is being masked in the logs
4. Check that sensitive apps trigger enhanced privacy mode

### Scenario 3: Performance Testing
1. Run the application with high activity:
   - Rapid typing
   - Fast mouse movements
   - Frequent app switching
2. Monitor CPU and memory usage
3. Verify performance stays within targets

### Scenario 4: Error Recovery Testing
1. Deny accessibility permissions (if prompted)
2. Verify graceful degradation
3. Grant permissions and restart
4. Verify full functionality resumes

## 🔧 Testing with Different Configurations

### Test Configuration 1: Minimal Monitoring
```rust
DataCaptureConfig {
    monitors: MonitorConfig {
        keystroke: KeystrokeConfig { enabled: true, ..Default::default() },
        mouse: MouseConfig { enabled: false, ..Default::default() },
        window: WindowConfig { enabled: false, ..Default::default() },
        screenshot: ScreenshotConfig { enabled: false, ..Default::default() },
        process: ProcessConfig { enabled: false, ..Default::default() },
        resource: ResourceConfig { enabled: false, ..Default::default() },
    },
    // ... rest of config
}
```

### Test Configuration 2: High Performance Mode
```rust
DataCaptureConfig {
    monitors: MonitorConfig {
        keystroke: KeystrokeConfig { 
            enabled: true,
            buffer_size: 5000,
            coalescence_ms: 1, // Very responsive
            ..Default::default() 
        },
        // Enable all monitors
        // ... 
    },
    performance: PerformanceConfig {
        max_cpu_percent: 0.5, // Very strict
        event_buffer_size: 50000,
        event_batch_size: 1000,
        ..Default::default()
    },
}
```

### Test Configuration 3: Maximum Privacy Mode
```rust
DataCaptureConfig {
    privacy: PrivacyConfig {
        pii_detection: true,
        mask_emails: true,
        mask_ssn: true,
        mask_credit_cards: true,
        mask_passwords: true,
        sensitive_app_list: vec![
            "Terminal".to_string(),
            "1Password".to_string(),
            "SSH".to_string(),
            "VPN".to_string(),
        ],
        screenshot_privacy_zones: vec![
            PrivacyZone {
                x: 0, y: 0, width: 400, height: 100, blur_radius: 20
            }
        ],
    },
    // ...
}
```

## 🐛 Troubleshooting

### Common Issues

**Module fails to start:**
- Check accessibility permissions on macOS
- Verify no other monitoring software conflicts
- Check available system resources

**No events being captured:**
- Verify monitors are enabled in configuration
- Check that applications have focus
- Confirm system is generating the expected events

**High CPU/Memory usage:**
- Reduce buffer sizes in configuration
- Disable unused monitors
- Check for event storms (rapid repeated events)

**Privacy filtering not working:**
- Verify `pii_detection` is enabled
- Check that specific masking flags are set
- Confirm sensitive app list includes target applications

### Debug Mode
Run with maximum logging to see detailed operation:
```bash
RUST_LOG=skelly_jelly_data_capture=trace,debug cargo run --example manual_test
```

### Performance Profiling
Use system tools to monitor resource usage:
```bash
# macOS - Monitor CPU and memory
top -pid $(pgrep manual_test)

# Monitor file descriptors
lsof -p $(pgrep manual_test)
```

## 📊 Expected Results

### Successful Test Output Example:
```
🚀 Starting Data Capture Module Manual Test

=== Test 1: Module Creation ===
✅ Module created successfully

=== Test 2: Module Lifecycle ===
✅ Module started successfully
📊 Module Statistics:
  Events captured: 47
  Events dropped: 0
  Active monitors: 5
  CPU usage: 0.8%
  Memory usage: 12MB
✅ Module stopped successfully

=== Test 3: Configuration Updates ===
✅ Configuration updated successfully

=== Test 4: Privacy Filtering ===
🔒 Email: 'Contact me at john.doe@example.com' → 'Contact me at [EMAIL]'
🔒 SSN: 'My SSN is 123-45-6789' → 'My SSN is [SSN]'
✅ Privacy features tested successfully

=== Test 5: Performance Monitoring ===
⏱️  Iteration 1: CPU=0.7%, Memory=12MB, Events=52
⏱️  Iteration 2: CPU=0.8%, Memory=12MB, Events=67
✅ Performance monitoring completed

🎉 All manual tests completed successfully!
```

## 🎯 Success Criteria

The module passes manual testing if:

- ✅ All 6 test phases complete without errors
- ✅ CPU usage remains below 2% during normal operation
- ✅ Memory usage stays below 50MB
- ✅ Events are captured and processed correctly
- ✅ Privacy filtering works as expected
- ✅ Performance targets are met consistently
- ✅ Module handles errors gracefully
- ✅ Configuration changes take effect properly

## 📝 Reporting Issues

If you encounter issues during testing:

1. **Capture the full error output**
2. **Note your system configuration** (OS, permissions, etc.)
3. **Include the test configuration** you were using
4. **Describe the expected vs actual behavior**
5. **Include performance metrics** if relevant

This comprehensive testing approach ensures the data-capture module works correctly in real-world scenarios and meets all performance and privacy requirements.