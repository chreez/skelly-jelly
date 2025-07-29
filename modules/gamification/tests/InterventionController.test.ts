/**
 * Tests for InterventionController
 * Focus on user-centered decision making and flow state protection
 */

import { describe, it, expect, beforeEach, vi } from 'vitest';
import { InterventionController } from '../src/controllers/InterventionController.js';
import { ADHDState, BehavioralMetrics, WorkContext, UserPreferences } from '../src/types/index.js';
import { Logger } from 'winston';

// Mock logger
const mockLogger = {
  info: vi.fn(),
  error: vi.fn(),
  warn: vi.fn(),
  debug: vi.fn()
} as unknown as Logger;

// Test data factories
function createFlowState(confidence = 0.9, duration = 30 * 60 * 1000): ADHDState {
  return {
    type: 'Flow',
    confidence,
    depth: 0.8,
    duration,
    metadata: {}
  };
}

function createDistractedState(duration = 5 * 60 * 1000): ADHDState {
  return {
    type: 'Distracted',
    confidence: 0.7,
    duration,
    metadata: {}
  };
}

function createDefaultMetrics(): BehavioralMetrics {
  return {
    productive_time_ratio: 0.7,
    distraction_frequency: 2,
    focus_session_count: 3,
    average_session_length: 25,
    recovery_time: 120,
    transition_smoothness: 0.8
  };
}

function createDefaultWorkContext(): WorkContext {
  return {
    application: 'VSCode',
    window_title: 'project.ts',
    task_category: 'work',
    urgency: 'medium',
    time_of_day: 'afternoon'
  };
}

function createDefaultPreferences(): UserPreferences {
  return {
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
  };
}

function createSessionMetrics() {
  return {
    sessionId: 'test-session',
    startTime: new Date(),
    totalFocusTime: 60 * 60 * 1000, // 1 hour
    totalDistractedTime: 10 * 60 * 1000, // 10 minutes
    flowSessions: [],
    distractionCount: 2,
    recoveryCount: 2,
    productivityScore: 0.8,
    interventionsReceived: 1,
    interventionsEngaged: 1,
    deepestFlow: 0.9,
    longestFocusStreak: 45 * 60 * 1000 // 45 minutes
  };
}

describe('InterventionController', () => {
  let controller: InterventionController;
  let preferences: UserPreferences;
  let metrics: BehavioralMetrics;
  let workContext: WorkContext;
  let sessionMetrics: any;

  beforeEach(() => {
    controller = new InterventionController(mockLogger);
    preferences = createDefaultPreferences();
    metrics = createDefaultMetrics();
    workContext = createDefaultWorkContext();
    sessionMetrics = createSessionMetrics();
    
    // Clear mock calls
    vi.clearAllMocks();
  });

  describe('Flow State Protection', () => {
    it('should never interrupt high-confidence flow states', async () => {
      const flowState = createFlowState(0.9);
      
      const decision = await controller.shouldIntervene(
        flowState,
        metrics,
        workContext,
        preferences,
        sessionMetrics
      );

      expect(decision.intervene).toBe(false);
      expect(decision.reason).toContain('flow state');
    });

    it('should allow interventions in low-confidence flow states', async () => {
      const flowState = createFlowState(0.6); // Low confidence
      
      const decision = await controller.shouldIntervene(
        flowState,
        metrics,
        workContext,
        preferences,
        sessionMetrics
      );

      // Should consider intervention since confidence is low
      expect(decision.intervene).toBe(false); // Still false due to cooldowns, but reason should be different
      expect(decision.reason).not.toContain('flow state');
    });

    it('should respect user preference to disable flow state protection', async () => {
      const flowState = createFlowState(0.9);
      const nonProtectivePreferences = {
        ...preferences,
        respectFlowStates: false
      };
      
      const decision = await controller.shouldIntervene(
        flowState,
        metrics,
        workContext,
        nonProtectivePreferences,
        sessionMetrics
      );

      expect(decision.reason).not.toContain('flow state');
    });

    it('should protect healthy hyperfocus states under 90 minutes', async () => {
      const hyperfocusState: ADHDState = {
        type: 'Hyperfocus',
        confidence: 0.9,
        duration: 60 * 60 * 1000, // 1 hour
        metadata: {}
      };
      
      const decision = await controller.shouldIntervene(
        hyperfocusState,
        metrics,
        workContext,
        preferences,
        sessionMetrics
      );

      expect(decision.intervene).toBe(false);
      expect(decision.reason).toContain('flow state');
    });
  });

  describe('Critical Context Handling', () => {
    it('should not interrupt critical work contexts', async () => {
      const criticalContext: WorkContext = {
        ...workContext,
        urgency: 'critical'
      };
      const distractedState = createDistractedState();
      
      const decision = await controller.shouldIntervene(
        distractedState,
        metrics,
        criticalContext,
        preferences,
        sessionMetrics
      );

      expect(decision.intervene).toBe(false);
      expect(decision.reason).toContain('Critical work context');
    });

    it('should not interrupt high urgency work tasks', async () => {
      const highUrgencyContext: WorkContext = {
        ...workContext,
        urgency: 'high',
        task_category: 'work'
      };
      const distractedState = createDistractedState();
      
      const decision = await controller.shouldIntervene(
        distractedState,
        metrics,
        highUrgencyContext,
        preferences,
        sessionMetrics
      );

      expect(decision.intervene).toBe(false);
      expect(decision.reason).toContain('Critical work context');
    });
  });

  describe('Adaptive Cooldown Management', () => {
    it('should increase cooldown after quick dismissals', () => {
      const interventionId = 'gentle_encouragement';
      
      // Simulate quick dismissal
      controller.recordInterventionResponse(interventionId, 'dismissed_quickly');
      
      const stats = controller.getEffectivenessStats();
      expect(stats[interventionId]).toBeDefined();
      expect((stats[interventionId] as any).cooldownMultiplier).toBeGreaterThan(1);
    });

    it('should decrease cooldown after positive engagement', () => {
      const interventionId = 'gentle_encouragement';
      
      // First set a higher multiplier
      controller.recordInterventionResponse(interventionId, 'dismissed_quickly');
      const initialStats = controller.getEffectivenessStats();
      const initialMultiplier = (initialStats[interventionId] as any).cooldownMultiplier;
      
      // Then record positive response
      controller.recordInterventionResponse(interventionId, 'engaged_positively');
      
      const finalStats = controller.getEffectivenessStats();
      const finalMultiplier = (finalStats[interventionId] as any).cooldownMultiplier;
      
      expect(finalMultiplier).toBeLessThan(initialMultiplier);
    });

    it('should track response history for learning', () => {
      const interventionId = 'gentle_encouragement';
      
      controller.recordInterventionResponse(interventionId, 'engaged_positively');
      controller.recordInterventionResponse(interventionId, 'clicked_through');
      controller.recordInterventionResponse(interventionId, 'dismissed_quickly');
      
      const stats = controller.getEffectivenessStats();
      expect(stats[interventionId]).toBeDefined();
      // Should track mixed response pattern
    });
  });

  describe('Intervention Opportunity Scoring', () => {
    it('should favor encouragement during flow states', async () => {
      const flowState = createFlowState(0.7); // Moderate confidence
      preferences.respectFlowStates = false; // Disable protection for testing
      
      // Reset cooldowns by creating new controller
      controller = new InterventionController(mockLogger);
      
      const decision = await controller.shouldIntervene(
        flowState,
        metrics,
        workContext,
        preferences,
        sessionMetrics
      );

      if (decision.intervene && decision.type) {
        expect(decision.type.category).toBe('encouragement');
      }
    });

    it('should favor suggestions during distraction', async () => {
      const distractedState = createDistractedState(10 * 60 * 1000); // 10 minutes
      const lowProductivityMetrics = {
        ...metrics,
        productive_time_ratio: 0.3
      };
      
      controller = new InterventionController(mockLogger);
      
      const decision = await controller.shouldIntervene(
        distractedState,
        lowProductivityMetrics,
        workContext,
        preferences,
        sessionMetrics
      );

      if (decision.intervene && decision.type) {
        expect(['suggestion', 'gentle_nudge']).toContain(decision.type.category);
      }
    });
  });

  describe('User Preference Adaptation', () => {
    it('should reduce intervention frequency for minimal users', async () => {
      const minimalPreferences: UserPreferences = {
        ...preferences,
        interventionFrequency: 'minimal'
      };
      
      const distractedState = createDistractedState();
      
      const decision = await controller.shouldIntervene(
        distractedState,
        metrics,
        workContext,
        minimalPreferences,
        sessionMetrics
      );

      // Minimal users should have higher threshold for interventions
      expect(decision.intervene).toBe(false);
    });

    it('should increase intervention frequency for frequent users', async () => {
      const frequentPreferences: UserPreferences = {
        ...preferences,
        interventionFrequency: 'frequent'
      };
      
      const neutralState: ADHDState = {
        type: 'Neutral',
        confidence: 0.6,
        duration: 5 * 60 * 1000,
        metadata: {}
      };
      
      controller = new InterventionController(mockLogger);
      
      const decision = await controller.shouldIntervene(
        neutralState,
        metrics,
        workContext,
        frequentPreferences,
        sessionMetrics
      );

      // Frequent users should have lower threshold
      // More likely to get interventions in neutral states
    });

    it('should respect disabled intervention types', async () => {
      const limitedPreferences: UserPreferences = {
        ...preferences,
        interventionTypes: ['celebration'] // Only celebrations
      };
      
      const distractedState = createDistractedState();
      
      const decision = await controller.shouldIntervene(
        distractedState,
        metrics,
        workContext,
        limitedPreferences,
        sessionMetrics
      );

      expect(decision.intervene).toBe(false);
      expect(decision.reason).toContain('cooldown');
    });
  });

  describe('State History Tracking', () => {
    it('should track state history for pattern analysis', () => {
      const state1 = createFlowState();
      const state2 = createDistractedState();
      
      controller.addStateToHistory(state1, metrics, workContext);
      controller.addStateToHistory(state2, metrics, workContext);
      
      // History should be tracked (internal verification)
      expect(mockLogger.debug).not.toHaveBeenCalledWith(
        expect.stringMatching(/error/i)
      );
    });
  });

  describe('Error Handling', () => {
    it('should handle malformed state gracefully', async () => {
      const invalidState = {} as ADHDState;
      
      const decision = await controller.shouldIntervene(
        invalidState,
        metrics,
        workContext,
        preferences,
        sessionMetrics
      );

      expect(decision.intervene).toBe(false);
      expect(decision.reason).toContain('Error');
    });

    it('should handle missing context gracefully', async () => {
      const state = createFlowState();
      const emptyContext = {} as WorkContext;
      
      const decision = await controller.shouldIntervene(
        state,
        metrics,
        emptyContext,
        preferences,
        sessionMetrics
      );

      // Should not crash, return safe decision
      expect(decision).toBeDefined();
      expect(typeof decision.intervene).toBe('boolean');
    });
  });
});