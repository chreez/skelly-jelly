//! Integration tests for module registration framework
//! Tests the core functionality of Story 1.1: Module Registration Framework

use std::time::Duration;
use uuid::Uuid;

use skelly_jelly_event_bus::{
    create_event_bus, BusMessage, MessagePayload, ModuleId, MessageFilter, DeliveryMode, MessageType,
    ModuleInfo, ModuleStatus, SystemHealth,
};

/// Test Task 1.1.1: Module Registration Framework
#[tokio::test]
async fn test_module_registration_api() {
    // Create and start event bus
    let event_bus = create_event_bus().expect("Failed to create event bus");
    event_bus.start().await.expect("Failed to start event bus");

    // Test module registration
    let data_capture_info = ModuleInfo::new(ModuleId::DataCapture)
        .with_version("1.0.0".to_string())
        .with_metadata("capability".to_string(), "screenshot".to_string());

    // Register the module
    assert!(event_bus.register_module(data_capture_info.clone()).is_ok());

    // Verify module is registered
    let retrieved_info = event_bus.registry().get_module_info(ModuleId::DataCapture);
    assert!(retrieved_info.is_some());
    let info = retrieved_info.unwrap();
    assert_eq!(info.module_id, ModuleId::DataCapture);
    assert_eq!(info.status, ModuleStatus::Starting);
    assert_eq!(info.version, Some("1.0.0".to_string()));

    // Test health check endpoints
    let health_summary = event_bus.registry().get_health_summary();
    assert_eq!(health_summary.total_modules, 1);
    assert_eq!(health_summary.starting_count, 1);
    assert_eq!(health_summary.overall_health, SystemHealth::Transitioning);

    // Mark module as ready
    assert!(event_bus.registry().mark_module_ready(ModuleId::DataCapture).is_ok());

    // Verify status change
    let updated_info = event_bus.registry().get_module_info(ModuleId::DataCapture).unwrap();
    assert_eq!(updated_info.status, ModuleStatus::Healthy);

    // Test error handling - try to register same module again
    let duplicate_info = ModuleInfo::new(ModuleId::DataCapture);
    assert!(event_bus.register_module(duplicate_info).is_err());

    event_bus.shutdown().await.expect("Failed to shutdown");
}

/// Test Task 1.1.2: End-to-End Message Flow
#[tokio::test]
async fn test_complete_message_flow() {
    let event_bus = create_event_bus().expect("Failed to create event bus");
    event_bus.start().await.expect("Failed to start event bus");

    // Register all modules in the correct dependency order
    let modules = vec![
        ModuleInfo::new(ModuleId::EventBus),
        ModuleInfo::new(ModuleId::Orchestrator),
        ModuleInfo::new(ModuleId::Storage),
        ModuleInfo::new(ModuleId::DataCapture),
        ModuleInfo::new(ModuleId::AnalysisEngine),
        ModuleInfo::new(ModuleId::Gamification),
        ModuleInfo::new(ModuleId::AiIntegration),
        ModuleInfo::new(ModuleId::CuteFigurine),
    ];

    // Register all modules
    for module_info in modules {
        assert!(event_bus.register_module(module_info).is_ok());
    }

    // Mark all modules as ready
    for module_id in [
        ModuleId::EventBus,
        ModuleId::Orchestrator,
        ModuleId::Storage,
        ModuleId::DataCapture,
        ModuleId::AnalysisEngine,
        ModuleId::Gamification,
        ModuleId::AiIntegration,
        ModuleId::CuteFigurine,
    ] {
        assert!(event_bus.registry().mark_module_ready(module_id).is_ok());
    }

    // Verify all modules are registered and healthy
    let health_summary = event_bus.registry().get_health_summary();
    assert_eq!(health_summary.total_modules, 8);
    assert_eq!(health_summary.healthy_count, 8);
    assert_eq!(health_summary.overall_health, SystemHealth::Healthy);

    // Set up subscription for testing message flow
    let subscription_id = event_bus.subscribe(
        ModuleId::Storage,
        MessageFilter::types(vec![MessageType::RawEvent]),
        DeliveryMode::Reliable { timeout: Duration::from_secs(5) },
    ).await.expect("Failed to subscribe");

    // Test DataCapture → Storage flow
    let raw_event = skelly_jelly_event_bus::message::RawEvent::keystroke(
        "a".to_string(),
        Duration::from_millis(100),
        vec![],
    );

    let message = BusMessage::new(
        ModuleId::DataCapture,
        MessagePayload::RawEvent(raw_event),
    );

    let message_id = event_bus.publish(message).await.expect("Failed to publish message");
    assert!(!message_id.is_nil());

    // Wait for message propagation
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Clean up
    event_bus.unsubscribe(subscription_id).await.expect("Failed to unsubscribe");
    event_bus.shutdown().await.expect("Failed to shutdown");
}

/// Test Task 1.1.3: Performance Validation (throughput testing)
#[tokio::test]
async fn test_module_registration_performance() {
    let event_bus = create_event_bus().expect("Failed to create event bus");
    event_bus.start().await.expect("Failed to start event bus");

    // Register modules and measure performance
    let start_time = std::time::Instant::now();

    // Register all 8 modules quickly
    for (i, module_id) in [
        ModuleId::EventBus,
        ModuleId::Orchestrator,
        ModuleId::Storage,
        ModuleId::DataCapture,
        ModuleId::AnalysisEngine,
        ModuleId::Gamification,
        ModuleId::AiIntegration,
        ModuleId::CuteFigurine,
    ].iter().enumerate() {
        let module_info = ModuleInfo::new(*module_id)
            .with_version(format!("1.{}.0", i))
            .with_metadata("test".to_string(), "performance".to_string());
        
        assert!(event_bus.register_module(module_info).is_ok());
        assert!(event_bus.registry().mark_module_ready(*module_id).is_ok());
    }

    let registration_time = start_time.elapsed();
    println!("Module registration time: {:?}", registration_time);

    // Should be very fast - all 8 modules registered in <10ms
    assert!(registration_time < Duration::from_millis(10));

    // Test health check performance
    let health_start = std::time::Instant::now();
    let health_summary = event_bus.registry().get_health_summary();
    let health_time = health_start.elapsed();

    println!("Health check time: {:?}", health_time);
    assert!(health_time < Duration::from_millis(1)); // Should be sub-millisecond

    // Verify correct health summary
    assert_eq!(health_summary.total_modules, 8);
    assert_eq!(health_summary.healthy_count, 8);
    assert_eq!(health_summary.overall_health, SystemHealth::Healthy);

    event_bus.shutdown().await.expect("Failed to shutdown");
}

/// Test Task 1.1.4: Failure Handling
#[tokio::test]
async fn test_module_failure_scenarios() {
    let event_bus = create_event_bus().expect("Failed to create event bus");
    event_bus.start().await.expect("Failed to start event bus");

    // Register test modules
    let modules = vec![
        ModuleInfo::new(ModuleId::DataCapture),
        ModuleInfo::new(ModuleId::Storage),
        ModuleInfo::new(ModuleId::AnalysisEngine),
    ];

    for module_info in modules {
        assert!(event_bus.register_module(module_info).is_ok());
    }

    // Mark modules as ready
    assert!(event_bus.registry().mark_module_ready(ModuleId::DataCapture).is_ok());
    assert!(event_bus.registry().mark_module_ready(ModuleId::Storage).is_ok());
    assert!(event_bus.registry().mark_module_ready(ModuleId::AnalysisEngine).is_ok());

    // Simulate module crash - mark as unhealthy
    assert!(event_bus.registry().update_module_status(
        ModuleId::Storage,
        ModuleStatus::Unhealthy,
        None,
    ).is_ok());

    // Check system health reflects the unhealthy module
    let health_summary = event_bus.registry().get_health_summary();
    assert_eq!(health_summary.unhealthy_count, 1);
    assert_eq!(health_summary.healthy_count, 2);
    assert_eq!(health_summary.overall_health, SystemHealth::Critical);

    // Test recovery - mark module as degraded, then healthy
    assert!(event_bus.registry().update_module_status(
        ModuleId::Storage,
        ModuleStatus::Degraded,
        Some(Duration::from_millis(200)),
    ).is_ok());

    let health_summary = event_bus.registry().get_health_summary();
    assert_eq!(health_summary.degraded_count, 1);
    assert_eq!(health_summary.overall_health, SystemHealth::Degraded);

    // Full recovery
    assert!(event_bus.registry().update_module_status(
        ModuleId::Storage,
        ModuleStatus::Healthy,
        Some(Duration::from_millis(50)),
    ).is_ok());

    let health_summary = event_bus.registry().get_health_summary();
    assert_eq!(health_summary.healthy_count, 3);
    assert_eq!(health_summary.overall_health, SystemHealth::Healthy);

    // Test graceful shutdown
    assert!(event_bus.registry().mark_module_shutting_down(ModuleId::AnalysisEngine).is_ok());

    let health_summary = event_bus.registry().get_health_summary();
    assert_eq!(health_summary.shutting_down_count, 1);
    assert_eq!(health_summary.overall_health, SystemHealth::Transitioning);

    event_bus.shutdown().await.expect("Failed to shutdown");
}

/// Test system startup sequence validation
#[tokio::test]
async fn test_dependency_ordered_startup() {
    let event_bus = create_event_bus().expect("Failed to create event bus");
    event_bus.start().await.expect("Failed to start event bus");

    // Define the correct startup order based on HLD
    let startup_order = vec![
        ModuleId::EventBus,
        ModuleId::Orchestrator,
        ModuleId::Storage,
        ModuleId::DataCapture,
        ModuleId::AnalysisEngine,
        ModuleId::Gamification,
        ModuleId::AiIntegration,
        ModuleId::CuteFigurine,
    ];

    // Register modules in startup order
    for module_id in startup_order {
        let module_info = ModuleInfo::new(module_id);
        assert!(event_bus.register_module(module_info).is_ok());
        
        // Small delay to simulate startup sequence
        tokio::time::sleep(Duration::from_millis(1)).await;
        
        assert!(event_bus.registry().mark_module_ready(module_id).is_ok());
    }

    // Verify all modules are registered in healthy state
    let health_summary = event_bus.registry().get_health_summary();
    assert_eq!(health_summary.total_modules, 8);
    assert_eq!(health_summary.healthy_count, 8);
    assert_eq!(health_summary.overall_health, SystemHealth::Healthy);

    // Test that all modules are tracked correctly
    for module_id in [
        ModuleId::EventBus,
        ModuleId::Orchestrator,
        ModuleId::Storage,
        ModuleId::DataCapture,
        ModuleId::AnalysisEngine,
        ModuleId::Gamification,
        ModuleId::AiIntegration,
        ModuleId::CuteFigurine,
    ] {
        let module_info = event_bus.registry().get_module_info(module_id);
        assert!(module_info.is_some());
        assert_eq!(module_info.unwrap().status, ModuleStatus::Healthy);
    }

    event_bus.shutdown().await.expect("Failed to shutdown");
}