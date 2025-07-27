import { useCallback, useEffect, useRef, useState } from 'react';
import { globalEventBus } from '../services/EventBusService';
import { createEvent } from '../types/events.types';

export interface PerformanceMetrics {
  fps: number;
  frameTime: number;
  memoryUsage: number;
  cpuUsage: number;
  renderTime: number;
  eventProcessingTime: number;
  totalFrames: number;
  droppedFrames: number;
  averageFrameTime: number;
  worstFrameTime: number;
  isThrottled: boolean;
}

export interface PerformanceThresholds {
  targetFPS: number;
  maxFrameTime: number;
  maxMemoryUsage: number;
  maxCPUUsage: number;
}

export interface UsePerformanceMonitorOptions {
  enabled?: boolean;
  reportInterval?: number; // milliseconds
  thresholds?: Partial<PerformanceThresholds>;
  onPerformanceIssue?: (metrics: PerformanceMetrics, issue: string) => void;
  onThresholdExceeded?: (
    metric: keyof PerformanceMetrics,
    value: number,
    threshold: number
  ) => void;
}

const DEFAULT_THRESHOLDS: PerformanceThresholds = {
  targetFPS: 30,
  maxFrameTime: 33.33, // ~30fps
  maxMemoryUsage: 50 * 1024 * 1024, // 50MB
  maxCPUUsage: 50, // 50%
};

export function usePerformanceMonitor(
  containerRef?: React.RefObject<HTMLElement>,
  options: UsePerformanceMonitorOptions = {}
): PerformanceMetrics {
  const {
    enabled = true,
    reportInterval = 1000,
    thresholds = {},
    onPerformanceIssue,
    onThresholdExceeded,
  } = options;

  const finalThresholds = { ...DEFAULT_THRESHOLDS, ...thresholds };

  const [metrics, setMetrics] = useState<PerformanceMetrics>({
    fps: 0,
    frameTime: 0,
    memoryUsage: 0,
    cpuUsage: 0,
    renderTime: 0,
    eventProcessingTime: 0,
    totalFrames: 0,
    droppedFrames: 0,
    averageFrameTime: 0,
    worstFrameTime: 0,
    isThrottled: false,
  });

  // Performance tracking state
  const frameCountRef = useRef(0);
  const lastReportTimeRef = useRef(performance.now());
  const frameTimesRef = useRef<number[]>([]);
  const lastFrameTimeRef = useRef(performance.now());
  const performanceObserverRef = useRef<PerformanceObserver>();
  const measurementsRef = useRef<{
    renderTimes: number[];
    eventTimes: number[];
    gcTimes: number[];
  }>({
    renderTimes: [],
    eventTimes: [],
    gcTimes: [],
  });

  // Get memory usage
  const getMemoryUsage = useCallback((): number => {
    if ('memory' in performance) {
      return (performance as any).memory.usedJSHeapSize || 0;
    }
    return 0;
  }, []);

  // Estimate CPU usage (rough approximation)
  const estimateCPUUsage = useCallback((): number => {
    const currentTime = performance.now();
    const frameTimes = frameTimesRef.current;

    if (frameTimes.length < 10) return 0;

    const recentFrameTimes = frameTimes.slice(-10);
    const averageFrameTime =
      recentFrameTimes.reduce((sum, time) => sum + time, 0) / recentFrameTimes.length;
    const targetFrameTime = 1000 / finalThresholds.targetFPS;

    // Rough estimation: if we're taking longer than target, we're using more CPU
    return Math.min(100, (averageFrameTime / targetFrameTime) * 100);
  }, [finalThresholds.targetFPS]);

  // Check if performance is being throttled
  const checkThrottling = useCallback((): boolean => {
    // Check if the tab is hidden
    if (document.hidden) return true;

    // Check if frame rate is significantly below target
    const currentFPS = metrics.fps;
    return currentFPS < finalThresholds.targetFPS * 0.7;
  }, [metrics.fps, finalThresholds.targetFPS]);

  // Performance measurement function
  const measurePerformance = useCallback(() => {
    const now = performance.now();
    const deltaTime = now - lastFrameTimeRef.current;

    frameCountRef.current++;
    frameTimesRef.current.push(deltaTime);

    // Keep only recent frame times (last 60 frames)
    if (frameTimesRef.current.length > 60) {
      frameTimesRef.current = frameTimesRef.current.slice(-60);
    }

    lastFrameTimeRef.current = now;

    // Report metrics periodically
    const timeSinceLastReport = now - lastReportTimeRef.current;
    if (timeSinceLastReport >= reportInterval) {
      updateMetrics();
      lastReportTimeRef.current = now;
    }
  }, [reportInterval]);

  // Update metrics
  const updateMetrics = useCallback(() => {
    const now = performance.now();
    const timeSinceLastReport = now - lastReportTimeRef.current;
    const frameCount = frameCountRef.current;

    // Calculate FPS
    const fps = frameCount > 0 ? (frameCount / timeSinceLastReport) * 1000 : 0;

    // Calculate frame time statistics
    const frameTimes = frameTimesRef.current;
    const currentFrameTime = frameTimes.length > 0 ? frameTimes[frameTimes.length - 1] : 0;
    const averageFrameTime =
      frameTimes.length > 0
        ? frameTimes.reduce((sum, time) => sum + time, 0) / frameTimes.length
        : 0;
    const worstFrameTime = frameTimes.length > 0 ? Math.max(...frameTimes) : 0;

    // Calculate dropped frames (frames that took longer than target)
    const targetFrameTime = 1000 / finalThresholds.targetFPS;
    const droppedFrames = frameTimes.filter((time) => time > targetFrameTime).length;

    // Get other metrics
    const memoryUsage = getMemoryUsage();
    const cpuUsage = estimateCPUUsage();
    const isThrottled = checkThrottling();

    // Calculate render and event processing times
    const measurements = measurementsRef.current;
    const renderTime =
      measurements.renderTimes.length > 0
        ? measurements.renderTimes.reduce((sum, time) => sum + time, 0) /
          measurements.renderTimes.length
        : 0;
    const eventProcessingTime =
      measurements.eventTimes.length > 0
        ? measurements.eventTimes.reduce((sum, time) => sum + time, 0) /
          measurements.eventTimes.length
        : 0;

    const newMetrics: PerformanceMetrics = {
      fps: Math.round(fps),
      frameTime: currentFrameTime,
      memoryUsage,
      cpuUsage,
      renderTime,
      eventProcessingTime,
      totalFrames: frameCount,
      droppedFrames,
      averageFrameTime,
      worstFrameTime,
      isThrottled,
    };

    setMetrics(newMetrics);

    // Check thresholds and emit events
    checkThresholds(newMetrics);

    // Emit performance event
    globalEventBus.emit(createEvent.performance('frame_time', currentFrameTime, targetFrameTime));

    // Reset counters
    frameCountRef.current = 0;
    measurements.renderTimes = [];
    measurements.eventTimes = [];
    measurements.gcTimes = [];
  }, [finalThresholds, getMemoryUsage, estimateCPUUsage, checkThrottling]);

  // Check performance thresholds
  const checkThresholds = useCallback(
    (currentMetrics: PerformanceMetrics) => {
      const checks = [
        {
          metric: 'fps' as const,
          value: currentMetrics.fps,
          threshold: finalThresholds.targetFPS,
          comparison: '<',
        },
        {
          metric: 'frameTime' as const,
          value: currentMetrics.frameTime,
          threshold: finalThresholds.maxFrameTime,
          comparison: '>',
        },
        {
          metric: 'memoryUsage' as const,
          value: currentMetrics.memoryUsage,
          threshold: finalThresholds.maxMemoryUsage,
          comparison: '>',
        },
        {
          metric: 'cpuUsage' as const,
          value: currentMetrics.cpuUsage,
          threshold: finalThresholds.maxCPUUsage,
          comparison: '>',
        },
      ];

      checks.forEach(({ metric, value, threshold, comparison }) => {
        const exceeded = comparison === '>' ? value > threshold : value < threshold;

        if (exceeded) {
          onThresholdExceeded?.(metric, value, threshold);

          // Emit performance issue event
          globalEventBus.emit(
            createEvent.performance(`${metric}_threshold_exceeded` as any, value, threshold)
          );

          // Call performance issue callback
          onPerformanceIssue?.(
            currentMetrics,
            `${metric} threshold exceeded: ${value} vs ${threshold}`
          );
        }
      });
    },
    [finalThresholds, onThresholdExceeded, onPerformanceIssue]
  );

  // Set up Performance Observer for more detailed metrics
  useEffect(() => {
    if (!enabled || !('PerformanceObserver' in window)) return;

    try {
      const observer = new PerformanceObserver((list) => {
        const entries = list.getEntries();

        entries.forEach((entry) => {
          switch (entry.entryType) {
            case 'measure':
              if (entry.name.startsWith('render')) {
                measurementsRef.current.renderTimes.push(entry.duration);
              } else if (entry.name.startsWith('event')) {
                measurementsRef.current.eventTimes.push(entry.duration);
              }
              break;

            case 'navigation':
              // Track page load performance
              break;

            case 'gc':
              measurementsRef.current.gcTimes.push(entry.duration);
              break;
          }
        });
      });

      observer.observe({ entryTypes: ['measure', 'navigation'] });
      performanceObserverRef.current = observer;

      return () => {
        observer.disconnect();
      };
    } catch (error) {
      console.warn('Performance Observer not supported:', error);
    }
  }, [enabled]);

  // Set up animation frame monitoring
  useEffect(() => {
    if (!enabled) return;

    let animationId: number;

    const animationLoop = () => {
      measurePerformance();
      animationId = requestAnimationFrame(animationLoop);
    };

    animationId = requestAnimationFrame(animationLoop);

    return () => {
      if (animationId) {
        cancelAnimationFrame(animationId);
      }
    };
  }, [enabled, measurePerformance]);

  // Monitor container visibility for performance optimization
  useEffect(() => {
    if (!containerRef?.current) return;

    const observer = new IntersectionObserver(
      ([entry]) => {
        if (!entry.isIntersecting && metrics.fps > 0) {
          // Container is not visible, we can reduce performance monitoring
          globalEventBus.emit(createEvent.performance('visibility_change', 0));
        } else if (entry.isIntersecting) {
          globalEventBus.emit(createEvent.performance('visibility_change', 1));
        }
      },
      { threshold: 0.1 }
    );

    observer.observe(containerRef.current);

    return () => observer.disconnect();
  }, [containerRef, metrics.fps]);

  // Cleanup
  useEffect(() => {
    return () => {
      if (performanceObserverRef.current) {
        performanceObserverRef.current.disconnect();
      }
    };
  }, []);

  return metrics;
}

// Utility functions for performance optimization
export const performanceUtils = {
  /**
   * Mark a performance measurement
   */
  mark: (name: string) => {
    if ('performance' in window && performance.mark) {
      performance.mark(name);
    }
  },

  /**
   * Measure performance between marks
   */
  measure: (name: string, startMark: string, endMark?: string) => {
    if ('performance' in window && performance.measure) {
      try {
        performance.measure(name, startMark, endMark);
      } catch (error) {
        // Marks might not exist
      }
    }
  },

  /**
   * Check if the browser tab is visible
   */
  isTabVisible: () => !document.hidden,

  /**
   * Check if the device is a low-end device
   */
  isLowEndDevice: () => {
    if ('deviceMemory' in navigator) {
      return (navigator as any).deviceMemory < 4;
    }
    if ('hardwareConcurrency' in navigator) {
      return navigator.hardwareConcurrency < 4;
    }
    return false;
  },

  /**
   * Get connection quality
   */
  getConnectionQuality: () => {
    if ('connection' in navigator) {
      const connection = (navigator as any).connection;
      return {
        effectiveType: connection.effectiveType || 'unknown',
        downlink: connection.downlink || 0,
        rtt: connection.rtt || 0,
      };
    }
    return { effectiveType: 'unknown', downlink: 0, rtt: 0 };
  },
};
