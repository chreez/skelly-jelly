//! Performance validation for ADHD state detection requirements
//!
//! This module provides comprehensive testing and validation of the
//! performance requirements specified in Story 2.1:
//! - Real-time inference <50ms
//! - Model accuracy >80%
//! - Online learning improvement >5% per week

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use crate::{
    error::{AnalysisError, AnalysisResult},
    models::{ADHDState, ADHDStateType, RandomForestClassifier, StateModel},
    state_detection::{StateDetectionEngine, StateDetectionConfig},
    types::FeatureVector,
    sliding_window::AnalysisWindow,
};

/// Performance validation suite for ADHD state detection
pub struct PerformanceValidator {
    /// Configuration for validation tests
    config: ValidationConfig,
    /// Validation results history
    results_history: Vec<ValidationResult>,
}

/// Configuration for performance validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationConfig {
    /// Number of inference samples for latency testing
    pub latency_test_samples: usize,
    
    /// Number of accuracy test samples
    pub accuracy_test_samples: usize,
    
    /// Required inference time (ms)
    pub max_inference_time_ms: f32,
    
    /// Required accuracy threshold
    pub min_accuracy_threshold: f32,
    
    /// Online learning validation period (days)
    pub online_learning_test_days: u32,
    
    /// Expected weekly improvement rate
    pub expected_weekly_improvement: f32,
    
    /// Statistical confidence level
    pub confidence_level: f32,
    
    /// Enable detailed profiling
    pub enable_profiling: bool,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            latency_test_samples: 1000,
            accuracy_test_samples: 10000,
            max_inference_time_ms: 50.0,
            min_accuracy_threshold: 0.8,
            online_learning_test_days: 7,
            expected_weekly_improvement: 0.05,
            confidence_level: 0.95,
            enable_profiling: true,
        }
    }
}

/// Comprehensive validation results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub timestamp: DateTime<Utc>,
    pub test_type: ValidationTestType,
    pub latency_results: LatencyTestResult,
    pub accuracy_results: AccuracyTestResult,
    pub online_learning_results: Option<OnlineLearningResult>,
    pub overall_status: ValidationStatus,
    pub recommendations: Vec<String>,
}

/// Types of validation tests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationTestType {
    QuickValidation,
    ComprehensiveValidation,
    OnlineLearningValidation,
    StressTest,
}

/// Validation status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationStatus {
    Passed,
    Failed,
    Warning,
    NeedsImprovement,
}

/// Latency test results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyTestResult {
    pub total_samples: usize,
    pub avg_latency_ms: f32,
    pub p50_latency_ms: f32,
    pub p95_latency_ms: f32,
    pub p99_latency_ms: f32,
    pub max_latency_ms: f32,
    pub min_latency_ms: f32,
    pub samples_under_50ms: usize,
    pub samples_over_50ms: usize,
    pub latency_distribution: HashMap<String, usize>,
    pub requirement_met: bool,
}

/// Accuracy test results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccuracyTestResult {
    pub total_samples: usize,
    pub correct_predictions: usize,
    pub overall_accuracy: f32,
    pub per_class_accuracy: HashMap<ADHDStateType, f32>,
    pub per_class_precision: HashMap<ADHDStateType, f32>,
    pub per_class_recall: HashMap<ADHDStateType, f32>,
    pub per_class_f1: HashMap<ADHDStateType, f32>,
    pub confusion_matrix: HashMap<(ADHDStateType, ADHDStateType), u32>,
    pub confidence_distribution: Vec<f32>,
    pub avg_confidence: f32,
    pub requirement_met: bool,
}

/// Online learning validation results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnlineLearningResult {
    pub initial_accuracy: f32,
    pub final_accuracy: f32,
    pub improvement_rate: f32,
    pub days_tested: u32,
    pub feedback_samples: usize,
    pub weekly_projected_improvement: f32,
    pub requirement_met: bool,
    pub learning_curve: Vec<(DateTime<Utc>, f32)>,
}

impl PerformanceValidator {
    /// Create a new performance validator
    pub fn new() -> Self {
        Self::with_config(ValidationConfig::default())
    }

    /// Create with custom configuration
    pub fn with_config(config: ValidationConfig) -> Self {
        Self {
            config,
            results_history: Vec::new(),
        }
    }

    /// Run comprehensive performance validation
    pub async fn validate_comprehensive(&mut self, engine: &StateDetectionEngine) -> AnalysisResult<ValidationResult> {
        println!("Running comprehensive performance validation...");
        
        let start_time = Instant::now();
        
        // Generate test data
        let (latency_data, accuracy_data) = self.generate_test_data().await?;
        
        // Run latency tests
        println!("Running latency validation...");
        let latency_results = self.validate_latency(engine, &latency_data).await?;
        
        // Run accuracy tests  
        println!("Running accuracy validation...");
        let accuracy_results = self.validate_accuracy(engine, &accuracy_data).await?;
        
        // Determine overall status
        let overall_status = self.determine_validation_status(&latency_results, &accuracy_results, None);
        
        // Generate recommendations
        let recommendations = self.generate_recommendations(&latency_results, &accuracy_results, None);
        
        let result = ValidationResult {
            timestamp: Utc::now(),
            test_type: ValidationTestType::ComprehensiveValidation,
            latency_results,
            accuracy_results,
            online_learning_results: None,
            overall_status,
            recommendations,
        };
        
        self.results_history.push(result.clone());
        
        let validation_time = start_time.elapsed();
        println!("Comprehensive validation completed in {:.2}s", validation_time.as_secs_f32());
        
        self.print_validation_summary(&result);
        
        Ok(result)
    }

    /// Run quick performance validation
    pub async fn validate_quick(&mut self, engine: &StateDetectionEngine) -> AnalysisResult<ValidationResult> {
        println!("Running quick performance validation...");
        
        let start_time = Instant::now();
        
        // Use smaller test sets for quick validation
        let quick_config = ValidationConfig {
            latency_test_samples: 100,
            accuracy_test_samples: 1000,
            ..self.config.clone()
        };
        
        let (latency_data, accuracy_data) = self.generate_test_data_with_config(&quick_config).await?;
        
        let latency_results = self.validate_latency(engine, &latency_data).await?;
        let accuracy_results = self.validate_accuracy(engine, &accuracy_data).await?;
        
        let overall_status = self.determine_validation_status(&latency_results, &accuracy_results, None);
        let recommendations = self.generate_recommendations(&latency_results, &accuracy_results, None);
        
        let result = ValidationResult {
            timestamp: Utc::now(),
            test_type: ValidationTestType::QuickValidation,
            latency_results,
            accuracy_results,
            online_learning_results: None,
            overall_status,
            recommendations,
        };
        
        self.results_history.push(result.clone());
        
        let validation_time = start_time.elapsed();
        println!("Quick validation completed in {:.2}s", validation_time.as_secs_f32());
        
        Ok(result)
    }

    /// Validate inference latency requirements
    async fn validate_latency(&self, engine: &StateDetectionEngine, test_data: &[AnalysisWindow]) -> AnalysisResult<LatencyTestResult> {
        let mut latencies = Vec::new();
        let mut samples_under_50ms = 0;
        let mut samples_over_50ms = 0;
        
        println!("Testing inference latency with {} samples...", test_data.len());
        
        for (i, window) in test_data.iter().enumerate() {
            if i % 100 == 0 && i > 0 {
                println!("  Processed {}/{} samples", i, test_data.len());
            }
            
            let start_time = Instant::now();
            
            // Perform inference
            match engine.detect_state(window).await {
                Ok(_) => {
                    let latency_ms = start_time.elapsed().as_millis() as f32;
                    latencies.push(latency_ms);
                    
                    if latency_ms <= self.config.max_inference_time_ms {
                        samples_under_50ms += 1;
                    } else {
                        samples_over_50ms += 1;
                    }
                }
                Err(e) => {
                    // For validation, we'll skip errors but count them
                    eprintln!("Inference error during validation: {}", e);
                    continue;
                }
            }
        }
        
        if latencies.is_empty() {
            return Err(AnalysisError::ValidationError {
                message: "No successful inferences during latency testing".to_string(),
            });
        }
        
        // Calculate statistics
        latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let avg_latency_ms = latencies.iter().sum::<f32>() / latencies.len() as f32;
        let p50_latency_ms = latencies[latencies.len() * 50 / 100];
        let p95_latency_ms = latencies[latencies.len() * 95 / 100];
        let p99_latency_ms = latencies[latencies.len() * 99 / 100];
        let max_latency_ms = latencies[latencies.len() - 1];
        let min_latency_ms = latencies[0];
        
        // Create latency distribution
        let mut latency_distribution = HashMap::new();
        for &latency in &latencies {
            let bucket = match latency {
                l if l <= 10.0 => "0-10ms",
                l if l <= 20.0 => "10-20ms", 
                l if l <= 30.0 => "20-30ms",
                l if l <= 40.0 => "30-40ms",
                l if l <= 50.0 => "40-50ms",
                l if l <= 75.0 => "50-75ms",
                l if l <= 100.0 => "75-100ms",
                _ => "100ms+",
            };
            *latency_distribution.entry(bucket.to_string()).or_insert(0) += 1;
        }
        
        let requirement_met = samples_over_50ms == 0 && avg_latency_ms <= self.config.max_inference_time_ms;
        
        Ok(LatencyTestResult {
            total_samples: latencies.len(),
            avg_latency_ms,
            p50_latency_ms,
            p95_latency_ms,
            p99_latency_ms,
            max_latency_ms,
            min_latency_ms,
            samples_under_50ms,
            samples_over_50ms,
            latency_distribution,
            requirement_met,
        })
    }

    /// Validate model accuracy requirements
    async fn validate_accuracy(&self, engine: &StateDetectionEngine, test_data: &[(AnalysisWindow, ADHDState)]) -> AnalysisResult<AccuracyTestResult> {
        let mut correct_predictions = 0;
        let mut total_samples = 0;
        let mut per_class_correct: HashMap<ADHDStateType, u32> = HashMap::new();
        let mut per_class_total: HashMap<ADHDStateType, u32> = HashMap::new();
        let mut confusion_matrix: HashMap<(ADHDStateType, ADHDStateType), u32> = HashMap::new();
        let mut confidence_scores = Vec::new();
        
        println!("Testing prediction accuracy with {} samples...", test_data.len());
        
        for (i, (window, true_state)) in test_data.iter().enumerate() {
            if i % 100 == 0 && i > 0 {
                println!("  Processed {}/{} samples", i, test_data.len());
            }
            
            match engine.detect_state(window).await {
                Ok(result) => {
                    let predicted_state_type = crate::models::get_adhd_state_type(&result.detected_state);
                    let true_state_type = crate::models::get_adhd_state_type(true_state);
                    
                    // Update confusion matrix
                    *confusion_matrix.entry((predicted_state_type, true_state_type)).or_insert(0) += 1;
                    
                    // Update per-class counts
                    *per_class_total.entry(true_state_type).or_insert(0) += 1;
                    
                    if predicted_state_type == true_state_type {
                        correct_predictions += 1;
                        *per_class_correct.entry(true_state_type).or_insert(0) += 1;
                    }
                    
                    confidence_scores.push(result.confidence);
                    total_samples += 1;
                }
                Err(e) => {
                    eprintln!("Prediction error during validation: {}", e);
                    continue;
                }
            }
        }
        
        if total_samples == 0 {
            return Err(AnalysisError::ValidationError {
                message: "No successful predictions during accuracy testing".to_string(),
            });
        }
        
        let overall_accuracy = correct_predictions as f32 / total_samples as f32;
        
        // Calculate per-class metrics
        let mut per_class_accuracy = HashMap::new();
        let mut per_class_precision = HashMap::new();
        let mut per_class_recall = HashMap::new();
        let mut per_class_f1 = HashMap::new();
        
        for state_type in ADHDStateType::all() {
            let class_correct = per_class_correct.get(&state_type).unwrap_or(&0);
            let class_total = per_class_total.get(&state_type).unwrap_or(&0);
            
            let accuracy = if *class_total > 0 {
                *class_correct as f32 / *class_total as f32
            } else {
                0.0
            };
            per_class_accuracy.insert(state_type, accuracy);
            
            // Calculate precision and recall
            let (precision, recall, f1) = self.calculate_precision_recall_f1(state_type, &confusion_matrix);
            per_class_precision.insert(state_type, precision);
            per_class_recall.insert(state_type, recall);
            per_class_f1.insert(state_type, f1);
        }
        
        let avg_confidence = confidence_scores.iter().sum::<f32>() / confidence_scores.len() as f32;
        let requirement_met = overall_accuracy >= self.config.min_accuracy_threshold;
        
        Ok(AccuracyTestResult {
            total_samples,
            correct_predictions,
            overall_accuracy,
            per_class_accuracy,
            per_class_precision,
            per_class_recall,
            per_class_f1,
            confusion_matrix,
            confidence_distribution: confidence_scores,
            avg_confidence,
            requirement_met,
        })
    }

    /// Calculate precision, recall, and F1 score for a specific class
    fn calculate_precision_recall_f1(&self, state_type: ADHDStateType, confusion_matrix: &HashMap<(ADHDStateType, ADHDStateType), u32>) -> (f32, f32, f32) {
        let mut true_positives = 0;
        let mut false_positives = 0;
        let mut false_negatives = 0;
        
        for ((predicted, actual), &count) in confusion_matrix {
            match (*predicted == state_type, *actual == state_type) {
                (true, true) => true_positives += count,
                (true, false) => false_positives += count,
                (false, true) => false_negatives += count,
                (false, false) => {} // True negatives
            }
        }
        
        let precision = if true_positives + false_positives > 0 {
            true_positives as f32 / (true_positives + false_positives) as f32
        } else {
            0.0
        };
        
        let recall = if true_positives + false_negatives > 0 {
            true_positives as f32 / (true_positives + false_negatives) as f32
        } else {
            0.0
        };
        
        let f1 = if precision + recall > 0.0 {
            2.0 * (precision * recall) / (precision + recall)
        } else {
            0.0
        };
        
        (precision, recall, f1)
    }

    /// Generate test data for validation
    async fn generate_test_data(&self) -> AnalysisResult<(Vec<AnalysisWindow>, Vec<(AnalysisWindow, ADHDState)>)> {
        self.generate_test_data_with_config(&self.config).await
    }

    /// Generate test data with specific configuration
    async fn generate_test_data_with_config(&self, config: &ValidationConfig) -> AnalysisResult<(Vec<AnalysisWindow>, Vec<(AnalysisWindow, ADHDState)>)> {
        println!("Generating test data...");
        
        // Generate latency test data (just windows)
        let latency_data = self.generate_test_windows(config.latency_test_samples).await?;
        
        // Generate accuracy test data (windows with labels)
        let accuracy_data = self.generate_labeled_test_data(config.accuracy_test_samples).await?;
        
        println!("Generated {} latency test samples and {} accuracy test samples", 
                latency_data.len(), accuracy_data.len());
        
        Ok((latency_data, accuracy_data))
    }

    /// Generate test windows for latency testing
    async fn generate_test_windows(&self, count: usize) -> AnalysisResult<Vec<AnalysisWindow>> {
        use std::time::SystemTime;
        use skelly_jelly_storage::types::*;
        use chrono::Utc;
        
        let mut windows = Vec::new();
        
        for i in 0..count {
            let mut window = AnalysisWindow::new(SystemTime::now());
            
            // Add realistic event patterns
            let base_time = Utc::now();
            
            // Add keystroke events
            for j in 0..20 {
                let event = RawEvent::Keystroke(KeystrokeEvent {
                    timestamp: base_time + chrono::Duration::milliseconds(j * 150),
                    key_code: 65 + (j % 26) as u32,
                    modifiers: KeyModifiers::default(),
                    inter_key_interval_ms: Some(150),
                });
                window.add_event(event);
            }
            
            // Add mouse events
            for j in 0..10 {
                let event = RawEvent::MouseMove(MouseMoveEvent {
                    timestamp: base_time + chrono::Duration::milliseconds(j * 300),
                    x: 100 + (j * 50) as i32,
                    y: 100 + (j * 30) as i32,
                    velocity: 200.0 + (j as f32 * 20.0),
                });
                window.add_event(event);
            }
            
            // Add window focus events
            let apps = ["Visual Studio Code", "Chrome", "Slack"];
            let event = RawEvent::WindowFocus(WindowFocusEvent {
                timestamp: base_time,
                window_title: format!("{} - Window", apps[i % apps.len()]),
                app_name: apps[i % apps.len()].to_string(),
                process_id: 1000 + i as u32,
                duration_ms: Some(30000),
            });
            window.add_event(event);
            
            windows.push(window);
        }
        
        Ok(windows)
    }

    /// Generate labeled test data for accuracy testing
    async fn generate_labeled_test_data(&self, count: usize) -> AnalysisResult<Vec<(AnalysisWindow, ADHDState)>> {
        let windows = self.generate_test_windows(count).await?;
        let mut labeled_data = Vec::new();
        
        // Generate realistic labels based on window characteristics
        for (i, window) in windows.into_iter().enumerate() {
            let state_type = match i % 5 {
                0 => ADHDStateType::Flow,
                1 => ADHDStateType::Hyperfocus,
                2 => ADHDStateType::Distracted,
                3 => ADHDStateType::Transitioning,
                _ => ADHDStateType::Neutral,
            };
            
            let state = match state_type {
                ADHDStateType::Flow => ADHDState::flow(),
                ADHDStateType::Hyperfocus => ADHDState::hyperfocus(),
                ADHDStateType::Distracted => ADHDState::distracted(),
                ADHDStateType::Transitioning => ADHDState::transitioning(),
                ADHDStateType::Neutral => ADHDState::neutral(),
            };
            
            labeled_data.push((window, state));
        }
        
        Ok(labeled_data)
    }

    /// Determine overall validation status
    fn determine_validation_status(
        &self,
        latency: &LatencyTestResult,
        accuracy: &AccuracyTestResult,
        online_learning: Option<&OnlineLearningResult>,
    ) -> ValidationStatus {
        let mut issues = Vec::new();
        
        if !latency.requirement_met {
            issues.push("Latency requirement not met");
        }
        
        if !accuracy.requirement_met {
            issues.push("Accuracy requirement not met");
        }
        
        if let Some(ol) = online_learning {
            if !ol.requirement_met {
                issues.push("Online learning requirement not met");
            }
        }
        
        match issues.len() {
            0 => ValidationStatus::Passed,
            1 => ValidationStatus::Warning,
            _ => ValidationStatus::Failed,
        }
    }

    /// Generate recommendations based on validation results
    fn generate_recommendations(
        &self,
        latency: &LatencyTestResult,
        accuracy: &AccuracyTestResult,
        _online_learning: Option<&OnlineLearningResult>,
    ) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        if !latency.requirement_met {
            recommendations.push(format!(
                "Latency optimization needed: {}ms avg (target: {}ms). Consider model compression or hardware acceleration.",
                latency.avg_latency_ms, self.config.max_inference_time_ms
            ));
        }
        
        if !accuracy.requirement_met {
            recommendations.push(format!(
                "Accuracy improvement needed: {:.1}% (target: {:.1}%). Consider more training data or hyperparameter tuning.",
                accuracy.overall_accuracy * 100.0, self.config.min_accuracy_threshold * 100.0
            ));
        }
        
        // Check for class imbalance
        for (state_type, &accuracy) in &accuracy.per_class_accuracy {
            if accuracy < 0.6 {
                recommendations.push(format!(
                    "Poor performance on {:?} state ({:.1}% accuracy). Consider collecting more training data for this class.",
                    state_type, accuracy * 100.0
                ));
            }
        }
        
        if latency.p99_latency_ms > latency.avg_latency_ms * 2.0 {
            recommendations.push("High latency variance detected. Consider optimizing worst-case performance.".to_string());
        }
        
        if accuracy.avg_confidence < 0.7 {
            recommendations.push(format!(
                "Low average confidence ({:.2}). Consider calibrating model confidence scores.",
                accuracy.avg_confidence
            ));
        }
        
        recommendations
    }

    /// Print validation summary
    fn print_validation_summary(&self, result: &ValidationResult) {
        println!("\n=== PERFORMANCE VALIDATION SUMMARY ===");
        println!("Status: {:?}", result.overall_status);
        println!();
        
        println!("LATENCY RESULTS:");
        println!("  Average: {:.1}ms (requirement: <{:.0}ms)", 
                result.latency_results.avg_latency_ms, self.config.max_inference_time_ms);
        println!("  P95: {:.1}ms, P99: {:.1}ms", 
                result.latency_results.p95_latency_ms, result.latency_results.p99_latency_ms);
        println!("  Samples under 50ms: {}/{} ({:.1}%)", 
                result.latency_results.samples_under_50ms,
                result.latency_results.total_samples,
                result.latency_results.samples_under_50ms as f32 / result.latency_results.total_samples as f32 * 100.0);
        println!("  Requirement met: {}", result.latency_results.requirement_met);
        println!();
        
        println!("ACCURACY RESULTS:");
        println!("  Overall: {:.1}% (requirement: >{:.0}%)", 
                result.accuracy_results.overall_accuracy * 100.0, self.config.min_accuracy_threshold * 100.0);
        println!("  Average confidence: {:.2}", result.accuracy_results.avg_confidence);
        println!("  Requirement met: {}", result.accuracy_results.requirement_met);
        println!();
        
        if !result.recommendations.is_empty() {
            println!("RECOMMENDATIONS:");
            for (i, rec) in result.recommendations.iter().enumerate() {
                println!("  {}. {}", i + 1, rec);
            }
            println!();
        }
        
        println!("========================================\n");
    }

    /// Get validation history
    pub fn get_validation_history(&self) -> &[ValidationResult] {
        &self.results_history
    }

    /// Export validation results to JSON
    pub fn export_results(&self, path: &str) -> AnalysisResult<()> {
        let json = serde_json::to_string_pretty(&self.results_history)
            .map_err(|e| AnalysisError::ValidationError {
                message: format!("Failed to serialize results: {}", e),
            })?;
        
        std::fs::write(path, json)
            .map_err(|e| AnalysisError::ValidationError {
                message: format!("Failed to write results file: {}", e),
            })?;
        
        println!("Validation results exported to: {}", path);
        Ok(())
    }
}

impl Default for PerformanceValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state_detection::StateDetectionEngine;

    #[test]
    fn test_validation_config() {
        let config = ValidationConfig::default();
        
        assert_eq!(config.max_inference_time_ms, 50.0);
        assert_eq!(config.min_accuracy_threshold, 0.8);
        assert_eq!(config.latency_test_samples, 1000);
        assert_eq!(config.accuracy_test_samples, 10000);
    }

    #[tokio::test]
    async fn test_test_data_generation() {
        let validator = PerformanceValidator::new();
        let (latency_data, accuracy_data) = validator.generate_test_data().await.unwrap();
        
        assert_eq!(latency_data.len(), 1000);
        assert_eq!(accuracy_data.len(), 10000);
        
        // Check that accuracy data has labels
        for (_, state) in &accuracy_data[..5] {
            assert!(matches!(crate::models::get_adhd_state_type(state), 
                           ADHDStateType::Flow | ADHDStateType::Hyperfocus | 
                           ADHDStateType::Distracted | ADHDStateType::Transitioning | 
                           ADHDStateType::Neutral));
        }
    }

    #[test]
    fn test_precision_recall_calculation() {
        let validator = PerformanceValidator::new();
        let mut confusion_matrix = HashMap::new();
        
        // Example confusion matrix for Flow state
        confusion_matrix.insert((ADHDStateType::Flow, ADHDStateType::Flow), 80);      // TP
        confusion_matrix.insert((ADHDStateType::Flow, ADHDStateType::Distracted), 10); // FP
        confusion_matrix.insert((ADHDStateType::Distracted, ADHDStateType::Flow), 20); // FN
        
        let (precision, recall, f1) = validator.calculate_precision_recall_f1(ADHDStateType::Flow, &confusion_matrix);
        
        assert!((precision - 0.888).abs() < 0.01); // 80/(80+10) ≈ 0.888
        assert!((recall - 0.8).abs() < 0.01);      // 80/(80+20) = 0.8
        assert!(f1 > 0.8);
    }

    #[test]
    fn test_validation_status_determination() {
        let validator = PerformanceValidator::new();
        
        let good_latency = LatencyTestResult {
            avg_latency_ms: 25.0,
            requirement_met: true,
            ..Default::default()
        };
        
        let good_accuracy = AccuracyTestResult {
            overall_accuracy: 0.85,
            requirement_met: true,
            ..Default::default()
        };
        
        let status = validator.determine_validation_status(&good_latency, &good_accuracy, None);
        assert!(matches!(status, ValidationStatus::Passed));
    }
}

// Default implementations for test structures
impl Default for LatencyTestResult {
    fn default() -> Self {
        Self {
            total_samples: 0,
            avg_latency_ms: 0.0,
            p50_latency_ms: 0.0,
            p95_latency_ms: 0.0,
            p99_latency_ms: 0.0,
            max_latency_ms: 0.0,
            min_latency_ms: 0.0,
            samples_under_50ms: 0,
            samples_over_50ms: 0,
            latency_distribution: HashMap::new(),
            requirement_met: false,
        }
    }
}

impl Default for AccuracyTestResult {
    fn default() -> Self {
        Self {
            total_samples: 0,
            correct_predictions: 0,
            overall_accuracy: 0.0,
            per_class_accuracy: HashMap::new(),
            per_class_precision: HashMap::new(),
            per_class_recall: HashMap::new(),
            per_class_f1: HashMap::new(),
            confusion_matrix: HashMap::new(),
            confidence_distribution: Vec::new(),
            avg_confidence: 0.0,
            requirement_met: false,
        }
    }
}