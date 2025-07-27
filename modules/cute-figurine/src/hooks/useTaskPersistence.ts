import { useState, useEffect, useCallback } from 'react';
import {
  TaskPersistenceService,
  TaskSession,
  TaskMetrics,
  TaskGoal,
  taskPersistence,
} from '../services/TaskPersistenceService';
import { useCompanionStore } from '../state/companionStore';

export interface UseTaskPersistenceReturn {
  // Current session
  currentSession: TaskSession | null;
  isSessionActive: boolean;

  // Session management
  startSession: (name: string) => string;
  endSession: (productivity?: number) => TaskSession | null;
  updateSession: (updates: Partial<TaskSession>) => void;

  // Data access
  metrics: TaskMetrics;
  recentSessions: TaskSession[];
  goals: TaskGoal[];

  // Actions
  refreshData: () => void;
  exportData: () => any;
  clearData: () => void;

  // Goal management
  updateGoal: (goalId: string, updates: Partial<TaskGoal>) => void;

  // Computed values
  todaysProgress: number;
  weeklyProgress: number;
  streakDays: number;
  isGoalMet: (goalId: string) => boolean;
}

export const useTaskPersistence = (): UseTaskPersistenceReturn => {
  const [currentSession, setCurrentSession] = useState<TaskSession | null>(null);
  const [metrics, setMetrics] = useState<TaskMetrics>(() => taskPersistence.getMetrics());
  const [recentSessions, setRecentSessions] = useState<TaskSession[]>([]);
  const [goals, setGoals] = useState<TaskGoal[]>([]);

  const companionState = useCompanionStore();

  // Refresh all data from persistence service
  const refreshData = useCallback(() => {
    setCurrentSession(taskPersistence.getCurrentSession());
    setMetrics(taskPersistence.getMetrics());
    setRecentSessions(taskPersistence.getSessions(10)); // Last 10 sessions
    setGoals(taskPersistence.getGoals());
  }, []);

  // Initialize data on mount
  useEffect(() => {
    refreshData();

    // Set up periodic refresh
    const interval = setInterval(refreshData, 30000); // Every 30 seconds

    return () => clearInterval(interval);
  }, [refreshData]);

  // Session management
  const startSession = useCallback(
    (name: string): string => {
      const sessionId = taskPersistence.startSession(name, {
        mood: companionState.mood,
        energy: companionState.energy,
        happiness: companionState.happiness,
        focus: companionState.focus,
        meltLevel: companionState.meltLevel,
        position: companionState.position,
      });

      refreshData();
      return sessionId;
    },
    [companionState, refreshData]
  );

  const endSession = useCallback(
    (productivity?: number): TaskSession | null => {
      const completedSession = taskPersistence.endSession(productivity, {
        mood: companionState.mood,
        energy: companionState.energy,
        happiness: companionState.happiness,
        focus: companionState.focus,
        meltLevel: companionState.meltLevel,
        position: companionState.position,
      });

      refreshData();
      return completedSession;
    },
    [companionState, refreshData]
  );

  const updateSession = useCallback(
    (updates: Partial<TaskSession>) => {
      taskPersistence.updateCurrentSession(updates);
      refreshData();
    },
    [refreshData]
  );

  // Goal management
  const updateGoal = useCallback(
    (goalId: string, updates: Partial<TaskGoal>) => {
      taskPersistence.updateGoal(goalId, updates);
      refreshData();
    },
    [refreshData]
  );

  // Data management
  const exportData = useCallback(() => {
    return taskPersistence.exportData();
  }, []);

  const clearData = useCallback(() => {
    taskPersistence.clearData();
    refreshData();
  }, [refreshData]);

  // Computed values
  const isSessionActive = currentSession !== null;

  const todaysProgress =
    goals.find((g) => g.type === 'daily' && g.unit === 'minutes')?.current || 0;
  const weeklyProgress = goals.find((g) => g.type === 'weekly')?.current || 0;
  const streakDays = metrics.streakDays;

  const isGoalMet = useCallback(
    (goalId: string): boolean => {
      const goal = goals.find((g) => g.id === goalId);
      return goal ? goal.current >= goal.target : false;
    },
    [goals]
  );

  return {
    // Current session
    currentSession,
    isSessionActive,

    // Session management
    startSession,
    endSession,
    updateSession,

    // Data access
    metrics,
    recentSessions,
    goals,

    // Actions
    refreshData,
    exportData,
    clearData,

    // Goal management
    updateGoal,

    // Computed values
    todaysProgress,
    weeklyProgress,
    streakDays,
    isGoalMet,
  };
};

// Hook for productivity tracking integration
export const useProductivityTracking = () => {
  const { updateSession, currentSession } = useTaskPersistence();
  const companionState = useCompanionStore();

  const recordProductivityEvent = useCallback(
    (
      eventType: 'focus_start' | 'focus_end' | 'break' | 'distraction' | 'achievement',
      value?: number
    ) => {
      if (!currentSession) return;

      const updates: Partial<TaskSession> = {};

      switch (eventType) {
        case 'focus_start':
          updates.focusLevel = Math.min((currentSession.focusLevel || 50) + 10, 100);
          break;
        case 'focus_end':
          updates.productivity = Math.min((currentSession.productivity || 0) + (value || 5), 100);
          break;
        case 'break':
          // Slight productivity boost for taking healthy breaks
          updates.productivity = Math.min((currentSession.productivity || 0) + 2, 100);
          break;
        case 'distraction':
          updates.focusLevel = Math.max((currentSession.focusLevel || 50) - 5, 0);
          break;
        case 'achievement':
          updates.productivity = Math.min((currentSession.productivity || 0) + (value || 10), 100);
          updates.rewards = (currentSession.rewards || 0) + 1;
          break;
      }

      updateSession(updates);
    },
    [currentSession, updateSession]
  );

  const recordIntervention = useCallback(() => {
    if (currentSession) {
      updateSession({
        interventions: (currentSession.interventions || 0) + 1,
      });
    }
  }, [currentSession, updateSession]);

  return {
    recordProductivityEvent,
    recordIntervention,
    currentProductivity: currentSession?.productivity || 0,
    currentFocusLevel: currentSession?.focusLevel || companionState.focus,
  };
};
