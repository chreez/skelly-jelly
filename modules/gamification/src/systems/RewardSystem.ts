/**
 * RewardSystem - User-Centered Reward Distribution
 * 
 * Implements variable ratio reinforcement and meaningful achievements
 * while maintaining intrinsic motivation and avoiding patronizing rewards.
 */

import {
  ADHDState,
  BehavioralMetrics,
  RewardEvent,
  Achievement,
  Milestone,
  VisualReward,
  Reward,
  WalletBalance,
  PersonalRecord,
  UserPreferences,
  SessionMetrics,
  GamificationConfig
} from '../types/index.js';
import { Logger } from 'winston';
import { v4 as uuidv4 } from 'uuid';

// Helper function to satisfy UUID type
const createUUID = (): string => uuidv4();
import { differenceInMinutes, isToday, isThisWeek } from 'date-fns';

export interface AchievementDefinition {
  id: string;
  name: string;
  description: string;
  icon: string;
  category: Achievement['category'];
  rarity: Achievement['rarity'];
  hidden: boolean;
  trigger: AchievementTrigger;
  rewards: Reward[];
}

export interface AchievementTrigger {
  type: 'focus_duration' | 'session_count' | 'streak' | 'recovery_speed' | 'consistency' | 'special_event';
  threshold?: number;
  timeframe?: 'session' | 'daily' | 'weekly' | 'monthly' | 'all_time';
  conditions?: Record<string, unknown>;
}

export class RewardSystem {
  private wallet: WalletBalance;
  private unlockedAchievements: Set<string>;
  private achievementProgress: Map<string, number>;
  private rewardQueue: RewardEvent[];
  private distributionEngine: RewardDistributionEngine;
  private achievementDefinitions: Map<string, AchievementDefinition>;
  private logger: Logger;
  private config: GamificationConfig;

  constructor(logger: Logger, config: GamificationConfig) {
    this.logger = logger;
    this.config = config;
    
    this.wallet = {
      coins: 0,
      totalEarned: 0,
      totalSpent: 0,
      pendingRewards: [],
      lifetimeBalance: 0
    };
    
    this.unlockedAchievements = new Set();
    this.achievementProgress = new Map();
    this.rewardQueue = [];
    this.distributionEngine = new RewardDistributionEngine(config.rewards.variableRatioBase, logger);
    this.achievementDefinitions = new Map();
    
    this.initializeAchievements();
  }

  /**
   * Process a state change and determine rewards
   * Focus on meaningful recognition rather than constant reinforcement
   */
  async processStateChange(
    oldState: ADHDState,
    newState: ADHDState,
    duration: number,
    metrics: BehavioralMetrics,
    sessionMetrics: SessionMetrics,
    preferences: UserPreferences
  ): Promise<RewardEvent[]> {
    const rewards: RewardEvent[] = [];

    try {
      // Reward sustained focus (meaningful durations only)
      if (this.isSustainedFocusWorthy(oldState, duration)) {
        const focusReward = await this.grantFocusReward(duration, oldState.depth || 0.5);
        if (focusReward) rewards.push(focusReward);
      }

      // Reward recovery from distraction (show resilience)
      if (this.isRecoveryWorthy(oldState, newState, sessionMetrics)) {
        const recoveryReward = await this.grantRecoveryReward(oldState, newState);
        if (recoveryReward) rewards.push(recoveryReward);
      }

      // Check for achievement unlocks
      const newAchievements = await this.checkAchievements(
        newState, 
        duration, 
        metrics, 
        sessionMetrics,
        this.getRecentHistory(sessionMetrics)
      );
      rewards.push(...newAchievements);

      // Check for milestone completions
      const milestoneRewards = await this.checkMilestones(sessionMetrics);
      rewards.push(...milestoneRewards);

      // Apply variable ratio reinforcement (sparingly)
      if (this.shouldApplyVariableRatio(preferences, sessionMetrics)) {
        const bonusReward = await this.grantVariableRatioBonus(newState, metrics);
        if (bonusReward) rewards.push(bonusReward);
      }

      // Update wallet and process rewards
      this.processRewards(rewards);

      this.logger.info('State change rewards processed', {
        oldState: oldState.type,
        newState: newState.type,
        duration: Math.round(duration / 1000),
        rewardsGranted: rewards.length,
        coinsEarned: rewards.reduce((sum, r) => sum + (r.amount || 0), 0)
      });

      return rewards;

    } catch (error) {
      this.logger.error('Error processing state change rewards', { error });
      return [];
    }
  }

  /**
   * Grant coins for focused work
   * Only rewards meaningful focus periods to maintain value
   */
  private async grantFocusReward(duration: number, depth: number): Promise<RewardEvent | null> {
    const minutes = Math.floor(duration / (60 * 1000));
    
    // Only reward meaningful focus periods (15+ minutes)
    if (minutes < 15) {
      return null;
    }

    // Calculate coins based on duration and depth
    const baseCoins = Math.floor(minutes * this.config.rewards.coinsPerFocusMinute);
    const depthBonus = Math.floor(baseCoins * depth * 0.5); // Depth adds up to 50% bonus
    const totalCoins = baseCoins + depthBonus;

    // Determine visual celebration intensity
    let celebrationType: 'subtle' | 'noticeable' | 'celebration' = 'subtle';
    if (minutes >= 60) celebrationType = 'celebration';
    else if (minutes >= 30) celebrationType = 'noticeable';

    return {
      id: createUUID(),
      type: 'coins',
      amount: totalCoins,
      message: this.generateFocusMessage(minutes, depth),
      visual: {
        animation: 'coin_burst',
        duration: celebrationType === 'celebration' ? 3000 : 1500,
        intensity: celebrationType === 'celebration' ? 'high' : 'medium',
        colors: ['#FFD700', '#FFA500', '#FF8C00'],
        position: 'companion'
      },
      priority: minutes >= 60 ? 'high' : 'medium',
      timestamp: new Date(),
      celebrationType
    };
  }

  /**
   * Reward recovery from distraction
   * Celebrates resilience and ability to refocus
   */
  private async grantRecoveryReward(oldState: ADHDState, newState: ADHDState): Promise<RewardEvent | null> {
    // Only reward if transitioning to a better state
    if (!this.isBetterState(newState.type, oldState.type)) {
      return null;
    }

    const baseCoins = 5; // Small reward for recovery
    const speedBonus = oldState.duration < 5 * 60 * 1000 ? 3 : 0; // Bonus for quick recovery

    return {
      id: createUUID(),
      type: 'coins',
      amount: baseCoins + speedBonus,
      message: this.generateRecoveryMessage(oldState.duration),
      visual: {
        animation: 'gentle_glow',
        duration: 2000,
        intensity: 'subtle',
        colors: ['#98FB98', '#90EE90'],
        position: 'companion'
      },
      priority: 'low',
      timestamp: new Date(),
      celebrationType: 'subtle'
    };
  }

  /**
   * Check for achievement unlocks
   * Focus on meaningful accomplishments
   */
  private async checkAchievements(
    state: ADHDState,
    duration: number,
    metrics: BehavioralMetrics,
    sessionMetrics: SessionMetrics,
    history: ADHDState[]
  ): Promise<RewardEvent[]> {
    const rewards: RewardEvent[] = [];

    for (const [id, achievement] of this.achievementDefinitions) {
      if (this.unlockedAchievements.has(id)) {
        continue; // Already unlocked
      }

      if (await this.checkAchievementTrigger(achievement, state, duration, metrics, sessionMetrics, history)) {
        const reward = await this.unlockAchievement(achievement);
        if (reward) {
          rewards.push(reward);
        }
      }
    }

    return rewards;
  }

  /**
   * Check for milestone completions
   */
  private async checkMilestones(sessionMetrics: SessionMetrics): Promise<RewardEvent[]> {
    const rewards: RewardEvent[] = [];
    
    // Focus time milestones
    const focusHours = sessionMetrics.totalFocusTime / (60 * 60 * 1000);
    const milestones = [
      { hours: 1, name: "First Hour", coins: 50 },
      { hours: 2, name: "Two Hour Hero", coins: 100 },
      { hours: 4, name: "Focus Champion", coins: 200 },
      { hours: 8, name: "Productivity Legend", coins: 500 }
    ];

    for (const milestone of milestones) {
      if (focusHours >= milestone.hours && !this.hasMilestone(`focus_${milestone.hours}h`)) {
        const reward = await this.grantMilestoneReward(milestone);
        if (reward) {
          rewards.push(reward);
          this.markMilestoneCompleted(`focus_${milestone.hours}h`);
        }
      }
    }

    return rewards;
  }

  /**
   * Variable ratio reinforcement
   * Used sparingly to maintain intrinsic motivation
   */
  private async grantVariableRatioBonus(state: ADHDState, metrics: BehavioralMetrics): Promise<RewardEvent | null> {
    if (!this.distributionEngine.shouldReward()) {
      return null;
    }

    const bonusAmount = Math.floor(Math.random() * 15) + 5; // 5-20 coins

    return {
      id: createUUID(),
      type: 'bonus',
      amount: bonusAmount,
      message: this.generateBonusMessage(),
      visual: {
        animation: 'sparkle',
        duration: 1500,
        intensity: 'medium',
        colors: ['#FF69B4', '#DDA0DD', '#DA70D6'],
        position: 'companion'
      },
      priority: 'low',
      timestamp: new Date(),
      celebrationType: 'subtle'
    };
  }

  /**
   * Get current wallet balance
   */
  getWallet(): WalletBalance {
    return { ...this.wallet };
  }

  /**
   * Get unlocked achievements
   */
  getUnlockedAchievements(): Achievement[] {
    return Array.from(this.unlockedAchievements)
      .map(id => this.achievementDefinitions.get(id))
      .filter((achievement): achievement is AchievementDefinition => achievement !== undefined)
      .map(def => this.convertToAchievement(def));
  }

  /**
   * Spend coins (for future features like customization)
   */
  async spendCoins(amount: number, reason: string): Promise<boolean> {
    if (this.wallet.coins < amount) {
      return false;
    }

    this.wallet.coins -= amount;
    this.wallet.totalSpent += amount;

    this.logger.info('Coins spent', { amount, reason, remainingCoins: this.wallet.coins });
    return true;
  }

  // === Private Helper Methods ===

  private isSustainedFocusWorthy(state: ADHDState, duration: number): boolean {
    const minutes = duration / (60 * 1000);
    
    // Only Flow and focused Hyperfocus sessions count
    if (state.type === 'Flow') {
      return minutes >= 15; // Minimum meaningful focus
    }
    
    if (state.type === 'Hyperfocus') {
      return minutes >= 20 && minutes <= 120; // Reward healthy hyperfocus
    }

    return false;
  }

  private isRecoveryWorthy(oldState: ADHDState, newState: ADHDState, sessionMetrics: SessionMetrics): boolean {
    // Must be transitioning from distraction to focus
    if (oldState.type !== 'Distracted') {
      return false;
    }

    // Must be transitioning to a better state
    if (!['Flow', 'Transitioning'].includes(newState.type)) {
      return false;
    }

    // Don't over-reward if user recovers too frequently (suggests shallow focus)
    const recentRecoveries = sessionMetrics.recoveryCount || 0;
    return recentRecoveries <= 5; // Max 5 recovery rewards per session
  }

  private shouldApplyVariableRatio(preferences: UserPreferences, sessionMetrics: SessionMetrics): boolean {
    // Respect user preference for minimal rewards
    if (!preferences.rewardTypes.includes('coins')) {
      return false;
    }

    // Don't spam rewards - limit variable ratio to once per 30 minutes
    const sessionMinutes = (Date.now() - sessionMetrics.startTime.getTime()) / (60 * 1000);
    const maxRewards = Math.floor(sessionMinutes / 30);
    const currentRewards = sessionMetrics.interventionsReceived || 0;

    return currentRewards < maxRewards;
  }

  private isBetterState(newType: ADHDState['type'], oldType: ADHDState['type']): boolean {
    const stateRanking = {
      'Distracted': 1,
      'Neutral': 2,
      'Transitioning': 3,
      'Flow': 4,
      'Hyperfocus': 4 // Same level as flow for transition purposes
    };

    return stateRanking[newType] > stateRanking[oldType];
  }

  private async checkAchievementTrigger(
    achievement: AchievementDefinition,
    state: ADHDState,
    duration: number,
    metrics: BehavioralMetrics,
    sessionMetrics: SessionMetrics,
    history: ADHDState[]
  ): Promise<boolean> {
    const trigger = achievement.trigger;

    switch (trigger.type) {
      case 'focus_duration':
        return this.checkFocusDurationTrigger(trigger, sessionMetrics);
      
      case 'session_count':
        return this.checkSessionCountTrigger(trigger, sessionMetrics);
      
      case 'streak':
        return this.checkStreakTrigger(trigger, sessionMetrics);
      
      case 'recovery_speed':
        return this.checkRecoverySpeedTrigger(trigger, sessionMetrics);
      
      case 'consistency':
        return this.checkConsistencyTrigger(trigger, metrics, sessionMetrics);
      
      default:
        return false;
    }
  }

  private checkFocusDurationTrigger(trigger: AchievementTrigger, sessionMetrics: SessionMetrics): boolean {
    if (!trigger.threshold) return false;
    
    const focusMinutes = sessionMetrics.totalFocusTime / (60 * 1000);
    return focusMinutes >= trigger.threshold;
  }

  private checkSessionCountTrigger(trigger: AchievementTrigger, sessionMetrics: SessionMetrics): boolean {
    if (!trigger.threshold) return false;
    
    return sessionMetrics.flowSessions.length >= trigger.threshold;
  }

  private checkStreakTrigger(trigger: AchievementTrigger, sessionMetrics: SessionMetrics): boolean {
    // This would need integration with ProgressTracker for streak data
    // For now, return false as placeholder
    return false;
  }

  private checkRecoverySpeedTrigger(trigger: AchievementTrigger, sessionMetrics: SessionMetrics): boolean {
    if (!trigger.threshold) return false;
    
    // Check if recovery count is high (indicating good recovery skills)
    return sessionMetrics.recoveryCount >= trigger.threshold;
  }

  private checkConsistencyTrigger(
    trigger: AchievementTrigger, 
    metrics: BehavioralMetrics, 
    sessionMetrics: SessionMetrics
  ): boolean {
    if (!trigger.threshold) return false;
    
    return metrics.productive_time_ratio >= trigger.threshold;
  }

  private async unlockAchievement(achievement: AchievementDefinition): Promise<RewardEvent | null> {
    this.unlockedAchievements.add(achievement.id);

    // Grant achievement rewards
    for (const reward of achievement.rewards) {
      if (reward.type === 'coins' && reward.amount) {
        this.wallet.coins += reward.amount;
        this.wallet.totalEarned += reward.amount;
      }
    }

    this.logger.info('Achievement unlocked', {
      id: achievement.id,
      name: achievement.name,
      category: achievement.category,
      rarity: achievement.rarity
    });

    return {
      id: createUUID(),
      type: 'achievement',
      achievement: this.convertToAchievement(achievement),
      message: `🏆 Achievement Unlocked: ${achievement.name}!`,
      visual: {
        animation: 'achievement_unlock',
        duration: 4000,
        intensity: achievement.rarity === 'legendary' ? 'high' : 'medium',
        colors: this.getAchievementColors(achievement.rarity),
        position: 'center'
      },
      priority: 'high',
      timestamp: new Date(),
      celebrationType: 'celebration'
    };
  }

  private convertToAchievement(def: AchievementDefinition): Achievement {
    return {
      id: def.id,
      name: def.name,
      description: def.description,
      icon: def.icon,
      category: def.category,
      rarity: def.rarity,
      hidden: def.hidden,
      unlockedAt: new Date(),
      rewards: def.rewards
    };
  }

  private getAchievementColors(rarity: Achievement['rarity']): string[] {
    switch (rarity) {
      case 'common':
        return ['#87CEEB', '#B0E0E6'];
      case 'rare':
        return ['#9370DB', '#BA55D3'];
      case 'epic':
        return ['#FF69B4', '#FF1493'];
      case 'legendary':
        return ['#FFD700', '#FFA500', '#FF4500'];
      default:
        return ['#87CEEB', '#B0E0E6'];
    }
  }

  private async grantMilestoneReward(milestone: { hours: number; name: string; coins: number }): Promise<RewardEvent> {
    this.wallet.coins += milestone.coins;
    this.wallet.totalEarned += milestone.coins;

    return {
      id: createUUID(),
      type: 'milestone',
      amount: milestone.coins,
      milestone: {
        type: 'focus_time',
        name: milestone.name,
        description: `Focused for ${milestone.hours} hour${milestone.hours > 1 ? 's' : ''}`,
        reward: milestone.coins,
        threshold: milestone.hours,
        unlockedAt: new Date(),
        icon: '🎯'
      },
      message: `🎯 Milestone: ${milestone.name}! +${milestone.coins} coins`,
      visual: {
        animation: 'celebration',
        duration: 3000,
        intensity: 'high',
        colors: ['#FFD700', '#FF8C00', '#FF6347'],
        position: 'center'
      },
      priority: 'high',
      timestamp: new Date(),
      celebrationType: 'celebration'
    };
  }

  private processRewards(rewards: RewardEvent[]): void {
    for (const reward of rewards) {
      if (reward.amount) {
        this.wallet.coins += reward.amount;
        this.wallet.totalEarned += reward.amount;
        this.wallet.lifetimeBalance += reward.amount;
      }
    }
  }

  private generateFocusMessage(minutes: number, depth: number): string {
    if (minutes >= 60) {
      return `🔥 Incredible ${minutes} minute focus session! You're in the zone!`;
    } else if (minutes >= 30) {
      return `💪 Solid ${minutes} minute focus! Keep up the great work!`;
    } else {
      return `✨ Nice ${minutes} minute focus session!`;
    }
  }

  private generateRecoveryMessage(distractionDuration: number): string {
    const minutes = Math.floor(distractionDuration / (60 * 1000));
    
    if (minutes < 2) {
      return "🎯 Quick recovery! Your focus is strong today.";
    } else if (minutes < 5) {
      return "✨ Nice job getting back on track!";
    } else {
      return "🦴 Your skeleton pal is proud of your persistence!";
    }
  }

  private generateBonusMessage(): string {
    const messages = [
      "🌟 Surprise bonus! Keep being awesome!",
      "✨ Random sparkle of appreciation!",
      "🎁 A little extra something for your effort!",
      "🦴 Your skeleton friend felt generous!"
    ];
    
    return messages[Math.floor(Math.random() * messages.length)] || "Great work!";
  }

  private getRecentHistory(sessionMetrics: SessionMetrics): ADHDState[] {
    // This would integrate with actual history tracking
    // For now, return empty array
    return [];
  }

  private completedMilestones = new Set<string>();

  private hasMilestone(milestoneId: string): boolean {
    return this.completedMilestones.has(milestoneId);
  }

  private markMilestoneCompleted(milestoneId: string): void {
    this.completedMilestones.add(milestoneId);
  }

  private initializeAchievements(): void {
    // Define meaningful achievements that celebrate real progress
    const achievements: AchievementDefinition[] = [
      {
        id: 'first_focus',
        name: 'Getting Started',
        description: 'Complete your first 15-minute focus session',
        icon: '🌱',
        category: 'focus',
        rarity: 'common',
        hidden: false,
        trigger: { type: 'focus_duration', threshold: 15, timeframe: 'session' },
        rewards: [{ type: 'coins', amount: 25 }]
      },
      {
        id: 'focus_master',
        name: 'Focus Master',
        description: 'Maintain focus for 2 hours in a single day',
        icon: '🧠',
        category: 'focus',
        rarity: 'rare',
        hidden: false,
        trigger: { type: 'focus_duration', threshold: 120, timeframe: 'daily' },
        rewards: [{ type: 'coins', amount: 200 }]
      },
      {
        id: 'quick_recovery',
        name: 'Bounce Back',
        description: 'Recover from distractions 10 times in one session',
        icon: '⚡',
        category: 'recovery',
        rarity: 'rare',
        hidden: false,
        trigger: { type: 'recovery_speed', threshold: 10 },
        rewards: [{ type: 'coins', amount: 100 }]
      },
      {
        id: 'consistency_champion',
        name: 'Consistency Champion',
        description: 'Maintain 80% productivity for a full session',
        icon: '🏆',
        category: 'consistency',
        rarity: 'epic',
        hidden: false,
        trigger: { type: 'consistency', threshold: 0.8 },
        rewards: [{ type: 'coins', amount: 300 }]
      },
      {
        id: 'deep_flow',
        name: 'In The Zone',
        description: 'Enter deep flow state for 45+ minutes',
        icon: '🌊',
        category: 'focus',
        rarity: 'epic',
        hidden: true,
        trigger: { type: 'focus_duration', threshold: 45, timeframe: 'session' },
        rewards: [{ type: 'coins', amount: 250 }]
      }
    ];

    for (const achievement of achievements) {
      this.achievementDefinitions.set(achievement.id, achievement);
    }
  }
}

/**
 * Variable Ratio Reinforcement Engine
 * Implements unpredictable reward timing to maintain engagement
 */
class RewardDistributionEngine {
  private baseRate: number;
  private streak = 0;
  private lastReward: Date | null = null;
  private logger: Logger;

  constructor(baseRate: number, logger: Logger) {
    this.baseRate = baseRate;
    this.logger = logger;
  }

  shouldReward(): boolean {
    const timeSinceLastReward = this.lastReward 
      ? Date.now() - this.lastReward.getTime() 
      : Infinity;
      
    // Time multiplier: Higher chance after longer gaps (max 2x)
    const timeMultiplier = Math.min(timeSinceLastReward / (30 * 60 * 1000), 2);
    
    // Streak multiplier: Lower chance after recent rewards (min 0.3x)
    const streakMultiplier = Math.max(1 - (this.streak * 0.1), 0.3);
    
    const chance = this.baseRate * timeMultiplier * streakMultiplier;
    const shouldReward = Math.random() < chance;

    if (shouldReward) {
      this.lastReward = new Date();
      this.streak += 1;
    } else {
      this.streak = Math.max(0, this.streak - 1);
    }

    return shouldReward;
  }

  reset(): void {
    this.streak = 0;
    this.lastReward = null;
  }
}