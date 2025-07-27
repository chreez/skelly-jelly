# Storage Module

High-performance event storage and batching system for Skelly-Jelly, handling 1000+ events/second with automatic screenshot lifecycle management.

## Overview

The Storage Module serves as the critical buffer between real-time event capture and batch analysis processing. It receives all raw events from the Data Capture module, manages screenshot storage and lifecycle, batches events into 30-second windows, and ensures privacy through automatic cleanup.

## Key Features

- **High Performance**: Handles 1000+ events/second with <2% CPU usage
- **Smart Batching**: 30-second analysis windows with automatic forwarding
- **Screenshot Management**: Size-based routing (memory vs. disk) with automatic cleanup
- **Privacy-First**: Automatic deletion of screenshots after analysis
- **Time-Series Optimized**: SQLite with custom schema for behavioral data
- **Developer Mode**: Retains last 5 screenshots for debugging

## Architecture

```
Data Capture → RawEvent → Storage → EventBatch → Analysis Engine
                            ↓
                      Screenshot Lifecycle
                      (Capture → Store → Delete)
```

## Quick Start

### Installation

```bash
# From the project root
cd modules/storage
cargo build

# Run tests
cargo test

# Run benchmarks
cargo bench
```

### Basic Usage

```rust
use storage::{StorageModule, StorageConfig};

// Initialize with default config
let config = StorageConfig::default();
let mut storage = StorageModule::new(config).await?;

// Start processing events
storage.run().await?;
```

### Configuration

```toml
# config/storage.toml
[storage]
batch_window_seconds = 30
screenshot_memory_threshold_mb = 5
database_path = "~/.skelly-jelly/events.db"
```

## Module Structure

```
storage/
├── src/
│   ├── lib.rs              # Module entry point and public API
│   ├── types.rs            # Data structures and interfaces
│   ├── receiver.rs         # Event bus receiver implementation
│   ├── batch_manager.rs    # Event batching logic
│   ├── screenshot.rs       # Screenshot lifecycle management
│   ├── database.rs         # SQLite operations
│   ├── metrics.rs          # Performance monitoring
│   └── error.rs            # Error types and handling
├── tests/
│   ├── unit/               # Component-level tests
│   ├── integration/        # End-to-end tests
│   └── benchmarks/         # Performance benchmarks
└── BUILD.bazel             # Build configuration
```

## Implementation Guide

### Phase 1: Core Event Storage (Week 1)

**Goals**: Basic event reception and database storage

**Tasks**:
1. Set up module structure and dependencies
2. Implement event receiver with Event Bus integration
3. Create SQLite schema and basic operations
4. Add simple event persistence without batching
5. Write unit tests for core functionality

**Success Criteria**:
- Can receive events from Event Bus
- Events stored in SQLite database
- Basic test coverage (>80%)

### Phase 2: Event Batching (Week 2)

**Goals**: Implement 30-second window batching

**Tasks**:
1. Create `BatchManager` component
2. Implement timer-based window closing
3. Add event aggregation logic
4. Send batches to Analysis Engine
5. Add integration tests for batching

**Success Criteria**:
- Events grouped into 30-second windows
- Batches sent via Event Bus
- No events lost during batching

### Phase 3: Screenshot Management (Week 3)

**Goals**: Implement screenshot storage with lifecycle

**Tasks**:
1. Create `ScreenshotManager` component
2. Implement size-based routing (memory vs. disk)
3. Add metadata extraction
4. Implement automatic cleanup after 30 seconds
5. Add dev mode retention feature

**Success Criteria**:
- Screenshots routed based on 5MB threshold
- Automatic deletion after analysis
- Metadata preserved permanently

### Phase 4: Performance Optimization (Week 4)

**Goals**: Achieve performance targets

**Tasks**:
1. Add write batching for database
2. Implement LZ4 compression
3. Add performance metrics collection
4. Optimize memory usage
5. Run load tests and benchmarks

**Success Criteria**:
- <2% CPU usage under normal load
- <100MB memory footprint
- 1000+ events/second throughput

## Development Guidelines

### Code Style

```rust
// Use descriptive names
pub struct EventBatch { /* ... */ }  // Good
pub struct EB { /* ... */ }          // Bad

// Document public APIs
/// Processes incoming events and batches them for analysis
pub async fn process_events(&mut self) -> Result<()> {
    // Implementation
}

// Handle errors explicitly
match self.database.write(&event).await {
    Ok(_) => self.metrics.record_write(),
    Err(e) => {
        error!("Database write failed: {}", e);
        self.handle_write_failure(e).await?;
    }
}
```

### Testing Strategy

```rust
// Unit test example
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_event_batching() {
        let mut batch_manager = BatchManager::new(Duration::from_secs(30));
        
        // Add events
        for i in 0..100 {
            batch_manager.add_event(create_test_event(i)).await.unwrap();
        }
        
        // Force window close
        let batch = batch_manager.close_window().await.unwrap();
        
        assert_eq!(batch.events.len(), 100);
    }
}
```

### Performance Considerations

1. **Use async/await**: All I/O operations should be async
2. **Batch writes**: Group database writes to reduce overhead
3. **Memory pools**: Pre-allocate buffers for events
4. **Zero-copy**: Use references where possible
5. **Profile regularly**: Run benchmarks before each PR

### Error Handling

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Screenshot storage failed: {0}")]
    Screenshot(String),
    
    #[error("Resource limit exceeded: {resource}")]
    ResourceLimit { resource: String },
}

// Use Result type everywhere
pub type Result<T> = std::result::Result<T, StorageError>;
```

## API Reference

### Public Interface

```rust
/// Main storage module
pub struct StorageModule {
    // Private fields
}

impl StorageModule {
    /// Create new instance with configuration
    pub async fn new(config: StorageConfig) -> Result<Self>;
    
    /// Start processing events
    pub async fn run(&mut self) -> Result<()>;
    
    /// Graceful shutdown
    pub async fn shutdown(&mut self) -> Result<()>;
    
    /// Get current metrics
    pub fn metrics(&self) -> &PerformanceMetrics;
}
```

### Event Types

See `src/types.rs` for complete event definitions.

## Troubleshooting

### High Memory Usage

1. Check screenshot cache size
2. Verify cleanup is running
3. Review batch sizes

### Database Write Failures

1. Check disk space
2. Verify file permissions
3. Check for lock contention

### Event Loss

1. Enable debug logging
2. Check backpressure handling
3. Monitor Event Bus metrics

## Integration with Other Modules

### Data Capture (Input)
- Receives all `RawEvent` types via Event Bus
- No acknowledgment required (fire-and-forget)

### Analysis Engine (Output)
- Sends `EventBatch` every 30 seconds
- Includes screenshot references (not data)

### Event Bus
- Subscribe to `BusMessage::RawEvent`
- Publish `BusMessage::EventBatch`

## Performance Benchmarks

```bash
# Run all benchmarks
cargo bench

# Expected results on M3 Pro:
# event_ingestion: 1,500 events/sec
# batch_creation: 8ms per 30-second window
# screenshot_processing: 35ms for 5MB image
```

## Future Enhancements

- [ ] Distributed storage for multi-device sync
- [ ] Advanced compression (Zstd)
- [ ] Real-time replication
- [ ] GraphQL API for queries
- [ ] GPU-accelerated screenshot processing

## Contributing

1. Follow the implementation phases
2. Write tests for new features
3. Run benchmarks before submitting PR
4. Update documentation

## License

Part of the Skelly-Jelly project. See root LICENSE file.