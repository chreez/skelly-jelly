# Orchestrator Module

The Orchestrator is the system lifecycle manager and health monitor for Skelly-Jelly. It ensures all modules start in the correct order, maintain healthy operation, and shut down gracefully.

## Overview

The Orchestrator module provides centralized control and coordination for all other modules in the Skelly-Jelly system. It manages:

- **Module Lifecycle**: Start, stop, and restart modules in dependency order
- **Health Monitoring**: Continuous health checks and failure detection  
- **Configuration Management**: Centralized config distribution and hot-reloading
- **Resource Coordination**: Monitor and manage system resource allocation
- **Failure Recovery**: Automated recovery strategies for module failures
- **System Metrics**: Aggregate and expose system-wide metrics

## Key Components

### Module Registry
- Tracks all registered modules and their dependencies
- Manages dependency graph for startup ordering
- Stores module descriptors with configuration and timeouts

### Lifecycle Controller
- Handles module startup and shutdown sequences
- Respects dependency ordering
- Manages module state transitions
- Coordinates graceful system shutdown

### Health Monitor
- Performs periodic health checks on all modules
- Tracks health metrics and failure rates
- Detects degraded and unhealthy modules
- Reports system-wide health status

### Resource Manager
- Monitors CPU, memory, and system resource usage
- Enforces resource limits per module
- Implements throttling for resource violations
- Provides system resource metrics

### Recovery Manager
- Implements automated failure recovery strategies
- Supports multiple recovery patterns (restart, degraded mode, etc.)
- Tracks recovery history and escalation
- Prevents cascading failures

### Configuration Manager
- Manages centralized configuration storage
- Supports hot-reloading of configuration changes
- Validates configuration updates
- Distributes config changes to modules

## Module Dependencies

The orchestrator manages the following startup order based on dependencies:

1. **Orchestrator** (self)
2. **Event Bus** (required by all)
3. **Storage** (required by Analysis Engine)
4. **Data Capture** (produces events)
5. **Analysis Engine** (requires Storage)
6. **Gamification** (requires Analysis Engine)
7. **AI Integration** (requires Gamification)
8. **Cute Figurine** (requires AI Integration)

## Usage

### Basic Usage

```rust
use orchestrator::{create_orchestrator, OrchestratorConfig};
use event_bus::create_event_bus;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create event bus
    let event_bus = create_event_bus().await?;
    
    // Create orchestrator
    let config = OrchestratorConfig::default();
    let orchestrator = create_orchestrator(config, event_bus).await?;
    
    // Start the system
    orchestrator.start_system().await?;
    
    // Get system health
    let health = orchestrator.get_system_health().await;
    println!("System status: {:?}", health.status);
    
    // Stop the system
    orchestrator.stop_system(Duration::from_secs(30)).await?;
    
    Ok(())
}
```

### Configuration

```rust
use orchestrator::OrchestratorConfig;
use std::time::Duration;

let config = OrchestratorConfig {
    startup_timeout: Duration::from_secs(60),
    module_start_delay: Duration::from_secs(1),
    parallel_startup: false,
    health_check_interval: Duration::from_secs(30),
    health_check_timeout: Duration::from_secs(5),
    unhealthy_threshold: 3,
    auto_recovery: true,
    max_recovery_attempts: 3,
    recovery_backoff: Duration::from_secs(10),
    resource_check_interval: Duration::from_secs(10),
    throttle_threshold: 0.9,
};
```

### Health Monitoring

```rust
// Get overall system health
let health = orchestrator.get_system_health().await;

// Check specific module health
if let Some(state) = orchestrator.get_module_state(ModuleId::DataCapture).await {
    match state {
        ModuleState::Running { since } => {
            println!("Data capture running since: {:?}", since);
        }
        ModuleState::Failed { error, attempts } => {
            println!("Data capture failed: {} (attempts: {})", error, attempts);
        }
        _ => println!("Data capture state: {:?}", state),
    }
}
```

### Module Management

```rust
// Restart a specific module
orchestrator.restart_module(ModuleId::AnalysisEngine).await?;

// Update module configuration
let new_config = serde_json::json!({
    "analysis_interval": 30,
    "batch_size": 1000
});
orchestrator.update_config(ModuleId::AnalysisEngine, new_config).await?;

// Register a new module
let descriptor = ModuleDescriptor::new(
    ModuleId::CustomModule,
    "custom-module".to_string()
)
.with_dependencies(vec![ModuleId::EventBus])
.with_required(false);

orchestrator.register_module(descriptor).await?;
```

## Recovery Strategies

The orchestrator supports multiple recovery strategies:

- **Restart**: Simple restart with exponential backoff
- **Restart with Reset**: Restart and clear module state/queues
- **Degraded Mode**: Disable features and reduce load
- **System Restart**: Full system restart for critical failures
- **Manual**: Require administrator intervention

## Resource Management

Resource limits are enforced per module:

```rust
// CPU: 25%, Memory: 512MB, File handles: 1000, Threads: 10
ResourceLimits::new(25.0, 512)
    .with_file_handles(1000)
    .with_threads(10)
```

When limits are exceeded, the orchestrator can:
- Reduce processing frequency
- Pause processing temporarily  
- Limit concurrent operations

## Performance Characteristics

- **Startup Time**: Full system startup <10 seconds
- **Health Check Overhead**: <1% CPU overhead
- **Resource Monitoring**: <0.5% CPU overhead
- **Memory Usage**: <50MB for orchestrator itself
- **Module Capacity**: Supports 20+ modules

## Error Handling

The orchestrator handles various failure scenarios:

- Module startup failures
- Dependency resolution failures
- Resource exhaustion
- Health check failures
- Configuration errors
- Communication failures

Each failure type has appropriate recovery strategies and escalation paths.

## Integration

The orchestrator integrates with all other Skelly-Jelly modules through the event bus:

- Publishes module lifecycle events
- Subscribes to error reports and health status
- Distributes configuration updates
- Coordinates system-wide operations

## Testing

Run tests with:

```bash
cargo test
```

For integration testing with other modules:

```bash
cargo test --features integration
```