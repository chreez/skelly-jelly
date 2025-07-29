/**
 * GamificationModule - Main Module Integration
 * 
 * Orchestrates all gamification components and integrates with the event bus
 * to provide non-intrusive motivational support for ADHD users.
 */

import {
  ADHDState,
  BehavioralMetrics,
  StateChangeEvent,
  ProgressSummary,
  RewardEvent,
  InterventionDecision,
  AnimationCommand,
  UserPreferences,
  UserProfile,
  GamificationConfig,
  GamificationMetrics,
  SessionMetrics,
  WorkContext,
  GamificationError,
  ErrorContext
} from './types/index.js';

import { InterventionController } from './controllers/InterventionController.js';
import { RewardSystem } from './systems/RewardSystem.js';
import { ProgressTracker } from './trackers/ProgressTracker.js';
import { CompanionBehaviorManager } from './managers/CompanionBehaviorManager.js';
import { MotivationEngine } from './engines/MotivationEngine.js';

import { Logger } from 'winston';
import { v4 as uuidv4 } from 'uuid';

// Helper function to satisfy UUID type
const createUUID = (): string => uuidv4();

// Event bus integration (would import from actual event-bus module)
export interface EventBusMessage {
  id: string;
  type: string;
  payload: any;
  timestamp: Date;
  source: string;
}

export interface EventBusInterface {
  subscribe(messageType: string, handler: (message: EventBusMessage) => Promise<void>): Promise<string>;
  publish(messageType: string, payload: any): Promise<void>;
  unsubscribe(subscriptionId: string): Promise<void>;
}

export interface GamificationModule {
  // Core operations
  processStateChange(event: StateChangeEvent): Promise<void>;
  getProgress(): Promise<ProgressSummary>;
  triggerIntervention(type: string): Promise<boolean>;
  updatePreferences(preferences: UserPreferences): Promise<void>;
  getMetrics(): Promise<GamificationMetrics>;
  
  // Lifecycle management
  start(): Promise<void>;
  stop(): Promise<void>;
  isRunning(): boolean;
}

export class GamificationModuleImpl implements GamificationModule {
  private interventionController: InterventionController;
  private rewardSystem: RewardSystem;
  private progressTracker: ProgressTracker;
  private companionBehavior: CompanionBehaviorManager;
  private motivationEngine: MotivationEngine;
  
  private eventBus: EventBusInterface;
  private logger: Logger;
  private config: GamificationConfig;
  private userProfile: UserProfile;
  
  private subscriptionIds: string[] = [];
  private isActive = false;
  private lastProcessedState?: ADHDState;
  private errorCount = 0;
  private maxErrorsBeforeRestart = 5;
  
  // Performance metrics
  private metrics: GamificationMetrics = {
    interventionsDelivered: 0,
    interventionEngagementRate: 0,
    rewardsGranted: 0,
    achievementsUnlocked: 0,
    averageResponseTime: 0,
    userSatisfactionScore: 0.5,
    systemPerformance: {
      averageProcessingTime: 0,
      memoryUsage: 0,
      errorRate: 0
    }
  };

  constructor(
    eventBus: EventBusInterface,
    logger: Logger,
    config: GamificationConfig,
    userProfile: UserProfile
  ) {
    this.eventBus = eventBus;
    this.logger = logger;
    this.config = config;
    this.userProfile = userProfile;
    
    // Initialize components
    this.interventionController = new InterventionController(logger, config.performance.maxHistoryEntries);
    this.rewardSystem = new RewardSystem(logger, config);
    this.progressTracker = new ProgressTracker(logger, config);
    this.companionBehavior = new CompanionBehaviorManager(logger, config, userProfile.preferences);
    this.motivationEngine = new MotivationEngine(logger, config);
  }

  /**
   * Start the gamification module and subscribe to events
   */
  async start(): Promise<void> {
    try {
      this.logger.info('Starting Gamification Module', {
        userId: this.userProfile.id,
        interventionFrequency: this.userProfile.preferences.interventionFrequency,
        companionEnabled: this.userProfile.preferences.companionAnimations
      });

      // Subscribe to state change events from analysis engine
      const stateChangeSubscription = await this.eventBus.subscribe(
        'StateChange',
        this.handleStateChangeEvent.bind(this)
      );
      this.subscriptionIds.push(stateChangeSubscription);

      // Subscribe to intervention responses from AI integration
      const responseSubscription = await this.eventBus.subscribe(
        'InterventionResponse',
        this.handleInterventionResponse.bind(this)
      );
      this.subscriptionIds.push(responseSubscription);

      // Subscribe to user interaction events
      const interactionSubscription = await this.eventBus.subscribe(
        'UserInteraction',
        this.handleUserInteraction.bind(this)
      );
      this.subscriptionIds.push(interactionSubscription);

      // Start new session tracking
      this.progressTracker.startNewSession();
      
      this.isActive = true;
      this.logger.info('Gamification Module started successfully');

    } catch (error) {
      this.logger.error('Failed to start Gamification Module', { error });
      throw error;
    }
  }

  /**
   * Stop the gamification module and clean up subscriptions
   */
  async stop(): Promise<void> {
    try {
      this.logger.info('Stopping Gamification Module');

      // Unsubscribe from all events
      for (const subscriptionId of this.subscriptionIds) {
        await this.eventBus.unsubscribe(subscriptionId);
      }
      this.subscriptionIds = [];

      this.isActive = false;
      this.logger.info('Gamification Module stopped');

    } catch (error) {
      this.logger.error('Error stopping Gamification Module', { error });
      throw error;
    }
  }

  /**
   * Check if module is running
   */
  isRunning(): boolean {
    return this.isActive;
  }

  /**
   * Process incoming state change events
   * Core orchestration method that coordinates all gamification responses
   */
  async processStateChange(event: StateChangeEvent): Promise<void> {
    const startTime = Date.now();
    
    try {
      this.logger.debug('Processing state change', {
        from: event.previousState.type,
        to: event.currentState.type,
        confidence: event.confidence
      });

      // Add state to intervention controller history
      this.interventionController.addStateToHistory(
        event.currentState,
        event.metrics,
        event.workContext
      );

      // Update progress tracking
      const progressUpdate = await this.progressTracker.updateProgress(
        event.currentState,
        event.metrics,
        event.transitionTime,
        event.workContext
      );

      // Process rewards for state transition
      const rewardEvents = await this.rewardSystem.processStateChange(
        event.previousState,
        event.currentState,
        event.currentState.duration || 0,
        event.metrics,
        progressUpdate.session,
        this.userProfile.preferences
      );

      // Update metrics
      this.metrics.rewardsGranted += rewardEvents.length;
      this.metrics.achievementsUnlocked += rewardEvents.filter(r => r.type === 'achievement').length;

      // Evaluate intervention opportunity
      const interventionDecision = await this.interventionController.shouldIntervene(
        event.currentState,
        event.metrics,
        event.workContext || this.createDefaultWorkContext(),
        this.userProfile.preferences,
        progressUpdate.session
      );

      // Generate companion animations
      const animationCommands = await this.companionBehavior.updateCompanionState(
        event.currentState,
        rewardEvents,
        interventionDecision,
        event.metrics
      );

      // Send animation commands to cute-figurine module
      for (const command of animationCommands) {
        await this.eventBus.publish('AnimationCommand', command);
      }

      // Handle intervention if approved
      if (interventionDecision.intervene && interventionDecision.type) {
        await this.executeIntervention(interventionDecision, event, progressUpdate.session);
      }

      // Publish reward events
      for (const reward of rewardEvents) {
        await this.eventBus.publish('RewardEvent', reward);
      }

      // Update performance metrics
      this.updatePerformanceMetrics(Date.now() - startTime);
      this.lastProcessedState = event.currentState;
      this.errorCount = 0; // Reset error count on successful processing

      this.logger.debug('State change processed successfully', {
        processingTime: Date.now() - startTime,
        rewardsGranted: rewardEvents.length,
        interventionTriggered: interventionDecision.intervene,
        animationCommands: animationCommands.length
      });

    } catch (error) {
      this.handleProcessingError(error, event);
    }
  }

  /**
   * Get current progress summary
   */
  async getProgress(): Promise<ProgressSummary> {
    try {
      const session = this.progressTracker.getCurrentSession();
      const dailyMetrics = this.progressTracker.getDailyMetrics(new Date());
      const allTimeMetrics = this.progressTracker.getAllTimeMetrics();
      const streaks = this.progressTracker.getStreaks();
      const achievements = this.rewardSystem.getUnlockedAchievements();
      const wallet = this.rewardSystem.getWallet();

      // Generate trends analysis
      const trends = {
        focusTrend: 'stable' as const,
        productivityTrend: 'stable' as const,
        consistencyTrend: 'stable' as const,
        timespan: 'weekly' as const,
        confidence: 0.7,
        recommendations: this.progressTracker.getInsights()
      };

      const levelProgress = {
        currentLevel: allTimeMetrics.currentLevel,
        previousLevel: allTimeMetrics.currentLevel,
        experiencePoints: 0, // Would be calculated
        experienceToNext: 100,
        rewards: []
      };

      return {
        session,
        daily: dailyMetrics || this.createEmptyDailyMetrics(),
        allTime: allTimeMetrics,
        streaks,
        achievements,
        wallet,
        level: levelProgress,
        trends
      };

    } catch (error) {
      this.logger.error('Error getting progress summary', { error });
      throw error;
    }
  }

  /**
   * Manually trigger an intervention (for testing or special events)
   */
  async triggerIntervention(type: string): Promise<boolean> {
    try {
      if (!this.lastProcessedState) {
        this.logger.warn('Cannot trigger intervention - no state history');
        return false;
      }

      const decision: InterventionDecision = {
        intervene: true,
        type: {
          id: type,
          category: 'encouragement',
          minCooldown: 0,
          adaptiveCooldown: false,
          requiredState: ['Flow', 'Neutral', 'Distracted', 'Transitioning', 'Hyperfocus'],
          maxPerHour: 10,
          respectsFlow: false
        },
        confidence: 1.0,
        reason: 'Manual trigger',
        message: 'Manual intervention triggered'
      };

      const sessionMetrics = this.progressTracker.getCurrentSession();
      const fakeEvent: StateChangeEvent = {
        previousState: this.lastProcessedState,
        currentState: this.lastProcessedState,
        transitionTime: new Date(),
        confidence: 1.0,
        metrics: {
          productive_time_ratio: 0.7,
          distraction_frequency: 2,
          focus_session_count: 3,
          average_session_length: 25,
          recovery_time: 120,
          transition_smoothness: 0.8
        },
        sessionId: sessionMetrics.sessionId
      };

      await this.executeIntervention(decision, fakeEvent, sessionMetrics);
      
      this.logger.info('Manual intervention triggered', { type });
      return true;

    } catch (error) {
      this.logger.error('Error triggering manual intervention', { error, type });
      return false;
    }
  }

  /**
   * Update user preferences
   */
  async updatePreferences(preferences: UserPreferences): Promise<void> {
    try {
      this.userProfile.preferences = { ...this.userProfile.preferences, ...preferences };
      
      this.logger.info('User preferences updated', {
        interventionFrequency: preferences.interventionFrequency,
        companionAnimations: preferences.companionAnimations,
        soundEnabled: preferences.soundEnabled
      });

    } catch (error) {
      this.logger.error('Error updating preferences', { error });
      throw error;
    }
  }

  /**
   * Get current module metrics
   */
  async getMetrics(): Promise<GamificationMetrics> {
    try {
      // Update system performance metrics
      this.metrics.systemPerformance.memoryUsage = process.memoryUsage().heapUsed / 1024 / 1024; // MB
      this.metrics.systemPerformance.errorRate = this.errorCount / Math.max(this.metrics.interventionsDelivered, 1);

      return { ...this.metrics };

    } catch (error) {
      this.logger.error('Error getting metrics', { error });
      throw error;
    }
  }

  // === Private Helper Methods ===

  private async handleStateChangeEvent(message: EventBusMessage): Promise<void> {
    try {
      const event = message.payload as StateChangeEvent;
      await this.processStateChange(event);
    } catch (error) {
      this.logger.error('Error handling state change event', { error, messageId: message.id });
    }
  }

  private async handleInterventionResponse(message: EventBusMessage): Promise<void> {
    try {
      const response = message.payload as { 
        requestId: string; 
        userResponse: 'positive' | 'negative' | 'neutral';
        interventionId: string;
      };

      // Record response for adaptive learning
      this.interventionController.recordInterventionResponse(
        response.interventionId,
        response.userResponse === 'positive' ? 'engaged_positively' :
        response.userResponse === 'negative' ? 'dismissed_quickly' : 'ignored'
      );

      // Update engagement metrics
      this.updateEngagementMetrics(response.userResponse);

      this.logger.debug('Intervention response recorded', {
        requestId: response.requestId,
        response: response.userResponse
      });

    } catch (error) {
      this.logger.error('Error handling intervention response', { error });
    }
  }

  private async handleUserInteraction(message: EventBusMessage): Promise<void> {
    try {
      const interaction = message.payload as {
        type: 'click' | 'dismiss' | 'engage';
        target: 'companion' | 'message' | 'reward';
        timestamp: Date;
      };

      // Update companion state based on interaction
      if (interaction.target === 'companion') {
        await this.companionBehavior.triggerReaction('excitement', 'medium');
      }

      this.logger.debug('User interaction recorded', {
        type: interaction.type,
        target: interaction.target
      });

    } catch (error) {
      this.logger.error('Error handling user interaction', { error });
    }
  }

  private async executeIntervention(
    decision: InterventionDecision,
    event: StateChangeEvent,
    sessionMetrics: SessionMetrics
  ): Promise<void> {
    try {
      // Generate motivational message
      const message = await this.motivationEngine.generateMessage({
        currentState: event.currentState,
        previousState: event.previousState,
        metrics: event.metrics,
        workContext: event.workContext || this.createDefaultWorkContext(),
        interventionType: decision.type!.id,
        userProfile: this.userProfile,
        sessionMetrics
      });

      // Create intervention request for AI integration
      const interventionRequest = {
        requestId: createUUID(),
        interventionType: decision.type!.id,
        message: message.text,
        urgency: this.mapPriorityToUrgency(decision.type!.category),
        context: {
          userState: event.currentState.type,
          confidence: decision.confidence,
          sessionId: sessionMetrics.sessionId,
          workContext: event.workContext
        },
        displayDuration: message.duration,
        style: message.style,
        actionButton: message.actionButton
      };

      // Send to AI integration module
      await this.eventBus.publish('InterventionRequest', interventionRequest);

      // Update metrics
      this.metrics.interventionsDelivered++;
      
      this.logger.info('Intervention executed', {
        type: decision.type!.id,
        requestId: interventionRequest.requestId,
        confidence: decision.confidence
      });

    } catch (error) {
      this.logger.error('Error executing intervention', { error });
      throw error;
    }
  }

  private createDefaultWorkContext(): WorkContext {
    return {
      application: 'unknown',
      window_title: 'unknown',
      task_category: 'unknown',
      urgency: 'medium',
      time_of_day: this.getTimeOfDay()
    };
  }

  private createEmptyDailyMetrics() {
    return {
      date: new Date(),
      totalFocusTime: 0,
      sessionCount: 0,
      averageSessionLength: 0,
      bestFlowSession: null,
      interventionResponseRate: 0,
      productivityTrend: 'stable' as const,
      streakDays: 0
    };
  }

  private getTimeOfDay(): WorkContext['time_of_day'] {
    const hour = new Date().getHours();
    if (hour >= 5 && hour < 12) return 'morning';
    if (hour >= 12 && hour < 17) return 'afternoon';
    if (hour >= 17 && hour < 21) return 'evening';
    return 'night';
  }

  private mapPriorityToUrgency(category: string): string {
    const urgencyMap = {
      celebration: 'high',
      milestone: 'high',
      encouragement: 'medium',
      suggestion: 'medium',
      gentle_nudge: 'low'
    };
    
    return urgencyMap[category as keyof typeof urgencyMap] || 'medium';
  }

  private updatePerformanceMetrics(processingTime: number): void {
    // Update rolling average of processing time
    this.metrics.systemPerformance.averageProcessingTime = 
      (this.metrics.systemPerformance.averageProcessingTime * 0.9) + (processingTime * 0.1);
  }

  private updateEngagementMetrics(response: 'positive' | 'negative' | 'neutral'): void {
    const engagementValue = response === 'positive' ? 1 : response === 'negative' ? 0 : 0.5;
    
    // Update rolling average
    this.metrics.interventionEngagementRate = 
      (this.metrics.interventionEngagementRate * 0.9) + (engagementValue * 0.1);
    
    // Update satisfaction score
    this.metrics.userSatisfactionScore = 
      (this.metrics.userSatisfactionScore * 0.95) + (engagementValue * 0.05);
  }

  private handleProcessingError(error: unknown, event: StateChangeEvent): void {
    this.errorCount++;
    
    const errorContext: ErrorContext = {
      errorCode: GamificationError.MetricCalculationError,
      message: error instanceof Error ? error.message : 'Unknown error',
      timestamp: new Date(),
      context: {
        eventType: 'StateChange',
        currentState: event.currentState.type,
        previousState: event.previousState.type,
        errorCount: this.errorCount
      },
      recoverable: this.errorCount < this.maxErrorsBeforeRestart
    };

    this.logger.error('Error processing state change', errorContext);

    // If too many errors, suggest restart
    if (this.errorCount >= this.maxErrorsBeforeRestart) {
      this.logger.error('Too many errors - module restart recommended', {
        errorCount: this.errorCount,
        maxErrors: this.maxErrorsBeforeRestart
      });
      
      // Could trigger automatic restart or notify orchestrator
      this.eventBus.publish('Error', {
        module: 'gamification',
        error: errorContext,
        severity: 'critical',
        actionRequired: 'restart'
      }).catch(err => {
        this.logger.error('Failed to publish error event', { err });
      });
    }

    // Update error rate
    this.metrics.systemPerformance.errorRate = this.errorCount / Math.max(this.metrics.interventionsDelivered, 1);
  }
}

/**
 * Factory function to create and configure the gamification module
 */
export function createGamificationModule(
  eventBus: EventBusInterface,
  logger: Logger,
  config?: Partial<GamificationConfig>,
  userProfile?: Partial<UserProfile>
): GamificationModule {
  // Default configuration
  const defaultConfig: GamificationConfig = {
    intervention: {
      minCooldownMinutes: 15,
      adaptiveCooldown: true,
      maxInterventionsPerHour: 3,
      respectFlowStates: true,
      flowStateThreshold: 0.8,
      emergencyOverride: false
    },
    rewards: {
      coinsPerFocusMinute: 1,
      bonusMultiplier: 1.5,
      achievementCoins: {
        common: 25,
        rare: 100,
        epic: 250,
        legendary: 500
      },
      variableRatioBase: 0.15,
      streakBonusMultiplier: 1.2,
      milestoneRewards: {
        first_hour: 50,
        daily_goal: 100,
        weekly_streak: 200
      }
    },
    progress: {
      sessionTimeoutMinutes: 30,
      streakRequirementDays: 3,
      milestoneThresholds: [1, 5, 10, 25, 50, 100],
      metricUpdateInterval: 30,
      historyRetentionDays: 90
    },
    companion: {
      animationDuration: 3000,
      expressionVariety: true,
      idleVariations: true,
      reactionSensitivity: 0.7,
      personalityTraits: {
        cheerfulness: 0.7,
        humor: 0.5,
        formality: 0.2,
        supportiveness: 0.8
      }
    },
    messages: {
      maxLength: 150,
      personalizedGeneration: true,
      templateVariety: true,
      adaptiveTone: true,
      contextAwareness: true
    },
    performance: {
      maxHistoryEntries: 1000,
      batchUpdateSize: 10,
      cacheTimeout: 300,
      animationQueueSize: 20
    }
  };

  // Default user profile
  const defaultUserProfile: UserProfile = {
    id: createUUID(),
    preferences: {
      interventionFrequency: 'moderate',
      interventionTypes: ['encouragement', 'suggestion', 'celebration', 'gentle_nudge'],
      respectFlowStates: true,
      customCooldowns: {},
      soundEnabled: true,
      visualNotifications: true,
      notificationPosition: 'companion',
      animationIntensity: 'moderate',
      rewardTypes: ['coins', 'achievements', 'milestones', 'celebrations'],
      celebrationStyle: 'noticeable',
      progressVisibility: 'always',
      messageStyle: 'encouraging',
      personalizedMessages: true,
      messageFrequency: 'medium',
      companionPersonality: 'supportive',
      companionAnimations: true,
      companionSounds: true
    },
    statistics: {
      totalUsageTime: 0,
      averageSessionLength: 0,
      preferredWorkingHours: [9, 17],
      mostProductiveTimeSlots: ['morning'],
      responsePatterns: {
        encouragement: [],
        suggestion: [],
        celebration: [],
        gentle_nudge: [],
        milestone: []
      },
      improvementMetrics: {
        focusImprovement: 0,
        distractionReduction: 0,
        recoveryTimeImprovement: 0
      }
    },
    personalityProfile: {
      motivationStyle: 'achievement',
      feedbackPreference: 'immediate',
      challengeLevel: 'moderate',
      gamificationElements: ['progress_bars', 'achievements', 'levels', 'streaks'],
      communicationStyle: 'encouraging'
    },
    accessibilityNeeds: {
      reducedMotion: false,
      highContrast: false,
      largeText: false,
      screenReaderCompatible: false
    }
  };

  // Merge configurations
  const finalConfig = { ...defaultConfig, ...config };
  const finalUserProfile = { ...defaultUserProfile, ...userProfile };

  return new GamificationModuleImpl(eventBus, logger, finalConfig, finalUserProfile);
}