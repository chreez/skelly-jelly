export interface BenchmarkResult {
  name: string;
  duration: number;
  iterations: number;
  averageTime: number;
  minTime: number;
  maxTime: number;
  memoryBefore: number;
  memoryAfter: number;
  memoryDelta: number;
  timestamp: Date;
  metadata?: Record<string, any>;
}

export interface BenchmarkSuite {
  name: string;
  results: BenchmarkResult[];
  totalDuration: number;
  timestamp: Date;
  environment: {
    userAgent: string;
    platform: string;
    memory?: number;
    cores?: number;
  };
}

export interface PerformanceThresholds {
  frameTime: number; // ms
  memoryUsage: number; // MB
  animationLatency: number; // ms
  eventProcessingTime: number; // ms
  renderTime: number; // ms
}

export class PerformanceBenchmark {
  private static instance: PerformanceBenchmark;
  private results: Map<string, BenchmarkResult[]> = new Map();
  private activeTests: Map<string, { startTime: number; iterations: number }> = new Map();
  private thresholds: PerformanceThresholds = {
    frameTime: 16.67, // 60fps target
    memoryUsage: 50, // 50MB
    animationLatency: 100, // 100ms
    eventProcessingTime: 5, // 5ms
    renderTime: 10, // 10ms
  };

  public static getInstance(): PerformanceBenchmark {
    if (!PerformanceBenchmark.instance) {
      PerformanceBenchmark.instance = new PerformanceBenchmark();
    }
    return PerformanceBenchmark.instance;
  }

  // Benchmarking API
  public async benchmark(
    name: string,
    testFunction: () => void | Promise<void>,
    iterations: number = 100
  ): Promise<BenchmarkResult> {
    const times: number[] = [];
    const memoryBefore = this.getMemoryUsage();

    // Warm up
    for (let i = 0; i < Math.min(10, iterations); i++) {
      await testFunction();
    }

    // Actual benchmark
    for (let i = 0; i < iterations; i++) {
      const start = performance.now();
      await testFunction();
      const end = performance.now();
      times.push(end - start);
    }

    const memoryAfter = this.getMemoryUsage();
    const totalDuration = times.reduce((sum, time) => sum + time, 0);

    const result: BenchmarkResult = {
      name,
      duration: totalDuration,
      iterations,
      averageTime: totalDuration / iterations,
      minTime: Math.min(...times),
      maxTime: Math.max(...times),
      memoryBefore,
      memoryAfter,
      memoryDelta: memoryAfter - memoryBefore,
      timestamp: new Date(),
    };

    this.addResult(name, result);
    return result;
  }

  public startTimer(name: string): void {
    this.activeTests.set(name, {
      startTime: performance.now(),
      iterations: 1,
    });
  }

  public endTimer(name: string, metadata?: Record<string, any>): BenchmarkResult | null {
    const test = this.activeTests.get(name);
    if (!test) return null;

    const endTime = performance.now();
    const duration = endTime - test.startTime;

    const result: BenchmarkResult = {
      name,
      duration,
      iterations: test.iterations,
      averageTime: duration,
      minTime: duration,
      maxTime: duration,
      memoryBefore: 0,
      memoryAfter: this.getMemoryUsage(),
      memoryDelta: 0,
      timestamp: new Date(),
      metadata,
    };

    this.activeTests.delete(name);
    this.addResult(name, result);
    return result;
  }

  // Specialized Benchmarks
  public async benchmarkAnimation(
    animationFunction: () => Promise<void>,
    frames: number = 60
  ): Promise<BenchmarkResult> {
    const frameTimes: number[] = [];
    let totalMemoryDelta = 0;
    const memoryBefore = this.getMemoryUsage();

    for (let frame = 0; frame < frames; frame++) {
      const frameStart = performance.now();
      await animationFunction();
      const frameEnd = performance.now();

      frameTimes.push(frameEnd - frameStart);

      // Check memory periodically
      if (frame % 10 === 0) {
        const currentMemory = this.getMemoryUsage();
        totalMemoryDelta += currentMemory - memoryBefore;
      }
    }

    const totalDuration = frameTimes.reduce((sum, time) => sum + time, 0);
    const averageFrameTime = totalDuration / frames;
    const fps = 1000 / averageFrameTime;

    const result: BenchmarkResult = {
      name: 'animation_performance',
      duration: totalDuration,
      iterations: frames,
      averageTime: averageFrameTime,
      minTime: Math.min(...frameTimes),
      maxTime: Math.max(...frameTimes),
      memoryBefore,
      memoryAfter: this.getMemoryUsage(),
      memoryDelta: totalMemoryDelta,
      timestamp: new Date(),
      metadata: {
        fps,
        droppedFrames: frameTimes.filter((time) => time > this.thresholds.frameTime).length,
        frameTimeVariance: this.calculateVariance(frameTimes),
      },
    };

    this.addResult('animation', result);
    return result;
  }

  public async benchmarkEventProcessing(
    eventHandler: (event: any) => void,
    eventCount: number = 1000
  ): Promise<BenchmarkResult> {
    const events = Array.from({ length: eventCount }, (_, i) => ({
      type: 'test_event',
      id: i,
      timestamp: Date.now(),
      data: { index: i },
    }));

    return this.benchmark(
      'event_processing',
      () => {
        events.forEach((event) => eventHandler(event));
      },
      1 // Single iteration with many events
    );
  }

  public async benchmarkStateUpdates(
    stateUpdateFunction: () => void,
    updateCount: number = 100
  ): Promise<BenchmarkResult> {
    return this.benchmark('state_updates', stateUpdateFunction, updateCount);
  }

  public async benchmarkRenderingPerformance(
    renderFunction: () => Promise<void>,
    renderCount: number = 50
  ): Promise<BenchmarkResult> {
    return this.benchmark('rendering_performance', renderFunction, renderCount);
  }

  // Memory and Resource Monitoring
  public getMemoryUsage(): number {
    if ('memory' in performance) {
      const memory = (performance as any).memory;
      return memory.usedJSHeapSize / (1024 * 1024); // Convert to MB
    }
    return 0;
  }

  public async measureMemoryLeak(
    testFunction: () => void,
    iterations: number = 100,
    interval: number = 10
  ): Promise<{
    memoryGrowth: number[];
    leakDetected: boolean;
    averageGrowth: number;
  }> {
    const memorySnapshots: number[] = [];

    for (let i = 0; i < iterations; i++) {
      testFunction();

      if (i % interval === 0) {
        // Force garbage collection if available
        if ('gc' in window) {
          (window as any).gc();
        }

        // Wait a bit for GC
        await new Promise((resolve) => setTimeout(resolve, 10));

        memorySnapshots.push(this.getMemoryUsage());
      }
    }

    const memoryGrowth = memorySnapshots
      .slice(1)
      .map((current, index) => current - memorySnapshots[index]);

    const averageGrowth =
      memoryGrowth.reduce((sum, growth) => sum + growth, 0) / memoryGrowth.length;
    const leakDetected = averageGrowth > 1; // More than 1MB average growth indicates potential leak

    return {
      memoryGrowth,
      leakDetected,
      averageGrowth,
    };
  }

  // Performance Analysis
  public analyzePerformance(): {
    summary: Record<
      string,
      {
        averageTime: number;
        totalRuns: number;
        worstTime: number;
        bestTime: number;
        memoryImpact: number;
      }
    >;
    recommendations: string[];
    score: number;
  } {
    const summary: Record<string, any> = {};
    const recommendations: string[] = [];
    let totalScore = 100;

    this.results.forEach((results, testName) => {
      const avgTime = results.reduce((sum, r) => sum + r.averageTime, 0) / results.length;
      const worstTime = Math.max(...results.map((r) => r.maxTime));
      const bestTime = Math.min(...results.map((r) => r.minTime));
      const avgMemoryImpact = results.reduce((sum, r) => sum + r.memoryDelta, 0) / results.length;

      summary[testName] = {
        averageTime: avgTime,
        totalRuns: results.length,
        worstTime,
        bestTime,
        memoryImpact: avgMemoryImpact,
      };

      // Performance scoring and recommendations
      if (testName === 'animation' && avgTime > this.thresholds.frameTime) {
        totalScore -= 15;
        recommendations.push(
          `Animation frame time (${avgTime.toFixed(2)}ms) exceeds 60fps target (${this.thresholds.frameTime}ms)`
        );
      }

      if (avgMemoryImpact > 10) {
        totalScore -= 10;
        recommendations.push(
          `High memory usage detected in ${testName} (${avgMemoryImpact.toFixed(2)}MB average impact)`
        );
      }

      if (testName === 'event_processing' && avgTime > this.thresholds.eventProcessingTime) {
        totalScore -= 10;
        recommendations.push(`Event processing is slow (${avgTime.toFixed(2)}ms average)`);
      }

      if (testName === 'rendering_performance' && avgTime > this.thresholds.renderTime) {
        totalScore -= 10;
        recommendations.push(
          `Rendering performance is below target (${avgTime.toFixed(2)}ms average)`
        );
      }
    });

    return {
      summary,
      recommendations,
      score: Math.max(0, totalScore),
    };
  }

  // Threshold Management
  public setThresholds(newThresholds: Partial<PerformanceThresholds>): void {
    this.thresholds = { ...this.thresholds, ...newThresholds };
  }

  public getThresholds(): PerformanceThresholds {
    return { ...this.thresholds };
  }

  // Suite Management
  public createSuite(name: string): BenchmarkSuite {
    const environment = {
      userAgent: navigator.userAgent,
      platform: navigator.platform,
      memory: this.getMemoryUsage(),
      cores: navigator.hardwareConcurrency,
    };

    return {
      name,
      results: [],
      totalDuration: 0,
      timestamp: new Date(),
      environment,
    };
  }

  public async runSuite(
    suite: BenchmarkSuite,
    benchmarks: Array<{
      name: string;
      test: () => void | Promise<void>;
      iterations?: number;
    }>
  ): Promise<BenchmarkSuite> {
    const suiteStartTime = performance.now();

    for (const benchmark of benchmarks) {
      const result = await this.benchmark(benchmark.name, benchmark.test, benchmark.iterations);
      suite.results.push(result);
    }

    suite.totalDuration = performance.now() - suiteStartTime;
    return suite;
  }

  // Data Management
  public getResults(testName?: string): BenchmarkResult[] {
    if (testName) {
      return this.results.get(testName) || [];
    }

    const allResults: BenchmarkResult[] = [];
    this.results.forEach((results) => {
      allResults.push(...results);
    });
    return allResults.sort((a, b) => b.timestamp.getTime() - a.timestamp.getTime());
  }

  public exportResults(): string {
    const data = {
      timestamp: new Date(),
      results: Object.fromEntries(this.results),
      thresholds: this.thresholds,
      analysis: this.analyzePerformance(),
    };

    return JSON.stringify(data, null, 2);
  }

  public clearResults(testName?: string): void {
    if (testName) {
      this.results.delete(testName);
    } else {
      this.results.clear();
    }
  }

  // Utility Methods
  private addResult(testName: string, result: BenchmarkResult): void {
    if (!this.results.has(testName)) {
      this.results.set(testName, []);
    }
    this.results.get(testName)!.push(result);

    // Keep only last 100 results per test
    const results = this.results.get(testName)!;
    if (results.length > 100) {
      results.splice(0, results.length - 100);
    }
  }

  private calculateVariance(numbers: number[]): number {
    const mean = numbers.reduce((sum, num) => sum + num, 0) / numbers.length;
    const squaredDifferences = numbers.map((num) => Math.pow(num - mean, 2));
    return squaredDifferences.reduce((sum, diff) => sum + diff, 0) / numbers.length;
  }
}

// Global instance
export const performanceBenchmark = PerformanceBenchmark.getInstance();
