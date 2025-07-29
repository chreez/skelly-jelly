# 🦴 Skelly-Jelly Demo

## Running the Demo

```bash
./demo/run_demo.sh
```

## What the Demo Shows

This demo simulates the complete Skelly-Jelly system workflow:

### 1. **Data Capture** 📊
- Monitors keyboard patterns (steady vs irregular typing)
- Tracks mouse movement
- Counts window switches

### 2. **Storage** 💾
- Batches events into 30-second windows
- Prepares data for analysis

### 3. **Analysis Engine** 🧠
- Extracts behavioral features
- Classifies ADHD states:
  - **FLOW**: Regular patterns, minimal distractions
  - **DISTRACTED**: Irregular patterns, frequent window switches

### 4. **Gamification** 🎮
- Respects flow states (no interruptions)
- Awards focus coins
- Decides when interventions are needed

### 5. **AI Integration** 🤖
- Generates personalized, encouraging messages
- Maintains skeleton personality

### 6. **Cute Figurine** 💀
- Shows animations matching the user's state
- Provides visual feedback

## Demo Flow

1. **Phase 1**: User in flow state
   - Steady typing detected
   - No intervention needed
   - Coins awarded

2. **Phase 2**: Distraction detected
   - Irregular patterns
   - Multiple window switches
   - System prepares to help

3. **Phase 3**: Gentle intervention
   - Personalized message
   - Skeleton waves encouragingly
   - Non-judgmental support

4. **Phase 4**: Return to flow
   - User recovers quickly
   - Achievement unlocked
   - Skeleton celebrates

## Architecture Demonstrated

```
User Input → Data Capture → Storage → Analysis Engine
                                           ↓
                                    State Detection
                                           ↓
                                     Gamification
                                           ↓
                                    AI Integration
                                           ↓
                                    Cute Figurine
                                           ↓
                                    User Support
```

## Key Features Shown

- ✅ **Privacy First**: All processing happens locally
- ✅ **Non-Intrusive**: Respects flow states
- ✅ **Encouraging**: Positive, supportive messages
- ✅ **Adaptive**: Learns from user patterns
- ✅ **Fun**: Skeleton companion adds delight

## Integration Progress

### ✅ Completed (Phase 1)
- **Real Module Integration**: Rust modules properly linked with Cargo workspace
- **TypeScript Bridge**: IPC communication system for Gamification & Cute Figurine
- **Build System**: Unified npm + cargo build pipeline
- **Event-Driven Architecture**: Real-time event processing loop
- **Configuration System**: TOML-based configuration with environment variables

### 🚧 Ready for Implementation (Phase 2)
The architectural foundation is complete! Next steps in [NEXT_STEPS.md](./NEXT_STEPS.md):

1. **Real Behavioral Monitoring** (Week 1-2)
   - Cross-platform keystroke/mouse monitoring
   - Privacy-preserving feature extraction
   - Real-time pattern analysis

2. **ML State Classification** (Week 1-2) 
   - ONNX model integration for <50ms inference
   - Flow/Distracted/Hyperfocus detection
   - User-specific calibration

3. **WebGL Skeleton Companion** (Week 3-4)
   - 3D skeleton with bone animation
   - Desktop window with transparency
   - Emotion-driven expressions

This demo shows the integration architecture working. The framework is ready for real functionality!

---

*Your skeleton friend is here to help, not judge! 💀✨*