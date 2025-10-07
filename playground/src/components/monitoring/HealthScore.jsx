import React, { useState, useEffect } from 'react';

/**
 * HealthScore Component
 *
 * Displays overall system health score (0-100) with visual indicator
 * Color-coded: Green (80-100), Yellow (50-79), Red (0-49)
 *
 * Auto-refreshes every 15 seconds
 */
const HealthScore = () => {
  const [score, setScore] = useState(null);
  const [status, setStatus] = useState(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);
  const [lastUpdate, setLastUpdate] = useState(new Date());

  const fetchHealthScore = async () => {
    try {
      setLoading(true);
      setError(null);

      const response = await fetch('/api/monitoring/health-score');

      if (!response.ok) {
        // Fallback: calculate from detailed health
        const detailedResponse = await fetch('/api/health/detailed');
        if (detailedResponse.ok) {
          const data = await detailedResponse.json();
          const calculatedScore = calculateHealthScore(data);
          setScore(calculatedScore);
          setStatus(getHealthStatus(calculatedScore));
          setLastUpdate(new Date());
          setLoading(false);
          return;
        }
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }

      const data = await response.json();
      setScore(data.score || data.health_score || 0);
      setStatus(data.status || getHealthStatus(data.score || data.health_score));
      setLastUpdate(new Date());
    } catch (err) {
      console.error('Failed to fetch health score:', err);
      setError(err.message);
    } finally {
      setLoading(false);
    }
  };

  // Calculate health score from detailed health data
  const calculateHealthScore = (healthData) => {
    if (!healthData || !healthData.dependencies) return 0;

    const deps = Object.values(healthData.dependencies);
    if (deps.length === 0) return 100;

    const healthyCount = deps.filter(dep => {
      const status = dep.status ? dep.status.toLowerCase() : '';
      return status === 'healthy' || status === 'ready' || status === 'ok';
    }).length;

    return Math.round((healthyCount / deps.length) * 100);
  };

  const getHealthStatus = (scoreValue) => {
    if (scoreValue >= 80) return 'excellent';
    if (scoreValue >= 50) return 'good';
    if (scoreValue >= 30) return 'degraded';
    return 'critical';
  };

  const getScoreColor = (scoreValue) => {
    if (scoreValue >= 80) return 'text-green-600';
    if (scoreValue >= 50) return 'text-yellow-600';
    return 'text-red-600';
  };

  const getScoreIcon = (scoreValue) => {
    if (scoreValue >= 80) return '🟢';
    if (scoreValue >= 50) return '🟡';
    return '🔴';
  };

  const getStatusText = (statusValue) => {
    const statuses = {
      excellent: 'Excellent',
      good: 'Good',
      degraded: 'Degraded',
      critical: 'Critical',
    };
    return statuses[statusValue] || 'Unknown';
  };

  useEffect(() => {
    fetchHealthScore();

    // Auto-refresh every 15 seconds
    const interval = setInterval(fetchHealthScore, 15000);
    return () => clearInterval(interval);
  }, []);

  if (loading && score === null) {
    return (
      <div className="bg-white rounded-lg shadow p-6 border border-gray-200">
        <h3 className="text-lg font-semibold text-gray-900 mb-4 text-center">
          Health Score
        </h3>
        <div className="flex flex-col items-center justify-center py-8">
          <div className="animate-pulse">
            <div className="h-24 w-24 bg-gray-200 rounded-full"></div>
          </div>
          <p className="text-sm text-gray-500 mt-4">Loading...</p>
        </div>
      </div>
    );
  }

  if (error && score === null) {
    return (
      <div className="bg-white rounded-lg shadow p-6 border border-red-200">
        <h3 className="text-lg font-semibold text-gray-900 mb-4 text-center">
          Health Score
        </h3>
        <div className="flex flex-col items-center justify-center py-8">
          <span className="text-6xl mb-4">❌</span>
          <p className="text-sm text-red-600 mb-4">{error}</p>
          <button
            onClick={fetchHealthScore}
            className="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 text-sm"
          >
            Retry
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className="bg-white rounded-lg shadow p-6 border border-gray-200">
      {/* Header */}
      <div className="flex items-center justify-between mb-4">
        <h3 className="text-lg font-semibold text-gray-900">Health Score</h3>
        <button
          onClick={fetchHealthScore}
          disabled={loading}
          className="text-xs text-blue-600 hover:text-blue-700 disabled:text-gray-400"
        >
          {loading ? 'Refreshing...' : '🔄 Refresh'}
        </button>
      </div>

      {/* Score Display */}
      <div className="flex flex-col items-center justify-center py-6">
        <div className="relative">
          <div className="text-6xl mb-2">{getScoreIcon(score)}</div>
          <div className={`text-6xl font-bold ${getScoreColor(score)}`}>
            {score}
          </div>
        </div>
        <div className="text-lg font-medium text-gray-600 mt-3">
          {getStatusText(status)}
        </div>
      </div>

      {/* Details */}
      {status === 'degraded' && (
        <div className="mt-4 p-3 bg-yellow-50 border border-yellow-200 rounded">
          <p className="text-sm text-yellow-800">
            ⚠️ Some services are experiencing issues
          </p>
        </div>
      )}

      {status === 'critical' && (
        <div className="mt-4 p-3 bg-red-50 border border-red-200 rounded">
          <p className="text-sm text-red-800">
            🚨 Multiple services are down - immediate attention required
          </p>
        </div>
      )}

      {/* Footer */}
      <div className="mt-4 pt-4 border-t border-gray-200 text-center">
        <span className="text-xs text-gray-500">
          Last updated: {lastUpdate.toLocaleTimeString()}
        </span>
      </div>
    </div>
  );
};

export default HealthScore;
