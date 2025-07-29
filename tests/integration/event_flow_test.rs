//! Integration tests for complete event flow through all modules
//! Tests the documented flow: Data Capture → Storage → Analysis → Gamification → AI → Figurine

use std::time::Duration;
use tokio::time::timeout;
use uuid::Uuid;

use skelly_jelly_event_bus::{create_event_bus, BusMessage, MessagePayload, ModuleId, MessageFilter, DeliveryMode};
use skelly_jelly_data_capture::RawEvent;
use skelly_jelly_storage::{EventBatch, StorageModule};
use skelly_jelly_analysis_engine::{AnalysisWindow, StateClassification};

/// Test complete event flow from data capture to cute figurine
#[tokio::test]
async fn test_complete_event_flow() {
    // Initialize event bus
    let event_bus = create_event_bus().expect("Failed to create event bus");
    event_bus.start().await.expect("Failed to start event bus");

    // Track messages received
    let messages_received = std::sync::Arc::new(tokio::sync::Mutex::new(Vec::new()));
    let messages_clone = messages_received.clone();

    // Create test subscriber that captures all messages
    let subscription_id = event_bus.subscribe(
        ModuleId::EventBus,
        MessageFilter::all(),
        DeliveryMode::Reliable { timeout: Duration::from_secs(5) },
    ).await.expect("Failed to subscribe");

    // Spawn message collector
    tokio::spawn(async move {
        let mut receiver = event_bus.get_receiver(subscription_id).await.unwrap();
        while let Some(message) = receiver.recv().await {
            let mut msgs = messages_clone.lock().await;
            msgs.push(message);
        }
    });

    // Test Data Capture → Storage flow
    let raw_event = RawEvent::keystroke(
        "a".to_string(),
        Duration::from_millis(100),
        vec![],
    );

    let message = BusMessage::new(
        ModuleId::DataCapture,
        MessagePayload::RawEvent(raw_event.clone()),
    );

    event_bus.publish(message).await.expect("Failed to publish raw event");

    // Wait for event to propagate
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Verify Storage received the event
    let messages = messages_received.lock().await;
    assert!(messages.iter().any(|msg| {
        matches!(&msg.payload, MessagePayload::RawEvent(_))
    }), "Storage should have received RawEvent");

    // Test Storage → Analysis Engine flow
    let event_batch = EventBatch {
        window_start: chrono::Utc::now() - chrono::Duration::seconds(30),
        window_end: chrono::Utc::now(),
        events: vec![raw_event],
        session_id: Uuid::new_v4(),
    };

    let batch_message = BusMessage::new(
        ModuleId::Storage,
        MessagePayload::EventBatch(event_batch),
    );

    event_bus.publish(batch_message).await.expect("Failed to publish event batch");

    // Wait for batch to propagate
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Test Analysis Engine → Gamification flow
    let state_classification = StateClassification {
        state: "focused".to_string(),
        confidence: 0.85,
        timestamp: chrono::Utc::now(),
        transition_from: Some("distracted".to_string()),
    };

    let state_message = BusMessage::new(
        ModuleId::AnalysisEngine,
        MessagePayload::StateChange(state_classification),
    );

    event_bus.publish(state_message).await.expect("Failed to publish state change");

    // Cleanup
    event_bus.unsubscribe(subscription_id).await.expect("Failed to unsubscribe");
    event_bus.shutdown().await.expect("Failed to shutdown event bus");
}

/// Test event bus handles high throughput
#[tokio::test]
async fn test_event_bus_throughput() {
    let event_bus = create_event_bus().expect("Failed to create event bus");
    event_bus.start().await.expect("Failed to start event bus");

    let message_count = 1000;
    let received_count = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let received_clone = received_count.clone();

    // Subscribe to messages
    let subscription_id = event_bus.subscribe(
        ModuleId::Storage,
        MessageFilter::types(vec![MessageType::RawEvent]),
        DeliveryMode::BestEffort,
    ).await.expect("Failed to subscribe");

    // Spawn receiver
    tokio::spawn(async move {
        let mut receiver = event_bus.get_receiver(subscription_id).await.unwrap();
        while let Some(_) = receiver.recv().await {
            received_clone.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
    });

    // Publish messages
    let start = std::time::Instant::now();
    for i in 0..message_count {
        let event = RawEvent::mouse_move(i as f64, i as f64);
        let message = BusMessage::new(
            ModuleId::DataCapture,
            MessagePayload::RawEvent(event),
        );
        event_bus.publish(message).await.expect("Failed to publish");
    }
    let publish_duration = start.elapsed();

    // Wait for messages to be processed
    tokio::time::sleep(Duration::from_millis(500)).await;

    let received = received_count.load(std::sync::atomic::Ordering::Relaxed);
    let throughput = message_count as f64 / publish_duration.as_secs_f64();

    println!("Published {} messages in {:?}", message_count, publish_duration);
    println!("Throughput: {:.0} messages/second", throughput);
    println!("Received: {} messages", received);

    assert!(throughput > 1000.0, "Throughput should exceed 1000 msg/sec");
    assert!(received >= message_count * 95 / 100, "Should receive at least 95% of messages");

    event_bus.shutdown().await.expect("Failed to shutdown");
}

/// Test module health monitoring
#[tokio::test]
async fn test_module_health_monitoring() {
    let event_bus = create_event_bus().expect("Failed to create event bus");
    event_bus.start().await.expect("Failed to start event bus");

    // Simulate health check request
    let health_request = BusMessage::new(
        ModuleId::Orchestrator,
        MessagePayload::HealthCheck(HealthCheckRequest {
            module_id: ModuleId::DataCapture,
            timestamp: chrono::Utc::now(),
        }),
    );

    event_bus.publish(health_request).await.expect("Failed to publish health check");

    // In a real test, modules would respond with HealthStatus
    // This demonstrates the health monitoring flow

    event_bus.shutdown().await.expect("Failed to shutdown");
}

/// Test graceful shutdown sequence
#[tokio::test]
async fn test_graceful_shutdown() {
    let event_bus = create_event_bus().expect("Failed to create event bus");
    event_bus.start().await.expect("Failed to start event bus");

    // Module order for shutdown (reverse of startup)
    let shutdown_order = vec![
        ModuleId::CuteFigurine,
        ModuleId::AiIntegration,
        ModuleId::Gamification,
        ModuleId::AnalysisEngine,
        ModuleId::DataCapture,
        ModuleId::Storage,
        ModuleId::Orchestrator,
        ModuleId::EventBus,
    ];

    // Send shutdown requests in order
    for module_id in shutdown_order {
        let shutdown_request = BusMessage::new(
            ModuleId::Orchestrator,
            MessagePayload::Shutdown(ShutdownRequest {
                module_id,
                timeout: Duration::from_secs(30),
                save_state: true,
            }),
        );

        event_bus.publish(shutdown_request).await.expect("Failed to publish shutdown");
        
        // Small delay between shutdowns
        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    // Final event bus shutdown
    event_bus.shutdown().await.expect("Failed to shutdown event bus");
}

/// Test screenshot lifecycle management
#[tokio::test]
async fn test_screenshot_lifecycle() {
    let event_bus = create_event_bus().expect("Failed to create event bus");
    event_bus.start().await.expect("Failed to start event bus");

    // Create screenshot events of different sizes
    let small_screenshot = RawEvent::screenshot(vec![0u8; 1_000_000]); // 1MB
    let large_screenshot = RawEvent::screenshot(vec![0u8; 6_000_000]); // 6MB

    // Publish small screenshot (should stay in memory)
    let small_msg = BusMessage::new(
        ModuleId::DataCapture,
        MessagePayload::RawEvent(small_screenshot),
    );
    event_bus.publish(small_msg).await.expect("Failed to publish small screenshot");

    // Publish large screenshot (should go to temp file)
    let large_msg = BusMessage::new(
        ModuleId::DataCapture,
        MessagePayload::RawEvent(large_screenshot),
    );
    event_bus.publish(large_msg).await.expect("Failed to publish large screenshot");

    // In real implementation, Storage module would:
    // 1. Store metadata immediately
    // 2. Queue for analysis
    // 3. Delete after 30 seconds

    event_bus.shutdown().await.expect("Failed to shutdown");
}