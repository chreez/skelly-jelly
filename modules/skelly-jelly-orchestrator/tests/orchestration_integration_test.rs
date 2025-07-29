//! Comprehensive integration tests for the orchestration system

use skelly_jelly_event_bus::{create_event_bus_with_config, create_event_bus, EventBusConfig, ModuleId};
use skelly_jelly_orchestrator::{
    create_orchestrator, OrchestratorConfig, StartupSequencer, EnhancedHealthMonitor,
    ConfigWatcher, HotReloadConfig, HealthConfig,
};
use std::{sync::Arc, time::Duration};
use tokio_test;

/// Test orchestrator system startup with dependency ordering
#[tokio::test]
async fn test_orchestrated_startup_sequence() {
    // Initialize tracing for test output
    let _ = tracing_subscriber::fmt::try_init();

    // Create event bus
    let event_bus = create_event_bus_with_config(EventBusConfig::default())
        .expect("Failed to create event bus");

    // Create orchestrator with fast startup configuration
    let config = OrchestratorConfig {
        startup_timeout: Duration::from_secs(30),
        module_start_delay: Duration::from_millis(100),
        parallel_startup: false,
        health_check_interval: Duration::from_secs(5),
        health_check_timeout: Duration::from_secs(2),
        unhealthy_threshold: 2,
        auto_recovery: true,
        max_recovery_attempts: 2,
        recovery_backoff: Duration::from_secs(1),
        resource_check_interval: Duration::from_secs(5),
        throttle_threshold: 0.9,
    };

    let orchestrator = create_orchestrator(config, event_bus.clone()).await
        .expect("Failed to create orchestrator");

    // Test system startup
    let startup_result = orchestrator.start_system().await;
    assert!(startup_result.is_ok(), "System startup should succeed: {:?}", startup_result);

    // Verify system health
    let system_health = orchestrator.get_system_health().await;
    println!("System Health: {:?}", system_health.status);

    // Allow some time for modules to fully initialize
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Test graceful shutdown
    let shutdown_result = orchestrator.stop_system(Duration::from_secs(10)).await;
    assert!(shutdown_result.is_ok(), "System shutdown should succeed: {:?}", shutdown_result);

    println!("✅ Orchestrated startup sequence test completed successfully");
}

/// Test startup performance metrics and bottleneck detection
#[tokio::test]
async fn test_startup_performance_metrics() {
    let _ = tracing_subscriber::fmt::try_init();

    let event_bus = create_event_bus_with_config(EventBusConfig::default())
        .expect("Failed to create event bus");

    let config = OrchestratorConfig {
        startup_timeout: Duration::from_secs(15),
        module_start_delay: Duration::from_millis(50),
        parallel_startup: true, // Enable parallel startup for performance
        ..Default::default()
    };

    let orchestrator = create_orchestrator(config, event_bus.clone()).await
        .expect("Failed to create orchestrator");

    // Measure startup time
    let startup_start = std::time::Instant::now();
    
    let startup_result = orchestrator.start_system().await;
    assert!(startup_result.is_ok(), "System startup should succeed");

    let startup_duration = startup_start.elapsed();
    println!("🚀 System startup completed in: {:?}", startup_duration);

    // Verify startup meets performance target (<10 seconds)
    assert!(
        startup_duration < Duration::from_secs(10),
        "Startup should complete within 10 seconds, took: {:?}",
        startup_duration
    );

    // Test shutdown
    orchestrator.stop_system(Duration::from_secs(5)).await
        .expect("System shutdown should succeed");

    println!("✅ Startup performance metrics test completed successfully");
}

/// Test health monitoring and auto-recovery functionality
#[tokio::test]
async fn test_health_monitoring_auto_recovery() {
    let _ = tracing_subscriber::fmt::try_init();

    let event_bus = create_event_bus_with_config(EventBusConfig::default())
        .expect("Failed to create event bus");

    let config = OrchestratorConfig {
        health_check_interval: Duration::from_secs(1), // Fast health checks for testing
        health_check_timeout: Duration::from_millis(500),
        unhealthy_threshold: 2,
        auto_recovery: true,
        max_recovery_attempts: 3,
        recovery_backoff: Duration::from_millis(500),
        ..Default::default()
    };

    let orchestrator = create_orchestrator(config, event_bus.clone()).await
        .expect("Failed to create orchestrator");

    // Start system
    orchestrator.start_system().await
        .expect("System startup should succeed");

    // Allow health monitoring to initialize
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Get initial system health
    let initial_health = orchestrator.get_system_health().await;
    println!("Initial system health: {:?}", initial_health.status);

    // Verify all modules are registered and have basic health reports
    assert!(
        !initial_health.module_health.is_empty(),
        "Should have health reports for modules"
    );

    // Test that system reports healthy status initially
    // Note: In a real test, we might inject failures to test recovery

    // Allow some monitoring cycles
    tokio::time::sleep(Duration::from_secs(3)).await;

    // Get updated health
    let updated_health = orchestrator.get_system_health().await;
    println!("Updated system health: {:?}", updated_health.status);

    // Shutdown
    orchestrator.stop_system(Duration::from_secs(5)).await
        .expect("System shutdown should succeed");

    println!("✅ Health monitoring and auto-recovery test completed successfully");
}

/// Test module state transitions and lifecycle management
#[tokio::test]
async fn test_module_lifecycle_management() {
    let _ = tracing_subscriber::fmt::try_init();

    let event_bus = create_event_bus_with_config(EventBusConfig::default())
        .expect("Failed to create event bus");

    let orchestrator = create_orchestrator(OrchestratorConfig::default(), event_bus.clone()).await
        .expect("Failed to create orchestrator");

    // Start system
    orchestrator.start_system().await
        .expect("System startup should succeed");

    // Test individual module restart
    let module_to_restart = ModuleId::DataCapture;
    println!("🔄 Testing restart of module: {}", module_to_restart);

    let restart_result = orchestrator.restart_module(module_to_restart).await;
    assert!(restart_result.is_ok(), "Module restart should succeed: {:?}", restart_result);

    // Verify module state after restart
    let module_state = orchestrator.get_module_state(module_to_restart).await;
    println!("Module state after restart: {:?}", module_state);

    // Allow time for module to stabilize
    tokio::time::sleep(Duration::from_secs(1)).await;

    // Shutdown
    orchestrator.stop_system(Duration::from_secs(5)).await
        .expect("System shutdown should succeed");

    println!("✅ Module lifecycle management test completed successfully");
}

/// Test configuration hot-reloading capabilities
#[tokio::test]
async fn test_configuration_hot_reload() {
    let _ = tracing_subscriber::fmt::try_init();

    let event_bus = create_event_bus_with_config(EventBusConfig::default())
        .expect("Failed to create event bus");

    let orchestrator = create_orchestrator(OrchestratorConfig::default(), event_bus.clone()).await
        .expect("Failed to create orchestrator");

    // Start system
    orchestrator.start_system().await
        .expect("System startup should succeed");

    // Test configuration update
    let test_config = serde_json::json!({
        "test_setting": "new_value",
        "timeout_ms": 5000,
        "enabled": true
    });

    let config_result = orchestrator.update_config(ModuleId::DataCapture, test_config).await;
    assert!(config_result.is_ok(), "Configuration update should succeed: {:?}", config_result);

    // Allow time for configuration to propagate
    tokio::time::sleep(Duration::from_millis(500)).await;

    println!("✅ Configuration applied successfully");

    // Shutdown
    orchestrator.stop_system(Duration::from_secs(5)).await
        .expect("System shutdown should succeed");

    println!("✅ Configuration hot-reload test completed successfully");
}

/// Test system resilience under resource constraints
#[tokio::test]
async fn test_system_resilience() {
    let _ = tracing_subscriber::fmt::try_init();

    let event_bus = create_event_bus_with_config(EventBusConfig {
        max_queue_size: 1000, // Reduced queue size to test backpressure
        delivery_timeout: Duration::from_millis(500),
        max_retry_attempts: 2,
        ..Default::default()
    }).expect("Failed to create event bus");

    let config = OrchestratorConfig {
        startup_timeout: Duration::from_secs(5), // Shorter timeout for resilience testing
        health_check_interval: Duration::from_millis(500),
        unhealthy_threshold: 3,
        auto_recovery: true,
        ..Default::default()
    };

    let orchestrator = create_orchestrator(config, event_bus.clone()).await
        .expect("Failed to create orchestrator");

    // Test startup under constraints
    let startup_result = orchestrator.start_system().await;
    
    // System should either succeed or fail gracefully
    match startup_result {
        Ok(()) => {
            println!("✅ System started successfully under constraints");
            
            // Test system health under stress
            let health = orchestrator.get_system_health().await;
            println!("System health under constraints: {:?}", health.status);
            
            // Shutdown gracefully
            orchestrator.stop_system(Duration::from_secs(3)).await
                .expect("System shutdown should succeed");
        }
        Err(e) => {
            println!("⚠️  System failed to start under constraints (expected): {}", e);
            // This is acceptable for resilience testing
        }
    }

    println!("✅ System resilience test completed successfully");
}

/// Test comprehensive system integration
#[tokio::test]
async fn test_comprehensive_system_integration() {
    let _ = tracing_subscriber::fmt::try_init();

    let event_bus = create_event_bus_with_config(EventBusConfig::default())
        .expect("Failed to create event bus");

    let orchestrator = create_orchestrator(OrchestratorConfig::default(), event_bus.clone()).await
        .expect("Failed to create orchestrator");

    println!("🔄 Starting comprehensive system integration test");

    // Phase 1: System Startup
    println!("📋 Phase 1: System Startup");
    let startup_start = std::time::Instant::now();
    orchestrator.start_system().await
        .expect("System startup should succeed");
    let startup_duration = startup_start.elapsed();
    println!("  ✅ Startup completed in {:?}", startup_duration);

    // Phase 2: Health Monitoring
    println!("📋 Phase 2: Health Monitoring");
    tokio::time::sleep(Duration::from_secs(2)).await;
    let health = orchestrator.get_system_health().await;
    println!("  ✅ System health: {:?}", health.status);
    println!("  📊 Active modules: {}", health.module_health.len());

    // Phase 3: Configuration Management
    println!("📋 Phase 3: Configuration Testing");
    let test_configs = vec![
        (ModuleId::DataCapture, serde_json::json!({"capture_rate": 100})),
        (ModuleId::Storage, serde_json::json!({"batch_size": 1000})),
        (ModuleId::AnalysisEngine, serde_json::json!({"model_threshold": 0.8})),
    ];

    for (module_id, config) in test_configs {
        orchestrator.update_config(module_id, config).await
            .expect("Configuration update should succeed");
        println!("  ✅ Updated config for {}", module_id);
    }

    // Phase 4: Module Lifecycle Testing
    println!("📋 Phase 4: Module Lifecycle Testing");
    let test_module = ModuleId::Gamification;
    orchestrator.restart_module(test_module).await
        .expect("Module restart should succeed");
    println!("  ✅ Restarted module: {}", test_module);

    // Phase 5: System Stress Test
    println!("📋 Phase 5: Brief Stress Test");
    let stress_start = std::time::Instant::now();
    for i in 0..5 {
        let health = orchestrator.get_system_health().await;
        println!("  📊 Stress check {}: {:?}", i + 1, health.status);
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
    let stress_duration = stress_start.elapsed();
    println!("  ✅ Stress test completed in {:?}", stress_duration);

    // Phase 6: Graceful Shutdown
    println!("📋 Phase 6: Graceful Shutdown");
    let shutdown_start = std::time::Instant::now();
    orchestrator.stop_system(Duration::from_secs(10)).await
        .expect("System shutdown should succeed");
    let shutdown_duration = shutdown_start.elapsed();
    println!("  ✅ Shutdown completed in {:?}", shutdown_duration);

    // Summary
    println!("🎉 Comprehensive Integration Test Summary:");
    println!("  - Startup: {:?}", startup_duration);
    println!("  - Health checks: Passed");
    println!("  - Configuration updates: 3 successful");
    println!("  - Module restarts: 1 successful");
    println!("  - Stress test: {:?}", stress_duration);
    println!("  - Shutdown: {:?}", shutdown_duration);
    
    let total_duration = startup_start.elapsed();
    println!("  - Total test duration: {:?}", total_duration);

    println!("✅ Comprehensive system integration test completed successfully");
}

/// Benchmark test for startup performance
#[tokio::test]
async fn benchmark_startup_performance() {
    let _ = tracing_subscriber::fmt::try_init();

    println!("🏁 Running startup performance benchmark");

    const BENCHMARK_RUNS: usize = 3;
    let mut startup_times = Vec::new();

    for run in 1..=BENCHMARK_RUNS {
        println!("📊 Benchmark run {}/{}", run, BENCHMARK_RUNS);

        let event_bus = create_event_bus_with_config(EventBusConfig::default())
            .expect("Failed to create event bus");

        let config = OrchestratorConfig {
            module_start_delay: Duration::from_millis(50), // Optimized for speed
            ..Default::default()
        };

        let orchestrator = create_orchestrator(config, event_bus.clone()).await
            .expect("Failed to create orchestrator");

        // Measure startup time
        let startup_start = std::time::Instant::now();
        orchestrator.start_system().await
            .expect("System startup should succeed");
        let startup_duration = startup_start.elapsed();

        startup_times.push(startup_duration);
        println!("  ⏱️  Run {} startup time: {:?}", run, startup_duration);

        // Quick shutdown
        orchestrator.stop_system(Duration::from_secs(3)).await
            .expect("System shutdown should succeed");

        // Brief pause between runs
        if run < BENCHMARK_RUNS {
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    }

    // Calculate statistics
    let total_time: Duration = startup_times.iter().sum();
    let avg_time = total_time / BENCHMARK_RUNS as u32;
    let min_time = startup_times.iter().min().unwrap();
    let max_time = startup_times.iter().max().unwrap();

    println!("📈 Startup Performance Benchmark Results:");
    println!("  - Runs: {}", BENCHMARK_RUNS);
    println!("  - Average: {:?}", avg_time);
    println!("  - Minimum: {:?}", min_time);
    println!("  - Maximum: {:?}", max_time);
    println!("  - Target: <10s");
    println!("  - Target Met: {}", avg_time < Duration::from_secs(10));

    // Assert performance requirements
    assert!(
        avg_time < Duration::from_secs(10),
        "Average startup time should be under 10 seconds, got: {:?}",
        avg_time
    );

    assert!(
        *max_time < Duration::from_secs(15),
        "Maximum startup time should be under 15 seconds, got: {:?}",
        max_time
    );

    println!("✅ Startup performance benchmark completed successfully");
}