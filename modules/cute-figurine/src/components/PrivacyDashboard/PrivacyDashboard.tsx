/**
 * Privacy-Preserving Analytics Dashboard
 * 
 * Provides users complete control over their data with export/deletion
 * capabilities and transparency into all privacy operations.
 */

import React, { useState, useEffect, useCallback } from 'react';
import './PrivacyDashboard.css';

// Privacy data types
interface PrivacyStats {
  screenshotsStored: number;
  screenshotsAnalyzed: number;
  screenshotsDeleted: number;
  totalDataSize: string;
  oldestDataAge: string;
  piiDetectionsToday: number;
  piiAccuracy: number;
  networkIsolationVerified: boolean;
  localInferenceVerified: boolean;
  encryptionEnabled: boolean;
  lastScreenshotCleanup: string;
}

interface AuditLogEntry {
  timestamp: string;
  action: string;
  details: string;
  success: boolean;
  dataAffected: string;
}

interface ExportOptions {
  format: 'json' | 'csv' | 'xml';
  dateRange: 'today' | 'week' | 'month' | 'all';
  includeScreenshots: boolean;
  includeBehavioralData: boolean;
  includeAuditLog: boolean;
  anonymize: boolean;
}

interface DeletionOptions {
  dataType: 'all' | 'screenshots' | 'behavioral' | 'audit_logs';
  dateRange: 'today' | 'week' | 'month' | 'all';
  secureOverwrite: boolean;
}

const PrivacyDashboard: React.FC = () => {
  const [stats, setStats] = useState<PrivacyStats | null>(null);
  const [auditLog, setAuditLog] = useState<AuditLogEntry[]>([]);
  const [exportOptions, setExportOptions] = useState<ExportOptions>({
    format: 'json',
    dateRange: 'week',
    includeScreenshots: false,
    includeBehavioralData: true,
    includeAuditLog: true,
    anonymize: true,
  });
  const [deletionOptions, setDeletionOptions] = useState<DeletionOptions>({
    dataType: 'screenshots',
    dateRange: 'week',
    secureOverwrite: true,
  });
  const [loading, setLoading] = useState(false);
  const [activeTab, setActiveTab] = useState<'overview' | 'export' | 'delete' | 'audit'>('overview');

  // Load privacy statistics
  const loadPrivacyStats = useCallback(async () => {
    try {
      setLoading(true);
      // API call to get privacy statistics
      const response = await fetch('/api/privacy/stats');
      const data = await response.json();
      setStats(data);
    } catch (error) {
      console.error('Failed to load privacy stats:', error);
    } finally {
      setLoading(false);
    }
  }, []);

  // Load audit log
  const loadAuditLog = useCallback(async () => {
    try {
      const response = await fetch('/api/privacy/audit-log');
      const data = await response.json();
      setAuditLog(data);
    } catch (error) {
      console.error('Failed to load audit log:', error);
    }
  }, []);

  // Export user data
  const handleExportData = async () => {
    try {
      setLoading(true);
      
      const response = await fetch('/api/privacy/export', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(exportOptions),
      });

      if (response.ok) {
        // Trigger download
        const blob = await response.blob();
        const url = window.URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = `skelly-jelly-data-${Date.now()}.${exportOptions.format}`;
        document.body.appendChild(a);
        a.click();
        window.URL.revokeObjectURL(url);
        document.body.removeChild(a);
        
        alert('Data exported successfully!');
      } else {
        throw new Error('Export failed');
      }
    } catch (error) {
      console.error('Export failed:', error);
      alert('Failed to export data. Please try again.');
    } finally {
      setLoading(false);
    }
  };

  // Delete user data
  const handleDeleteData = async () => {
    const confirmMessage = `Are you sure you want to delete ${deletionOptions.dataType} data from ${deletionOptions.dateRange}? This action cannot be undone.`;
    
    if (!window.confirm(confirmMessage)) {
      return;
    }

    try {
      setLoading(true);
      
      const response = await fetch('/api/privacy/delete', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(deletionOptions),
      });

      if (response.ok) {
        const result = await response.json();
        alert(`Successfully deleted ${result.itemsDeleted} items. ${result.bytesFreed} bytes freed.`);
        
        // Refresh stats
        await loadPrivacyStats();
        await loadAuditLog();
      } else {
        throw new Error('Deletion failed');
      }
    } catch (error) {
      console.error('Deletion failed:', error);
      alert('Failed to delete data. Please try again.');
    } finally {
      setLoading(false);
    }
  };

  // Trigger immediate screenshot cleanup
  const handleForceCleanup = async () => {
    try {
      setLoading(true);
      
      const response = await fetch('/api/privacy/force-cleanup', {
        method: 'POST',
      });

      if (response.ok) {
        const result = await response.json();
        alert(`Cleanup completed. ${result.screenshotsDeleted} screenshots deleted.`);
        await loadPrivacyStats();
      } else {
        throw new Error('Cleanup failed');
      }
    } catch (error) {
      console.error('Cleanup failed:', error);
      alert('Failed to run cleanup. Please try again.');
    } finally {
      setLoading(false);
    }
  };

  // Verify network isolation in real-time
  const handleVerifyNetworkIsolation = async () => {
    try {
      setLoading(true);
      
      const response = await fetch('/api/privacy/verify-network-isolation');
      
      if (response.ok) {
        const result = await response.json();
        const message = `Network Isolation Report:
• Total Operations: ${result.totalOperations}
• Network Attempts: ${result.networkAttempts}
• Local Processing Rate: ${(result.localProcessingRate * 100).toFixed(1)}%
• Isolation Verified: ${result.isolationVerified ? 'YES' : 'NO'}

${result.isolationVerified ? '✅ Zero external network calls confirmed' : '❌ Network access detected!'}`;
        
        alert(message);
        await loadPrivacyStats();
      } else {
        throw new Error('Verification failed');
      }
    } catch (error) {
      console.error('Network isolation verification failed:', error);
      alert('Failed to verify network isolation. Please try again.');
    } finally {
      setLoading(false);
    }
  };

  // Test PII detection accuracy
  const handleTestPIIDetection = async () => {
    try {
      setLoading(true);
      
      const response = await fetch('/api/privacy/test-pii-detection', {
        method: 'POST',
      });

      if (response.ok) {
        const result = await response.json();
        const message = `PII Detection Test Results:
• Test Cases: ${result.totalTestCases}
• Detected: ${result.totalDetections}
• High Confidence (≥95%): ${result.highConfidenceDetections}
• Accuracy: ${result.accuracy.toFixed(1)}%
• False Positives: ${result.falsePositiveRate.toFixed(1)}%

${result.accuracy >= 95.0 && result.falsePositiveRate <= 1.0 ? '✅ Requirements met' : '❌ Requirements not met'}`;
        
        alert(message);
        await loadPrivacyStats();
      } else {
        throw new Error('PII test failed');
      }
    } catch (error) {
      console.error('PII detection test failed:', error);
      alert('Failed to test PII detection. Please try again.');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadPrivacyStats();
    loadAuditLog();
    
    // Set up auto-refresh every 30 seconds
    const interval = setInterval(() => {
      loadPrivacyStats();
      if (activeTab === 'audit') {
        loadAuditLog();
      }
    }, 30000);

    return () => clearInterval(interval);
  }, [loadPrivacyStats, loadAuditLog, activeTab]);

  const renderOverviewTab = () => (
    <div className=\"privacy-overview\">
      <div className=\"privacy-stats-grid\">
        <div className=\"stat-card\">
          <h3>Screenshots</h3>
          <div className=\"stat-value\">{stats?.screenshotsStored || 0}</div>
          <div className=\"stat-label\">Currently Stored</div>
          <div className=\"stat-detail\">
            {stats?.screenshotsAnalyzed || 0} analyzed, {stats?.screenshotsDeleted || 0} auto-deleted
          </div>
        </div>

        <div className=\"stat-card\">
          <h3>Data Size</h3>
          <div className=\"stat-value\">{stats?.totalDataSize || '0 MB'}</div>
          <div className=\"stat-label\">Total Storage Used</div>
          <div className=\"stat-detail\">
            Oldest data: {stats?.oldestDataAge || 'N/A'}
          </div>
        </div>

        <div className=\"stat-card\">
          <h3>PII Protection</h3>
          <div className=\"stat-value\">{stats?.piiDetectionsToday || 0}</div>
          <div className=\"stat-label\">Detections Today</div>
          <div className=\"stat-detail\">
            {((stats?.piiAccuracy || 0) * 100).toFixed(1)}% accuracy
          </div>
        </div>

        <div className=\"stat-card privacy-guarantee\">
          <h3>Privacy Status</h3>
          <div className=\"stat-value\">
            {stats?.networkIsolationVerified && stats?.localInferenceVerified ? '✓ SECURE' : '⚠️ CHECKING'}
          </div>
          <div className=\"stat-label\">
            {stats?.networkIsolationVerified ? '100% Local' : 'Verifying...'}
          </div>
          <div className=\"stat-detail\">
            Network Isolation: {stats?.networkIsolationVerified ? '✅' : '🔍'}<br/>
            Local Inference: {stats?.localInferenceVerified ? '✅' : '🔍'}<br/>
            Encryption: {stats?.encryptionEnabled ? '✅' : '❌'}
          </div>
        </div>
      </div>

      <div className=\"privacy-controls\">
        <h3>Quick Actions</h3>
        <div className=\"control-buttons\">
          <button 
            className=\"btn-secondary\"
            onClick={handleForceCleanup}
            disabled={loading}
          >
            🧹 Force Screenshot Cleanup
          </button>
          <button 
            className=\"btn-secondary\"
            onClick={handleVerifyNetworkIsolation}
            disabled={loading}
          >
            🔍 Verify Network Isolation
          </button>
          <button 
            className=\"btn-secondary\"
            onClick={handleTestPIIDetection}
            disabled={loading}
          >
            🧪 Test PII Detection
          </button>
          <button 
            className=\"btn-primary\"
            onClick={() => setActiveTab('export')}
          >
            📦 Export My Data
          </button>
          <button 
            className=\"btn-danger\"
            onClick={() => setActiveTab('delete')}
          >
            🗑️ Delete Data
          </button>
        </div>
      </div>

      <div className=\"privacy-guarantees\">
        <h3>Privacy Guarantees</h3>
        <ul>
          <li>
            {stats?.screenshotsStored === 0 || (stats?.lastScreenshotCleanup && 
              new Date(stats.lastScreenshotCleanup).getTime() > Date.now() - 30000) ? '✅' : '⏱️'} 
            All screenshots deleted after 30 seconds
            {stats?.lastScreenshotCleanup && 
              <span className=\"detail\"> (Last cleanup: {new Date(stats.lastScreenshotCleanup).toLocaleTimeString()})</span>
            }
          </li>
          <li>
            {stats?.localInferenceVerified ? '✅' : '🔍'} 
            ML inference runs 100% locally
          </li>
          <li>
            {stats?.piiAccuracy && stats.piiAccuracy >= 0.95 ? '✅' : '🔍'} 
            PII automatically detected and masked 
            <span className=\"detail\">({((stats?.piiAccuracy || 0) * 100).toFixed(1)}% accuracy)</span>
          </li>
          <li>
            {stats?.networkIsolationVerified ? '✅' : '🔍'} 
            Zero external network calls
          </li>
          <li>
            {stats?.encryptionEnabled ? '✅' : '❌'} 
            User-controlled data encryption
          </li>
          <li>✅ Complete audit trail</li>
        </ul>
      </div>
    </div>
  );

  const renderExportTab = () => (
    <div className=\"privacy-export\">
      <h3>Export Your Data</h3>
      <p>Download a complete copy of your data for portability or backup.</p>

      <div className=\"export-options\">
        <div className=\"option-group\">
          <label>Export Format:</label>
          <select 
            value={exportOptions.format}
            onChange={(e) => setExportOptions({
              ...exportOptions, 
              format: e.target.value as 'json' | 'csv' | 'xml'
            })}
          >
            <option value=\"json\">JSON (Recommended)</option>
            <option value=\"csv\">CSV (Spreadsheet)</option>
            <option value=\"xml\">XML</option>
          </select>
        </div>

        <div className=\"option-group\">
          <label>Date Range:</label>
          <select 
            value={exportOptions.dateRange}
            onChange={(e) => setExportOptions({
              ...exportOptions, 
              dateRange: e.target.value as 'today' | 'week' | 'month' | 'all'
            })}
          >
            <option value=\"today\">Today</option>
            <option value=\"week\">Last Week</option>
            <option value=\"month\">Last Month</option>
            <option value=\"all\">All Data</option>
          </select>
        </div>

        <div className=\"option-group\">
          <label>Include Data Types:</label>
          <div className=\"checkbox-group\">
            <label>
              <input 
                type=\"checkbox\"
                checked={exportOptions.includeScreenshots}
                onChange={(e) => setExportOptions({
                  ...exportOptions,
                  includeScreenshots: e.target.checked
                })}
              />
              Screenshots (Warning: Large file size)
            </label>
            <label>
              <input 
                type=\"checkbox\"
                checked={exportOptions.includeBehavioralData}
                onChange={(e) => setExportOptions({
                  ...exportOptions,
                  includeBehavioralData: e.target.checked
                })}
              />
              Behavioral Data (Keystroke/Mouse patterns)
            </label>
            <label>
              <input 
                type=\"checkbox\"
                checked={exportOptions.includeAuditLog}
                onChange={(e) => setExportOptions({
                  ...exportOptions,
                  includeAuditLog: e.target.checked
                })}
              />
              Privacy Audit Log
            </label>
          </div>
        </div>

        <div className=\"option-group\">
          <label>
            <input 
              type=\"checkbox\"
              checked={exportOptions.anonymize}
              onChange={(e) => setExportOptions({
                ...exportOptions,
                anonymize: e.target.checked
              })}
            />
            Anonymize Personal Data (Recommended)
          </label>
        </div>
      </div>

      <button 
        className=\"btn-primary export-btn\"
        onClick={handleExportData}
        disabled={loading}
      >
        {loading ? 'Exporting...' : '📦 Export Data'}
      </button>
    </div>
  );

  const renderDeleteTab = () => (
    <div className=\"privacy-delete\">
      <h3>Delete Your Data</h3>
      <p className=\"warning\">⚠️ Data deletion is permanent and cannot be undone.</p>

      <div className=\"delete-options\">
        <div className=\"option-group\">
          <label>Data Type to Delete:</label>
          <select 
            value={deletionOptions.dataType}
            onChange={(e) => setDeletionOptions({
              ...deletionOptions, 
              dataType: e.target.value as 'all' | 'screenshots' | 'behavioral' | 'audit_logs'
            })}
          >
            <option value=\"screenshots\">Screenshots Only</option>
            <option value=\"behavioral\">Behavioral Data Only</option>
            <option value=\"audit_logs\">Audit Logs Only</option>
            <option value=\"all\">All Data (Complete Reset)</option>
          </select>
        </div>

        <div className=\"option-group\">
          <label>Date Range:</label>
          <select 
            value={deletionOptions.dateRange}
            onChange={(e) => setDeletionOptions({
              ...deletionOptions, 
              dateRange: e.target.value as 'today' | 'week' | 'month' | 'all'
            })}
          >
            <option value=\"today\">Today Only</option>
            <option value=\"week\">Last Week</option>
            <option value=\"month\">Last Month</option>
            <option value=\"all\">All Time</option>
          </select>
        </div>

        <div className=\"option-group\">
          <label>
            <input 
              type=\"checkbox\"
              checked={deletionOptions.secureOverwrite}
              onChange={(e) => setDeletionOptions({
                ...deletionOptions,
                secureOverwrite: e.target.checked
              })}
            />
            Secure Overwrite (3-pass military-grade deletion)
          </label>
        </div>
      </div>

      <button 
        className=\"btn-danger delete-btn\"
        onClick={handleDeleteData}
        disabled={loading}
      >
        {loading ? 'Deleting...' : '🗑️ Delete Data'}
      </button>
    </div>
  );

  const renderAuditTab = () => (
    <div className=\"privacy-audit\">
      <h3>Privacy Audit Log</h3>
      <p>Complete transparency into all privacy operations.</p>

      <div className=\"audit-log\">
        {auditLog.length === 0 ? (
          <div className=\"no-data\">No audit entries found.</div>
        ) : (
          <div className=\"audit-entries\">
            {auditLog.map((entry, index) => (
              <div key={index} className={`audit-entry ${entry.success ? 'success' : 'failure'}`}>
                <div className=\"audit-header\">
                  <span className=\"audit-timestamp\">{entry.timestamp}</span>
                  <span className={`audit-status ${entry.success ? 'success' : 'failure'}`}>
                    {entry.success ? '✅' : '❌'}
                  </span>
                </div>
                <div className=\"audit-action\">{entry.action}</div>
                <div className=\"audit-details\">{entry.details}</div>
                <div className=\"audit-data\">Data: {entry.dataAffected}</div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );

  return (
    <div className=\"privacy-dashboard\">
      <div className=\"dashboard-header\">
        <h2>🔒 Privacy Dashboard</h2>
        <p>Complete control over your personal data</p>
      </div>

      <div className=\"dashboard-tabs\">
        <button 
          className={`tab ${activeTab === 'overview' ? 'active' : ''}`}
          onClick={() => setActiveTab('overview')}
        >
          Overview
        </button>
        <button 
          className={`tab ${activeTab === 'export' ? 'active' : ''}`}
          onClick={() => setActiveTab('export')}
        >
          Export Data
        </button>
        <button 
          className={`tab ${activeTab === 'delete' ? 'active' : ''}`}
          onClick={() => setActiveTab('delete')}
        >
          Delete Data
        </button>
        <button 
          className={`tab ${activeTab === 'audit' ? 'active' : ''}`}
          onClick={() => setActiveTab('audit')}
        >
          Audit Log
        </button>
      </div>

      <div className=\"dashboard-content\">
        {loading && <div className=\"loading-overlay\">Processing...</div>}
        
        {activeTab === 'overview' && renderOverviewTab()}
        {activeTab === 'export' && renderExportTab()}
        {activeTab === 'delete' && renderDeleteTab()}
        {activeTab === 'audit' && renderAuditTab()}
      </div>
    </div>
  );
};

export default PrivacyDashboard;