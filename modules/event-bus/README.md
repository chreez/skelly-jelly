# Event Bus Module

High-performance message broker for inter-module communication in the Skelly-Jelly project.

## Overview

The Event Bus serves as the central nervous system for Skelly-Jelly, providing type-safe, high-performance messaging between all system modules. It implements a publish-subscribe pattern with configurable delivery guarantees and comprehensive monitoring.

## Key Features

- **Type Safety**: Compile-time message type validation
- **High Performance**: Handles 1000+ messages/second with <1ms latency
- **Reliability**: Configurable delivery guarantees and automatic retry
- **Monitoring**: Comprehensive metrics and performance tracking
- **Scalability**: Lock-free data structures and parallel processing

## Architecture

### Core Components

1. **BusMessage**: Message envelope with metadata
2. **MessageRouter**: High-performance routing engine
3. **SubscriptionManager**: Manages active subscriptions
4. **MetricsCollector**: Real-time performance monitoring

### Message Flow

```
Publisher → EventBus → MessageRouter → SubscriptionManager → Subscribers
                   ↓
              MetricsCollector
```

## Quick Start

### Basic Usage

```rust
use skelly_jelly_event_bus::{
    create_event_bus, BusMessage, MessagePayload, ModuleId, 
    MessageFilter, DeliveryMode
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create and start the event bus
    let bus = create_event_bus()?;
    bus.start().await?;

    // Create a subscription
    let filter = MessageFilter::types(vec![MessageType::RawEvent]);
    let subscription_id = bus.subscribe(
        ModuleId::Storage,
        filter,
        DeliveryMode::BestEffort,
    ).await?;

    // Publish a message
    let message = BusMessage::new(
        ModuleId::DataCapture,
        MessagePayload::ModuleReady(ModuleId::DataCapture),
    );
    let message_id = bus.publish(message).await?;

    // Clean up
    bus.unsubscribe(subscription_id).await?;
    bus.shutdown().await?;
    Ok(())
}
```

### Advanced Configuration

```rust
use skelly_jelly_event_bus::{EventBusConfig, create_event_bus_with_config};
use std::time::Duration;

let config = EventBusConfig {
    max_queue_size: 50_000,
    delivery_timeout: Duration::from_secs(10),
    max_retry_attempts: 5,
    dead_letter_queue_size: 5_000,
    metrics_interval: Duration::from_secs(5),
    slow_handler_threshold: Duration::from_millis(50),
};

let bus = create_event_bus_with_config(config)?;
```

## Message Types

### System Messages

- `ModuleReady`: Module startup notification
- `HealthCheck`: Health check requests
- `ConfigUpdate`: Configuration changes
- `Shutdown`: Graceful shutdown request
- `Error`: Error reporting

### Domain Messages

- `RawEvent`: Captured user activity
- `EventBatch`: Batch of processed events
- `StateChange`: User state transitions
- `InterventionRequest`: Intervention recommendations
- `AnimationCommand`: UI animation instructions

## Subscription Filters

### Message Type Filtering

```rust
// Subscribe to specific message types
let filter = MessageFilter::types(vec![
    MessageType::RawEvent,
    MessageType::StateChange,
]);
```

### Source Module Filtering

```rust
// Subscribe to messages from specific modules
let filter = MessageFilter::sources(vec![
    ModuleId::DataCapture,
    ModuleId::AnalysisEngine,
]);
```

### Custom Filtering

```rust
// Custom predicate filtering
let filter = MessageFilter::all()
    .with_predicate(|msg| {
        msg.priority >= MessagePriority::High
    });
```

## Delivery Modes

### Best Effort

```rust
DeliveryMode::BestEffort
```
- Fast, fire-and-forget delivery
- No acknowledgment required
- Suitable for high-frequency updates

### Reliable Delivery

```rust
DeliveryMode::Reliable { 
    timeout: Duration::from_secs(5) 
}
```
- Guaranteed delivery with acknowledgment
- Automatic retry on failure
- Use for critical messages

### Latest Only

```rust
DeliveryMode::LatestOnly
```
- Only delivers the most recent message
- Drops older undelivered messages
- Perfect for status updates

## Performance Optimization

### Direct Channels

High-frequency message paths use direct channels for optimal performance:

- **Data Capture → Storage**: RawEvent messages
- **Analysis Engine → Gamification**: State changes
- **Gamification → AI Integration**: Intervention requests

### Message Batching

The router automatically batches similar messages to reduce overhead:

```rust
// Batching is automatic for compatible message types
// No code changes required
```

### Metrics and Monitoring

```rust
let metrics = bus.metrics().await?;
println!("Messages published: {}", metrics.messages_published);
println!("P95 latency: {:.2}ms", metrics.delivery_latency.p95_ms);
println!("Success rate: {:.1}%", 
    metrics.messages_delivered as f64 / metrics.messages_published as f64 * 100.0);
```

## Error Handling

### Common Errors

- `QueueFull`: Message queue at capacity
- `SubscriberUnavailable`: Temporary subscriber failure
- `DeliveryTimeout`: Message delivery took too long
- `BusShuttingDown`: Bus is in shutdown process

### Retry Strategy

```rust
match bus.publish(message).await {
    Ok(id) => println!("Published: {}", id),
    Err(EventBusError::QueueFull { .. }) => {
        // Wait and retry
        tokio::time::sleep(Duration::from_millis(100)).await;
        // Retry logic here
    }
    Err(e) => eprintln!("Publish error: {}", e),
}
```

## Integration with Other Modules

### Data Capture Module

```rust
// High-frequency event publishing
let raw_event = RawEvent {
    event_type: "keystroke".to_string(),
    data: serde_json::json!({"key": "a"}),
    window_title: Some("VS Code".to_string()),
    timestamp: Utc::now(),
};

bus.publish(BusMessage::new(
    ModuleId::DataCapture,
    MessagePayload::RawEvent(raw_event),
)).await?;
```

### Storage Module

```rust
// Subscribe to raw events
let filter = MessageFilter::types(vec![MessageType::RawEvent]);
let subscription = bus.subscribe(
    ModuleId::Storage,
    filter,
    DeliveryMode::Reliable { timeout: Duration::from_secs(5) },
).await?;
```

### Analysis Engine

```rust
// Publish state changes
let state_change = StateClassification {
    state: "focused".to_string(),
    confidence: 0.95,
    timestamp: Utc::now(),
    transition_from: Some("distracted".to_string()),
};

bus.publish(BusMessage::with_priority(
    ModuleId::AnalysisEngine,
    MessagePayload::StateChange(state_change),
    MessagePriority::High,
)).await?;
```

## Testing

### Unit Tests

```bash
cargo test
```

### Integration Tests

```bash
cargo test --features integration
```

### Performance Benchmarks

```bash
cargo bench
```

## Configuration

### Environment Variables

- `SKELLY_EVENT_BUS_QUEUE_SIZE`: Maximum queue size (default: 10,000)
- `SKELLY_EVENT_BUS_WORKERS`: Number of worker threads (default: 4)
- `SKELLY_EVENT_BUS_TIMEOUT`: Delivery timeout in seconds (default: 5)

### Runtime Configuration

```rust
let mut config = EventBusConfig::default();
config.max_queue_size = 20_000;
config.delivery_timeout = Duration::from_secs(10);
```

## Monitoring and Metrics

### Key Metrics

- **Throughput**: Messages per second
- **Latency**: P50, P95, P99 delivery times
- **Error Rate**: Failed delivery percentage
- **Queue Depth**: Current message backlog

### Health Checks

```rust
let metrics = bus.metrics().await?;
let is_healthy = metrics.current_queue_depth < metrics.max_queue_size * 0.8
    && metrics.delivery_latency.p95_ms < 100.0;
```

## Troubleshooting

### High Latency

1. Check queue depth: `metrics.current_queue_depth`
2. Monitor slow subscribers
3. Consider increasing worker threads
4. Enable direct channels for high-frequency routes

### Message Loss

1. Verify subscriber is connected
2. Check delivery mode settings
3. Monitor dead letter queue
4. Increase queue buffer sizes

### Memory Usage

1. Monitor subscription count
2. Check for disconnected subscribers
3. Verify message cleanup
4. Consider message size limits

## Performance Targets

- **Throughput**: 1,000+ messages/second
- **Latency P95**: <1ms
- **Memory**: <100MB under normal load
- **CPU**: <10% on modern hardware

## License

MIT License - see LICENSE file for details.