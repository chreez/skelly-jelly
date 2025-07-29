/**
 * Basic Usage Example for Gamification Module
 * 
 * Demonstrates how to set up and use the gamification system
 * for supporting ADHD users with non-intrusive motivation.
 */

import { createGamificationModule, DEFAULT_CONFIG, USER_PROFILES } from '../src/index.js';
import { Logger } from 'winston';
import { v4 as uuidv4 } from 'uuid';

// Mock event bus for demonstration
class MockEventBus {
  private handlers = new Map<string, Function[]>();
  
  async subscribe(messageType: string, handler: Function): Promise<string> {
    const subscriptionId = uuidv4();
    
    if (!this.handlers.has(messageType)) {
      this.handlers.set(messageType, []);
    }
    
    this.handlers.get(messageType)!.push(handler);
    console.log(`📡 Subscribed to ${messageType} events`);
    
    return subscriptionId;
  }
  
  async publish(messageType: string, payload: any): Promise<void> {
    console.log(`📤 Publishing ${messageType}:`, payload);
    
    const handlers = this.handlers.get(messageType) || [];
    for (const handler of handlers) {
      try {
        await handler({
          id: uuidv4(),
          type: messageType,
          payload,
          timestamp: new Date(),
          source: 'mock'
        });
      } catch (error) {
        console.error(`Error handling ${messageType}:`, error);
      }
    }
  }
  
  async unsubscribe(subscriptionId: string): Promise<void> {
    console.log(`📡 Unsubscribed: ${subscriptionId}`);
  }
}

// Simple console logger
const logger = {
  info: (msg: string, meta?: any) => console.log(`ℹ️  ${msg}`, meta || ''),
  error: (msg: string, meta?: any) => console.error(`❌ ${msg}`, meta || ''),
  warn: (msg: string, meta?: any) => console.warn(`⚠️  ${msg}`, meta || ''),
  debug: (msg: string, meta?: any) => console.log(`🔍 ${msg}`, meta || '')
} as Logger;

async function demonstrateGamificationModule() {
  console.log('🦴 Skelly-Jelly Gamification Module Demo\n');
  
  // Create mock event bus
  const eventBus = new MockEventBus();
  
  // Create gamification module with moderate user profile
  const gamificationModule = createGamificationModule(
    eventBus,
    logger,
    DEFAULT_CONFIG,
    {
      id: uuidv4(),
      name: 'Demo User',
      ...USER_PROFILES.moderate
    }
  );
  
  try {
    // Start the module
    console.log('🚀 Starting gamification module...\n');
    await gamificationModule.start();
    
    // Simulate a series of state changes throughout a work session
    console.log('📊 Simulating work session state changes...\n');
    
    // 1. User starts working (neutral to transitioning)
    await simulateStateChange(eventBus, {
      previousState: {
        type: 'Neutral',
        confidence: 0.5,
        duration: 0
      },
      currentState: {
        type: 'Transitioning',
        confidence: 0.6,
        duration: 2 * 60 * 1000 // 2 minutes
      },
      metrics: {
        productive_time_ratio: 0.3,
        distraction_frequency: 0,
        focus_session_count: 0,
        average_session_length: 0,
        recovery_time: 0,
        transition_smoothness: 0.7
      }
    });
    
    // Wait a moment for processing
    await sleep(1000);
    
    // 2. User enters flow state
    await simulateStateChange(eventBus, {
      previousState: {
        type: 'Transitioning',
        confidence: 0.6,
        duration: 2 * 60 * 1000
      },
      currentState: {
        type: 'Flow',
        confidence: 0.9,
        depth: 0.8,
        duration: 25 * 60 * 1000 // 25 minutes of focus
      },
      metrics: {
        productive_time_ratio: 0.8,
        distraction_frequency: 1,
        focus_session_count: 1,
        average_session_length: 25,
        recovery_time: 120,
        transition_smoothness: 0.9
      }
    });
    
    await sleep(1000);
    
    // 3. User gets distracted
    await simulateStateChange(eventBus, {
      previousState: {
        type: 'Flow',
        confidence: 0.9,
        depth: 0.8,
        duration: 25 * 60 * 1000
      },
      currentState: {
        type: 'Distracted',
        confidence: 0.7,
        duration: 5 * 60 * 1000 // 5 minutes distracted
      },
      metrics: {
        productive_time_ratio: 0.75,
        distraction_frequency: 2,
        focus_session_count: 1,
        average_session_length: 22,
        recovery_time: 300,
        transition_smoothness: 0.6
      }
    });
    
    await sleep(1000);
    
    // 4. User recovers to flow
    await simulateStateChange(eventBus, {
      previousState: {
        type: 'Distracted',
        confidence: 0.7,
        duration: 5 * 60 * 1000
      },
      currentState: {
        type: 'Flow',
        confidence: 0.8,
        depth: 0.7,
        duration: 35 * 60 * 1000 // 35 minutes of focus
      },
      metrics: {
        productive_time_ratio: 0.82,
        distraction_frequency: 2,
        focus_session_count: 2,
        average_session_length: 30,
        recovery_time: 180,
        transition_smoothness: 0.8
      }
    });
    
    await sleep(1000);
    
    // Check progress
    console.log('\n📈 Getting progress summary...\n');
    const progress = await gamificationModule.getProgress();
    
    console.log('Session Progress:');
    console.log(`  Focus Time: ${Math.round(progress.session.totalFocusTime / (60 * 1000))} minutes`);
    console.log(`  Productivity Score: ${Math.round(progress.session.productivityScore * 100)}%`);
    console.log(`  Flow Sessions: ${progress.session.flowSessions.length}`);
    console.log(`  Recovery Count: ${progress.session.recoveryCount}`);
    
    console.log('\nWallet:');
    console.log(`  Coins: ${progress.wallet.coins}`);
    console.log(`  Total Earned: ${progress.wallet.totalEarned}`);
    
    console.log('\nAchievements:');
    progress.achievements.forEach(achievement => {
      console.log(`  🏆 ${achievement.name}: ${achievement.description}`);
    });
    
    console.log('\nStreaks:');
    progress.streaks.forEach(streak => {
      console.log(`  🔥 ${streak.type}: ${streak.current} (best: ${streak.best})`);
    });
    
    // Test manual intervention
    console.log('\n🎯 Testing manual intervention...\n');
    const interventionResult = await gamificationModule.triggerIntervention('encouragement');
    console.log(`Manual intervention triggered: ${interventionResult}`);
    
    await sleep(1000);
    
    // Get module metrics
    console.log('\n📊 Module Performance Metrics:\n');
    const metrics = await gamificationModule.getMetrics();
    console.log('Performance:');
    console.log(`  Interventions Delivered: ${metrics.interventionsDelivered}`);
    console.log(`  Rewards Granted: ${metrics.rewardsGranted}`);
    console.log(`  Achievements Unlocked: ${metrics.achievementsUnlocked}`);
    console.log(`  Engagement Rate: ${Math.round(metrics.interventionEngagementRate * 100)}%`);
    console.log(`  Average Response Time: ${Math.round(metrics.averageResponseTime)}ms`);
    console.log(`  Memory Usage: ${Math.round(metrics.systemPerformance.memoryUsage)}MB`);
    
    // Test preference updates
    console.log('\n⚙️  Testing preference updates...\n');
    await gamificationModule.updatePreferences({
      interventionFrequency: 'minimal',
      celebrationStyle: 'subtle',
      messageStyle: 'minimal'
    });
    console.log('Preferences updated to minimal mode');
    
  } catch (error) {
    console.error('Demo error:', error);
  } finally {
    // Clean shutdown
    console.log('\n🛑 Stopping gamification module...\n');
    await gamificationModule.stop();
    console.log('✅ Demo completed successfully!');
  }
}

// Helper functions
async function simulateStateChange(eventBus: MockEventBus, stateChange: any) {
  const event = {
    ...stateChange,
    transitionTime: new Date(),
    confidence: stateChange.currentState.confidence,
    sessionId: uuidv4(),
    workContext: {
      application: 'VSCode',
      window_title: 'demo.ts',
      task_category: 'work' as const,
      urgency: 'medium' as const,
      time_of_day: 'afternoon' as const
    }
  };
  
  console.log(`🔄 State change: ${stateChange.previousState.type} → ${stateChange.currentState.type}`);
  await eventBus.publish('StateChange', event);
}

function sleep(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms));
}

// Run the demo
if (import.meta.url === `file://${process.argv[1]}`) {
  demonstrateGamificationModule().catch(console.error);
}