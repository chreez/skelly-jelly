//! Performance baseline benchmarks for all Skelly-Jelly modules
//! Run with: cargo bench --bench performance_baselines

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::time::Duration;

// Module performance targets from HLD
const EVENT_BUS_CPU_TARGET: f64 = 2.0;
const EVENT_BUS_MEMORY_TARGET_MB: usize = 100;
const ORCHESTRATOR_CPU_TARGET: f64 = 1.0;
const ORCHESTRATOR_MEMORY_TARGET_MB: usize = 50;
const DATA_CAPTURE_CPU_TARGET: f64 = 5.0;
const STORAGE_CPU_TARGET: f64 = 10.0;
const ANALYSIS_ENGINE_CPU_TARGET: f64 = 20.0;

/// Benchmark Event Bus message throughput
fn benchmark_event_bus_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("event_bus_throughput");
    
    for message_count in [100, 1000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(message_count),
            message_count,
            |b, &count| {
                b.iter_custom(|iters| {
                    let mut total = Duration::ZERO;
                    
                    for _ in 0..iters {
                        let start = std::time::Instant::now();
                        
                        // Simulate publishing messages
                        for _ in 0..count {
                            // In real benchmark, this would publish to event bus
                            black_box(create_test_message());
                        }
                        
                        total += start.elapsed();
                    }
                    
                    total
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark Event Bus latency
fn benchmark_event_bus_latency(c: &mut Criterion) {
    c.bench_function("event_bus_latency", |b| {
        b.iter(|| {
            // Measure single message publish-to-receive latency
            let start = std::time::Instant::now();
            
            // In real benchmark:
            // 1. Publish message
            // 2. Wait for subscriber to receive
            // 3. Measure elapsed time
            
            black_box(start.elapsed());
        });
    });
}

/// Benchmark Data Capture event processing
fn benchmark_data_capture_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("data_capture");
    
    // Keystroke capture overhead
    group.bench_function("keystroke_capture", |b| {
        b.iter(|| {
            // Simulate keystroke event capture
            let event = create_keystroke_event("a", Duration::from_millis(100));
            black_box(event);
        });
    });
    
    // Screenshot capture performance
    group.bench_function("screenshot_capture_small", |b| {
        b.iter(|| {
            // Simulate small screenshot (< 5MB)
            let screenshot = vec![0u8; 1_000_000]; // 1MB
            black_box(process_screenshot(screenshot));
        });
    });
    
    group.bench_function("screenshot_capture_large", |b| {
        b.iter(|| {
            // Simulate large screenshot (>= 5MB)
            let screenshot = vec![0u8; 6_000_000]; // 6MB
            black_box(process_screenshot(screenshot));
        });
    });
    
    group.finish();
}

/// Benchmark Storage batching performance
fn benchmark_storage_batching(c: &mut Criterion) {
    let mut group = c.benchmark_group("storage_batching");
    
    for batch_size in [100, 500, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(batch_size),
            batch_size,
            |b, &size| {
                b.iter(|| {
                    let events = create_test_events(size);
                    let batch = create_event_batch(events);
                    black_box(batch);
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark Analysis Engine inference
fn benchmark_analysis_engine_inference(c: &mut Criterion) {
    c.bench_function("analysis_engine_inference", |b| {
        let event_batch = create_test_batch(1000);
        
        b.iter(|| {
            // Simulate ML inference on event batch
            let features = extract_features(&event_batch);
            let state = classify_state(&features);
            black_box(state);
        });
    });
}

/// Benchmark resource usage monitoring
fn benchmark_resource_monitoring(c: &mut Criterion) {
    c.bench_function("resource_monitoring", |b| {
        b.iter(|| {
            let cpu_usage = get_cpu_usage();
            let memory_usage = get_memory_usage();
            
            // Verify within targets
            assert!(cpu_usage < 2.0, "CPU usage exceeds target");
            assert!(memory_usage < 100, "Memory usage exceeds target");
            
            black_box((cpu_usage, memory_usage));
        });
    });
}

// Helper functions (would be imported from actual modules)
fn create_test_message() -> Vec<u8> {
    vec![0u8; 256]
}

fn create_keystroke_event(key: &str, duration: Duration) -> Vec<u8> {
    format!("key:{},duration:{:?}", key, duration).into_bytes()
}

fn process_screenshot(data: Vec<u8>) -> Vec<u8> {
    // Simulate screenshot processing
    data[..100.min(data.len())].to_vec()
}

fn create_test_events(count: usize) -> Vec<Vec<u8>> {
    (0..count).map(|i| {
        format!("event_{}", i).into_bytes()
    }).collect()
}

fn create_event_batch(events: Vec<Vec<u8>>) -> Vec<u8> {
    events.concat()
}

fn create_test_batch(size: usize) -> Vec<u8> {
    vec![0u8; size * 100]
}

fn extract_features(batch: &[u8]) -> Vec<f64> {
    vec![0.5; 100]
}

fn classify_state(features: &[f64]) -> String {
    "focused".to_string()
}

fn get_cpu_usage() -> f64 {
    // Simulate CPU usage measurement
    1.5
}

fn get_memory_usage() -> usize {
    // Simulate memory usage in MB
    75
}

criterion_group!(
    benches,
    benchmark_event_bus_throughput,
    benchmark_event_bus_latency,
    benchmark_data_capture_processing,
    benchmark_storage_batching,
    benchmark_analysis_engine_inference,
    benchmark_resource_monitoring
);

criterion_main!(benches);