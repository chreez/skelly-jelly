/**
 * InterventionController - User-Centered Intervention Timing
 * 
 * Manages when and how to provide support without disrupting user flow.
 * Core principle: Respect user attention and never interrupt deep focus.
 */

import { 
  ADHDState, 
  BehavioralMetrics, 
  WorkContext, 
  InterventionDecision, 
  InterventionType, 
  UserResponse, 
  UserPreferences,
  SessionMetrics,
  GamificationError
} from '../types/index.js';
import { Logger } from 'winston';
import { addMinutes, isAfter, differenceInMinutes } from 'date-fns';

export interface CooldownTracker {
  lastTriggered: Map<string, Date>;
  cooldownMultipliers: Map<string, number>;
  userResponseHistory: Map<string, UserResponse[]>;
}

export interface EffectivenessTracker {
  interventionSuccessRate: Map<string, number>;
  userEngagementMetrics: Map<string, number>;
  contextualSuccess: Map<string, number>;
}

export interface CircularBuffer<T> {
  data: T[];
  maxSize: number;
  currentIndex: number;
  size: number;
}

export interface StateSnapshot {
  state: ADHDState;
  metrics: BehavioralMetrics;
  timestamp: Date;
  workContext?: WorkContext;
}

export class InterventionController {
  private cooldowns: CooldownTracker;
  private stateHistory: CircularBuffer<StateSnapshot>;
  private effectivenessTracker: EffectivenessTracker;
  private logger: Logger;
  
  // Built-in intervention types focused on user support
  private readonly interventionTypes: Map<string, InterventionType> = new Map([
    ['gentle_encouragement', {
      id: 'gentle_encouragement',
      category: 'encouragement',
      minCooldown: 20, // 20 minutes minimum
      adaptiveCooldown: true,
      requiredState: ['Flow', 'Neutral'],
      maxPerHour: 2,
      respectsFlow: true
    }],
    ['recovery_support', {
      id: 'recovery_support',
      category: 'suggestion',
      minCooldown: 15,
      adaptiveCooldown: true,
      requiredState: ['Distracted', 'Transitioning'],
      maxPerHour: 3,
      respectsFlow: true
    }],
    ['milestone_celebration', {
      id: 'milestone_celebration',
      category: 'celebration',
      minCooldown: 5, // Celebrations can be more frequent
      adaptiveCooldown: false,
      requiredState: ['Flow', 'Neutral', 'Transitioning'],
      maxPerHour: 5,
      respectsFlow: false // Celebrations are worth brief interruption
    }],
    ['gentle_nudge', {
      id: 'gentle_nudge',
      category: 'gentle_nudge',
      minCooldown: 30,
      adaptiveCooldown: true,
      requiredState: ['Distracted'],
      maxPerHour: 2,
      respectsFlow: true
    }],
    ['break_suggestion', {
      id: 'break_suggestion',
      category: 'suggestion',
      minCooldown: 45,
      adaptiveCooldown: true,
      requiredState: ['Hyperfocus'],
      maxPerHour: 1,
      respectsFlow: false // Hyperfocus breaks are important for health
    }]
  ]);

  constructor(logger: Logger, historySize: number = 100) {
    this.logger = logger;
    this.cooldowns = {
      lastTriggered: new Map(),
      cooldownMultipliers: new Map(),
      userResponseHistory: new Map()
    };
    
    this.stateHistory = {
      data: [],
      maxSize: historySize,
      currentIndex: 0,
      size: 0
    };
    
    this.effectivenessTracker = {
      interventionSuccessRate: new Map(),
      userEngagementMetrics: new Map(),
      contextualSuccess: new Map()
    };

    this.initializeInterventionTypes();
  }

  /**
   * Main decision point: Should we intervene right now?
   * Prioritizes user experience and flow state protection.
   */
  async shouldIntervene(
    currentState: ADHDState,
    metrics: BehavioralMetrics,
    context: WorkContext,
    userPreferences: UserPreferences,
    sessionMetrics: SessionMetrics
  ): Promise<InterventionDecision> {
    try {
      // NEVER interrupt flow states with high confidence
      if (this.isInProtectedFlowState(currentState, userPreferences)) {
        return { 
          intervene: false, 
          reason: 'Protecting user flow state',
          confidence: currentState.confidence 
        };
      }

      // Check if user is in a critical work context
      if (this.isInCriticalContext(context)) {
        return { 
          intervene: false, 
          reason: 'Critical work context detected' 
        };
      }

      // Get available interventions (not on cooldown)
      const availableInterventions = this.getAvailableInterventions(userPreferences);
      if (availableInterventions.length === 0) {
        return { 
          intervene: false, 
          reason: 'All appropriate interventions on cooldown' 
        };
      }

      // Check hourly intervention limits
      if (this.hasExceededHourlyLimits(sessionMetrics)) {
        return { 
          intervene: false, 
          reason: 'Hourly intervention limit reached' 
        };
      }

      // Evaluate intervention opportunity
      const opportunity = this.evaluateInterventionOpportunity(
        currentState, 
        metrics, 
        context, 
        availableInterventions,
        userPreferences
      );

      if (!opportunity.bestIntervention) {
        return { 
          intervene: false, 
          reason: 'No suitable intervention found' 
        };
      }

      // Use adaptive threshold based on user preferences
      const threshold = this.getAdaptiveThreshold(userPreferences);
      
      if (opportunity.score >= threshold) {
        this.logger.info('Intervention approved', {
          type: opportunity.bestIntervention.id,
          score: opportunity.score,
          threshold,
          state: currentState.type,
          confidence: opportunity.confidence
        });

        return {
          intervene: true,
          type: opportunity.bestIntervention,
          confidence: opportunity.confidence,
          message: opportunity.suggestedMessage,
          reason: `High-value intervention opportunity (score: ${opportunity.score.toFixed(2)})`
        };
      }

      return { 
        intervene: false, 
        reason: `Opportunity score ${opportunity.score.toFixed(2)} below threshold ${threshold.toFixed(2)}` 
      };

    } catch (error) {
      this.logger.error('Error in intervention decision', { error, state: currentState.type });
      return { 
        intervene: false, 
        reason: 'Error in intervention evaluation' 
      };
    }
  }

  /**
   * Record user response to intervention for adaptive learning
   */
  recordInterventionResponse(
    interventionId: string, 
    response: UserResponse, 
    context?: Record<string, unknown>
  ): void {
    try {
      const intervention = this.interventionTypes.get(interventionId);
      if (!intervention) {
        this.logger.warn('Unknown intervention ID for response recording', { interventionId });
        return;
      }

      // Update cooldown based on response
      this.updateCooldownBasedOnResponse(intervention, response);
      
      // Track response history
      const history = this.cooldowns.userResponseHistory.get(interventionId) || [];
      history.push(response);
      // Keep only recent history (last 20 responses)
      if (history.length > 20) {
        history.shift();
      }
      this.cooldowns.userResponseHistory.set(interventionId, history);

      // Update effectiveness metrics
      this.updateEffectivenessMetrics(interventionId, response, context);

      this.logger.info('Intervention response recorded', {
        interventionId,
        response,
        cooldownMultiplier: this.cooldowns.cooldownMultipliers.get(interventionId) || 1
      });

    } catch (error) {
      this.logger.error('Error recording intervention response', { error, interventionId, response });
    }
  }

  /**
   * Add state to history for pattern analysis
   */
  addStateToHistory(
    state: ADHDState, 
    metrics: BehavioralMetrics, 
    context?: WorkContext
  ): void {
    const snapshot: StateSnapshot = {
      state,
      metrics,
      timestamp: new Date(),
      workContext: context
    };

    this.stateHistory.data[this.stateHistory.currentIndex] = snapshot;
    this.stateHistory.currentIndex = (this.stateHistory.currentIndex + 1) % this.stateHistory.maxSize;
    this.stateHistory.size = Math.min(this.stateHistory.size + 1, this.stateHistory.maxSize);
  }

  /**
   * Get intervention effectiveness stats for monitoring
   */
  getEffectivenessStats(): Record<string, unknown> {
    const stats: Record<string, unknown> = {};
    
    for (const [interventionId] of this.interventionTypes) {
      const successRate = this.effectivenessTracker.interventionSuccessRate.get(interventionId) || 0;
      const engagementRate = this.effectivenessTracker.userEngagementMetrics.get(interventionId) || 0;
      const cooldownMultiplier = this.cooldowns.cooldownMultipliers.get(interventionId) || 1;
      
      stats[interventionId] = {
        successRate,
        engagementRate,
        cooldownMultiplier,
        isAdaptivelyReduced: cooldownMultiplier > 1.5
      };
    }
    
    return stats;
  }

  // === Private Helper Methods ===

  private isInProtectedFlowState(state: ADHDState, preferences: UserPreferences): boolean {
    if (!preferences.respectFlowStates) {
      return false;
    }

    // Protect flow states with high confidence
    if (state.type === 'Flow' && state.confidence > 0.8) {
      return true;
    }

    // Protect hyperfocus if it's been less than 90 minutes (prevent burnout while respecting focus)
    if (state.type === 'Hyperfocus' && state.duration < 90 * 60 * 1000) {
      return true;
    }

    return false;
  }

  private isInCriticalContext(context: WorkContext): boolean {
    return context.urgency === 'critical' || 
           (context.urgency === 'high' && context.task_category === 'work');
  }

  private getAvailableInterventions(preferences: UserPreferences): InterventionType[] {
    const available: InterventionType[] = [];
    
    for (const [id, intervention] of this.interventionTypes) {
      // Check if user has enabled this intervention category
      if (!preferences.interventionTypes.includes(intervention.category)) {
        continue;
      }

      // Check cooldown
      if (!this.canTriggerIntervention(intervention)) {
        continue;
      }

      available.push(intervention);
    }

    return available;
  }

  private canTriggerIntervention(intervention: InterventionType): boolean {
    const lastTriggered = this.cooldowns.lastTriggered.get(intervention.id);
    if (!lastTriggered) {
      return true;
    }

    const multiplier = this.cooldowns.cooldownMultipliers.get(intervention.id) || 1;
    const cooldownMinutes = intervention.minCooldown * multiplier;
    const nextAvailable = addMinutes(lastTriggered, cooldownMinutes);

    return isAfter(new Date(), nextAvailable);
  }

  private hasExceededHourlyLimits(sessionMetrics: SessionMetrics): boolean {
    const oneHourAgo = new Date(Date.now() - 60 * 60 * 1000);
    return sessionMetrics.interventionsReceived >= 5; // Global hourly limit
  }

  private evaluateInterventionOpportunity(
    state: ADHDState,
    metrics: BehavioralMetrics,
    context: WorkContext,
    availableInterventions: InterventionType[],
    preferences: UserPreferences
  ): { score: number; bestIntervention?: InterventionType; confidence: number; suggestedMessage: string } {
    let bestScore = 0;
    let bestIntervention: InterventionType | undefined = undefined;
    let bestMessage = '';

    for (const intervention of availableInterventions) {
      const score = this.calculateInterventionScore(intervention, state, metrics, context, preferences);
      
      if (score > bestScore) {
        bestScore = score;
        bestIntervention = intervention;
        bestMessage = this.generateContextualMessage(intervention, state, metrics);
      }
    }

    return {
      score: bestScore,
      bestIntervention,
      confidence: Math.min(bestScore, 1.0),
      suggestedMessage: bestMessage
    };
  }

  private calculateInterventionScore(
    intervention: InterventionType,
    state: ADHDState,
    metrics: BehavioralMetrics,
    context: WorkContext,
    preferences: UserPreferences
  ): number {
    let score = 0;

    // Base score for state appropriateness
    if (intervention.requiredState.includes(state.type)) {
      score += 0.3;
    } else {
      return 0; // Invalid state for this intervention
    }

    // Boost score based on intervention effectiveness history
    const historicalSuccess = this.effectivenessTracker.interventionSuccessRate.get(intervention.id) || 0.5;
    score += historicalSuccess * 0.3;

    // Context-specific scoring
    switch (intervention.category) {
      case 'encouragement':
        // More valuable during moderate flow or after recovery
        if (state.type === 'Flow' && state.confidence > 0.6) {
          score += 0.2;
        }
        break;

      case 'suggestion':
        // More valuable during distraction or low productivity
        if (state.type === 'Distracted' && metrics.productive_time_ratio < 0.5) {
          score += 0.3;
        }
        break;

      case 'celebration':
        // High value for achievements and milestones
        score += 0.4; // Celebrations are almost always valuable
        break;

      case 'gentle_nudge':
        // Valuable for prolonged distraction
        if (state.type === 'Distracted' && state.duration > 10 * 60 * 1000) {
          score += 0.3;
        }
        break;
    }

    // Time-of-day adjustments
    const hour = new Date().getHours();
    if (context.time_of_day === 'morning' && intervention.category === 'encouragement') {
      score += 0.1; // Morning encouragement is often well-received
    }
    if (context.time_of_day === 'evening' && intervention.category === 'celebration') {
      score += 0.1; // End-of-day celebrations feel natural
    }

    // User preference adjustments
    if (preferences.interventionFrequency === 'minimal') {
      score *= 0.7; // Reduce score for minimal preference users
    } else if (preferences.interventionFrequency === 'frequent') {
      score *= 1.2; // Boost for users who want more interventions
    }

    return Math.min(score, 1.0);
  }

  private getAdaptiveThreshold(preferences: UserPreferences): number {
    const baseThreshold = 0.6;

    switch (preferences.interventionFrequency) {
      case 'minimal':
        return baseThreshold + 0.2; // Higher threshold = fewer interventions
      case 'frequent':
        return baseThreshold - 0.1; // Lower threshold = more interventions
      default:
        return baseThreshold;
    }
  }

  private updateCooldownBasedOnResponse(intervention: InterventionType, response: UserResponse): void {
    if (!intervention.adaptiveCooldown) {
      return;
    }

    const currentMultiplier = this.cooldowns.cooldownMultipliers.get(intervention.id) || 1;
    let newMultiplier = currentMultiplier;

    switch (response) {
      case 'dismissed_quickly':
        newMultiplier = Math.min(currentMultiplier * 1.3, 3.0); // Increase cooldown, max 3x
        break;
      case 'ignored':
        newMultiplier = Math.min(currentMultiplier * 1.2, 2.5);
        break;
      case 'engaged_positively':
      case 'acted_upon':
        newMultiplier = Math.max(currentMultiplier * 0.9, 0.5); // Decrease cooldown, min 0.5x
        break;
      case 'clicked_through':
        newMultiplier = Math.max(currentMultiplier * 0.95, 0.8);
        break;
    }

    this.cooldowns.cooldownMultipliers.set(intervention.id, newMultiplier);
    this.cooldowns.lastTriggered.set(intervention.id, new Date());
  }

  private updateEffectivenessMetrics(
    interventionId: string, 
    response: UserResponse, 
    context?: Record<string, unknown>
  ): void {
    // Calculate success (positive responses)
    const isSuccess = ['engaged_positively', 'acted_upon', 'clicked_through'].includes(response);
    
    const currentSuccess = this.effectivenessTracker.interventionSuccessRate.get(interventionId) || 0.5;
    const newSuccess = (currentSuccess * 0.9) + (isSuccess ? 0.1 : 0); // Exponential moving average
    
    this.effectivenessTracker.interventionSuccessRate.set(interventionId, newSuccess);

    // Track engagement (any interaction)
    const isEngagement = response !== 'ignored';
    const currentEngagement = this.effectivenessTracker.userEngagementMetrics.get(interventionId) || 0.5;
    const newEngagement = (currentEngagement * 0.9) + (isEngagement ? 0.1 : 0);
    
    this.effectivenessTracker.userEngagementMetrics.set(interventionId, newEngagement);
  }

  private generateContextualMessage(
    intervention: InterventionType, 
    state: ADHDState, 
    metrics: BehavioralMetrics
  ): string {
    // Simple message generation - in a full implementation, this would use the MotivationEngine
    const templates = this.getMessageTemplates();
    
    switch (intervention.category) {
      case 'encouragement':
        if (state.type === 'Flow') {
          return this.randomChoice(templates.encouragement.flow_entry) || 'Great focus!';
        }
        return this.randomChoice(templates.encouragement.sustained_focus) || 'Keep it up!';

      case 'suggestion':
        if (state.type === 'Distracted') {
          return this.randomChoice(templates.gentle_nudge.distraction_detected) || 'Ready to refocus?';
        }
        return this.randomChoice(templates.gentle_nudge.refocus_tip) || 'Try a fresh approach!';

      case 'celebration':
        return this.randomChoice(templates.celebration.milestone_reached) || 'Well done!';

      case 'gentle_nudge':
        return this.randomChoice(templates.gentle_nudge.distraction_detected) || 'Everything okay?';

      default:
        return 'Keep up the great work!';
    }
  }

  private getMessageTemplates() {
    return {
      encouragement: {
        flow_entry: [
          "Nice! You're getting into the zone 🎯",
          "Feel that focus building? Keep it up!",
          "Your concentration is looking great right now"
        ],
        sustained_focus: [
          "You've been doing great for a while now!",
          "Solid focus session happening here 💪",
          "Your productivity is on point today!"
        ]
      },
      gentle_nudge: {
        distraction_detected: [
          "Hey, everything okay over there?",
          "Need a quick reset? That's totally fine!",
          "Your skeleton friend noticed you might be stuck"
        ],
        refocus_tip: [
          "Maybe try a different approach?",
          "Sometimes a fresh start helps!",
          "No judgment - we all get sidetracked sometimes"
        ]
      },
      celebration: {
        milestone_reached: [
          "🎉 Amazing progress! You're doing great!",
          "NEW ACHIEVEMENT unlocked! Well done!",
          "Skeleton dance party! You're crushing it! 🦴💃"
        ]
      }
    };
  }

  private randomChoice<T>(array: T[]): T | undefined {
    if (array.length === 0) {
      return undefined;
    }
    return array[Math.floor(Math.random() * array.length)];
  }

  private initializeInterventionTypes(): void {
    // Set initial effectiveness baselines
    for (const [id] of this.interventionTypes) {
      this.effectivenessTracker.interventionSuccessRate.set(id, 0.5);
      this.effectivenessTracker.userEngagementMetrics.set(id, 0.5);
      this.cooldowns.cooldownMultipliers.set(id, 1.0);
    }
  }
}