// Skelly-Jelly Core Data Schemas
// These TypeScript interfaces define the data contracts between layers

// ============= Event Schemas =============
// OWNERSHIP: Data Capture module creates and emits all RawEvent types

interface RawEvent {
  timestamp: bigint;  // nanoseconds since epoch
  session_id: string; // UUID
  event_type: EventType;
  data: KeystrokeEvent | MouseEvent | WindowEvent | ScreenshotEvent;
}

enum EventType {
  KEYSTROKE = 1,      // Owner: data-capture/keystroke_monitor
  MOUSE_MOVE = 2,     // Owner: data-capture/mouse_monitor
  MOUSE_CLICK = 3,    // Owner: data-capture/mouse_monitor
  WINDOW_FOCUS = 4,   // Owner: data-capture/window_monitor
  WINDOW_SWITCH = 5,  // Owner: data-capture/window_monitor
  SCREENSHOT = 6,     // Owner: data-capture/screenshot_capturer
  PROCESS_SPAWN = 7,  // Owner: data-capture/process_monitor
  RESOURCE_USAGE = 8  // Owner: data-capture/resource_monitor
}

interface KeystrokeEvent {
  key_code?: number;
  is_backspace: boolean;
  is_delete: boolean;
  inter_key_interval_ms: number;
  session_character_count: number;
  word_completed: boolean;
}

interface MouseEvent {
  x: number;
  y: number;
  velocity_pixels_per_ms?: number;
  acceleration?: number;
  click_count?: number;
  idle_time_ms: number;
}

interface WindowEvent {
  app_name: string;
  window_title: string;
  dwell_time_ms: number;
  is_browser: boolean;
  url_domain?: string;  // privacy: domain only
}

interface ScreenshotEvent {
  capture_reason: "scheduled" | "context_switch" | "error_detected";
  
  // Extracted metadata (stored permanently)
  has_code_editor: boolean;
  has_terminal: boolean;
  text_density: number;  // words per screen
  color_complexity: number;  // distinct hues
  ui_element_count: number;
  masked_regions: Rectangle[];  // PII locations
  
  // Temporary image reference (deleted after analysis)
  image_ref?: ScreenshotReference;
}

interface Rectangle {
  x: number;
  y: number;
  width: number;
  height: number;
}

// Screenshot handling strategy
interface ScreenshotReference {
  id: string;  // UUID
  storage_type: "memory" | "temp_file";
  path?: string;  // if temp_file
  size_bytes: number;
  expires_at: bigint;  // timestamp when to delete
  processed: boolean;  // has Analysis Engine seen it?
}

// ============= Event Ownership & Flow =============
// Data Capture Module → Storage Module → Analysis Engine → Gamification → AI

interface EventEmitter {
  module: ModuleName;
  component: string;
  version: string;
}

enum ModuleName {
  DATA_CAPTURE = "data-capture",
  STORAGE = "storage", 
  ANALYSIS_ENGINE = "analysis-engine",
  GAMIFICATION = "gamification",
  AI_INTEGRATION = "ai-integration",
  CUTE_FIGURINE = "cute-figurine"
}

// ============= Analysis Schemas =============

interface AnalysisWindow {
  window_start: bigint;
  window_end: bigint;
  metrics: WindowMetrics;
  state_classification: StateClassification;
  confidence: number;  // 0.0 - 1.0
}

interface WindowMetrics {
  // Keystroke metrics
  mean_iki_ms: number;
  iki_coefficient_variation: number;
  burst_count: number;
  backspace_rate: number;
  typing_velocity_wpm: number;
  
  // Mouse metrics
  mean_velocity: number;
  click_rate_per_min: number;
  idle_percentage: number;
  
  // Window metrics
  switch_count: number;
  mean_dwell_time_ms: number;
  unique_apps: number;
  
  // Resource metrics
  cpu_usage_percent: number;
  memory_delta_mb: number;
  process_spawn_rate: number;
}

interface StateClassification {
  state: ADHDState;
  sub_state?: string;
  trigger_metrics: string[];  // which metrics triggered this
}

enum ADHDState {
  FLOW = "flow",
  HYPERFOCUS = "hyperfocus",
  PRODUCTIVE_SWITCHING = "productive_switching",
  DISTRACTED = "distracted",
  PERSEVERATION = "perseveration",
  IDLE = "idle",
  BREAK = "break"
}

// ============= Intervention Schemas =============

interface InterventionRequest {
  current_state: StateClassification;
  work_context: WorkContext;
  last_intervention_ms_ago: number;
  user_energy_level: "high" | "medium" | "low";
  time_of_day_factor: number;  // 0.0 (worst) - 1.0 (best)
}

interface WorkContext {
  detected_activity: ActivityType;
  current_task?: string;
  error_state?: ErrorContext;
  session_duration_ms: number;
  productive_streak_count: number;
}

enum ActivityType {
  CODING = "coding",
  DEBUGGING = "debugging",
  DESIGNING = "designing",
  WRITING = "writing",
  RESEARCHING = "researching",
  COMMUNICATING = "communicating",
  PLANNING = "planning",
  UNKNOWN = "unknown"
}

interface ErrorContext {
  error_type: "syntax" | "runtime" | "logic" | "design";
  persistence_ms: number;
  attempted_fixes: number;
}

interface InterventionResponse {
  should_intervene: boolean;
  intervention_type?: InterventionType;
  message?: string;
  animation?: AnimationCommand;
  delay_ms?: number;  // wait before showing
}

enum InterventionType {
  GENTLE_NUDGE = "gentle_nudge",
  BREAK_SUGGESTION = "break_suggestion",
  HELPFUL_TIP = "helpful_tip",
  CELEBRATION = "celebration",
  CONTEXT_SWITCH_WARNING = "context_switch_warning",
  HYPERFOCUS_CHECK = "hyperfocus_check"
}

// ============= Animation Schemas =============

interface AnimationCommand {
  animation_id: string;
  duration_ms: number;
  expression?: FigurineExpression;
  movement?: FigurineMovement;
  effects?: VisualEffect[];
  message?: TextBubble;  // Optional message to display
}

interface TextBubble {
  text: string;
  duration_ms: number;
  style: "encouragement" | "tip" | "celebration" | "gentle";
  position: "above" | "beside";  // relative to figurine
}

enum FigurineExpression {
  NEUTRAL = "neutral",
  HAPPY = "happy",
  FOCUSED = "focused",
  SLEEPY = "sleepy",
  ENCOURAGING = "encouraging",
  CELEBRATING = "celebrating",
  CONCERNED = "concerned"
}

interface FigurineMovement {
  type: "bounce" | "sway" | "melt" | "solidify" | "breathe";
  amplitude: number;  // 0.0 - 1.0
  frequency: number;  // Hz
}

interface VisualEffect {
  type: "sparkle" | "glow" | "trail" | "bubble";
  color: string;  // hex
  duration_ms: number;
  position: "around_figurine" | "from_figurine" | "to_screen_edge";
}

// ============= Configuration Schemas =============

interface SystemConfig {
  llm_provider: LLMProvider;
  llm_api_key: string;  // encrypted
  capture_settings: CaptureSettings;
  intervention_settings: InterventionSettings;
  figurine_position: Position;
}

interface LLMProvider {
  type: "llama.cpp" | "openai" | "anthropic";
  model_path?: string;  // for local models
  model_name: string;
  max_tokens: number;
  temperature: number;
}

interface CaptureSettings {
  screenshot_interval_ms: number;
  screenshot_on_context_switch: boolean;
  mask_sensitive_content: boolean;
  capture_domains: boolean;
  resource_sampling_hz: number;
}

interface InterventionSettings {
  min_interval_ms: number;  // minimum time between interventions
  hyperfocus_alert_ms: number;  // alert after X ms of hyperfocus
  distraction_threshold_ms: number;  // intervene after X ms distracted
  work_hours: TimeRange[];
  quiet_hours: TimeRange[];
}

interface TimeRange {
  start_hour: number;  // 0-23
  start_minute: number;  // 0-59
  end_hour: number;
  end_minute: number;
  days: DayOfWeek[];
}

enum DayOfWeek {
  MONDAY = 1,
  TUESDAY = 2,
  WEDNESDAY = 3,
  THURSDAY = 4,
  FRIDAY = 5,
  SATURDAY = 6,
  SUNDAY = 7
}

interface Position {
  x: number;  // pixels from left
  y: number;  // pixels from top
  anchor: "top-left" | "top-right" | "bottom-left" | "bottom-right";
}

// ============= Storage Schemas (SQL) =============

// These would be implemented as SQL tables, shown here as TypeScript for consistency

interface EventRecord {
  id: bigint;  // auto-increment
  timestamp: bigint;
  session_id: string;
  event_type: number;  // EventType enum
  data: Uint8Array;  // Protocol Buffer encoded
}

interface AnalysisRecord {
  id: bigint;
  window_start: bigint;
  window_end: bigint;
  state: string;  // ADHDState enum
  confidence: number;
  metrics_json: string;  // JSON blob of WindowMetrics
}

interface InterventionRecord {
  id: bigint;
  timestamp: bigint;
  intervention_type: string;
  message?: string;
  user_dismissed: boolean;
  user_feedback?: "helpful" | "annoying" | "neutral";
  context_json: string;  // JSON blob of context
}

// ============= Message Bus Events =============

type BusMessage = 
  | { type: "raw_event"; data: RawEvent }
  | { type: "analysis_complete"; data: AnalysisWindow }
  | { type: "state_change"; data: StateClassification }
  | { type: "intervention_request"; data: InterventionRequest }
  | { type: "intervention_response"; data: InterventionResponse }
  | { type: "animation_command"; data: AnimationCommand }
  | { type: "config_update"; data: Partial<SystemConfig> }
  | { type: "shutdown"; reason: string };