import { useState, useEffect, useCallback, useRef } from 'react';
import {
  PerformanceBenchmark,
  BenchmarkResult,
  BenchmarkSuite,
  performanceBenchmark,
} from '../services/PerformanceBenchmark';
import { useCompanionStore } from '../state/companionStore';

export interface UsePerformanceBenchmarkReturn {
  // Benchmarking functions
  benchmark: (
    name: string,
    testFunction: () => void | Promise<void>,
    iterations?: number
  ) => Promise<BenchmarkResult>;
  startTimer: (name: string) => void;
  endTimer: (name: string, metadata?: Record<string, any>) => BenchmarkResult | null;

  // Specialized benchmarks
  benchmarkAnimation: (
    animationFunction: () => Promise<void>,
    frames?: number
  ) => Promise<BenchmarkResult>;
  benchmarkStateUpdate: (
    updateFunction: () => void,
    iterations?: number
  ) => Promise<BenchmarkResult>;
  benchmarkEventProcessing: (
    eventHandler: (event: any) => void,
    eventCount?: number
  ) => Promise<BenchmarkResult>;

  // Results and analysis
  results: BenchmarkResult[];
  analysis: ReturnType<PerformanceBenchmark['analyzePerformance']> | null;

  // Actions
  runFullSuite: () => Promise<BenchmarkSuite>;
  clearResults: () => void;
  exportResults: () => string;

  // State
  isRunning: boolean;
  currentTest: string | null;
}

export const usePerformanceBenchmark = (): UsePerformanceBenchmarkReturn => {
  const [results, setResults] = useState<BenchmarkResult[]>([]);
  const [analysis, setAnalysis] = useState<ReturnType<
    PerformanceBenchmark['analyzePerformance']
  > | null>(null);
  const [isRunning, setIsRunning] = useState(false);
  const [currentTest, setCurrentTest] = useState<string | null>(null);

  const companionStore = useCompanionStore();
  const activeTimers = useRef<Set<string>>(new Set());

  // Refresh results and analysis
  const refreshData = useCallback(() => {
    const latestResults = performanceBenchmark.getResults();
    setResults(latestResults);

    if (latestResults.length > 0) {
      setAnalysis(performanceBenchmark.analyzePerformance());
    }
  }, []);

  // Initialize data on mount
  useEffect(() => {
    refreshData();
  }, [refreshData]);

  // Wrapped benchmark function
  const benchmark = useCallback(
    async (
      name: string,
      testFunction: () => void | Promise<void>,
      iterations: number = 100
    ): Promise<BenchmarkResult> => {
      setIsRunning(true);
      setCurrentTest(name);

      try {
        const result = await performanceBenchmark.benchmark(name, testFunction, iterations);
        refreshData();
        return result;
      } finally {
        setIsRunning(false);
        setCurrentTest(null);
      }
    },
    [refreshData]
  );

  // Timer functions
  const startTimer = useCallback((name: string) => {
    performanceBenchmark.startTimer(name);
    activeTimers.current.add(name);
    setCurrentTest(name);
  }, []);

  const endTimer = useCallback(
    (name: string, metadata?: Record<string, any>) => {
      const result = performanceBenchmark.endTimer(name, metadata);
      activeTimers.current.delete(name);

      if (activeTimers.current.size === 0) {
        setCurrentTest(null);
      }

      refreshData();
      return result;
    },
    [refreshData]
  );

  // Specialized benchmarks
  const benchmarkAnimation = useCallback(
    async (
      animationFunction: () => Promise<void>,
      frames: number = 60
    ): Promise<BenchmarkResult> => {
      setIsRunning(true);
      setCurrentTest('animation_performance');

      try {
        const result = await performanceBenchmark.benchmarkAnimation(animationFunction, frames);
        refreshData();
        return result;
      } finally {
        setIsRunning(false);
        setCurrentTest(null);
      }
    },
    [refreshData]
  );

  const benchmarkStateUpdate = useCallback(
    async (updateFunction: () => void, iterations: number = 100): Promise<BenchmarkResult> => {
      return benchmark('state_updates', updateFunction, iterations);
    },
    [benchmark]
  );

  const benchmarkEventProcessing = useCallback(
    async (
      eventHandler: (event: any) => void,
      eventCount: number = 1000
    ): Promise<BenchmarkResult> => {
      setIsRunning(true);
      setCurrentTest('event_processing');

      try {
        const result = await performanceBenchmark.benchmarkEventProcessing(
          eventHandler,
          eventCount
        );
        refreshData();
        return result;
      } finally {
        setIsRunning(false);
        setCurrentTest(null);
      }
    },
    [refreshData]
  );

  // Full test suite
  const runFullSuite = useCallback(async (): Promise<BenchmarkSuite> => {
    setIsRunning(true);
    setCurrentTest('full_suite');

    try {
      const suite = performanceBenchmark.createSuite('Cute Figurine Performance Suite');

      const benchmarks = [
        {
          name: 'companion_state_update',
          test: () => {
            companionStore.updateMood(companionStore.mood);
            companionStore.updateEnergy(Math.random() * 100);
            companionStore.updatePosition({
              x: Math.random() * 100,
              y: Math.random() * 100,
            });
          },
          iterations: 1000,
        },
        {
          name: 'message_queue_processing',
          test: () => {
            const message = {
              text: 'Performance test message',
              priority: Math.floor(Math.random() * 5),
              duration: 3000,
            };
            companionStore.queueMessage(message);
            companionStore.clearMessage();
          },
          iterations: 500,
        },
        {
          name: 'mood_transitions',
          test: () => {
            const moods = ['happy', 'focused', 'tired', 'excited', 'melting'];
            const randomMood = moods[Math.floor(Math.random() * moods.length)];
            companionStore.updateMood(randomMood as any);
          },
          iterations: 200,
        },
        {
          name: 'intervention_cooldown',
          test: () => {
            companionStore.recordInteraction();
            companionStore.canIntervene();
            companionStore.updateInterventionCooldown(30);
          },
          iterations: 300,
        },
        {
          name: 'animation_selection',
          test: () => {
            companionStore.getCurrentAnimation();
          },
          iterations: 1000,
        },
      ];

      const completedSuite = await performanceBenchmark.runSuite(suite, benchmarks);
      refreshData();
      return completedSuite;
    } finally {
      setIsRunning(false);
      setCurrentTest(null);
    }
  }, [companionStore, refreshData]);

  // Data management
  const clearResults = useCallback(() => {
    performanceBenchmark.clearResults();
    refreshData();
  }, [refreshData]);

  const exportResults = useCallback(() => {
    return performanceBenchmark.exportResults();
  }, []);

  return {
    // Benchmarking functions
    benchmark,
    startTimer,
    endTimer,

    // Specialized benchmarks
    benchmarkAnimation,
    benchmarkStateUpdate,
    benchmarkEventProcessing,

    // Results and analysis
    results,
    analysis,

    // Actions
    runFullSuite,
    clearResults,
    exportResults,

    // State
    isRunning,
    currentTest,
  };
};

// Hook for automatic performance monitoring during component lifecycle
export const usePerformanceMonitoring = (componentName: string) => {
  const { startTimer, endTimer } = usePerformanceBenchmark();
  const mountTime = useRef<number | null>(null);

  useEffect(() => {
    // Component mount timing
    mountTime.current = performance.now();
    startTimer(`${componentName}_mount`);

    return () => {
      // Component unmount timing
      if (mountTime.current) {
        endTimer(`${componentName}_mount`, {
          totalLifetime: performance.now() - mountTime.current,
        });
      }
    };
  }, [componentName, startTimer, endTimer]);

  // Function to measure render timing
  const measureRender = useCallback(
    (renderFunction: () => void) => {
      const renderStart = performance.now();
      renderFunction();
      const renderEnd = performance.now();

      endTimer(`${componentName}_render`, {
        renderTime: renderEnd - renderStart,
      });
    },
    [componentName, endTimer]
  );

  return {
    measureRender,
    startTimer: (name: string) => startTimer(`${componentName}_${name}`),
    endTimer: (name: string, metadata?: Record<string, any>) =>
      endTimer(`${componentName}_${name}`, metadata),
  };
};

// Hook for measuring function performance
export const useMeasuredFunction = <T extends (...args: any[]) => any>(
  functionName: string,
  originalFunction: T,
  shouldMeasure: boolean = true
): T => {
  const { benchmark } = usePerformanceBenchmark();

  return useCallback(
    (...args: Parameters<T>) => {
      if (!shouldMeasure) {
        return originalFunction(...args);
      }

      const start = performance.now();
      const result = originalFunction(...args);
      const end = performance.now();

      // For async functions
      if (result instanceof Promise) {
        return result.finally(() => {
          performanceBenchmark.endTimer(functionName, {
            duration: end - start,
            args: args.length,
          });
        });
      }

      // For sync functions
      performanceBenchmark.endTimer(functionName, {
        duration: end - start,
        args: args.length,
      });

      return result;
    },
    [functionName, originalFunction, shouldMeasure]
  ) as T;
};
