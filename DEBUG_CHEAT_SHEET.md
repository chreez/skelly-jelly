# 🔧 Skelly-Jelly Debug Cheat Sheet

> Quick reference for debugging the ADHD companion system

## 🚀 Quick Start Debug Commands

```bash
# Basic debug run
RUST_LOG=debug cargo run --bin skelly-jelly

# Full trace logging
RUST_LOG=trace cargo run

# Development mode with mocked data
cargo run --features="dev-mode,mock-data"

# Health check
cargo run -- --health-check
```

## 🎯 Common Debug Scenarios

### 1. **System Won't Start**
```bash
# Check all dependencies
cargo build --workspace && npm run build --workspaces

# Start with minimal logging
RUST_LOG=warn cargo run

# Check module registration
RUST_LOG=skelly_jelly_orchestrator=debug cargo run
```

### 2. **High CPU Usage**
```bash
# Performance monitoring
cargo run --features="perf-monitoring"

# Check event bus throughput
RUST_LOG=skelly_jelly_event_bus=debug cargo run

# Memory profiling
cargo run -- --memory-profile
```

### 3. **ML Model Issues**
```bash
# Test ML pipeline
cargo test -p skelly-jelly-analysis-engine

# ML debugging with timing
RUST_LOG=skelly_jelly_analysis_engine=trace cargo run

# Validate model accuracy
cargo run --example performance_validation_demo
```

### 4. **Companion Not Appearing**
```bash
# Frontend debug mode
cd modules/cute-figurine && npm run dev:debug

# Check WebGL support
# Browser → about:gpu

# Test animations
npm run test:e2e
```

### 5. **Event Flow Problems**
```bash
# Trace message routing
RUST_LOG=skelly_jelly_event_bus=trace cargo run

# Check module health
cargo run -- --health-check --module=all

# Test throughput (should be 663K+ msg/sec)
cargo test performance_test --release
```

## 📊 Debug Environment Variables

```bash
# Essential debugging
export RUST_LOG=debug
export SKELLY_LOG_FILE=./debug.log

# Performance monitoring
export SKELLY_MAX_CPU=2.0
export SKELLY_MAX_MEMORY_MB=200

# Development features
export SKELLY_DEV_MODE=true
export SKELLY_MOCK_DATA=true
```

## 🔍 Module-Specific Debugging

### Event Bus (Message Routing)
```bash
RUST_LOG=skelly_jelly_event_bus=trace cargo run
# Look for: Message throughput, subscription management, queue depth
```

### Analysis Engine (ML/ADHD Detection)
```bash
RUST_LOG=skelly_jelly_analysis_engine=debug cargo run
# Look for: Feature extraction, inference timing (<50ms), state transitions
```

### Orchestrator (System Coordination)
```bash
RUST_LOG=skelly_jelly_orchestrator=debug cargo run
# Look for: Module startup sequence, health monitoring, resource allocation
```

### Data Capture (Behavioral Monitoring)
```bash
RUST_LOG=skelly_jelly_data_capture=debug cargo run
# Look for: Keystroke patterns, window events, privacy filtering
```

### AI Integration (Contextual Help)
```bash
RUST_LOG=skelly_jelly_ai_integration=debug cargo run
# Look for: Intervention timing, context detection, personality consistency
```

## 🧪 Quick Test Commands

```bash
# Full system integration test
cargo test --test integration

# Performance benchmarks
cargo test performance_test --release

# Privacy validation
cargo test privacy_test

# Frontend E2E tests
cd modules/cute-figurine && npm run test:e2e

# Event bus stress test
cargo test --test stress_test --release
```

## ⚡ Performance Debugging

### Check Resource Usage
```bash
# System resource monitoring
cargo run -- --performance-report

# Memory usage breakdown
cargo run -- --memory-profile

# CPU usage by module
cargo run --features="perf-profiling"
```

### Validate Performance Targets
```bash
# CPU: Should be <2%
htop | grep skelly-jelly

# Memory: Should be <200MB
ps aux | grep skelly-jelly

# Event throughput: Should be >1K msg/sec
cargo test throughput_test --release
```

## 🔧 Configuration Debug

```bash
# Validate config file
cargo run -- --validate-config

# Show active configuration
cargo run -- --show-config

# Test configuration changes
cargo run -- --test-config config/debug.toml
```

## 🚨 Emergency Debug

### System Completely Broken
```bash
# Nuclear option - rebuild everything
cargo clean && npm run clean
cargo build --release && npm run build

# Reset to minimal config
mv config/default.toml config/default.toml.backup
echo '[system]\nlog_level = "debug"' > config/default.toml

# Start with absolute minimum
cargo run --bin skelly-jelly --features="minimal"
```

### Companion Frozen/Crashed
```bash
# Check if process is running
ps aux | grep skelly-jelly

# Force restart companion UI
cd modules/cute-figurine
npm run dev  # Will restart frontend

# Check system tray for hanging process
```

### Privacy/Security Concerns
```bash
# Verify local-only processing
netstat -an | grep skelly  # Should show no external connections

# Check screenshot deletion
ls -la /tmp/skelly-* || echo "No screenshots found (good!)"

# Validate PII masking
cargo test privacy_pii_test
```

## 📝 Debug Report Generation

```bash
# Generate comprehensive debug report
cargo run -- --debug-report > skelly_debug_$(date +%Y%m%d_%H%M%S).txt

# Include system information
cargo run -- --system-info >> debug_report.txt

# Add performance metrics
cargo run -- --performance-report >> debug_report.txt
```

## 🆘 When All Else Fails

1. **Check GitHub Issues** - Your problem might be known
2. **Generate debug report** - `cargo run -- --debug-report`
3. **Join Discord** - Community help available
4. **Submit issue** - Include debug report and system info

---

## 📋 Debug Checklist

- [ ] All dependencies installed (Rust 1.75+, Node 18+, Python 3.13+)
- [ ] Workspace builds successfully (`cargo build --workspace`)
- [ ] Health check passes (`cargo run -- --health-check`)
- [ ] Event bus operational (663K+ msg/sec throughput)
- [ ] ML model loaded and responding (<50ms inference)
- [ ] Companion UI accessible (check system tray)
- [ ] Resource usage within limits (<2% CPU, <200MB memory)
- [ ] Privacy features validated (local-only processing)

*Keep calm and debug on! 🦴🔧*