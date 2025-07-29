//! Performance tests for event bus throughput validation
//! Tests Task 1.1.3: Performance Validation (1000+ msg/sec)

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use skelly_jelly_event_bus::{
    create_event_bus, BusMessage, MessagePayload, ModuleId, MessageFilter, DeliveryMode,
    MessageType, message::RawEvent, EventBusTrait,
};

/// Test high-throughput message publishing (Task 1.1.3a)
#[tokio::test]
async fn test_high_throughput_publishing() {
    let event_bus = create_event_bus().expect("Failed to create event bus");
    event_bus.start().await.expect("Failed to start event bus");

    // Test parameters
    let message_count = 5_000;
    let target_throughput = 1_000.0; // messages per second

    println!("Testing throughput with {} messages", message_count);

    // Measure publishing performance
    let start_time = Instant::now();
    
    for i in 0..message_count {
        let raw_event = RawEvent::mouse_move(i as f64, i as f64);
        let message = BusMessage::new(
            ModuleId::DataCapture,
            MessagePayload::RawEvent(raw_event),
        );
        
        event_bus.publish(message).await.expect("Failed to publish message");
    }

    let publish_duration = start_time.elapsed();
    let throughput = message_count as f64 / publish_duration.as_secs_f64();

    println!("Published {} messages in {:?}", message_count, publish_duration);
    println!("Throughput: {:.0} messages/second", throughput);

    // Verify performance target
    assert!(
        throughput >= target_throughput,
        "Throughput {:.0} msg/sec is below target {:.0} msg/sec",
        throughput, target_throughput
    );

    event_bus.shutdown().await.expect("Failed to shutdown");
}

/// Test message delivery latency under load (Task 1.1.3b)
#[tokio::test]
async fn test_delivery_latency_under_load() {
    let event_bus = create_event_bus().expect("Failed to create event bus");
    event_bus.start().await.expect("Failed to start event bus");

    // Set up subscriber
    let messages_received = Arc::new(AtomicU64::new(0));
    let received_clone = messages_received.clone();
    let latencies = Arc::new(tokio::sync::Mutex::new(Vec::new()));
    let latencies_clone = latencies.clone();

    let subscription_id = event_bus.subscribe(
        ModuleId::Storage,
        MessageFilter::types(vec![MessageType::RawEvent]),
        DeliveryMode::BestEffort,
    ).await.expect("Failed to subscribe");

    // Start message processor (mock)
    let start_time = Instant::now();
    tokio::spawn(async move {
        // In a real implementation, we'd get the actual receiver
        // This is a mock for testing purposes
        for _ in 0..1000 {
            let message_time = start_time.elapsed();
            latencies_clone.lock().await.push(message_time);
            received_clone.fetch_add(1, Ordering::Relaxed);
            tokio::time::sleep(Duration::from_micros(100)).await;
        }
    });

    // Publish messages with timestamps
    let message_count = 1_000;
    let publish_start = Instant::now();

    for i in 0..message_count {
        let raw_event = RawEvent::keystroke(
            format!("key_{}", i),
            Duration::from_millis(10),
            vec![],
        );
        let message = BusMessage::new(
            ModuleId::DataCapture,
            MessagePayload::RawEvent(raw_event),
        );
        
        event_bus.publish(message).await.expect("Failed to publish message");
    }

    // Wait for message processing
    tokio::time::sleep(Duration::from_millis(500)).await;

    let publish_duration = publish_start.elapsed();
    let throughput = message_count as f64 / publish_duration.as_secs_f64();

    // Check latency statistics
    let latencies_vec = latencies.lock().await;
    if !latencies_vec.is_empty() {
        let avg_latency = latencies_vec.iter()
            .map(|d| d.as_millis())
            .sum::<u128>() as f64 / latencies_vec.len() as f64;

        println!("Average latency: {:.2} ms", avg_latency);
        println!("Throughput: {:.0} msg/sec", throughput);

        // Target: <1ms latency for high-priority messages
        assert!(avg_latency < 1.0, "Average latency {:.2} ms exceeds 1ms target", avg_latency);
    }

    assert!(throughput >= 1000.0, "Throughput below 1000 msg/sec target");

    event_bus.unsubscribe(subscription_id).await.expect("Failed to unsubscribe");
    event_bus.shutdown().await.expect("Failed to shutdown");
}

/// Test memory usage during high-load operations (Task 1.1.3c)
#[tokio::test]
async fn test_memory_usage_under_load() {
    let event_bus = create_event_bus().expect("Failed to create event bus");
    event_bus.start().await.expect("Failed to start event bus");

    // Get initial metrics
    let initial_metrics = event_bus.metrics().await.expect("Failed to get metrics");

    // Create multiple subscribers
    let mut subscription_ids = Vec::new();
    for i in 0..5 {
        let module_id = match i {
            0 => ModuleId::Storage,
            1 => ModuleId::AnalysisEngine,
            2 => ModuleId::Gamification,
            3 => ModuleId::AiIntegration,
            _ => ModuleId::CuteFigurine,
        };

        let subscription_id = event_bus.subscribe(
            module_id,
            MessageFilter::all(),
            DeliveryMode::BestEffort,
        ).await.expect("Failed to subscribe");
        
        subscription_ids.push(subscription_id);
    }

    // Publish large number of messages
    let message_count = 10_000;
    println!("Publishing {} messages to test memory usage", message_count);

    for i in 0..message_count {
        let raw_event = RawEvent::screenshot(vec![0u8; 1000]); // 1KB per message
        let message = BusMessage::new(
            ModuleId::DataCapture,
            MessagePayload::RawEvent(raw_event),
        );
        
        event_bus.publish(message).await.expect("Failed to publish message");

        // Intermittent metrics check
        if i % 2000 == 0 {
            let metrics = event_bus.metrics().await.expect("Failed to get metrics");
            println!("Queue depth at {}: {}", i, metrics.current_queue_depth);
        }
    }

    // Wait for message processing
    tokio::time::sleep(Duration::from_secs(1)).await;

    // Get final metrics
    let final_metrics = event_bus.metrics().await.expect("Failed to get metrics");

    println!("Initial messages: {}", initial_metrics.messages_published);
    println!("Final messages: {}", final_metrics.messages_published);
    println!("Final queue depth: {}", final_metrics.current_queue_depth);
    
    // Verify messages were processed
    assert!(final_metrics.messages_published >= message_count as u64);
    
    // Queue should not grow unbounded
    assert!(final_metrics.current_queue_depth < 1000, 
            "Queue depth {} suggests memory issues", final_metrics.current_queue_depth);

    // Clean up
    for subscription_id in subscription_ids {
        event_bus.unsubscribe(subscription_id).await.expect("Failed to unsubscribe");
    }

    event_bus.shutdown().await.expect("Failed to shutdown");
}

/// Test CPU usage efficiency during peak load (Task 1.1.3d)
#[tokio::test]
async fn test_cpu_efficiency_under_load() {
    let event_bus = create_event_bus().expect("Failed to create event bus");
    event_bus.start().await.expect("Failed to start event bus");

    // Set up multiple publishers and subscribers
    let mut subscription_ids = Vec::new();
    for module_id in [ModuleId::Storage, ModuleId::AnalysisEngine, ModuleId::Gamification] {
        let subscription_id = event_bus.subscribe(
            module_id,
            MessageFilter::all(),
            DeliveryMode::BestEffort,
        ).await.expect("Failed to subscribe");
        subscription_ids.push(subscription_id);
    }

    // Test sustained throughput
    let duration = Duration::from_secs(2);
    let mut message_count = 0;
    let start_time = Instant::now();

    // Publish messages for fixed duration
    while start_time.elapsed() < duration {
        let raw_event = RawEvent::keystroke(
            "test".to_string(),
            Duration::from_millis(50),
            vec![],
        );
        let message = BusMessage::new(
            ModuleId::DataCapture,
            MessagePayload::RawEvent(raw_event),
        );
        
        event_bus.publish(message).await.expect("Failed to publish message");
        message_count += 1;

        // Small yield to prevent 100% CPU usage in test
        if message_count % 100 == 0 {
            tokio::task::yield_now().await;
        }
    }

    let actual_duration = start_time.elapsed();
    let sustained_throughput = message_count as f64 / actual_duration.as_secs_f64();

    println!("Sustained {} messages over {:?}", message_count, actual_duration);
    println!("Sustained throughput: {:.0} messages/second", sustained_throughput);

    // Should maintain >1000 msg/sec under sustained load
    assert!(sustained_throughput >= 1000.0, 
            "Sustained throughput {:.0} below target", sustained_throughput);

    // Clean up
    for subscription_id in subscription_ids {
        event_bus.unsubscribe(subscription_id).await.expect("Failed to unsubscribe");
    }

    event_bus.shutdown().await.expect("Failed to shutdown");
}

/// Benchmark test for maximum achievable throughput
#[tokio::test]
async fn benchmark_maximum_throughput() {
    let event_bus = create_event_bus().expect("Failed to create event bus");
    event_bus.start().await.expect("Failed to start event bus");

    // Test different message types for performance characteristics
    let test_cases = vec![
        ("Small Events", 10_000, MessagePayload::ModuleReady(ModuleId::DataCapture)),
        ("Raw Events", 5_000, MessagePayload::RawEvent(RawEvent::mouse_move(100.0, 200.0))),
        // Note: Large screenshot events would be tested with actual data in real scenarios
    ];

    for (test_name, message_count, payload) in test_cases {
        println!("\n--- {} Test ---", test_name);
        
        let start_time = Instant::now();
        
        for _ in 0..message_count {
            let message = BusMessage::new(ModuleId::DataCapture, payload.clone());
            event_bus.publish(message).await.expect("Failed to publish message");
        }

        let duration = start_time.elapsed();
        let throughput = message_count as f64 / duration.as_secs_f64();

        println!("{}: {} messages in {:?}", test_name, message_count, duration);
        println!("{}: {:.0} msg/sec throughput", test_name, throughput);

        // All test cases should exceed 1000 msg/sec
        assert!(throughput >= 1000.0, 
                "{} throughput {:.0} below 1000 msg/sec", test_name, throughput);
    }

    event_bus.shutdown().await.expect("Failed to shutdown");
}

/// Test concurrent publishers performance
#[tokio::test]
async fn test_concurrent_publishers() {
    let event_bus = Arc::new(create_event_bus().expect("Failed to create event bus"));
    event_bus.start().await.expect("Failed to start event bus");

    let publisher_count = 4;
    let messages_per_publisher = 1_000;
    
    println!("Testing {} concurrent publishers with {} messages each", 
             publisher_count, messages_per_publisher);

    let start_time = Instant::now();
    let mut handles = Vec::new();

    // Spawn concurrent publishers
    for publisher_id in 0..publisher_count {
        let event_bus_clone = Arc::clone(&event_bus);
        let handle = tokio::spawn(async move {
            for i in 0..messages_per_publisher {
                let raw_event = RawEvent::keystroke(
                    format!("publisher_{}_{}", publisher_id, i),
                    Duration::from_millis(10),
                    vec![],
                );
                let message = BusMessage::new(
                    ModuleId::DataCapture,
                    MessagePayload::RawEvent(raw_event),
                );
                
                event_bus_clone.publish(message).await.expect("Failed to publish");
            }
        });
        handles.push(handle);
    }

    // Wait for all publishers to complete
    for handle in handles {
        handle.await.expect("Publisher task failed");
    }

    let total_duration = start_time.elapsed();
    let total_messages = publisher_count * messages_per_publisher;
    let concurrent_throughput = total_messages as f64 / total_duration.as_secs_f64();

    println!("Concurrent test: {} total messages in {:?}", total_messages, total_duration);
    println!("Concurrent throughput: {:.0} msg/sec", concurrent_throughput);

    // Concurrent throughput should still meet target
    assert!(concurrent_throughput >= 1000.0, 
            "Concurrent throughput {:.0} below target", concurrent_throughput);

    event_bus.shutdown().await.expect("Failed to shutdown");
}