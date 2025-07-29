/**
 * Skelly-Jelly Gamification Module
 * 
 * Non-intrusive motivational system for ADHD users.
 * Provides ambient support without disrupting focus states.
 * 
 * @author Skelly-Jelly Team
 * @version 0.1.0
 */

// Main module exports
export {
  GamificationModule,
  GamificationModuleImpl,
  createGamificationModule,
  EventBusInterface,
  EventBusMessage
} from './GamificationModule.js';

// Core controllers and systems
export { InterventionController } from './controllers/InterventionController.js';
export { RewardSystem } from './systems/RewardSystem.js';
export { ProgressTracker } from './trackers/ProgressTracker.js';
export { CompanionBehaviorManager } from './managers/CompanionBehaviorManager.js';
export { MotivationEngine } from './engines/MotivationEngine.js';

// Type definitions
export * from './types/index.js';

// Configuration helpers
export const DEFAULT_CONFIG = {
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

// User profile templates
export const USER_PROFILES = {
  minimal: {
    preferences: {
      interventionFrequency: 'minimal' as const,
      animationIntensity: 'minimal' as const,
      messageStyle: 'minimal' as const,
      celebrationStyle: 'subtle' as const
    }
  },
  moderate: {
    preferences: {
      interventionFrequency: 'moderate' as const,
      animationIntensity: 'moderate' as const,
      messageStyle: 'encouraging' as const,
      celebrationStyle: 'noticeable' as const
    }
  },
  engaging: {
    preferences: {
      interventionFrequency: 'frequent' as const,
      animationIntensity: 'full' as const,
      messageStyle: 'encouraging' as const,
      celebrationStyle: 'enthusiastic' as const
    }
  }
};

// Utility functions
export function createMinimalConfig() {
  return {
    ...DEFAULT_CONFIG,
    intervention: {
      ...DEFAULT_CONFIG.intervention,
      minCooldownMinutes: 30,
      maxInterventionsPerHour: 2
    },
    companion: {
      ...DEFAULT_CONFIG.companion,
      personalityTraits: {
        cheerfulness: 0.4,
        humor: 0.2,
        formality: 0.7,
        supportiveness: 0.6
      }
    }
  };
}

export function createEngagingConfig() {
  return {
    ...DEFAULT_CONFIG,
    intervention: {
      ...DEFAULT_CONFIG.intervention,
      minCooldownMinutes: 10,
      maxInterventionsPerHour: 5
    },
    rewards: {
      ...DEFAULT_CONFIG.rewards,
      bonusMultiplier: 2.0,
      variableRatioBase: 0.25
    },
    companion: {
      ...DEFAULT_CONFIG.companion,
      personalityTraits: {
        cheerfulness: 0.9,
        humor: 0.8,
        formality: 0.1,
        supportiveness: 0.9
      }
    }
  };
}

// Version info
export const VERSION = '0.1.0';
export const MODULE_NAME = 'gamification';
export const COMPATIBLE_VERSIONS = {
  'event-bus': '^0.1.0',
  'analysis-engine': '^0.1.0',
  'ai-integration': '^0.1.0',
  'cute-figurine': '^0.1.0'
};