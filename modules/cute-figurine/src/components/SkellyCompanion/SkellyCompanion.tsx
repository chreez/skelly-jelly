import React, { useEffect, useRef, useState, useCallback } from 'react';
import { useCompanionStore } from '../../state/companionStore';
import { usePreferenceStore, preferenceUtils } from '../../state/preferenceStore';
import { globalEventBus } from '../../services/EventBusService';
import { globalMessageQueue } from '../../state/messageQueue';
import { AnimationCanvas } from '../AnimationCanvas';
import { TextBubble } from '../TextBubble';
import { ControlPanel } from '../ControlPanel';
import { TaskDashboard } from '../TaskDashboard';
import { PerformanceDashboard } from '../PerformanceDashboard';
import { useDragAndDrop } from '../../hooks/useDragAndDrop';
import { useAccessibility } from '../../hooks/useAccessibility';
import { usePerformanceMonitor } from '../../hooks/usePerformanceMonitor';
import { usePerformanceMonitoring } from '../../hooks/usePerformanceBenchmark';
import { createEvent, EventTypes } from '../../types/events.types';
import type { CompanionState, Position } from '../../types/state.types';

export interface SkellyCompanionProps {
  // Core properties
  id?: string;
  initialState?: Partial<CompanionState>;
  position?: Position;

  // Event handlers
  onStateChange?: (state: CompanionState) => void;
  onInteraction?: (interaction: any) => void;
  onPositionChange?: (position: Position) => void;
  onAnimationChange?: (animation: string) => void;

  // Accessibility
  ariaLabel?: string;
  role?: string;

  // Behavior configuration
  enableInteractions?: boolean;
  enableDragAndDrop?: boolean;
  enableKeyboardControls?: boolean;
  autoHideControls?: boolean;

  // Visual configuration
  showControlPanel?: boolean;
  showTaskDashboard?: boolean;
  showPerformanceDashboard?: boolean;
  showDebugInfo?: boolean;
  className?: string;
  style?: React.CSSProperties;
}

export const SkellyCompanion: React.FC<SkellyCompanionProps> = ({
  id = 'skelly-main',
  initialState,
  position: initialPosition,
  onStateChange,
  onInteraction,
  onPositionChange,
  onAnimationChange,
  ariaLabel = 'Skelly the companion',
  role = 'img',
  enableInteractions = true,
  enableDragAndDrop = true,
  enableKeyboardControls = true,
  autoHideControls = true,
  showControlPanel: controlPanelProp,
  showTaskDashboard = false,
  showPerformanceDashboard = false,
  showDebugInfo = false,
  className = '',
  style = {},
}) => {
  const containerRef = useRef<HTMLDivElement>(null);
  const [isInitialized, setIsInitialized] = useState(false);
  const [isVisible, setIsVisible] = useState(true);
  const [lastInteraction, setLastInteraction] = useState<Date>(new Date());

  // Store hooks
  const companionState = useCompanionStore();
  const preferences = usePreferenceStore();

  // Performance monitoring
  const performanceMetrics = usePerformanceMonitor(containerRef);
  const { measureRender } = usePerformanceMonitoring('SkellyCompanion');

  // Accessibility support
  const {
    announceMessage,
    handleKeyPress,
    // focusableElements,
    // currentFocusIndex,
  } = useAccessibility({
    enabled: enableKeyboardControls,
    containerRef,
    onInteraction: (type, data) => {
      handleInteraction(type, data);
    },
  });

  // Drag and drop functionality
  const { isDragging, dragPosition, dragHandlers } = useDragAndDrop({
    enabled: enableDragAndDrop,
    containerRef,
    initialPosition: companionState.position,
    onPositionChange: (newPosition) => {
      companionState.updatePosition(newPosition);
      onPositionChange?.(newPosition);
    },
    onDragStart: () => {
      handleInteraction('drag', { phase: 'start' });
    },
    onDragEnd: () => {
      handleInteraction('drag', { phase: 'end' });
    },
  });

  // Determine if control panel should be shown
  const showControlPanel = controlPanelProp ?? preferences.showControlPanel;

  // Auto-hide controls logic
  const shouldHideControls =
    autoHideControls && showControlPanel && Date.now() - lastInteraction.getTime() > 5000;

  // Initialize component
  useEffect(() => {
    // Apply initial state
    if (initialState) {
      Object.entries(initialState).forEach(([key, value]) => {
        if (key in companionState && value !== undefined) {
          const updateMethod = `update${key.charAt(0).toUpperCase()}${key.slice(1)}`;
          if (typeof companionState[updateMethod as keyof typeof companionState] === 'function') {
            (companionState[updateMethod as keyof typeof companionState] as any)(value);
          }
        }
      });
    }

    // Apply initial position
    if (initialPosition) {
      companionState.updatePosition(initialPosition);
    }

    // Set up animation config from preferences
    const animConfig = preferenceUtils.getAnimationSettings();
    companionState.setAnimationConfig(animConfig);

    // Emit system initialization event
    globalEventBus.emit(createEvent.system('initialized', { componentId: id }));

    setIsInitialized(true);
  }, []);

  // Subscribe to state changes and notify parent
  useEffect(() => {
    if (!isInitialized) return;

    const unsubscribe = useCompanionStore.subscribe(
      (state) => ({
        mood: state.mood,
        energy: state.energy,
        happiness: state.happiness,
        focus: state.focus,
        activity: state.activity,
        meltLevel: state.meltLevel,
      }),
      (currentState, previousState) => {
        if (currentState !== previousState) {
          onStateChange?.(companionState as CompanionState);

          // Announce state changes for accessibility
          if (currentState.mood !== previousState?.mood) {
            announceMessage(`Mood changed to ${currentState.mood}`);

            // Emit state change event
            globalEventBus.emit({
              type: EventTypes.COMPANION_STATE_CHANGE,
              source: id,
              timestamp: Date.now(),
              payload: {
                previousMood: previousState?.mood,
                currentMood: currentState.mood,
                trigger: 'state_update',
                timestamp: Date.now(),
              },
            });
          }
        }
      }
    );

    return unsubscribe;
  }, [isInitialized, onStateChange, announceMessage, id]);

  // Handle interaction events
  const handleInteraction = useCallback(
    (type: string, data?: any) => {
      if (!enableInteractions) return;

      setLastInteraction(new Date());

      // Emit user interaction event
      globalEventBus.emit(
        createEvent.userInteraction(type as 'click' | 'hover' | 'pet' | 'drag' | 'keyboard', {
          position: data?.position || companionState.position,
          intensity: data?.intensity,
          duration: data?.duration,
          data,
        })
      );

      // Call parent handler
      onInteraction?.({ type, data, timestamp: Date.now() });
    },
    [enableInteractions, companionState.position, onInteraction]
  );

  // Handle visibility changes (performance optimization)
  useEffect(() => {
    if (!containerRef.current) return;

    const observer = new IntersectionObserver(
      ([entry]) => {
        setIsVisible(entry.isIntersecting);
      },
      { threshold: 0.1 }
    );

    observer.observe(containerRef.current);

    return () => observer.disconnect();
  }, []);

  // Handle window focus/blur for performance
  useEffect(() => {
    const handleVisibilityChange = () => {
      const isPageVisible = !document.hidden;

      if (isPageVisible !== isVisible) {
        globalEventBus.emit(
          createEvent.system(isPageVisible ? 'resumed' : 'paused', { reason: 'visibility_change' })
        );
      }
    };

    document.addEventListener('visibilitychange', handleVisibilityChange);
    return () => document.removeEventListener('visibilitychange', handleVisibilityChange);
  }, [isVisible]);

  // Animation change handler
  const handleAnimationChange = useCallback(
    (animationName: string) => {
      onAnimationChange?.(animationName);
    },
    [onAnimationChange]
  );

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      globalEventBus.emit(createEvent.system('disposed', { componentId: id }));
    };
  }, [id]);

  // Compute final position (considering drag state)
  const finalPosition = dragPosition || companionState.position;

  // Compute container styles
  const containerStyles: React.CSSProperties = {
    position: 'fixed',
    left: finalPosition.x,
    top: finalPosition.y,
    opacity: companionState.transparency,
    transform: `scale(${companionState.scale}) rotate(${companionState.rotation}deg)`,
    cursor: isDragging ? 'grabbing' : enableDragAndDrop ? 'grab' : 'default',
    userSelect: 'none',
    zIndex: 9999,
    transition: isDragging ? 'none' : 'opacity 0.3s ease, transform 0.3s ease',
    pointerEvents: isVisible ? 'auto' : 'none',
    ...style,
  };

  if (!isInitialized) {
    return null; // Don't render until initialized
  }

  return (
    <div
      ref={containerRef}
      className={`skelly-companion-container ${className}`}
      style={containerStyles}
      role={role}
      aria-label={ariaLabel}
      aria-live="polite"
      tabIndex={enableKeyboardControls ? 0 : -1}
      onKeyDown={enableKeyboardControls ? handleKeyPress : undefined}
      onClick={() => handleInteraction('click')}
      onMouseEnter={() => handleInteraction('hover', { type: 'enter' })}
      onMouseLeave={() => handleInteraction('hover', { type: 'leave' })}
      {...(enableDragAndDrop ? dragHandlers : {})}
    >
      {/* Main animation canvas */}
      <AnimationCanvas
        width={200}
        height={200}
        visible={isVisible}
        onAnimationChange={handleAnimationChange}
        performanceMode={!isVisible ? 'paused' : 'active'}
      />

      {/* Text bubble for messages */}
      {companionState.currentMessage && (
        <TextBubble
          message={companionState.currentMessage}
          position="top"
          onDismiss={() => globalMessageQueue.clearCurrent()}
          onInteraction={() => handleInteraction('click', { target: 'message' })}
        />
      )}

      {/* Control panel */}
      {showControlPanel && !shouldHideControls && (
        <ControlPanel
          position="bottom"
          onInteraction={(type, data) => handleInteraction(type, data)}
          compact={preferences.autoHideControls}
        />
      )}

      {/* Debug information */}
      {showDebugInfo && (
        <div
          className="skelly-debug-info"
          style={{
            position: 'absolute',
            top: -40,
            left: 0,
            fontSize: '10px',
            backgroundColor: 'rgba(0,0,0,0.7)',
            color: 'white',
            padding: '4px 8px',
            borderRadius: '4px',
            whiteSpace: 'nowrap',
            pointerEvents: 'none',
          }}
        >
          {companionState.mood} | {Math.round(companionState.energy)}% |{performanceMetrics.fps}fps
          | {companionState.getCurrentAnimation()}
        </div>
      )}

      {/* Accessibility announcements */}
      <div
        className="sr-only"
        aria-live="polite"
        aria-atomic="true"
        style={{ position: 'absolute', left: '-10000px' }}
      >
        {/* Screen reader announcements will be rendered here */}
      </div>

      {/* Task Dashboard */}
      {showTaskDashboard && (
        <TaskDashboard
          visible={showTaskDashboard}
          compact={false}
          onClose={() => {
            // Handle dashboard close if needed
          }}
        />
      )}

      {/* Performance Dashboard */}
      {showPerformanceDashboard && (
        <PerformanceDashboard
          visible={showPerformanceDashboard}
          compact={false}
          onClose={() => {
            // Handle dashboard close if needed
          }}
        />
      )}
    </div>
  );
};

// Default export
export default SkellyCompanion;
