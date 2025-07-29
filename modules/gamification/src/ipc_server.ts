#!/usr/bin/env node
/**
 * IPC Server for Gamification Module
 * 
 * Handles communication with the Rust main process via stdin/stdout
 */

import { GamificationEngine } from './core/engine.js';
import { AdhdState, StateDetection } from './types/events.js';
import { logger } from './utils/logger.js';

interface TypeScriptMessage {
  module: string;
  action: string;
  payload: any;
  timestamp: number;
}

class GamificationIPCServer {
  private engine: GamificationEngine;
  private isShuttingDown = false;

  constructor() {
    this.engine = new GamificationEngine();
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
    if (stateData.state === AdhdState.Flow) {
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
      case AdhdState.Distracted:
        return 'distraction_detected';
      case AdhdState.Hyperfocus:
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