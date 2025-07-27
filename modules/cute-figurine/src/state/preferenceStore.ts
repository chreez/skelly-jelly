import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import { immer } from 'zustand/middleware/immer';
import type { CompanionPreferences, Position } from '../types';

interface PreferenceStore {
  // Core preferences
  preferences: CompanionPreferences;

  // UI preferences
  showControlPanel: boolean;
  autoHideControls: boolean;
  enableKeyboardShortcuts: boolean;
  enableSounds: boolean;

  // Performance preferences
  performanceMode: 'auto' | 'performance' | 'quality';
  frameRateTarget: number;
  enableParticles: boolean;
  maxParticleCount: number;

  // Accessibility preferences
  enableHighContrast: boolean;
  fontSize: 'small' | 'medium' | 'large';
  enableScreenReaderAnnouncements: boolean;
  keyboardNavigationEnabled: boolean;

  // Behavior preferences
  interventionFrequency: 'low' | 'medium' | 'high';
  celebrationIntensity: 'subtle' | 'normal' | 'enthusiastic';
  messageDisplayDuration: number; // seconds

  // Advanced preferences
  developerMode: boolean;
  debugAnimations: boolean;
  showPerformanceMetrics: boolean;
  enableCustomShaders: boolean;

  // Actions
  updatePreferences: (preferences: Partial<CompanionPreferences>) => void;
  setUIPreference: <
    K extends keyof Omit<
      PreferenceStore,
      | 'preferences'
      | 'updatePreferences'
      | 'setUIPreference'
      | 'resetToDefaults'
      | 'exportPreferences'
      | 'importPreferences'
    >,
  >(
    key: K,
    value: PreferenceStore[K]
  ) => void;
  resetToDefaults: () => void;
  exportPreferences: () => string;
  importPreferences: (data: string) => boolean;
}

const getDefaultPreferences = (): CompanionPreferences => ({
  position: {
    x: typeof window !== 'undefined' ? window.innerWidth - 220 : 800,
    y: typeof window !== 'undefined' ? window.innerHeight - 220 : 600,
  },
  transparency: 1,
  animationQuality: 'medium',
  reducedMotion:
    typeof window !== 'undefined'
      ? window.matchMedia('(prefers-reduced-motion: reduce)').matches
      : false,
  scale: 1,
});

const getDefaultUIPreferences = () => ({
  showControlPanel: false,
  autoHideControls: true,
  enableKeyboardShortcuts: true,
  enableSounds: true,
  performanceMode: 'auto' as const,
  frameRateTarget: 30,
  enableParticles: true,
  maxParticleCount: 100,
  enableHighContrast: false,
  fontSize: 'medium' as const,
  enableScreenReaderAnnouncements: true,
  keyboardNavigationEnabled: true,
  interventionFrequency: 'medium' as const,
  celebrationIntensity: 'normal' as const,
  messageDisplayDuration: 5,
  developerMode: false,
  debugAnimations: false,
  showPerformanceMetrics: false,
  enableCustomShaders: true,
});

export const usePreferenceStore = create<PreferenceStore>()(
  persist(
    immer((set, get) => ({
      // Initial state
      preferences: getDefaultPreferences(),
      ...getDefaultUIPreferences(),

      // Actions
      updatePreferences: (newPreferences: Partial<CompanionPreferences>) =>
        set((state) => {
          state.preferences = { ...state.preferences, ...newPreferences };
        }),

      setUIPreference: (key, value) =>
        set((state) => {
          (state as any)[key] = value;
        }),

      resetToDefaults: () =>
        set((state) => {
          state.preferences = getDefaultPreferences();
          Object.assign(state, getDefaultUIPreferences());
        }),

      exportPreferences: () => {
        const state = get();
        const exportData = {
          preferences: state.preferences,
          ui: {
            showControlPanel: state.showControlPanel,
            autoHideControls: state.autoHideControls,
            enableKeyboardShortcuts: state.enableKeyboardShortcuts,
            enableSounds: state.enableSounds,
            performanceMode: state.performanceMode,
            frameRateTarget: state.frameRateTarget,
            enableParticles: state.enableParticles,
            maxParticleCount: state.maxParticleCount,
            enableHighContrast: state.enableHighContrast,
            fontSize: state.fontSize,
            enableScreenReaderAnnouncements: state.enableScreenReaderAnnouncements,
            keyboardNavigationEnabled: state.keyboardNavigationEnabled,
            interventionFrequency: state.interventionFrequency,
            celebrationIntensity: state.celebrationIntensity,
            messageDisplayDuration: state.messageDisplayDuration,
            developerMode: state.developerMode,
            debugAnimations: state.debugAnimations,
            showPerformanceMetrics: state.showPerformanceMetrics,
            enableCustomShaders: state.enableCustomShaders,
          },
          version: '1.0.0',
          exportDate: new Date().toISOString(),
        };

        return JSON.stringify(exportData, null, 2);
      },

      importPreferences: (data: string) => {
        try {
          const importData = JSON.parse(data);

          // Validate structure
          if (!importData.preferences || !importData.ui) {
            return false;
          }

          set((state) => {
            // Import core preferences
            state.preferences = { ...getDefaultPreferences(), ...importData.preferences };

            // Import UI preferences with validation
            const uiDefaults = getDefaultUIPreferences();
            Object.keys(uiDefaults).forEach((key) => {
              if (importData.ui[key] !== undefined) {
                (state as any)[key] = importData.ui[key];
              }
            });
          });

          return true;
        } catch (error) {
          console.error('Failed to import preferences:', error);
          return false;
        }
      },
    })),
    {
      name: 'skelly-companion-preferences',
      version: 1,
    }
  )
);

// Utility functions for preference management
export const preferenceUtils = {
  /**
   * Get computed animation settings based on preferences
   */
  getAnimationSettings: () => {
    const store = usePreferenceStore.getState();
    const { preferences, performanceMode, frameRateTarget, enableParticles } = store;

    let fps = frameRateTarget;
    let quality = preferences.animationQuality;
    let enableShaders = store.enableCustomShaders;

    // Auto-adjust based on performance mode
    if (performanceMode === 'performance') {
      fps = Math.min(fps, 20);
      quality = 'low';
      enableShaders = false;
    } else if (performanceMode === 'quality') {
      fps = Math.max(fps, 30);
      quality = 'high';
    }

    return {
      fps,
      quality,
      enableShaders: enableShaders && !preferences.reducedMotion,
      reducedMotion: preferences.reducedMotion,
      enableParticles: enableParticles && !preferences.reducedMotion,
      maxParticleCount: store.maxParticleCount,
    };
  },

  /**
   * Get accessibility settings
   */
  getAccessibilitySettings: () => {
    const store = usePreferenceStore.getState();
    return {
      enableHighContrast: store.enableHighContrast,
      fontSize: store.fontSize,
      enableScreenReaderAnnouncements: store.enableScreenReaderAnnouncements,
      keyboardNavigationEnabled: store.keyboardNavigationEnabled,
      reducedMotion: store.preferences.reducedMotion,
    };
  },

  /**
   * Get interaction behavior settings
   */
  getBehaviorSettings: () => {
    const store = usePreferenceStore.getState();
    return {
      interventionFrequency: store.interventionFrequency,
      celebrationIntensity: store.celebrationIntensity,
      messageDisplayDuration: store.messageDisplayDuration * 1000, // Convert to ms
      enableSounds: store.enableSounds,
    };
  },

  /**
   * Auto-detect optimal performance settings based on device capabilities
   */
  autoDetectPerformanceSettings: () => {
    const store = usePreferenceStore.getState();

    if (store.performanceMode !== 'auto') return;

    // Simple device capability detection
    const isLowEnd = (() => {
      // Check for indicators of low-end devices
      if (typeof navigator !== 'undefined') {
        const memory = (navigator as any).deviceMemory;
        const cores = navigator.hardwareConcurrency;
        const connection = (navigator as any).connection;

        return (
          (memory && memory < 4) ||
          (cores && cores < 4) ||
          (connection &&
            (connection.effectiveType === 'slow-2g' || connection.effectiveType === '2g'))
        );
      }
      return false;
    })();

    if (isLowEnd) {
      store.setUIPreference('frameRateTarget', 15);
      store.setUIPreference('enableParticles', false);
      store.setUIPreference('maxParticleCount', 20);
      store.updatePreferences({ animationQuality: 'low' });
    }
  },

  /**
   * Validate position to ensure it's within screen bounds
   */
  validatePosition: (position: Position): Position => {
    if (typeof window === 'undefined') return position;

    const maxX = window.innerWidth - 200; // Account for companion width
    const maxY = window.innerHeight - 200; // Account for companion height

    return {
      x: Math.max(0, Math.min(position.x, maxX)),
      y: Math.max(0, Math.min(position.y, maxY)),
    };
  },
};

// Initialize performance detection on load
if (typeof window !== 'undefined') {
  // Run auto-detection after a short delay to let the page load
  setTimeout(() => {
    preferenceUtils.autoDetectPerformanceSettings();
  }, 1000);

  // Handle reduced motion preference changes
  const mediaQuery = window.matchMedia('(prefers-reduced-motion: reduce)');
  const handleReducedMotionChange = (e: MediaQueryListEvent) => {
    usePreferenceStore.getState().updatePreferences({ reducedMotion: e.matches });
  };
  mediaQuery.addEventListener('change', handleReducedMotionChange);

  // Handle window resize for position validation
  const handleResize = () => {
    const store = usePreferenceStore.getState();
    const validatedPosition = preferenceUtils.validatePosition(store.preferences.position);
    if (
      validatedPosition.x !== store.preferences.position.x ||
      validatedPosition.y !== store.preferences.position.y
    ) {
      store.updatePreferences({ position: validatedPosition });
    }
  };
  window.addEventListener('resize', handleResize);
}
