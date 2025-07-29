//! Training pipeline for ADHD state detection models
//!
//! Comprehensive training system with hyperparameter optimization,
//! cross-validation, and model export capabilities for production deployment.

use ndarray::{Array1, Array2};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    path::Path,
    time::Instant,
};

use crate::{
    error::{AnalysisError, AnalysisResult},
    models::{
        ADHDState, ADHDStateType, ONNXClassifier, RandomForestClassifier, StateModel,
        ModelMetadata, ONNXConfig, RandomForestConfig,
    },
    types::FeatureVector,
};

/// Complete training pipeline for ADHD state detection
pub struct TrainingPipeline {
    /// Configuration for training
    config: TrainingConfig,
    /// Training dataset
    training_data: Vec<(FeatureVector, ADHDState)>,
    /// Validation dataset
    validation_data: Vec<(FeatureVector, ADHDState)>,
    /// Test dataset
    test_data: Vec<(FeatureVector, ADHDState)>,
    /// Best model found during training
    best_model: Option<Box<dyn StateModel>>,
    /// Training metrics history
    training_history: Vec<TrainingEpoch>,
}

/// Training epoch results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingEpoch {
    pub epoch: usize,
    pub training_accuracy: f32,
    pub validation_accuracy: f32,
    pub training_loss: f32,
    pub validation_loss: f32,
    pub inference_time_ms: f32,
    pub hyperparameters: HashMap<String, f32>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Hyperparameter optimization results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HyperparameterResults {
    pub best_params: HashMap<String, f32>,
    pub best_accuracy: f32,
    pub best_model_type: String,
    pub optimization_iterations: usize,
    pub total_training_time_secs: f32,
    pub cross_validation_scores: Vec<f32>,
}

impl TrainingPipeline {
    /// Create a new training pipeline
    pub fn new(config: TrainingConfig) -> Self {
        Self {
            config,
            training_data: Vec::new(),
            validation_data: Vec::new(),
            test_data: Vec::new(),
            best_model: None,
            training_history: Vec::new(),
        }
    }

    /// Load training data from various sources
    pub fn load_data(&mut self, data_path: &str) -> AnalysisResult<()> {
        println!("Loading training data from: {}", data_path);

        // Load data based on file extension
        let data = if data_path.ends_with(".json") {
            self.load_json_data(data_path)?
        } else if data_path.ends_with(".csv") {
            self.load_csv_data(data_path)?
        } else {
            return Err(AnalysisError::InvalidInput {
                message: format!("Unsupported data format: {}", data_path),
            });
        };

        println!("Loaded {} samples", data.len());

        // Split data into training/validation/test sets
        self.split_data(data)?;

        println!("Data split - Training: {}, Validation: {}, Test: {}", 
                self.training_data.len(), 
                self.validation_data.len(), 
                self.test_data.len());

        Ok(())
    }

    /// Load data from JSON format
    fn load_json_data(&self, path: &str) -> AnalysisResult<Vec<(FeatureVector, ADHDState)>> {
        let content = fs::read_to_string(path)
            .map_err(|e| AnalysisError::DataLoadError {
                path: path.to_string(),
                message: format!("Failed to read JSON file: {}", e),
            })?;

        let data: Vec<TrainingExample> = serde_json::from_str(&content)
            .map_err(|e| AnalysisError::DataLoadError {
                path: path.to_string(),
                message: format!("Failed to parse JSON: {}", e),
            })?;

        let mut samples = Vec::new();
        for example in data {
            let feature_vector = FeatureVector {
                keystroke_features: example.features.keystroke_features,
                mouse_features: example.features.mouse_features,
                window_features: example.features.window_features,
                temporal_features: example.features.temporal_features,
                resource_features: example.features.resource_features,
                screenshot_features: example.features.screenshot_features,
            };

            let state = match example.label.as_str() {
                "flow" => ADHDState::flow(),
                "hyperfocus" => ADHDState::hyperfocus(),
                "distracted" => ADHDState::distracted(),
                "transitioning" => ADHDState::transitioning(),
                "neutral" => ADHDState::neutral(),
                _ => return Err(AnalysisError::DataLoadError {
                    path: path.to_string(),
                    message: format!("Unknown state label: {}", example.label),
                }),
            };

            samples.push((feature_vector, state));
        }

        Ok(samples)
    }

    /// Load data from CSV format
    fn load_csv_data(&self, path: &str) -> AnalysisResult<Vec<(FeatureVector, ADHDState)>> {
        // Simplified CSV loading - in practice, you'd use a CSV library
        let content = fs::read_to_string(path)
            .map_err(|e| AnalysisError::DataLoadError {
                path: path.to_string(),
                message: format!("Failed to read CSV file: {}", e),
            })?;

        let mut samples = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        
        if lines.is_empty() {
            return Ok(samples);
        }

        // Skip header line
        for (line_num, line) in lines.iter().skip(1).enumerate() {
            let parts: Vec<&str> = line.split(',').collect();
            
            if parts.len() < 46 { // 45 features + 1 label
                return Err(AnalysisError::DataLoadError {
                    path: path.to_string(),
                    message: format!("Invalid CSV format at line {}: expected 46 columns, got {}", 
                                   line_num + 2, parts.len()),
                });
            }

            // Parse features (first 45 columns)
            let mut feature_vec = vec![0.0f32; 45];
            for (i, part) in parts.iter().take(45).enumerate() {
                feature_vec[i] = part.parse::<f32>()
                    .map_err(|e| AnalysisError::DataLoadError {
                        path: path.to_string(),
                        message: format!("Failed to parse feature {} at line {}: {}", i, line_num + 2, e),
                    })?;
            }

            // Create feature vector
            let feature_vector = FeatureVector {
                keystroke_features: [
                    feature_vec[0], feature_vec[1], feature_vec[2], feature_vec[3], feature_vec[4],
                    feature_vec[5], feature_vec[6], feature_vec[7], feature_vec[8], feature_vec[9],
                ],
                mouse_features: [
                    feature_vec[10], feature_vec[11], feature_vec[12], feature_vec[13],
                    feature_vec[14], feature_vec[15], feature_vec[16], feature_vec[17],
                ],
                window_features: [
                    feature_vec[18], feature_vec[19], feature_vec[20], feature_vec[21],
                    feature_vec[22], feature_vec[23],
                ],
                temporal_features: [
                    feature_vec[24], feature_vec[25], feature_vec[26], feature_vec[27], feature_vec[28],
                ],
                resource_features: [
                    feature_vec[29], feature_vec[30], feature_vec[31], feature_vec[32],
                ],
                screenshot_features: Some([
                    feature_vec[33], feature_vec[34], feature_vec[35], feature_vec[36], feature_vec[37],
                    feature_vec[38], feature_vec[39], feature_vec[40], feature_vec[41], feature_vec[42],
                    feature_vec[43], feature_vec[44],
                ]),
            };

            // Parse label (last column)
            let label = parts[45].trim();
            let state = match label {
                "flow" => ADHDState::flow(),
                "hyperfocus" => ADHDState::hyperfocus(),
                "distracted" => ADHDState::distracted(),
                "transitioning" => ADHDState::transitioning(),
                "neutral" => ADHDState::neutral(),
                _ => return Err(AnalysisError::DataLoadError {
                    path: path.to_string(),
                    message: format!("Unknown state label at line {}: {}", line_num + 2, label),
                }),
            };

            samples.push((feature_vector, state));
        }

        Ok(samples)
    }

    /// Split data into training/validation/test sets
    fn split_data(&mut self, mut data: Vec<(FeatureVector, ADHDState)>) -> AnalysisResult<()> {
        if data.is_empty() {
            return Err(AnalysisError::DataLoadError {
                path: "memory".to_string(),
                message: "No data to split".to_string(),
            });
        }

        // Shuffle data for random splits
        use rand::seq::SliceRandom;
        let mut rng = rand::thread_rng();
        data.shuffle(&mut rng);

        let total_size = data.len();
        let train_size = (total_size as f32 * self.config.train_split) as usize;
        let val_size = (total_size as f32 * self.config.validation_split) as usize;

        // Split data
        self.test_data = data.split_off(train_size + val_size);
        self.validation_data = data.split_off(train_size);
        self.training_data = data;

        // Ensure minimum sample sizes
        if self.training_data.len() < self.config.min_training_samples {
            return Err(AnalysisError::DataLoadError {
                path: "memory".to_string(),
                message: format!("Insufficient training samples: {} < {}", 
                               self.training_data.len(), self.config.min_training_samples),
            });
        }

        Ok(())
    }

    /// Run hyperparameter optimization
    pub fn optimize_hyperparameters(&mut self) -> AnalysisResult<HyperparameterResults> {
        println!("Starting hyperparameter optimization...");
        let start_time = Instant::now();

        let mut best_accuracy = 0.0;
        let mut best_params = HashMap::new();
        let mut best_model_type = String::new();
        let mut cross_validation_scores = Vec::new();

        // Random search for hyperparameters
        for iteration in 0..self.config.max_optimization_iterations {
            println!("Optimization iteration {}/{}", iteration + 1, self.config.max_optimization_iterations);

            // Generate random hyperparameters
            let params = self.generate_random_hyperparameters();
            let model_type = self.select_model_type(&params);

            // Train model with these parameters
            let mut model = self.create_model(&model_type, &params)?;
            model.train(&self.training_data)?;

            // Evaluate with cross-validation
            let cv_scores = self.cross_validate(&model_type, &params)?;
            let avg_cv_score = cv_scores.iter().sum::<f32>() / cv_scores.len() as f32;

            cross_validation_scores.extend(cv_scores);

            println!("Model: {}, Params: {:?}, CV Score: {:.4}", model_type, params, avg_cv_score);

            // Update best if this is better
            if avg_cv_score > best_accuracy {
                best_accuracy = avg_cv_score;
                best_params = params;
                best_model_type = model_type;
                println!("New best accuracy: {:.4}", best_accuracy);
            }

            // Early stopping if target accuracy reached
            if best_accuracy >= self.config.target_accuracy {
                println!("Target accuracy {:.4} reached, stopping optimization", self.config.target_accuracy);
                break;
            }
        }

        let total_time = start_time.elapsed().as_secs_f32();

        // Train final model with best parameters
        println!("Training final model with best parameters...");
        let mut final_model = self.create_model(&best_model_type, &best_params)?;
        final_model.train(&self.training_data)?;

        // Validate accuracy requirement
        let validation_accuracy = self.evaluate_model(&*final_model)?;
        if validation_accuracy < 0.8 {
            return Err(AnalysisError::TrainingFailed {
                message: format!("Final model accuracy {:.3} below 80% requirement", validation_accuracy),
            });
        }

        self.best_model = Some(final_model);

        let results = HyperparameterResults {
            best_params,
            best_accuracy,
            best_model_type,
            optimization_iterations: iteration + 1,
            total_training_time_secs: total_time,
            cross_validation_scores,
        };

        println!("Hyperparameter optimization completed in {:.2}s", total_time);
        Ok(results)
    }

    /// Generate random hyperparameters
    fn generate_random_hyperparameters(&self) -> HashMap<String, f32> {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let mut params = HashMap::new();

        // Random Forest parameters
        params.insert("n_trees".to_string(), rng.gen_range(50..200) as f32);
        params.insert("max_depth".to_string(), rng.gen_range(5..20) as f32);
        params.insert("min_samples_split".to_string(), rng.gen_range(2..10) as f32);
        params.insert("min_samples_leaf".to_string(), rng.gen_range(1..5) as f32);

        // Learning parameters
        params.insert("learning_rate".to_string(), rng.gen_range(0.001..0.1));
        params.insert("temporal_smoothing".to_string(), rng.gen_range(0.1..0.9));

        params
    }

    /// Select model type based on parameters
    fn select_model_type(&self, _params: &HashMap<String, f32>) -> String {
        // For now, focus on Random Forest as it's well-implemented
        // In the future, you could select based on parameter combinations
        "random_forest".to_string()
    }

    /// Create model instance with parameters
    fn create_model(&self, model_type: &str, params: &HashMap<String, f32>) -> AnalysisResult<Box<dyn ModelTrait>> {
        match model_type {
            "random_forest" => {
                let config = RandomForestConfig {
                    n_trees: params.get("n_trees").unwrap_or(&100.0) as usize,
                    max_depth: Some(params.get("max_depth").unwrap_or(&10.0) as usize),
                    min_samples_split: params.get("min_samples_split").unwrap_or(&2.0) as usize,
                    min_samples_leaf: params.get("min_samples_leaf").unwrap_or(&1.0) as usize,
                    temporal_smoothing_alpha: *params.get("temporal_smoothing").unwrap_or(&0.7),
                    ..Default::default()
                };
                
                let model = RandomForestClassifier::with_config(config);
                Ok(Box::new(TrainableRandomForest::new(model)))
            }
            _ => Err(AnalysisError::InvalidInput {
                message: format!("Unknown model type: {}", model_type),
            }),
        }
    }

    /// Perform k-fold cross-validation
    fn cross_validate(&self, model_type: &str, params: &HashMap<String, f32>) -> AnalysisResult<Vec<f32>> {
        let k = self.config.cross_validation_folds;
        let mut scores = Vec::new();

        // Combine training and validation data for cross-validation
        let mut all_data = self.training_data.clone();
        all_data.extend(self.validation_data.clone());

        let fold_size = all_data.len() / k;

        for fold in 0..k {
            // Create train/test split for this fold
            let test_start = fold * fold_size;
            let test_end = if fold == k - 1 { all_data.len() } else { (fold + 1) * fold_size };

            let test_data = &all_data[test_start..test_end];
            let train_data: Vec<_> = all_data.iter()
                .enumerate()
                .filter(|(i, _)| *i < test_start || *i >= test_end)
                .map(|(_, item)| item.clone())
                .collect();

            // Train model on fold training data
            let mut model = self.create_model(model_type, params)?;
            model.train(&train_data)?;

            // Evaluate on fold test data
            let accuracy = self.evaluate_model_on_data(&*model, test_data)?;
            scores.push(accuracy);
        }

        Ok(scores)
    }

    /// Evaluate model accuracy
    fn evaluate_model(&self, model: &dyn ModelTrait) -> AnalysisResult<f32> {
        self.evaluate_model_on_data(model, &self.validation_data)
    }

    /// Evaluate model on specific dataset
    fn evaluate_model_on_data(&self, model: &dyn ModelTrait, data: &[(FeatureVector, ADHDState)]) -> AnalysisResult<f32> {
        let mut correct = 0;
        let mut total = 0;

        for (features, true_state) in data {
            let prediction = model.predict_sync(features)?;
            let predicted_state = prediction.most_likely_state().0;
            let true_state_type = crate::models::get_adhd_state_type(true_state);

            if predicted_state == true_state_type {
                correct += 1;
            }
            total += 1;
        }

        if total > 0 {
            Ok(correct as f32 / total as f32)
        } else {
            Ok(0.0)
        }
    }

    /// Export the best model to ONNX format
    pub fn export_to_onnx(&self, output_path: &str) -> AnalysisResult<()> {
        let model = self.best_model.as_ref().ok_or_else(|| AnalysisError::ModelNotFound {
            model_name: "best_model_not_trained".to_string(),
        })?;

        println!("Exporting model to ONNX format: {}", output_path);

        // Create model metadata
        let metadata = ModelMetadata {
            version: "1.0.0".to_string(),
            feature_count: 45,
            class_count: 5,
            accuracy: self.evaluate_model(&**model)?,
            model_type: "RandomForest".to_string(),
            training_date: chrono::Utc::now(),
            feature_importance: self.get_feature_importance(&**model),
        };

        // For now, we'll create a placeholder ONNX export
        // In a full implementation, you would use libraries like:
        // - sklearn-onnx for scikit-learn models
        // - torch.onnx for PyTorch models
        // - tf2onnx for TensorFlow models
        
        // Save metadata
        let onnx_classifier = ONNXClassifier::new()?;
        onnx_classifier.export_model_to_onnx(output_path, &metadata)?;

        println!("Model exported successfully with {:.3} accuracy", metadata.accuracy);
        Ok(())
    }

    /// Get feature importance from the model
    fn get_feature_importance(&self, model: &dyn ModelTrait) -> HashMap<String, f32> {
        model.feature_importance().into_iter().collect()
    }

    /// Get training statistics
    pub fn get_training_stats(&self) -> TrainingStats {
        TrainingStats {
            total_samples: self.training_data.len() + self.validation_data.len() + self.test_data.len(),
            training_samples: self.training_data.len(),
            validation_samples: self.validation_data.len(),
            test_samples: self.test_data.len(),
            epochs_completed: self.training_history.len(),
            best_accuracy: self.training_history.iter()
                .map(|epoch| epoch.validation_accuracy)
                .fold(0.0f32, |a, b| a.max(b)),
        }
    }
}

/// Training configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingConfig {
    /// Data split ratios
    pub train_split: f32,
    pub validation_split: f32,
    
    /// Hyperparameter optimization
    pub max_optimization_iterations: usize,
    pub target_accuracy: f32,
    pub cross_validation_folds: usize,
    
    /// Training constraints
    pub min_training_samples: usize,
    pub max_training_time_minutes: u32,
    
    /// Export settings
    pub export_onnx: bool,
    pub export_path: String,
}

impl Default for TrainingConfig {
    fn default() -> Self {
        Self {
            train_split: 0.7,
            validation_split: 0.15,  // Remaining 0.15 for test
            max_optimization_iterations: 50,
            target_accuracy: 0.85,
            cross_validation_folds: 5,
            min_training_samples: 1000,
            max_training_time_minutes: 60,
            export_onnx: true,
            export_path: "models/adhd_classifier.onnx".to_string(),
        }
    }
}

/// Training statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingStats {
    pub total_samples: usize,
    pub training_samples: usize,
    pub validation_samples: usize,
    pub test_samples: usize,
    pub epochs_completed: usize,
    pub best_accuracy: f32,
}

/// JSON training example format
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TrainingExample {
    pub features: FeatureVector,
    pub label: String,
    pub timestamp: Option<chrono::DateTime<chrono::Utc>>,
    pub confidence: Option<f32>,
}

/// Trait for trainable models (abstraction for training)
trait ModelTrait: Send + Sync {
    fn train(&mut self, data: &[(FeatureVector, ADHDState)]) -> AnalysisResult<()>;
    fn predict_sync(&self, features: &FeatureVector) -> AnalysisResult<crate::models::StateDistribution>;
    fn feature_importance(&self) -> Vec<(String, f32)>;
}

/// Wrapper for RandomForestClassifier to implement ModelTrait
struct TrainableRandomForest {
    classifier: RandomForestClassifier,
}

impl TrainableRandomForest {
    fn new(classifier: RandomForestClassifier) -> Self {
        Self { classifier }
    }
}

impl ModelTrait for TrainableRandomForest {
    fn train(&mut self, data: &[(FeatureVector, ADHDState)]) -> AnalysisResult<()> {
        self.classifier.train(data)
    }

    fn predict_sync(&self, features: &FeatureVector) -> AnalysisResult<crate::models::StateDistribution> {
        // Convert async to sync for training evaluation
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(self.classifier.predict(features))
    }

    fn feature_importance(&self) -> Vec<(String, f32)> {
        self.classifier.feature_importance()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_training_pipeline_creation() {
        let config = TrainingConfig::default();
        let pipeline = TrainingPipeline::new(config);
        
        assert_eq!(pipeline.training_data.len(), 0);
        assert_eq!(pipeline.validation_data.len(), 0);
        assert_eq!(pipeline.test_data.len(), 0);
    }

    #[test]
    fn test_hyperparameter_generation() {
        let config = TrainingConfig::default();
        let pipeline = TrainingPipeline::new(config);
        
        let params = pipeline.generate_random_hyperparameters();
        assert!(params.contains_key("n_trees"));
        assert!(params.contains_key("max_depth"));
        assert!(params.contains_key("learning_rate"));
    }

    #[test]
    fn test_model_type_selection() {
        let config = TrainingConfig::default();
        let pipeline = TrainingPipeline::new(config);
        
        let params = HashMap::new();
        let model_type = pipeline.select_model_type(&params);
        assert_eq!(model_type, "random_forest");
    }

    #[test]
    fn test_data_split_ratios() {
        let config = TrainingConfig {
            train_split: 0.6,
            validation_split: 0.2,
            ..Default::default()
        };
        
        assert_eq!(config.train_split + config.validation_split, 0.8);
        // Test split should be 0.2 (remaining)
    }
}