/**
 * CompanionBehaviorManager - Skeleton Animation Coordination
 * 
 * Manages the melty skeleton companion's behavior and animations
 * to provide ambient support that enhances rather than distracts from user focus.
 */

import {
  ADHDState,
  BehavioralMetrics,
  RewardEvent,
  InterventionDecision,
  AnimationCommand,
  Expression,
  GlowEffect,
  ParticleEffect,
  SoundEffect,
  UserPreferences,
  GamificationConfig
} from '../types/index.js';
import { Logger } from 'winston';
import { v4 as uuidv4 } from 'uuid';

// Helper function to satisfy UUID type
const createUUID = (): string => uuidv4();
import { createMachine, interpret, assign } from 'xstate';

export interface CompanionState {
  currentAnimation: string;
  currentExpression: Expression;
  mood: 'supportive' | 'excited' | 'calm' | 'concerned' | 'proud';
  energy: number; // 0-1, affects animation intensity
  lastInteraction: Date;
  isVisible: boolean;
}

export interface AnimationQueue {
  commands: PriorityAnimationCommand[];
  maxSize: number;
  isProcessing: boolean;
}

interface PriorityAnimationCommand extends AnimationCommand {
  queueTime: Date;
  executionTime?: Date;
}

export class CompanionBehaviorManager {
  private animationStateMachine: any;
  private animationQueue: AnimationQueue;
  private companionState: CompanionState;
  private personalityEngine: PersonalityEngine;
  private logger: Logger;
  private config: GamificationConfig;

  constructor(logger: Logger, config: GamificationConfig, userPreferences: UserPreferences) {
    this.logger = logger;
    this.config = config;
    
    this.companionState = {
      currentAnimation: 'idle_breathing',
      currentExpression: {
        type: 'neutral',
        intensity: 0.5,
        duration: 5000,
        eyeState: 'normal',
        mouthState: 'neutral'
      },
      mood: 'supportive',
      energy: 0.6,
      lastInteraction: new Date(),
      isVisible: true
    };
    
    this.animationQueue = {
      commands: [],
      maxSize: 10,
      isProcessing: false
    };
    
    this.personalityEngine = new PersonalityEngine(config.companion.personalityTraits, userPreferences);
    this.initializeStateMachine();
  }

  /**
   * Update companion state based on user state and events
   * Focus on subtle, supportive presence that doesn't interrupt
   */
  async updateCompanionState(
    userState: ADHDState,
    rewardEvents: RewardEvent[],
    intervention?: InterventionDecision,
    metrics?: BehavioralMetrics
  ): Promise<AnimationCommand[]> {
    const commands: AnimationCommand[] = [];

    try {
      // Update companion mood based on user state
      this.updateCompanionMood(userState, metrics);
      
      // Generate base state animation
      const baseAnimation = this.getBaseStateAnimation(userState);
      commands.push(baseAnimation);
      
      // Add expression overlay
      const expression = this.generateExpression(userState, this.companionState.mood);
      const expressionCommand = this.createExpressionCommand(expression);
      commands.push(expressionCommand);
      
      // Process reward events
      for (const reward of rewardEvents) {
        const rewardAnimation = this.createRewardAnimation(reward);
        if (rewardAnimation) {
          commands.push(rewardAnimation);
        }
      }
      
      // Handle intervention animation
      if (intervention?.intervene) {
        const interventionAnimation = this.createInterventionAnimation(intervention);
        if (interventionAnimation) {
          commands.push(interventionAnimation);
        }
      }
      
      // Add personality modifiers
      const personalizedCommands = this.personalityEngine.modifyAnimations(commands, userState);
      
      // Queue commands with priority management
      await this.queueAnimations(personalizedCommands);
      
      // Generate idle variations if companion has been static
      if (this.shouldGenerateIdleVariation()) {
        const idleVariation = this.generateIdleVariation();
        commands.push(idleVariation);
      }

      this.logger.debug('Companion state updated', {
        userState: userState.type,
        mood: this.companionState.mood,
        energy: this.companionState.energy,
        commandCount: commands.length,
        rewardEvents: rewardEvents.length
      });

      return this.processAnimationQueue();

    } catch (error) {
      this.logger.error('Error updating companion state', { error });
      // Return safe fallback animation
      return [this.getSafeDefaultAnimation()];
    }
  }

  /**
   * Get current companion state for external systems
   */
  getCompanionState(): CompanionState {
    return { ...this.companionState };
  }

  /**
   * Manually trigger a companion reaction (for testing or special events)
   */
  async triggerReaction(
    reactionType: 'celebration' | 'encouragement' | 'concern' | 'excitement',
    intensity: 'subtle' | 'medium' | 'high' = 'medium'
  ): Promise<AnimationCommand> {
    const reaction = this.createManualReaction(reactionType, intensity);
    await this.queueAnimations([reaction]);
    return reaction;
  }

  /**
   * Update companion visibility based on user preferences
   */
  updateVisibility(visible: boolean): void {
    this.companionState.isVisible = visible;
    
    if (!visible) {
      this.queueAnimations([{
        id: createUUID(),
        type: 'base_state',
        animation: 'fade_out',
        duration: 1000,
        priority: 'high',
        interruptible: false
      }]);
    } else {
      this.queueAnimations([{
        id: createUUID(),
        type: 'base_state',
        animation: 'fade_in',
        duration: 1000,
        priority: 'high',
        interruptible: false
      }]);
    }
  }

  // === Private Helper Methods ===

  private getBaseStateAnimation(userState: ADHDState): AnimationCommand {
    const stateAnimations = {
      Flow: {
        animation: 'gentle_float',
        intensity: Math.min(userState.confidence || 0.5, 0.8),
        glow: { 
          intensity: (userState.confidence || 0.5) * 0.8, 
          color: this.personalityEngine.getFlowColor(),
          pulseRate: 0.3
        }
      },
      Hyperfocus: {
        animation: 'focused_stillness',
        intensity: 0.9,
        glow: { 
          intensity: 1.0, 
          color: '#E27B4A',
          pulseRate: 0.1 // Very slow pulse to not distract
        }
      },
      Distracted: {
        animation: 'gentle_sway',
        intensity: 0.6,
        glow: { 
          intensity: 0.3, 
          color: '#E2D84A',
          pulseRate: 0.5
        }
      },
      Transitioning: {
        animation: 'soft_morphing',
        intensity: 0.7,
        glow: { 
          intensity: 0.5, 
          color: '#B84AE2',
          pulseRate: 0.4
        }
      },
      Neutral: {
        animation: 'idle_breathing',
        intensity: 0.5,
        glow: { 
          intensity: 0.4, 
          color: '#FFFFFF',
          pulseRate: 0.2
        }
      }
    };

    const anim = stateAnimations[userState.type];
    
    return {
      id: createUUID(),
      type: 'base_state',
      animation: anim.animation,
      duration: undefined, // Continuous
      loop: true,
      priority: 'low',
      glow: {
        ...anim.glow,
        fadeIn: 2000,
        fadeOut: 2000
      },
      interruptible: true
    };
  }

  private updateCompanionMood(userState: ADHDState, metrics?: BehavioralMetrics): void {
    const previousMood = this.companionState.mood;
    
    // Base mood on user state
    switch (userState.type) {
      case 'Flow':
        this.companionState.mood = userState.confidence && userState.confidence > 0.8 ? 'proud' : 'supportive';
        this.companionState.energy = Math.min(userState.confidence || 0.5, 0.8);
        break;
        
      case 'Hyperfocus':
        this.companionState.mood = 'calm';
        this.companionState.energy = 0.3; // Low energy to not distract
        break;
        
      case 'Distracted':
        this.companionState.mood = userState.duration && userState.duration > 10 * 60 * 1000 ? 'concerned' : 'supportive';
        this.companionState.energy = 0.4;
        break;
        
      case 'Transitioning':
        this.companionState.mood = 'supportive';
        this.companionState.energy = 0.6;
        break;
        
      default:
        this.companionState.mood = 'supportive';
        this.companionState.energy = 0.5;
    }

    // Adjust based on productivity metrics
    if (metrics) {
      if (metrics.productive_time_ratio > 0.8) {
        this.companionState.mood = 'proud';
        this.companionState.energy = Math.min(this.companionState.energy + 0.2, 1.0);
      } else if (metrics.productive_time_ratio < 0.3) {
        this.companionState.mood = 'concerned';
      }
    }

    if (previousMood !== this.companionState.mood) {
      this.logger.debug('Companion mood changed', {
        from: previousMood,
        to: this.companionState.mood,
        userState: userState.type,
        energy: this.companionState.energy
      });
    }
  }

  private generateExpression(userState: ADHDState, mood: CompanionState['mood']): Expression {
    const moodExpressions = {
      supportive: {
        type: 'happy' as const,
        intensity: 0.6,
        eyeState: 'normal' as const,
        mouthState: 'slight_smile' as const
      },
      excited: {
        type: 'excited' as const,
        intensity: 0.8,
        eyeState: 'wide' as const,
        mouthState: 'smile' as const
      },
      calm: {
        type: 'neutral' as const,
        intensity: 0.4,
        eyeState: 'half_closed' as const,
        mouthState: 'neutral' as const
      },
      concerned: {
        type: 'concerned' as const,
        intensity: 0.5,
        eyeState: 'normal' as const,
        mouthState: 'neutral' as const
      },
      proud: {
        type: 'proud' as const,
        intensity: 0.9,
        eyeState: 'sparkle' as const,
        mouthState: 'smile' as const
      }
    };

    const expression = moodExpressions[mood];
    
    return {
      ...expression,
      duration: this.config.companion.animationDuration || 3000
    };
  }

  private createExpressionCommand(expression: Expression): AnimationCommand {
    return {
      id: createUUID(),
      type: 'expression',
      expression,
      duration: expression.duration,
      priority: 'medium',
      interruptible: true
    };
  }

  private createRewardAnimation(reward: RewardEvent): AnimationCommand | null {
    // Only create animations for visible rewards that user wants to see
    if (reward.celebrationType === 'subtle' && !this.shouldShowSubtleRewards()) {
      return null;
    }

    const animationMap = {
      coins: this.createCoinAnimation(reward),
      achievement: this.createAchievementAnimation(reward),
      milestone: this.createMilestoneAnimation(reward),
      bonus: this.createBonusAnimation(reward),
      streak: this.createStreakAnimation(reward)
    };

    return animationMap[reward.type] || null;
  }

  private createCoinAnimation(reward: RewardEvent): AnimationCommand {
    const intensity = reward.celebrationType === 'celebration' ? 'high' : 'medium';
    
    return {
      id: createUUID(),
      type: 'celebration',
      animation: 'coin_collect',
      duration: 2000,
      priority: reward.priority === 'high' ? 'high' : 'medium',
      particles: {
        type: 'coins',
        count: Math.min((reward.amount || 0) / 5, 20),
        duration: 1500,
        spread: 45,
        speed: 0.5,
        colors: ['#FFD700', '#FFA500']
      },
      sound: this.personalityEngine.shouldPlaySound() ? {
        type: 'coin',
        volume: 0.3,
        respectsUserSettings: true
      } : undefined,
      interruptible: false
    };
  }

  private createAchievementAnimation(reward: RewardEvent): AnimationCommand {
    return {
      id: createUUID(),
      type: 'celebration',
      animation: 'achievement_celebration',
      duration: 4000,
      priority: 'high',
      glow: {
        intensity: 1.0,
        color: this.getAchievementGlowColor(reward.achievement?.rarity || 'common'),
        pulseRate: 0.8,
        fadeIn: 500,
        fadeOut: 1000
      },
      particles: {
        type: 'stars',
        count: 15,
        duration: 3000,
        spread: 90,
        speed: 0.3,
        colors: this.getAchievementColors(reward.achievement?.rarity || 'common')
      },
      sound: this.personalityEngine.shouldPlaySound() ? {
        type: 'achievement',
        volume: 0.4,
        respectsUserSettings: true
      } : undefined,
      interruptible: false
    };
  }

  private createMilestoneAnimation(reward: RewardEvent): AnimationCommand {
    return {
      id: createUUID(),
      type: 'celebration',
      animation: 'milestone_dance',
      duration: 3000,
      priority: 'high',
      glow: {
        intensity: 0.9,
        color: '#FFD700',
        pulseRate: 1.0,
        fadeIn: 300,
        fadeOut: 800
      },
      particles: {
        type: 'sparkles',
        count: 25,
        duration: 2500,
        spread: 120,
        speed: 0.4,
        colors: ['#FFD700', '#FF8C00', '#FF6347']
      },
      sound: this.personalityEngine.shouldPlaySound() ? {
        type: 'celebration',
        volume: 0.5,
        respectsUserSettings: true
      } : undefined,
      interruptible: false
    };
  }

  private createBonusAnimation(reward: RewardEvent): AnimationCommand {
    return {
      id: createUUID(),
      type: 'reaction',
      animation: 'happy_wiggle',
      duration: 1500,
      priority: 'low',
      particles: {
        type: 'sparkles',
        count: 8,
        duration: 1200,
        spread: 30,
        speed: 0.2,
        colors: ['#FF69B4', '#DDA0DD']
      },
      interruptible: true
    };
  }

  private createStreakAnimation(reward: RewardEvent): AnimationCommand {
    return {
      id: createUUID(),
      type: 'celebration',
      animation: 'streak_celebration',
      duration: 2500,
      priority: 'medium',
      glow: {
        intensity: 0.8,
        color: '#32CD32',
        pulseRate: 0.6,
        fadeIn: 400,
        fadeOut: 600
      },
      particles: {
        type: 'hearts',
        count: 12,
        duration: 2000,
        spread: 60,
        speed: 0.3,
        colors: ['#32CD32', '#98FB98']
      },
      interruptible: false
    };
  }

  private createInterventionAnimation(intervention: InterventionDecision): AnimationCommand | null {
    if (!intervention.type) return null;

    const animationMap = {
      encouragement: 'supportive_nod',
      suggestion: 'thoughtful_gesture',
      celebration: 'celebration_dance',
      gentle_nudge: 'gentle_attention',
      milestone: 'proud_pose'
    };

    const animation = animationMap[intervention.type.category];
    if (!animation) return null;

    return {
      id: createUUID(),
      type: 'reaction',
      animation,
      duration: 2500,
      priority: 'medium',
      glow: {
        intensity: 0.6,
        color: this.personalityEngine.getInterventionColor(intervention.type.category),
        pulseRate: 0.4,
        fadeIn: 800,
        fadeOut: 800
      },
      sound: this.personalityEngine.shouldPlaySound() ? {
        type: 'gentle_notification',
        volume: 0.2,
        respectsUserSettings: true
      } : undefined,
      interruptible: true
    };
  }

  private createManualReaction(
    reactionType: 'celebration' | 'encouragement' | 'concern' | 'excitement',
    intensity: 'subtle' | 'medium' | 'high'
  ): AnimationCommand {
    const animations = {
      celebration: 'celebration_dance',
      encouragement: 'supportive_nod',
      concern: 'concerned_lean',
      excitement: 'excited_bounce'
    };

    const intensitySettings = {
      subtle: { duration: 1500, glowIntensity: 0.4, particleCount: 5 },
      medium: { duration: 2500, glowIntensity: 0.6, particleCount: 10 },
      high: { duration: 4000, glowIntensity: 0.9, particleCount: 20 }
    };

    const settings = intensitySettings[intensity];

    return {
      id: createUUID(),
      type: 'reaction',
      animation: animations[reactionType],
      duration: settings.duration,
      priority: intensity === 'high' ? 'high' : 'medium',
      glow: {
        intensity: settings.glowIntensity,
        color: this.personalityEngine.getReactionColor(reactionType),
        pulseRate: intensity === 'high' ? 0.8 : 0.4,
        fadeIn: 500,
        fadeOut: 1000
      },
      particles: {
        type: reactionType === 'celebration' ? 'stars' : 'sparkles',
        count: settings.particleCount,
        duration: settings.duration * 0.8,
        spread: 45,
        speed: 0.3,
        colors: this.personalityEngine.getReactionColors(reactionType)
      },
      interruptible: intensity !== 'high'
    };
  }

  private shouldGenerateIdleVariation(): boolean {
    const timeSinceLastInteraction = Date.now() - this.companionState.lastInteraction.getTime();
    const idleThreshold = this.config.companion.animationDuration * 3; // 3x animation duration
    
    return timeSinceLastInteraction > idleThreshold && 
           this.config.companion.idleVariations && 
           Math.random() < 0.3; // 30% chance
  }

  private generateIdleVariation(): AnimationCommand {
    const variations = [
      'gentle_stretch',
      'look_around',
      'soft_bounce',
      'breathing_variation',
      'subtle_glow_shift'
    ];

    const animation = variations[Math.floor(Math.random() * variations.length)];

    return {
      id: createUUID(),
      type: 'idle_variation',
      animation,
      duration: 3000,
      priority: 'low',
      interruptible: true
    };
  }

  private async queueAnimations(commands: AnimationCommand[]): Promise<void> {
    const priorityCommands = commands.map(cmd => ({
      ...cmd,
      queueTime: new Date()
    }));

    // Add to queue with priority sorting
    this.animationQueue.commands.push(...priorityCommands);
    this.animationQueue.commands.sort((a, b) => {
      const priorityOrder = { 'urgent': 4, 'high': 3, 'medium': 2, 'low': 1 };
      return priorityOrder[b.priority] - priorityOrder[a.priority];
    });

    // Trim queue if too large
    if (this.animationQueue.commands.length > this.animationQueue.maxSize) {
      this.animationQueue.commands = this.animationQueue.commands.slice(0, this.animationQueue.maxSize);
    }
  }

  private async processAnimationQueue(): Promise<AnimationCommand[]> {
    if (this.animationQueue.isProcessing || this.animationQueue.commands.length === 0) {
      return [];
    }

    this.animationQueue.isProcessing = true;
    
    try {
      // Process up to 3 commands at once for efficiency
      const commandsToProcess = this.animationQueue.commands.splice(0, 3);
      
      // Mark execution time
      commandsToProcess.forEach(cmd => {
        cmd.executionTime = new Date();
      });

      return commandsToProcess;
      
    } finally {
      this.animationQueue.isProcessing = false;
    }
  }

  private getSafeDefaultAnimation(): AnimationCommand {
    return {
      id: createUUID(),
      type: 'base_state',
      animation: 'idle_breathing',
      duration: undefined,
      loop: true,
      priority: 'low',
      glow: {
        intensity: 0.3,
        color: '#FFFFFF',
        pulseRate: 0.2
      },
      interruptible: true
    };
  }

  private shouldShowSubtleRewards(): boolean {
    // Check user preferences for subtle reward visibility
    return true; // Would integrate with user preferences
  }

  private getAchievementGlowColor(rarity: string): string {
    const colors = {
      common: '#87CEEB',
      rare: '#9370DB',
      epic: '#FF69B4',
      legendary: '#FFD700'
    };
    return colors[rarity as keyof typeof colors] || colors.common;
  }

  private getAchievementColors(rarity: string): string[] {
    const colorSets = {
      common: ['#87CEEB', '#B0E0E6'],
      rare: ['#9370DB', '#BA55D3'],
      epic: ['#FF69B4', '#FF1493'],
      legendary: ['#FFD700', '#FFA500', '#FF4500']
    };
    return colorSets[rarity as keyof typeof colorSets] || colorSets.common;
  }

  private initializeStateMachine(): void {
    // XState machine for companion behavior
    this.animationStateMachine = createMachine({
      id: 'companion',
      initial: 'idle',
      context: {
        energy: 0.5,
        mood: 'supportive'
      },
      states: {
        idle: {
          on: {
            USER_FOCUS: 'supportive',
            USER_DISTRACTED: 'concerned',
            REWARD_EVENT: 'celebrating'
          }
        },
        supportive: {
          on: {
            USER_LOST_FOCUS: 'concerned',
            ACHIEVEMENT: 'celebrating',
            TIMEOUT: 'idle'
          }
        },
        concerned: {
          on: {
            USER_RECOVERED: 'supportive',
            INTERVENTION_SHOWN: 'encouraging',
            TIMEOUT: 'idle'
          }
        },
        celebrating: {
          on: {
            CELEBRATION_COMPLETE: 'supportive',
            TIMEOUT: 'idle'
          }
        },
        encouraging: {
          on: {
            USER_RESPONDED: 'supportive',
            TIMEOUT: 'idle'
          }
        }
      }
    });
  }
}

/**
 * Personality Engine - Adds character to companion behavior
 */
class PersonalityEngine {
  private traits: GamificationConfig['companion']['personalityTraits'];
  private preferences: UserPreferences;

  constructor(traits: GamificationConfig['companion']['personalityTraits'], preferences: UserPreferences) {
    this.traits = traits;
    this.preferences = preferences;
  }

  modifyAnimations(commands: AnimationCommand[], userState: ADHDState): AnimationCommand[] {
    return commands.map(cmd => {
      const modified = { ...cmd };
      
      // Adjust based on cheerfulness
      if (modified.glow && this.traits.cheerfulness > 0.7) {
        modified.glow.intensity *= 1.2;
      }
      
      // Adjust based on formality
      if (this.traits.formality < 0.3 && modified.animation) {
        // Make animations more playful
        if (modified.animation.includes('gentle')) {
          modified.animation = modified.animation.replace('gentle', 'playful');
        }
      }
      
      // Adjust based on user preferences
      if (!this.preferences.companionAnimations) {
        modified.animation = 'minimal_presence';
        modified.duration = Math.min(modified.duration || 1000, 1000);
      }
      
      return modified;
    });
  }

  getFlowColor(): string {
    const colors = {
      high_cheer: '#4A90E2',
      supportive: '#5CB85C',
      formal: '#337AB7'
    };
    
    if (this.traits.cheerfulness > 0.8) return colors.high_cheer;
    if (this.traits.supportiveness > 0.7) return colors.supportive;
    return colors.formal;
  }

  getInterventionColor(category: string): string {
    const baseColors = {
      encouragement: '#32CD32',
      suggestion: '#FFA500',
      celebration: '#FFD700',
      gentle_nudge: '#87CEEB'
    };
    
    return baseColors[category as keyof typeof baseColors] || '#FFFFFF';
  }

  getReactionColor(reaction: string): string {
    const colors = {
      celebration: '#FFD700',
      encouragement: '#32CD32',
      concern: '#FFA500',
      excitement: '#FF69B4'
    };
    
    return colors[reaction as keyof typeof colors] || '#FFFFFF';
  }

  getReactionColors(reaction: string): string[] {
    const colorSets = {
      celebration: ['#FFD700', '#FFA500', '#FF6347'],
      encouragement: ['#32CD32', '#98FB98'],
      concern: ['#FFA500', '#FFD700'],
      excitement: ['#FF69B4', '#DDA0DD', '#DA70D6']
    };
    
    return colorSets[reaction as keyof typeof colorSets] || ['#FFFFFF'];
  }

  shouldPlaySound(): boolean {
    return this.preferences.companionSounds && this.preferences.soundEnabled;
  }
}