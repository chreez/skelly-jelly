import { useCallback, useEffect, useRef, useState } from 'react';
import type { Position } from '../types/state.types';

export interface UseDragAndDropOptions {
  enabled?: boolean;
  containerRef: React.RefObject<HTMLElement>;
  initialPosition?: Position;
  constrainToViewport?: boolean;
  snapToGrid?: boolean;
  gridSize?: number;
  onDragStart?: (position: Position) => void;
  onDrag?: (position: Position) => void;
  onDragEnd?: (position: Position) => void;
  onPositionChange?: (position: Position) => void;
}

export interface UseDragAndDropReturn {
  isDragging: boolean;
  dragPosition: Position | null;
  dragHandlers: {
    onMouseDown: (e: React.MouseEvent) => void;
    onTouchStart: (e: React.TouchEvent) => void;
  };
  setPosition: (position: Position) => void;
}

export function useDragAndDrop(options: UseDragAndDropOptions): UseDragAndDropReturn {
  const {
    enabled = true,
    containerRef,
    initialPosition = { x: 0, y: 0 },
    constrainToViewport = true,
    snapToGrid = false,
    gridSize = 10,
    onDragStart,
    onDrag,
    onDragEnd,
    onPositionChange,
  } = options;

  const [isDragging, setIsDragging] = useState(false);
  const [dragPosition, setDragPosition] = useState<Position | null>(null);
  const dragStartRef = useRef<{ x: number; y: number; elementX: number; elementY: number } | null>(
    null
  );
  const animationFrameRef = useRef<number>();

  // Constrain position to viewport bounds
  const constrainPosition = useCallback(
    (position: Position): Position => {
      if (!constrainToViewport) return position;

      const element = containerRef.current;
      if (!element) return position;

      const rect = element.getBoundingClientRect();
      const viewportWidth = window.innerWidth;
      const viewportHeight = window.innerHeight;

      let { x, y } = position;

      // Constrain to viewport
      x = Math.max(0, Math.min(x, viewportWidth - rect.width));
      y = Math.max(0, Math.min(y, viewportHeight - rect.height));

      // Snap to grid if enabled
      if (snapToGrid) {
        x = Math.round(x / gridSize) * gridSize;
        y = Math.round(y / gridSize) * gridSize;
      }

      return { x, y };
    },
    [constrainToViewport, snapToGrid, gridSize, containerRef]
  );

  // Handle drag movement
  const handleDragMove = useCallback(
    (clientX: number, clientY: number) => {
      if (!isDragging || !dragStartRef.current) return;

      const { x: startX, y: startY, elementX, elementY } = dragStartRef.current;
      const deltaX = clientX - startX;
      const deltaY = clientY - startY;

      const newPosition = constrainPosition({
        x: elementX + deltaX,
        y: elementY + deltaY,
      });

      setDragPosition(newPosition);
      onDrag?.(newPosition);
    },
    [isDragging, constrainPosition, onDrag]
  );

  // Mouse event handlers
  const handleMouseDown = useCallback(
    (e: React.MouseEvent) => {
      if (!enabled || e.button !== 0) return; // Only left mouse button

      e.preventDefault();
      e.stopPropagation();

      const element = containerRef.current;
      if (!element) return;

      const rect = element.getBoundingClientRect();
      const elementPosition = {
        x: rect.left,
        y: rect.top,
      };

      dragStartRef.current = {
        x: e.clientX,
        y: e.clientY,
        elementX: elementPosition.x,
        elementY: elementPosition.y,
      };

      setIsDragging(true);
      setDragPosition(elementPosition);
      onDragStart?.(elementPosition);

      // Add cursor style to body
      document.body.style.cursor = 'grabbing';
      document.body.style.userSelect = 'none';
    },
    [enabled, containerRef, onDragStart]
  );

  const handleMouseMove = useCallback(
    (e: MouseEvent) => {
      if (!isDragging) return;

      // Use requestAnimationFrame for smooth animation
      if (animationFrameRef.current) {
        cancelAnimationFrame(animationFrameRef.current);
      }

      animationFrameRef.current = requestAnimationFrame(() => {
        handleDragMove(e.clientX, e.clientY);
      });
    },
    [isDragging, handleDragMove]
  );

  const handleMouseUp = useCallback(() => {
    if (!isDragging) return;

    setIsDragging(false);

    if (animationFrameRef.current) {
      cancelAnimationFrame(animationFrameRef.current);
    }

    // Restore cursor
    document.body.style.cursor = '';
    document.body.style.userSelect = '';

    if (dragPosition) {
      onDragEnd?.(dragPosition);
      onPositionChange?.(dragPosition);
    }

    setDragPosition(null);
    dragStartRef.current = null;
  }, [isDragging, dragPosition, onDragEnd, onPositionChange]);

  // Touch event handlers
  const handleTouchStart = useCallback(
    (e: React.TouchEvent) => {
      if (!enabled || e.touches.length !== 1) return;

      e.preventDefault();
      e.stopPropagation();

      const touch = e.touches[0];
      const element = containerRef.current;
      if (!element) return;

      const rect = element.getBoundingClientRect();
      const elementPosition = {
        x: rect.left,
        y: rect.top,
      };

      dragStartRef.current = {
        x: touch.clientX,
        y: touch.clientY,
        elementX: elementPosition.x,
        elementY: elementPosition.y,
      };

      setIsDragging(true);
      setDragPosition(elementPosition);
      onDragStart?.(elementPosition);
    },
    [enabled, containerRef, onDragStart]
  );

  const handleTouchMove = useCallback(
    (e: TouchEvent) => {
      if (!isDragging || e.touches.length !== 1) return;

      e.preventDefault();

      const touch = e.touches[0];

      if (animationFrameRef.current) {
        cancelAnimationFrame(animationFrameRef.current);
      }

      animationFrameRef.current = requestAnimationFrame(() => {
        handleDragMove(touch.clientX, touch.clientY);
      });
    },
    [isDragging, handleDragMove]
  );

  const handleTouchEnd = useCallback(
    (e: TouchEvent) => {
      if (!isDragging) return;

      e.preventDefault();
      handleMouseUp(); // Reuse mouse up logic
    },
    [isDragging, handleMouseUp]
  );

  // Set up global event listeners
  useEffect(() => {
    if (!isDragging) return;

    // Mouse events
    document.addEventListener('mousemove', handleMouseMove, { passive: false });
    document.addEventListener('mouseup', handleMouseUp);

    // Touch events
    document.addEventListener('touchmove', handleTouchMove, { passive: false });
    document.addEventListener('touchend', handleTouchEnd, { passive: false });
    document.addEventListener('touchcancel', handleTouchEnd, { passive: false });

    // Cleanup
    return () => {
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseup', handleMouseUp);
      document.removeEventListener('touchmove', handleTouchMove);
      document.removeEventListener('touchend', handleTouchEnd);
      document.removeEventListener('touchcancel', handleTouchEnd);

      if (animationFrameRef.current) {
        cancelAnimationFrame(animationFrameRef.current);
      }
    };
  }, [isDragging, handleMouseMove, handleMouseUp, handleTouchMove, handleTouchEnd]);

  // Handle escape key to cancel drag
  useEffect(() => {
    if (!isDragging) return;

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        e.preventDefault();
        setIsDragging(false);
        setDragPosition(null);
        dragStartRef.current = null;

        // Restore cursor
        document.body.style.cursor = '';
        document.body.style.userSelect = '';

        if (animationFrameRef.current) {
          cancelAnimationFrame(animationFrameRef.current);
        }
      }
    };

    document.addEventListener('keydown', handleKeyDown);
    return () => document.removeEventListener('keydown', handleKeyDown);
  }, [isDragging]);

  // Programmatic position setter
  const setPosition = useCallback(
    (position: Position) => {
      const constrainedPosition = constrainPosition(position);
      onPositionChange?.(constrainedPosition);
    },
    [constrainPosition, onPositionChange]
  );

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      if (animationFrameRef.current) {
        cancelAnimationFrame(animationFrameRef.current);
      }

      // Restore body styles
      document.body.style.cursor = '';
      document.body.style.userSelect = '';
    };
  }, []);

  return {
    isDragging,
    dragPosition,
    dragHandlers: {
      onMouseDown: handleMouseDown,
      onTouchStart: handleTouchStart,
    },
    setPosition,
  };
}
