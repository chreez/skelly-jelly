# 🦴 Skelly-Jelly Installation & Debug Guide

> Complete installation, development setup, and debugging guide for the Skelly-Jelly ADHD companion system

## 📋 Table of Contents
- [Installation](#installation)
- [Development Setup](#development-setup)
- [Debug Mode](#debug-mode)
- [Architecture Overview](#architecture-overview)
- [Troubleshooting](#troubleshooting)
- [Configuration](#configuration)

---

## 🚀 Installation

### Prerequisites
- **Rust**: 1.75+ (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)
- **Node.js**: 18.0+ with npm 9.0+
- **Python**: 3.13+ with uv package manager
- **Git**: For cloning and development
- **Platform**: macOS 14+ (primary), Windows 10/11, or Linux

### Quick Installation

```bash
# 1. Clone the repository
git clone <repository-url>
cd skelly-jelly

# 2. Install uv package manager (if needed)
curl -LsSf https://astral.sh/uv/install.sh | sh

# 3. Install Python dependencies
uv sync

# 4. Install Rust dependencies
cargo build --release

# 5. Install Node.js dependencies
npm install

# 6. Build TypeScript modules
npm run build:ts

# 7. Run the system
./target/release/skelly-jelly
```

### Alternative Installation Methods

#### Using UV (Python-focused)
```bash
uv sync
uv run python main.py
```

#### Using Cargo (Rust-focused)
```bash
cargo run --bin skelly-jelly
```

#### Using NPM (Development mode)
```bash
npm run dev  # Starts all services in development mode
```

---

## 🛠️ Development Setup

### Workspace Structure
```
skelly-jelly/
├── src/                          # Main Rust binaries
├── modules/                      # Modular architecture
│   ├── event-bus/                # Rust - Message routing (663K+ msg/sec)
│   ├── skelly-jelly-orchestrator/ # Rust - System coordination
│   ├── data-capture/             # Rust - Behavioral monitoring
│   ├── storage/                  # Rust - Data persistence
│   ├── analysis-engine/          # Rust - ML/ADHD state detection
│   ├── skelly-jelly-ai-integration/ # Rust - AI/LLM integration
│   ├── gamification/             # TypeScript - Reward system
│   └── cute-figurine/            # TypeScript/React - UI companion
├── config/                       # Configuration files
├── docs/                         # Technical documentation
└── tests/                        # Integration tests
```

### Development Workflow

#### 1. **Full System Development**
```bash
# Terminal 1: Start Rust backend
cargo run --bin skelly-jelly-full

# Terminal 2: Start TypeScript frontend (in modules/cute-figurine/)
cd modules/cute-figurine
npm run dev

# Terminal 3: Run integration tests
cargo test --workspace
npm test --workspaces
```

#### 2. **Module-Specific Development**
```bash
# Work on specific module
cd modules/analysis-engine
cargo test
cargo run --example performance_validation_demo

# Frontend module development
cd modules/cute-figurine
npm run dev
npm run test:e2e
```

#### 3. **Integration Demo Mode**
```bash
# Run integration demo with all modules
cargo run --bin skelly-jelly-integration

# Or use npm script
npm run demo
```

---

## 🔍 Debug Mode

### Debug Modes Available

#### 1. **Basic Debug Mode**
```bash
# Enable debug logging
RUST_LOG=debug cargo run --bin skelly-jelly

# Or with Python wrapper
uv run python main.py --debug
```

#### 2. **Verbose Debug with Tracing**
```bash
# Full tracing for all modules
RUST_LOG=trace,skelly_jelly=debug cargo run

# Module-specific debugging
RUST_LOG=skelly_jelly_event_bus=trace cargo run
```

#### 3. **Development Debug Mode**
```bash
# Run with simulated data (no system monitoring)
cargo run --bin skelly-jelly --features="dev-mode,mock-data"

# Frontend debug mode
cd modules/cute-figurine
npm run dev:debug
```

### Debug Configuration

Create `config/debug.toml`:
```toml
[debug]
level = "trace"
modules = ["event-bus", "orchestrator", "analysis-engine"]
output_file = "debug.log"
performance_monitoring = true

[debug.event_bus]
trace_messages = true
performance_metrics = true

[debug.ml_model]
feature_extraction_logs = true
model_inference_timing = true

[debug.companion]
animation_debug = true
state_transitions = true
```

### Debug Features

#### Performance Monitoring
```bash
# Run with performance profiling  
cargo run --bin skelly-jelly --features="perf-profiling"

# View performance dashboard
# Navigate to http://localhost:3000/debug after startup
```

#### ML Model Debugging
```bash
# Run with ML debugging enabled
RUST_LOG=skelly_jelly_analysis_engine=debug cargo run

# Features logged:
# - Feature extraction pipeline
# - Model inference timing (<50ms validation)
# - ADHD state transitions
# - Confidence scoring
```

#### Event Bus Debugging
```bash
# Monitor message flow (663K+ msg/sec)
RUST_LOG=skelly_jelly_event_bus=trace cargo run

# View message routing, subscription management, and throughput
```

---

## 🏗️ Architecture Overview

### System Architecture
```
┌─────────────────────────────────────────────────────────────┐
│                    Skelly-Jelly System                     │
├─────────────────────────────────────────────────────────────┤
│  Frontend (TypeScript/React)                               │
│  ├── Cute Figurine (WebGL animations)                      │
│  ├── Gamification (Reward system)                          │
│  └── Privacy Dashboard (User controls)                     │
├─────────────────────────────────────────────────────────────┤
│  Backend (Rust)                                            │
│  ├── Event Bus (663K+ msg/sec)                             │
│  ├── Orchestrator (System coordination)                    │
│  ├── Data Capture (Behavioral monitoring)                  │
│  ├── Analysis Engine (ML/ADHD detection)                   │
│  ├── AI Integration (LLM/contextual help)                  │
│  └── Storage (Privacy-first persistence)                   │
└─────────────────────────────────────────────────────────────┘
```

### Message Flow
```
Data Capture → Event Bus → Analysis Engine → AI Integration → Gamification → Cute Figurine
      ↓            ↓              ↓               ↓              ↓           ↓
   Storage ←── Orchestrator ←─ Health Monitoring ←─ Error Handling ←─ User Feedback
```

### Key Performance Metrics
- **Event Bus**: 663,187 messages/second throughput
- **ML Inference**: <50ms ADHD state detection
- **Resource Usage**: <2% CPU, <200MB memory
- **Privacy**: 100% local processing, zero network calls

---

## 🚨 Troubleshooting

### Common Issues & Solutions

#### 1. **System Won't Start**
```bash
# Check dependencies
cargo --version  # Should be 1.75+
node --version   # Should be 18.0+
python --version # Should be 3.13+

# Verify modules build
cargo build --workspace
npm run build --workspaces

# Check permissions (macOS)
# System Preferences → Security & Privacy → Accessibility → Add skelly-jelly
```

#### 2. **High CPU Usage**
```bash
# Check if in calibration phase (first 30 minutes)
tail -f logs/performance.log

# Enable performance monitoring
cargo run --features="perf-monitoring"

# Reduce monitoring frequency
# Edit config/default.toml → monitoring.frequency_ms = 5000
```

#### 3. **Companion Not Appearing**
```bash
# Check cute-figurine module
cd modules/cute-figurine
npm run test:e2e

# Verify WebGL support
# Open browser → about:gpu

# Check system tray for application icon
```

#### 4. **ML Model Not Working**
```bash
# Verify ONNX runtime
cargo test -p skelly-jelly-analysis-engine

# Check model files
ls -la models/  # Should contain ONNX model files

# Test ML pipeline
cargo run --example ml_integration_test
```

#### 5. **Event Bus Issues**
```bash
# Test message throughput
cargo test -p skelly-jelly-event-bus performance_test

# Check message routing
RUST_LOG=skelly_jelly_event_bus=debug cargo run

# Verify module registration
# All 8 modules should register on startup
```

### Debug Commands Reference

#### System Health Check
```bash
# Full system health validation
cargo run --bin skelly-jelly -- --health-check

# Module-specific health
cargo run --bin skelly-jelly -- --health-check --module=event-bus
```

#### Performance Analysis
```bash
# Performance benchmark
cargo run --bin skelly-jelly -- --benchmark

# Memory usage analysis
cargo run --bin skelly-jelly -- --memory-profile

# Event throughput test
cargo run --bin skelly-jelly -- --throughput-test
```

#### Configuration Validation
```bash
# Validate configuration files
cargo run --bin skelly-jelly -- --validate-config

# Show active configuration
cargo run --bin skelly-jelly -- --show-config

# Test configuration changes
cargo run --bin skelly-jelly -- --test-config config/debug.toml
```

---

## ⚙️ Configuration

### Configuration Files

#### Main Configuration (`config/default.toml`)
```toml
[system]
log_level = "info"
max_memory_mb = 200
max_cpu_percent = 2.0

[event_bus]
max_throughput = 1000000  # 1M messages/second
queue_size = 10000
worker_threads = 4

[ml_model]
inference_timeout_ms = 50
confidence_threshold = 0.8
model_path = "models/adhd_classifier.onnx"

[privacy]
screenshot_retention_seconds = 30
pii_masking_enabled = true
local_only_processing = true

[companion]
intervention_cooldown_ms = 900000  # 15 minutes
work_hours = "09:00-17:00"
animation_fps = 60
```

#### Debug Configuration (`config/debug.toml`)
```toml
[debug]
level = "trace"
performance_monitoring = true
ml_debugging = true
event_tracing = true

[debug.modules]
event_bus = { trace_messages = true, performance_metrics = true }
analysis_engine = { feature_logs = true, timing_logs = true }
companion = { animation_debug = true, state_logs = true }
```

#### Development Configuration (`config/dev.toml`)
```toml
[development]
mock_data = true
hot_reload = true
dev_server_port = 3000

[development.mock]
adhd_states = ["focused", "distracted", "hyperfocused"]
synthetic_events = true
fake_ml_inference = false  # Still use real ML for accuracy
```

### Environment Variables
```bash
# Logging
export RUST_LOG=debug
export SKELLY_LOG_FILE=./logs/skelly-jelly.log

# Performance
export SKELLY_MAX_CPU=2.0
export SKELLY_MAX_MEMORY_MB=200

# Development
export SKELLY_DEV_MODE=true
export SKELLY_CONFIG_PATH=./config/debug.toml
```

### Runtime Configuration
```bash
# Start with specific configuration
cargo run --bin skelly-jelly -- --config ./config/production.toml

# Override specific settings
cargo run --bin skelly-jelly -- --max-cpu 1.5 --max-memory 150

# Enable specific debug modules
cargo run --bin skelly-jelly -- --debug-modules event-bus,analysis-engine
```

---

## 🧪 Testing & Validation

### Test Suite Commands
```bash
# Full test suite
npm test  # Runs both Rust and TypeScript tests

# Rust tests only
cargo test --workspace

# TypeScript tests only
npm run test --workspaces

# Integration tests
cargo test --test integration

# Performance validation
cargo test --test performance_test --release
```

### Manual Testing
```bash
# Test ADHD state detection
cargo run --example manual_adhd_test

# Test intervention system
cargo run --example intervention_test

# Test privacy features
cargo run --example privacy_validation
```

---

## 📚 Additional Resources

- **[Project Documentation](docs/)** - Complete technical documentation
- **[API Reference](docs/API_REFERENCE.md)** - Module interfaces and APIs
- **[Architecture Decisions](docs/adr/)** - Design decisions and rationale
- **[Module Specifications](modules/docs/)** - Individual module documentation

---

## 🆘 Getting Help

### Debug Information Collection
```bash
# Generate debug report
cargo run --bin skelly-jelly -- --debug-report > debug_report.txt

# System information
cargo run --bin skelly-jelly -- --system-info

# Performance metrics
cargo run --bin skelly-jelly -- --performance-report
```

### Support Channels
- **GitHub Issues**: Technical problems and bug reports
- **Discord**: Community support and discussion
- **Email**: security@skelly-jelly.com for security issues

---

*Made with melty skeleton magic 🦴✨*