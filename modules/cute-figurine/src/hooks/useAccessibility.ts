import { useCallback, useEffect, useRef, useState } from 'react';
import { usePreferenceStore } from '../state/preferenceStore';

export interface UseAccessibilityOptions {
  enabled?: boolean;
  containerRef: React.RefObject<HTMLElement>;
  onInteraction?: (type: string, data?: any) => void;
  announcements?: boolean;
  keyboardNavigation?: boolean;
}

export interface UseAccessibilityReturn {
  announceMessage: (message: string, priority?: 'polite' | 'assertive') => void;
  handleKeyPress: (e: React.KeyboardEvent) => void;
  focusableElements: HTMLElement[];
  currentFocusIndex: number;
  isHighContrast: boolean;
  fontSize: string;
}

export function useAccessibility(options: UseAccessibilityOptions): UseAccessibilityReturn {
  const {
    enabled = true,
    containerRef,
    onInteraction,
    announcements = true,
    keyboardNavigation = true,
  } = options;

  const preferences = usePreferenceStore();
  const [focusableElements, setFocusableElements] = useState<HTMLElement[]>([]);
  const [currentFocusIndex, setCurrentFocusIndex] = useState(-1);
  const announcementRef = useRef<HTMLDivElement>();
  const lastAnnouncementRef = useRef<string>('');

  // Get accessibility preferences
  const isHighContrast = preferences.enableHighContrast;
  const fontSize = preferences.fontSize;
  const enableScreenReader = preferences.enableScreenReaderAnnouncements;

  // Create or get announcement element
  const getAnnouncementElement = useCallback(() => {
    if (announcementRef.current) return announcementRef.current;

    const element = document.createElement('div');
    element.setAttribute('aria-live', 'polite');
    element.setAttribute('aria-atomic', 'true');
    element.setAttribute('class', 'sr-only skelly-announcements');
    element.style.cssText = `
      position: absolute !important;
      left: -10000px !important;
      width: 1px !important;
      height: 1px !important;
      overflow: hidden !important;
      clip: rect(1px, 1px, 1px, 1px) !important;
      clip-path: inset(50%) !important;
      white-space: nowrap !important;
    `;

    document.body.appendChild(element);
    announcementRef.current = element;
    return element;
  }, []);

  // Announce message to screen readers
  const announceMessage = useCallback(
    (message: string, priority: 'polite' | 'assertive' = 'polite') => {
      if (!enabled || !announcements || !enableScreenReader) return;
      if (message === lastAnnouncementRef.current) return; // Prevent duplicate announcements

      const element = getAnnouncementElement();
      element.setAttribute('aria-live', priority);

      // Clear and set new message
      element.textContent = '';
      setTimeout(() => {
        element.textContent = message;
        lastAnnouncementRef.current = message;
      }, 100);

      // Clear message after a delay to avoid cluttering
      setTimeout(() => {
        if (element.textContent === message) {
          element.textContent = '';
        }
      }, 5000);
    },
    [enabled, announcements, enableScreenReader, getAnnouncementElement]
  );

  // Update focusable elements
  const updateFocusableElements = useCallback(() => {
    if (!containerRef.current || !keyboardNavigation) return;

    const container = containerRef.current;
    const focusableSelectors = [
      'button',
      'input',
      'select',
      'textarea',
      'a[href]',
      '[tabindex]:not([tabindex="-1"])',
      '[role="button"]',
      '[role="link"]',
      '.skelly-interactive',
    ].join(', ');

    const elements = Array.from(container.querySelectorAll(focusableSelectors)) as HTMLElement[];

    // Filter out disabled elements
    const filteredElements = elements.filter(
      (el) =>
        !el.hasAttribute('disabled') &&
        !el.getAttribute('aria-disabled') &&
        el.offsetParent !== null // Element is visible
    );

    setFocusableElements(filteredElements);
  }, [containerRef, keyboardNavigation]);

  // Handle keyboard navigation
  const handleKeyPress = useCallback(
    (e: React.KeyboardEvent) => {
      if (!enabled || !keyboardNavigation) return;

      const { key, shiftKey, ctrlKey, altKey } = e;

      // Handle different key combinations
      switch (key) {
        case 'Tab':
          // Let browser handle tab navigation
          break;

        case 'Enter':
        case ' ':
          e.preventDefault();
          onInteraction?.('keyboard', { key, action: 'activate' });
          announceMessage('Companion activated');
          break;

        case 'ArrowUp':
        case 'ArrowDown':
        case 'ArrowLeft':
        case 'ArrowRight':
          e.preventDefault();
          handleArrowNavigation(key, shiftKey);
          break;

        case 'Home':
          e.preventDefault();
          onInteraction?.('keyboard', { key, action: 'move_to_corner', corner: 'top-left' });
          announceMessage('Moved to top left corner');
          break;

        case 'End':
          e.preventDefault();
          onInteraction?.('keyboard', { key, action: 'move_to_corner', corner: 'bottom-right' });
          announceMessage('Moved to bottom right corner');
          break;

        case 'Escape':
          e.preventDefault();
          onInteraction?.('keyboard', { key, action: 'dismiss' });
          announceMessage('Dismissed');
          break;

        case 'h':
        case 'H':
          if (!shiftKey && !ctrlKey && !altKey) {
            e.preventDefault();
            onInteraction?.('keyboard', { key, action: 'help' });
            announceMessage('Help: Use arrow keys to move, Enter to interact, Escape to dismiss');
          }
          break;

        case 'p':
        case 'P':
          if (!shiftKey && !ctrlKey && !altKey) {
            e.preventDefault();
            onInteraction?.('keyboard', { key, action: 'pet' });
            announceMessage('Giving companion a pet');
          }
          break;

        case 'c':
        case 'C':
          if (!shiftKey && !ctrlKey && !altKey) {
            e.preventDefault();
            onInteraction?.('keyboard', { key, action: 'controls' });
            announceMessage('Toggling control panel');
          }
          break;
      }
    },
    [enabled, keyboardNavigation, onInteraction, announceMessage]
  );

  // Handle arrow key navigation
  const handleArrowNavigation = useCallback(
    (key: string, shiftKey: boolean) => {
      const moveDistance = shiftKey ? 50 : 10; // Larger steps with Shift
      const direction = {
        ArrowUp: { x: 0, y: -moveDistance },
        ArrowDown: { x: 0, y: moveDistance },
        ArrowLeft: { x: -moveDistance, y: 0 },
        ArrowRight: { x: moveDistance, y: 0 },
      }[key];

      if (direction) {
        onInteraction?.('keyboard', {
          key,
          action: 'move',
          direction,
          distance: moveDistance,
        });
        announceMessage(`Moved ${key.replace('Arrow', '').toLowerCase()} ${moveDistance} pixels`);
      }
    },
    [onInteraction, announceMessage]
  );

  // Focus management
  const focusElement = useCallback(
    (index: number) => {
      if (index >= 0 && index < focusableElements.length) {
        focusableElements[index].focus();
        setCurrentFocusIndex(index);
      }
    },
    [focusableElements]
  );

  const focusNext = useCallback(() => {
    const nextIndex = (currentFocusIndex + 1) % focusableElements.length;
    focusElement(nextIndex);
  }, [currentFocusIndex, focusableElements.length, focusElement]);

  const focusPrevious = useCallback(() => {
    const prevIndex = currentFocusIndex <= 0 ? focusableElements.length - 1 : currentFocusIndex - 1;
    focusElement(prevIndex);
  }, [currentFocusIndex, focusableElements.length, focusElement]);

  // Set up reduced motion detection
  useEffect(() => {
    const mediaQuery = window.matchMedia('(prefers-reduced-motion: reduce)');

    const handleReducedMotionChange = (e: MediaQueryListEvent) => {
      if (e.matches) {
        announceMessage('Reduced motion detected, animations simplified');
      }
    };

    mediaQuery.addEventListener('change', handleReducedMotionChange);
    return () => mediaQuery.removeEventListener('change', handleReducedMotionChange);
  }, [announceMessage]);

  // Set up high contrast detection
  useEffect(() => {
    const mediaQuery = window.matchMedia('(prefers-contrast: high)');

    const handleContrastChange = (e: MediaQueryListEvent) => {
      if (e.matches) {
        announceMessage('High contrast mode detected');
      }
    };

    mediaQuery.addEventListener('change', handleContrastChange);
    return () => mediaQuery.removeEventListener('change', handleContrastChange);
  }, [announceMessage]);

  // Update focusable elements when container changes
  useEffect(() => {
    updateFocusableElements();

    if (!containerRef.current) return;

    const observer = new MutationObserver(updateFocusableElements);
    observer.observe(containerRef.current, {
      childList: true,
      subtree: true,
      attributes: true,
      attributeFilter: ['disabled', 'aria-disabled', 'tabindex'],
    });

    return () => observer.disconnect();
  }, [updateFocusableElements]);

  // Apply accessibility styles
  useEffect(() => {
    if (!containerRef.current) return;

    const container = containerRef.current;

    // Apply high contrast styles
    if (isHighContrast) {
      container.style.filter = 'contrast(1.5)';
      container.style.outline = '2px solid currentColor';
    } else {
      container.style.filter = '';
      container.style.outline = '';
    }

    // Apply font size
    const fontSizeMap = {
      small: '0.875rem',
      medium: '1rem',
      large: '1.25rem',
    };
    container.style.fontSize = fontSizeMap[fontSize] || fontSizeMap.medium;
  }, [isHighContrast, fontSize, containerRef]);

  // Announce component state changes
  useEffect(() => {
    if (!enabled || !announcements) return;

    // Initial announcement
    announceMessage('Skelly companion loaded and ready');

    return () => {
      // Cleanup announcement element
      if (announcementRef.current) {
        document.body.removeChild(announcementRef.current);
        announcementRef.current = undefined;
      }
    };
  }, [enabled, announcements, announceMessage]);

  return {
    announceMessage,
    handleKeyPress,
    focusableElements,
    currentFocusIndex,
    isHighContrast,
    fontSize,
  };
}
