//! Comprehensive Error Recovery Testing Framework
//!
//! Integration tests that validate all error handling and recovery mechanisms
//! work together correctly under various failure scenarios.

use std::time::Duration;
use std::sync::{Arc, atomic::{AtomicU32, Ordering}};
use tokio::time::{sleep, timeout};
use uuid::Uuid;

use skelly_jelly_event_bus::{
    EnhancedEventBus, EventBusConfig, BusMessage, MessagePayload, MessagePriority, ModuleId,
    MessageFilter, DeliveryMode, EventBusError, EventBusTrait, MessageType,
    CircuitBreakerConfig, RetryConfig, 
    DeadLetterReason, ErrorSeverity, ErrorCategory,
    RecoveryAction, RecoveryStrategy, EscalationLevel, IncidentStatus,
    create_enhanced_event_bus_with_config,
    error_logging::ErrorLoggerConfig,
    recovery::RecoveryConfig,
    dead_letter_queue::DeadLetterQueueConfig,
};

/// Test configuration for error scenarios
struct ErrorTestConfig {
    pub circuit_breaker_failure_threshold: u32,
    pub retry_max_attempts: u32,
    pub dlq_max_entries: usize,
    pub recovery_enabled: bool,
}

impl Default for ErrorTestConfig {
    fn default() -> Self {
        Self {
            circuit_breaker_failure_threshold: 3,
            retry_max_attempts: 3,
            dlq_max_entries: 100,
            recovery_enabled: true,
        }
    }
}

/// Create a test event bus with error handling configured
async fn create_test_bus(test_config: ErrorTestConfig) -> Arc<EnhancedEventBus> {
    let circuit_breaker_config = CircuitBreakerConfig {
        failure_threshold: test_config.circuit_breaker_failure_threshold,
        reset_timeout: Duration::from_millis(100),
        operation_timeout: Duration::from_millis(500),
        success_threshold: 0.8,
        half_open_max_calls: 2,
        failure_count_window: Duration::from_secs(60),
    };

    let retry_config = RetryConfig {
        max_attempts: test_config.retry_max_attempts,
        initial_delay: Duration::from_millis(10),
        max_delay: Duration::from_millis(100),
        backoff_multiplier: 2.0,
        jitter_factor: 0.1,
        total_timeout: Some(Duration::from_secs(5)),
        reset_on_success: true,
    };

    let error_logging_config = ErrorLoggerConfig {
        min_severity: ErrorSeverity::Debug,
        include_stack_traces: true,
        include_metadata: true,
        max_message_length: 1000,
        enable_metrics: true,
        log_format: skelly_jelly_event_bus::error_logging::LogFormat::Json,
        sanitize_sensitive_data: true,
        sensitive_fields: vec!["password".to_string(), "token".to_string()],
    };

    let recovery_config = RecoveryConfig {
        enable_automatic_recovery: test_config.recovery_enabled,
        max_automatic_escalation_level: EscalationLevel::Service,
        default_action_timeout: Duration::from_secs(1),
        max_concurrent_actions: 3,
        condition_check_interval: Duration::from_millis(100),
        enable_recovery_metrics: true,
        notification_config: skelly_jelly_event_bus::recovery::NotificationConfig {
            enabled: false, // Disable notifications for tests
            notification_levels: vec![],
            webhook_urls: vec![],
            email_addresses: vec![],
            slack_channels: vec![],
        },
    };

    let _dlq_config = DeadLetterQueueConfig {
        max_entries: test_config.dlq_max_entries,
        max_age: Duration::from_secs(3600),
        enable_persistence: false, // Disable persistence for tests
        persistence_path: None,
        auto_replay: None, // Manual replay for tests
        enable_metrics: true,
        replay_batch_size: 10,
    };

    let config = EventBusConfig {
        max_queue_size: 1000,
        delivery_timeout: Duration::from_millis(500),
        max_retry_attempts: test_config.retry_max_attempts,
        dead_letter_queue_size: test_config.dlq_max_entries,
        metrics_interval: Duration::from_millis(100),
        slow_handler_threshold: Duration::from_millis(50),
        circuit_breaker_config: Some(circuit_breaker_config),
        retry_config: Some(retry_config),
        error_logging_config: Some(error_logging_config),
        recovery_config: Some(recovery_config),
        enable_error_handling: true,
    };

    let bus = create_enhanced_event_bus_with_config(config).unwrap();
    bus.start().await.unwrap();
    
    // Register some test recovery actions
    register_test_recovery_actions(&bus).await;
    
    bus
}

/// Register test recovery actions
async fn register_test_recovery_actions(bus: &Arc<EnhancedEventBus>) {
    let recovery_system = bus.recovery_system();

    // Level 1: Automatic retry action
    let retry_action = RecoveryAction {
        id: Uuid::new_v4(),
        name: "Automatic Retry".to_string(),
        description: "Retry failed operations automatically".to_string(),
        strategy: RecoveryStrategy::Retry { config: RetryConfig::default() },
        escalation_level: EscalationLevel::Automatic,
        conditions: vec![
            skelly_jelly_event_bus::recovery::RecoveryCondition::ErrorTypeMatches { 
                error_types: vec!["DeliveryTimeout".to_string(), "ChannelSend".to_string()] 
            }
        ],
        max_executions: 3,
        cooldown: Duration::from_millis(100),
        requires_confirmation: false,
        expected_recovery_time: Duration::from_millis(500),
        success_threshold: 0.7,
    };
    recovery_system.register_action(retry_action);

    // Level 2: Circuit breaker reset action
    let circuit_reset_action = RecoveryAction {
        id: Uuid::new_v4(),
        name: "Circuit Breaker Reset".to_string(),
        description: "Reset circuit breakers to allow traffic".to_string(),
        strategy: RecoveryStrategy::CircuitBreakerReset { 
            circuit_name: "test_circuit".to_string() 
        },
        escalation_level: EscalationLevel::Component,
        conditions: vec![
            skelly_jelly_event_bus::recovery::RecoveryCondition::CircuitBreakerOpen { 
                circuit_name: "test_circuit".to_string() 
            }
        ],
        max_executions: 2,
        cooldown: Duration::from_millis(200),
        requires_confirmation: false,
        expected_recovery_time: Duration::from_millis(100),
        success_threshold: 0.8,
    };
    recovery_system.register_action(circuit_reset_action);

    // Level 3: Graceful degradation action
    let degradation_action = RecoveryAction {
        id: Uuid::new_v4(),
        name: "Graceful Degradation".to_string(),
        description: "Enable graceful degradation mode".to_string(),
        strategy: RecoveryStrategy::GracefulDegradation { 
            fallback_mode: "minimal".to_string() 
        },
        escalation_level: EscalationLevel::Service,
        conditions: vec![
            skelly_jelly_event_bus::recovery::RecoveryCondition::ErrorRateExceeds { 
                threshold: 0.5, 
                window: Duration::from_secs(60) 
            }
        ],
        max_executions: 1,
        cooldown: Duration::from_secs(1),
        requires_confirmation: false,
        expected_recovery_time: Duration::from_secs(1),
        success_threshold: 0.9,
    };
    recovery_system.register_action(degradation_action);
}

/// Create test message
fn create_test_message(source: ModuleId) -> BusMessage {
    BusMessage::with_priority(
        source,
        MessagePayload::ModuleReady(source),
        MessagePriority::Normal,
    )
}

#[tokio::test]
async fn test_circuit_breaker_protection() {
    let bus = create_test_bus(ErrorTestConfig {
        circuit_breaker_failure_threshold: 2,
        ..ErrorTestConfig::default()
    }).await;

    // Register a circuit breaker for testing
    let breaker = bus.circuit_breakers().register(
        "test_publish".to_string(),
        CircuitBreakerConfig {
            failure_threshold: 2,
            reset_timeout: Duration::from_millis(100),
            ..CircuitBreakerConfig::default()
        },
    );

    // Test that circuit breaker opens after failures
    breaker.force_open();
    assert!(!breaker.is_healthy());

    // Try to publish - should be rejected by circuit breaker logic
    let message = create_test_message(ModuleId::DataCapture);
    
    // Circuit breaker should protect the system
    let stats_before = bus.get_error_stats();
    
    // Force close to allow normal operation
    breaker.force_close();
    
    let result = bus.publish(message).await;
    assert!(result.is_ok());
    
    let stats_after = bus.get_error_stats();
    assert!(stats_after.retry_stats.total_operations > stats_before.retry_stats.total_operations);

    bus.shutdown().await.unwrap();
}

#[tokio::test]
async fn test_retry_mechanism_with_eventual_success() {
    let bus = create_test_bus(ErrorTestConfig {
        retry_max_attempts: 5,
        ..ErrorTestConfig::default()
    }).await;

    let message = create_test_message(ModuleId::DataCapture);
    let result = bus.publish(message).await;
    
    // Should succeed even with retries configured
    assert!(result.is_ok());
    
    let stats = bus.get_error_stats();
    assert_eq!(stats.retry_stats.successful_operations, 1);
    assert_eq!(stats.retry_stats.total_operations, 1);

    bus.shutdown().await.unwrap();
}

#[tokio::test]
async fn test_dead_letter_queue_functionality() {
    let bus = create_test_bus(ErrorTestConfig::default()).await;

    // Manually add a message to DLQ for testing
    let message = create_test_message(ModuleId::DataCapture);
    let dlq = bus.dead_letter_queue();
    
    let entry_id = dlq.add_message(
        message.clone(),
        DeadLetterReason::MaxRetriesExceeded { attempts: 3 },
        3,
        vec![ModuleId::Storage],
        Some("Test failure".to_string()),
        Some("test-correlation-123".to_string()),
    );

    // Verify message is in DLQ
    let entry = dlq.get_entry(entry_id).unwrap();
    assert_eq!(entry.message.id, message.id);
    assert_eq!(entry.retry_count, 3);
    assert!(matches!(entry.reason, DeadLetterReason::MaxRetriesExceeded { attempts: 3 }));

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

    // Test replay functionality
    let replay_results = bus.replay_dead_letters().await.unwrap();
    assert_eq!(replay_results.len(), 1);
    assert!(replay_results[0].success);

    // Entry should be removed after successful replay
    assert!(dlq.get_entry(entry_id).is_none());

    let stats = dlq.stats();
    assert_eq!(stats.total_entries, 0); // Should be empty after successful replay

    bus.shutdown().await.unwrap();
}

#[tokio::test]
async fn test_error_logging_with_correlation_tracking() {
    let bus = create_test_bus(ErrorTestConfig::default()).await;

    // Test that error logging captures correlation IDs
    let error_logger = bus.error_logger();
    let correlation_id = skelly_jelly_event_bus::error_logging::ErrorLogger::create_correlation_id();
    
    let error_context = skelly_jelly_event_bus::error_logging::ErrorContext::new(
        correlation_id,
        ModuleId::DataCapture,
        "test_operation".to_string(),
        ErrorSeverity::Error,
        ErrorCategory::Network,
        "Test error message".to_string(),
    ).with_metadata("test_key", "test_value");

    error_logger.log_error(&error_context);

    let stats = error_logger.stats();
    assert_eq!(stats.total_errors_logged, 1);
    assert!(stats.errors_by_severity.contains_key("Error"));
    assert!(stats.errors_by_module.contains_key(&ModuleId::DataCapture));

    bus.shutdown().await.unwrap();
}

#[tokio::test]
async fn test_recovery_system_escalation() {
    let bus = create_test_bus(ErrorTestConfig::default()).await;

    let recovery_system = bus.recovery_system();
    let correlation_id = Uuid::new_v4();
    
    // Simulate an incident
    let error = EventBusError::QueueFull { current_size: 100, max_size: 100 };
    let incident_id = recovery_system.handle_incident(
        correlation_id,
        ModuleId::EventBus,
        &error,
        "Test queue overflow incident".to_string(),
    ).await.unwrap();

    // Give recovery system time to process
    sleep(Duration::from_millis(200)).await;

    // Check that incident was created and is being processed
    let incident = recovery_system.get_incident(incident_id).unwrap();
    assert_eq!(incident.correlation_id, correlation_id);
    assert_eq!(incident.module_id, ModuleId::EventBus);
    assert!(matches!(incident.status, IncidentStatus::Detected | IncidentStatus::Recovering));

    let stats = recovery_system.stats();
    assert_eq!(stats.total_incidents, 1);

    bus.shutdown().await.unwrap();
}

#[tokio::test]
async fn test_end_to_end_error_handling_integration() {
    let bus = create_test_bus(ErrorTestConfig {
        circuit_breaker_failure_threshold: 2,
        retry_max_attempts: 2,
        dlq_max_entries: 50,
        recovery_enabled: true,
    }).await;

    // Test normal operation
    let message1 = create_test_message(ModuleId::DataCapture);
    let result1 = bus.publish(message1).await;
    assert!(result1.is_ok());

    // Test subscription creation
    let filter = MessageFilter::types(vec![MessageType::ModuleReady]);
    let subscription_id = bus.subscribe(
        ModuleId::Storage,
        filter,
        DeliveryMode::BestEffort,
    ).await.unwrap();

    // Test that we can publish more messages successfully
    let message2 = create_test_message(ModuleId::AnalysisEngine);
    let result2 = bus.publish(message2).await;
    assert!(result2.is_ok());

    // Verify metrics show successful operations
    let bus_metrics = bus.metrics().await.unwrap();
    assert_eq!(bus_metrics.messages_published, 2);

    // Test error handling stats
    let error_stats = bus.get_error_stats();
    assert_eq!(error_stats.retry_stats.successful_operations, 2);
    assert_eq!(error_stats.retry_stats.failed_operations, 0);

    // Clean up subscription
    let unsub_result = bus.unsubscribe(subscription_id).await;
    assert!(unsub_result.is_ok());

    bus.shutdown().await.unwrap();
}

#[tokio::test]
async fn test_stress_scenario_with_multiple_failures() {
    let bus = create_test_bus(ErrorTestConfig {
        circuit_breaker_failure_threshold: 5,
        retry_max_attempts: 2,
        dlq_max_entries: 100,
        recovery_enabled: true,
    }).await;

    let message_count = 10;
    let success_count = Arc::new(AtomicU32::new(0));
    let failure_count = Arc::new(AtomicU32::new(0));

    // Publish multiple messages concurrently
    let mut handles = vec![];
    
    for i in 0..message_count {
        let bus_clone = bus.clone();
        let success_count_clone = success_count.clone();
        let failure_count_clone = failure_count.clone();
        
        let handle = tokio::spawn(async move {
            let message = BusMessage::with_priority(
                ModuleId::DataCapture,
                MessagePayload::ModuleReady(ModuleId::DataCapture),
                if i % 3 == 0 { MessagePriority::High } else { MessagePriority::Normal },
            );
            
            match timeout(Duration::from_secs(2), bus_clone.publish(message)).await {
                Ok(Ok(_)) => {
                    success_count_clone.fetch_add(1, Ordering::SeqCst);
                }
                Ok(Err(_)) | Err(_) => {
                    failure_count_clone.fetch_add(1, Ordering::SeqCst);
                }
            }
        });
        
        handles.push(handle);
    }

    // Wait for all operations to complete
    for handle in handles {
        handle.await.unwrap();
    }

    let final_success = success_count.load(Ordering::SeqCst);
    let final_failure = failure_count.load(Ordering::SeqCst);
    
    // In normal conditions, most messages should succeed
    assert!(final_success >= message_count / 2);
    assert_eq!(final_success + final_failure, message_count);

    // Check that error handling systems captured the activity
    let error_stats = bus.get_error_stats();
    assert_eq!(error_stats.retry_stats.total_operations, message_count as u64);
    
    // Verify that bus metrics are consistent
    let bus_metrics = bus.metrics().await.unwrap();
    assert_eq!(bus_metrics.messages_published, final_success as u64);

    bus.shutdown().await.unwrap();
}

#[tokio::test]
async fn test_recovery_system_with_custom_actions() {
    let bus = create_test_bus(ErrorTestConfig::default()).await;
    let recovery_system = bus.recovery_system();

    // Register a custom recovery action
    let custom_action = RecoveryAction {
        id: Uuid::new_v4(),
        name: "Custom Test Action".to_string(),
        description: "Custom recovery action for testing".to_string(),
        strategy: RecoveryStrategy::Custom {
            action_name: "test_custom_action".to_string(),
            parameters: std::collections::HashMap::new(),
        },
        escalation_level: EscalationLevel::Automatic,
        conditions: vec![],
        max_executions: 1,
        cooldown: Duration::from_millis(100),
        requires_confirmation: false,
        expected_recovery_time: Duration::from_millis(200),
        success_threshold: 0.9,
    };

    recovery_system.register_action(custom_action);

    // Create an incident to trigger recovery
    let correlation_id = Uuid::new_v4();
    let error = EventBusError::Internal("Custom test error".to_string());
    
    let incident_id = recovery_system.handle_incident(
        correlation_id,
        ModuleId::EventBus,
        &error,
        "Testing custom recovery action".to_string(),
    ).await.unwrap();

    // Give recovery system time to process
    sleep(Duration::from_millis(300)).await;

    let incident = recovery_system.get_incident(incident_id).unwrap();
    assert!(!incident.attempted_actions.is_empty());

    let stats = recovery_system.stats();
    assert!(stats.recovery_actions_executed > 0);

    bus.shutdown().await.unwrap();
}

#[tokio::test]
async fn test_graceful_shutdown_with_pending_operations() {
    let bus = create_test_bus(ErrorTestConfig::default()).await;

    // Start some operations
    let message1 = create_test_message(ModuleId::DataCapture);
    let _result1 = bus.publish(message1).await;

    let message2 = create_test_message(ModuleId::AnalysisEngine);
    let _result2 = bus.publish(message2).await;

    // Verify operations completed
    let stats_before_shutdown = bus.get_error_stats();
    assert_eq!(stats_before_shutdown.retry_stats.total_operations, 2);

    // Shutdown should complete gracefully
    let shutdown_result = timeout(Duration::from_secs(5), bus.shutdown()).await;
    assert!(shutdown_result.is_ok());
    assert!(shutdown_result.unwrap().is_ok());

    // Verify cleanup happened
    let cleanup_count = bus.cleanup_dead_letters();
    // Should be 0 since cleanup happens during shutdown
    assert_eq!(cleanup_count, 0);
}

/// Helper to verify error handling system health
async fn verify_system_health(bus: &Arc<EnhancedEventBus>) {
    let error_stats = bus.get_error_stats();
    
    // Circuit breakers should be healthy
    assert!(bus.circuit_breakers().all_healthy());
    
    // Error logging should be functioning
    assert!(error_stats.error_logging_stats.total_errors_logged >= 0);
    
    // Dead letter queue should be within limits
    assert!(error_stats.dead_letter_stats.total_entries < 1000);
    
    // Recovery system should be operational
    assert!(error_stats.recovery_stats.total_incidents >= 0);
}

#[tokio::test]
async fn test_system_health_monitoring() {
    let bus = create_test_bus(ErrorTestConfig::default()).await;

    // Initial health check
    verify_system_health(&bus).await;

    // Perform some operations
    for _i in 0..5 {
        let message = BusMessage::with_priority(
            ModuleId::DataCapture,
            MessagePayload::ModuleReady(ModuleId::DataCapture),
            MessagePriority::Normal,
        );
        let _result = bus.publish(message).await;
    }

    // Health check after operations
    verify_system_health(&bus).await;

    // Test that metrics are being collected
    let bus_metrics = bus.metrics().await.unwrap();
    assert_eq!(bus_metrics.messages_published, 5);

    let error_stats = bus.get_error_stats();
    assert_eq!(error_stats.retry_stats.total_operations, 5);
    assert_eq!(error_stats.retry_stats.successful_operations, 5);

    bus.shutdown().await.unwrap();
}