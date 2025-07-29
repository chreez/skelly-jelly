/**
 * ProgressTracker - User Progress Analytics and Insights
 * 
 * Tracks meaningful progress indicators and provides actionable insights
 * while respecting user privacy and focusing on self-improvement.
 */

import {
  ADHDState,
  BehavioralMetrics,
  SessionMetrics,
  DailyMetrics,
  AllTimeMetrics,
  StreakInfo,
  FlowSession,
  PersonalRecord,
  TrendAnalysis,
  ProgressUpdate,
  LevelProgress,
  Milestone,
  UserPreferences,
  GamificationConfig
} from '../types/index.js';
import { Logger } from 'winston';
import { v4 as uuidv4 } from 'uuid';

// Helper function to satisfy UUID type
const createUUID = (): string => uuidv4();
import { 
  isToday, 
  isYesterday, 
  startOfDay, 
  endOfDay, 
  differenceInDays,
  differenceInMinutes,
  format,
  subDays,
  isSameDay
} from 'date-fns';

export class ProgressTracker {
  private currentSession: SessionMetrics;
  private historicalData: Map<string, DailyMetrics>; // date string -> metrics
  private personalRecords: Map<string, PersonalRecord>;
  private streaks: Map<string, StreakInfo>;
  private levelSystem: LevelSystemManager;
  private logger: Logger;
  private config: GamificationConfig;

  constructor(logger: Logger, config: GamificationConfig) {
    this.logger = logger;
    this.config = config;
    
    this.currentSession = this.createNewSession();
    this.historicalData = new Map();
    this.personalRecords = new Map();
    this.streaks = new Map();
    this.levelSystem = new LevelSystemManager(logger);
    
    this.initializeStreaks();
    this.loadHistoricalData();
  }

  /**
   * Update progress with new state information
   * Focus on meaningful metrics that help users understand their patterns
   */
  async updateProgress(
    state: ADHDState,
    metrics: BehavioralMetrics,
    timestamp: Date,
    workContext?: any
  ): Promise<ProgressUpdate> {
    try {
      // Update current session
      this.updateSessionMetrics(state, metrics, timestamp);
      
      // Check for new personal records
      const newRecords = await this.checkPersonalRecords(state, metrics);
      
      // Update streak information
      const streakUpdate = this.updateStreaks(state, timestamp);
      
      // Analyze trends
      const trends = await this.analyzeTrends();
      
      // Check for milestones
      const milestones = this.checkMilestones();
      
      // Check for level progression
      const levelUp = this.levelSystem.checkLevelProgression(this.currentSession, this.getAllTimeMetrics());

      const update: ProgressUpdate = {
        session: { ...this.currentSession },
        records: newRecords,
        streaks: Array.from(this.streaks.values()),
        trends,
        milestones,
        levelUp
      };

      this.logger.debug('Progress updated', {
        sessionId: this.currentSession.sessionId,
        totalFocusMinutes: Math.round(this.currentSession.totalFocusTime / (60 * 1000)),
        productivityScore: this.currentSession.productivityScore,
        newRecords: newRecords.length,
        levelUp: levelUp !== undefined
      });

      return update;

    } catch (error) {
      this.logger.error('Error updating progress', { error });
      throw error;
    }
  }

  /**
   * Start a new session
   */
  startNewSession(): void {
    this.finalizePreviousSession();
    this.currentSession = this.createNewSession();
    
    this.logger.info('New session started', {
      sessionId: this.currentSession.sessionId,
      timestamp: this.currentSession.startTime
    });
  }

  /**
   * Get current session summary
   */
  getCurrentSession(): SessionMetrics {
    return { ...this.currentSession };
  }

  /**
   * Get daily metrics for a specific date
   */
  getDailyMetrics(date: Date): DailyMetrics | null {
    const dateKey = format(date, 'yyyy-MM-dd');
    return this.historicalData.get(dateKey) || null;
  }

  /**
   * Get all-time metrics
   */
  getAllTimeMetrics(): AllTimeMetrics {
    const allDailyMetrics = Array.from(this.historicalData.values());
    
    const totalFocusHours = allDailyMetrics.reduce((sum, day) => sum + day.totalFocusTime, 0) / (60 * 60 * 1000);
    const totalSessions = allDailyMetrics.reduce((sum, day) => sum + day.sessionCount, 0);
    const averageProductivity = allDailyMetrics.length > 0 
      ? allDailyMetrics.reduce((sum, day) => sum + (day.averageSessionLength * day.sessionCount), 0) / totalSessions || 0
      : 0;

    const streaks = Array.from(this.streaks.values());
    const bestStreak = Math.max(...streaks.map(s => s.best), 0);
    
    const currentLevel = this.levelSystem.getCurrentLevel();
    
    return {
      totalFocusHours,
      totalSessions,
      averageProductivity,
      bestStreak,
      achievementsUnlocked: 0, // This would come from RewardSystem
      currentLevel,
      favoriteTimeOfDay: this.calculateFavoriteTimeOfDay(),
      improvementRate: this.calculateImprovementRate()
    };
  }

  /**
   * Get streak information
   */
  getStreaks(): StreakInfo[] {
    return Array.from(this.streaks.values());
  }

  /**
   * Get personal records
   */
  getPersonalRecords(): PersonalRecord[] {
    return Array.from(this.personalRecords.values());
  }

  /**
   * Get productivity insights
   */
  getInsights(): string[] {
    const insights: string[] = [];
    
    // Session insights
    if (this.currentSession.totalFocusTime > 60 * 60 * 1000) {
      insights.push("You've focused for over an hour today! That's excellent sustained attention.");
    }
    
    if (this.currentSession.recoveryCount > 0) {
      insights.push(`You've bounced back from distractions ${this.currentSession.recoveryCount} times - great resilience!`);
    }

    // Weekly trends
    const weeklyTrend = this.getWeeklyTrend();
    if (weeklyTrend === 'improving') {
      insights.push("Your focus has been improving this week - keep up the momentum!");
    }

    // Time of day patterns
    const favoriteTime = this.calculateFavoriteTimeOfDay();
    if (favoriteTime !== 'unknown') {
      insights.push(`You tend to focus best in the ${favoriteTime} - consider scheduling important work then.`);
    }

    return insights;
  }

  // === Private Helper Methods ===

  private createNewSession(): SessionMetrics {
    return {
      sessionId: createUUID(),
      startTime: new Date(),
      totalFocusTime: 0,
      totalDistractedTime: 0,
      flowSessions: [],
      distractionCount: 0,
      recoveryCount: 0,
      productivityScore: 0,
      interventionsReceived: 0,
      interventionsEngaged: 0,
      deepestFlow: 0,
      longestFocusStreak: 0
    };
  }

  private updateSessionMetrics(state: ADHDState, metrics: BehavioralMetrics, timestamp: Date): void {
    // Update time tracking
    if (state.type === 'Flow' || state.type === 'Hyperfocus') {
      this.currentSession.totalFocusTime += state.duration || 0;
      
      // Track focus sessions
      if (state.duration && state.duration > 10 * 60 * 1000) { // 10+ minute sessions
        const flowSession: FlowSession = {
          startTime: new Date(timestamp.getTime() - state.duration),
          endTime: timestamp,
          averageDepth: state.depth || 0.5,
          peakDepth: state.depth || 0.5,
          duration: state.duration,
          quality: this.categorizeFlowQuality(state.duration, state.depth || 0.5)
        };
        
        this.currentSession.flowSessions.push(flowSession);
      }
      
      // Update deepest flow
      if (state.depth && state.depth > this.currentSession.deepestFlow) {
        this.currentSession.deepestFlow = state.depth;
      }
      
      // Update longest focus streak
      if (state.duration && state.duration > this.currentSession.longestFocusStreak) {
        this.currentSession.longestFocusStreak = state.duration;
      }
    }
    
    if (state.type === 'Distracted') {
      this.currentSession.totalDistractedTime += state.duration || 0;
      this.currentSession.distractionCount++;
    }

    // Update productivity score (rolling average)
    this.currentSession.productivityScore = this.calculateProductivityScore();

    // Update session end time
    this.currentSession.endTime = timestamp;
  }

  private categorizeFlowQuality(duration: number, depth: number): FlowSession['quality'] {
    const minutes = duration / (60 * 1000);
    
    if (depth > 0.8 && minutes > 45) return 'profound';
    if (depth > 0.6 && minutes > 30) return 'deep';
    if (depth > 0.4 && minutes > 15) return 'moderate';
    return 'light';
  }

  private calculateProductivityScore(): number {
    const totalTime = this.currentSession.totalFocusTime + this.currentSession.totalDistractedTime;
    if (totalTime === 0) return 0;
    
    return this.currentSession.totalFocusTime / totalTime;
  }

  private async checkPersonalRecords(state: ADHDState, metrics: BehavioralMetrics): Promise<PersonalRecord[]> {
    const newRecords: PersonalRecord[] = [];
    
    // Check longest focus session
    if (state.duration && state.duration > 0) {
      const currentBest = this.personalRecords.get('longest_focus')?.value || 0;
      if (state.duration > currentBest) {
        const record: PersonalRecord = {
          type: 'longest_focus',
          value: state.duration,
          previousBest: currentBest,
          improvement: state.duration - currentBest,
          timestamp: new Date(),
          context: `${Math.round(state.duration / (60 * 1000))} minute ${state.type.toLowerCase()} session`
        };
        
        this.personalRecords.set('longest_focus', record);
        newRecords.push(record);
      }
    }
    
    // Check best productivity score
    const currentProductivity = this.currentSession.productivityScore;
    const bestProductivity = this.personalRecords.get('best_productivity')?.value || 0;
    if (currentProductivity > bestProductivity && currentProductivity > 0.8) {
      const record: PersonalRecord = {
        type: 'best_productivity',
        value: currentProductivity,
        previousBest: bestProductivity,
        improvement: currentProductivity - bestProductivity,
        timestamp: new Date(),
        context: `${Math.round(currentProductivity * 100)}% productivity score`
      };
      
      this.personalRecords.set('best_productivity', record);
      newRecords.push(record);
    }
    
    // Check deepest flow
    if (state.depth && state.depth > 0) {
      const bestFlow = this.personalRecords.get('deepest_flow')?.value || 0;
      if (state.depth > bestFlow) {
        const record: PersonalRecord = {
          type: 'deepest_flow',
          value: state.depth,
          previousBest: bestFlow,
          improvement: state.depth - bestFlow,
          timestamp: new Date(),
          context: `Flow depth of ${Math.round(state.depth * 100)}%`
        };
        
        this.personalRecords.set('deepest_flow', record);
        newRecords.push(record);
      }
    }

    return newRecords;
  }

  private updateStreaks(state: ADHDState, timestamp: Date): StreakInfo[] {
    const updatedStreaks: StreakInfo[] = [];
    
    // Daily goal streak (simplified - would need goal definition)
    const dailyGoalStreak = this.streaks.get('daily_goal') || {
      type: 'daily_goal',
      current: 0,
      best: 0,
      lastUpdate: new Date(),
      requirement: 60, // 60 minutes of focus
      nextMilestone: 7
    };
    
    const todaysFocus = this.currentSession.totalFocusTime / (60 * 1000);
    if (todaysFocus >= dailyGoalStreak.requirement) {
      if (isToday(dailyGoalStreak.lastUpdate) || isYesterday(dailyGoalStreak.lastUpdate)) {
        dailyGoalStreak.current += 1;
      } else {
        dailyGoalStreak.current = 1; // Reset streak
      }
      
      if (dailyGoalStreak.current > dailyGoalStreak.best) {
        dailyGoalStreak.best = dailyGoalStreak.current;
      }
      
      dailyGoalStreak.lastUpdate = timestamp;
      this.streaks.set('daily_goal', dailyGoalStreak);
      updatedStreaks.push(dailyGoalStreak);
    }

    return updatedStreaks;
  }

  private async analyzeTrends(): Promise<TrendAnalysis> {
    const recentDays = this.getRecentDailyMetrics(7);
    
    if (recentDays.length < 3) {
      return {
        focusTrend: 'stable',
        productivityTrend: 'stable',
        consistencyTrend: 'stable',
        timespan: 'weekly',
        confidence: 0.3,
        recommendations: ['Continue building your focus habits to see meaningful trends.']
      };
    }

    const focusTrend = this.calculateTrend(recentDays.map(d => d.totalFocusTime));
    const productivityTrend = this.calculateTrend(recentDays.map(d => d.averageSessionLength));
    const consistencyTrend = this.calculateTrend(recentDays.map(d => d.sessionCount));

    const recommendations = this.generateRecommendations(focusTrend, productivityTrend, consistencyTrend);

    return {
      focusTrend,
      productivityTrend,
      consistencyTrend,
      timespan: 'weekly',
      confidence: Math.min(recentDays.length / 7, 0.9),
      recommendations
    };
  }

  private calculateTrend(values: number[]): 'improving' | 'stable' | 'declining' {
    if (values.length < 3) return 'stable';
    
    const recent = values.slice(-3);
    const earlier = values.slice(0, -3);
    
    const recentAvg = recent.reduce((a, b) => a + b, 0) / recent.length;
    const earlierAvg = earlier.reduce((a, b) => a + b, 0) / earlier.length;
    
    const changePercent = (recentAvg - earlierAvg) / earlierAvg;
    
    if (changePercent > 0.1) return 'improving';
    if (changePercent < -0.1) return 'declining';
    return 'stable';
  }

  private generateRecommendations(
    focusTrend: TrendAnalysis['focusTrend'],
    productivityTrend: TrendAnalysis['productivityTrend'],
    consistencyTrend: TrendAnalysis['consistencyTrend']
  ): string[] {
    const recommendations: string[] = [];
    
    if (focusTrend === 'declining') {
      recommendations.push('Your focus time has been decreasing. Consider adjusting your environment or taking more breaks.');
    } else if (focusTrend === 'improving') {
      recommendations.push('Great job! Your focus time is trending upward. Keep up the excellent work!');
    }
    
    if (productivityTrend === 'declining') {
      recommendations.push('Your session quality could improve. Try shorter, more focused work blocks.');
    }
    
    if (consistencyTrend === 'declining') {
      recommendations.push('Consider establishing a more regular routine to maintain consistency.');
    }
    
    if (recommendations.length === 0) {
      recommendations.push('You\'re maintaining good focus patterns. Keep building on this foundation!');
    }
    
    return recommendations;
  }

  private checkMilestones(): Milestone[] {
    const milestones: Milestone[] = [];
    
    // Focus time milestones
    const totalFocusMinutes = this.currentSession.totalFocusTime / (60 * 1000);
    const timeMilestones = [
      { minutes: 30, name: 'Half Hour Hero', reward: 25 },
      { minutes: 60, name: 'Hour Champion', reward: 50 },
      { minutes: 120, name: 'Two Hour Warrior', reward: 100 },
      { minutes: 240, name: 'Focus Legend', reward: 250 }
    ];
    
    for (const milestone of timeMilestones) {
      if (totalFocusMinutes >= milestone.minutes && !this.hasMilestone(`session_${milestone.minutes}m`)) {
        milestones.push({
          type: 'focus_time',
          name: milestone.name,
          description: `Focused for ${milestone.minutes} minutes in one session`,
          reward: milestone.reward,
          threshold: milestone.minutes,
          unlockedAt: new Date(),
          icon: '🎯'
        });
        
        this.markMilestoneAchieved(`session_${milestone.minutes}m`);
      }
    }
    
    return milestones;
  }

  private achievedMilestones = new Set<string>();

  private hasMilestone(milestoneId: string): boolean {
    return this.achievedMilestones.has(milestoneId);
  }

  private markMilestoneAchieved(milestoneId: string): void {
    this.achievedMilestones.add(milestoneId);
  }

  private finalizePreviousSession(): void {
    if (!this.currentSession.endTime) {
      this.currentSession.endTime = new Date();
    }
    
    // Convert to daily metrics and store
    const dateKey = format(this.currentSession.startTime, 'yyyy-MM-dd');
    const existing = this.historicalData.get(dateKey);
    
    const dailyMetrics: DailyMetrics = {
      date: startOfDay(this.currentSession.startTime),
      totalFocusTime: (existing?.totalFocusTime || 0) + this.currentSession.totalFocusTime,
      sessionCount: (existing?.sessionCount || 0) + 1,
      averageSessionLength: this.currentSession.totalFocusTime / (60 * 1000), // in minutes
      bestFlowSession: this.getBestFlowSession(),
      interventionResponseRate: this.currentSession.interventionsEngaged / Math.max(this.currentSession.interventionsReceived, 1),
      productivityTrend: 'stable', // Would be calculated from recent days
      streakDays: this.streaks.get('daily_goal')?.current || 0
    };
    
    this.historicalData.set(dateKey, dailyMetrics);
    
    this.logger.info('Session finalized', {
      sessionId: this.currentSession.sessionId,
      duration: differenceInMinutes(this.currentSession.endTime, this.currentSession.startTime),
      focusMinutes: Math.round(this.currentSession.totalFocusTime / (60 * 1000)),
      productivityScore: this.currentSession.productivityScore
    });
  }

  private getBestFlowSession(): FlowSession | null {
    if (this.currentSession.flowSessions.length === 0) return null;
    
    return this.currentSession.flowSessions.reduce((best, current) => 
      current.duration > best.duration ? current : best
    );
  }

  private getRecentDailyMetrics(days: number): DailyMetrics[] {
    const recent: DailyMetrics[] = [];
    
    for (let i = 0; i < days; i++) {
      const date = subDays(new Date(), i);
      const dateKey = format(date, 'yyyy-MM-dd');
      const metrics = this.historicalData.get(dateKey);
      if (metrics) {
        recent.unshift(metrics); // Add to beginning to maintain chronological order
      }
    }
    
    return recent;
  }

  private calculateFavoriteTimeOfDay(): string {
    // This would analyze historical data to find peak performance times
    // For now, return a placeholder
    return 'morning';
  }

  private calculateImprovementRate(): number {
    const recentWeeks = this.getRecentDailyMetrics(14);
    if (recentWeeks.length < 7) return 0;
    
    const firstWeek = recentWeeks.slice(0, 7);
    const secondWeek = recentWeeks.slice(7, 14);
    
    const firstWeekAvg = firstWeek.reduce((sum, day) => sum + day.totalFocusTime, 0) / firstWeek.length;
    const secondWeekAvg = secondWeek.reduce((sum, day) => sum + day.totalFocusTime, 0) / secondWeek.length;
    
    if (firstWeekAvg === 0) return 0;
    
    return ((secondWeekAvg - firstWeekAvg) / firstWeekAvg) * 100;
  }

  private getWeeklyTrend(): 'improving' | 'stable' | 'declining' {
    const weeklyData = this.getRecentDailyMetrics(7);
    return this.calculateTrend(weeklyData.map(d => d.totalFocusTime));
  }

  private initializeStreaks(): void {
    this.streaks.set('daily_goal', {
      type: 'daily_goal',
      current: 0,
      best: 0,
      lastUpdate: new Date(),
      requirement: 60, // 60 minutes
      nextMilestone: 7
    });
    
    this.streaks.set('focus_time', {
      type: 'focus_time',
      current: 0,
      best: 0,
      lastUpdate: new Date(),
      requirement: 30, // 30 minutes
      nextMilestone: 5
    });
  }

  private loadHistoricalData(): void {
    // In a real implementation, this would load from persistent storage
    // For now, initialize empty
  }
}

/**
 * Level System Manager
 * Provides progression mechanics without being game-like
 */
class LevelSystemManager {
  private currentLevel = 1;
  private experiencePoints = 0;
  private logger: Logger;

  constructor(logger: Logger) {
    this.logger = logger;
  }

  checkLevelProgression(session: SessionMetrics, allTime: AllTimeMetrics): LevelProgress | undefined {
    const oldLevel = this.currentLevel;
    const newXP = this.calculateExperienceGain(session);
    
    this.experiencePoints += newXP;
    this.currentLevel = this.calculateLevelFromXP(this.experiencePoints);
    
    if (this.currentLevel > oldLevel) {
      this.logger.info('Level up achieved', {
        oldLevel,
        newLevel: this.currentLevel,
        totalXP: this.experiencePoints
      });

      return {
        currentLevel: this.currentLevel,
        previousLevel: oldLevel,
        experiencePoints: this.experiencePoints,
        experienceToNext: this.getXPToNextLevel(),
        rewards: this.getLevelUpRewards(this.currentLevel),
        title: this.getLevelTitle(this.currentLevel)
      };
    }

    return undefined;
  }

  getCurrentLevel(): number {
    return this.currentLevel;
  }

  private calculateExperienceGain(session: SessionMetrics): number {
    const focusMinutes = session.totalFocusTime / (60 * 1000);
    const qualityMultiplier = session.productivityScore || 0.5;
    
    return Math.floor(focusMinutes * qualityMultiplier);
  }

  private calculateLevelFromXP(xp: number): number {
    // Progressive XP requirements: 100, 250, 450, 700, 1000, etc.
    let level = 1;
    let requiredXP = 100;
    let totalXP = 0;
    
    while (totalXP + requiredXP <= xp) {
      totalXP += requiredXP;
      level++;
      requiredXP += 150; // Increase requirement by 150 each level
    }
    
    return level;
  }

  private getXPToNextLevel(): number {
    const currentLevelXP = this.calculateTotalXPForLevel(this.currentLevel);
    const nextLevelXP = this.calculateTotalXPForLevel(this.currentLevel + 1);
    
    return nextLevelXP - this.experiencePoints;
  }

  private calculateTotalXPForLevel(level: number): number {
    let totalXP = 0;
    let requiredXP = 100;
    
    for (let l = 1; l < level; l++) {
      totalXP += requiredXP;
      requiredXP += 150;
    }
    
    return totalXP;
  }

  private getLevelUpRewards(level: number): any[] {
    // Return appropriate rewards for the level
    return [
      { type: 'coins', amount: level * 50 },
      { type: 'title', item: this.getLevelTitle(level) }
    ];
  }

  private getLevelTitle(level: number): string {
    const titles = [
      'Novice',
      'Apprentice', 
      'Practitioner',
      'Adept',
      'Expert',
      'Master',
      'Grandmaster',
      'Legend'
    ];
    
    const titleIndex = Math.min(Math.floor((level - 1) / 5), titles.length - 1);
    return titles[titleIndex] || 'Novice';
  }
}