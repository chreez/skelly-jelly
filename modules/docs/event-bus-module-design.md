# Event Bus Module Design

## Module Purpose and Responsibilities

The Event Bus is the central nervous system of Skelly-Jelly, providing a high-performance, type-safe message broker for all inter-module communication. It implements a publish-subscribe pattern with strict message ordering and delivery guarantees while maintaining minimal latency.

### Core Responsibilities
- **Message Routing**: Direct messages from producers to interested consumers
- **Type Safety**: Enforce compile-time message type validation
- **Performance**: Handle 1000+ messages/second with <1ms latency
- **Reliability**: Guarantee message delivery with configurable acknowledgment
- **Monitoring**: Track message flow and performance metrics
- **Back-pressure**: Handle overload scenarios gracefully

## Key Components and Their Functions

### 1. Message Types Registry
```rust
// Core message envelope
pub struct BusMessage {
    pub id: Uuid,
    pub timestamp: SystemTime,
    pub source: ModuleId,
    pub payload: MessagePayload,
    pub correlation_id: Option<Uuid>,
    pub priority: MessagePriority,
}

// All possible message types
pub enum MessagePayload {
    // From Data Capture
    RawEvent(RawEvent),
    
    // From Storage
    EventBatch(EventBatch),
    StorageStatus(StorageMetrics),
    
    // From Analysis Engine
    AnalysisComplete(AnalysisWindow),
    StateChange(StateClassification),
    
    // From Gamification
    InterventionRequest(InterventionRequest),
    RewardEvent(RewardEvent),
    
    // From AI Integration
    InterventionResponse(InterventionResponse),
    AnimationCommand(AnimationCommand),
    
    // From Orchestrator
    HealthCheck(HealthCheckRequest),
    ConfigUpdate(ConfigUpdate),
    
    // System messages
    Shutdown(ShutdownRequest),
    ModuleReady(ModuleId),
    Error(ErrorReport),
}
```

### 2. Message Router
```rust
pub struct MessageRouter {
    // Topic-based routing table
    subscriptions: Arc<RwLock<HashMap<MessageTopic, Vec<SubscriberHandle>>>>,
    
    // Direct module-to-module channels for high-frequency messages
    direct_channels: HashMap<(ModuleId, ModuleId), mpsc::Sender<BusMessage>>,
    
    // Message queue for async delivery
    message_queue: Arc<Mutex<VecDeque<QueuedMessage>>>,
    
    // Metrics collector
    metrics: Arc<Metrics>,
}

impl MessageRouter {
    pub async fn publish(&self, message: BusMessage) -> Result<MessageId> {
        // Record metrics
        self.metrics.record_publish(&message);
        
        // Route based on message type and subscribers
        match &message.payload {
            MessagePayload::RawEvent(_) => {
                // High-frequency path - use direct channel to Storage
                self.send_direct(ModuleId::Storage, message).await?
            }
            _ => {
                // Standard pub-sub routing
                self.route_to_subscribers(message).await?
            }
        }
    }
}
```

### 3. Subscription Manager
```rust
pub struct SubscriptionManager {
    // Active subscriptions
    subscriptions: Arc<RwLock<Vec<Subscription>>>,
    
    // Subscriber health tracking
    subscriber_health: Arc<DashMap<SubscriberId, SubscriberHealth>>,
}

pub struct Subscription {
    pub id: SubscriptionId,
    pub subscriber: ModuleId,
    pub topics: Vec<MessageTopic>,
    pub filter: Option<MessageFilter>,
    pub delivery_mode: DeliveryMode,
    pub channel: SubscriberChannel,
}

pub enum DeliveryMode {
    // Guaranteed delivery with acknowledgment
    Reliable { timeout: Duration },
    
    // Best effort, no acknowledgment required
    BestEffort,
    
    // Latest value only (for status updates)
    LatestOnly,
}
```

### 4. Performance Optimizations
```rust
pub struct PerformanceOptimizer {
    // Ring buffer for high-frequency events
    event_ring: Arc<RingBuffer<RawEvent>>,
    
    // Batch aggregator for similar messages
    batch_aggregator: BatchAggregator,
    
    // Zero-copy message passing for large payloads
    zero_copy_pool: Arc<MemoryPool>,
}

impl PerformanceOptimizer {
    pub fn optimize_delivery(&self, message: &BusMessage) -> DeliveryStrategy {
        match message.payload {
            MessagePayload::RawEvent(_) => {
                // Use ring buffer for events
                DeliveryStrategy::RingBuffer
            }
            MessagePayload::ScreenshotData(_) => {
                // Use zero-copy for large data
                DeliveryStrategy::ZeroCopy
            }
            _ => DeliveryStrategy::Standard
        }
    }
}
```

### 5. Error Handling and Recovery
```rust
pub struct ErrorHandler {
    // Dead letter queue for failed messages
    dead_letter_queue: Arc<Mutex<VecDeque<FailedMessage>>>,
    
    // Retry policy configuration
    retry_policies: HashMap<MessageType, RetryPolicy>,
    
    // Circuit breaker for failing subscribers
    circuit_breakers: Arc<DashMap<SubscriberId, CircuitBreaker>>,
}

pub struct RetryPolicy {
    pub max_attempts: u32,
    pub backoff: BackoffStrategy,
    pub dead_letter_after: Duration,
}
```

## Integration Points with Other Modules

### Data Flow Architecture
```
┌─────────────────┐
│  Data Capture   │──────RawEvent────────►│                │
└─────────────────┘                       │                │
                                          │                │
┌─────────────────┐                       │                │
│    Storage      │◄─────RawEvent─────────│                │
│                 │──────EventBatch───────►│   Event Bus   │
└─────────────────┘                       │                │
                                          │                │
┌─────────────────┐                       │                │
│ Analysis Engine │◄─────EventBatch───────│                │
│                 │──StateClassification──►│                │
└─────────────────┘                       │                │
                                          │                │
┌─────────────────┐                       │                │
│  Gamification   │◄──StateClassification─│                │
│                 │───InterventionRequest─►│                │
└─────────────────┘                       │                │
```

### Module-Specific Channels
- **Data Capture → Storage**: Direct channel for RawEvent (1000+ msg/sec)
- **Storage → Analysis Engine**: Batched channel for EventBatch (2 msg/min)
- **Analysis Engine → Gamification**: Priority channel for state changes
- **Gamification → AI Integration**: Request-response channel
- **AI Integration → Cute Figurine**: Command channel
- **Orchestrator → All**: Broadcast channel for system messages

## Technology Choices

### Core Technology: Rust
- **Reasoning**: Performance, memory safety, and excellent concurrency primitives
- **Key Libraries**:
  - `tokio`: Async runtime with channels
  - `crossbeam-channel`: High-performance MPMC channels
  - `dashmap`: Concurrent hashmap for subscriptions
  - `parking_lot`: Fast synchronization primitives

### Message Serialization
- **Primary**: Custom binary protocol for internal messages
- **Secondary**: Protocol Buffers for extensibility
- **JSON**: Only for debugging and external APIs

### Performance Libraries
- **ring-buffer**: Lock-free ring buffer for events
- **zerocopy**: Zero-copy deserialization
- **mimalloc**: Fast memory allocator

## Data Structures and Interfaces

### Public API
```rust
#[async_trait]
pub trait EventBus: Send + Sync {
    /// Publish a message to the bus
    async fn publish(&self, message: BusMessage) -> Result<MessageId>;
    
    /// Subscribe to messages matching a filter
    async fn subscribe(
        &self,
        subscriber: ModuleId,
        filter: MessageFilter,
        handler: Box<dyn MessageHandler>,
    ) -> Result<SubscriptionId>;
    
    /// Unsubscribe from messages
    async fn unsubscribe(&self, subscription_id: SubscriptionId) -> Result<()>;
    
    /// Get bus metrics
    async fn metrics(&self) -> BusMetrics;
}

#[async_trait]
pub trait MessageHandler: Send + Sync {
    async fn handle(&self, message: BusMessage) -> Result<()>;
}
```

### Message Filters
```rust
pub struct MessageFilter {
    // Filter by message types
    pub types: Option<Vec<MessageType>>,
    
    // Filter by source module
    pub sources: Option<Vec<ModuleId>>,
    
    // Custom predicate function
    pub predicate: Option<Box<dyn Fn(&BusMessage) -> bool + Send + Sync>>,
}
```

### Configuration
```rust
pub struct EventBusConfig {
    // Performance tuning
    pub max_queue_size: usize,           // Default: 10,000
    pub ring_buffer_size: usize,         // Default: 65,536
    pub batch_timeout: Duration,         // Default: 100ms
    
    // Reliability
    pub delivery_timeout: Duration,      // Default: 5s
    pub max_retry_attempts: u32,         // Default: 3
    pub dead_letter_queue_size: usize,   // Default: 1,000
    
    // Monitoring
    pub metrics_interval: Duration,      // Default: 10s
    pub slow_handler_threshold: Duration, // Default: 100ms
}
```

## Performance Considerations

### Throughput Targets
- **RawEvent messages**: 1000+ msg/sec sustained
- **EventBatch messages**: 100 msg/sec
- **State changes**: 10 msg/sec
- **Commands**: 50 msg/sec

### Latency Targets
- **P50 latency**: <100μs
- **P95 latency**: <1ms
- **P99 latency**: <10ms
- **Max latency**: <100ms (with back-pressure)

### Memory Management
- **Ring buffer**: Pre-allocated 64KB for events
- **Message pool**: Object pool for message allocation
- **Zero-copy**: For messages >10KB
- **Bounded queues**: Prevent memory exhaustion

### Optimization Strategies
1. **Lock-free data structures** where possible
2. **NUMA-aware memory allocation** on supported systems
3. **CPU affinity** for router threads
4. **Batch processing** for similar messages
5. **Adaptive routing** based on load

## Error Handling Strategies

### Message Delivery Failures
```rust
pub enum DeliveryError {
    // Subscriber is temporarily unavailable
    SubscriberUnavailable { subscriber: ModuleId, retry_after: Duration },
    
    // Subscriber rejected the message
    MessageRejected { reason: String },
    
    // Timeout waiting for acknowledgment
    DeliveryTimeout { elapsed: Duration },
    
    // Queue is full
    QueueFull { current_size: usize, max_size: usize },
}
```

### Recovery Mechanisms
1. **Automatic Retry**: Exponential backoff with jitter
2. **Circuit Breaker**: Temporarily stop delivery to failing subscribers
3. **Dead Letter Queue**: Store undeliverable messages
4. **Fallback Routing**: Alternative delivery paths
5. **Graceful Degradation**: Drop non-critical messages under load

### Monitoring and Alerting
```rust
pub struct BusMetrics {
    // Throughput metrics
    pub messages_published: Counter,
    pub messages_delivered: Counter,
    pub messages_failed: Counter,
    
    // Latency metrics
    pub delivery_latency: Histogram,
    pub queue_depth: Gauge,
    
    // Health metrics
    pub subscriber_health: HashMap<ModuleId, SubscriberHealth>,
    pub memory_usage: MemoryMetrics,
}
```

## Security Considerations

### Message Validation
- Type-safe message definitions prevent injection
- Size limits prevent memory exhaustion
- Rate limiting per publisher

### Access Control
- Module authentication via secure tokens
- Topic-based access control
- Audit logging for sensitive messages

### Data Protection
- No persistence of sensitive data
- Memory scrubbing for security-critical messages
- Encrypted channels for external communication (future)

## Testing Strategy

### Unit Tests
- Message routing logic
- Filter matching algorithms
- Performance optimization paths
- Error handling scenarios

### Integration Tests
- Multi-module message flow
- Load testing with simulated modules
- Failure scenario testing
- Memory leak detection

### Performance Tests
- Throughput benchmarks
- Latency distribution analysis
- Memory usage under load
- CPU usage profiling

### Chaos Testing
- Random message drops
- Subscriber failures
- Network partitions (for distributed future)
- Resource exhaustion scenarios