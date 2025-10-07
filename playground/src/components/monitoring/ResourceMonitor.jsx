import React, { useState, useEffect } from 'react';

/**
 * ResourceMonitor Component
 *
 * Displays resource utilization metrics:
 * - Browser Pool capacity
 * - Memory usage
 * - Queue depth
 * - Rate limiter statistics
 *
 * Auto-refreshes every 10 seconds
 */
const ResourceMonitor = () => {
  const [resources, setResources] = useState({
    browserPool: null,
    memory: null,
    rateLimiter: null,
    queueStats: null,
  });
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);
  const [lastUpdate, setLastUpdate] = useState(new Date());

  const fetchResourceData = async () => {
    try {
      setLoading(true);
      setError(null);

      // Fetch all resource endpoints in parallel
      const [browserPoolRes, memoryRes, rateLimiterRes, queueRes] = await Promise.all([
        fetch('/api/resources/browser-pool').catch(e => ({ ok: false, error: e.message })),
        fetch('/api/resources/memory').catch(e => ({ ok: false, error: e.message })),
        fetch('/api/resources/rate-limiter').catch(e => ({ ok: false, error: e.message })),
        fetch('/api/workers/stats/queue').catch(e => ({ ok: false, error: e.message })),
      ]);

      const data = {
        browserPool: browserPoolRes.ok ? await browserPoolRes.json() : null,
        memory: memoryRes.ok ? await memoryRes.json() : null,
        rateLimiter: rateLimiterRes.ok ? await rateLimiterRes.json() : null,
        queueStats: queueRes.ok ? await queueRes.json() : null,
      };

      setResources(data);
      setLastUpdate(new Date());
    } catch (err) {
      console.error('Failed to fetch resource data:', err);
      setError(err.message);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchResourceData();

    // Auto-refresh every 10 seconds
    const interval = setInterval(fetchResourceData, 10000);
    return () => clearInterval(interval);
  }, []);

  const formatBytes = (bytes) => {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return Math.round(bytes / Math.pow(k, i) * 100) / 100 + ' ' + sizes[i];
  };

  const getUtilizationColor = (percentage) => {
    if (percentage >= 90) return 'bg-red-500';
    if (percentage >= 70) return 'bg-yellow-500';
    return 'bg-green-500';
  };

  const calculatePercentage = (used, total) => {
    if (!total || total === 0) return 0;
    return Math.round((used / total) * 100);
  };

  if (loading && !resources.browserPool && !resources.memory) {
    return (
      <div className="bg-white rounded-lg shadow p-6 border border-gray-200">
        <h3 className="text-lg font-semibold text-gray-900 mb-4">Resources</h3>
        <div className="animate-pulse space-y-4">
          <div className="h-16 bg-gray-200 rounded"></div>
          <div className="h-16 bg-gray-200 rounded"></div>
          <div className="h-16 bg-gray-200 rounded"></div>
        </div>
      </div>
    );
  }

  return (
    <div className="bg-white rounded-lg shadow p-6 border border-gray-200">
      {/* Header */}
      <div className="flex items-center justify-between mb-4">
        <h3 className="text-lg font-semibold text-gray-900">Resources</h3>
        <button
          onClick={fetchResourceData}
          disabled={loading}
          className="text-xs text-blue-600 hover:text-blue-700 disabled:text-gray-400"
        >
          {loading ? 'Refreshing...' : '🔄 Refresh'}
        </button>
      </div>

      <div className="space-y-4">
        {/* Browser Pool */}
        {resources.browserPool && (
          <div className="p-4 bg-gray-50 rounded-lg">
            <div className="flex items-center justify-between mb-2">
              <span className="text-sm font-medium text-gray-700">Browser Pool</span>
              <span className="text-xs text-gray-500">
                {resources.browserPool.in_use || 0}/{resources.browserPool.capacity || 0}
              </span>
            </div>
            <div className="w-full bg-gray-200 rounded-full h-2.5 mb-2">
              <div
                className={`h-2.5 rounded-full ${getUtilizationColor(
                  calculatePercentage(
                    resources.browserPool.in_use,
                    resources.browserPool.capacity
                  )
                )}`}
                style={{
                  width: `${calculatePercentage(
                    resources.browserPool.in_use,
                    resources.browserPool.capacity
                  )}%`,
                }}
              ></div>
            </div>
            <div className="flex justify-between text-xs text-gray-600">
              <span>Available: {resources.browserPool.available || 0}</span>
              {resources.browserPool.waiting !== undefined && (
                <span>Waiting: {resources.browserPool.waiting}</span>
              )}
            </div>
          </div>
        )}

        {/* Memory Usage */}
        {resources.memory && (
          <div className="p-4 bg-gray-50 rounded-lg">
            <div className="flex items-center justify-between mb-2">
              <span className="text-sm font-medium text-gray-700">Memory</span>
              <span className="text-xs text-gray-500">
                {formatBytes(resources.memory.used || resources.memory.rss || 0)}
              </span>
            </div>
            {resources.memory.rss !== undefined && (
              <div className="grid grid-cols-2 gap-2 text-xs text-gray-600 mt-2">
                <div>RSS: {formatBytes(resources.memory.rss)}</div>
                {resources.memory.heap_used && (
                  <div>Heap: {formatBytes(resources.memory.heap_used)}</div>
                )}
              </div>
            )}
            {resources.memory.pressure && (
              <div className="mt-2 text-xs text-yellow-600">
                ⚠️ High memory pressure
              </div>
            )}
          </div>
        )}

        {/* Queue Depth */}
        {resources.queueStats && (
          <div className="p-4 bg-gray-50 rounded-lg">
            <div className="flex items-center justify-between mb-2">
              <span className="text-sm font-medium text-gray-700">Queue</span>
              <span className="text-xs text-gray-500">
                {resources.queueStats.pending || 0} pending
              </span>
            </div>
            <div className="grid grid-cols-2 gap-2 text-xs text-gray-600">
              <div>Running: {resources.queueStats.running || 0}</div>
              <div>Completed: {resources.queueStats.completed || 0}</div>
            </div>
            {resources.queueStats.failed !== undefined && (
              <div className="mt-2 text-xs text-red-600">
                Failed: {resources.queueStats.failed}
              </div>
            )}
          </div>
        )}

        {/* Rate Limiter */}
        {resources.rateLimiter && (
          <div className="p-4 bg-gray-50 rounded-lg">
            <div className="flex items-center justify-between mb-2">
              <span className="text-sm font-medium text-gray-700">Rate Limiter</span>
              <span className="text-xs text-gray-500">
                {resources.rateLimiter.enabled ? '✅ Enabled' : '⚠️ Disabled'}
              </span>
            </div>
            <div className="grid grid-cols-2 gap-2 text-xs text-gray-600">
              <div>Total: {resources.rateLimiter.total_requests || 0}</div>
              <div>Throttled: {resources.rateLimiter.throttled || 0}</div>
            </div>
            {resources.rateLimiter.throttle_rate !== undefined && (
              <div className="mt-2 text-xs text-gray-600">
                Rate: {(resources.rateLimiter.throttle_rate * 100).toFixed(1)}%
              </div>
            )}
          </div>
        )}
      </div>

      {/* Error State */}
      {error && (
        <div className="mt-4 p-3 bg-red-50 border border-red-200 rounded text-sm text-red-600">
          {error}
        </div>
      )}

      {/* Footer */}
      <div className="mt-4 pt-4 border-t border-gray-200">
        <span className="text-xs text-gray-500">
          Last updated: {lastUpdate.toLocaleTimeString()}
        </span>
      </div>
    </div>
  );
};

export default ResourceMonitor;
