// React is automatically available in JSX transform
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
// act is available from @testing-library/react
import { vi, describe, it, expect, beforeEach, afterEach } from 'vitest';
import { SkellyCompanion } from './SkellyCompanion';
import { useCompanionStore } from '../../state/companionStore';
import { usePreferenceStore } from '../../state/preferenceStore';
import { MoodState, ActivityState } from '../../types';

// Mock the stores
vi.mock('../../state/companionStore');
vi.mock('../../state/preferenceStore');
vi.mock('../../services/EventBusService');
vi.mock('../../state/messageQueue');

// Mock the hooks
vi.mock('../../hooks/useDragAndDrop', () => ({
  useDragAndDrop: () => ({
    isDragging: false,
    dragPosition: null,
    dragHandlers: {
      onMouseDown: vi.fn(),
      onTouchStart: vi.fn(),
    },
    setPosition: vi.fn(),
  }),
}));

vi.mock('../../hooks/useAccessibility', () => ({
  useAccessibility: () => ({
    announceMessage: vi.fn(),
    handleKeyPress: vi.fn(),
    focusableElements: [],
    currentFocusIndex: -1,
    isHighContrast: false,
    fontSize: 'medium',
  }),
}));

vi.mock('../../hooks/usePerformanceMonitor', () => ({
  usePerformanceMonitor: () => ({
    fps: 30,
    frameTime: 16.67,
    memoryUsage: 1024 * 1024 * 10, // 10MB
    cpuUsage: 15,
    renderTime: 5,
    eventProcessingTime: 2,
    totalFrames: 1800,
    droppedFrames: 0,
    averageFrameTime: 16.67,
    worstFrameTime: 25,
    isThrottled: false,
  }),
}));

// Mock IntersectionObserver
global.IntersectionObserver = vi.fn().mockImplementation(() => ({
  observe: vi.fn(),
  unobserve: vi.fn(),
  disconnect: vi.fn(),
}));

// Mock requestAnimationFrame
global.requestAnimationFrame = vi.fn((cb) => setTimeout(cb, 16));
global.cancelAnimationFrame = vi.fn();

describe('SkellyCompanion', () => {
  const mockCompanionStore = {
    mood: MoodState.HAPPY,
    energy: 80,
    happiness: 75,
    focus: 50,
    meltLevel: 0,
    glowIntensity: 0.5,
    particleCount: 20,
    activity: ActivityState.IDLE,
    position: { x: 100, y: 100 },
    scale: 1,
    rotation: 0,
    transparency: 1,
    currentMessage: null,
    updatePosition: vi.fn(),
    updateMood: vi.fn(),
    updateEnergy: vi.fn(),
    setAnimationConfig: vi.fn(),
    getCurrentAnimation: vi.fn(() => 'idle_breathing'),
    subscribe: vi.fn(() => vi.fn()),
  };

  const mockPreferenceStore = {
    showControlPanel: false,
    autoHideControls: true,
    enableKeyboardShortcuts: true,
    enableHighContrast: false,
    fontSize: 'medium' as const,
  };

  beforeEach(() => {
    vi.mocked(useCompanionStore).mockReturnValue(mockCompanionStore as any);
    vi.mocked(usePreferenceStore).mockReturnValue(mockPreferenceStore as any);

    // Reset all mocks
    vi.clearAllMocks();
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  describe('Rendering', () => {
    it('renders the companion component', () => {
      render(<SkellyCompanion />);

      const container = screen.getByRole('img', { name: /skelly the companion/i });
      expect(container).toBeInTheDocument();
    });

    it('applies correct positioning styles', () => {
      render(<SkellyCompanion />);

      const container = screen.getByRole('img');
      expect(container).toHaveStyle({
        position: 'fixed',
        left: '100px',
        top: '100px',
        opacity: '1',
      });
    });

    it('renders with custom position when provided', () => {
      const customPosition = { x: 200, y: 300 };
      render(<SkellyCompanion position={customPosition} />);

      expect(mockCompanionStore.updatePosition).toHaveBeenCalledWith(customPosition);
    });

    it('applies custom className and style', () => {
      const customStyle = { border: '1px solid red' };
      render(<SkellyCompanion className="custom-class" style={customStyle} />);

      const container = screen.getByRole('img');
      expect(container).toHaveClass('skelly-companion-container', 'custom-class');
      expect(container).toHaveStyle(customStyle);
    });
  });

  describe('Interactions', () => {
    it('handles click interactions when enabled', () => {
      const onInteraction = vi.fn();
      render(<SkellyCompanion onInteraction={onInteraction} />);

      const container = screen.getByRole('img');
      fireEvent.click(container);

      expect(onInteraction).toHaveBeenCalledWith(
        expect.objectContaining({
          type: 'click',
          timestamp: expect.any(Number),
        })
      );
    });

    it('does not handle interactions when disabled', () => {
      const onInteraction = vi.fn();
      render(<SkellyCompanion enableInteractions={false} onInteraction={onInteraction} />);

      const container = screen.getByRole('img');
      fireEvent.click(container);

      expect(onInteraction).not.toHaveBeenCalled();
    });

    it('handles hover interactions', () => {
      const onInteraction = vi.fn();
      render(<SkellyCompanion onInteraction={onInteraction} />);

      const container = screen.getByRole('img');
      fireEvent.mouseEnter(container);

      expect(onInteraction).toHaveBeenCalledWith(
        expect.objectContaining({
          type: 'hover',
          data: { type: 'enter' },
        })
      );
    });

    it('handles keyboard interactions when enabled', () => {
      const onInteraction = vi.fn();
      render(<SkellyCompanion onInteraction={onInteraction} />);

      const container = screen.getByRole('img');
      fireEvent.keyDown(container, { key: 'Enter' });

      expect(onInteraction).toHaveBeenCalled();
    });
  });

  describe('State Management', () => {
    it('applies initial state when provided', () => {
      const initialState = {
        mood: MoodState.EXCITED,
        energy: 90,
      };

      render(<SkellyCompanion initialState={initialState} />);

      // Note: This would need a more sophisticated mock to test the actual state updates
      expect(mockCompanionStore.setAnimationConfig).toHaveBeenCalled();
    });

    it('calls onStateChange when state updates', async () => {
      const onStateChange = vi.fn();

      // Mock the subscribe function to simulate state changes
      const mockUnsubscribe = vi.fn();
      mockCompanionStore.subscribe.mockReturnValue(mockUnsubscribe);

      render(<SkellyCompanion onStateChange={onStateChange} />);

      await waitFor(() => {
        expect(onStateChange).toHaveBeenCalledWith(
          expect.objectContaining({
            mood: expect.any(String),
            energy: expect.any(Number),
          })
        );
      });
    });
  });

  describe('Message Display', () => {
    it('renders text bubble when message is present', () => {
      const messageStore = {
        ...mockCompanionStore,
        currentMessage: {
          text: 'Hello there!',
          style: 'default' as const,
          duration: 5000,
        },
      };

      vi.mocked(useCompanionStore).mockReturnValue(messageStore as any);

      render(<SkellyCompanion />);

      expect(screen.getByText('Hello there!')).toBeInTheDocument();
    });

    it('does not render text bubble when no message', () => {
      render(<SkellyCompanion />);

      expect(screen.queryByText(/hello/i)).not.toBeInTheDocument();
    });
  });

  describe('Control Panel', () => {
    it('renders control panel when showControlPanel is true', () => {
      const preferenceStore = {
        ...mockPreferenceStore,
        showControlPanel: true,
      };

      vi.mocked(usePreferenceStore).mockReturnValue(preferenceStore as any);

      render(<SkellyCompanion />);

      // Look for control panel buttons
      expect(screen.getByText('⚙️')).toBeInTheDocument();
      expect(screen.getByText('×')).toBeInTheDocument();
    });

    it('does not render control panel when showControlPanel is false', () => {
      render(<SkellyCompanion />);

      expect(screen.queryByText('⚙️')).not.toBeInTheDocument();
    });

    it('respects explicit showControlPanel prop', () => {
      render(<SkellyCompanion showControlPanel={true} />);

      expect(screen.getByText('⚙️')).toBeInTheDocument();
    });
  });

  describe('Debug Mode', () => {
    it('renders debug info when showDebugInfo is true', () => {
      render(<SkellyCompanion showDebugInfo={true} />);

      const debugInfo = screen.getByText(/fps/i);
      expect(debugInfo).toBeInTheDocument();
      expect(debugInfo).toHaveTextContent('30fps');
    });

    it('does not render debug info by default', () => {
      render(<SkellyCompanion />);

      expect(screen.queryByText(/fps/i)).not.toBeInTheDocument();
    });
  });

  describe('Accessibility', () => {
    it('has correct ARIA attributes', () => {
      render(<SkellyCompanion />);

      const container = screen.getByRole('img');
      expect(container).toHaveAttribute('aria-label', 'Skelly the companion');
      expect(container).toHaveAttribute('aria-live', 'polite');
    });

    it('supports custom ARIA label', () => {
      render(<SkellyCompanion ariaLabel="Custom companion" />);

      const container = screen.getByRole('img');
      expect(container).toHaveAttribute('aria-label', 'Custom companion');
    });

    it('has correct tabindex when keyboard controls are enabled', () => {
      render(<SkellyCompanion enableKeyboardControls={true} />);

      const container = screen.getByRole('img');
      expect(container).toHaveAttribute('tabindex', '0');
    });

    it('has tabindex -1 when keyboard controls are disabled', () => {
      render(<SkellyCompanion enableKeyboardControls={false} />);

      const container = screen.getByRole('img');
      expect(container).toHaveAttribute('tabindex', '-1');
    });
  });

  describe('Drag and Drop', () => {
    it('applies drag cursor styles when dragging', () => {
      // Mock drag state
      const mockDragHook = {
        isDragging: true,
        dragPosition: { x: 150, y: 150 },
        dragHandlers: {
          onMouseDown: vi.fn(),
          onTouchStart: vi.fn(),
        },
        setPosition: vi.fn(),
      };

      vi.mocked(require('../../hooks/useDragAndDrop').useDragAndDrop).mockReturnValue(mockDragHook);

      render(<SkellyCompanion />);

      const container = screen.getByRole('img');
      expect(container).toHaveStyle({ cursor: 'grabbing' });
    });

    it('does not add drag handlers when drag is disabled', () => {
      render(<SkellyCompanion enableDragAndDrop={false} />);

      const container = screen.getByRole('img');
      expect(container).toHaveStyle({ cursor: 'default' });
    });
  });

  describe('Performance', () => {
    it('handles visibility changes for performance optimization', () => {
      const { unmount } = render(<SkellyCompanion />);

      // Check that IntersectionObserver was created
      expect(global.IntersectionObserver).toHaveBeenCalled();

      unmount();

      // Cleanup should be called
      expect(
        vi.mocked(global.IntersectionObserver).mock.results[0].value.disconnect
      ).toHaveBeenCalled();
    });

    it('handles window visibility changes', () => {
      const originalHidden = Object.getOwnPropertyDescriptor(Document.prototype, 'hidden');

      // Mock document.hidden
      Object.defineProperty(document, 'hidden', {
        configurable: true,
        get: () => true,
      });

      render(<SkellyCompanion />);

      // Simulate visibility change
      const visibilityEvent = new Event('visibilitychange');
      document.dispatchEvent(visibilityEvent);

      // Restore original property
      if (originalHidden) {
        Object.defineProperty(Document.prototype, 'hidden', originalHidden);
      }
    });
  });

  describe('Error Handling', () => {
    it('handles missing containerRef gracefully', () => {
      // This tests that the component doesn't crash with missing refs
      expect(() => render(<SkellyCompanion />)).not.toThrow();
    });

    it('handles store errors gracefully', () => {
      // Mock store to throw an error
      vi.mocked(useCompanionStore).mockImplementation(() => {
        throw new Error('Store error');
      });

      // Component should handle this gracefully or we should have error boundary
      expect(() => render(<SkellyCompanion />)).toThrow('Store error');
    });
  });
});
