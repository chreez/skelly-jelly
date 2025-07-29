# ADR 001: Event Bus Architecture

## Status
Accepted

## Context
Skelly-Jelly requires communication between 8 different modules written in multiple languages (Rust and TypeScript). We need a reliable, high-performance way for modules to exchange messages while maintaining loose coupling.

## Decision
We will implement a custom Event Bus as the central message broker for all inter-module communication.

## Rationale

### Why Event Bus over Direct Communication

1. **Loose Coupling**: Modules don't need to know about each other's existence
2. **Scalability**: Easy to add new modules without modifying existing ones
3. **Observability**: Central point for monitoring all system communication
4. **Reliability**: Can implement delivery guarantees, retries, and dead letter queues
5. **Testing**: Easy to mock and test module interactions

### Why Custom Implementation over Message Queue Solutions

1. **Performance**: Native Rust implementation optimized for our use case
2. **No External Dependencies**: Reduces deployment complexity
3. **Tailored Features**: Can implement exactly what we need
4. **Memory Efficiency**: In-process communication without serialization overhead
5. **Cross-Language Support**: Clean API for both Rust and TypeScript modules

## Consequences

### Positive
- High performance (1000+ msg/sec demonstrated)
- Low latency (<1ms for high-priority messages)
- Type-safe message definitions
- Built-in monitoring and metrics
- Flexible delivery modes

### Negative
- Additional complexity vs direct function calls
- Need to maintain custom implementation
- Learning curve for new developers
- Potential single point of failure

### Mitigations
- Comprehensive documentation and examples
- Extensive test coverage
- Circuit breakers and fallback mechanisms
- Clear error messages and debugging tools

## Implementation Details

The Event Bus provides:
- Publish-subscribe messaging pattern
- Multiple delivery modes (best-effort, reliable, latest-only)
- Message filtering by type and source
- Automatic retry with exponential backoff
- Dead letter queue for failed messages
- Performance metrics collection

## References
- Event Bus implementation: `modules/event-bus/`
- Integration tests: `tests/integration/event_flow_test.rs`
- Module contracts: `contracts/`