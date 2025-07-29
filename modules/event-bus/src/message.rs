//! Message types and definitions for the event bus

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};
use uuid::Uuid;

/// Unique identifier for a module in the system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModuleId {
    DataCapture,
    Storage,
    AnalysisEngine,
    Gamification,
    AiIntegration,
    CuteFigurine,
    Orchestrator,
    EventBus,
}

impl std::fmt::Display for ModuleId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModuleId::DataCapture => write!(f, "data-capture"),
            ModuleId::Storage => write!(f, "storage"),
            ModuleId::AnalysisEngine => write!(f, "analysis-engine"),
            ModuleId::Gamification => write!(f, "gamification"),
            ModuleId::AiIntegration => write!(f, "ai-integration"),
            ModuleId::CuteFigurine => write!(f, "cute-figurine"),
            ModuleId::Orchestrator => write!(f, "orchestrator"),
            ModuleId::EventBus => write!(f, "event-bus"),
        }
    }
}

/// Priority levels for message processing
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum MessagePriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

impl Default for MessagePriority {
    fn default() -> Self {
        MessagePriority::Normal
    }
}

/// Core message envelope that wraps all inter-module communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusMessage {
    /// Unique identifier for this message
    pub id: Uuid,
    
    /// When the message was created
    pub timestamp: SystemTime,
    
    /// Which module sent this message
    pub source: ModuleId,
    
    /// The actual message payload
    pub payload: MessagePayload,
    
    /// Optional correlation ID for request-response patterns
    pub correlation_id: Option<Uuid>,
    
    /// Priority for message processing
    pub priority: MessagePriority,
}

impl BusMessage {
    /// Create a new message with the specified payload
    pub fn new(source: ModuleId, payload: MessagePayload) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: SystemTime::now(),
            source,
            payload,
            correlation_id: None,
            priority: MessagePriority::default(),
        }
    }

    /// Create a new message with specified priority
    pub fn with_priority(source: ModuleId, payload: MessagePayload, priority: MessagePriority) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: SystemTime::now(),
            source,
            payload,
            correlation_id: None,
            priority,
        }
    }

    /// Create a correlated response message
    pub fn reply_to(&self, source: ModuleId, payload: MessagePayload) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: SystemTime::now(),
            source,
            payload,
            correlation_id: Some(self.id),
            priority: self.priority,
        }
    }

    /// Get the message type from the payload
    pub fn message_type(&self) -> MessageType {
        self.payload.message_type()
    }
}

/// All possible message types in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessagePayload {
    // From Data Capture
    RawEvent(RawEvent),
    
    // From Storage
    EventBatch(EventBatch),
    StorageStatus(StorageMetrics),
    
    // From Analysis Engine
    AnalysisComplete(AnalysisWindow),
    StateChange(StateClassification),
    
    // From Gamification
    InterventionRequest(InterventionRequest),
    RewardEvent(RewardEvent),
    
    // From AI Integration
    InterventionResponse(InterventionResponse),
    AnimationCommand(AnimationCommand),
    
    // From Orchestrator
    HealthCheck(HealthCheckRequest),
    ConfigUpdate(ConfigUpdate),
    
    // System messages
    Shutdown(ShutdownRequest),
    ModuleReady(ModuleId),
    Error(ErrorReport),
}

impl MessagePayload {
    /// Get the message type for routing purposes
    pub fn message_type(&self) -> MessageType {
        match self {
            MessagePayload::RawEvent(_) => MessageType::RawEvent,
            MessagePayload::EventBatch(_) => MessageType::EventBatch,
            MessagePayload::StorageStatus(_) => MessageType::StorageStatus,
            MessagePayload::AnalysisComplete(_) => MessageType::AnalysisComplete,
            MessagePayload::StateChange(_) => MessageType::StateChange,
            MessagePayload::InterventionRequest(_) => MessageType::InterventionRequest,
            MessagePayload::RewardEvent(_) => MessageType::RewardEvent,
            MessagePayload::InterventionResponse(_) => MessageType::InterventionResponse,
            MessagePayload::AnimationCommand(_) => MessageType::AnimationCommand,
            MessagePayload::HealthCheck(_) => MessageType::HealthCheck,
            MessagePayload::ConfigUpdate(_) => MessageType::ConfigUpdate,
            MessagePayload::Shutdown(_) => MessageType::Shutdown,
            MessagePayload::ModuleReady(_) => MessageType::ModuleReady,
            MessagePayload::Error(_) => MessageType::Error,
        }
    }
}

/// Message type enumeration for filtering and routing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MessageType {
    RawEvent,
    EventBatch,
    StorageStatus,
    AnalysisComplete,
    StateChange,
    InterventionRequest,
    RewardEvent,
    InterventionResponse,
    AnimationCommand,
    HealthCheck,
    ConfigUpdate,
    Shutdown,
    ModuleReady,
    Error,
}

// Message payload data structures - simplified for initial implementation
// These will be expanded based on actual module requirements

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawEvent {
    pub event_type: String,
    pub data: serde_json::Value,
    pub window_title: Option<String>,
    pub timestamp: DateTime<Utc>,
}

impl RawEvent {
    /// Create a keystroke event
    pub fn keystroke(key: String, duration: Duration, modifiers: Vec<String>) -> Self {
        Self {
            event_type: "keystroke".to_string(),
            data: serde_json::json!({
                "key": key,
                "duration_ms": duration.as_millis(),
                "modifiers": modifiers
            }),
            window_title: None,
            timestamp: Utc::now(),
        }
    }

    /// Create a mouse move event
    pub fn mouse_move(x: f64, y: f64) -> Self {
        Self {
            event_type: "mouse_move".to_string(),
            data: serde_json::json!({
                "x": x,
                "y": y
            }),
            window_title: None,
            timestamp: Utc::now(),
        }
    }

    /// Create a screenshot event
    pub fn screenshot(data: Vec<u8>) -> Self {
        Self {
            event_type: "screenshot".to_string(),
            data: serde_json::json!({
                "size_bytes": data.len(),
                "format": "png"
            }),
            window_title: None,
            timestamp: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventBatch {
    pub window_start: DateTime<Utc>,
    pub window_end: DateTime<Utc>,
    pub events: Vec<RawEvent>,
    pub session_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageMetrics {
    pub total_events: u64,
    pub storage_size_bytes: u64,
    pub last_batch_time: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisWindow {
    pub window_id: Uuid,
    pub state: String,
    pub confidence: f64,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateClassification {
    pub state: String,
    pub confidence: f64,
    pub timestamp: DateTime<Utc>,
    pub transition_from: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterventionRequest {
    pub request_id: Uuid,
    pub intervention_type: String,
    pub urgency: String,
    pub context: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardEvent {
    pub reward_id: Uuid,
    pub reward_type: String,
    pub points: u32,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterventionResponse {
    pub request_id: Uuid,
    pub response_text: String,
    pub animation_cues: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationCommand {
    pub command_id: Uuid,
    pub animation_type: String,
    pub parameters: serde_json::Value,
    pub duration_ms: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckRequest {
    pub module_id: ModuleId,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResponse {
    pub module_id: ModuleId,
    pub status: String,
    pub timestamp: DateTime<Utc>,
    pub response_time_ms: u64,
    pub details: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigUpdate {
    pub config_key: String,
    pub config_value: serde_json::Value,
    pub target_module: Option<ModuleId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShutdownRequest {
    pub module_id: ModuleId,
    pub timeout: Duration,
    pub save_state: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorReport {
    pub error_id: Uuid,
    pub error_type: String,
    pub message: String,
    pub module: ModuleId,
    pub timestamp: DateTime<Utc>,
    pub context: Option<serde_json::Value>,
}