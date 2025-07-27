import { MoodState, ActivityState, Position } from './state.types';

// Base event interface
export interface CompanionEvent {
  type: string;
  source: string;
  timestamp: number;
  payload: any;
}

// Animation command events
export interface AnimationCommandEvent extends CompanionEvent {
  type: 'AnimationCommand';
  payload: {
    command:
      | 'play'
      | 'transition'
      | 'setMood'
      | 'setEnergy'
      | 'setMeltLevel'
      | 'stop'
      | 'pause'
      | 'resume';
    animation?: string;
    parameters?: {
      mood?: MoodState;
      energy?: number;
      meltLevel?: number;
      loop?: boolean;
      duration?: number;
      transition?: boolean;
      weight?: number;
      timeScale?: number;
    };
    message?: {
      text: string;
      style?: 'default' | 'intervention' | 'celebration' | 'encouragement';
      priority?: number;
    };
    priority?: number;
  };
}

// State classification events
export interface StateClassificationEvent extends CompanionEvent {
  type: 'StateClassification';
  payload: {
    state: 'focused' | 'distracted' | 'tired' | 'excited' | 'stressed' | 'happy' | 'neutral';
    confidence: number; // 0-1
    context?: {
      application?: string;
      timeOfDay?: string;
      duration?: number;
      previousState?: string;
    };
    source: 'user_input' | 'ai_analysis' | 'time_based' | 'activity_monitor';
  };
}

// Intervention request events
export interface InterventionRequestEvent extends CompanionEvent {
  type: 'InterventionRequest';
  payload: {
    interventionType:
      | 'break_reminder'
      | 'focus_help'
      | 'celebration'
      | 'encouragement'
      | 'check_in';
    message?: string;
    priority?: number;
    urgency?: 'low' | 'medium' | 'high';
    trigger?: {
      reason: string;
      data?: any;
    };
  };
}

// Reward earned events
export interface RewardEarnedEvent extends CompanionEvent {
  type: 'RewardEarned';
  payload: {
    celebrationLevel: 'small' | 'medium' | 'large';
    message?: string;
    value: number; // points to add to happiness
    achievement?: string;
    category?: 'productivity' | 'focus' | 'milestone' | 'streak';
  };
}

// User interaction events
export interface UserInteractionEvent extends CompanionEvent {
  type: 'UserInteraction';
  payload: {
    interactionType: 'click' | 'hover' | 'pet' | 'drag' | 'keyboard' | 'voice';
    position?: Position;
    intensity?: number; // 0-1 for things like pressure
    duration?: number; // for sustained interactions
    data?: any; // additional interaction data
  };
}

// Performance monitoring events
export interface PerformanceEvent extends CompanionEvent {
  type: 'Performance';
  payload: {
    metric:
      | 'frame_time'
      | 'memory_usage'
      | 'cpu_usage'
      | 'events_per_second'
      | 'animation_quality'
      | 'visibility_change';
    value: number;
    threshold?: number;
    action?: 'warning' | 'auto_adjust' | 'report';
  };
}

// System events
export interface SystemEvent extends CompanionEvent {
  type: 'System';
  payload: {
    event: 'initialized' | 'disposed' | 'paused' | 'resumed' | 'config_changed' | 'error';
    data?: any;
    error?: string;
  };
}

// Custom application events
export interface ApplicationEvent extends CompanionEvent {
  type: 'Application';
  payload: {
    event:
      | 'focus_session_start'
      | 'focus_session_end'
      | 'break_start'
      | 'break_end'
      | 'pomodoro_complete';
    data?: {
      duration?: number;
      productivity_score?: number;
      focus_level?: number;
      break_type?: string;
    };
  };
}

// Error events
export interface ErrorEvent extends CompanionEvent {
  type: 'Error';
  payload: {
    error: string;
    severity: 'low' | 'medium' | 'high' | 'critical';
    context?: any;
    originalEvent?: CompanionEvent;
    handlerId?: string;
  };
}

// Legacy compatibility
export interface WorkContextEvent extends CompanionEvent {
  type: 'WorkContext';
  payload: {
    activity: 'coding' | 'writing' | 'designing' | 'researching' | 'idle';
    language?: string;
    tool?: string;
    complexity?: 'low' | 'medium' | 'high';
  };
}

// Internal events emitted by Cute Figurine
export interface CompanionInteractionEvent extends CompanionEvent {
  type: 'CompanionInteraction';
  payload: {
    interaction: 'pet' | 'click' | 'hover' | 'drag';
    position: Position;
    timestamp: number;
  };
}

export interface CompanionStateChangeEvent extends CompanionEvent {
  type: 'CompanionStateChange';
  payload: {
    previousMood: MoodState;
    currentMood: MoodState;
    trigger: string;
    timestamp: number;
  };
}

// Union type for all event types
export type SkellyEvent =
  | AnimationCommandEvent
  | StateClassificationEvent
  | InterventionRequestEvent
  | RewardEarnedEvent
  | UserInteractionEvent
  | PerformanceEvent
  | SystemEvent
  | ApplicationEvent
  | ErrorEvent
  | WorkContextEvent
  | CompanionInteractionEvent
  | CompanionStateChangeEvent;

// Legacy compatibility
export type CuteFigurineEvent = SkellyEvent;

// Event type names for type safety
export const EventTypes = {
  ANIMATION_COMMAND: 'AnimationCommand' as const,
  STATE_CLASSIFICATION: 'StateClassification' as const,
  INTERVENTION_REQUEST: 'InterventionRequest' as const,
  REWARD_EARNED: 'RewardEarned' as const,
  USER_INTERACTION: 'UserInteraction' as const,
  PERFORMANCE: 'Performance' as const,
  SYSTEM: 'System' as const,
  APPLICATION: 'Application' as const,
  ERROR: 'Error' as const,
  WORK_CONTEXT: 'WorkContext' as const,
  COMPANION_INTERACTION: 'CompanionInteraction' as const,
  COMPANION_STATE_CHANGE: 'CompanionStateChange' as const,
} as const;

// Event priority levels
export enum EventPriority {
  LOW = 1,
  MEDIUM = 2,
  HIGH = 3,
  URGENT = 4,
  CRITICAL = 5,
}

// Event creation helpers
export const createEvent = {
  animationCommand: (
    command: AnimationCommandEvent['payload']['command'],
    source: string = 'user',
    options: Partial<AnimationCommandEvent['payload']> = {}
  ): AnimationCommandEvent => ({
    type: EventTypes.ANIMATION_COMMAND,
    source,
    timestamp: Date.now(),
    payload: {
      command,
      ...options,
    },
  }),

  stateClassification: (
    state: StateClassificationEvent['payload']['state'],
    confidence: number,
    source: StateClassificationEvent['payload']['source'],
    context?: StateClassificationEvent['payload']['context']
  ): StateClassificationEvent => ({
    type: EventTypes.STATE_CLASSIFICATION,
    source,
    timestamp: Date.now(),
    payload: {
      state,
      confidence,
      context,
      source,
    },
  }),

  interventionRequest: (
    interventionType: InterventionRequestEvent['payload']['interventionType'],
    source: string = 'system',
    options: Partial<InterventionRequestEvent['payload']> = {}
  ): InterventionRequestEvent => ({
    type: EventTypes.INTERVENTION_REQUEST,
    source,
    timestamp: Date.now(),
    payload: {
      interventionType,
      priority: EventPriority.MEDIUM,
      urgency: 'medium',
      ...options,
    },
  }),

  rewardEarned: (
    celebrationLevel: RewardEarnedEvent['payload']['celebrationLevel'],
    value: number,
    source: string = 'system',
    options: Partial<RewardEarnedEvent['payload']> = {}
  ): RewardEarnedEvent => ({
    type: EventTypes.REWARD_EARNED,
    source,
    timestamp: Date.now(),
    payload: {
      celebrationLevel,
      value,
      category: 'productivity',
      ...options,
    },
  }),

  userInteraction: (
    interactionType: UserInteractionEvent['payload']['interactionType'],
    options: Partial<UserInteractionEvent['payload']> = {}
  ): UserInteractionEvent => ({
    type: EventTypes.USER_INTERACTION,
    source: 'user',
    timestamp: Date.now(),
    payload: {
      interactionType,
      ...options,
    },
  }),

  performance: (
    metric: PerformanceEvent['payload']['metric'],
    value: number,
    threshold?: number
  ): PerformanceEvent => ({
    type: EventTypes.PERFORMANCE,
    source: 'system',
    timestamp: Date.now(),
    payload: {
      metric,
      value,
      threshold,
    },
  }),

  system: (event: SystemEvent['payload']['event'], data?: any, error?: string): SystemEvent => ({
    type: EventTypes.SYSTEM,
    source: 'system',
    timestamp: Date.now(),
    payload: {
      event,
      data,
      error,
    },
  }),

  application: (
    event: ApplicationEvent['payload']['event'],
    data?: ApplicationEvent['payload']['data']
  ): ApplicationEvent => ({
    type: EventTypes.APPLICATION,
    source: 'application',
    timestamp: Date.now(),
    payload: {
      event,
      data,
    },
  }),

  error: (
    error: string,
    severity: ErrorEvent['payload']['severity'] = 'medium',
    context?: any,
    originalEvent?: CompanionEvent
  ): ErrorEvent => ({
    type: EventTypes.ERROR,
    source: 'system',
    timestamp: Date.now(),
    payload: {
      error,
      severity,
      context,
      originalEvent,
    },
  }),
};
