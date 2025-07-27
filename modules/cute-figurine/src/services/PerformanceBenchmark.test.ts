import { describe, it, expect, beforeEach, vi } from 'vitest';
import { PerformanceBenchmark, BenchmarkResult } from './PerformanceBenchmark';

// Mock performance.now and performance.memory
const mockPerformance = {
  now: vi.fn(),
  memory: {
    usedJSHeapSize: 1024 * 1024 * 10, // 10MB
  },
};

Object.defineProperty(global, 'performance', {
  value: mockPerformance,
  writable: true,
});

describe('PerformanceBenchmark', () => {
  let benchmark: PerformanceBenchmark;
  let timeCounter = 0;

  beforeEach(() => {
    vi.clearAllMocks();
    timeCounter = 0;

    // Mock performance.now to return predictable values
    mockPerformance.now.mockImplementation(() => {
      timeCounter += 10; // Each call adds 10ms
      return timeCounter;
    });

    // Create new instance for each test
    benchmark = new (PerformanceBenchmark as any)();
  });

  describe('Basic Benchmarking', () => {
    it('benchmarks a synchronous function correctly', async () => {
      const testFunction = vi.fn();
      const iterations = 5;

      const result = await benchmark.benchmark('sync_test', testFunction, iterations);

      expect(testFunction).toHaveBeenCalledTimes(iterations + Math.min(10, iterations)); // +warmup
      expect(result).toMatchObject({
        name: 'sync_test',
        iterations,
        averageTime: 10, // Based on our mock (10ms per call)
        minTime: 10,
        maxTime: 10,
      });
      expect(result.duration).toBeGreaterThan(0);
      expect(result.timestamp).toBeInstanceOf(Date);
    });

    it('benchmarks an async function correctly', async () => {
      const testFunction = vi.fn().mockResolvedValue(undefined);
      const iterations = 3;

      const result = await benchmark.benchmark('async_test', testFunction, iterations);

      expect(testFunction).toHaveBeenCalledTimes(iterations + Math.min(10, iterations));
      expect(result.name).toBe('async_test');
      expect(result.iterations).toBe(iterations);
    });

    it('handles function that throws errors', async () => {
      const testFunction = vi.fn().mockImplementation(() => {
        throw new Error('Test error');
      });

      // The benchmark should still complete and catch errors
      await expect(benchmark.benchmark('error_test', testFunction, 2)).rejects.toThrow();
    });
  });

  describe('Timer Functions', () => {
    it('starts and ends timer correctly', () => {
      const testName = 'timer_test';

      benchmark.startTimer(testName);

      // Simulate some time passing
      timeCounter += 50;

      const result = benchmark.endTimer(testName, { custom: 'metadata' });

      expect(result).toMatchObject({
        name: testName,
        duration: 50,
        iterations: 1,
        averageTime: 50,
        minTime: 50,
        maxTime: 50,
        metadata: { custom: 'metadata' },
      });
    });

    it('returns null for non-existent timer', () => {
      const result = benchmark.endTimer('non_existent');
      expect(result).toBeNull();
    });

    it('handles multiple concurrent timers', () => {
      benchmark.startTimer('timer1');
      benchmark.startTimer('timer2');

      timeCounter += 30;
      const result1 = benchmark.endTimer('timer1');

      timeCounter += 20;
      const result2 = benchmark.endTimer('timer2');

      expect(result1?.duration).toBe(30);
      expect(result2?.duration).toBe(50); // Total time for timer2
    });
  });

  describe('Specialized Benchmarks', () => {
    it('benchmarks animation performance', async () => {
      const animationFunction = vi.fn().mockResolvedValue(undefined);
      const frames = 10;

      const result = await benchmark.benchmarkAnimation(animationFunction, frames);

      expect(animationFunction).toHaveBeenCalledTimes(frames);
      expect(result.name).toBe('animation_performance');
      expect(result.iterations).toBe(frames);
      expect(result.metadata).toHaveProperty('fps');
      expect(result.metadata).toHaveProperty('droppedFrames');
      expect(result.metadata).toHaveProperty('frameTimeVariance');
    });

    it('benchmarks event processing', async () => {
      const eventHandler = vi.fn();
      const eventCount = 100;

      const result = await benchmark.benchmarkEventProcessing(eventHandler, eventCount);

      expect(eventHandler).toHaveBeenCalledTimes(eventCount);
      expect(result.name).toBe('event_processing');
      expect(result.iterations).toBe(1); // Single iteration with many events
    });

    it('benchmarks state updates', async () => {
      const stateFunction = vi.fn();
      const updateCount = 50;

      const result = await benchmark.benchmarkStateUpdates(stateFunction, updateCount);

      expect(stateFunction).toHaveBeenCalledTimes(updateCount + Math.min(10, updateCount));
      expect(result.name).toBe('state_updates');
      expect(result.iterations).toBe(updateCount);
    });

    it('benchmarks rendering performance', async () => {
      const renderFunction = vi.fn().mockResolvedValue(undefined);
      const renderCount = 25;

      const result = await benchmark.benchmarkRenderingPerformance(renderFunction, renderCount);

      expect(renderFunction).toHaveBeenCalledTimes(renderCount + Math.min(10, renderCount));
      expect(result.name).toBe('rendering_performance');
      expect(result.iterations).toBe(renderCount);
    });
  });

  describe('Memory Monitoring', () => {
    it('gets memory usage correctly', () => {
      const memoryUsage = benchmark.getMemoryUsage();
      expect(memoryUsage).toBe(10); // 10MB based on our mock
    });

    it('handles missing performance.memory gracefully', () => {
      const originalMemory = (performance as any).memory;
      delete (performance as any).memory;

      const memoryUsage = benchmark.getMemoryUsage();
      expect(memoryUsage).toBe(0);

      // Restore for other tests
      (performance as any).memory = originalMemory;
    });

    it('measures memory leaks', async () => {
      const testFunction = vi.fn();

      // Mock increasing memory usage
      let memoryCounter = 10;
      vi.spyOn(benchmark, 'getMemoryUsage').mockImplementation(() => {
        memoryCounter += 0.5; // Simulate memory growth
        return memoryCounter;
      });

      const result = await benchmark.measureMemoryLeak(testFunction, 50, 5);

      expect(result.memoryGrowth.length).toBeGreaterThan(0);
      expect(result.averageGrowth).toBeGreaterThan(0);
      expect(result.leakDetected).toBe(false); // 0.5MB growth per interval is below 1MB threshold
    });
  });

  describe('Performance Analysis', () => {
    it('analyzes performance correctly', async () => {
      // Add some test results
      await benchmark.benchmark('fast_test', () => {}, 10);

      // Mock slower performance for animation
      timeCounter = 0;
      mockPerformance.now.mockImplementation(() => {
        timeCounter += 20; // 20ms per call (above 16.67ms threshold)
        return timeCounter;
      });

      await benchmark.benchmarkAnimation(async () => {}, 5);

      const analysis = benchmark.analyzePerformance();

      expect(analysis).toHaveProperty('summary');
      expect(analysis).toHaveProperty('recommendations');
      expect(analysis).toHaveProperty('score');
      expect(analysis.score).toBeLessThan(100); // Should be penalized for slow animation
      expect(analysis.recommendations.length).toBeGreaterThan(0);
    });

    it('provides appropriate recommendations', async () => {
      // Create a scenario with poor performance
      timeCounter = 0;
      mockPerformance.now.mockImplementation(() => {
        timeCounter += 25; // Very slow performance
        return timeCounter;
      });

      await benchmark.benchmarkAnimation(async () => {}, 3);
      await benchmark.benchmark('event_processing', () => {}, 5);

      const analysis = benchmark.analyzePerformance();

      expect(analysis.recommendations).toContain(
        expect.stringMatching(/Animation frame time.*exceeds 60fps target/)
      );
      expect(analysis.score).toBeLessThan(85); // Should be significantly penalized
    });
  });

  describe('Threshold Management', () => {
    it('sets and gets thresholds correctly', () => {
      const newThresholds = {
        frameTime: 20,
        memoryUsage: 100,
      };

      benchmark.setThresholds(newThresholds);
      const thresholds = benchmark.getThresholds();

      expect(thresholds.frameTime).toBe(20);
      expect(thresholds.memoryUsage).toBe(100);
      expect(thresholds.animationLatency).toBe(100); // Should keep default
    });
  });

  describe('Suite Management', () => {
    it('creates benchmark suite correctly', () => {
      const suite = benchmark.createSuite('Test Suite');

      expect(suite.name).toBe('Test Suite');
      expect(suite.results).toEqual([]);
      expect(suite.timestamp).toBeInstanceOf(Date);
      expect(suite.environment).toHaveProperty('userAgent');
      expect(suite.environment).toHaveProperty('platform');
    });

    it('runs benchmark suite correctly', async () => {
      const suite = benchmark.createSuite('Test Suite');
      const benchmarks = [
        {
          name: 'test1',
          test: vi.fn(),
          iterations: 5,
        },
        {
          name: 'test2',
          test: vi.fn(),
        },
      ];

      const completedSuite = await benchmark.runSuite(suite, benchmarks);

      expect(completedSuite.results).toHaveLength(2);
      expect(completedSuite.totalDuration).toBeGreaterThan(0);
      expect(completedSuite.results[0].name).toBe('test1');
      expect(completedSuite.results[1].name).toBe('test2');
    });
  });

  describe('Data Management', () => {
    it('stores and retrieves results correctly', async () => {
      await benchmark.benchmark('test1', () => {}, 5);
      await benchmark.benchmark('test2', () => {}, 3);

      const allResults = benchmark.getResults();
      const test1Results = benchmark.getResults('test1');

      expect(allResults).toHaveLength(2);
      expect(test1Results).toHaveLength(1);
      expect(test1Results[0].name).toBe('test1');
    });

    it('limits stored results to 100 per test', async () => {
      // Add 150 results
      for (let i = 0; i < 150; i++) {
        await benchmark.benchmark('test', () => {}, 1);
      }

      const results = benchmark.getResults('test');
      expect(results).toHaveLength(100); // Should be limited to 100
    });

    it('exports results correctly', async () => {
      await benchmark.benchmark('export_test', () => {}, 2);

      const exportData = benchmark.exportResults();
      const parsed = JSON.parse(exportData);

      expect(parsed).toHaveProperty('timestamp');
      expect(parsed).toHaveProperty('results');
      expect(parsed).toHaveProperty('thresholds');
      expect(parsed).toHaveProperty('analysis');
      expect(parsed.results.export_test).toHaveLength(1);
    });

    it('clears results correctly', async () => {
      await benchmark.benchmark('clear_test1', () => {}, 2);
      await benchmark.benchmark('clear_test2', () => {}, 2);

      // Clear specific test
      benchmark.clearResults('clear_test1');
      expect(benchmark.getResults('clear_test1')).toHaveLength(0);
      expect(benchmark.getResults('clear_test2')).toHaveLength(1);

      // Clear all
      benchmark.clearResults();
      expect(benchmark.getResults()).toHaveLength(0);
    });
  });
});
