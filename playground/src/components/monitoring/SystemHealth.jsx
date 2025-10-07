import React, { useState, useEffect } from 'react';

/**
 * SystemHealth Component
 *
 * Displays overall system health status with color-coded indicators
 * for each service dependency (Redis, WASM, HTTP Client, etc.)
 *
 * Auto-refreshes every 30 seconds
 */
const SystemHealth = () => {
  const [healthData, setHealthData] = useState(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);
  const [lastUpdate, setLastUpdate] = useState(new Date());

  const fetchHealthData = async () => {
    try {
      setLoading(true);
      setError(null);

      // Fetch detailed health status
      const response = await fetch('/api/health/detailed');

      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }

      const data = await response.json();
      setHealthData(data);
      setLastUpdate(new Date());
    } catch (err) {
      console.error('Failed to fetch health data:', err);
      setError(err.message);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchHealthData();

    // Auto-refresh every 30 seconds
    const interval = setInterval(fetchHealthData, 30000);
    return () => clearInterval(interval);
  }, []);

  const getStatusIcon = (status) => {
    if (!status || status === 'unknown') return '❓';

    const normalized = status.toLowerCase();
    if (normalized === 'healthy' || normalized === 'ready' || normalized === 'ok') return '✅';
    if (normalized === 'degraded' || normalized === 'warning') return '⚠️';
    if (normalized === 'unhealthy' || normalized === 'error' || normalized === 'failed') return '❌';
    return '❓';
  };

  const getOverallStatus = () => {
    if (!healthData || !healthData.dependencies) return { icon: '❓', text: 'Unknown' };

    const statuses = Object.values(healthData.dependencies).map(dep =>
      dep.status ? dep.status.toLowerCase() : 'unknown'
    );

    const hasUnhealthy = statuses.some(s => s === 'unhealthy' || s === 'error' || s === 'failed');
    const hasDegraded = statuses.some(s => s === 'degraded' || s === 'warning');

    if (hasUnhealthy) return { icon: '❌', text: 'Systems Down' };
    if (hasDegraded) return { icon: '⚠️', text: 'Degraded' };
    return { icon: '✅', text: 'All Systems Go' };
  };

  if (loading && !healthData) {
    return (
      <div className="bg-white rounded-lg shadow p-6 border border-gray-200">
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-lg font-semibold text-gray-900">System Health</h3>
          <span className="text-sm text-gray-500">Loading...</span>
        </div>
        <div className="animate-pulse space-y-3">
          <div className="h-8 bg-gray-200 rounded"></div>
          <div className="h-8 bg-gray-200 rounded"></div>
          <div className="h-8 bg-gray-200 rounded"></div>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="bg-white rounded-lg shadow p-6 border border-red-200">
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-lg font-semibold text-gray-900">System Health</h3>
          <span className="text-sm text-red-600">❌ Error</span>
        </div>
        <div className="text-sm text-red-600 mb-4">{error}</div>
        <button
          onClick={fetchHealthData}
          className="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 text-sm"
        >
          Retry
        </button>
      </div>
    );
  }

  const overallStatus = getOverallStatus();

  return (
    <div className="bg-white rounded-lg shadow p-6 border border-gray-200">
      {/* Header */}
      <div className="flex items-center justify-between mb-4">
        <h3 className="text-lg font-semibold text-gray-900">System Health</h3>
        <div className="flex items-center gap-2">
          <span className="text-2xl">{overallStatus.icon}</span>
          <span className="text-sm font-medium text-gray-700">{overallStatus.text}</span>
        </div>
      </div>

      {/* Dependencies */}
      {healthData?.dependencies && (
        <div className="space-y-2">
          {Object.entries(healthData.dependencies).map(([name, info]) => (
            <div
              key={name}
              className="flex items-center justify-between py-2 px-3 bg-gray-50 rounded hover:bg-gray-100 transition-colors"
            >
              <div className="flex items-center gap-3">
                <span className="text-xl">{getStatusIcon(info.status)}</span>
                <span className="text-sm font-medium text-gray-700 capitalize">
                  {name.replace(/_/g, ' ')}
                </span>
              </div>
              <div className="text-right">
                {info.response_time_ms !== undefined && (
                  <span className="text-xs text-gray-500">
                    {info.response_time_ms}ms
                  </span>
                )}
                {info.message && (
                  <div className="text-xs text-gray-600 mt-1">
                    {info.message}
                  </div>
                )}
                {info.details && typeof info.details === 'string' && (
                  <div className="text-xs text-gray-500 mt-1">
                    {info.details}
                  </div>
                )}
                {info.connections !== undefined && (
                  <span className="text-xs text-gray-500">
                    {info.connections} connections
                  </span>
                )}
                {info.active_count !== undefined && (
                  <span className="text-xs text-gray-500">
                    {info.active_count}/{info.total_count || '?'} active
                  </span>
                )}
              </div>
            </div>
          ))}
        </div>
      )}

      {/* Footer */}
      <div className="mt-4 pt-4 border-t border-gray-200 flex items-center justify-between">
        <span className="text-xs text-gray-500">
          Last updated: {lastUpdate.toLocaleTimeString()}
        </span>
        <button
          onClick={fetchHealthData}
          disabled={loading}
          className="text-xs text-blue-600 hover:text-blue-700 disabled:text-gray-400"
        >
          {loading ? 'Refreshing...' : '🔄 Refresh'}
        </button>
      </div>
    </div>
  );
};

export default SystemHealth;
