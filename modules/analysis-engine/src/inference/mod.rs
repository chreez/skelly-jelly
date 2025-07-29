//! Real-time inference engine for ADHD state detection
//!
//! This module provides a high-performance inference pipeline that:
//! - Processes event batches in real-time with <50ms latency
//! - Coordinates feature extraction and ML model inference
//! - Manages prediction caching and optimization
//! - Handles concurrent inference requests efficiently
//! - Provides comprehensive performance monitoring

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};
use tokio::sync::{RwLock, Semaphore};
use uuid::Uuid;

use crate::{
    error::{AnalysisError, AnalysisResult},
    models::{ADHDState, StateDistribution},
    state_detection::{StateDetectionEngine, StateDetectionResult},
    sliding_window::AnalysisWindow,
    types::FeatureVector,
};

/// High-performance real-time inference engine
pub struct InferenceEngine {
    /// State detection engine
    state_detector: Arc<StateDetectionEngine>,
    
    /// Configuration for inference
    config: InferenceConfig,
    
    /// Prediction cache for performance optimization
    prediction_cache: Arc<RwLock<PredictionCache>>,
    
    /// Concurrency control for inference requests
    inference_semaphore: Arc<Semaphore>,
    
    /// Performance metrics
    metrics: Arc<InferenceMetrics>,
    
    /// Request tracking
    active_requests: Arc<RwLock<HashMap<Uuid, InferenceRequest>>>,
}

/// Cached prediction result
#[derive(Debug, Clone)]
struct CachedPrediction {
    result: StateDetectionResult,
    cache_time: Instant,
    hit_count: u32,
}

/// Prediction cache with LRU eviction
struct PredictionCache {
    cache: HashMap<FeatureCacheKey, CachedPrediction>,
    max_size: usize,
    ttl: Duration,
}

/// Cache key based on feature vector hash
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct FeatureCacheKey {
    feature_hash: u64,
    window_duration_ms: u64,
}

/// Active inference request tracking
#[derive(Debug, Clone)]
struct InferenceRequest {
    id: Uuid,
    start_time: Instant,
    window_id: Uuid,
    priority: InferencePriority,
}

/// Priority levels for inference requests
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum InferencePriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

/// Performance metrics for the inference engine
pub struct InferenceMetrics {
    total_requests: AtomicU64,
    successful_inferences: AtomicU64,
    failed_inferences: AtomicU64,
    cache_hits: AtomicU64,
    cache_misses: AtomicU64,
    avg_latency_ms: RwLock<f32>,
    max_latency_ms: RwLock<f32>,
    concurrent_requests: RwLock<u32>,
    throughput_per_sec: RwLock<f32>,
}

impl InferenceEngine {
    /// Create a new inference engine
    pub fn new(state_detector: Arc<StateDetectionEngine>) -> Self {
        Self::with_config(state_detector, InferenceConfig::default())
    }
    
    /// Create with custom configuration
    pub fn with_config(
        state_detector: Arc<StateDetectionEngine>,
        config: InferenceConfig,
    ) -> Self {
        let cache = PredictionCache {
            cache: HashMap::new(),
            max_size: config.cache_max_size,
            ttl: config.cache_ttl,
        };
        
        let max_concurrent = config.max_concurrent_inferences;
        
        Self {
            state_detector,
            config,
            prediction_cache: Arc::new(RwLock::new(cache)),
            inference_semaphore: Arc::new(Semaphore::new(max_concurrent)),
            metrics: Arc::new(InferenceMetrics::new()),
            active_requests: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Perform real-time inference on analysis window
    pub async fn infer(&self, window: &AnalysisWindow) -> AnalysisResult<StateDetectionResult> {
        self.infer_with_priority(window, InferencePriority::Normal).await
    }
    
    /// Perform inference with specified priority
    pub async fn infer_with_priority(
        &self,
        window: &AnalysisWindow,
        priority: InferencePriority,
    ) -> AnalysisResult<StateDetectionResult> {
        let request_id = Uuid::new_v4();
        let start_time = Instant::now();
        
        // Acquire semaphore permit for concurrency control
        let _permit = self.acquire_inference_permit(priority).await?;
        
        // Track request
        let request = InferenceRequest {
            id: request_id,
            start_time,
            window_id: window.window_id,
            priority,
        };
        self.track_request_start(request.clone()).await;
        
        // Increment request counter
        self.metrics.total_requests.fetch_add(1, Ordering::Relaxed);
        
        let result = self.perform_inference_internal(window).await;
        
        // Track completion
        self.track_request_completion(request_id, start_time, result.is_ok()).await;
        
        match result {
            Ok(detection_result) => {
                self.metrics.successful_inferences.fetch_add(1, Ordering::Relaxed);
                Ok(detection_result)
            }
            Err(error) => {
                self.metrics.failed_inferences.fetch_add(1, Ordering::Relaxed);
                Err(error)
            }
        }
    }
    
    /// Internal inference implementation with caching
    async fn perform_inference_internal(&self, window: &AnalysisWindow) -> AnalysisResult<StateDetectionResult> {
        // Check cache first
        if let Some(cached_result) = self.check_cache(window).await? {
            self.metrics.cache_hits.fetch_add(1, Ordering::Relaxed);
            return Ok(cached_result);
        }
        
        self.metrics.cache_misses.fetch_add(1, Ordering::Relaxed);
        
        // Perform actual inference
        let detection_result = self.state_detector.detect_state(window).await?;
        
        // Cache the result
        self.cache_result(window, &detection_result).await?;
        
        Ok(detection_result)
    }
    
    /// Check prediction cache for existing result
    async fn check_cache(&self, window: &AnalysisWindow) -> AnalysisResult<Option<StateDetectionResult>> {
        if !self.config.enable_caching {
            return Ok(None);
        }
        
        let cache_key = self.create_cache_key(window).await?;
        let mut cache = self.prediction_cache.write().await;
        
        let cache_ttl = cache.ttl;
        if let Some(cached_prediction) = cache.cache.get_mut(&cache_key) {
            // Check if cache entry is still valid
            if cached_prediction.cache_time.elapsed() <= cache_ttl {
                cached_prediction.hit_count += 1;
                return Ok(Some(cached_prediction.result.clone()));
            } else {
                // Mark for removal (will be removed after the scope)
                let expired_key = cache_key.clone();
                drop(cached_prediction); // Drop the mutable borrow
                cache.cache.remove(&expired_key);
            }
        }
        
        Ok(None)
    }
    
    /// Cache inference result
    async fn cache_result(&self, window: &AnalysisWindow, result: &StateDetectionResult) -> AnalysisResult<()> {
        if !self.config.enable_caching {
            return Ok(());
        }
        
        let cache_key = self.create_cache_key(window).await?;
        let mut cache = self.prediction_cache.write().await;
        
        // Evict old entries if cache is full
        if cache.cache.len() >= cache.max_size {
            self.evict_cache_entries(&mut cache).await;
        }
        
        let cached_prediction = CachedPrediction {
            result: result.clone(),
            cache_time: Instant::now(),
            hit_count: 0,
        };
        
        cache.cache.insert(cache_key, cached_prediction);
        
        Ok(())
    }
    
    /// Create cache key for window
    async fn create_cache_key(&self, window: &AnalysisWindow) -> AnalysisResult<FeatureCacheKey> {
        // Create a simple hash of the window events for cache key
        let events_hash = self.hash_window_events(window);
        let duration_ms = window.duration().as_millis() as u64;
        
        Ok(FeatureCacheKey {
            feature_hash: events_hash,
            window_duration_ms: duration_ms,
        })
    }
    
    /// Hash window events for cache key
    fn hash_window_events(&self, window: &AnalysisWindow) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        
        // Hash event count and basic properties
        window.events.len().hash(&mut hasher);
        
        // Hash first few and last few events for efficiency
        let event_sample_size = 10.min(window.events.len());
        for event in window.events.iter().take(event_sample_size) {
            event.timestamp().timestamp_millis().hash(&mut hasher);
        }
        
        hasher.finish()
    }
    
    /// Evict old cache entries using LRU policy
    async fn evict_cache_entries(&self, cache: &mut PredictionCache) {
        // Find entries to evict (oldest first, then by hit count)
        let mut entries_to_remove: Vec<_> = cache.cache.iter()
            .map(|(k, v)| (k.clone(), v.cache_time, v.hit_count))
            .collect();
        
        entries_to_remove.sort_by(|a, b| {
            // First by age (older first)
            let age_cmp = b.1.cmp(&a.1);
            if age_cmp == std::cmp::Ordering::Equal {
                // Then by hit count (fewer hits first)
                a.2.cmp(&b.2)
            } else {
                age_cmp
            }
        });
        
        // Remove oldest 25% of entries
        let remove_count = (cache.max_size / 4).max(1);
        for (key, _, _) in entries_to_remove.iter().take(remove_count) {
            cache.cache.remove(key);
        }
    }
    
    /// Acquire inference permit with priority handling
    async fn acquire_inference_permit(&self, priority: InferencePriority) -> AnalysisResult<tokio::sync::SemaphorePermit> {
        let timeout = match priority {
            InferencePriority::Critical => Duration::from_millis(10),
            InferencePriority::High => Duration::from_millis(25),
            InferencePriority::Normal => Duration::from_millis(50),
            InferencePriority::Low => Duration::from_millis(100),
        };
        
        tokio::time::timeout(timeout, self.inference_semaphore.acquire())
            .await
            .map_err(|_| AnalysisError::TimeoutError {
                operation: "acquire_inference_permit".to_string(),
                timeout_ms: timeout.as_millis() as u64,
            })?
            .map_err(|_| AnalysisError::ConcurrencyError {
                operation: "semaphore_acquisition".to_string(),
            })
    }
    
    /// Track request start
    async fn track_request_start(&self, request: InferenceRequest) {
        let mut active_requests = self.active_requests.write().await;
        active_requests.insert(request.id, request);
        
        let concurrent_count = active_requests.len() as u32;
        *self.metrics.concurrent_requests.write().await = concurrent_count;
    }
    
    /// Track request completion and update metrics
    async fn track_request_completion(&self, request_id: Uuid, start_time: Instant, success: bool) {
        let latency_ms = start_time.elapsed().as_millis() as f32;
        
        // Remove from active requests
        {
            let mut active_requests = self.active_requests.write().await;
            active_requests.remove(&request_id);
            *self.metrics.concurrent_requests.write().await = active_requests.len() as u32;
        }
        
        // Update latency metrics
        {
            let mut avg_latency = self.metrics.avg_latency_ms.write().await;
            let total_requests = self.metrics.total_requests.load(Ordering::Relaxed) as f32;
            *avg_latency = (*avg_latency * (total_requests - 1.0) + latency_ms) / total_requests;
        }
        
        {
            let mut max_latency = self.metrics.max_latency_ms.write().await;
            if latency_ms > *max_latency {
                *max_latency = latency_ms;
            }
        }
        
        // Check latency requirement
        if latency_ms > self.config.max_inference_latency_ms {
            eprintln!("Warning: Inference latency {}ms exceeds requirement of {}ms", 
                     latency_ms, self.config.max_inference_latency_ms);
        }
    }
    
    /// Batch inference for multiple windows
    pub async fn batch_infer(&self, windows: &[AnalysisWindow]) -> Vec<AnalysisResult<StateDetectionResult>> {
        let mut results = Vec::with_capacity(windows.len());
        
        // Process windows concurrently up to semaphore limit
        let futures: Vec<_> = windows.iter()
            .map(|window| self.infer(window))
            .collect();
        
        // Wait for all to complete
        for future in futures {
            results.push(future.await);
        }
        
        results
    }
    
    /// Get current performance metrics
    pub async fn get_metrics(&self) -> InferenceEngineMetrics {
        let total_requests = self.metrics.total_requests.load(Ordering::Relaxed);
        let successful = self.metrics.successful_inferences.load(Ordering::Relaxed);
        let failed = self.metrics.failed_inferences.load(Ordering::Relaxed);
        let cache_hits = self.metrics.cache_hits.load(Ordering::Relaxed);
        let cache_misses = self.metrics.cache_misses.load(Ordering::Relaxed);
        
        let success_rate = if total_requests > 0 {
            successful as f32 / total_requests as f32
        } else {
            0.0
        };
        
        let cache_hit_rate = if cache_hits + cache_misses > 0 {
            cache_hits as f32 / (cache_hits + cache_misses) as f32
        } else {
            0.0
        };
        
        InferenceEngineMetrics {
            total_requests,
            successful_inferences: successful,
            failed_inferences: failed,
            success_rate,
            cache_hits,
            cache_misses,
            cache_hit_rate,
            avg_latency_ms: *self.metrics.avg_latency_ms.read().await,
            max_latency_ms: *self.metrics.max_latency_ms.read().await,
            concurrent_requests: *self.metrics.concurrent_requests.read().await,
            throughput_per_sec: *self.metrics.throughput_per_sec.read().await,
        }
    }
    
    /// Warm up cache with training data
    pub async fn warmup_cache(&self, training_windows: &[AnalysisWindow]) -> AnalysisResult<()> {
        println!("Warming up inference cache with {} windows...", training_windows.len());
        
        let warmup_results = self.batch_infer(training_windows).await;
        let successful_warmups = warmup_results.iter().filter(|r| r.is_ok()).count();
        
        println!("Cache warmup completed: {}/{} successful inferences", 
                successful_warmups, training_windows.len());
        
        Ok(())
    }
    
    /// Clear prediction cache
    pub async fn clear_cache(&self) {
        let mut cache = self.prediction_cache.write().await;
        cache.cache.clear();
    }
    
    /// Get cache statistics
    pub async fn get_cache_stats(&self) -> CacheStatistics {
        let cache = self.prediction_cache.read().await;
        let current_size = cache.cache.len();
        let max_size = cache.max_size;
        
        let total_hit_count: u32 = cache.cache.values().map(|pred| pred.hit_count).sum();
        let avg_age_ms = if !cache.cache.is_empty() {
            cache.cache.values()
                .map(|pred| pred.cache_time.elapsed().as_millis() as u32)
                .sum::<u32>() / cache.cache.len() as u32
        } else {
            0
        };
        
        CacheStatistics {
            current_size,
            max_size,
            utilization: current_size as f32 / max_size as f32,
            total_hit_count,
            avg_age_ms,
            ttl_ms: cache.ttl.as_millis() as u32,
        }
    }
}

impl InferenceMetrics {
    fn new() -> Self {
        Self {
            total_requests: AtomicU64::new(0),
            successful_inferences: AtomicU64::new(0),
            failed_inferences: AtomicU64::new(0),
            cache_hits: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
            avg_latency_ms: RwLock::new(0.0),
            max_latency_ms: RwLock::new(0.0),
            concurrent_requests: RwLock::new(0),
            throughput_per_sec: RwLock::new(0.0),
        }
    }
}

/// Configuration for the inference engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceConfig {
    /// Maximum inference latency requirement (ms)
    pub max_inference_latency_ms: f32,
    
    /// Maximum concurrent inferences
    pub max_concurrent_inferences: usize,
    
    /// Enable prediction caching
    pub enable_caching: bool,
    
    /// Maximum cache size (number of entries)
    pub cache_max_size: usize,
    
    /// Cache time-to-live
    pub cache_ttl: Duration,
    
    /// Batch processing configuration
    pub max_batch_size: usize,
    pub batch_timeout_ms: u64,
    
    /// Performance monitoring
    pub enable_metrics: bool,
    pub metrics_update_interval: Duration,
}

impl Default for InferenceConfig {
    fn default() -> Self {
        Self {
            max_inference_latency_ms: 50.0,
            max_concurrent_inferences: 10,
            enable_caching: true,
            cache_max_size: 1000,
            cache_ttl: Duration::from_secs(300), // 5 minutes
            max_batch_size: 50,
            batch_timeout_ms: 100,
            enable_metrics: true,
            metrics_update_interval: Duration::from_secs(30),
        }
    }
}

/// Performance metrics snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceEngineMetrics {
    pub total_requests: u64,
    pub successful_inferences: u64,
    pub failed_inferences: u64,
    pub success_rate: f32,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub cache_hit_rate: f32,
    pub avg_latency_ms: f32,
    pub max_latency_ms: f32,
    pub concurrent_requests: u32,
    pub throughput_per_sec: f32,
}

/// Cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStatistics {
    pub current_size: usize,
    pub max_size: usize,
    pub utilization: f32,
    pub total_hit_count: u32,
    pub avg_age_ms: u32,
    pub ttl_ms: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{sliding_window::AnalysisWindow, state_detection::StateDetectionEngine};
    use std::time::SystemTime;

    #[tokio::test]
    async fn test_inference_engine_creation() {
        let state_detector = Arc::new(StateDetectionEngine::new());
        let engine = InferenceEngine::new(state_detector);
        
        let metrics = engine.get_metrics().await;
        assert_eq!(metrics.total_requests, 0);
    }

    #[tokio::test]
    async fn test_cache_key_generation() {
        let state_detector = Arc::new(StateDetectionEngine::new());
        let engine = InferenceEngine::new(state_detector);
        
        let window = AnalysisWindow::new(SystemTime::now());
        let cache_key = engine.create_cache_key(&window).await.unwrap();
        
        assert!(cache_key.feature_hash > 0);
        assert!(cache_key.window_duration_ms >= 0);
    }

    #[tokio::test]
    async fn test_priority_inference() {
        let state_detector = Arc::new(StateDetectionEngine::new());
        let config = InferenceConfig {
            max_concurrent_inferences: 1,
            ..Default::default()
        };
        let engine = InferenceEngine::with_config(state_detector, config);
        
        let window = AnalysisWindow::new(SystemTime::now());
        
        // This should fail because model is not trained, but we're testing the priority system
        let result = engine.infer_with_priority(&window, InferencePriority::High).await;
        assert!(result.is_err()); // Expected to fail due to untrained model
    }

    #[tokio::test]
    async fn test_cache_eviction() {
        let state_detector = Arc::new(StateDetectionEngine::new());
        let config = InferenceConfig {
            cache_max_size: 5,
            ..Default::default()
        };
        let engine = InferenceEngine::with_config(state_detector, config);
        
        let mut cache = PredictionCache {
            cache: HashMap::new(),
            max_size: 5,
            ttl: Duration::from_secs(300),
        };
        
        // Fill cache beyond capacity
        for i in 0..10 {
            let key = FeatureCacheKey {
                feature_hash: i,
                window_duration_ms: 1000,
            };
            let cached = CachedPrediction {
                result: StateDetectionResult {
                    window_id: Uuid::new_v4(),
                    timestamp: chrono::Utc::now(),
                    detected_state: crate::models::ADHDState::neutral(),
                    state_distribution: StateDistribution::new(),
                    confidence: 0.5,
                    temporal_stability: 0.5,
                    processing_time_ms: 25.0,
                    feature_importance: vec![],
                    intervention_readiness: 0.5,
                    transition_stability: 0.5,
                },
                cache_time: Instant::now(),
                hit_count: 0,
            };
            cache.cache.insert(key, cached);
        }
        
        engine.evict_cache_entries(&mut cache).await;
        
        // Should have evicted some entries
        assert!(cache.cache.len() < 10);
    }

    #[tokio::test]
    async fn test_metrics_tracking() {
        let state_detector = Arc::new(StateDetectionEngine::new());
        let engine = InferenceEngine::new(state_detector);
        
        let start_time = Instant::now();
        engine.track_request_completion(Uuid::new_v4(), start_time, true).await;
        
        let metrics = engine.get_metrics().await;
        assert!(metrics.avg_latency_ms >= 0.0);
    }
}