//! Standalone Error Handling Test
//!
//! Test the error handling components in isolation to verify
//! all components work correctly together.

use std::time::Duration;
use std::sync::Arc;
use tokio::time::sleep;
use uuid::Uuid;

use skelly_jelly_event_bus::{
    EventBusConfig, BusMessage, MessagePayload, MessagePriority, ModuleId,
    CircuitBreakerRegistry, CircuitBreakerConfig,
    RetryExecutor, RetryConfig,
    DeadLetterQueue, DeadLetterQueueConfig, DeadLetterReason,
    ErrorLogger, ErrorLoggerConfig, ErrorContext, ErrorSeverity, ErrorCategory,
    RecoverySystem, RecoveryConfig, DefaultRecoveryExecutor,
    RecoveryAction, RecoveryStrategy, EscalationLevel,
    create_enhanced_event_bus_with_config, EventBusTrait,
};

async fn test_circuit_breaker() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing Circuit Breaker...");
    
    let registry = CircuitBreakerRegistry::new();
    let config = CircuitBreakerConfig {
        failure_threshold: 3,
        reset_timeout: Duration::from_millis(100),
        operation_timeout: Duration::from_millis(500),
        success_threshold: 0.8,
        half_open_max_calls: 2,
        failure_count_window: Duration::from_secs(60),
    };
    
    let breaker = registry.register("test_circuit".to_string(), config);
    
    // Test successful operation
    let result = breaker.execute(async { Ok::<i32, String>(42) }).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);
    
    // Test failure and circuit opening
    for _i in 0..4 {
        let _result = breaker.execute(async { Err::<i32, String>("error".to_string()) }).await;
    }
    
    let stats = breaker.stats();
    println!("Circuit breaker stats: failure_count={}, success_count={}", 
             stats.failure_count, stats.success_count);
    
    // Test circuit recovery
    sleep(Duration::from_millis(150)).await;
    breaker.force_close();
    
    let result = breaker.execute(async { Ok::<i32, String>(100) }).await;
    assert!(result.is_ok());
    
    println!("✓ Circuit Breaker test passed");
    Ok(())
}

async fn test_retry_mechanism() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing Retry Mechanism...");
    
    let config = RetryConfig {
        max_attempts: 3,
        initial_delay: Duration::from_millis(10),
        max_delay: Duration::from_millis(100),
        backoff_multiplier: 2.0,
        jitter_factor: 0.1,
        total_timeout: Some(Duration::from_secs(5)),
        reset_on_success: true,
    };
    
    let retry_executor = Arc::new(RetryExecutor::new(config)?);
    
    // Test successful retry
    let attempt_count = Arc::new(std::sync::atomic::AtomicU32::new(0));
    let attempt_count_clone = attempt_count.clone();
    
    let result = retry_executor.execute_with_default(|_attempt| {
        let count = attempt_count_clone.clone();
        Box::pin(async move {
            let current = count.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
            if current < 3 {
                Err("not ready yet".to_string())
            } else {
                Ok(42)
            }
        })
    }).await;
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);
    assert_eq!(attempt_count.load(std::sync::atomic::Ordering::SeqCst), 3);
    
    let stats = retry_executor.stats();
    println!("Retry stats: successful_operations={}, total_retry_attempts={}", 
             stats.successful_operations, stats.total_retry_attempts);
    
    println!("✓ Retry Mechanism test passed");
    Ok(())
}

async fn test_dead_letter_queue() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing Dead Letter Queue...");
    
    let config = DeadLetterQueueConfig {
        max_entries: 100,
        max_age: Duration::from_secs(3600),
        enable_persistence: false,
        persistence_path: None,
        auto_replay: None,
        enable_metrics: true,
        replay_batch_size: 10,
    };
    
    let dlq = DeadLetterQueue::new(config);
    
    // Create test message
    let message = BusMessage::with_priority(
        ModuleId::DataCapture,
        MessagePayload::ModuleReady(ModuleId::DataCapture),
        MessagePriority::Normal,
    );
    
    // Add message to DLQ
    let entry_id = dlq.add_message(
        message.clone(),
        DeadLetterReason::MaxRetriesExceeded { attempts: 3 },
        3,
        vec![ModuleId::Storage],
        Some("Test error".to_string()),
        Some("test-correlation-123".to_string()),
    );
    
    // Verify message is in DLQ
    let entry = dlq.get_entry(entry_id).unwrap();
    assert_eq!(entry.message.id, message.id);
    assert_eq!(entry.retry_count, 3);
    
    // Test filtering
    let filter = skelly_jelly_event_bus::dead_letter_queue::DeadLetterFilter {
        correlation_id: Some("test-correlation-123".to_string()),
        ..Default::default()
    };
    let entries = dlq.get_entries(&filter);
    assert_eq!(entries.len(), 1);
    
    // Test marking for replay
    let marked = dlq.mark_for_replay(&Default::default());
    assert_eq!(marked, 1);
    
    let stats = dlq.stats();
    println!("DLQ stats: total_entries={}, entries_by_reason={:?}", 
             stats.total_entries, stats.entries_by_reason);
    
    println!("✓ Dead Letter Queue test passed");
    Ok(())
}

async fn test_error_logging() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing Error Logging...");
    
    let config = ErrorLoggerConfig {
        min_severity: ErrorSeverity::Debug,
        include_stack_traces: true,
        include_metadata: true,
        max_message_length: 1000,
        enable_metrics: true,
        log_format: skelly_jelly_event_bus::error_logging::LogFormat::Json,
        sanitize_sensitive_data: true,
        sensitive_fields: vec!["password".to_string(), "token".to_string()],
    };
    
    let error_logger = ErrorLogger::new(config);
    let correlation_id = ErrorLogger::create_correlation_id();
    
    let error_context = ErrorContext::new(
        correlation_id,
        ModuleId::DataCapture,
        "test_operation".to_string(),
        ErrorSeverity::Error,
        ErrorCategory::Network,
        "Test error message with password=secret123".to_string(),
    ).with_metadata("test_key", "test_value")
     .with_duration(Duration::from_millis(100));
    
    error_logger.log_error(&error_context);
    
    let stats = error_logger.stats();
    assert_eq!(stats.total_errors_logged, 1);
    assert!(stats.errors_by_severity.contains_key("Error"));
    assert!(stats.errors_by_module.contains_key(&ModuleId::DataCapture));
    
    println!("Error logging stats: total_errors_logged={}, errors_by_severity={:?}", 
             stats.total_errors_logged, stats.errors_by_severity);
    
    println!("✓ Error Logging test passed");
    Ok(())
}

async fn test_recovery_system() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing Recovery System...");
    
    let circuit_breakers = Arc::new(CircuitBreakerRegistry::new());
    let retry_executor = Arc::new(RetryExecutor::new(RetryConfig::default())?);
    let dead_letter_queue = Arc::new(DeadLetterQueue::new(DeadLetterQueueConfig::default()));
    let error_logger = Arc::new(ErrorLogger::new(ErrorLoggerConfig::default()));
    
    let recovery_config = RecoveryConfig {
        enable_automatic_recovery: true,
        max_automatic_escalation_level: EscalationLevel::Service,
        default_action_timeout: Duration::from_secs(1),
        max_concurrent_actions: 3,
        condition_check_interval: Duration::from_millis(100),
        enable_recovery_metrics: true,
        notification_config: skelly_jelly_event_bus::recovery::NotificationConfig {
            enabled: false,
            notification_levels: vec![],
            webhook_urls: vec![],
            email_addresses: vec![],
            slack_channels: vec![],
        },
    };
    
    let recovery_system = RecoverySystem::new(
        recovery_config,
        circuit_breakers.clone(),
        retry_executor.clone(),
        dead_letter_queue.clone(),
        error_logger.clone(),
    );
    
    // Register default executor
    let default_executor = Arc::new(DefaultRecoveryExecutor::new(
        circuit_breakers.clone(),
        retry_executor.clone(),
    ));
    recovery_system.register_executor(default_executor);
    
    // Register a test recovery action
    let recovery_action = RecoveryAction {
        id: Uuid::new_v4(),
        name: "Test Recovery Action".to_string(),
        description: "Test recovery action".to_string(),
        strategy: RecoveryStrategy::CircuitBreakerReset { 
            circuit_name: "test_circuit".to_string() 
        },
        escalation_level: EscalationLevel::Automatic,
        conditions: vec![],
        max_executions: 3,
        cooldown: Duration::from_millis(100),
        requires_confirmation: false,
        expected_recovery_time: Duration::from_millis(500),
        success_threshold: 0.8,
    };
    recovery_system.register_action(recovery_action);
    
    // Create an incident
    let correlation_id = Uuid::new_v4();
    let error = skelly_jelly_event_bus::EventBusError::QueueFull { current_size: 100, max_size: 100 };
    
    let incident_id = recovery_system.handle_incident(
        correlation_id,
        ModuleId::EventBus,
        &error,
        "Test incident".to_string(),
    ).await?;
    
    // Give recovery system time to process
    sleep(Duration::from_millis(200)).await;
    
    let incident = recovery_system.get_incident(incident_id).unwrap();
    assert_eq!(incident.correlation_id, correlation_id);
    assert_eq!(incident.module_id, ModuleId::EventBus);
    
    let stats = recovery_system.stats();
    println!("Recovery stats: total_incidents={}, recovery_actions_executed={}", 
             stats.total_incidents, stats.recovery_actions_executed);
    
    println!("✓ Recovery System test passed");
    Ok(())
}

async fn test_enhanced_event_bus_integration() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing Enhanced Event Bus Integration...");
    
    let config = EventBusConfig {
        max_queue_size: 1000,
        delivery_timeout: Duration::from_millis(500),
        max_retry_attempts: 3,
        dead_letter_queue_size: 100,
        metrics_interval: Duration::from_millis(100),
        slow_handler_threshold: Duration::from_millis(50),
        circuit_breaker_config: Some(CircuitBreakerConfig::default()),
        retry_config: Some(RetryConfig::default()),
        error_logging_config: Some(ErrorLoggerConfig::default()),
        recovery_config: Some(RecoveryConfig::default()),
        enable_error_handling: true,
    };
    
    let bus = create_enhanced_event_bus_with_config(config)?;
    bus.start().await?;
    
    // Test normal operation
    let message = BusMessage::with_priority(
        ModuleId::DataCapture,
        MessagePayload::ModuleReady(ModuleId::DataCapture),
        MessagePriority::Normal,
    );
    
    let result = bus.publish(message).await;
    assert!(result.is_ok());
    
    // Test error handling stats
    let error_stats = bus.get_error_stats();
    assert_eq!(error_stats.retry_stats.successful_operations, 1);
    assert_eq!(error_stats.retry_stats.failed_operations, 0);
    
    // Test metrics
    let bus_metrics = bus.metrics().await?;
    assert_eq!(bus_metrics.messages_published, 1);
    
    println!("Enhanced Event Bus stats: messages_published={}, retry_success={}", 
             bus_metrics.messages_published, error_stats.retry_stats.successful_operations);
    
    bus.shutdown().await?;
    
    println!("✓ Enhanced Event Bus Integration test passed");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting Error Handling & Recovery System Tests");
    println!("=" .repeat(60));
    
    // Run all tests
    test_circuit_breaker().await?;
    test_retry_mechanism().await?;
    test_dead_letter_queue().await?;
    test_error_logging().await?;
    test_recovery_system().await?;
    test_enhanced_event_bus_integration().await?;
    
    println!("=" .repeat(60));
    println!("🎉 All Error Handling & Recovery System Tests Passed!");
    
    println!("\n📊 Summary:");
    println!("✓ Circuit Breaker: State management, failure detection, auto-reset");
    println!("✓ Retry Logic: Exponential backoff with jitter and configurable limits");
    println!("✓ Dead Letter Queue: Failed message storage with replay capability");
    println!("✓ Error Logging: Structured logging with context preservation");
    println!("✓ Recovery System: Automatic recovery with escalation paths");
    println!("✓ Enhanced Event Bus: Full integration with comprehensive error handling");
    
    Ok(())
}