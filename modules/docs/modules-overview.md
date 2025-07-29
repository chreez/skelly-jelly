# Skelly-Jelly Modules Overview

## System Architecture

Skelly-Jelly consists of 8 core modules that work together to provide ADHD-focused assistance through a melty skeleton companion. All modules communicate through a central Event Bus, ensuring loose coupling and maintainability.

## Module Summary

### 1. **Event Bus** (Rust)
- **Purpose**: Central message broker for all inter-module communication
- **Location**: `modules/event-bus/`
- **Key Features**:
  - Publish-subscribe messaging with <1ms latency
  - Type-safe message routing with compile-time validation
  - Ring buffers for high-frequency events (1000+ msg/sec)
  - Backpressure handling and dead letter queues
- **Integration**: All modules register as publishers/subscribers

### 2. **Orchestrator** (Rust)
- **Purpose**: System lifecycle and health management
- **Location**: `modules/orchestrator/`
- **Key Features**:
  - Module startup/shutdown sequencing
  - Health monitoring with automatic recovery
  - Configuration management with hot-reloading
  - Resource allocation and throttling
- **Integration**: Controls all module lifecycles via Event Bus

### 3. **Data Capture** (Rust) ✅ *Implemented*
- **Purpose**: Non-invasive system monitoring
- **Location**: `modules/data-capture/`
- **Key Features**:
  - OS-level event hooks (keyboard, mouse, window, process)
  - Screenshot capture with privacy masking
  - <1% CPU overhead with zero-impact design
  - Cross-platform support (macOS first)
- **Integration**: Publishes RawEvent to Event Bus

### 4. **Storage** (Rust) ✅ *Implemented*
- **Purpose**: High-performance event storage and batching
- **Location**: `modules/storage/`
- **Key Features**:
  - SQLite-based time-series storage
  - 30-second event batching for analysis
  - Screenshot lifecycle management
  - Configurable retention policies
- **Integration**: Receives RawEvent, publishes EventBatch

### 5. **Analysis Engine** (Rust)
- **Purpose**: ML-based ADHD state detection
- **Location**: `modules/analysis-engine/`
- **Key Features**:
  - Real-time behavioral analysis (<50ms inference)
  - Local-only ML inference (ONNX/CoreML)
  - Screenshot context extraction
  - Online learning for personalization
- **Integration**: Receives EventBatch, publishes StateClassification

### 6. **Gamification** (TypeScript)
- **Purpose**: Intervention timing and reward logic
- **Location**: `modules/gamification/`
- **Key Features**:
  - Non-intrusive intervention scheduling
  - Variable ratio reinforcement system
  - Progress tracking and achievements
  - Flow state detection and respect
- **Integration**: Receives StateClassification, publishes InterventionRequest

### 7. **AI Integration** (Rust + TypeScript)
- **Purpose**: Context-aware assistance generation
- **Location**: `modules/ai-integration/`
- **Key Features**:
  - Local LLM inference (Mistral 7B/Phi-3)
  - Privacy-preserving API fallback
  - Work-specific suggestions
  - Consistent skeleton personality
- **Integration**: Receives InterventionRequest, publishes AnimationCommand

### 8. **Cute Figurine** (TypeScript) ✅ *Implemented*
- **Purpose**: Visual companion and UI
- **Location**: `modules/cute-figurine/`
- **Key Features**:
  - Melty skeleton animations
  - Text bubble messages
  - Draggable positioning
  - WebGL rendering with <10% CPU
- **Integration**: Receives AnimationCommand (read-only consumer)

## Technology Stack

### Core Languages
- **Rust**: Performance-critical modules (event-bus, orchestrator, data-capture, storage, analysis-engine)
- **TypeScript**: UI and game logic (cute-figurine, gamification)
- **Hybrid**: AI integration (Rust for inference, TypeScript for personality)

### Key Dependencies
- **Async Runtime**: tokio (Rust modules)
- **Serialization**: Protocol Buffers, serde
- **ML Inference**: ONNX Runtime, llama.cpp
- **UI Framework**: React + WebGL (Vite bundler)
- **Database**: SQLite with time-series extensions
- **Testing**: Jest (TS), cargo test (Rust)

## Module Communication Flow

```
Data Capture → Storage → Analysis Engine → Gamification → AI Integration → Cute Figurine
     ↓            ↓            ↓               ↓              ↓               ↓
  Event Bus ← Event Bus ← Event Bus ← Event Bus ← Event Bus ← Event Bus
     ↑                                                                        
Orchestrator (manages all module lifecycles and health)
```

## Resource Allocation

| Module | CPU Target | Memory Target | Language |
|--------|------------|---------------|----------|
| Event Bus | 2% | 50MB | Rust |
| Orchestrator | 1% | 30MB | Rust |
| Data Capture | 5% | 50MB | Rust |
| Storage | 10% | 200MB | Rust |
| Analysis Engine | 20% | 500MB | Rust |
| Gamification | 5% | 100MB | TypeScript |
| AI Integration | 30% | 4GB | Rust/TS |
| Cute Figurine | 10% | 200MB | TypeScript |
| **Total System** | **<83%** | **<5.13GB** | - |

## Privacy and Security

- **Local-Only**: All processing happens on device
- **No Telemetry**: Zero external communication (except optional AI APIs)
- **Screenshot Privacy**: Images deleted after 30-second analysis
- **PII Protection**: Automatic masking in captured data
- **User Control**: All features can be disabled/configured

## Development Status

- ✅ **Implemented**: cute-figurine, data-capture, storage
- 📋 **Designed**: event-bus, orchestrator, analysis-engine, gamification, ai-integration
- 🚧 **Next Steps**: 
  1. Implement event-bus (foundation for all communication)
  2. Implement orchestrator (system management)
  3. Implement analysis-engine (behavioral analysis)
  4. Implement gamification (intervention logic)
  5. Implement ai-integration (assistance generation)

## Quick Start for Developers

```bash
# Clone the repository
git clone https://github.com/your-org/skelly-jelly.git
cd skelly-jelly

# Install dependencies
just setup

# Run tests
just test

# Start development mode
just dev

# Build all modules
just build
```

## Module Documentation

Each module has detailed documentation in `modules/docs/`:
- `event-bus-module-design.md` - Message broker architecture
- `skelly-jelly-orchestrator-module-design.md` - System management design
- `analysis-engine-module-design.md` - ML inference pipeline
- `gamification-module-design.md` - Intervention and rewards
- `skelly-jelly-ai-integration-module-design.md` - LLM assistance design
- `integration-specifications.md` - How modules work together
- `data-capture-module-design.md` - System monitoring design
- `storage-design.md` - Event storage architecture

## Contributing

When adding new modules:
1. Follow the established module structure
2. Register with Event Bus using typed messages
3. Implement health checks for Orchestrator monitoring
4. Add integration tests with mock modules
5. Document in `modules/docs/`
6. Update this overview

## License

[License details here]