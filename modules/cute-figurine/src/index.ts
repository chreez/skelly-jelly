// Main component exports
export { SkellyCompanion } from './components/SkellyCompanion/SkellyCompanion';
export type { SkellyCompanionProps } from './components/SkellyCompanion/SkellyCompanion';

// Additional components
export { TaskDashboard } from './components/TaskDashboard';
export type { TaskDashboardProps } from './components/TaskDashboard';

export { PerformanceDashboard } from './components/PerformanceDashboard';
export type { PerformanceDashboardProps } from './components/PerformanceDashboard';

// Hooks
export { useTaskPersistence, useProductivityTracking } from './hooks/useTaskPersistence';
export type { UseTaskPersistenceReturn } from './hooks/useTaskPersistence';

export {
  usePerformanceBenchmark,
  usePerformanceMonitoring,
  useMeasuredFunction,
} from './hooks/usePerformanceBenchmark';
export type { UsePerformanceBenchmarkReturn } from './hooks/usePerformanceBenchmark';

// Services
export { taskPersistence } from './services/TaskPersistenceService';
export type {
  TaskSession,
  TaskMetrics,
  TaskGoal,
  CrossSessionData,
} from './services/TaskPersistenceService';

export { performanceBenchmark } from './services/PerformanceBenchmark';
export type {
  BenchmarkResult,
  BenchmarkSuite,
  PerformanceThresholds,
} from './services/PerformanceBenchmark';

// Types
export type { MoodState, ActivityState, CompanionState, Position } from './types/state.types';

export type {
  SkellyEvent,
  AnimationCommandEvent,
  StateClassificationEvent,
  InterventionRequestEvent,
  RewardEarnedEvent,
  UserInteractionEvent,
} from './types/events.types';

// Store hooks
export { useCompanionStore } from './state/companionStore';
export { usePreferenceStore } from './state/preferenceStore';
