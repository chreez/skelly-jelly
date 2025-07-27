import { describe, it, expect, beforeEach, vi } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { useCompanionStore } from './companionStore';
import { MoodState, ActivityState } from '../types';

// Mock localStorage
const localStorageMock = {
  getItem: vi.fn(),
  setItem: vi.fn(),
  removeItem: vi.fn(),
  clear: vi.fn(),
};
global.localStorage = localStorageMock as any;

// Mock window properties
Object.defineProperty(window, 'innerWidth', {
  writable: true,
  configurable: true,
  value: 1024,
});

Object.defineProperty(window, 'innerHeight', {
  writable: true,
  configurable: true,
  value: 768,
});

describe('companionStore', () => {
  beforeEach(() => {
    // Reset store state before each test
    useCompanionStore.setState(
      {
        id: 'skelly-main',
        name: 'Skelly',
        mood: MoodState.HAPPY,
        energy: 80,
        happiness: 75,
        focus: 50,
        meltLevel: 0,
        glowIntensity: 0.5,
        particleCount: 0,
        activity: ActivityState.IDLE,
        lastInteraction: new Date(),
        interventionCooldown: 0,
        position: { x: window.innerWidth - 220, y: window.innerHeight - 220 },
        scale: 1,
        rotation: 0,
        transparency: 1,
        animationConfig: {
          fps: 30,
          quality: 'medium',
          enableShaders: true,
          reducedMotion: false,
        },
        currentMessage: null,
        messageQueue: [],
      },
      true
    );

    vi.clearAllMocks();
  });

  describe('Initial State', () => {
    it('initializes with correct default values', () => {
      const { result } = renderHook(() => useCompanionStore());

      expect(result.current.mood).toBe(MoodState.HAPPY);
      expect(result.current.energy).toBe(80);
      expect(result.current.happiness).toBe(75);
      expect(result.current.focus).toBe(50);
      expect(result.current.meltLevel).toBe(0);
      expect(result.current.activity).toBe(ActivityState.IDLE);
    });

    it('sets initial position based on window size', () => {
      const { result } = renderHook(() => useCompanionStore());

      expect(result.current.position.x).toBe(window.innerWidth - 220);
      expect(result.current.position.y).toBe(window.innerHeight - 220);
    });
  });

  describe('Mood Updates', () => {
    it('updates mood and applies side effects', () => {
      const { result } = renderHook(() => useCompanionStore());

      act(() => {
        result.current.updateMood(MoodState.EXCITED);
      });

      expect(result.current.mood).toBe(MoodState.EXCITED);
      expect(result.current.glowIntensity).toBe(0.8);
      expect(result.current.particleCount).toBe(50);
      expect(result.current.energy).toBeGreaterThan(80);
    });

    it('applies melting effects when mood is melting', () => {
      const { result } = renderHook(() => useCompanionStore());

      act(() => {
        result.current.updateMood(MoodState.MELTING);
      });

      expect(result.current.mood).toBe(MoodState.MELTING);
      expect(result.current.meltLevel).toBeGreaterThan(0);
      expect(result.current.glowIntensity).toBe(0.2);
      expect(result.current.energy).toBeLessThan(80);
    });

    it('applies focused effects when mood is focused', () => {
      const { result } = renderHook(() => useCompanionStore());

      act(() => {
        result.current.updateMood(MoodState.FOCUSED);
      });

      expect(result.current.mood).toBe(MoodState.FOCUSED);
      expect(result.current.glowIntensity).toBe(0.3);
      expect(result.current.focus).toBeGreaterThan(50);
    });
  });

  describe('Energy Management', () => {
    it('updates energy within valid range', () => {
      const { result } = renderHook(() => useCompanionStore());

      act(() => {
        result.current.updateEnergy(120); // Above max
      });
      expect(result.current.energy).toBe(100);

      act(() => {
        result.current.updateEnergy(-10); // Below min
      });
      expect(result.current.energy).toBe(0);

      act(() => {
        result.current.updateEnergy(50); // Valid range
      });
      expect(result.current.energy).toBe(50);
    });

    it('auto-adjusts mood based on energy levels', () => {
      const { result } = renderHook(() => useCompanionStore());

      // Low energy should trigger tired mood
      act(() => {
        result.current.updateEnergy(15);
      });
      expect(result.current.mood).toBe(MoodState.TIRED);
      expect(result.current.meltLevel).toBeGreaterThan(0);

      // High energy should recover from tired mood
      act(() => {
        result.current.updateMood(MoodState.TIRED);
        result.current.updateEnergy(85);
      });
      expect(result.current.mood).toBe(MoodState.HAPPY);
    });

    it('affects glow intensity based on energy', () => {
      const { result } = renderHook(() => useCompanionStore());

      act(() => {
        result.current.updateEnergy(100);
      });
      expect(result.current.glowIntensity).toBeGreaterThan(0.5);

      act(() => {
        result.current.updateEnergy(20);
      });
      expect(result.current.glowIntensity).toBeLessThan(0.3);
    });
  });

  describe('Melt Level Management', () => {
    it('constrains melt level to valid range', () => {
      const { result } = renderHook(() => useCompanionStore());

      act(() => {
        result.current.updateMeltLevel(150); // Above max
      });
      expect(result.current.meltLevel).toBe(100);

      act(() => {
        result.current.updateMeltLevel(-10); // Below min
      });
      expect(result.current.meltLevel).toBe(0);
    });

    it('forces melting mood when melt level is high', () => {
      const { result } = renderHook(() => useCompanionStore());

      act(() => {
        result.current.updateMeltLevel(75);
      });

      expect(result.current.mood).toBe(MoodState.MELTING);
      expect(result.current.energy).toBeLessThan(80);
    });

    it('recovers from melting mood when melt level is low', () => {
      const { result } = renderHook(() => useCompanionStore());

      // Set melting state
      act(() => {
        result.current.updateMood(MoodState.MELTING);
        result.current.updateEnergy(90);
        result.current.updateMeltLevel(15);
      });

      expect(result.current.mood).toBe(MoodState.HAPPY);
    });
  });

  describe('Activity Management', () => {
    it('updates activity and applies side effects', () => {
      const { result } = renderHook(() => useCompanionStore());

      act(() => {
        result.current.setActivity(ActivityState.WORKING);
      });

      expect(result.current.activity).toBe(ActivityState.WORKING);
      expect(result.current.focus).toBeGreaterThan(50);
    });

    it('applies resting benefits', () => {
      const { result } = renderHook(() => useCompanionStore());
      const initialEnergy = result.current.energy;
      const initialMeltLevel = result.current.meltLevel;

      act(() => {
        result.current.setActivity(ActivityState.RESTING);
      });

      expect(result.current.energy).toBeGreaterThan(initialEnergy);
      expect(result.current.meltLevel).toBeLessThanOrEqual(initialMeltLevel);
    });

    it('applies interaction benefits', () => {
      const { result } = renderHook(() => useCompanionStore());
      const initialHappiness = result.current.happiness;

      act(() => {
        result.current.setActivity(ActivityState.INTERACTING);
      });

      expect(result.current.happiness).toBeGreaterThan(initialHappiness);
    });
  });

  describe('Position Management', () => {
    it('updates position correctly', () => {
      const { result } = renderHook(() => useCompanionStore());
      const newPosition = { x: 100, y: 200 };

      act(() => {
        result.current.updatePosition(newPosition);
      });

      expect(result.current.position).toEqual(newPosition);
    });

    it('updates partial position', () => {
      const { result } = renderHook(() => useCompanionStore());
      const originalPosition = result.current.position;

      act(() => {
        result.current.updatePosition({ x: 300 });
      });

      expect(result.current.position.x).toBe(300);
      expect(result.current.position.y).toBe(originalPosition.y);
    });
  });

  describe('Message Queue', () => {
    it('queues messages correctly', () => {
      const { result } = renderHook(() => useCompanionStore());
      const message = {
        text: 'Test message',
        duration: 3000,
        priority: 2,
      };

      act(() => {
        result.current.queueMessage(message);
      });

      expect(result.current.currentMessage).toEqual(
        expect.objectContaining({
          text: 'Test message',
          duration: 3000,
          priority: 2,
        })
      );
    });

    it('sorts messages by priority', () => {
      const { result } = renderHook(() => useCompanionStore());

      const lowPriorityMessage = { text: 'Low', priority: 1 };
      const highPriorityMessage = { text: 'High', priority: 3 };

      act(() => {
        result.current.queueMessage(lowPriorityMessage);
        result.current.queueMessage(highPriorityMessage);
      });

      // First message should be the low priority one (already processing)
      expect(result.current.currentMessage?.text).toBe('Low');

      // Clear current message to process queue
      act(() => {
        result.current.clearMessage();
      });

      // Next message should be high priority
      expect(result.current.currentMessage?.text).toBe('High');
    });

    it('clears messages correctly', () => {
      const { result } = renderHook(() => useCompanionStore());

      act(() => {
        result.current.queueMessage({ text: 'Test', priority: 1 });
      });

      expect(result.current.currentMessage).not.toBeNull();

      act(() => {
        result.current.clearMessage();
      });

      expect(result.current.currentMessage).toBeNull();
    });
  });

  describe('Animation Management', () => {
    it('returns correct animation based on state', () => {
      const { result } = renderHook(() => useCompanionStore());

      // Test default state
      expect(result.current.getCurrentAnimation()).toBe('idle_breathing');

      // Test high melt level
      act(() => {
        result.current.updateMeltLevel(75);
      });
      expect(result.current.getCurrentAnimation()).toBe('melting_heavy');

      // Test focused working
      act(() => {
        result.current.updateMeltLevel(0);
        result.current.updateMood(MoodState.FOCUSED);
        result.current.setActivity(ActivityState.WORKING);
      });
      expect(result.current.getCurrentAnimation()).toBe('focused_breathing');
    });

    it('prioritizes melting animations', () => {
      const { result } = renderHook(() => useCompanionStore());

      act(() => {
        result.current.updateMood(MoodState.EXCITED);
        result.current.updateMeltLevel(50);
      });

      expect(result.current.getCurrentAnimation()).toBe('melting_medium');
    });
  });

  describe('Intervention System', () => {
    it('records interactions correctly', () => {
      const { result } = renderHook(() => useCompanionStore());
      const initialHappiness = result.current.happiness;

      act(() => {
        result.current.recordInteraction();
      });

      expect(result.current.happiness).toBeGreaterThan(initialHappiness);
      expect(result.current.interventionCooldown).toBe(30);
      expect(result.current.lastInteraction).toBeInstanceOf(Date);
    });

    it('manages intervention cooldown', () => {
      const { result } = renderHook(() => useCompanionStore());

      act(() => {
        result.current.updateInterventionCooldown(15);
      });

      expect(result.current.interventionCooldown).toBe(15);
      expect(result.current.canIntervene()).toBe(false);

      act(() => {
        result.current.updateInterventionCooldown(0);
      });

      expect(result.current.canIntervene()).toBe(true);
    });

    it('prevents intervention during resting', () => {
      const { result } = renderHook(() => useCompanionStore());

      act(() => {
        result.current.setActivity(ActivityState.RESTING);
        result.current.updateInterventionCooldown(0);
      });

      expect(result.current.canIntervene()).toBe(false);
    });
  });

  describe('Computed Values', () => {
    it('calculates energy level correctly', () => {
      const { result } = renderHook(() => useCompanionStore());

      act(() => {
        result.current.updateEnergy(25);
      });
      expect(result.current.getEnergyLevel()).toBe('low');

      act(() => {
        result.current.updateEnergy(50);
      });
      expect(result.current.getEnergyLevel()).toBe('medium');

      act(() => {
        result.current.updateEnergy(75);
      });
      expect(result.current.getEnergyLevel()).toBe('high');
    });

    it('calculates mood intensity correctly', () => {
      const { result } = renderHook(() => useCompanionStore());

      // High energy and happiness should give higher intensity
      act(() => {
        result.current.updateEnergy(90);
        result.current.updateHappiness(90);
      });

      const highIntensity = result.current.getMoodIntensity();

      // Low energy and happiness should give lower intensity
      act(() => {
        result.current.updateEnergy(20);
        result.current.updateHappiness(20);
      });

      const lowIntensity = result.current.getMoodIntensity();

      expect(highIntensity).toBeGreaterThan(lowIntensity);
    });

    it('applies mood-specific intensity modifiers', () => {
      const { result } = renderHook(() => useCompanionStore());

      // Set base values
      act(() => {
        result.current.updateEnergy(70);
        result.current.updateHappiness(70);
      });

      const baseIntensity = result.current.getMoodIntensity();

      // Excited mood should increase intensity
      act(() => {
        result.current.updateMood(MoodState.EXCITED);
      });

      const excitedIntensity = result.current.getMoodIntensity();
      expect(excitedIntensity).toBeGreaterThan(baseIntensity);

      // Melting mood should decrease intensity
      act(() => {
        result.current.updateMood(MoodState.MELTING);
      });

      const meltingIntensity = result.current.getMoodIntensity();
      expect(meltingIntensity).toBeLessThan(baseIntensity);
    });
  });

  describe('Reset Functions', () => {
    it('resets to defaults correctly', () => {
      const { result } = renderHook(() => useCompanionStore());

      // Modify state
      act(() => {
        result.current.updateMood(MoodState.MELTING);
        result.current.updateEnergy(20);
        result.current.updateMeltLevel(80);
      });

      // Reset
      act(() => {
        result.current.resetToDefaults();
      });

      expect(result.current.mood).toBe(MoodState.HAPPY);
      expect(result.current.energy).toBe(80);
      expect(result.current.meltLevel).toBe(0);
    });

    it('resets melt level correctly', () => {
      const { result } = renderHook(() => useCompanionStore());

      act(() => {
        result.current.updateMood(MoodState.MELTING);
        result.current.updateMeltLevel(80);
      });

      expect(result.current.mood).toBe(MoodState.MELTING);

      act(() => {
        result.current.resetMeltLevel();
      });

      expect(result.current.meltLevel).toBe(0);
      expect(result.current.mood).not.toBe(MoodState.MELTING);
    });
  });
});
