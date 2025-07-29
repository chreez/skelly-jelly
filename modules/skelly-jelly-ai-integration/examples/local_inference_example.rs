//! Local inference example
//!
//! Demonstrates using the AI Integration module with local-only processing.

use ai_integration::{AIIntegration, AIIntegrationImpl, AIIntegrationConfig};
use skelly_jelly_event_bus::message::InterventionRequest;
use std::collections::HashMap;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();

    println!("🦴 Skelly AI Integration - Local Inference Example");
    println!("=================================================");

    // Create privacy-focused configuration (local-only)
    let config = AIIntegrationConfig::privacy_focused();
    println!("✅ Created privacy-focused configuration");

    // Create AI integration instance
    let mut ai = AIIntegrationImpl::new(config);
    println!("✅ Created AI integration instance");

    // Initialize the system
    match ai.initialize().await {
        Ok(()) => println!("✅ AI Integration initialized successfully"),
        Err(e) => {
            println!("⚠️  AI Integration initialization warning: {}", e);
            println!("   This is expected without a local model file.");
            println!("   The system will use template responses as fallback.");
        }
    }

    // Check health status
    let health = ai.health_check().await;
    println!("\n📊 Health Status:");
    println!("   Overall: {:?}", health.overall_status);
    println!("   Local Model: {:?}", health.local_model_status);
    println!("   Memory Usage: {}MB", health.memory_usage_mb);

    // Create sample intervention requests
    let requests = vec![
        create_sample_request("encouragement", "distracted", "Writing a report but keep getting distracted"),
        create_sample_request("suggestion", "neutral", "Working on code but feeling stuck"),
        create_sample_request("celebration", "flow", "Just completed a major feature!"),
        create_sample_request("break_reminder", "hyperfocus", "Been coding for 3 hours straight"),
    ];

    println!("\n💬 Sample Interactions:");
    println!("========================");

    for (i, request) in requests.iter().enumerate() {
        println!("\n{}. Request: {} ({})", 
            i + 1, 
            request.intervention_type, 
            extract_state_from_context(&request.context)
        );

        match ai.process_intervention(request.clone()).await {
            Ok(response) => {
                println!("   🦴 Skelly: {}", response.response_text);
                if !response.animation_cues.is_empty() {
                    println!("   🎭 Animation: {:?}", response.animation_cues);
                }
            }
            Err(e) => {
                println!("   ❌ Error: {}", e);
                println!("   💡 User-friendly: {}", e.user_message());
            }
        }
    }

    // Demonstrate personality customization
    println!("\n🎭 Personality Customization:");
    println!("=============================");

    let cheerful_traits = ai_integration::PersonalityTraits {
        cheerfulness: 0.9,
        humor: 0.8,
        supportiveness: 0.9,
        casualness: 0.8,
        pun_frequency: 0.2, // More puns!
        directness: 0.6,
    };

    ai.update_personality(cheerful_traits).await?;
    println!("✅ Updated personality to be more cheerful and punny");

    // Test with updated personality
    let pun_request = create_sample_request("encouragement", "neutral", "Feeling a bit down about my progress");
    match ai.process_intervention(pun_request).await {
        Ok(response) => {
            println!("   🦴 Cheerful Skelly: {}", response.response_text);
        }
        Err(e) => {
            println!("   ❌ Error: {}", e);
        }
    }

    // Generate animation commands
    println!("\n🎬 Animation Generation:");
    println!("========================");

    let animation_tests = vec![
        ("Amazing work! You're crushing it! 🎉", ai_integration::CompanionMood::Celebrating),
        ("Time for a break - stretch those bones!", ai_integration::CompanionMood::Sleepy),
        ("Let's focus on the task at hand", ai_integration::CompanionMood::Supportive),
    ];

    for (text, mood) in animation_tests {
        match ai.generate_animation(text, mood).await {
            Ok(animation) => {
                println!("   Text: \"{}\"", text);
                println!("   Animation: {} ({}ms)", animation.animation_type, animation.duration_ms);
            }
            Err(e) => {
                println!("   ❌ Animation Error: {}", e);
            }
        }
    }

    // Show usage statistics
    println!("\n📈 Usage Statistics:");
    println!("====================");
    let stats = ai.get_usage_stats().await;
    println!("   Total Requests: {}", stats.requests_processed);
    println!("   Local Generations: {}", stats.local_generations);
    println!("   Template Responses: {}", stats.template_responses);
    println!("   Average Response Time: {:.1}ms", stats.average_response_time_ms);
    println!("   Total Tokens: {}", stats.total_tokens_used);
    println!("   Privacy Violations Blocked: {}", stats.privacy_violations_blocked);

    println!("\n✨ Example completed successfully!");
    println!("   Privacy: All processing stayed local 🔒");
    println!("   Performance: Fast template-based responses ⚡");
    println!("   Personality: Consistent Skelly character 🦴");

    Ok(())
}

fn create_sample_request(intervention_type: &str, state: &str, description: &str) -> InterventionRequest {
    InterventionRequest {
        request_id: Uuid::new_v4(),
        intervention_type: intervention_type.to_string(),
        urgency: "normal".to_string(),
        context: serde_json::json!({
            "state": state,
            "description": description,
            "application": "example_app",
            "window_title": format!("Example - {}", description),
            "productive_time_ratio": 0.7,
            "distraction_frequency": 0.3,
            "focus_session_count": 3,
            "privacy_level": "local_only"
        }),
    }
}

fn extract_state_from_context(context: &serde_json::Value) -> String {
    context.get("state")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string()
}