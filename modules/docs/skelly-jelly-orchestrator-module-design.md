# Orchestrator Module Design

## Module Purpose and Responsibilities

The Orchestrator is the system lifecycle manager and health monitor for Skelly-Jelly. It ensures all modules start in the correct order, maintain healthy operation, and shut down gracefully. It acts as the central coordinator for system-wide operations and configuration management.

### Core Responsibilities
- **Lifecycle Management**: Start, stop, and restart modules in dependency order
- **Health Monitoring**: Continuous health checks and failure detection
- **Configuration Distribution**: Centralized config management and hot-reloading
- **Resource Coordination**: Monitor and manage system resource allocation
- **Failure Recovery**: Automated recovery strategies for module failures
- **System Metrics**: Aggregate and expose system-wide metrics

## Key Components and Their Functions

### 1. Module Registry
```rust
pub struct ModuleRegistry {
    // All registered modules
    modules: HashMap<ModuleId, ModuleDescriptor>,
    
    // Dependency graph for startup ordering
    dependency_graph: DependencyGraph,
    
    // Module state tracking
    module_states: Arc<DashMap<ModuleId, ModuleState>>,
    
    // Module handles for lifecycle control
    module_handles: HashMap<ModuleId, ModuleHandle>,
}

pub struct ModuleDescriptor {
    pub id: ModuleId,
    pub name: String,
    pub version: Version,
    pub dependencies: Vec<ModuleId>,
    pub required: bool,  // If false, system can run without it
    pub startup_timeout: Duration,
    pub shutdown_timeout: Duration,
    pub health_check_interval: Duration,
}

pub enum ModuleState {
    NotStarted,
    Starting { since: Instant },
    Running { since: Instant, health: HealthStatus },
    Stopping { since: Instant },
    Stopped { reason: StopReason },
    Failed { error: String, attempts: u32 },
}
```

### 2. Lifecycle Controller
```rust
pub struct LifecycleController {
    registry: Arc<ModuleRegistry>,
    event_bus: Arc<EventBus>,
    startup_sequencer: StartupSequencer,
    shutdown_coordinator: ShutdownCoordinator,
}

impl LifecycleController {
    pub async fn start_system(&self) -> Result<()> {
        // Compute startup order from dependency graph
        let startup_order = self.startup_sequencer.compute_order()?;
        
        for module_id in startup_order {
            self.start_module(module_id).await?;
            
            // Wait for module to be ready
            self.wait_for_ready(module_id).await?;
        }
        
        Ok(())
    }
    
    async fn start_module(&self, module_id: ModuleId) -> Result<()> {
        match module_id {
            ModuleId::EventBus => {
                // Special case: EventBus starts first
                self.start_event_bus().await?
            }
            ModuleId::Storage => {
                let config = self.load_config::<StorageConfig>()?;
                spawn_storage_module(config, self.event_bus.clone()).await?
            }
            ModuleId::DataCapture => {
                let config = self.load_config::<DataCaptureConfig>()?;
                spawn_data_capture_module(config, self.event_bus.clone()).await?
            }
            // ... other modules
        }
    }
}
```

### 3. Health Monitor
```rust
pub struct HealthMonitor {
    // Health check tasks for each module
    health_checkers: HashMap<ModuleId, JoinHandle<()>>,
    
    // Health status cache
    health_cache: Arc<DashMap<ModuleId, HealthReport>>,
    
    // Failure detection
    failure_detector: FailureDetector,
    
    // Recovery strategies
    recovery_manager: RecoveryManager,
}

pub struct HealthReport {
    pub module_id: ModuleId,
    pub status: HealthStatus,
    pub last_check: Instant,
    pub metrics: HealthMetrics,
    pub issues: Vec<HealthIssue>,
}

pub struct HealthMetrics {
    pub cpu_usage: f32,
    pub memory_usage: usize,
    pub message_queue_depth: usize,
    pub error_rate: f32,
    pub response_time_ms: f32,
}

impl HealthMonitor {
    async fn check_module_health(&self, module_id: ModuleId) -> HealthReport {
        // Send health check request via event bus
        let request = HealthCheckRequest {
            module_id,
            timeout: Duration::from_secs(5),
        };
        
        match self.event_bus.request_response(request).await {
            Ok(response) => self.evaluate_health(response),
            Err(_) => HealthReport::unhealthy(module_id, "No response"),
        }
    }
}
```

### 4. Configuration Manager
```rust
pub struct ConfigurationManager {
    // Configuration storage
    config_store: Arc<RwLock<ConfigStore>>,
    
    // Config file watcher for hot-reload
    file_watcher: FileWatcher,
    
    // Config validation
    validators: HashMap<ConfigType, Box<dyn ConfigValidator>>,
    
    // Config change notifications
    change_notifier: ConfigChangeNotifier,
}

pub struct ConfigStore {
    // Global configuration
    global: GlobalConfig,
    
    // Module-specific configs
    module_configs: HashMap<ModuleId, ModuleConfig>,
    
    // Runtime overrides
    overrides: HashMap<String, Value>,
}

impl ConfigurationManager {
    pub async fn update_config(&self, module_id: ModuleId, config: ModuleConfig) -> Result<()> {
        // Validate configuration
        self.validate_config(&config)?;
        
        // Store new config
        self.config_store.write().await.update(module_id, config.clone());
        
        // Notify module of config change
        self.event_bus.publish(BusMessage::config_update(module_id, config)).await?;
        
        Ok(())
    }
    
    pub fn watch_config_files(&self) -> Result<()> {
        self.file_watcher.watch("config/", move |event| {
            if let Some(config) = self.parse_config_change(event) {
                self.update_config(config.module_id, config).await?;
            }
        })
    }
}
```

### 5. Resource Manager
```rust
pub struct ResourceManager {
    // System resource monitoring
    system_monitor: SystemMonitor,
    
    // Resource allocation tracking
    allocations: Arc<RwLock<ResourceAllocations>>,
    
    // Resource limits per module
    resource_limits: HashMap<ModuleId, ResourceLimits>,
    
    // Throttling controller
    throttle_controller: ThrottleController,
}

pub struct ResourceLimits {
    pub max_cpu_percent: f32,
    pub max_memory_mb: usize,
    pub max_file_handles: usize,
    pub max_threads: usize,
}

pub struct ResourceAllocations {
    pub cpu_usage: HashMap<ModuleId, f32>,
    pub memory_usage: HashMap<ModuleId, usize>,
    pub thread_count: HashMap<ModuleId, usize>,
}

impl ResourceManager {
    pub async fn enforce_limits(&self) -> Result<()> {
        let current = self.system_monitor.get_usage().await?;
        
        for (module_id, usage) in current.iter() {
            if let Some(limits) = self.resource_limits.get(module_id) {
                if usage.exceeds(limits) {
                    self.throttle_controller.throttle(module_id, usage, limits).await?;
                }
            }
        }
        
        Ok(())
    }
}
```

### 6. Recovery Strategies
```rust
pub struct RecoveryManager {
    strategies: HashMap<ModuleId, RecoveryStrategy>,
    recovery_history: Arc<Mutex<RecoveryHistory>>,
}

pub enum RecoveryStrategy {
    // Simple restart with exponential backoff
    Restart {
        max_attempts: u32,
        backoff: ExponentialBackoff,
    },
    
    // Restart with state reset
    RestartWithReset {
        reset_state: bool,
        clear_queues: bool,
    },
    
    // Fallback to degraded mode
    DegradedMode {
        disable_features: Vec<FeatureFlag>,
        reduce_load: f32,
    },
    
    // Complete system restart
    SystemRestart {
        save_state: bool,
        notify_user: bool,
    },
}

impl RecoveryManager {
    pub async fn recover_module(&self, module_id: ModuleId, failure: ModuleFailure) -> Result<()> {
        let strategy = self.strategies.get(&module_id)
            .unwrap_or(&RecoveryStrategy::default());
            
        match strategy {
            RecoveryStrategy::Restart { max_attempts, backoff } => {
                let attempts = self.recovery_history.get_attempts(module_id);
                if attempts < *max_attempts {
                    let delay = backoff.calculate(attempts);
                    tokio::time::sleep(delay).await;
                    self.lifecycle_controller.restart_module(module_id).await?;
                }
            }
            // ... other strategies
        }
    }
}
```

## Integration Points with Other Modules

### Module Dependencies and Startup Order
```
1. Orchestrator (self)
2. Event Bus (required by all)
3. Storage (required by Analysis Engine)
4. Data Capture (produces events)
5. Analysis Engine (requires Storage)
6. Gamification (requires Analysis Engine)
7. AI Integration (requires Gamification)
8. Cute Figurine (requires AI Integration)
```

### Communication Patterns
- **Health Checks**: Request-response pattern with all modules
- **Configuration Updates**: Broadcast to affected modules
- **Lifecycle Events**: Publish module state changes
- **Resource Alerts**: Direct messages to affected modules

## Technology Choices

### Core Technology: Rust
- **Reasoning**: System-level control, reliability, performance
- **Key Libraries**:
  - `tokio`: Async runtime and task management
  - `sysinfo`: System resource monitoring
  - `notify`: File system watching for config changes
  - `petgraph`: Dependency graph management

### System Integration
- **Process Management**: Native OS APIs for process control
- **Resource Monitoring**: Platform-specific APIs (Windows Performance Counters, Linux /proc, macOS mach)
- **Service Management**: systemd on Linux, launchd on macOS, Windows Service API

## Data Structures and Interfaces

### Public API
```rust
#[async_trait]
pub trait Orchestrator: Send + Sync {
    /// Start the entire system
    async fn start_system(&self) -> Result<()>;
    
    /// Stop the entire system gracefully
    async fn stop_system(&self, timeout: Duration) -> Result<()>;
    
    /// Get system health status
    async fn get_system_health(&self) -> SystemHealth;
    
    /// Update module configuration
    async fn update_config(&self, module_id: ModuleId, config: ModuleConfig) -> Result<()>;
    
    /// Restart a specific module
    async fn restart_module(&self, module_id: ModuleId) -> Result<()>;
}
```

### System Health Model
```rust
pub struct SystemHealth {
    pub status: SystemStatus,
    pub uptime: Duration,
    pub module_health: HashMap<ModuleId, HealthReport>,
    pub resource_usage: SystemResources,
    pub active_issues: Vec<SystemIssue>,
}

pub enum SystemStatus {
    Healthy,
    Degraded { reason: String },
    Critical { failing_modules: Vec<ModuleId> },
}

pub struct SystemResources {
    pub total_cpu_usage: f32,
    pub total_memory_mb: usize,
    pub disk_usage_mb: usize,
    pub network_bandwidth_kbps: f32,
}
```

### Configuration Schema
```rust
pub struct OrchestratorConfig {
    // Startup configuration
    pub startup_timeout: Duration,        // Default: 60s
    pub module_start_delay: Duration,     // Default: 1s
    pub parallel_startup: bool,           // Default: false
    
    // Health monitoring
    pub health_check_interval: Duration,  // Default: 30s
    pub health_check_timeout: Duration,   // Default: 5s
    pub unhealthy_threshold: u32,         // Default: 3 failed checks
    
    // Recovery settings
    pub auto_recovery: bool,              // Default: true
    pub max_recovery_attempts: u32,       // Default: 3
    pub recovery_backoff: Duration,       // Default: 10s
    
    // Resource management
    pub resource_check_interval: Duration, // Default: 10s
    pub throttle_threshold: f32,          // Default: 90%
}
```

## Performance Considerations

### Startup Performance
- **Parallel Startup**: Start independent modules concurrently
- **Lazy Loading**: Defer non-critical module startup
- **Pre-warming**: Initialize heavy resources during startup
- **Target**: Full system startup <10 seconds

### Runtime Overhead
- **Health Checks**: <1% CPU overhead
- **Resource Monitoring**: <0.5% CPU overhead
- **Configuration Watch**: Negligible (inotify/FSEvents)
- **Memory**: <50MB for orchestrator itself

### Scalability
- **Module Count**: Support 20+ modules
- **Health Check Frequency**: Adaptive based on system load
- **Recovery Operations**: Queued to prevent thundering herd

## Error Handling Strategies

### Startup Failures
```rust
pub enum StartupError {
    // Critical module failed to start
    CriticalModuleFailed { module: ModuleId, error: String },
    
    // Dependency not satisfied
    DependencyMissing { module: ModuleId, missing: ModuleId },
    
    // Resource unavailable
    InsufficientResources { required: SystemResources, available: SystemResources },
    
    // Configuration invalid
    InvalidConfiguration { module: ModuleId, errors: Vec<String> },
}
```

### Runtime Failures
1. **Module Crash**: Automatic restart with backoff
2. **Resource Exhaustion**: Throttle or degrade functionality
3. **Configuration Error**: Rollback to last known good
4. **Communication Failure**: Circuit breaker activation
5. **Cascading Failure**: Emergency shutdown of affected modules

### Recovery Procedures
```rust
pub struct RecoveryProcedure {
    // Pre-recovery checks
    pub pre_checks: Vec<HealthCheck>,
    
    // Recovery steps
    pub steps: Vec<RecoveryStep>,
    
    // Post-recovery validation
    pub validation: Vec<ValidationCheck>,
    
    // Rollback plan
    pub rollback: Option<RollbackPlan>,
}
```

## Security Considerations

### Module Isolation
- Each module runs with least privileges
- Resource limits enforced via OS mechanisms
- Inter-module communication only via Event Bus

### Configuration Security
- Sensitive configs encrypted at rest
- Config changes require validation
- Audit log for all config modifications

### Health Check Security
- Health endpoints authenticated
- Rate limiting on health checks
- No sensitive data in health reports

## Testing Strategy

### Unit Tests
- Dependency graph algorithms
- Recovery strategy selection
- Configuration validation
- Resource calculation

### Integration Tests
- Full system startup/shutdown
- Module failure scenarios
- Configuration hot-reload
- Resource limit enforcement

### Chaos Tests
- Random module failures
- Resource starvation
- Configuration corruption
- Network partitions

### Performance Tests
- Startup time benchmarks
- Health check overhead
- Recovery operation timing
- Memory usage over time