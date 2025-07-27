import { create } from 'zustand';
import { subscribeWithSelector } from 'zustand/middleware';
import { immer } from 'zustand/middleware/immer';
import { persist } from 'zustand/middleware';
import type { CompanionState, Position, Message, CompanionPreferences } from '../types';
import { MoodState, ActivityState } from '../types';

interface AnimationConfig {
  fps: number;
  quality: 'low' | 'medium' | 'high';
  enableShaders: boolean;
  reducedMotion: boolean;
}

interface CompanionStore extends CompanionState {
  // Configuration
  animationConfig: AnimationConfig;
  preferences: CompanionPreferences;

  // Message handling
  currentMessage: Message | null;
  messageQueue: Message[];

  // State updates
  updateMood: (mood: MoodState) => void;
  updateEnergy: (energy: number) => void;
  updateHappiness: (happiness: number) => void;
  updateFocus: (focus: number) => void;
  updateMeltLevel: (level: number) => void;
  updateGlowIntensity: (intensity: number) => void;
  setActivity: (activity: ActivityState) => void;

  // Position management
  updatePosition: (position: Partial<Position>) => void;
  savePosition: () => void;

  // Animation config
  setAnimationConfig: (config: Partial<AnimationConfig>) => void;

  // Message handling
  queueMessage: (message: Message) => void;
  clearMessage: () => void;
  processMessageQueue: () => void;

  // Interaction handling
  recordInteraction: () => void;
  updateInterventionCooldown: (seconds: number) => void;

  // Computed getters
  canIntervene: () => boolean;
  getCurrentAnimation: () => string;
  getEnergyLevel: () => 'low' | 'medium' | 'high';
  getMoodIntensity: () => number;

  // Preferences
  updatePreferences: (preferences: Partial<CompanionPreferences>) => void;

  // Reset functions
  resetToDefaults: () => void;
  resetMeltLevel: () => void;
}

const getDefaultState = (): CompanionState => ({
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
  position: {
    x: typeof window !== 'undefined' ? window.innerWidth - 220 : 800,
    y: typeof window !== 'undefined' ? window.innerHeight - 220 : 600,
  },
  scale: 1,
  rotation: 0,
  transparency: 1,
});

const getDefaultPreferences = (): CompanionPreferences => ({
  position: { x: 800, y: 600 },
  transparency: 1,
  animationQuality: 'medium',
  reducedMotion: false,
  scale: 1,
});

const getDefaultAnimationConfig = (): AnimationConfig => ({
  fps: 30,
  quality: 'medium',
  enableShaders: true,
  reducedMotion:
    typeof window !== 'undefined'
      ? window.matchMedia('(prefers-reduced-motion: reduce)').matches
      : false,
});

export const useCompanionStore = create<CompanionStore>()(
  persist(
    subscribeWithSelector(
      immer((set, get) => ({
        // Initial state
        ...getDefaultState(),
        animationConfig: getDefaultAnimationConfig(),
        preferences: getDefaultPreferences(),
        currentMessage: null,
        messageQueue: [],

        // State update actions
        updateMood: (mood: MoodState) =>
          set((state) => {
            state.mood = mood;

            // Mood-specific side effects
            switch (mood) {
              case MoodState.MELTING:
                state.meltLevel = Math.min(state.meltLevel + 20, 100);
                state.glowIntensity = 0.2;
                state.energy = Math.max(state.energy - 10, 0);
                break;
              case MoodState.EXCITED:
                state.glowIntensity = 0.8;
                state.particleCount = 50;
                state.energy = Math.min(state.energy + 5, 100);
                break;
              case MoodState.FOCUSED:
                state.glowIntensity = 0.3;
                state.focus = Math.min(state.focus + 10, 100);
                break;
              case MoodState.TIRED:
                state.glowIntensity = 0.1;
                state.meltLevel = Math.min(state.meltLevel + 5, 60);
                break;
              case MoodState.CELEBRATING:
                state.glowIntensity = 1.0;
                state.particleCount = 100;
                state.happiness = Math.min(state.happiness + 15, 100);
                break;
              case MoodState.HAPPY:
                state.glowIntensity = 0.5;
                state.particleCount = 20;
                state.meltLevel = Math.max(state.meltLevel - 5, 0);
                break;
            }
          }),

        updateEnergy: (energy: number) =>
          set((state) => {
            const newEnergy = Math.max(0, Math.min(100, energy));
            state.energy = newEnergy;

            // Auto-adjust mood and melt based on energy
            if (
              newEnergy < 20 &&
              state.mood !== MoodState.TIRED &&
              state.mood !== MoodState.MELTING
            ) {
              state.mood = MoodState.TIRED;
              state.meltLevel = Math.min(state.meltLevel + 10, 80);
            } else if (newEnergy > 80 && state.mood === MoodState.TIRED) {
              state.mood = MoodState.HAPPY;
              state.meltLevel = Math.max(state.meltLevel - 20, 0);
            }

            // Energy affects glow intensity
            state.glowIntensity = Math.max(0.1, (newEnergy / 100) * 0.7);
          }),

        updateHappiness: (happiness: number) =>
          set((state) => {
            state.happiness = Math.max(0, Math.min(100, happiness));

            // High happiness reduces melt level
            if (state.happiness > 80) {
              state.meltLevel = Math.max(state.meltLevel - 3, 0);
            }
          }),

        updateFocus: (focus: number) =>
          set((state) => {
            state.focus = Math.max(0, Math.min(100, focus));

            // High focus improves mood
            if (state.focus > 75 && state.mood !== MoodState.FOCUSED) {
              state.mood = MoodState.FOCUSED;
            }
          }),

        updateMeltLevel: (level: number) =>
          set((state) => {
            const newLevel = Math.max(0, Math.min(100, level));
            state.meltLevel = newLevel;

            // High melt level forces melting mood
            if (newLevel > 70 && state.mood !== MoodState.MELTING) {
              state.mood = MoodState.MELTING;
              state.energy = Math.max(state.energy - 15, 0);
            } else if (newLevel < 20 && state.mood === MoodState.MELTING) {
              state.mood = state.energy > 60 ? MoodState.HAPPY : MoodState.TIRED;
            }
          }),

        updateGlowIntensity: (intensity: number) =>
          set((state) => {
            state.glowIntensity = Math.max(0, Math.min(1, intensity));
          }),

        setActivity: (activity: ActivityState) =>
          set((state) => {
            state.activity = activity;

            // Activity-specific effects
            switch (activity) {
              case ActivityState.WORKING:
                if (state.focus < 70) {
                  state.focus = Math.min(state.focus + 5, 100);
                }
                break;
              case ActivityState.RESTING:
                state.energy = Math.min(state.energy + 2, 100);
                state.meltLevel = Math.max(state.meltLevel - 1, 0);
                break;
              case ActivityState.INTERACTING:
                state.happiness = Math.min(state.happiness + 3, 100);
                break;
            }
          }),

        // Position management
        updatePosition: (position: Partial<Position>) =>
          set((state) => {
            state.position = { ...state.position, ...position };
          }),

        savePosition: () => {
          const { position } = get();
          set((state) => {
            state.preferences.position = { ...position };
          });
        },

        // Animation configuration
        setAnimationConfig: (config: Partial<AnimationConfig>) =>
          set((state) => {
            state.animationConfig = { ...state.animationConfig, ...config };
          }),

        // Message handling
        queueMessage: (message: Message) =>
          set((state) => {
            const messageWithId = {
              ...message,
              id: message.id || `msg_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
              duration: message.duration || 5000,
              priority: message.priority || 1,
            };

            // Add to queue sorted by priority
            state.messageQueue.push(messageWithId);
            state.messageQueue.sort((a, b) => (b.priority || 1) - (a.priority || 1));

            // Process queue if no current message
            if (!state.currentMessage) {
              get().processMessageQueue();
            }
          }),

        clearMessage: () =>
          set((state) => {
            state.currentMessage = null;
            // Process next message in queue
            setTimeout(() => get().processMessageQueue(), 100);
          }),

        processMessageQueue: () =>
          set((state) => {
            if (state.messageQueue.length > 0 && !state.currentMessage) {
              state.currentMessage = state.messageQueue.shift() || null;

              // Auto-clear message after duration
              if (state.currentMessage) {
                setTimeout(() => {
                  get().clearMessage();
                }, state.currentMessage.duration || 5000);
              }
            }
          }),

        // Interaction handling
        recordInteraction: () =>
          set((state) => {
            state.lastInteraction = new Date();
            state.happiness = Math.min(state.happiness + 5, 100);
            state.interventionCooldown = 30; // 30 second cooldown
          }),

        updateInterventionCooldown: (seconds: number) =>
          set((state) => {
            state.interventionCooldown = Math.max(0, seconds);
          }),

        // Computed getters
        canIntervene: () => {
          const { activity, interventionCooldown } = get();
          return interventionCooldown === 0 && activity !== ActivityState.RESTING;
        },

        getCurrentAnimation: () => {
          const { mood, activity, meltLevel, energy } = get();

          // Priority-based animation selection
          if (meltLevel > 70) return 'melting_heavy';
          if (meltLevel > 40) return 'melting_medium';
          if (activity === ActivityState.WORKING && mood === MoodState.FOCUSED)
            return 'focused_breathing';
          if (mood === MoodState.CELEBRATING) return 'celebration_dance';
          if (mood === MoodState.EXCITED) return 'happy_bounce';
          if (mood === MoodState.TIRED || energy < 30) return 'tired_sway';
          if (activity === ActivityState.INTERACTING) return 'interaction_wave';

          return 'idle_breathing';
        },

        getEnergyLevel: () => {
          const { energy } = get();
          if (energy < 33) return 'low';
          if (energy < 67) return 'medium';
          return 'high';
        },

        getMoodIntensity: () => {
          const { mood, energy, happiness } = get();
          const baseIntensity = (energy + happiness) / 200;

          // Mood-specific intensity modifiers
          switch (mood) {
            case MoodState.EXCITED:
            case MoodState.CELEBRATING:
              return Math.min(baseIntensity * 1.5, 1);
            case MoodState.MELTING:
            case MoodState.TIRED:
              return Math.max(baseIntensity * 0.5, 0.1);
            case MoodState.FOCUSED:
              return Math.min(baseIntensity * 1.2, 0.8);
            default:
              return baseIntensity;
          }
        },

        // Preferences
        updatePreferences: (preferences: Partial<CompanionPreferences>) =>
          set((state) => {
            state.preferences = { ...state.preferences, ...preferences };

            // Apply some preferences immediately
            if (preferences.transparency !== undefined) {
              state.transparency = preferences.transparency;
            }
            if (preferences.scale !== undefined) {
              state.scale = preferences.scale;
            }
            if (preferences.position) {
              state.position = { ...preferences.position };
            }
          }),

        // Reset functions
        resetToDefaults: () =>
          set((state) => {
            const defaults = getDefaultState();
            Object.assign(state, defaults);
            state.animationConfig = getDefaultAnimationConfig();
            state.preferences = getDefaultPreferences();
            state.currentMessage = null;
            state.messageQueue = [];
          }),

        resetMeltLevel: () =>
          set((state) => {
            state.meltLevel = 0;
            if (state.mood === MoodState.MELTING) {
              state.mood = state.energy > 60 ? MoodState.HAPPY : MoodState.TIRED;
            }
          }),
      }))
    ),
    {
      name: 'skelly-companion-store',
      partialize: (state) => ({
        preferences: state.preferences,
        position: state.position,
        animationConfig: state.animationConfig,
        mood: state.mood,
        energy: state.energy,
        happiness: state.happiness,
        focus: state.focus,
      }),
    }
  )
);

// Store subscriptions for side effects
if (typeof window !== 'undefined') {
  // Update intervention cooldown every second
  setInterval(() => {
    const store = useCompanionStore.getState();
    if (store.interventionCooldown > 0) {
      store.updateInterventionCooldown(store.interventionCooldown - 1);
    }
  }, 1000);

  // Save position when it changes
  useCompanionStore.subscribe(
    (state) => state.position,
    () => {
      const store = useCompanionStore.getState();
      store.savePosition();
    }
  );

  // Handle reduced motion preference changes
  const mediaQuery = window.matchMedia('(prefers-reduced-motion: reduce)');
  const handleReducedMotionChange = (e: MediaQueryListEvent) => {
    useCompanionStore.getState().setAnimationConfig({ reducedMotion: e.matches });
  };
  mediaQuery.addEventListener('change', handleReducedMotionChange);
}
