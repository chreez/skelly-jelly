import { MoodState, ActivityState, CompanionState } from '../types/state.types';

export interface TaskSession {
  id: string;
  name: string;
  startTime: Date;
  endTime?: Date;
  duration?: number;
  productivity?: number;
  focusLevel?: number;
  interventions?: number;
  rewards?: number;
  companionState: Partial<CompanionState>;
}

export interface TaskMetrics {
  totalSessions: number;
  totalDuration: number;
  averageProductivity: number;
  averageFocusLevel: number;
  streakDays: number;
  lastSessionDate: Date;
  weeklyGoal?: number;
  weeklyProgress: number;
}

export interface TaskGoal {
  id: string;
  type: 'daily' | 'weekly' | 'monthly';
  target: number;
  current: number;
  unit: 'minutes' | 'sessions' | 'focus_points';
  description: string;
  deadline?: Date;
}

export interface CrossSessionData {
  userId: string;
  sessions: TaskSession[];
  metrics: TaskMetrics;
  goals: TaskGoal[];
  preferences: {
    sessionReminders: boolean;
    goalNotifications: boolean;
    weeklyReports: boolean;
    sharingEnabled: boolean;
  };
  lastSync: Date;
}

export class TaskPersistenceService {
  private static instance: TaskPersistenceService;
  private storageKey = 'skelly-task-persistence';
  private currentSession: TaskSession | null = null;
  private syncInterval: NodeJS.Timeout | null = null;

  constructor() {
    this.initializePeriodicSync();
  }

  public static getInstance(): TaskPersistenceService {
    if (!TaskPersistenceService.instance) {
      TaskPersistenceService.instance = new TaskPersistenceService();
    }
    return TaskPersistenceService.instance;
  }

  // Session Management
  public startSession(name: string, companionState: Partial<CompanionState>): string {
    const sessionId = `session_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;

    this.currentSession = {
      id: sessionId,
      name,
      startTime: new Date(),
      companionState: { ...companionState },
      productivity: 0,
      focusLevel: companionState.focus || 50,
      interventions: 0,
      rewards: 0,
    };

    this.saveSession(this.currentSession);
    return sessionId;
  }

  public endSession(
    productivity?: number,
    finalState?: Partial<CompanionState>
  ): TaskSession | null {
    if (!this.currentSession) return null;

    const endTime = new Date();
    const duration = endTime.getTime() - this.currentSession.startTime.getTime();

    this.currentSession = {
      ...this.currentSession,
      endTime,
      duration,
      productivity: productivity || this.currentSession.productivity,
      companionState: finalState || this.currentSession.companionState,
    };

    this.saveSession(this.currentSession);
    this.updateMetrics(this.currentSession);

    const completedSession = this.currentSession;
    this.currentSession = null;

    return completedSession;
  }

  public getCurrentSession(): TaskSession | null {
    return this.currentSession;
  }

  public updateCurrentSession(updates: Partial<TaskSession>): void {
    if (this.currentSession) {
      this.currentSession = { ...this.currentSession, ...updates };
      this.saveSession(this.currentSession);
    }
  }

  // Data Persistence
  private saveSession(session: TaskSession): void {
    const data = this.loadData();
    const existingIndex = data.sessions.findIndex((s) => s.id === session.id);

    if (existingIndex >= 0) {
      data.sessions[existingIndex] = session;
    } else {
      data.sessions.push(session);
    }

    data.lastSync = new Date();
    this.saveData(data);
  }

  private loadData(): CrossSessionData {
    try {
      const stored = localStorage.getItem(this.storageKey);
      if (stored) {
        const data = JSON.parse(stored);
        // Convert date strings back to Date objects
        data.sessions = data.sessions.map((session: any) => ({
          ...session,
          startTime: new Date(session.startTime),
          endTime: session.endTime ? new Date(session.endTime) : undefined,
        }));
        data.lastSync = new Date(data.lastSync);
        data.metrics.lastSessionDate = new Date(data.metrics.lastSessionDate);
        return data;
      }
    } catch (error) {
      console.warn('Failed to load persisted task data:', error);
    }

    return this.createDefaultData();
  }

  private saveData(data: CrossSessionData): void {
    try {
      localStorage.setItem(this.storageKey, JSON.stringify(data));
    } catch (error) {
      console.error('Failed to save task persistence data:', error);
    }
  }

  private createDefaultData(): CrossSessionData {
    return {
      userId: `user_${Date.now()}`,
      sessions: [],
      metrics: {
        totalSessions: 0,
        totalDuration: 0,
        averageProductivity: 0,
        averageFocusLevel: 50,
        streakDays: 0,
        lastSessionDate: new Date(),
        weeklyProgress: 0,
      },
      goals: [
        {
          id: 'daily_focus',
          type: 'daily',
          target: 120, // 2 hours
          current: 0,
          unit: 'minutes',
          description: 'Daily focus time',
        },
        {
          id: 'weekly_sessions',
          type: 'weekly',
          target: 10,
          current: 0,
          unit: 'sessions',
          description: 'Weekly work sessions',
        },
      ],
      preferences: {
        sessionReminders: true,
        goalNotifications: true,
        weeklyReports: true,
        sharingEnabled: false,
      },
      lastSync: new Date(),
    };
  }

  // Analytics and Metrics
  private updateMetrics(session: TaskSession): void {
    const data = this.loadData();

    data.metrics.totalSessions += 1;
    data.metrics.totalDuration += session.duration || 0;
    data.metrics.lastSessionDate = session.endTime || new Date();

    // Calculate averages
    const sessions = data.sessions.filter((s) => s.endTime);
    data.metrics.averageProductivity =
      sessions.reduce((sum, s) => sum + (s.productivity || 0), 0) / sessions.length;
    data.metrics.averageFocusLevel =
      sessions.reduce((sum, s) => sum + (s.focusLevel || 50), 0) / sessions.length;

    // Update streak
    data.metrics.streakDays = this.calculateStreak(data.sessions);

    // Update goals
    this.updateGoalProgress(data, session);

    this.saveData(data);
  }

  private calculateStreak(sessions: TaskSession[]): number {
    const today = new Date();
    today.setHours(0, 0, 0, 0);

    let streak = 0;
    const currentDate = new Date(today);

    for (let i = 0; i < 30; i++) {
      // Check last 30 days max
      const dayStart = new Date(currentDate);
      const dayEnd = new Date(currentDate);
      dayEnd.setHours(23, 59, 59, 999);

      const hasSessionOnDay = sessions.some((session) => {
        const sessionDate = session.endTime || session.startTime;
        return sessionDate >= dayStart && sessionDate <= dayEnd;
      });

      if (hasSessionOnDay) {
        streak++;
      } else if (i > 0) {
        // Don't break on first day if no session today
        break;
      }

      currentDate.setDate(currentDate.getDate() - 1);
    }

    return streak;
  }

  private updateGoalProgress(data: CrossSessionData, session: TaskSession): void {
    const now = new Date();

    data.goals.forEach((goal) => {
      if (goal.type === 'daily') {
        const today = new Date(now);
        today.setHours(0, 0, 0, 0);

        const todaysSessions = data.sessions.filter((s) => {
          const sessionDate = s.endTime || s.startTime;
          return sessionDate >= today;
        });

        if (goal.unit === 'minutes') {
          goal.current =
            todaysSessions.reduce((sum, s) => sum + (s.duration || 0), 0) / (1000 * 60);
        } else if (goal.unit === 'sessions') {
          goal.current = todaysSessions.length;
        }
      } else if (goal.type === 'weekly') {
        const weekStart = new Date(now);
        weekStart.setDate(now.getDate() - now.getDay());
        weekStart.setHours(0, 0, 0, 0);

        const weekSessions = data.sessions.filter((s) => {
          const sessionDate = s.endTime || s.startTime;
          return sessionDate >= weekStart;
        });

        if (goal.unit === 'minutes') {
          goal.current = weekSessions.reduce((sum, s) => sum + (s.duration || 0), 0) / (1000 * 60);
        } else if (goal.unit === 'sessions') {
          goal.current = weekSessions.length;
        }
      }
    });
  }

  // Public API
  public getMetrics(): TaskMetrics {
    return this.loadData().metrics;
  }

  public getSessions(limit?: number): TaskSession[] {
    const data = this.loadData();
    const sessions = data.sessions
      .filter((s) => s.endTime) // Only completed sessions
      .sort((a, b) => (b.endTime?.getTime() || 0) - (a.endTime?.getTime() || 0));

    return limit ? sessions.slice(0, limit) : sessions;
  }

  public getGoals(): TaskGoal[] {
    return this.loadData().goals;
  }

  public updateGoal(goalId: string, updates: Partial<TaskGoal>): void {
    const data = this.loadData();
    const goalIndex = data.goals.findIndex((g) => g.id === goalId);

    if (goalIndex >= 0) {
      data.goals[goalIndex] = { ...data.goals[goalIndex], ...updates };
      this.saveData(data);
    }
  }

  public exportData(): CrossSessionData {
    return this.loadData();
  }

  public importData(importedData: CrossSessionData): void {
    this.saveData(importedData);
  }

  public clearData(): void {
    localStorage.removeItem(this.storageKey);
  }

  // Periodic sync for long-running sessions
  private initializePeriodicSync(): void {
    this.syncInterval = setInterval(() => {
      if (this.currentSession) {
        this.updateCurrentSession({
          duration: Date.now() - this.currentSession.startTime.getTime(),
        });
      }
    }, 60000); // Update every minute
  }

  public dispose(): void {
    if (this.syncInterval) {
      clearInterval(this.syncInterval);
      this.syncInterval = null;
    }
  }
}

// Global instance
export const taskPersistence = TaskPersistenceService.getInstance();
