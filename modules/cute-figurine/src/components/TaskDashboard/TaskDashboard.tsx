import React, { useState } from 'react';
import { useTaskPersistence } from '../../hooks/useTaskPersistence';

export interface TaskDashboardProps {
  visible?: boolean;
  compact?: boolean;
  onClose?: () => void;
}

export const TaskDashboard: React.FC<TaskDashboardProps> = ({
  visible = false,
  compact = false,
  onClose,
}) => {
  const {
    currentSession,
    isSessionActive,
    startSession,
    endSession,
    metrics,
    recentSessions,
    goals,
    todaysProgress,
    weeklyProgress,
    streakDays,
    isGoalMet,
  } = useTaskPersistence();

  const [newSessionName, setNewSessionName] = useState('');
  const [showDetails, setShowDetails] = useState(false);

  if (!visible) return null;

  const handleStartSession = () => {
    if (newSessionName.trim()) {
      startSession(newSessionName.trim());
      setNewSessionName('');
    }
  };

  const handleEndSession = () => {
    endSession(85); // Example productivity score
  };

  const formatDuration = (ms: number): string => {
    const minutes = Math.floor(ms / (1000 * 60));
    const hours = Math.floor(minutes / 60);
    if (hours > 0) {
      return `${hours}h ${minutes % 60}m`;
    }
    return `${minutes}m`;
  };

  const formatDate = (date: Date): string => {
    return new Intl.DateTimeFormat('en-US', {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    }).format(date);
  };

  return (
    <div
      style={{
        position: 'fixed',
        top: compact ? '20px' : '50px',
        right: '20px',
        width: compact ? '280px' : '320px',
        maxHeight: '70vh',
        overflowY: 'auto',
        background: 'rgba(255, 255, 255, 0.95)',
        backdropFilter: 'blur(10px)',
        border: '1px solid rgba(0, 0, 0, 0.1)',
        borderRadius: '12px',
        padding: '16px',
        boxShadow: '0 8px 32px rgba(0, 0, 0, 0.1)',
        fontSize: compact ? '12px' : '14px',
        zIndex: 1000,
      }}
    >
      {/* Header */}
      <div
        style={{
          display: 'flex',
          justifyContent: 'space-between',
          alignItems: 'center',
          marginBottom: '16px',
        }}
      >
        <h3 style={{ margin: 0, fontSize: compact ? '14px' : '16px' }}>📊 Task Dashboard</h3>
        {onClose && (
          <button
            onClick={onClose}
            style={{
              background: 'none',
              border: 'none',
              fontSize: '16px',
              cursor: 'pointer',
              padding: '4px',
            }}
          >
            ×
          </button>
        )}
      </div>

      {/* Current Session */}
      <div style={{ marginBottom: '16px' }}>
        <h4 style={{ margin: '0 0 8px 0', fontSize: compact ? '12px' : '14px' }}>
          Current Session
        </h4>
        {isSessionActive ? (
          <div
            style={{
              background: 'rgba(34, 197, 94, 0.1)',
              padding: '8px',
              borderRadius: '6px',
            }}
          >
            <div style={{ fontWeight: 'bold' }}>{currentSession?.name}</div>
            <div style={{ fontSize: compact ? '10px' : '12px', opacity: 0.7 }}>
              Started: {currentSession && formatDate(currentSession.startTime)}
            </div>
            <div style={{ fontSize: compact ? '10px' : '12px', opacity: 0.7 }}>
              Duration:{' '}
              {currentSession && formatDuration(Date.now() - currentSession.startTime.getTime())}
            </div>
            <button
              onClick={handleEndSession}
              style={{
                marginTop: '8px',
                padding: '4px 8px',
                background: '#ef4444',
                color: 'white',
                border: 'none',
                borderRadius: '4px',
                cursor: 'pointer',
                fontSize: compact ? '10px' : '12px',
              }}
            >
              End Session
            </button>
          </div>
        ) : (
          <div>
            <input
              type="text"
              placeholder="Session name..."
              value={newSessionName}
              onChange={(e) => setNewSessionName(e.target.value)}
              style={{
                width: '100%',
                padding: '6px',
                border: '1px solid #ccc',
                borderRadius: '4px',
                fontSize: compact ? '10px' : '12px',
                marginBottom: '8px',
              }}
            />
            <button
              onClick={handleStartSession}
              disabled={!newSessionName.trim()}
              style={{
                padding: '6px 12px',
                background: newSessionName.trim() ? '#22c55e' : '#ccc',
                color: 'white',
                border: 'none',
                borderRadius: '4px',
                cursor: newSessionName.trim() ? 'pointer' : 'not-allowed',
                fontSize: compact ? '10px' : '12px',
              }}
            >
              Start Session
            </button>
          </div>
        )}
      </div>

      {/* Today's Progress */}
      <div style={{ marginBottom: '16px' }}>
        <h4 style={{ margin: '0 0 8px 0', fontSize: compact ? '12px' : '14px' }}>
          Today's Progress
        </h4>
        <div
          style={{
            background: 'rgba(59, 130, 246, 0.1)',
            padding: '8px',
            borderRadius: '6px',
          }}
        >
          <div>Focus Time: {Math.round(todaysProgress)}min</div>
          <div>Streak: {streakDays} days 🔥</div>
          <div>Sessions: {metrics.totalSessions}</div>
        </div>
      </div>

      {/* Goals */}
      <div style={{ marginBottom: '16px' }}>
        <h4 style={{ margin: '0 0 8px 0', fontSize: compact ? '12px' : '14px' }}>Goals</h4>
        {goals.map((goal) => (
          <div
            key={goal.id}
            style={{
              marginBottom: '6px',
              padding: '6px',
              background: isGoalMet(goal.id)
                ? 'rgba(34, 197, 94, 0.1)'
                : 'rgba(156, 163, 175, 0.1)',
              borderRadius: '4px',
              fontSize: compact ? '10px' : '12px',
            }}
          >
            <div style={{ fontWeight: 'bold' }}>
              {goal.description} {isGoalMet(goal.id) ? '✅' : ''}
            </div>
            <div>
              {Math.round(goal.current)}/{goal.target} {goal.unit}
            </div>
            <div
              style={{
                width: '100%',
                height: '4px',
                background: '#e5e7eb',
                borderRadius: '2px',
                marginTop: '4px',
                overflow: 'hidden',
              }}
            >
              <div
                style={{
                  width: `${Math.min((goal.current / goal.target) * 100, 100)}%`,
                  height: '100%',
                  background: isGoalMet(goal.id) ? '#22c55e' : '#3b82f6',
                  transition: 'width 0.3s ease',
                }}
              />
            </div>
          </div>
        ))}
      </div>

      {/* Recent Sessions */}
      {!compact && (
        <div>
          <div
            style={{
              display: 'flex',
              justifyContent: 'space-between',
              alignItems: 'center',
              marginBottom: '8px',
            }}
          >
            <h4 style={{ margin: 0, fontSize: '14px' }}>Recent Sessions</h4>
            <button
              onClick={() => setShowDetails(!showDetails)}
              style={{
                background: 'none',
                border: 'none',
                fontSize: '12px',
                cursor: 'pointer',
                color: '#3b82f6',
              }}
            >
              {showDetails ? 'Hide' : 'Show'}
            </button>
          </div>
          {showDetails && (
            <div style={{ maxHeight: '150px', overflowY: 'auto' }}>
              {recentSessions.length === 0 ? (
                <div
                  style={{
                    padding: '12px',
                    textAlign: 'center',
                    color: '#6b7280',
                    fontSize: '12px',
                  }}
                >
                  No completed sessions yet
                </div>
              ) : (
                recentSessions.map((session) => (
                  <div
                    key={session.id}
                    style={{
                      padding: '6px',
                      marginBottom: '4px',
                      background: 'rgba(243, 244, 246, 0.5)',
                      borderRadius: '4px',
                      fontSize: '11px',
                    }}
                  >
                    <div style={{ fontWeight: 'bold' }}>{session.name}</div>
                    <div style={{ opacity: 0.7 }}>
                      {session.endTime && formatDate(session.endTime)} •
                      {session.duration && formatDuration(session.duration)} •{session.productivity}
                      % productivity
                    </div>
                  </div>
                ))
              )}
            </div>
          )}
        </div>
      )}
    </div>
  );
};
