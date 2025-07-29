//! Integration Demo - Shows all modules working together
//! 
//! This demonstrates the complete data flow through the Skelly-Jelly system:
//! 1. Data capture generates synthetic events
//! 2. Storage batches and persists them
//! 3. Analysis engine classifies ADHD states
//! 4. Gamification decides on interventions
//! 5. AI generates personalized messages
//! 6. Cute figurine displays animations

use anyhow::{Context, Result};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, debug};

// Simplified type imports (would come from actual modules)
use skelly_jelly_event_bus::{EventBus, BusMessage, ModuleId};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("info,skelly_jelly=debug")
        .init();
    
    info!("🦴 Skelly-Jelly Integration Demo Starting...");
    info!("This demo simulates the complete system workflow");
    
    // Create event bus
    let event_bus = Arc::new(EventBus::new_in_memory().await?);
    
    // Simulate module initialization
    info!("📦 Initializing modules...");
    sleep(Duration::from_millis(500)).await;
    
    // Demo scenario: User working, getting distracted, then recovering
    run_demo_scenario(event_bus).await?;
    
    info!("✨ Demo complete! Your skeleton friend waves goodbye 👋💀");
    
    Ok(())
}

async fn run_demo_scenario(event_bus: Arc<EventBus>) -> Result<()> {
    info!("\n🎬 Starting demo scenario...\n");
    
    // Phase 1: Focused work
    info!("Phase 1: User is focused and typing steadily");
    simulate_focused_work(&event_bus).await?;
    sleep(Duration::from_secs(2)).await;
    
    // Phase 2: Getting distracted
    info!("\nPhase 2: User starts getting distracted");
    simulate_distraction(&event_bus).await?;
    sleep(Duration::from_secs(2)).await;
    
    // Phase 3: Intervention and recovery
    info!("\nPhase 3: System provides intervention");
    simulate_intervention(&event_bus).await?;
    sleep(Duration::from_secs(2)).await;
    
    // Phase 4: Back to flow
    info!("\nPhase 4: User returns to flow state");
    simulate_flow_state(&event_bus).await?;
    sleep(Duration::from_secs(2)).await;
    
    // Phase 5: Celebration
    info!("\nPhase 5: Celebrating progress!");
    simulate_celebration(&event_bus).await?;
    
    Ok(())
}

async fn simulate_focused_work(event_bus: &Arc<EventBus>) -> Result<()> {
    // Data Capture → Event Bus
    debug!("📊 [Data Capture] Detecting steady typing pattern...");
    let keystroke_event = create_keystroke_event(100, 5); // 100ms intervals, low variance
    event_bus.publish(BusMessage::RawEvent(keystroke_event)).await?;
    
    // Storage → Event Bus
    debug!("💾 [Storage] Batching events...");
    sleep(Duration::from_millis(300)).await;
    
    // Analysis Engine → Event Bus
    debug!("🧠 [Analysis Engine] Analyzing behavioral patterns...");
    let state = create_state_event("Flow", 0.85);
    event_bus.publish(BusMessage::StateChange(state)).await?;
    
    // Gamification → Event Bus
    debug!("🎮 [Gamification] Flow state detected, no intervention needed");
    
    // Cute Figurine
    info!("💀 [Skeleton] *glows softly with contentment* 😊");
    
    Ok(())
}

async fn simulate_distraction(event_bus: &Arc<EventBus>) -> Result<()> {
    // Simulate erratic behavior
    debug!("📊 [Data Capture] Detecting irregular patterns...");
    let keystroke_event = create_keystroke_event(500, 200); // Irregular intervals
    event_bus.publish(BusMessage::RawEvent(keystroke_event)).await?;
    
    debug!("📊 [Data Capture] High window switching detected");
    let window_event = create_window_switch_event(5); // 5 switches in short time
    event_bus.publish(BusMessage::RawEvent(window_event)).await?;
    
    sleep(Duration::from_millis(300)).await;
    
    // Analysis detects distraction
    debug!("🧠 [Analysis Engine] Pattern indicates distraction");
    let state = create_state_event("Distracted", 0.78);
    event_bus.publish(BusMessage::StateChange(state)).await?;
    
    // Cute Figurine shows concern
    info!("💀 [Skeleton] *looks a bit wobbly and concerned* 😟");
    
    Ok(())
}

async fn simulate_intervention(event_bus: &Arc<EventBus>) -> Result<()> {
    // Gamification decides to intervene
    debug!("🎮 [Gamification] Distraction persisting, intervention recommended");
    let intervention_request = create_intervention_request("gentle_nudge");
    event_bus.publish(BusMessage::InterventionRequest(intervention_request)).await?;
    
    sleep(Duration::from_millis(200)).await;
    
    // AI generates message
    debug!("🤖 [AI Integration] Generating personalized message...");
    info!("💬 [AI] \"Hey there! Looks like your mind wandered a bit. No worries - happens to the best of us! Maybe try breaking your task into smaller chunks? 🦴\"");
    
    // Animation command
    let animation = create_animation_command("gentle_wave", 3000);
    event_bus.publish(BusMessage::AnimationCommand(animation)).await?;
    
    // Skeleton animation
    info!("💀 [Skeleton] *waves gently and offers encouraging smile* 👋😊");
    
    Ok(())
}

async fn simulate_flow_state(event_bus: &Arc<EventBus>) -> Result<()> {
    // User responds positively to intervention
    debug!("📊 [Data Capture] Detecting improved focus patterns");
    let keystroke_event = create_keystroke_event(80, 10); // Very steady typing
    event_bus.publish(BusMessage::RawEvent(keystroke_event)).await?;
    
    sleep(Duration::from_millis(300)).await;
    
    // Deep flow detected
    debug!("🧠 [Analysis Engine] Excellent flow state detected!");
    let state = create_state_event("Flow", 0.92);
    event_bus.publish(BusMessage::StateChange(state)).await?;
    
    // Gamification tracks progress
    debug!("🎮 [Gamification] Recording flow session, preparing reward");
    
    // Happy skeleton
    info!("💀 [Skeleton] *glows brightly with joy* ✨😄");
    
    Ok(())
}

async fn simulate_celebration(event_bus: &Arc<EventBus>) -> Result<()> {
    // Milestone reached
    debug!("🎮 [Gamification] 30-minute focus session completed!");
    let intervention_request = create_intervention_request("celebration");
    event_bus.publish(BusMessage::InterventionRequest(intervention_request)).await?;
    
    // Reward granted
    debug!("🎮 [Gamification] +50 focus coins earned! 🪙");
    
    // AI celebrates
    debug!("🤖 [AI Integration] Generating celebration message...");
    info!("💬 [AI] \"Amazing work! You just crushed a 30-minute focus session! Your skeleton friend is doing a happy dance! 🎉💀\"");
    
    // Celebration animation
    let animation = create_animation_command("happy_dance", 5000);
    event_bus.publish(BusMessage::AnimationCommand(animation)).await?;
    
    // Skeleton celebrates
    info!("💀 [Skeleton] *does an enthusiastic skeleton dance* 🕺💀✨");
    info!("🎉 Achievement Unlocked: Focus Champion!");
    
    Ok(())
}

// Helper functions to create mock events
fn create_keystroke_event(interval_ms: u32, variance: u32) -> serde_json::Value {
    serde_json::json!({
        "type": "keystroke",
        "timestamp": chrono::Utc::now(),
        "data": {
            "inter_key_interval_ms": interval_ms,
            "variance": variance,
            "key_count": 50
        }
    })
}

fn create_window_switch_event(count: u32) -> serde_json::Value {
    serde_json::json!({
        "type": "window_switch",
        "timestamp": chrono::Utc::now(),
        "data": {
            "switch_count": count,
            "duration_ms": 5000
        }
    })
}

fn create_state_event(state: &str, confidence: f32) -> serde_json::Value {
    serde_json::json!({
        "state": state,
        "confidence": confidence,
        "timestamp": chrono::Utc::now(),
        "metrics": {
            "productive_time_ratio": if state == "Flow" { 0.9 } else { 0.3 },
            "distraction_frequency": if state == "Distracted" { 0.8 } else { 0.2 }
        }
    })
}

fn create_intervention_request(intervention_type: &str) -> serde_json::Value {
    serde_json::json!({
        "request_id": uuid::Uuid::new_v4(),
        "intervention_type": intervention_type,
        "urgency": "normal",
        "context": {
            "work_type": "coding",
            "session_duration": 1800000
        }
    })
}

fn create_animation_command(animation_type: &str, duration_ms: u32) -> serde_json::Value {
    serde_json::json!({
        "command_id": uuid::Uuid::new_v4(),
        "animation_type": animation_type,
        "duration_ms": duration_ms,
        "parameters": {
            "intensity": 0.8,
            "loop": false
        }
    })
}