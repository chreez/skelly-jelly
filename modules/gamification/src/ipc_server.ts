#!/usr/bin/env node
/**
 * IPC Server for Gamification Module
 * 
 * Handles communication with the Rust main process via stdin/stdout
 */

import { MotivationEngine } from './engines/MotivationEngine.js';
import { ADHDState, Achievement, GamificationConfig } from './types/index.js';

// Define StateDetection based on usage in the code
interface StateDetection {
  state: ADHDState['type'];
  confidence: number;
}

// Define GameState interface
interface GameState {
  focusCoins: number;
  achievementsUnlocked: Achievement[];
  currentLevel: number;
  totalFocusTime: number;
}

// Simple logger implementation
const logger = {
  info: (msg: string, ...args: any[]) => console.log(`[INFO] ${msg}`, ...args),
  error: (msg: string, ...args: any[]) => console.error(`[ERROR] ${msg}`, ...args),
  warn: (msg: string, ...args: any[]) => console.warn(`[WARN] ${msg}`, ...args),
  debug: (msg: string, ...args: any[]) => console.log(`[DEBUG] ${msg}`, ...args),
};

// Simple config for MotivationEngine
const defaultConfig: GamificationConfig = {
  intervention: {
    minCooldownMinutes: 5,
    adaptiveCooldown: true,
    maxInterventionsPerHour: 10,
    respectFlowStates: true,
    flowStateThreshold: 0.7,
    emergencyOverride: false,
  },
  rewards: {
    coinsPerFocusMinute: 1,
    bonusMultiplier: 1.5,
    achievementCoins: {},
    variableRatioBase: 0.5,
    streakBonusMultiplier: 2.0,
    milestoneRewards: {},
  },
  progress: {
    sessionTimeoutMinutes: 30,
    streakRequirementDays: 7,
    milestoneThresholds: [100, 500, 1000],
    metricUpdateInterval: 60,
    historyRetentionDays: 90,
  },
  companion: {
    animationDuration: 1000,
    expressionVariety: true,
    idleVariations: true,
    reactionSensitivity: 0.7,
    personalityTraits: {
      cheerfulness: 0.8,
      humor: 0.6,
      formality: 0.4,
      supportiveness: 0.9,
    },
  },
  messages: {
    maxLength: 200,
    personalizedGeneration: true,
    templateVariety: true,
    adaptiveTone: true,
    contextAwareness: true,
  },
  performance: {
    maxHistoryEntries: 1000,
    batchUpdateSize: 50,
    cacheTimeout: 300,
    animationQueueSize: 10,
  },
};

// Gamification Engine Adapter - wraps MotivationEngine and adds missing methods
class GamificationEngineAdapter {
  private motivationEngine: MotivationEngine;
  private gameState: GameState;
  private interventionCooldown: number = 0;
  private lastInterventionTime: number = 0;

  constructor() {
    this.motivationEngine = new MotivationEngine(logger as any, defaultConfig);
    this.gameState = {
      focusCoins: 0,
      achievementsUnlocked: [],
      currentLevel: 1,
      totalFocusTime: 0,
    };
  }

  async processStateDetection(stateData: StateDetection): Promise<void> {
    // Process state detection - in a full implementation this would
    // update internal state and trigger appropriate responses
    console.log(`Processing state detection: ${stateData.state} (${stateData.confidence})`);
  }

  getGameState(): GameState {
    return { ...this.gameState };
  }

  shouldIntervene(stateData: StateDetection): boolean {
    const now = Date.now();
    const timeSinceLastIntervention = now - this.lastInterventionTime;
    
    // Respect cooldown period
    if (timeSinceLastIntervention < this.interventionCooldown) {
      return false;
    }

    // Don't intervene during flow states unless emergency
    if (stateData.state === 'Flow' && stateData.confidence > 0.7) {
      return false;
    }

    // Intervene for sustained distractions
    if (stateData.state === 'Distracted' && stateData.confidence > 0.6) {
      return true;
    }

    // Intervene for hyperfocus to suggest breaks
    if (stateData.state === 'Hyperfocus' && stateData.confidence > 0.8) {
      return true;
    }

    return false;
  }

  getInterventionCooldown(): number {
    return Math.max(0, this.interventionCooldown - (Date.now() - this.lastInterventionTime));
  }

  async awardFocusCoins(): Promise<number> {
    const coinsAwarded = Math.floor(Math.random() * 5) + 1; // 1-5 coins
    this.gameState.focusCoins += coinsAwarded;
    return coinsAwarded;
  }

  async awardCoins(amount: number, reason: string): Promise<number> {
    this.gameState.focusCoins += amount;
    console.log(`Awarded ${amount} coins for: ${reason}`);
    return this.gameState.focusCoins;
  }

  async checkAchievements(): Promise<Achievement[]> {
    // Simplified achievement checking - in a full implementation this would
    // check various criteria and unlock achievements
    const newAchievements: Achievement[] = [];
    
    // Example: First 100 coins achievement
    if (this.gameState.focusCoins >= 100 && 
        !this.gameState.achievementsUnlocked.some(a => a.id === 'first_hundred')) {
      const achievement: Achievement = {
        id: 'first_hundred',
        name: 'First Hundred',
        description: 'Earned your first 100 focus coins',
        icon: '💰',
        rarity: 'common',
        category: 'milestone',
        unlockedAt: new Date(),
        rewards: [{ type: 'coins', amount: 50 }],
        hidden: false,
      };
      this.gameState.achievementsUnlocked.push(achievement);
      newAchievements.push(achievement);
    }

    return newAchievements;
  }
}

interface TypeScriptMessage {
  module: string;
  action: string;
  payload: any;
  timestamp: number;
}

class GamificationIPCServer {
  private engine: GamificationEngineAdapter;
  private isShuttingDown = false;

  constructor() {
    this.engine = new GamificationEngineAdapter();
    this.setupProcessHandlers();
    this.startListening();
  }

  private setupProcessHandlers(): void {
    process.on('SIGINT', () => this.shutdown());
    process.on('SIGTERM', () => this.shutdown());
    process.on('uncaughtException', (error) => {
      logger.error('Uncaught exception in Gamification IPC:', error);
      this.shutdown();
    });
  }

  private startListening(): void {
    logger.info('🎮 Gamification IPC Server starting...');
    
    // Listen for messages from Rust via stdin
    process.stdin.setEncoding('utf8');
    process.stdin.on('data', (data: string) => {
      try {
        const lines = data.trim().split('\n');
        for (const line of lines) {
          if (line.trim()) {
            const message: TypeScriptMessage = JSON.parse(line);
            this.handleMessage(message);
          }
        }
      } catch (error) {
        logger.error('Failed to parse IPC message:', error);
      }
    });

    process.stdin.on('end', () => {
      logger.info('Stdin closed, shutting down...');
      this.shutdown();
    });

    logger.info('✅ Gamification IPC Server ready');
  }

  private async handleMessage(message: TypeScriptMessage): Promise<void> {
    if (this.isShuttingDown) return;

    logger.debug(`📨 Received: ${message.action}`, message.payload);

    try {
      switch (message.action) {
        case 'state_detected':
          await this.handleStateDetection(message.payload);
          break;
          
        case 'get_game_state':
          await this.sendGameState();
          break;
          
        case 'award_coins':
          await this.awardCoins(message.payload.amount, message.payload.reason);
          break;
          
        case 'check_achievements':
          await this.checkAchievements();
          break;
          
        case 'shutdown':
          this.shutdown();
          break;
          
        default:
          logger.warn(`Unknown action: ${message.action}`);
      }
    } catch (error) {
      logger.error(`Error handling ${message.action}:`, error);
    }
  }

  private async handleStateDetection(stateData: StateDetection): Promise<void> {
    logger.info(`🧠 Processing state: ${stateData.state} (confidence: ${stateData.confidence})`);
    
    // Update engine with new state
    await this.engine.processStateDetection(stateData);
    
    // Get current game state and send updates
    const gameState = this.engine.getGameState();
    
    // Send intervention decision back to Rust
    const interventionNeeded = this.engine.shouldIntervene(stateData);
    if (interventionNeeded) {
      this.sendMessage({
        module: 'gamification',
        action: 'intervention_recommended',
        payload: {
          state: stateData,
          reason: this.getInterventionReason(stateData),
          cooldown_remaining: this.engine.getInterventionCooldown()
        },
        timestamp: Date.now()
      });
    }
    
    // Award coins for focus states
    if (stateData.state === 'Flow') {
      const coinsAwarded = await this.engine.awardFocusCoins();
      if (coinsAwarded > 0) {
        this.sendMessage({
          module: 'gamification',
          action: 'coins_awarded',
          payload: {
            amount: coinsAwarded,
            reason: 'flow_state',
            total_coins: gameState.focusCoins
          },
          timestamp: Date.now()
        });
      }
    }
    
    // Check for achievements
    const newAchievements = await this.engine.checkAchievements();
    if (newAchievements.length > 0) {
      this.sendMessage({
        module: 'gamification',
        action: 'achievements_unlocked',
        payload: {
          achievements: newAchievements,
          total_unlocked: gameState.achievementsUnlocked.length
        },
        timestamp: Date.now()
      });
    }
  }

  private async sendGameState(): Promise<void> {
    const gameState = this.engine.getGameState();
    this.sendMessage({
      module: 'gamification',
      action: 'game_state_update',
      payload: gameState,
      timestamp: Date.now()
    });
  }

  private async awardCoins(amount: number, reason: string): Promise<void> {
    const newTotal = await this.engine.awardCoins(amount, reason);
    this.sendMessage({
      module: 'gamification',
      action: 'coins_awarded',
      payload: {
        amount,
        reason,
        total_coins: newTotal
      },
      timestamp: Date.now()
    });
  }

  private async checkAchievements(): Promise<void> {
    const newAchievements = await this.engine.checkAchievements();
    if (newAchievements.length > 0) {
      this.sendMessage({
        module: 'gamification',
        action: 'achievements_unlocked',
        payload: {
          achievements: newAchievements
        },
        timestamp: Date.now()
      });
    }
  }

  private getInterventionReason(state: StateDetection): string {
    switch (state.state) {
      case 'Distracted':
        return 'distraction_detected';
      case 'Hyperfocus':
        return 'break_reminder';
      default:
        return 'general_support';
    }
  }

  private sendMessage(message: TypeScriptMessage): void {
    if (this.isShuttingDown) return;
    
    try {
      const json = JSON.stringify(message);
      process.stdout.write(json + '\n');
      logger.debug(`📤 Sent: ${message.action}`);
    } catch (error) {
      logger.error('Failed to send IPC message:', error);
    }
  }

  private shutdown(): void {
    if (this.isShuttingDown) return;
    this.isShuttingDown = true;
    
    logger.info('🛑 Shutting down Gamification IPC Server...');
    
    // Send final state update
    this.sendMessage({
      module: 'gamification',
      action: 'shutdown_complete',
      payload: {},
      timestamp: Date.now()
    });
    
    // Give time for message to be sent
    setTimeout(() => {
      logger.info('✅ Gamification IPC Server shutdown complete');
      process.exit(0);
    }, 100);
  }
}

// Start the server
new GamificationIPCServer();