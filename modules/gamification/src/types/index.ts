/**
 * Core type definitions for the Gamification Module
 * 
 * Provides comprehensive typing for user-centered gamification features
 * with focus on non-intrusive design and accessibility.
 */

// UUID type definition
export type UUID = string;

// === Core State Types ===

export interface ADHDState {
  type: 'Flow' | 'Hyperfocus' | 'Distracted' | 'Transitioning' | 'Neutral';
  confidence: number;
  depth?: number; // Flow depth level (0-1)
  duration: number; // Duration in milliseconds
  metadata?: Record<string, unknown>;
}

export interface BehavioralMetrics {
  productive_time_ratio: number;
  distraction_frequency: number;
  focus_session_count: number;
  average_session_length: number;
  recovery_time: number;
  transition_smoothness: number;
}

export interface WorkContext {
  application: string;
  window_title: string;
  task_category: 'work' | 'creative' | 'research' | 'communication' | 'leisure' | 'unknown';
  urgency: 'low' | 'medium' | 'high' | 'critical';
  time_of_day: 'morning' | 'afternoon' | 'evening' | 'night';
}

// === Intervention System ===

export interface InterventionType {
  id: string;
  category: 'encouragement' | 'suggestion' | 'celebration' | 'gentle_nudge' | 'milestone';
  minCooldown: number; // minutes
  adaptiveCooldown: boolean;
  requiredState: ADHDState['type'][];
  maxPerHour: number;
  respectsFlow: boolean;
}

export interface InterventionDecision {
  intervene: boolean;
  type?: InterventionType;
  confidence?: number;
  message?: string;
  reason: string;
  suggestedDelay?: number; // milliseconds
}

export interface InterventionContext {
  currentState: ADHDState;
  previousState?: ADHDState;
  metrics: BehavioralMetrics;
  workContext: WorkContext;
  interventionType: string;
  userProfile: UserProfile;
  sessionMetrics: SessionMetrics;
}

export type UserResponse = 
  | 'dismissed_quickly' 
  | 'engaged_positively' 
  | 'clicked_through' 
  | 'ignored' 
  | 'acted_upon';

// === Reward System ===

export interface RewardEvent {
  id: UUID;
  type: 'coins' | 'achievement' | 'milestone' | 'bonus' | 'streak';
  amount?: number;
  achievement?: Achievement;
  milestone?: Milestone;
  message: string;
  visual: VisualReward;
  priority: 'low' | 'medium' | 'high';
  timestamp: Date;
  celebrationType: 'subtle' | 'noticeable' | 'celebration';
}

export interface Achievement {
  id: string;
  name: string;
  description: string;
  icon: string;
  rarity: 'common' | 'rare' | 'epic' | 'legendary';
  category: 'focus' | 'recovery' | 'consistency' | 'milestone' | 'special';
  unlockedAt?: Date;
  progress?: number;
  maxProgress?: number;
  rewards: Reward[];
  hidden: boolean; // For surprise achievements
}

export interface Milestone {
  type: 'focus_time' | 'session_count' | 'streak' | 'recovery' | 'consistency';
  name: string;
  description: string;
  reward: number; // coins
  threshold: number;
  unlockedAt: Date;
  icon: string;
}

export interface Reward {
  type: 'coins' | 'title' | 'theme' | 'animation' | 'companion_outfit';
  amount?: number;
  item?: string;
  unlockLevel?: number;
}

export interface VisualReward {
  animation: 'coin_burst' | 'gentle_glow' | 'sparkle' | 'celebration' | 'achievement_unlock';
  duration: number;
  intensity: 'subtle' | 'medium' | 'high';
  colors: string[];
  position: 'companion' | 'corner' | 'center' | 'toast';
}

// === Progress Tracking ===

export interface SessionMetrics {
  sessionId: UUID;
  startTime: Date;
  endTime?: Date;
  totalFocusTime: number; // milliseconds
  totalDistractedTime: number;
  flowSessions: FlowSession[];
  distractionCount: number;
  recoveryCount: number;
  productivityScore: number; // 0-1
  interventionsReceived: number;
  interventionsEngaged: number;
  deepestFlow: number; // highest flow depth achieved
  longestFocusStreak: number; // milliseconds
}

export interface FlowSession {
  startTime: Date;
  endTime: Date;
  averageDepth: number;
  peakDepth: number;
  duration: number;
  quality: 'light' | 'moderate' | 'deep' | 'profound';
}

export interface DailyMetrics {
  date: Date;
  totalFocusTime: number;
  sessionCount: number;
  averageSessionLength: number;
  bestFlowSession: FlowSession | null;
  interventionResponseRate: number;
  productivityTrend: 'improving' | 'stable' | 'declining';
  streakDays: number;
}

export interface AllTimeMetrics {
  totalFocusHours: number;
  totalSessions: number;
  averageProductivity: number;
  bestStreak: number;
  achievementsUnlocked: number;
  currentLevel: number;
  favoriteTimeOfDay: string;
  improvementRate: number; // weekly percentage
}

export interface StreakInfo {
  type: 'daily_goal' | 'focus_time' | 'session_count' | 'productivity';
  current: number;
  best: number;
  lastUpdate: Date;
  requirement: number;
  nextMilestone: number;
}

// === Companion Behavior ===

export interface AnimationCommand {
  id: UUID;
  type: 'base_state' | 'expression' | 'reaction' | 'celebration' | 'idle_variation';
  animation?: string;
  expression?: Expression;
  duration?: number;
  loop?: boolean;
  priority: 'low' | 'medium' | 'high' | 'urgent';
  delay?: number; // milliseconds before execution
  glow?: GlowEffect;
  particles?: ParticleEffect;
  sound?: SoundEffect;
  interruptible: boolean;
}

export interface Expression {
  type: 'happy' | 'focused' | 'concerned' | 'excited' | 'proud' | 'neutral' | 'sleepy';
  intensity: number; // 0-1
  duration: number;
  eyeState: 'normal' | 'closed' | 'half_closed' | 'wide' | 'sparkle';
  mouthState: 'neutral' | 'smile' | 'slight_smile' | 'open' | 'surprised';
}

export interface GlowEffect {
  intensity: number; // 0-1
  color: string;
  pulseRate?: number; // Hz
  fadeIn?: number; // milliseconds
  fadeOut?: number;
}

export interface ParticleEffect {
  type: 'sparkles' | 'coins' | 'stars' | 'hearts' | 'bubbles';
  count: number;
  duration: number;
  spread: number; // degrees
  speed: number;
  colors: string[];
}

export interface SoundEffect {
  type: 'chime' | 'coin' | 'achievement' | 'celebration' | 'gentle_notification';
  volume: number; // 0-1
  respectsUserSettings: boolean;
}

// === Motivation System ===

export interface MotivationalMessage {
  text: string;
  duration: number; // milliseconds to display
  style: MessageStyle;
  tone: MessageTone;
  audio?: SoundEffect;
  dismissible: boolean;
  actionButton?: ActionButton;
}

export interface MessageStyle {
  fontSize: 'small' | 'medium' | 'large';
  color: string;
  backgroundColor: string;
  position: 'top' | 'center' | 'bottom' | 'companion';
  animation: 'fade' | 'slide' | 'bounce' | 'none';
  urgency: 'low' | 'medium' | 'high';
}

export interface MessageTone {
  formality: number; // 0-1, 0=casual, 1=formal
  enthusiasm: number; // 0-1
  supportiveness: number; // 0-1
  directness: number; // 0-1
  humor: number; // 0-1
}

export interface ActionButton {
  text: string;
  action: 'dismiss' | 'learn_more' | 'take_break' | 'start_timer' | 'view_progress';
  style: 'primary' | 'secondary' | 'minimal';
}

export interface MessageTemplates {
  encouragement: {
    flow_entry: string[];
    sustained_focus: string[];
    recovery: string[];
    personal_record: string[];
  };
  gentle_nudge: {
    distraction_detected: string[];
    long_distraction: string[];
    break_suggestion: string[];
    refocus_tip: string[];
  };
  celebration: {
    milestone_reached: string[];
    achievement_unlocked: string[];
    daily_goal: string[];
    streak_milestone: string[];
  };
  suggestion: {
    break_reminder: string[];
    productivity_tip: string[];
    environment_adjustment: string[];
    time_management: string[];
  };
}

// === User Preferences ===

export interface UserProfile {
  id: UUID;
  name?: string;
  preferences: UserPreferences;
  statistics: UserStatistics;
  personalityProfile: PersonalityProfile;
  accessibilityNeeds: AccessibilityPreferences;
}

export interface UserPreferences {
  // Intervention preferences
  interventionFrequency: 'minimal' | 'moderate' | 'frequent';
  interventionTypes: InterventionType['category'][];
  respectFlowStates: boolean;
  customCooldowns: Record<string, number>;
  
  // Notification preferences
  soundEnabled: boolean;
  visualNotifications: boolean;
  notificationPosition: 'corner' | 'center' | 'companion';
  animationIntensity: 'minimal' | 'moderate' | 'full';
  
  // Reward preferences
  rewardTypes: ('coins' | 'achievements' | 'milestones' | 'celebrations')[];
  celebrationStyle: 'subtle' | 'noticeable' | 'enthusiastic';
  progressVisibility: 'always' | 'on_request' | 'milestones_only';
  
  // Message preferences
  messageStyle: 'encouraging' | 'informative' | 'minimal';
  personalizedMessages: boolean;
  messageFrequency: 'low' | 'medium' | 'high';
  
  // Companion preferences
  companionPersonality: 'supportive' | 'playful' | 'professional' | 'minimal';
  companionAnimations: boolean;
  companionSounds: boolean;
}

export interface PersonalityProfile {
  motivationStyle: 'achievement' | 'social' | 'autonomy' | 'mastery';
  feedbackPreference: 'immediate' | 'summary' | 'milestone';
  challengeLevel: 'easy' | 'moderate' | 'challenging';
  gamificationElements: ('progress_bars' | 'achievements' | 'levels' | 'streaks')[];
  communicationStyle: 'direct' | 'encouraging' | 'humorous' | 'professional';
}

export interface AccessibilityPreferences {
  reducedMotion: boolean;
  highContrast: boolean;
  largeText: boolean;
  screenReaderCompatible: boolean;
  colorBlindnessType?: 'protanopia' | 'deuteranopia' | 'tritanopia' | 'monochromacy';
  customColors?: {
    primary: string;
    secondary: string;
    success: string;
    warning: string;
    error: string;
  };
}

export interface UserStatistics {
  totalUsageTime: number;
  averageSessionLength: number;
  preferredWorkingHours: [number, number]; // [start, end] in 24h format
  mostProductiveTimeSlots: string[];
  responsePatterns: Record<InterventionType['category'], UserResponse[]>;
  improvementMetrics: {
    focusImprovement: number; // percentage
    distractionReduction: number;
    recoveryTimeImprovement: number;
  };
}

// === Configuration ===

export interface GamificationConfig {
  intervention: {
    minCooldownMinutes: number;
    adaptiveCooldown: boolean;
    maxInterventionsPerHour: number;
    respectFlowStates: boolean;
    flowStateThreshold: number; // confidence threshold
    emergencyOverride: boolean;
  };
  
  rewards: {
    coinsPerFocusMinute: number;
    bonusMultiplier: number;
    achievementCoins: Record<string, number>;
    variableRatioBase: number;
    streakBonusMultiplier: number;
    milestoneRewards: Record<string, number>;
  };
  
  progress: {
    sessionTimeoutMinutes: number;
    streakRequirementDays: number;
    milestoneThresholds: number[];
    metricUpdateInterval: number; // seconds
    historyRetentionDays: number;
  };
  
  companion: {
    animationDuration: number;
    expressionVariety: boolean;
    idleVariations: boolean;
    reactionSensitivity: number; // 0-1
    personalityTraits: {
      cheerfulness: number;
      humor: number;
      formality: number;
      supportiveness: number;
    };
  };
  
  messages: {
    maxLength: number;
    personalizedGeneration: boolean;
    templateVariety: boolean;
    adaptiveTone: boolean;
    contextAwareness: boolean;
  };
  
  performance: {
    maxHistoryEntries: number;
    batchUpdateSize: number;
    cacheTimeout: number; // seconds
    animationQueueSize: number;
  };
}

// === Events ===

export interface StateChangeEvent {
  previousState: ADHDState;
  currentState: ADHDState;
  transitionTime: Date;
  confidence: number;
  metrics: BehavioralMetrics;
  workContext?: WorkContext;
  sessionId: UUID;
}

export interface ProgressUpdate {
  session: SessionMetrics;
  records: PersonalRecord[];
  streaks: StreakInfo[];
  trends: TrendAnalysis;
  milestones: Milestone[];
  levelUp?: LevelProgress;
}

export interface PersonalRecord {
  type: 'longest_focus' | 'best_productivity' | 'fastest_recovery' | 'deepest_flow';
  value: number;
  previousBest: number;
  improvement: number;
  timestamp: Date;
  context: string;
}

export interface TrendAnalysis {
  focusTrend: 'improving' | 'stable' | 'declining';
  productivityTrend: 'improving' | 'stable' | 'declining';
  consistencyTrend: 'improving' | 'stable' | 'declining';
  timespan: 'daily' | 'weekly' | 'monthly';
  confidence: number;
  recommendations: string[];
}

export interface LevelProgress {
  currentLevel: number;
  previousLevel: number;
  experiencePoints: number;
  experienceToNext: number;
  rewards: Reward[];
  title?: string;
}

// === Utility Types ===

export interface WalletBalance {
  coins: number;
  totalEarned: number;
  totalSpent: number;
  pendingRewards: Reward[];
  lifetimeBalance: number;
}

export interface ProgressSummary {
  session: SessionMetrics;
  daily: DailyMetrics;
  allTime: AllTimeMetrics;
  streaks: StreakInfo[];
  achievements: Achievement[];
  wallet: WalletBalance;
  level: LevelProgress;
  trends: TrendAnalysis;
}

export interface GamificationMetrics {
  interventionsDelivered: number;
  interventionEngagementRate: number;
  rewardsGranted: number;
  achievementsUnlocked: number;
  averageResponseTime: number;
  userSatisfactionScore: number;
  systemPerformance: {
    averageProcessingTime: number;
    memoryUsage: number;
    errorRate: number;
  };
}

// === Error Types ===

export enum GamificationError {
  InterventionGenerationFailed = 'INTERVENTION_GEN_FAILED',
  CooldownCalculationError = 'COOLDOWN_CALC_ERROR',
  RewardGrantFailed = 'REWARD_GRANT_FAILED',
  AchievementUnlockFailed = 'ACHIEVEMENT_UNLOCK_FAILED',
  ProgressSaveFailed = 'PROGRESS_SAVE_FAILED',
  MetricCalculationError = 'METRIC_CALC_ERROR',
  AnimationQueueFull = 'ANIMATION_QUEUE_FULL',
  InvalidAnimationCommand = 'INVALID_ANIMATION_CMD',
  ConfigValidationError = 'CONFIG_VALIDATION_ERROR',
  UserProfileNotFound = 'USER_PROFILE_NOT_FOUND',
  StateTransitionInvalid = 'STATE_TRANSITION_INVALID',
}

export interface ErrorContext {
  errorCode: GamificationError;
  message: string;
  timestamp: Date;
  context: Record<string, unknown>;
  recoverable: boolean;
  retryCount?: number;
}