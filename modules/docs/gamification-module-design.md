# Gamification Module Design

## Module Purpose and Responsibilities

The Gamification Module provides the motivational engine for Skelly-Jelly through a non-intrusive reward system. It manages intervention timing, tracks user progress, and coordinates the melty skeleton companion's behavior to provide ambient support without disrupting focus states.

### Core Responsibilities
- **Intervention Timing**: Decide when and how to intervene based on ADHD state
- **Reward Management**: Track and distribute virtual rewards (coins, achievements)
- **Progress Tracking**: Monitor user patterns and improvement over time
- **Companion Behavior**: Control skeleton animations and expressions
- **Cooldown Management**: Respect user attention with adaptive notification frequency
- **Motivation Engine**: Generate appropriate encouragement without being patronizing

## Key Components and Their Functions

### 1. Intervention Controller
```typescript
export class InterventionController {
  // Cooldown tracking per intervention type
  private cooldowns: Map<InterventionType, CooldownTracker>;
  
  // User preference learning
  private preferenceEngine: UserPreferenceEngine;
  
  // State history for context
  private stateHistory: CircularBuffer<StateSnapshot>;
  
  // Intervention effectiveness tracker
  private effectivenessTracker: EffectivenessTracker;
  
  async shouldIntervene(
    currentState: ADHDState,
    metrics: BehavioralMetrics,
    context: WorkContext
  ): Promise<InterventionDecision> {
    // Never interrupt flow states
    if (currentState.type === 'Flow' && currentState.confidence > 0.8) {
      return { intervene: false, reason: 'User in flow state' };
    }
    
    // Check cooldowns
    const availableInterventions = this.getAvailableInterventions();
    if (availableInterventions.length === 0) {
      return { intervene: false, reason: 'All interventions on cooldown' };
    }
    
    // Evaluate intervention opportunity
    const opportunity = this.evaluateOpportunity(currentState, metrics, context);
    
    if (opportunity.score > this.preferenceEngine.getThreshold()) {
      return {
        intervene: true,
        type: opportunity.bestIntervention,
        confidence: opportunity.confidence,
        message: opportunity.suggestedMessage
      };
    }
    
    return { intervene: false, reason: 'No good opportunity' };
  }
}

interface InterventionType {
  id: string;
  category: 'encouragement' | 'suggestion' | 'celebration' | 'gentle_nudge';
  minCooldown: number;  // minimum minutes between interventions
  adaptiveCooldown: boolean;  // learns from user response
  requiredState: ADHDState['type'][];  // valid states for this intervention
}

class CooldownTracker {
  private lastTriggered: Map<string, Date> = new Map();
  private cooldownMultipliers: Map<string, number> = new Map();
  
  canTrigger(intervention: InterventionType): boolean {
    const last = this.lastTriggered.get(intervention.id);
    if (!last) return true;
    
    const multiplier = this.cooldownMultipliers.get(intervention.id) || 1;
    const cooldownMs = intervention.minCooldown * 60 * 1000 * multiplier;
    
    return Date.now() - last.getTime() > cooldownMs;
  }
  
  recordResponse(intervention: InterventionType, response: UserResponse) {
    // Adapt cooldown based on user response
    if (response === 'dismissed_quickly') {
      this.increaseCooldown(intervention.id);
    } else if (response === 'engaged_positively') {
      this.decreaseCooldown(intervention.id);
    }
  }
}
```

### 2. Reward System
```typescript
export class RewardSystem {
  // Token/coin management
  private wallet: UserWallet;
  
  // Achievement tracker
  private achievements: AchievementManager;
  
  // Reward distribution engine
  private distributionEngine: RewardDistributionEngine;
  
  // Visual reward queue
  private visualRewardQueue: Queue<VisualReward>;
  
  async processStateChange(
    oldState: ADHDState,
    newState: ADHDState,
    duration: number
  ): Promise<RewardEvent[]> {
    const rewards: RewardEvent[] = [];
    
    // Reward sustained focus
    if (oldState.type === 'Flow' && duration > 15 * 60 * 1000) {
      rewards.push(await this.grantFocusReward(duration));
    }
    
    // Reward recovery from distraction
    if (oldState.type === 'Distracted' && newState.type === 'Flow') {
      rewards.push(await this.grantRecoveryReward());
    }
    
    // Check for achievement unlocks
    const newAchievements = await this.achievements.check({
      state: newState,
      duration,
      history: this.getRecentHistory()
    });
    
    rewards.push(...newAchievements);
    
    // Apply variable ratio reinforcement
    if (this.distributionEngine.shouldReward()) {
      rewards.push(await this.grantRandomBonus());
    }
    
    return rewards;
  }
}

interface RewardEvent {
  type: 'coins' | 'achievement' | 'milestone' | 'bonus';
  amount?: number;
  achievement?: Achievement;
  message: string;
  visual: VisualReward;
  priority: 'low' | 'medium' | 'high';
}

class RewardDistributionEngine {
  // Variable ratio schedule parameters
  private baseRate = 0.15;  // 15% chance
  private streak = 0;
  private lastReward: Date | null = null;
  
  shouldReward(): boolean {
    // Implement variable ratio reinforcement
    // Higher chance after longer gaps, lower after recent rewards
    const timeSinceLastReward = this.lastReward 
      ? Date.now() - this.lastReward.getTime() 
      : Infinity;
      
    const timeMultiplier = Math.min(timeSinceLastReward / (30 * 60 * 1000), 2);
    const streakMultiplier = Math.max(1 - (this.streak * 0.1), 0.5);
    
    const chance = this.baseRate * timeMultiplier * streakMultiplier;
    
    return Math.random() < chance;
  }
}
```

### 3. Progress Tracker
```typescript
export class ProgressTracker {
  // Session metrics
  private sessionMetrics: SessionMetrics;
  
  // Long-term trends
  private trendAnalyzer: TrendAnalyzer;
  
  // Personal records
  private recordKeeper: PersonalRecordKeeper;
  
  // Streak tracking
  private streakTracker: StreakTracker;
  
  async updateProgress(
    state: ADHDState,
    metrics: BehavioralMetrics,
    timestamp: Date
  ): Promise<ProgressUpdate> {
    // Update session metrics
    this.sessionMetrics.update(state, metrics);
    
    // Check for personal records
    const newRecords = await this.recordKeeper.check({
      focusDuration: this.sessionMetrics.currentFocusDuration,
      productivityScore: metrics.productive_time_ratio,
      flowDepth: state.type === 'Flow' ? state.depth : 0
    });
    
    // Update streaks
    const streakUpdate = this.streakTracker.update(state, timestamp);
    
    // Analyze trends
    const trends = await this.trendAnalyzer.analyze(
      this.getHistoricalData(),
      this.sessionMetrics
    );
    
    return {
      session: this.sessionMetrics.getSummary(),
      records: newRecords,
      streaks: streakUpdate,
      trends,
      milestones: this.checkMilestones()
    };
  }
  
  private checkMilestones(): Milestone[] {
    const milestones: Milestone[] = [];
    
    // Time-based milestones
    const totalFocusTime = this.getTotalFocusTime();
    const focusMilestones = [
      { hours: 1, name: "First Hour", reward: 50 },
      { hours: 10, name: "Focus Apprentice", reward: 100 },
      { hours: 50, name: "Focus Journeyman", reward: 250 },
      { hours: 100, name: "Focus Master", reward: 500 }
    ];
    
    for (const milestone of focusMilestones) {
      if (this.crossedThreshold(totalFocusTime, milestone.hours * 3600)) {
        milestones.push({
          type: 'focus_time',
          name: milestone.name,
          reward: milestone.reward,
          unlockedAt: new Date()
        });
      }
    }
    
    return milestones;
  }
}

interface SessionMetrics {
  startTime: Date;
  totalFocusTime: number;
  totalDistractedTime: number;
  flowSessions: FlowSession[];
  distractionCount: number;
  recoveryCount: number;
  productivityScore: number;
  interventionsReceived: number;
  interventionsEngaged: number;
}
```

### 4. Companion Behavior Manager
```typescript
export class CompanionBehaviorManager {
  // Animation state machine
  private animationStateMachine: AnimationStateMachine;
  
  // Expression controller
  private expressionController: ExpressionController;
  
  // Personality engine
  private personalityEngine: PersonalityEngine;
  
  // Animation queue
  private animationQueue: PriorityQueue<Animation>;
  
  async updateCompanionState(
    userState: ADHDState,
    rewardEvents: RewardEvent[],
    intervention?: InterventionDecision
  ): Promise<AnimationCommand[]> {
    const commands: AnimationCommand[] = [];
    
    // Base state animation
    const baseAnimation = this.getBaseAnimation(userState);
    commands.push(baseAnimation);
    
    // Layer expression based on user state
    const expression = this.expressionController.getExpression(userState);
    commands.push({
      type: 'expression',
      expression,
      duration: 3000,
      priority: 'medium'
    });
    
    // Add reward animations
    for (const reward of rewardEvents) {
      const rewardAnimation = this.createRewardAnimation(reward);
      commands.push(rewardAnimation);
    }
    
    // Add intervention animation if needed
    if (intervention?.intervene) {
      const interventionAnim = this.createInterventionAnimation(intervention);
      commands.push(interventionAnim);
    }
    
    // Apply personality modifiers
    return this.personalityEngine.modifyAnimations(commands, userState);
  }
  
  private getBaseAnimation(state: ADHDState): AnimationCommand {
    const stateAnimations = {
      Flow: {
        animation: 'gentle_float',
        speed: 0.8,
        amplitude: 'low',
        glow: { intensity: 0.8, color: '#4A90E2' }
      },
      Hyperfocus: {
        animation: 'intense_focus',
        speed: 0.5,
        amplitude: 'minimal',
        glow: { intensity: 1.0, color: '#E27B4A' }
      },
      Distracted: {
        animation: 'restless_sway',
        speed: 1.2,
        amplitude: 'high',
        glow: { intensity: 0.3, color: '#E2D84A' }
      },
      Transitioning: {
        animation: 'morphing',
        speed: 1.0,
        amplitude: 'medium',
        glow: { intensity: 0.5, color: '#B84AE2' }
      },
      Neutral: {
        animation: 'idle_breathing',
        speed: 1.0,
        amplitude: 'medium',
        glow: { intensity: 0.4, color: '#FFFFFF' }
      }
    };
    
    return {
      type: 'base_state',
      ...stateAnimations[state.type],
      loop: true,
      priority: 'low'
    };
  }
}

interface AnimationCommand {
  type: 'base_state' | 'expression' | 'reaction' | 'celebration';
  animation?: string;
  expression?: Expression;
  duration?: number;
  loop?: boolean;
  priority: 'low' | 'medium' | 'high';
  glow?: GlowEffect;
  particles?: ParticleEffect;
}
```

### 5. Motivation Engine
```typescript
export class MotivationEngine {
  // Message generation
  private messageGenerator: MessageGenerator;
  
  // Tone calibration
  private toneCalibrator: ToneCalibrator;
  
  // Context awareness
  private contextAnalyzer: ContextAnalyzer;
  
  // Personalization
  private personalizationEngine: PersonalizationEngine;
  
  async generateMessage(
    context: InterventionContext
  ): Promise<MotivationalMessage> {
    // Analyze context
    const analysis = await this.contextAnalyzer.analyze(context);
    
    // Select appropriate tone
    const tone = this.toneCalibrator.selectTone(
      analysis.userMood,
      analysis.workType,
      analysis.timeOfDay
    );
    
    // Generate base message
    const baseMessage = await this.messageGenerator.generate({
      type: context.interventionType,
      state: context.currentState,
      workContext: context.workContext,
      tone
    });
    
    // Personalize message
    const personalizedMessage = await this.personalizationEngine.personalize(
      baseMessage,
      context.userProfile
    );
    
    return {
      text: personalizedMessage.text,
      duration: this.calculateDisplayDuration(personalizedMessage.text),
      style: this.getMessageStyle(tone),
      audio: personalizedMessage.audio,
      dismissible: true
    };
  }
  
  private getMessageTemplates(): MessageTemplates {
    return {
      encouragement: {
        flow_entry: [
          "Nice! You're getting into the zone 🎯",
          "Feel that focus building? Keep it up!",
          "Your concentration is impressive right now"
        ],
        sustained_focus: [
          "You've been crushing it for {duration} minutes!",
          "Solid focus session happening here 💪",
          "{duration} minutes of pure productivity!"
        ],
        recovery: [
          "Good job getting back on track!",
          "Nice recovery! Your skeleton pal is proud",
          "See? You got this! 🦴"
        ]
      },
      gentle_nudge: {
        distraction_detected: [
          "Hey, everything okay over there?",
          "Need a quick break? That's totally fine!",
          "Your skeleton friend noticed you might be stuck"
        ],
        long_distraction: [
          "Been a minute - want to try a different approach?",
          "Sometimes a fresh start helps!",
          "No judgment - we all get sidetracked sometimes"
        ]
      },
      celebration: {
        milestone_reached: [
          "🎉 You just hit {milestone}! Amazing!",
          "NEW RECORD! {achievement} unlocked!",
          "Skeleton dance party! You reached {milestone}! 🦴💃"
        ],
        daily_goal: [
          "Daily goal crushed! Time to celebrate 🎊",
          "You did it! Another productive day in the books",
          "Mission accomplished! Your skeleton is melting with joy"
        ]
      }
    };
  }
}
```

## Integration Points with Other Modules

### Input Sources
- **Analysis Engine**: Receives ADHDState classifications and BehavioralMetrics
- **Event Bus**: Subscribes to StateChange messages

### Output Consumers  
- **AI Integration**: Sends InterventionRequest for message generation
- **Cute Figurine**: Sends AnimationCommand for visual updates
- **Storage**: May log reward events and progress data

### Communication Flow
```
Analysis Engine --StateChange--> Gamification
                                      |
                                      ├─> Evaluate Intervention
                                      ├─> Process Rewards  
                                      ├─> Update Progress
                                      └─> Generate Animation
                                            |
                                            v
                                    AI Integration
                                            |
                                            v
                                    Cute Figurine
```

## Technology Choices

### Core Technology: TypeScript
- **Reasoning**: Complex state management, good for game-like logic, integrates well with frontend
- **Runtime**: Node.js for backend logic, shared types with frontend
- **Key Libraries**:
  - `xstate`: State machine for companion behavior
  - `date-fns`: Time calculation and cooldown management  
  - `joi`: Configuration validation
  - `winston`: Logging

### Data Storage
- **Progress Data**: SQLite via Storage module
- **Session State**: In-memory with periodic persistence
- **User Preferences**: JSON file with encryption

### Animation Coordination
- **Command Queue**: Priority queue for animation commands
- **State Sync**: WebSocket for real-time updates to frontend
- **Timing**: `node-cron` for scheduled events

## Data Structures and Interfaces

### Public API
```typescript
export interface GamificationModule {
  // Process state change from Analysis Engine
  processStateChange(event: StateChangeEvent): Promise<void>;
  
  // Get current progress summary
  getProgress(): Promise<ProgressSummary>;
  
  // Manual intervention trigger (for testing)
  triggerIntervention(type: InterventionType): Promise<boolean>;
  
  // Update user preferences
  updatePreferences(preferences: UserPreferences): Promise<void>;
  
  // Get module metrics
  getMetrics(): Promise<GamificationMetrics>;
}

export interface StateChangeEvent {
  previousState: ADHDState;
  currentState: ADHDState;
  transitionTime: Date;
  confidence: number;
  metrics: BehavioralMetrics;
  workContext?: WorkContext;
}

export interface ProgressSummary {
  session: SessionMetrics;
  daily: DailyMetrics;
  allTime: AllTimeMetrics;
  streaks: StreakInfo[];
  achievements: Achievement[];
  wallet: WalletBalance;
}
```

### Reward Models
```typescript
export interface Achievement {
  id: string;
  name: string;
  description: string;
  icon: string;
  rarity: 'common' | 'rare' | 'epic' | 'legendary';
  unlockedAt?: Date;
  progress?: number;  // For progressive achievements
  maxProgress?: number;
  rewards: Reward[];
}

export interface Reward {
  type: 'coins' | 'title' | 'theme' | 'animation';
  amount?: number;
  item?: string;
}

export interface WalletBalance {
  coins: number;
  totalEarned: number;
  totalSpent: number;
  pendingRewards: Reward[];
}
```

### Configuration
```typescript
export interface GamificationConfig {
  // Intervention settings
  intervention: {
    minCooldownMinutes: number;        // Default: 15
    adaptiveCooldown: boolean;         // Default: true
    maxInterventionsPerHour: number;   // Default: 3
    respectFlowStates: boolean;        // Default: true
  };
  
  // Reward settings  
  rewards: {
    coinsPerFocusMinute: number;       // Default: 1
    bonusMultiplier: number;           // Default: 1.5
    achievementCoins: Record<string, number>;
    variableRatioBase: number;         // Default: 0.15
  };
  
  // Progress tracking
  progress: {
    sessionTimeout: number;            // Default: 30 minutes
    streakRequirement: number;         // Default: 3 days
    milestoneThresholds: number[];
  };
  
  // Personality settings
  personality: {
    cheerfulness: number;              // 0-1, default: 0.7
    humor: number;                     // 0-1, default: 0.5
    formality: number;                 // 0-1, default: 0.2
  };
}
```

## Performance Considerations

### Response Time
- **Intervention Decision**: <10ms (all in-memory calculations)
- **Reward Processing**: <20ms including animations
- **Progress Update**: <50ms including persistence
- **Message Generation**: <100ms (may involve AI)

### Memory Usage
- **State History**: Circular buffer of 1000 states (~100KB)
- **Cooldown Tracking**: ~1KB per intervention type
- **Progress Data**: ~10MB for full history
- **Total Module**: <50MB resident memory

### Optimization Strategies
1. **Lazy Evaluation**: Calculate metrics only when needed
2. **Batch Updates**: Group progress updates every 30s
3. **Efficient Storage**: Compress historical data
4. **Caching**: Cache generated messages and animations
5. **Async Operations**: Non-blocking for all I/O

## Error Handling Strategies

### Failure Modes
```typescript
export enum GamificationError {
  // Intervention failures
  InterventionGenerationFailed = 'INTERVENTION_GEN_FAILED',
  CooldownCalculationError = 'COOLDOWN_CALC_ERROR',
  
  // Reward failures  
  RewardGrantFailed = 'REWARD_GRANT_FAILED',
  AchievementUnlockFailed = 'ACHIEVEMENT_UNLOCK_FAILED',
  
  // Progress failures
  ProgressSaveFailed = 'PROGRESS_SAVE_FAILED',
  MetricCalculationError = 'METRIC_CALC_ERROR',
  
  // Animation failures
  AnimationQueueFull = 'ANIMATION_QUEUE_FULL',
  InvalidAnimationCommand = 'INVALID_ANIMATION_CMD'
}
```

### Recovery Strategies
1. **Failed Interventions**: Log and skip, don't block flow
2. **Reward Failures**: Queue for retry, ensure eventual delivery
3. **Progress Loss**: Reconstruct from event history
4. **Animation Errors**: Fall back to default animations
5. **Corrupt State**: Reset to last known good state

### Graceful Degradation
- **No AI Available**: Use template messages
- **Storage Unavailable**: Keep session data in memory
- **Animation Errors**: Show static companion
- **Calculation Errors**: Use conservative defaults

## Security Considerations

### Data Protection
- **Progress Encryption**: Encrypt sensitive progress data
- **Preference Security**: Store preferences encrypted
- **No Cloud Sync**: All data stays local
- **Session Isolation**: Each session isolated

### Input Validation
- **State Validation**: Verify state transitions are valid
- **Metric Bounds**: Check all metrics are in expected ranges
- **Command Validation**: Validate all animation commands
- **Configuration Limits**: Enforce reasonable config bounds

## Testing Strategy

### Unit Tests
- Cooldown calculation logic
- Reward distribution algorithms
- Progress tracking accuracy
- Message generation variety

### Integration Tests
- State change processing flow
- Intervention timing accuracy
- Animation command generation
- Progress persistence

### Behavioral Tests
- User preference learning
- Intervention effectiveness
- Reward schedule adherence
- Personality consistency

### Performance Tests
- Response time under load
- Memory usage over time
- Animation queue handling
- Message generation speed