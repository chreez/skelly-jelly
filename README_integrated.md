# 🦴 Skelly-Jelly: Your ADHD Companion

> A neurodiversity-affirming ADHD focus companion that actually helps

[![Rust](https://img.shields.io/badge/rust-1.75+-orange.svg)](https://www.rust-lang.org/)
[![TypeScript](https://img.shields.io/badge/typescript-5.0+-blue.svg)](https://www.typescriptlang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## 🌟 What is Skelly-Jelly?

Skelly-Jelly is your melty skeleton companion who helps you stay focused without being annoying. Unlike productivity apps that shame you for getting distracted, Skelly celebrates your ADHD brain and provides gentle, gamified support.

### ✨ Key Features

- 🧠 **Smart ADHD Detection** - ML-powered analysis recognizes flow, distraction, and hyperfocus states
- 🎮 **Non-Intrusive Gamification** - Rewards and achievements that respect your attention
- 💀 **Adorable Companion** - A melty skeleton friend that reacts to your mental state  
- 🤖 **AI-Powered Support** - Context-aware suggestions for coding, writing, and creative work
- 🔒 **Privacy First** - All processing happens locally, your data never leaves your machine
- ⚡ **High Performance** - <50ms response time, minimal CPU usage

## 🚀 Quick Start

### Prerequisites

- Rust 1.75+ (for core modules)
- Node.js 18+ (for UI modules)
- 4GB RAM minimum (8GB recommended for local AI)

### Installation

```bash
# Clone the repository
git clone https://github.com/skelly-team/skelly-jelly.git
cd skelly-jelly

# Install dependencies
./scripts/install.sh

# Run the demo
./demo/run_demo.sh
```

### Running Skelly-Jelly

```bash
# Start with default configuration
cargo run --release

# Start with custom config
cargo run --release -- --config my-config.toml

# Run in demo mode
cargo run --release -- start --demo

# Check system health
cargo run --release -- health
```

## 🏗️ Architecture

Skelly-Jelly uses a modular, event-driven architecture for maximum performance and flexibility:

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   Data      │     │  Analysis   │     │Gamification │
│  Capture    │────▶│   Engine    │────▶│   Module    │
└─────────────┘     └─────────────┘     └─────────────┘
       │                   │                     │
       ▼                   ▼                     ▼
┌─────────────────────────────────────────────────────┐
│                    Event Bus                         │
└─────────────────────────────────────────────────────┘
       ▲                   ▲                     ▲
       │                   │                     │
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│  Storage    │     │     AI      │     │    Cute     │
│  Module     │     │Integration  │     │  Figurine   │
└─────────────┘     └─────────────┘     └─────────────┘
```

### Core Modules

1. **Event Bus** (`Rust`) - High-performance message broker for inter-module communication
2. **Orchestrator** (`Rust`) - Lifecycle management, health monitoring, and recovery
3. **Data Capture** (`Rust`) - Privacy-preserving behavioral data collection
4. **Storage** (`Rust`) - Secure local data persistence with SQLite
5. **Analysis Engine** (`Rust`) - Real-time ML-based ADHD state classification
6. **Gamification** (`TypeScript`) - Adaptive intervention timing and reward systems
7. **AI Integration** (`Rust`) - Local LLM with privacy-safe API fallback
8. **Cute Figurine** (`TypeScript`) - WebGL skeleton companion with 60fps animations

## 🎮 How It Works

1. **Behavioral Monitoring**: Tracks keystroke patterns, mouse movement, and window switching
2. **State Detection**: ML models analyze 30-second windows to classify ADHD states
3. **Smart Interventions**: Gamification system respects flow states and adapts to your patterns
4. **Personalized Messages**: AI generates contextual suggestions matching your personality
5. **Visual Feedback**: Your skeleton companion celebrates wins and offers gentle support

## ⚙️ Configuration

Skelly-Jelly is highly configurable. Create a `config/skelly-jelly.toml`:

```toml
# Respect your flow states
[gamification.intervention]
min_cooldown_minutes = 15
respect_flow_states = true
flow_state_threshold = 0.8

# Privacy-first AI
[ai_integration.privacy]
default_privacy_level = "LocalOnly"
enable_pii_detection = true

# Customize your companion
[cute_figurine]
window_width = 200
window_height = 200
always_on_top = true
enable_transparency = true
```

See [config/default.toml](config/default.toml) for all options.

## 🧪 Development

### Building from Source

```bash
# Build all modules
cargo build --workspace --release

# Run tests
cargo test --workspace

# Run benchmarks
cargo bench

# Enable debug logging
RUST_LOG=debug cargo run
```

### Module Development

Each module can be developed independently:

```bash
# Work on a specific module
cd modules/analysis-engine
cargo test
cargo bench

# TypeScript modules
cd modules/gamification
npm test
npm run build
```

## 📊 Performance

- **Memory Usage**: ~150MB base, +500MB with local AI
- **CPU Usage**: <5% idle, 10-20% during analysis
- **Inference Time**: <50ms for state classification
- **Event Processing**: 1000+ events/second
- **Battery Impact**: Minimal with intelligent batching

## 🔒 Privacy & Security

Your data is yours:

- ✅ **100% Local Processing** - No cloud services required
- ✅ **No Telemetry** - Zero data collection or analytics
- ✅ **Secure Storage** - Optional AES-256 encryption
- ✅ **PII Protection** - Automatic sanitization before any API calls
- ✅ **Open Source** - Audit the code yourself

## 📚 Documentation

- [Quick Start Guide](docs/QUICK_START.md) - Get running in 5 minutes
- [Architecture Overview](docs/modules-overview.md) - Deep dive into the system
- [Module Documentation](modules/docs/) - Detailed module specifications
- [Integration Guide](docs/integration-specifications.md) - How modules work together

## 🤝 Contributing

We welcome contributions! The ADHD community's input is especially valuable.

### Ways to Contribute

- 🐛 Report bugs and request features
- 💡 Suggest intervention strategies
- 🎨 Design new skeleton animations
- 🌍 Add language translations
- ♿ Improve accessibility
- 📖 Write documentation

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## 📜 License

MIT License - see [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- The ADHD community for invaluable feedback and testing
- Contributors to llama.cpp and ONNX Runtime
- The Rust and TypeScript ecosystems

---

*Remember: Your skeleton friend believes in you, even when focus feels impossible!* 💀✨

Made with melty skeleton magic by the Skelly Team