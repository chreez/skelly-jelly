# ADR 002: Hybrid Rust/TypeScript Architecture

## Status
Accepted

## Context
Skelly-Jelly needs to balance performance requirements (system monitoring, ML inference) with rapid UI development and web technologies for the companion interface.

## Decision
We will use a hybrid architecture with:
- **Rust**: For performance-critical backend modules
- **TypeScript**: For UI and user-facing modules

## Rationale

### Module Language Allocation

**Rust Modules** (6 modules):
- Event Bus - High-throughput message broker
- Orchestrator - System lifecycle management
- Data Capture - Low-level system hooks
- Storage - High-performance data handling
- Analysis Engine - ML inference and computation
- AI Integration - LLM inference and privacy

**TypeScript Modules** (2 modules):
- Cute Figurine - WebGL animations and UI
- Gamification - Game logic and user interaction

### Why This Split

1. **Performance Where Needed**: System monitoring and ML require native performance
2. **Developer Velocity**: UI development is faster in TypeScript/React
3. **Ecosystem Access**: 
   - Rust: System APIs, ML libraries, async runtime
   - TypeScript: WebGL, animation libraries, UI frameworks
4. **Type Safety**: Both languages provide strong typing
5. **Talent Pool**: Easier to find developers for each domain

## Consequences

### Positive
- Optimal performance for critical paths
- Rapid UI iteration and prototyping
- Access to best libraries in each ecosystem
- Clear separation of concerns
- Easier to hire specialists

### Negative
- Two build systems to maintain
- Cross-language communication complexity
- Multiple dependency management systems
- Context switching for developers
- Potential version mismatches

### Mitigations
- Unified workspace management (Cargo + npm workspaces)
- Protocol Buffers for cross-language serialization
- Shared contract definitions
- Comprehensive integration tests
- Clear module boundaries

## Implementation Details

### Build System
- Cargo workspace for Rust modules
- npm workspaces for TypeScript modules
- Unified scripts in root package.json

### Communication
- Event Bus handles all cross-language messaging
- JSON serialization for TypeScript compatibility
- Binary protocols (bincode) for Rust-to-Rust

### Development Workflow
```bash
# Full system build
cargo build --workspace && npm install && npm run build

# Development mode
cargo run & npm run dev
```

## References
- Cargo workspace: `Cargo.toml`
- npm workspace: `package.json`
- Module overview: `modules/docs/modules-overview.md`