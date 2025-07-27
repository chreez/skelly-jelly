import { describe, it, expect, beforeEach, vi } from 'vitest';
import { TaskPersistenceService, TaskSession, TaskMetrics } from './TaskPersistenceService';
import { MoodState, ActivityState } from '../types/state.types';

// Mock localStorage
const localStorageMock = {
  getItem: vi.fn(),
  setItem: vi.fn(),
  removeItem: vi.fn(),
  clear: vi.fn(),
};
global.localStorage = localStorageMock as any;

describe('TaskPersistenceService', () => {
  let service: TaskPersistenceService;

  beforeEach(() => {
    vi.clearAllMocks();
    localStorageMock.getItem.mockReturnValue(null);

    // Create a new instance for each test
    service = new (TaskPersistenceService as any)();
  });

  describe('Session Management', () => {
    it('starts a new session correctly', () => {
      const sessionName = 'Test Work Session';
      const companionState = {
        mood: MoodState.FOCUSED,
        energy: 80,
        focus: 75,
      };

      const sessionId = service.startSession(sessionName, companionState);

      expect(sessionId).toMatch(/^session_\d+_\w+$/);

      const currentSession = service.getCurrentSession();
      expect(currentSession).toMatchObject({
        id: sessionId,
        name: sessionName,
        companionState: expect.objectContaining(companionState),
        productivity: 0,
        focusLevel: 75,
        interventions: 0,
        rewards: 0,
      });
      expect(currentSession?.startTime).toBeInstanceOf(Date);
    });

    it('ends a session correctly', () => {
      const sessionName = 'Test Session';
      const companionState = { mood: MoodState.HAPPY, energy: 90 };

      const sessionId = service.startSession(sessionName, companionState);

      // Wait a moment to ensure duration > 0
      const endTime = new Date(Date.now() + 1000);
      vi.setSystemTime(endTime);

      const finalState = { mood: MoodState.TIRED, energy: 60 };
      const productivity = 85;

      const completedSession = service.endSession(productivity, finalState);

      expect(completedSession).toMatchObject({
        id: sessionId,
        name: sessionName,
        productivity,
        companionState: expect.objectContaining(finalState),
      });
      expect(completedSession?.endTime).toBeInstanceOf(Date);
      expect(completedSession?.duration).toBeGreaterThan(0);

      // Current session should be null after ending
      expect(service.getCurrentSession()).toBeNull();
    });

    it('updates current session correctly', () => {
      const sessionName = 'Test Session';
      service.startSession(sessionName, {});

      const updates = {
        productivity: 50,
        interventions: 2,
        rewards: 1,
      };

      service.updateCurrentSession(updates);

      const currentSession = service.getCurrentSession();
      expect(currentSession).toMatchObject(updates);
    });

    it('handles no current session gracefully', () => {
      const result = service.endSession(80);
      expect(result).toBeNull();

      service.updateCurrentSession({ productivity: 50 });
      // Should not throw an error
    });
  });

  describe('Data Persistence', () => {
    it('saves and loads session data correctly', () => {
      const sessionName = 'Persistent Session';
      const companionState = { mood: MoodState.EXCITED, energy: 95 };

      service.startSession(sessionName, companionState);

      // Verify localStorage.setItem was called
      expect(localStorageMock.setItem).toHaveBeenCalled();

      // Get the saved data
      const saveCall = localStorageMock.setItem.mock.calls[0];
      expect(saveCall[0]).toBe('skelly-task-persistence');

      const savedData = JSON.parse(saveCall[1]);
      expect(savedData.sessions).toHaveLength(1);
      expect(savedData.sessions[0]).toMatchObject({
        name: sessionName,
        companionState: expect.objectContaining(companionState),
      });
    });

    it('creates default data when no stored data exists', () => {
      const metrics = service.getMetrics();

      expect(metrics).toMatchObject({
        totalSessions: 0,
        totalDuration: 0,
        averageProductivity: 0,
        averageFocusLevel: 50,
        streakDays: 0,
        weeklyProgress: 0,
      });
      expect(metrics.lastSessionDate).toBeInstanceOf(Date);
    });

    it('handles localStorage errors gracefully', () => {
      localStorageMock.getItem.mockImplementation(() => {
        throw new Error('localStorage error');
      });

      // Should not throw, should return default data
      const metrics = service.getMetrics();
      expect(metrics.totalSessions).toBe(0);
    });
  });

  describe('Metrics and Analytics', () => {
    it('updates metrics after completing sessions', () => {
      // Complete a few sessions
      for (let i = 0; i < 3; i++) {
        service.startSession(`Session ${i + 1}`, { energy: 80 });

        // Simulate time passing
        vi.setSystemTime(Date.now() + 1000 * 60 * 30); // 30 minutes

        service.endSession(70 + i * 10, { energy: 60 });
      }

      const metrics = service.getMetrics();
      expect(metrics.totalSessions).toBe(3);
      expect(metrics.totalDuration).toBeGreaterThan(0);
      expect(metrics.averageProductivity).toBeCloseTo(80); // (70 + 80 + 90) / 3
    });

    it('calculates streak correctly', () => {
      const now = new Date();

      // Create sessions for today and yesterday
      const todaySession: TaskSession = {
        id: 'today',
        name: 'Today Session',
        startTime: new Date(now.getFullYear(), now.getMonth(), now.getDate(), 10, 0),
        endTime: new Date(now.getFullYear(), now.getMonth(), now.getDate(), 11, 0),
        duration: 3600000, // 1 hour
        companionState: {},
        productivity: 80,
        focusLevel: 70,
        interventions: 0,
        rewards: 1,
      };

      const yesterdaySession: TaskSession = {
        id: 'yesterday',
        name: 'Yesterday Session',
        startTime: new Date(now.getFullYear(), now.getMonth(), now.getDate() - 1, 10, 0),
        endTime: new Date(now.getFullYear(), now.getMonth(), now.getDate() - 1, 11, 0),
        duration: 3600000,
        companionState: {},
        productivity: 75,
        focusLevel: 65,
        interventions: 1,
        rewards: 0,
      };

      // Mock localStorage to return sessions
      const mockData = {
        userId: 'test-user',
        sessions: [todaySession, yesterdaySession],
        metrics: {
          totalSessions: 2,
          totalDuration: 7200000,
          averageProductivity: 77.5,
          averageFocusLevel: 67.5,
          streakDays: 0, // Will be calculated
          lastSessionDate: todaySession.endTime,
          weeklyProgress: 0,
        },
        goals: [],
        preferences: {
          sessionReminders: true,
          goalNotifications: true,
          weeklyReports: true,
          sharingEnabled: false,
        },
        lastSync: now,
      };

      localStorageMock.getItem.mockReturnValue(JSON.stringify(mockData));

      // Create new service instance to load mocked data
      const testService = new (TaskPersistenceService as any)();
      const metrics = testService.getMetrics();

      expect(metrics.streakDays).toBeGreaterThanOrEqual(1); // Should have at least 1 day streak
    });
  });

  describe('Goal Management', () => {
    it('updates daily goals correctly', () => {
      const goals = service.getGoals();
      const dailyGoal = goals.find((g) => g.type === 'daily');

      expect(dailyGoal).toBeDefined();
      expect(dailyGoal?.target).toBeGreaterThan(0);
      expect(dailyGoal?.unit).toBe('minutes');
    });

    it('updates goal correctly', () => {
      const goals = service.getGoals();
      const goalToUpdate = goals[0];

      const updates = { target: 180, current: 90 };
      service.updateGoal(goalToUpdate.id, updates);

      const updatedGoals = service.getGoals();
      const updatedGoal = updatedGoals.find((g) => g.id === goalToUpdate.id);

      expect(updatedGoal).toMatchObject(updates);
    });
  });

  describe('Data Export/Import', () => {
    it('exports data correctly', () => {
      service.startSession('Export Test', { energy: 80 });

      const exportedData = service.exportData();

      expect(exportedData).toHaveProperty('userId');
      expect(exportedData).toHaveProperty('sessions');
      expect(exportedData).toHaveProperty('metrics');
      expect(exportedData).toHaveProperty('goals');
      expect(exportedData).toHaveProperty('preferences');
      expect(exportedData.sessions).toHaveLength(1);
    });

    it('clears data correctly', () => {
      service.startSession('Clear Test', { energy: 80 });

      service.clearData();

      expect(localStorageMock.removeItem).toHaveBeenCalledWith('skelly-task-persistence');
    });
  });

  describe('Resource Management', () => {
    it('disposes correctly', () => {
      const spy = vi.spyOn(global, 'clearInterval');

      service.dispose();

      expect(spy).toHaveBeenCalled();
    });
  });
});
