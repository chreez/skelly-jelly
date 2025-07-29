//! Enhanced Error Logging with Structured Data and Correlation IDs
//!
//! Provides comprehensive error logging capabilities with structured data,
//! correlation tracking, and contextual information for distributed debugging.

use std::collections::HashMap;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use tracing::{error, warn, info, debug, span, Level, Span};
use uuid::Uuid;

use crate::{ModuleId, MessageId, EventBusError};

/// Unique identifier for correlating related operations
pub type CorrelationId = Uuid;

/// Unique identifier for tracing request flows
pub type TraceId = Uuid;

/// Severity levels for errors
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum ErrorSeverity {
    /// Informational messages for debugging
    Debug,
    /// Expected errors that don't affect system operation
    Info,
    /// Warnings about potential issues
    Warning,
    /// Errors that affect specific operations but system continues
    Error,
    /// Critical errors that may affect system stability
    Critical,
    /// Fatal errors that require immediate attention
    Fatal,
}

impl ErrorSeverity {
    /// Convert to tracing Level
    pub fn to_tracing_level(&self) -> Level {
        match self {
            ErrorSeverity::Debug => Level::DEBUG,
            ErrorSeverity::Info => Level::INFO,
            ErrorSeverity::Warning => Level::WARN,
            ErrorSeverity::Error => Level::ERROR,
            ErrorSeverity::Critical => Level::ERROR,
            ErrorSeverity::Fatal => Level::ERROR,
        }
    }
}

/// Categories for error classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ErrorCategory {
    /// Network communication errors
    Network,
    /// Data serialization/deserialization errors
    Serialization,
    /// Configuration and setup errors
    Configuration,
    /// Resource exhaustion (memory, disk, etc.)
    Resource,
    /// Authentication and authorization errors
    Security,
    /// Business logic validation errors
    Validation,
    /// External service integration errors
    Integration,
    /// Performance and timeout related errors
    Performance,
    /// Unknown or uncategorized errors
    Unknown,
}

/// Structured error context for logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
    /// Unique correlation ID for tracking related operations
    pub correlation_id: CorrelationId,
    
    /// Trace ID for request flow tracking
    pub trace_id: TraceId,
    
    /// Module where the error occurred
    pub module_id: ModuleId,
    
    /// Operation being performed when error occurred
    pub operation: String,
    
    /// Error severity level
    pub severity: ErrorSeverity,
    
    /// Error category for classification
    pub category: ErrorCategory,
    
    /// The actual error message
    pub error_message: String,
    
    /// Optional error code for programmatic handling
    pub error_code: Option<String>,
    
    /// Stack trace if available
    pub stack_trace: Option<String>,
    
    /// When the error occurred
    pub timestamp: SystemTime,
    
    /// Duration of the operation that failed
    pub operation_duration: Option<Duration>,
    
    /// Additional structured data relevant to the error
    pub metadata: HashMap<String, serde_json::Value>,
    
    /// Related message ID if applicable
    pub message_id: Option<MessageId>,
    
    /// Related subscription ID if applicable
    pub subscription_id: Option<Uuid>,
    
    /// User or system context
    pub user_context: Option<String>,
    
    /// Environment information
    pub environment: Option<String>,
    
    /// Version information
    pub version: Option<String>,
}

impl ErrorContext {
    /// Create a new error context with required fields
    pub fn new(
        correlation_id: CorrelationId,
        module_id: ModuleId,
        operation: String,
        severity: ErrorSeverity,
        category: ErrorCategory,
        error_message: String,
    ) -> Self {
        Self {
            correlation_id,
            trace_id: Uuid::new_v4(),
            module_id,
            operation,
            severity,
            category,
            error_message,
            error_code: None,
            stack_trace: None,
            timestamp: SystemTime::now(),
            operation_duration: None,
            metadata: HashMap::new(),
            message_id: None,
            subscription_id: None,
            user_context: None,
            environment: None,
            version: None,
        }
    }

    /// Add metadata to the error context
    pub fn with_metadata<T: Serialize>(mut self, key: &str, value: T) -> Self {
        if let Ok(json_value) = serde_json::to_value(value) {
            self.metadata.insert(key.to_string(), json_value);
        }
        self
    }

    /// Add message ID to the context
    pub fn with_message_id(mut self, message_id: MessageId) -> Self {
        self.message_id = Some(message_id);
        self
    }

    /// Add operation duration to the context
    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.operation_duration = Some(duration);
        self
    }

    /// Add error code to the context
    pub fn with_error_code(mut self, code: String) -> Self {
        self.error_code = Some(code);
        self
    }

    /// Add stack trace to the context
    pub fn with_stack_trace(mut self, stack_trace: String) -> Self {
        self.stack_trace = Some(stack_trace);
        self
    }
}

/// Enhanced error logger with structured logging capabilities
pub struct ErrorLogger {
    /// Configuration for the logger
    config: ErrorLoggerConfig,
    
    /// Statistics about logged errors
    stats: Arc<parking_lot::RwLock<ErrorStats>>,
}

/// Configuration for error logging behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorLoggerConfig {
    /// Minimum severity level to log
    pub min_severity: ErrorSeverity,
    
    /// Whether to include stack traces in logs
    pub include_stack_traces: bool,
    
    /// Whether to include metadata in logs
    pub include_metadata: bool,
    
    /// Maximum length for error messages
    pub max_message_length: usize,
    
    /// Whether to enable performance metrics
    pub enable_metrics: bool,
    
    /// Format for log output
    pub log_format: LogFormat,
    
    /// Whether to sanitize sensitive data
    pub sanitize_sensitive_data: bool,
    
    /// List of sensitive field names to sanitize
    pub sensitive_fields: Vec<String>,
}

/// Supported log formats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogFormat {
    /// Human-readable format
    Human,
    /// JSON format for structured logging
    Json,
    /// Key-value pairs format
    KeyValue,
}

impl Default for ErrorLoggerConfig {
    fn default() -> Self {
        Self {
            min_severity: ErrorSeverity::Warning,
            include_stack_traces: true,
            include_metadata: true,
            max_message_length: 1000,
            enable_metrics: true,
            log_format: LogFormat::Json,
            sanitize_sensitive_data: true,
            sensitive_fields: vec![
                "password".to_string(),
                "token".to_string(),
                "key".to_string(),
                "secret".to_string(),
                "auth".to_string(),
            ],
        }
    }
}

/// Statistics about error logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorStats {
    pub total_errors_logged: u64,
    pub errors_by_severity: HashMap<String, u64>,
    pub errors_by_category: HashMap<String, u64>,
    pub errors_by_module: HashMap<ModuleId, u64>,
    pub average_error_rate_per_minute: f64,
    pub last_error_timestamp: Option<SystemTime>,
    pub top_error_messages: Vec<(String, u64)>,
}

impl Default for ErrorStats {
    fn default() -> Self {
        Self {
            total_errors_logged: 0,
            errors_by_severity: HashMap::new(),
            errors_by_category: HashMap::new(),
            errors_by_module: HashMap::new(),
            average_error_rate_per_minute: 0.0,
            last_error_timestamp: None,
            top_error_messages: Vec::new(),
        }
    }
}

impl ErrorLogger {
    /// Create a new error logger with the given configuration
    pub fn new(config: ErrorLoggerConfig) -> Self {
        Self {
            config,
            stats: Arc::new(parking_lot::RwLock::new(ErrorStats::default())),
        }
    }

    /// Log an error with structured context
    pub fn log_error(&self, context: &ErrorContext) {
        // Check if we should log this severity level
        if context.severity < self.config.min_severity {
            return;
        }

        // Update statistics
        if self.config.enable_metrics {
            self.update_stats(context);
        }

        // Create a tracing span for context
        let span = span!(
            Level::ERROR, // Use a constant level instead of dynamic
            "error_event",
            correlation_id = %context.correlation_id,
            trace_id = %context.trace_id,
            module_id = ?context.module_id,
            operation = %context.operation,
            error_category = ?context.category,
            message_id = ?context.message_id,
        );

        let _guard = span.enter();

        // Format and log the error based on configuration
        match self.config.log_format {
            LogFormat::Json => self.log_json(context),
            LogFormat::Human => self.log_human(context),
            LogFormat::KeyValue => self.log_key_value(context),
        }
    }

    /// Log an error from EventBusError with automatic context creation
    pub fn log_event_bus_error(
        &self,
        error: &EventBusError,
        correlation_id: CorrelationId,
        module_id: ModuleId,
        operation: &str,
    ) {
        let (severity, category) = self.classify_event_bus_error(error);
        
        let context = ErrorContext::new(
            correlation_id,
            module_id,
            operation.to_string(),
            severity,
            category,
            error.to_string(),
        );

        self.log_error(&context);
    }

    /// Create a correlation ID for tracking related operations
    pub fn create_correlation_id() -> CorrelationId {
        Uuid::new_v4()
    }

    /// Create an operation context for tracking timing
    pub fn start_operation(&self, correlation_id: CorrelationId, operation: &str) -> OperationContext {
        OperationContext {
            correlation_id,
            operation: operation.to_string(),
            start_time: Instant::now(),
            logger: self,
        }
    }

    /// Get current error statistics
    pub fn stats(&self) -> ErrorStats {
        self.stats.read().clone()
    }

    /// Reset error statistics
    pub fn reset_stats(&self) {
        let mut stats = self.stats.write();
        *stats = ErrorStats::default();
    }

    /// Classify EventBusError for severity and category
    fn classify_event_bus_error(&self, error: &EventBusError) -> (ErrorSeverity, ErrorCategory) {
        match error {
            EventBusError::SubscriberUnavailable { .. } => (ErrorSeverity::Warning, ErrorCategory::Network),
            EventBusError::MessageRejected { .. } => (ErrorSeverity::Error, ErrorCategory::Validation),
            EventBusError::DeliveryTimeout { .. } => (ErrorSeverity::Warning, ErrorCategory::Performance),
            EventBusError::QueueFull { .. } => (ErrorSeverity::Critical, ErrorCategory::Resource),
            EventBusError::SubscriptionNotFound { .. } => (ErrorSeverity::Error, ErrorCategory::Validation),
            EventBusError::InvalidFilter { .. } => (ErrorSeverity::Error, ErrorCategory::Validation),
            EventBusError::BusShuttingDown => (ErrorSeverity::Info, ErrorCategory::Configuration),
            EventBusError::ChannelSend(_) => (ErrorSeverity::Error, ErrorCategory::Network),
            EventBusError::ChannelReceive(_) => (ErrorSeverity::Error, ErrorCategory::Network),
            EventBusError::Serialization(_) => (ErrorSeverity::Error, ErrorCategory::Serialization),
            EventBusError::Configuration(_) => (ErrorSeverity::Error, ErrorCategory::Configuration),
            EventBusError::ModuleAlreadyRegistered { .. } => (ErrorSeverity::Warning, ErrorCategory::Validation),
            EventBusError::ModuleNotFound { .. } => (ErrorSeverity::Error, ErrorCategory::Validation),
            EventBusError::InvalidHealthCheckResponse => (ErrorSeverity::Warning, ErrorCategory::Integration),
            EventBusError::Internal(_) => (ErrorSeverity::Critical, ErrorCategory::Unknown),
            EventBusError::Io(_) => (ErrorSeverity::Error, ErrorCategory::Resource),
        }
    }

    /// Log error in JSON format
    fn log_json(&self, context: &ErrorContext) {
        let mut log_data = serde_json::json!({
            "timestamp": context.timestamp.duration_since(UNIX_EPOCH).unwrap().as_secs(),
            "correlation_id": context.correlation_id,
            "trace_id": context.trace_id,
            "module_id": context.module_id,
            "operation": context.operation,
            "severity": context.severity,
            "category": context.category,
            "message": self.sanitize_message(&context.error_message),
        });

        if let Some(ref error_code) = context.error_code {
            log_data["error_code"] = serde_json::Value::String(error_code.clone());
        }

        if let Some(ref message_id) = context.message_id {
            log_data["message_id"] = serde_json::Value::String(message_id.to_string());
        }

        if let Some(duration) = context.operation_duration {
            log_data["duration_ms"] = serde_json::Value::Number(
                serde_json::Number::from(duration.as_millis() as u64)
            );
        }

        if self.config.include_metadata && !context.metadata.is_empty() {
            log_data["metadata"] = serde_json::Value::Object(
                context.metadata.iter()
                    .map(|(k, v)| (k.clone(), self.sanitize_value(v)))
                    .collect()
            );
        }

        if self.config.include_stack_traces {
            if let Some(ref stack_trace) = context.stack_trace {
                log_data["stack_trace"] = serde_json::Value::String(stack_trace.clone());
            }
        }

        match context.severity {
            ErrorSeverity::Debug => debug!("{}", log_data),
            ErrorSeverity::Info => info!("{}", log_data),
            ErrorSeverity::Warning => warn!("{}", log_data),
            ErrorSeverity::Error | ErrorSeverity::Critical | ErrorSeverity::Fatal => {
                error!("{}", log_data)
            }
        }
    }

    /// Log error in human-readable format
    fn log_human(&self, context: &ErrorContext) {
        let duration_str = context.operation_duration
            .map(|d| format!(" ({}ms)", d.as_millis()))
            .unwrap_or_default();

        let message = format!(
            "[{:?}] {} in {:?}: {}{} [correlation_id: {}]",
            context.severity,
            context.operation,
            context.module_id,
            self.sanitize_message(&context.error_message),
            duration_str,
            context.correlation_id
        );

        match context.severity {
            ErrorSeverity::Debug => debug!("{}", message),
            ErrorSeverity::Info => info!("{}", message),
            ErrorSeverity::Warning => warn!("{}", message),
            ErrorSeverity::Error | ErrorSeverity::Critical | ErrorSeverity::Fatal => {
                error!("{}", message)
            }
        }
    }

    /// Log error in key-value format
    fn log_key_value(&self, context: &ErrorContext) {
        let mut fields = vec![
            ("correlation_id", context.correlation_id.to_string()),
            ("module_id", format!("{:?}", context.module_id)),
            ("operation", context.operation.clone()),
            ("severity", format!("{:?}", context.severity)),
            ("category", format!("{:?}", context.category)),
            ("message", self.sanitize_message(&context.error_message)),
        ];

        if let Some(duration) = context.operation_duration {
            fields.push(("duration_ms", duration.as_millis().to_string()));
        }

        let log_str = fields.iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join(" ");

        match context.severity {
            ErrorSeverity::Debug => debug!("{}", log_str),
            ErrorSeverity::Info => info!("{}", log_str),
            ErrorSeverity::Warning => warn!("{}", log_str),
            ErrorSeverity::Error | ErrorSeverity::Critical | ErrorSeverity::Fatal => {
                error!("{}", log_str)
            }
        }
    }

    /// Sanitize sensitive data from messages
    fn sanitize_message(&self, message: &str) -> String {
        if !self.config.sanitize_sensitive_data {
            return message.to_string();
        }

        let mut sanitized = message.to_string();
        for sensitive_field in &self.config.sensitive_fields {
            // Simple pattern to replace sensitive data
            let pattern = format!("{}=[^\\s,}}]+", sensitive_field);
            let replacement = format!("{}=***", sensitive_field);
            sanitized = regex::Regex::new(&pattern)
                .unwrap_or_else(|_| regex::Regex::new("").unwrap())
                .replace_all(&sanitized, replacement.as_str())
                .to_string();
        }

        // Truncate if too long
        if sanitized.len() > self.config.max_message_length {
            sanitized.truncate(self.config.max_message_length - 3);
            sanitized.push_str("...");
        }

        sanitized
    }

    /// Sanitize sensitive data from JSON values
    fn sanitize_value(&self, value: &serde_json::Value) -> serde_json::Value {
        if !self.config.sanitize_sensitive_data {
            return value.clone();
        }

        match value {
            serde_json::Value::Object(obj) => {
                let mut sanitized = serde_json::Map::new();
                for (key, value) in obj {
                    if self.config.sensitive_fields.iter().any(|field| key.contains(field)) {
                        sanitized.insert(key.clone(), serde_json::Value::String("***".to_string()));
                    } else {
                        sanitized.insert(key.clone(), self.sanitize_value(value));
                    }
                }
                serde_json::Value::Object(sanitized)
            }
            serde_json::Value::Array(arr) => {
                serde_json::Value::Array(arr.iter().map(|v| self.sanitize_value(v)).collect())
            }
            _ => value.clone(),
        }
    }

    /// Update error statistics
    fn update_stats(&self, context: &ErrorContext) {
        let mut stats = self.stats.write();
        
        stats.total_errors_logged += 1;
        stats.last_error_timestamp = Some(context.timestamp);

        // Update severity counts
        let severity_key = format!("{:?}", context.severity);
        *stats.errors_by_severity.entry(severity_key).or_insert(0) += 1;

        // Update category counts
        let category_key = format!("{:?}", context.category);
        *stats.errors_by_category.entry(category_key).or_insert(0) += 1;

        // Update module counts
        *stats.errors_by_module.entry(context.module_id).or_insert(0) += 1;

        // Update top error messages (simple implementation)
        let message_key = context.error_message.clone();
        if let Some(pos) = stats.top_error_messages.iter().position(|(msg, _)| msg == &message_key) {
            stats.top_error_messages[pos].1 += 1;
        } else if stats.top_error_messages.len() < 10 {
            stats.top_error_messages.push((message_key, 1));
        }

        // Sort top messages by count
        stats.top_error_messages.sort_by(|a, b| b.1.cmp(&a.1));
    }
}

/// Context for tracking operation timing and logging
pub struct OperationContext<'a> {
    correlation_id: CorrelationId,
    operation: String,
    start_time: Instant,
    logger: &'a ErrorLogger,
}

impl<'a> OperationContext<'a> {
    /// Complete the operation successfully
    pub fn complete(self) {
        let duration = self.start_time.elapsed();
        debug!("Operation '{}' completed in {:?} [correlation_id: {}]", 
               self.operation, duration, self.correlation_id);
    }

    /// Complete the operation with an error
    pub fn complete_with_error(
        self,
        module_id: ModuleId,
        severity: ErrorSeverity,
        category: ErrorCategory,
        error_message: String,
    ) {
        let duration = self.start_time.elapsed();
        
        let context = ErrorContext::new(
            self.correlation_id,
            module_id,
            self.operation,
            severity,
            category,
            error_message,
        ).with_duration(duration);

        self.logger.log_error(&context);
    }

    /// Get the correlation ID for this operation
    pub fn correlation_id(&self) -> CorrelationId {
        self.correlation_id
    }
}

/// Create a default error logger
pub fn create_error_logger() -> ErrorLogger {
    ErrorLogger::new(ErrorLoggerConfig::default())
}

/// Create an error logger with custom configuration
pub fn create_error_logger_with_config(config: ErrorLoggerConfig) -> ErrorLogger {
    ErrorLogger::new(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_error_context_creation() {
        let correlation_id = Uuid::new_v4();
        let context = ErrorContext::new(
            correlation_id,
            ModuleId::EventBus,
            "test_operation".to_string(),
            ErrorSeverity::Error,
            ErrorCategory::Network,
            "Test error message".to_string(),
        )
        .with_metadata("key1", "value1")
        .with_error_code("E001".to_string())
        .with_duration(Duration::from_millis(100));

        assert_eq!(context.correlation_id, correlation_id);
        assert_eq!(context.operation, "test_operation");
        assert_eq!(context.severity, ErrorSeverity::Error);
        assert_eq!(context.category, ErrorCategory::Network);
        assert_eq!(context.error_code, Some("E001".to_string()));
        assert_eq!(context.operation_duration, Some(Duration::from_millis(100)));
        assert!(context.metadata.contains_key("key1"));
    }

    #[test]
    fn test_error_logger_stats() {
        let logger = create_error_logger();
        
        let context1 = ErrorContext::new(
            Uuid::new_v4(),
            ModuleId::EventBus,
            "operation1".to_string(),
            ErrorSeverity::Error,
            ErrorCategory::Network,
            "Error 1".to_string(),
        );

        let context2 = ErrorContext::new(
            Uuid::new_v4(),
            ModuleId::Storage,
            "operation2".to_string(),
            ErrorSeverity::Warning,
            ErrorCategory::Performance,
            "Error 2".to_string(),
        );

        logger.log_error(&context1);
        logger.log_error(&context2);

        let stats = logger.stats();
        assert_eq!(stats.total_errors_logged, 2);
        assert!(stats.errors_by_severity.contains_key("Error"));
        assert!(stats.errors_by_severity.contains_key("Warning"));
        assert!(stats.errors_by_module.contains_key(&ModuleId::EventBus));
        assert!(stats.errors_by_module.contains_key(&ModuleId::Storage));
    }

    #[test]
    fn test_operation_context() {
        let logger = create_error_logger();
        let correlation_id = Uuid::new_v4();
        
        let context = logger.start_operation(correlation_id, "test_operation");
        assert_eq!(context.correlation_id(), correlation_id);
        
        // Simulate some work
        thread::sleep(Duration::from_millis(10));
        
        context.complete_with_error(
            ModuleId::EventBus,
            ErrorSeverity::Error,
            ErrorCategory::Network,
            "Operation failed".to_string(),
        );

        let stats = logger.stats();
        assert_eq!(stats.total_errors_logged, 1);
    }

    #[test]
    fn test_event_bus_error_classification() {
        let logger = create_error_logger();
        
        let error = EventBusError::QueueFull { current_size: 100, max_size: 100 };
        let (severity, category) = logger.classify_event_bus_error(&error);
        
        assert_eq!(severity, ErrorSeverity::Critical);
        assert_eq!(category, ErrorCategory::Resource);
    }

    #[test]
    fn test_sensitive_data_sanitization() {
        let config = ErrorLoggerConfig {
            sanitize_sensitive_data: true,
            sensitive_fields: vec!["password".to_string(), "token".to_string()],
            ..ErrorLoggerConfig::default()
        };
        
        let logger = ErrorLogger::new(config);
        
        let message = "Login failed: password=secret123, token=abc123";
        let sanitized = logger.sanitize_message(message);
        
        assert!(sanitized.contains("password=***"));
        assert!(sanitized.contains("token=***"));
        assert!(!sanitized.contains("secret123"));
        assert!(!sanitized.contains("abc123"));
    }

    #[test]
    fn test_message_truncation() {
        let config = ErrorLoggerConfig {
            max_message_length: 20,
            ..ErrorLoggerConfig::default()
        };
        
        let logger = ErrorLogger::new(config);
        
        let long_message = "This is a very long error message that should be truncated";
        let truncated = logger.sanitize_message(long_message);
        
        assert_eq!(truncated.len(), 20);
        assert!(truncated.ends_with("..."));
    }
}