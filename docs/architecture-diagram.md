# Skelly-Jelly Architecture Diagram

## System Overview

```mermaid
graph TB
    subgraph "User Interface Layer"
        CF[Cute Figurine<br/>💀 WebGL Skeleton]
        UI[System Tray<br/>Settings UI]
    end
    
    subgraph "Application Layer"
        GM[Gamification Module<br/>🎮 TypeScript]
        AI[AI Integration<br/>🤖 Rust + LLM]
    end
    
    subgraph "Analysis Layer"
        AE[Analysis Engine<br/>🧠 Rust + ONNX]
    end
    
    subgraph "Data Layer"
        DC[Data Capture<br/>📊 Rust]
        ST[Storage Module<br/>💾 SQLite]
    end
    
    subgraph "Infrastructure Layer"
        EB[Event Bus<br/>🚌 Rust]
        OR[Orchestrator<br/>🎭 Rust]
    end
    
    %% User interactions
    User((User)) --> CF
    User --> DC
    
    %% Data flow
    DC -->|RawEvent| EB
    EB -->|RawEvent| ST
    ST -->|EventBatch| EB
    EB -->|EventBatch| AE
    AE -->|StateChange| EB
    EB -->|StateChange| GM
    GM -->|InterventionRequest| EB
    EB -->|InterventionRequest| AI
    AI -->|AnimationCommand| EB
    EB -->|AnimationCommand| CF
    AI -->|InterventionResponse| EB
    EB -->|InterventionResponse| GM
    
    %% Control flow
    OR -.->|Health Check| EB
    OR -.->|Config Update| EB
    EB -.->|Status| OR
    
    %% UI updates
    GM -->|Progress Update| UI
    CF -->|User Interaction| EB
    
    style CF fill:#f9f,stroke:#333,stroke-width:4px
    style EB fill:#bbf,stroke:#333,stroke-width:4px
    style AE fill:#bfb,stroke:#333,stroke-width:4px
```

## Data Flow Sequence

```mermaid
sequenceDiagram
    participant User
    participant DataCapture
    participant EventBus
    participant Storage
    participant AnalysisEngine
    participant Gamification
    participant AIIntegration
    participant CuteFigurine
    
    User->>DataCapture: Keyboard/Mouse Input
    DataCapture->>EventBus: RawEvent
    EventBus->>Storage: Store Event
    Storage->>Storage: Batch Events (30s)
    Storage->>EventBus: EventBatch
    EventBus->>AnalysisEngine: Process Batch
    AnalysisEngine->>AnalysisEngine: Extract Features
    AnalysisEngine->>AnalysisEngine: Classify State
    AnalysisEngine->>EventBus: StateChange
    EventBus->>Gamification: State Update
    
    alt Intervention Needed
        Gamification->>EventBus: InterventionRequest
        EventBus->>AIIntegration: Generate Message
        AIIntegration->>AIIntegration: Context Analysis
        AIIntegration->>EventBus: InterventionResponse
        EventBus->>Gamification: Response
        AIIntegration->>EventBus: AnimationCommand
        EventBus->>CuteFigurine: Play Animation
        CuteFigurine->>User: Visual Feedback
    else Flow State
        Gamification->>CuteFigurine: Ambient Animation
        CuteFigurine->>User: Happy Glow
    end
```

## Module Communication Matrix

| Module | Publishes | Subscribes To | Primary Language |
|--------|-----------|---------------|------------------|
| Data Capture | `RawEvent` | `ConfigUpdate`, `Shutdown` | Rust |
| Storage | `EventBatch` | `RawEvent`, `ConfigUpdate` | Rust |
| Analysis Engine | `StateChange`, `AnalysisComplete` | `EventBatch`, `ConfigUpdate` | Rust |
| Gamification | `InterventionRequest`, `RewardEvent` | `StateChange`, `InterventionResponse` | TypeScript |
| AI Integration | `InterventionResponse`, `AnimationCommand` | `InterventionRequest` | Rust |
| Cute Figurine | `UserInteraction` | `AnimationCommand`, `RewardEvent` | TypeScript |
| Orchestrator | `HealthCheck`, `ConfigUpdate` | `HealthStatus`, `ModuleError` | Rust |

## Performance Characteristics

```mermaid
graph LR
    subgraph "High Frequency (>100Hz)"
        DC1[Mouse Events]
        DC2[Keystroke Events]
    end
    
    subgraph "Medium Frequency (1-10Hz)"
        ST1[Event Batching]
        AE1[State Analysis]
    end
    
    subgraph "Low Frequency (<1Hz)"
        GM1[Interventions]
        AI1[AI Generation]
        CF1[Animations]
    end
    
    DC1 --> ST1
    DC2 --> ST1
    ST1 --> AE1
    AE1 --> GM1
    GM1 --> AI1
    AI1 --> CF1
```

## Resource Allocation

```mermaid
pie title CPU Usage Distribution
    "Analysis Engine" : 30
    "AI Integration" : 25
    "Data Capture" : 15
    "Cute Figurine" : 15
    "Storage" : 10
    "Event Bus" : 3
    "Other" : 2
```

```mermaid
pie title Memory Usage Distribution
    "AI Integration (LLM)" : 60
    "Analysis Engine" : 15
    "Storage Cache" : 10
    "Cute Figurine" : 8
    "Event Bus" : 4
    "Other Modules" : 3
```

## Security & Privacy Architecture

```mermaid
graph TB
    subgraph "External Boundary"
        API[Optional API Services]
    end
    
    subgraph "Local System"
        subgraph "Privacy Layer"
            PG[Privacy Guardian]
            PIID[PII Detector]
        end
        
        subgraph "Data Layer"
            LD[Local Data]
            LLMD[Local LLM]
        end
        
        subgraph "Processing"
            DC2[Data Capture]
            AE2[Analysis Engine]
        end
    end
    
    DC2 -->|Raw Data| LD
    LD -->|Sanitized| AE2
    AE2 -->|Features Only| LLMD
    
    LLMD -.->|If Enabled| PG
    PG -->|Sanitized| PIID
    PIID -->|Clean Data| API
    
    style PG fill:#f99,stroke:#333,stroke-width:2px
    style API stroke:#f00,stroke-width:2px,stroke-dasharray: 5 5
```

This architecture ensures:
- **Privacy**: All data processing happens locally by default
- **Performance**: Event-driven architecture with intelligent batching
- **Modularity**: Each component can be developed and tested independently
- **Reliability**: Orchestrator monitors health and manages recovery
- **User Experience**: Non-intrusive interventions that respect flow states