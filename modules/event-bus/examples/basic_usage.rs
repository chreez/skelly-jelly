//! Basic usage example for the event bus
//! 
//! This example demonstrates:
//! - Creating an event bus
//! - Publishing messages
//! - Subscribing to message types
//! - Collecting metrics

use skelly_jelly_event_bus::{
    create_event_bus, BusMessage, MessagePayload, MessageType, ModuleId,
    MessageFilter, DeliveryMode, EventBusTrait
};
use skelly_jelly_event_bus::message::RawEvent;
use chrono::Utc;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🚀 Starting Event Bus Example");

    // Create and start the event bus
    let bus = create_event_bus()?;
    bus.start().await?;
    println!("✅ Event bus started");

    // Create a subscription for RawEvent messages
    let filter = MessageFilter::types(vec![MessageType::RawEvent]);
    let subscription_id = bus.subscribe(
        ModuleId::Storage,
        filter,
        DeliveryMode::BestEffort,
    ).await?;
    println!("📬 Created subscription for RawEvent messages");

    // Create another subscription for all messages from DataCapture
    let filter = MessageFilter::sources(vec![ModuleId::DataCapture]);
    let subscription_id_2 = bus.subscribe(
        ModuleId::AnalysisEngine,
        filter,
        DeliveryMode::Reliable { timeout: Duration::from_secs(5) },
    ).await?;
    println!("📬 Created subscription for DataCapture messages");

    // Publish some messages
    println!("\n📤 Publishing messages...");

    for i in 0..5 {
        let raw_event = RawEvent {
            event_type: format!("test_event_{}", i),
            data: serde_json::json!({
                "sequence": i,
                "timestamp": Utc::now(),
                "data": format!("test data {}", i)
            }),
            window_title: Some(format!("Test Window {}", i)),
            timestamp: Utc::now(),
        };

        let message = BusMessage::new(
            ModuleId::DataCapture,
            MessagePayload::RawEvent(raw_event),
        );

        let message_id = bus.publish(message).await?;
        println!("  ✉️  Published message {} (sequence {})", message_id, i);
        
        // Small delay between messages
        sleep(Duration::from_millis(100)).await;
    }

    // Publish a module ready message
    let ready_message = BusMessage::new(
        ModuleId::Storage,
        MessagePayload::ModuleReady(ModuleId::Storage),
    );
    let ready_id = bus.publish(ready_message).await?;
    println!("  ✉️  Published ModuleReady message {}", ready_id);

    // Wait a moment for processing
    sleep(Duration::from_millis(500)).await;

    // Get metrics
    println!("\n📊 Bus Metrics:");
    let metrics = bus.metrics().await?;
    println!("  📈 Messages published: {}", metrics.messages_published);
    println!("  📈 Messages delivered: {}", metrics.messages_delivered);
    println!("  📈 Messages failed: {}", metrics.messages_failed);
    println!("  📈 Current queue depth: {}", metrics.current_queue_depth);
    println!("  📈 P95 delivery latency: {:.2}ms", metrics.delivery_latency.p95_ms);
    
    // Show per-module stats
    println!("\n🏢 Per-Module Statistics:");
    for (module, stats) in &metrics.module_stats {
        if stats.messages_published > 0 || stats.messages_received > 0 {
            println!("  {} - Published: {}, Received: {}, Subscriptions: {}", 
                     module, stats.messages_published, stats.messages_received, stats.subscriptions_active);
        }
    }

    // Show per-message-type stats
    println!("\n📧 Per-Message-Type Statistics:");
    for (msg_type, stats) in &metrics.message_type_stats {
        if stats.count > 0 {
            println!("  {:?} - Count: {}, Avg Size: {} bytes, Avg Latency: {:.2}ms", 
                     msg_type, stats.count, stats.avg_size_bytes, stats.avg_latency_ms);
        }
    }

    // Clean up subscriptions
    println!("\n🧹 Cleaning up...");
    bus.unsubscribe(subscription_id).await?;
    bus.unsubscribe(subscription_id_2).await?;
    println!("  🗑️  Removed subscriptions");

    // Shutdown the bus
    bus.shutdown().await?;
    println!("  🛑 Event bus shutdown complete");

    println!("\n🎉 Example completed successfully!");
    Ok(())
}