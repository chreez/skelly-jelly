//! Failure handling and recovery tests for the event bus
//! Tests Task 1.1.4: Failure Handling

use std::time::Duration;

use skelly_jelly_event_bus::{
    create_event_bus, BusMessage, MessagePayload, ModuleId, MessageFilter, DeliveryMode,
    MessageType, message::RawEvent, EventBusTrait, ModuleInfo, ModuleStatus, SystemHealth,
};

/// Test Task 1.1.4a: Module crash recovery
#[tokio::test]
async fn test_module_crash_recovery() {
    let event_bus = create_event_bus().expect("Failed to create event bus");
    event_bus.start().await.expect("Failed to start event bus");

    // Register modules
    let modules = vec![
        ModuleInfo::new(ModuleId::DataCapture),
        ModuleInfo::new(ModuleId::Storage),
        ModuleInfo::new(ModuleId::AnalysisEngine),
    ];

    for module_info in modules {
        let module_id = module_info.module_id;
        assert!(event_bus.register_module(module_info).is_ok());
        assert!(event_bus.registry().mark_module_ready(module_id).is_ok());
    }

    // Verify all modules are healthy
    let health = event_bus.registry().get_health_summary();
    assert_eq!(health.overall_health, SystemHealth::Healthy);
    assert_eq!(health.healthy_count, 3);

    // Simulate module crash
    assert!(event_bus.registry().update_module_status(
        ModuleId::Storage,
        ModuleStatus::Unhealthy,
        None,
    ).is_ok());

    // System should detect unhealthy module
    let health = event_bus.registry().get_health_summary();
    assert_eq!(health.overall_health, SystemHealth::Critical);
    assert_eq!(health.unhealthy_count, 1);

    // Simulate recovery sequence
    // 1. Mark as degraded (partial recovery)
    assert!(event_bus.registry().update_module_status(
        ModuleId::Storage,
        ModuleStatus::Degraded,
        Some(Duration::from_millis(200)),
    ).is_ok());

    let health = event_bus.registry().get_health_summary();
    assert_eq!(health.overall_health, SystemHealth::Degraded);

    // 2. Full recovery
    assert!(event_bus.registry().update_module_status(
        ModuleId::Storage,
        ModuleStatus::Healthy,
        Some(Duration::from_millis(50)),
    ).is_ok());

    let health = event_bus.registry().get_health_summary();
    assert_eq!(health.overall_health, SystemHealth::Healthy);
    assert_eq!(health.healthy_count, 3);

    event_bus.shutdown().await.expect("Failed to shutdown");
}

/// Test Task 1.1.4b: Message delivery failure scenarios
#[tokio::test]
async fn test_message_delivery_failures() {
    let event_bus = create_event_bus().expect("Failed to create event bus");
    event_bus.start().await.expect("Failed to start event bus");

    // Create subscription that will have delivery issues
    let subscription_id = event_bus.subscribe(
        ModuleId::Storage,
        MessageFilter::types(vec![MessageType::RawEvent]),
        DeliveryMode::Reliable { timeout: Duration::from_millis(100) },
    ).await.expect("Failed to subscribe");

    // Test 1: Normal message delivery
    let raw_event = RawEvent::keystroke(
        "test".to_string(),
        Duration::from_millis(10),
        vec![],
    );
    let message = BusMessage::new(
        ModuleId::DataCapture,
        MessagePayload::RawEvent(raw_event),
    );

    // This should succeed
    let result = event_bus.publish(message).await;
    assert!(result.is_ok());

    // Test 2: Queue saturation scenario
    // Publish many messages rapidly to test backpressure
    let mut success_count = 0;
    let mut failure_count = 0;

    for i in 0..1000 {
        let raw_event = RawEvent::mouse_move(i as f64, i as f64);
        let message = BusMessage::new(
            ModuleId::DataCapture,
            MessagePayload::RawEvent(raw_event),
        );

        match event_bus.publish(message).await {
            Ok(_) => success_count += 1,
            Err(_) => failure_count += 1,
        }
    }

    println!("Message delivery test: {} succeeded, {} failed", success_count, failure_count);

    // Should handle at least some messages successfully
    assert!(success_count > 0, "No messages were delivered successfully");

    // Test proper error handling
    if failure_count > 0 {
        println!("Backpressure handling working: {} messages rejected", failure_count);
    }

    event_bus.unsubscribe(subscription_id).await.expect("Failed to unsubscribe");
    event_bus.shutdown().await.expect("Failed to shutdown");
}

/// Test Task 1.1.4c: Network partition simulation
#[tokio::test]
async fn test_network_partition_handling() {
    let event_bus = create_event_bus().expect("Failed to create event bus");
    event_bus.start().await.expect("Failed to start event bus");

    // Register distributed modules
    let modules = vec![
        ModuleInfo::new(ModuleId::DataCapture).with_metadata("location".to_string(), "local".to_string()),
        ModuleInfo::new(ModuleId::Storage).with_metadata("location".to_string(), "remote".to_string()),
        ModuleInfo::new(ModuleId::AnalysisEngine).with_metadata("location".to_string(), "remote".to_string()),
    ];

    for module_info in modules {
        let module_id = module_info.module_id;
        assert!(event_bus.register_module(module_info).is_ok());
        assert!(event_bus.registry().mark_module_ready(module_id).is_ok());
    }

    let initial_health = event_bus.registry().get_health_summary();
    assert_eq!(initial_health.healthy_count, 3);

    // Simulate network partition - remote modules become unreachable
    assert!(event_bus.registry().update_module_status(
        ModuleId::Storage,
        ModuleStatus::Unhealthy,
        None,
    ).is_ok());

    assert!(event_bus.registry().update_module_status(
        ModuleId::AnalysisEngine,
        ModuleStatus::Unhealthy,
        None,
    ).is_ok());

    // System should detect the partition
    let partitioned_health = event_bus.registry().get_health_summary();
    assert_eq!(partitioned_health.overall_health, SystemHealth::Critical);
    assert_eq!(partitioned_health.unhealthy_count, 2);
    assert_eq!(partitioned_health.healthy_count, 1); // Only local module

    // Local module should still function
    let local_module = event_bus.registry().get_module_info(ModuleId::DataCapture).unwrap();
    assert_eq!(local_module.status, ModuleStatus::Healthy);

    // Simulate partition recovery
    tokio::time::sleep(Duration::from_millis(10)).await;

    // Remote modules come back online
    assert!(event_bus.registry().update_module_status(
        ModuleId::Storage,
        ModuleStatus::Healthy,
        Some(Duration::from_millis(100)),
    ).is_ok());

    assert!(event_bus.registry().update_module_status(
        ModuleId::AnalysisEngine,
        ModuleStatus::Healthy,
        Some(Duration::from_millis(120)),
    ).is_ok());

    // System should recover
    let recovered_health = event_bus.registry().get_health_summary();
    assert_eq!(recovered_health.overall_health, SystemHealth::Healthy);
    assert_eq!(recovered_health.healthy_count, 3);

    event_bus.shutdown().await.expect("Failed to shutdown");
}

/// Test Task 1.1.4d: Resource exhaustion recovery
#[tokio::test]
async fn test_resource_exhaustion_recovery() {
    let event_bus = create_event_bus().expect("Failed to create event bus");
    event_bus.start().await.expect("Failed to start event bus");

    // Register modules with different resource profiles
    let modules = vec![
        ModuleInfo::new(ModuleId::DataCapture)
            .with_metadata("memory_limit".to_string(), "100MB".to_string()),
        ModuleInfo::new(ModuleId::Storage)
            .with_metadata("memory_limit".to_string(), "500MB".to_string()),
        ModuleInfo::new(ModuleId::AnalysisEngine)
            .with_metadata("memory_limit".to_string(), "1GB".to_string()),
    ];

    for module_info in modules {
        let module_id = module_info.module_id;
        assert!(event_bus.register_module(module_info).is_ok());
        assert!(event_bus.registry().mark_module_ready(module_id).is_ok());
    }

    // Simulate memory pressure on Analysis Engine
    assert!(event_bus.registry().update_module_status(
        ModuleId::AnalysisEngine,
        ModuleStatus::Degraded,
        Some(Duration::from_millis(300)), // Slow response due to memory pressure
    ).is_ok());

    let health = event_bus.registry().get_health_summary();
    assert_eq!(health.overall_health, SystemHealth::Degraded);

    // Test that system continues to function under resource pressure
    let raw_event = RawEvent::keystroke(
        "test_under_pressure".to_string(),
        Duration::from_millis(10),
        vec![],
    );
    let message = BusMessage::new(
        ModuleId::DataCapture,
        MessagePayload::RawEvent(raw_event),
    );

    // Message should still be publishable despite degraded state
    let result = event_bus.publish(message).await;
    assert!(result.is_ok());

    // Simulate resource recovery (garbage collection, memory freed)
    tokio::time::sleep(Duration::from_millis(10)).await;

    assert!(event_bus.registry().update_module_status(
        ModuleId::AnalysisEngine,
        ModuleStatus::Healthy,
        Some(Duration::from_millis(50)),
    ).is_ok());

    let recovered_health = event_bus.registry().get_health_summary();
    assert_eq!(recovered_health.overall_health, SystemHealth::Healthy);

    event_bus.shutdown().await.expect("Failed to shutdown");
}

/// Test cascading failure prevention
#[tokio::test]
async fn test_cascading_failure_prevention() {
    let event_bus = create_event_bus().expect("Failed to create event bus");
    event_bus.start().await.expect("Failed to start event bus");

    // Register modules in dependency chain
    let modules = vec![
        ModuleInfo::new(ModuleId::DataCapture),
        ModuleInfo::new(ModuleId::Storage),        // Depends on DataCapture
        ModuleInfo::new(ModuleId::AnalysisEngine), // Depends on Storage
        ModuleInfo::new(ModuleId::Gamification),   // Depends on AnalysisEngine
    ];

    for module_info in modules {
        let module_id = module_info.module_id;
        assert!(event_bus.register_module(module_info).is_ok());
        assert!(event_bus.registry().mark_module_ready(module_id).is_ok());
    }

    let initial_health = event_bus.registry().get_health_summary();
    assert_eq!(initial_health.healthy_count, 4);

    // Storage module fails
    assert!(event_bus.registry().update_module_status(
        ModuleId::Storage,
        ModuleStatus::Unhealthy,
        None,
    ).is_ok());

    // Should not cause immediate cascade - other modules should remain healthy
    let after_storage_failure = event_bus.registry().get_health_summary();
    assert_eq!(after_storage_failure.unhealthy_count, 1);
    assert_eq!(after_storage_failure.healthy_count, 3); // Others still healthy

    // DataCapture should still function independently
    let data_capture = event_bus.registry().get_module_info(ModuleId::DataCapture).unwrap();
    assert_eq!(data_capture.status, ModuleStatus::Healthy);

    // Test that messages can still be published despite downstream failure
    let raw_event = RawEvent::keystroke(
        "cascade_test".to_string(),
        Duration::from_millis(10),
        vec![],
    );
    let message = BusMessage::new(
        ModuleId::DataCapture,
        MessagePayload::RawEvent(raw_event),
    );

    let result = event_bus.publish(message).await;
    assert!(result.is_ok());

    // Recovery should be possible
    assert!(event_bus.registry().update_module_status(
        ModuleId::Storage,
        ModuleStatus::Healthy,
        Some(Duration::from_millis(50)),
    ).is_ok());

    let recovered_health = event_bus.registry().get_health_summary();
    assert_eq!(recovered_health.overall_health, SystemHealth::Healthy);
    assert_eq!(recovered_health.healthy_count, 4);

    event_bus.shutdown().await.expect("Failed to shutdown");
}

/// Test graceful degradation under failures
#[tokio::test]
async fn test_graceful_degradation() {
    let event_bus = create_event_bus().expect("Failed to create event bus");
    event_bus.start().await.expect("Failed to start event bus");

    // Register all system modules
    let all_modules = vec![
        ModuleId::EventBus,
        ModuleId::Orchestrator,
        ModuleId::DataCapture,
        ModuleId::Storage,
        ModuleId::AnalysisEngine,
        ModuleId::Gamification,
        ModuleId::AiIntegration,
        ModuleId::CuteFigurine,
    ];

    for module_id in &all_modules {
        let module_info = ModuleInfo::new(*module_id);
        assert!(event_bus.register_module(module_info).is_ok());
        assert!(event_bus.registry().mark_module_ready(*module_id).is_ok());
    }

    let initial_health = event_bus.registry().get_health_summary();
    assert_eq!(initial_health.healthy_count, 8);
    assert_eq!(initial_health.overall_health, SystemHealth::Healthy);

    // Simulate multiple module failures
    assert!(event_bus.registry().update_module_status(
        ModuleId::AiIntegration,
        ModuleStatus::Unhealthy,
        None,
    ).is_ok());

    assert!(event_bus.registry().update_module_status(
        ModuleId::Gamification,
        ModuleStatus::Degraded,
        Some(Duration::from_millis(400)),
    ).is_ok());

    // System should degrade gracefully
    let degraded_health = event_bus.registry().get_health_summary();
    assert_eq!(degraded_health.overall_health, SystemHealth::Critical); // Due to unhealthy module
    assert_eq!(degraded_health.unhealthy_count, 1);
    assert_eq!(degraded_health.degraded_count, 1);
    assert_eq!(degraded_health.healthy_count, 6);

    // Core functionality should still work
    let raw_event = RawEvent::keystroke(
        "degraded_mode".to_string(),
        Duration::from_millis(10),
        vec![],
    );
    let message = BusMessage::new(
        ModuleId::DataCapture,
        MessagePayload::RawEvent(raw_event),
    );

    let result = event_bus.publish(message).await;
    assert!(result.is_ok(), "Core functionality should work in degraded mode");

    // Test that essential modules remain operational
    for essential_module in [ModuleId::EventBus, ModuleId::DataCapture, ModuleId::Storage] {
        let module_info = event_bus.registry().get_module_info(essential_module).unwrap();
        assert_eq!(module_info.status, ModuleStatus::Healthy);
    }

    event_bus.shutdown().await.expect("Failed to shutdown");
}

/// Test error propagation and handling
#[tokio::test]
async fn test_error_propagation() {
    let event_bus = create_event_bus().expect("Failed to create event bus");
    event_bus.start().await.expect("Failed to start event bus");

    // Test invalid module operations
    let result = event_bus.registry().get_module_info(ModuleId::DataCapture);
    assert!(result.is_none(), "Should return None for unregistered module");

    // Test status updates for non-existent modules
    let result = event_bus.registry().update_module_status(
        ModuleId::DataCapture,
        ModuleStatus::Healthy,
        None,
    );
    assert!(result.is_err(), "Should error for non-existent module");

    // Test duplicate registration
    let module_info = ModuleInfo::new(ModuleId::Storage);
    assert!(event_bus.register_module(module_info.clone()).is_ok());
    
    let result = event_bus.register_module(module_info);
    assert!(result.is_err(), "Should error on duplicate registration");

    event_bus.shutdown().await.expect("Failed to shutdown");
}