import React, { useState, useMemo } from 'react';
import { usePerformanceBenchmark } from '../../hooks/usePerformanceBenchmark';
// BenchmarkResult type is used internally by usePerformanceBenchmark

export interface PerformanceDashboardProps {
  visible?: boolean;
  compact?: boolean;
  onClose?: () => void;
}

export const PerformanceDashboard: React.FC<PerformanceDashboardProps> = ({
  visible = false,
  compact = false,
  onClose,
}) => {
  const { results, analysis, runFullSuite, clearResults, exportResults, isRunning, currentTest } =
    usePerformanceBenchmark();

  const [selectedTest, setSelectedTest] = useState<string>('all');
  const [showDetails, setShowDetails] = useState(false);

  const filteredResults = useMemo(() => {
    if (selectedTest === 'all') return results;
    return results.filter((result) => result.name === selectedTest);
  }, [results, selectedTest]);

  const testNames = useMemo(() => {
    const names = new Set(results.map((result) => result.name));
    return Array.from(names).sort();
  }, [results]);

  const formatTime = (ms: number): string => {
    if (ms < 1) return `${(ms * 1000).toFixed(2)}µs`;
    if (ms < 1000) return `${ms.toFixed(2)}ms`;
    return `${(ms / 1000).toFixed(2)}s`;
  };

  const formatMemory = (mb: number): string => {
    if (mb < 1) return `${(mb * 1024).toFixed(2)}KB`;
    return `${mb.toFixed(2)}MB`;
  };

  const getPerformanceColor = (value: number, threshold: number): string => {
    if (value <= threshold) return '#22c55e'; // Green
    if (value <= threshold * 1.5) return '#f59e0b'; // Yellow
    return '#ef4444'; // Red
  };

  const downloadResults = () => {
    const data = exportResults();
    const blob = new Blob([data], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `performance-results-${new Date().toISOString().split('T')[0]}.json`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  };

  if (!visible) return null;

  return (
    <div
      style={{
        position: 'fixed',
        top: compact ? '20px' : '50px',
        left: '20px',
        width: compact ? '300px' : '400px',
        maxHeight: '80vh',
        overflowY: 'auto',
        background: 'rgba(255, 255, 255, 0.95)',
        backdropFilter: 'blur(10px)',
        border: '1px solid rgba(0, 0, 0, 0.1)',
        borderRadius: '12px',
        padding: '16px',
        boxShadow: '0 8px 32px rgba(0, 0, 0, 0.1)',
        fontSize: compact ? '11px' : '13px',
        zIndex: 1000,
        fontFamily: 'monospace',
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
        <h3 style={{ margin: 0, fontSize: compact ? '14px' : '16px' }}>⚡ Performance Monitor</h3>
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

      {/* Controls */}
      <div style={{ marginBottom: '16px' }}>
        <div style={{ display: 'flex', gap: '8px', marginBottom: '8px', flexWrap: 'wrap' }}>
          <button
            onClick={runFullSuite}
            disabled={isRunning}
            style={{
              padding: '6px 12px',
              background: isRunning ? '#ccc' : '#3b82f6',
              color: 'white',
              border: 'none',
              borderRadius: '4px',
              cursor: isRunning ? 'not-allowed' : 'pointer',
              fontSize: compact ? '10px' : '12px',
            }}
          >
            {isRunning ? 'Running...' : 'Run Suite'}
          </button>
          <button
            onClick={clearResults}
            style={{
              padding: '6px 12px',
              background: '#ef4444',
              color: 'white',
              border: 'none',
              borderRadius: '4px',
              cursor: 'pointer',
              fontSize: compact ? '10px' : '12px',
            }}
          >
            Clear
          </button>
          <button
            onClick={downloadResults}
            disabled={results.length === 0}
            style={{
              padding: '6px 12px',
              background: results.length === 0 ? '#ccc' : '#22c55e',
              color: 'white',
              border: 'none',
              borderRadius: '4px',
              cursor: results.length === 0 ? 'not-allowed' : 'pointer',
              fontSize: compact ? '10px' : '12px',
            }}
          >
            Export
          </button>
        </div>

        {/* Test filter */}
        <select
          value={selectedTest}
          onChange={(e) => setSelectedTest(e.target.value)}
          style={{
            width: '100%',
            padding: '4px',
            border: '1px solid #ccc',
            borderRadius: '4px',
            fontSize: compact ? '10px' : '12px',
          }}
        >
          <option value="all">All Tests</option>
          {testNames.map((name) => (
            <option key={name} value={name}>
              {name}
            </option>
          ))}
        </select>
      </div>

      {/* Current Status */}
      {isRunning && (
        <div
          style={{
            marginBottom: '16px',
            padding: '8px',
            background: 'rgba(59, 130, 246, 0.1)',
            borderRadius: '6px',
          }}
        >
          <div style={{ fontWeight: 'bold' }}>🔄 Running: {currentTest}</div>
        </div>
      )}

      {/* Performance Score */}
      {analysis && (
        <div style={{ marginBottom: '16px' }}>
          <h4 style={{ margin: '0 0 8px 0', fontSize: compact ? '12px' : '14px' }}>
            Performance Score
          </h4>
          <div
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: '8px',
              marginBottom: '8px',
            }}
          >
            <div
              style={{
                width: '60px',
                height: '60px',
                borderRadius: '50%',
                background: `conic-gradient(${getPerformanceColor(100 - analysis.score, 80)} 0deg ${analysis.score * 3.6}deg, #e5e7eb ${analysis.score * 3.6}deg 360deg)`,
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                fontWeight: 'bold',
                fontSize: compact ? '12px' : '14px',
              }}
            >
              {analysis.score}
            </div>
            <div style={{ flex: 1 }}>
              <div style={{ fontSize: compact ? '11px' : '12px' }}>
                Overall Score: {analysis.score}/100
              </div>
              <div style={{ fontSize: compact ? '10px' : '11px', opacity: 0.7 }}>
                {analysis.recommendations.length} recommendations
              </div>
            </div>
          </div>

          {/* Recommendations */}
          {analysis.recommendations.length > 0 && (
            <div>
              <div
                style={{
                  fontSize: compact ? '10px' : '11px',
                  fontWeight: 'bold',
                  marginBottom: '4px',
                  cursor: 'pointer',
                  color: '#3b82f6',
                }}
                onClick={() => setShowDetails(!showDetails)}
              >
                {showDetails ? '▼' : '▶'} Recommendations ({analysis.recommendations.length})
              </div>
              {showDetails && (
                <div
                  style={{
                    maxHeight: '120px',
                    overflowY: 'auto',
                    fontSize: compact ? '9px' : '10px',
                  }}
                >
                  {analysis.recommendations.map((rec, index) => (
                    <div
                      key={index}
                      style={{
                        padding: '4px',
                        margin: '2px 0',
                        background: 'rgba(239, 68, 68, 0.1)',
                        borderRadius: '3px',
                        borderLeft: '3px solid #ef4444',
                      }}
                    >
                      {rec}
                    </div>
                  ))}
                </div>
              )}
            </div>
          )}
        </div>
      )}

      {/* Test Summary */}
      {analysis && (
        <div style={{ marginBottom: '16px' }}>
          <h4 style={{ margin: '0 0 8px 0', fontSize: compact ? '12px' : '14px' }}>Test Summary</h4>
          <div
            style={{
              maxHeight: '200px',
              overflowY: 'auto',
            }}
          >
            {Object.entries(analysis.summary).map(([testName, summary]) => (
              <div
                key={testName}
                style={{
                  padding: '6px',
                  margin: '4px 0',
                  background: 'rgba(243, 244, 246, 0.8)',
                  borderRadius: '4px',
                  fontSize: compact ? '9px' : '10px',
                }}
              >
                <div style={{ fontWeight: 'bold', marginBottom: '2px' }}>{testName}</div>
                <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '4px' }}>
                  <div>Avg: {formatTime(summary.averageTime)}</div>
                  <div>Runs: {summary.totalRuns}</div>
                  <div>Best: {formatTime(summary.bestTime)}</div>
                  <div>Worst: {formatTime(summary.worstTime)}</div>
                  <div style={{ gridColumn: 'span 2' }}>
                    Memory: {formatMemory(summary.memoryImpact)}
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Recent Results */}
      <div>
        <h4 style={{ margin: '0 0 8px 0', fontSize: compact ? '12px' : '14px' }}>
          Recent Results ({filteredResults.length})
        </h4>
        {filteredResults.length === 0 ? (
          <div
            style={{
              padding: '16px',
              textAlign: 'center',
              color: '#6b7280',
              fontSize: compact ? '10px' : '11px',
            }}
          >
            No benchmark results yet
          </div>
        ) : (
          <div
            style={{
              maxHeight: '200px',
              overflowY: 'auto',
            }}
          >
            {filteredResults.slice(0, 10).map((result, index) => (
              <div
                key={`${result.name}-${index}`}
                style={{
                  padding: '6px',
                  margin: '2px 0',
                  background: 'rgba(243, 244, 246, 0.5)',
                  borderRadius: '4px',
                  fontSize: compact ? '9px' : '10px',
                }}
              >
                <div
                  style={{
                    display: 'flex',
                    justifyContent: 'space-between',
                    fontWeight: 'bold',
                    marginBottom: '2px',
                  }}
                >
                  <span>{result.name}</span>
                  <span
                    style={{
                      color: getPerformanceColor(result.averageTime, 16.67),
                    }}
                  >
                    {formatTime(result.averageTime)}
                  </span>
                </div>
                <div
                  style={{
                    display: 'grid',
                    gridTemplateColumns: 'repeat(3, 1fr)',
                    gap: '4px',
                    opacity: 0.7,
                  }}
                >
                  <div>×{result.iterations}</div>
                  <div>
                    {formatTime(result.minTime)}-{formatTime(result.maxTime)}
                  </div>
                  <div>{formatMemory(result.memoryDelta)}</div>
                </div>
                {result.metadata && Object.keys(result.metadata).length > 0 && (
                  <div
                    style={{
                      marginTop: '4px',
                      fontSize: compact ? '8px' : '9px',
                      opacity: 0.6,
                    }}
                  >
                    {Object.entries(result.metadata)
                      .filter(([_, value]) => typeof value === 'number')
                      .map(
                        ([key, value]) =>
                          `${key}: ${typeof value === 'number' ? (value as number).toFixed(2) : value}`
                      )
                      .join(' | ')}
                  </div>
                )}
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
};
