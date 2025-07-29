//! Demonstration of the performance validation system for Story 2.1
//!
//! This example shows how to use the PerformanceValidator to validate
//! that the ADHD state detection meets the required performance criteria:
//! - Real-time inference <50ms
//! - Model accuracy >80%
//! - Comprehensive validation with detailed reporting

use std::time::Duration;
use tokio::time::sleep;

// For this demo, we'll use mock types since the full system has dependency issues
#[derive(Debug, Clone)]
pub struct MockStateDetectionEngine {
    pub inference_time_ms: f32,
    pub accuracy_rate: f32,
}

#[derive(Debug, Clone)]
pub struct MockAnalysisWindow {
    pub window_id: String,
    pub event_count: usize,
}

#[derive(Debug, Clone)]
pub struct MockADHDState {
    pub state_type: String,
    pub confidence: f32,
}

#[derive(Debug, Clone)]
pub struct MockStateDetectionResult {
    pub detected_state: MockADHDState,
    pub confidence: f32,
    pub intervention_readiness: bool,
    pub feature_importance: std::collections::HashMap<String, f32>,
}

impl MockStateDetectionEngine {
    pub fn new(inference_time_ms: f32, accuracy_rate: f32) -> Self {
        Self {
            inference_time_ms,
            accuracy_rate,
        }
    }

    pub async fn detect_state(&self, _window: &MockAnalysisWindow) -> Result<MockStateDetectionResult, String> {
        // Simulate inference time
        let sleep_time = Duration::from_millis(self.inference_time_ms as u64);
        sleep(sleep_time).await;

        // Generate mock result
        Ok(MockStateDetectionResult {
            detected_state: MockADHDState {
                state_type: "Flow".to_string(),
                confidence: self.accuracy_rate,
            },
            confidence: self.accuracy_rate,
            intervention_readiness: false,
            feature_importance: std::collections::HashMap::new(),
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Performance Validation Demo for Story 2.1 ===");
    println!();

    // Demo 1: Fast, accurate system (should pass all requirements)
    println!("Demo 1: High-performance system");
    println!("Expected: PASS (25ms inference, 90% accuracy)");
    let fast_engine = MockStateDetectionEngine::new(25.0, 0.90);
    run_validation_demo("High Performance", &fast_engine, 100).await?;
    println!();

    // Demo 2: Slow but accurate system (should fail latency requirement)
    println!("Demo 2: Slow but accurate system");
    println!("Expected: FAIL (75ms inference exceeds 50ms limit)");
    let slow_engine = MockStateDetectionEngine::new(75.0, 0.85);
    run_validation_demo("Slow System", &slow_engine, 50).await?;
    println!();

    // Demo 3: Fast but inaccurate system (should fail accuracy requirement)
    println!("Demo 3: Fast but inaccurate system");
    println!("Expected: FAIL (70% accuracy below 80% requirement)");
    let inaccurate_engine = MockStateDetectionEngine::new(30.0, 0.70);
    run_validation_demo("Inaccurate System", &inaccurate_engine, 100).await?;
    println!();

    // Demo 4: Edge case - exactly at limits
    println!("Demo 4: System at performance boundaries");
    println!("Expected: PASS (exactly 50ms inference, 80% accuracy)");
    let edge_engine = MockStateDetectionEngine::new(50.0, 0.80);
    run_validation_demo("Edge Case", &edge_engine, 75).await?;

    println!();
    println!("=== Performance Validation Demo Complete ===");
    println!();
    println!("Key Insights:");
    println!("1. Story 2.1 requires <50ms inference AND >80% accuracy");
    println!("2. Both requirements must be met for the system to pass validation");
    println!("3. The validation system provides detailed metrics and recommendations");
    println!("4. Real-world testing would use actual behavioral data and ML models");

    Ok(())
}

async fn run_validation_demo(
    name: &str,
    engine: &MockStateDetectionEngine,
    test_samples: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("--- {} Validation ---", name);
    
    let start_time = std::time::Instant::now();
    
    // Generate test data
    let mut test_windows = Vec::new();
    for i in 0..test_samples {
        test_windows.push(MockAnalysisWindow {
            window_id: format!("window_{}", i),
            event_count: 50 + (i % 30), // Realistic event counts
        });
    }

    // Run latency tests
    let mut latencies = Vec::new();
    let mut correct_predictions = 0;
    
    println!("Running validation with {} samples...", test_samples);
    
    for (i, window) in test_windows.iter().enumerate() {
        if i % 20 == 0 && i > 0 {
            println!("  Processed {}/{} samples", i, test_samples);
        }
        
        let inference_start = std::time::Instant::now();
        
        match engine.detect_state(window).await {
            Ok(result) => {
                let latency_ms = inference_start.elapsed().as_millis() as f32;
                latencies.push(latency_ms);
                
                // Mock accuracy check (in real system, this would compare against ground truth)
                if rand::random::<f32>() < engine.accuracy_rate {
                    correct_predictions += 1;
                }
            }
            Err(e) => {
                eprintln!("Inference error: {}", e);
                continue;
            }
        }
    }

    // Calculate results
    let total_samples = latencies.len();
    let avg_latency_ms = latencies.iter().sum::<f32>() / total_samples as f32;
    let accuracy = correct_predictions as f32 / total_samples as f32;
    
    latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let p95_latency_ms = latencies[latencies.len() * 95 / 100];
    let max_latency_ms = latencies[latencies.len() - 1];
    
    let samples_under_50ms = latencies.iter().filter(|&&l| l <= 50.0).count();
    let samples_over_50ms = total_samples - samples_under_50ms;
    
    // Determine if requirements are met
    let latency_requirement_met = samples_over_50ms == 0 && avg_latency_ms <= 50.0;
    let accuracy_requirement_met = accuracy >= 0.8;
    let overall_status = if latency_requirement_met && accuracy_requirement_met {
        "PASSED"
    } else {
        "FAILED"
    };

    // Print results
    println!();
    println!("VALIDATION RESULTS:");
    println!("  Status: {}", overall_status);
    println!();
    println!("  LATENCY METRICS:");
    println!("    Average: {:.1}ms (requirement: <50ms)", avg_latency_ms);
    println!("    P95: {:.1}ms", p95_latency_ms);
    println!("    Maximum: {:.1}ms", max_latency_ms);
    println!("    Samples under 50ms: {}/{} ({:.1}%)", 
             samples_under_50ms, total_samples,
             samples_under_50ms as f32 / total_samples as f32 * 100.0);
    println!("    Latency requirement met: {}", latency_requirement_met);
    println!();
    println!("  ACCURACY METRICS:");
    println!("    Overall accuracy: {:.1}% (requirement: >80%)", accuracy * 100.0);
    println!("    Correct predictions: {}/{}", correct_predictions, total_samples);
    println!("    Accuracy requirement met: {}", accuracy_requirement_met);
    println!();

    // Generate recommendations
    if !latency_requirement_met {
        println!("  RECOMMENDATIONS:");
        println!("    - Optimize model inference pipeline");
        println!("    - Consider model quantization or pruning");
        println!("    - Implement hardware acceleration (GPU/specialized chips)");
        println!("    - Review feature extraction efficiency");
    }
    
    if !accuracy_requirement_met {
        println!("  RECOMMENDATIONS:");
        println!("    - Collect more diverse training data");
        println!("    - Tune hyperparameters (Random Forest n_estimators, max_depth)");
        println!("    - Consider ensemble methods or advanced models");
        println!("    - Implement better feature engineering");
    }

    let validation_time = start_time.elapsed();
    println!("  Validation completed in {:.2}s", validation_time.as_secs_f32());

    Ok(())
}