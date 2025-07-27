//! Storage module type definitions and interfaces

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::fmt;
use std::time::{Duration, Instant};

/// Unique identifier for screenshots
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct ScreenshotId(Uuid);

impl ScreenshotId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
    
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl fmt::Display for ScreenshotId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Default for ScreenshotId {
    fn default() -> Self {
        Self::new()
    }
}

/// Image formats supported by the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImageFormat {
    Png,
    WebP,
    Heic,
}

/// Screen region for screenshot capture
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenRegion {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

/// Raw events received from Data Capture module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RawEvent {
    Keystroke(KeystrokeEvent),
    MouseMove(MouseMoveEvent),
    MouseClick(MouseClickEvent),
    WindowFocus(WindowFocusEvent),
    Screenshot(ScreenshotEvent),
    ProcessStart(ProcessEvent),
    ResourceUsage(ResourceEvent),
}

/// Keystroke event data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeystrokeEvent {
    pub timestamp: DateTime<Utc>,
    pub key_code: u32,
    pub modifiers: KeyModifiers,
    pub inter_key_interval_ms: Option<u32>,
}

/// Keyboard modifiers
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KeyModifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub meta: bool,
}

/// Mouse movement event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MouseMoveEvent {
    pub timestamp: DateTime<Utc>,
    pub x: i32,
    pub y: i32,
    pub velocity: f32, // pixels per second
}

/// Mouse click event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MouseClickEvent {
    pub timestamp: DateTime<Utc>,
    pub x: i32,
    pub y: i32,
    pub button: MouseButton,
    pub click_type: ClickType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClickType {
    Single,
    Double,
    Triple,
}

/// Window focus change event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowFocusEvent {
    pub timestamp: DateTime<Utc>,
    pub window_title: String,
    pub app_name: String,
    pub process_id: u32,
    pub duration_ms: Option<u32>, // Time spent in previous window
}

/// Screenshot capture event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenshotEvent {
    pub timestamp: DateTime<Utc>,
    pub screenshot_id: ScreenshotId,
    #[serde(skip_serializing, skip_deserializing)]
    pub data: Vec<u8>, // Raw image data (not serialized)
    pub format: ImageFormat,
    pub window_title: String,
    pub app_name: String,
    pub region: ScreenRegion,
    pub privacy_masked: bool,
}

/// Process lifecycle event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessEvent {
    pub timestamp: DateTime<Utc>,
    pub process_id: u32,
    pub process_name: String,
    pub event_type: ProcessEventType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProcessEventType {
    Started,
    Stopped,
    Crashed,
}

/// System resource usage event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceEvent {
    pub timestamp: DateTime<Utc>,
    pub cpu_percent: f32,
    pub memory_mb: u32,
    pub disk_io_mb_per_sec: f32,
    pub network_io_mb_per_sec: f32,
}

/// Batch of events for analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventBatch {
    pub window_id: Uuid,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub events: Vec<RawEvent>,
    pub screenshot_refs: Vec<ScreenshotId>,
}

/// Screenshot metadata stored permanently
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenshotMetadata {
    pub timestamp: DateTime<Utc>,
    pub window_title: String,
    pub app_name: String,
    pub screen_region: ScreenRegion,
    pub text_density: f32,
    pub dominant_colors: Vec<String>, // Hex colors
    pub ui_element_count: u32,
    pub privacy_masked: bool,
}

/// Event window for batching
pub struct EventWindow {
    pub window_id: Uuid,
    pub start_time: Instant,
    pub end_time: Option<Instant>,
    pub events: Vec<RawEvent>,
    pub screenshot_refs: Vec<ScreenshotId>,
}

impl EventWindow {
    pub fn new() -> Self {
        Self {
            window_id: Uuid::new_v4(),
            start_time: Instant::now(),
            end_time: None,
            events: Vec::with_capacity(1000),
            screenshot_refs: Vec::with_capacity(10),
        }
    }
}

/// Message types for Event Bus communication
#[derive(Debug, Clone)]
pub enum BusMessage {
    RawEvent(RawEvent),
    EventBatch(EventBatch),
    AnalysisComplete(AnalysisWindow),
    StateChange(StateClassification),
    InterventionRequest(InterventionRequest),
    AnimationCommand(AnimationCommand),
    Shutdown(String),
}

// Placeholder types for other modules
#[derive(Debug, Clone)]
pub struct AnalysisWindow;

#[derive(Debug, Clone)]
pub struct StateClassification;

#[derive(Debug, Clone)]
pub struct InterventionRequest;

#[derive(Debug, Clone)]
pub struct AnimationCommand;

impl Default for EventWindow {
    fn default() -> Self {
        Self::new()
    }
}

impl RawEvent {
    /// Get the timestamp of the event
    pub fn timestamp(&self) -> DateTime<Utc> {
        match self {
            Self::Keystroke(e) => e.timestamp,
            Self::MouseMove(e) => e.timestamp,
            Self::MouseClick(e) => e.timestamp,
            Self::WindowFocus(e) => e.timestamp,
            Self::Screenshot(e) => e.timestamp,
            Self::ProcessStart(e) => e.timestamp,
            Self::ResourceUsage(e) => e.timestamp,
        }
    }
    
    /// Get the event type as a string
    pub fn event_type(&self) -> &'static str {
        match self {
            Self::Keystroke(_) => "keystroke",
            Self::MouseMove(_) => "mouse_move",
            Self::MouseClick(_) => "mouse_click",
            Self::WindowFocus(_) => "window_focus",
            Self::Screenshot(_) => "screenshot",
            Self::ProcessStart(_) => "process_start",
            Self::ResourceUsage(_) => "resource_usage",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_screenshot_id() {
        let id1 = ScreenshotId::new();
        let id2 = ScreenshotId::new();
        assert_ne!(id1, id2);
        assert_eq!(id1.as_bytes().len(), 16);
    }
    
    #[test]
    fn test_event_timestamp() {
        let now = Utc::now();
        let event = RawEvent::Keystroke(KeystrokeEvent {
            timestamp: now,
            key_code: 65,
            modifiers: KeyModifiers {
                shift: false,
                ctrl: false,
                alt: false,
                meta: false,
            },
            inter_key_interval_ms: None,
        });
        assert_eq!(event.timestamp(), now);
        assert_eq!(event.event_type(), "keystroke");
    }
}