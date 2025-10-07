import { useState, useEffect } from 'react'
import axios from 'axios'
import { FaHeartbeat, FaArrowUp, FaArrowDown, FaMinus, FaSync } from 'react-icons/fa'

export default function HealthScoreCard() {
  const [healthData, setHealthData] = useState(null)
  const [isLoading, setIsLoading] = useState(true)
  const [error, setError] = useState(null)
  const [lastScore, setLastScore] = useState(null)
  const [autoRefresh, setAutoRefresh] = useState(true)

  const fetchHealthScore = async () => {
    try {
      setIsLoading(true)
      setError(null)
      const response = await axios.get('/api/monitoring/health-score')

      // Store previous score for trend calculation
      if (healthData) {
        setLastScore(healthData.score)
      }

      setHealthData(response.data)
    } catch (err) {
      console.error('Failed to fetch health score:', err)
      setError(err.response?.data?.error || 'Failed to load health score')
    } finally {
      setIsLoading(false)
    }
  }

  useEffect(() => {
    fetchHealthScore()
  }, [])

  // Auto-refresh every 30 seconds
  useEffect(() => {
    if (!autoRefresh) return

    const interval = setInterval(() => {
      fetchHealthScore()
    }, 30000)

    return () => clearInterval(interval)
  }, [autoRefresh])

  const getHealthStatus = (score) => {
    if (score >= 95) return { label: 'Excellent', color: 'text-green-600', bg: 'bg-green-50', border: 'border-green-200' }
    if (score >= 85) return { label: 'Good', color: 'text-blue-600', bg: 'bg-blue-50', border: 'border-blue-200' }
    if (score >= 70) return { label: 'Fair', color: 'text-yellow-600', bg: 'bg-yellow-50', border: 'border-yellow-200' }
    if (score >= 50) return { label: 'Poor', color: 'text-orange-600', bg: 'bg-orange-50', border: 'border-orange-200' }
    return { label: 'Critical', color: 'text-red-600', bg: 'bg-red-50', border: 'border-red-200' }
  }

  const getTrendIndicator = () => {
    if (!lastScore || !healthData) return null

    const diff = healthData.score - lastScore

    if (Math.abs(diff) < 2) {
      return {
        icon: <FaMinus className="text-gray-400" />,
        label: 'Stable',
        color: 'text-gray-600'
      }
    }

    if (diff > 0) {
      return {
        icon: <FaArrowUp className="text-green-500" />,
        label: `+${diff.toFixed(1)}`,
        color: 'text-green-600'
      }
    }

    return {
      icon: <FaArrowDown className="text-red-500" />,
      label: `${diff.toFixed(1)}`,
      color: 'text-red-600'
    }
  }

  const getScoreColor = (score) => {
    if (score >= 95) return 'text-green-600'
    if (score >= 85) return 'text-blue-600'
    if (score >= 70) return 'text-yellow-600'
    if (score >= 50) return 'text-orange-600'
    return 'text-red-600'
  }

  const getProgressColor = (score) => {
    if (score >= 95) return 'bg-green-500'
    if (score >= 85) return 'bg-blue-500'
    if (score >= 70) return 'bg-yellow-500'
    if (score >= 50) return 'bg-orange-500'
    return 'bg-red-500'
  }

  if (isLoading && !healthData) {
    return (
      <div className="bg-white rounded-xl shadow-sm border border-gray-200 p-6">
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-lg font-semibold text-gray-900 flex items-center">
            <FaHeartbeat className="mr-2 text-riptide-blue" />
            System Health Score
          </h3>
        </div>
        <div className="flex items-center justify-center h-40">
          <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-riptide-blue"></div>
        </div>
      </div>
    )
  }

  if (error) {
    return (
      <div className="bg-white rounded-xl shadow-sm border border-gray-200 p-6">
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-lg font-semibold text-gray-900 flex items-center">
            <FaHeartbeat className="mr-2 text-riptide-blue" />
            System Health Score
          </h3>
        </div>
        <div className="bg-red-50 border border-red-200 rounded-lg p-4 text-sm text-red-700">
          {error}
        </div>
      </div>
    )
  }

  const status = getHealthStatus(healthData.score)
  const trend = getTrendIndicator()

  return (
    <div className={`bg-white rounded-xl shadow-sm border ${status.border} p-6`}>
      {/* Header */}
      <div className="flex items-center justify-between mb-6">
        <h3 className="text-lg font-semibold text-gray-900 flex items-center">
          <FaHeartbeat className={`mr-2 ${status.color}`} />
          System Health Score
        </h3>
        <div className="flex items-center space-x-2">
          <label className="flex items-center text-xs text-gray-600">
            <input
              type="checkbox"
              checked={autoRefresh}
              onChange={(e) => setAutoRefresh(e.target.checked)}
              className="mr-1"
            />
            Auto-refresh
          </label>
          <button
            onClick={fetchHealthScore}
            disabled={isLoading}
            className="p-2 hover:bg-gray-100 rounded-lg transition-colors disabled:opacity-50"
            title="Refresh"
          >
            <FaSync className={`text-gray-600 ${isLoading ? 'animate-spin' : ''}`} />
          </button>
        </div>
      </div>

      {/* Score Display */}
      <div className="flex items-center justify-between mb-6">
        <div className="flex items-center space-x-4">
          {/* Large Score Number */}
          <div className="flex items-baseline">
            <span className={`text-6xl font-bold ${getScoreColor(healthData.score)}`}>
              {healthData.score.toFixed(0)}
            </span>
            <span className="text-2xl text-gray-400 ml-1">/100</span>
          </div>

          {/* Status Badge */}
          <div className={`px-4 py-2 rounded-full ${status.bg} ${status.color} font-semibold text-sm`}>
            {status.label}
          </div>
        </div>

        {/* Trend Indicator */}
        {trend && (
          <div className="flex flex-col items-end">
            <div className="flex items-center space-x-1">
              {trend.icon}
              <span className={`text-sm font-medium ${trend.color}`}>
                {trend.label}
              </span>
            </div>
            <span className="text-xs text-gray-500">vs last check</span>
          </div>
        )}
      </div>

      {/* Progress Bar */}
      <div className="mb-6">
        <div className="w-full bg-gray-200 rounded-full h-3">
          <div
            className={`h-3 rounded-full transition-all duration-500 ${getProgressColor(healthData.score)}`}
            style={{ width: `${healthData.score}%` }}
          />
        </div>
      </div>

      {/* Component Breakdown */}
      {healthData.components && Object.keys(healthData.components).length > 0 && (
        <div className="space-y-2">
          <h4 className="text-xs font-semibold text-gray-700 uppercase mb-3">
            Component Status
          </h4>
          <div className="grid grid-cols-2 gap-3">
            {Object.entries(healthData.components).map(([component, componentScore]) => {
              const componentStatus = getHealthStatus(componentScore)
              return (
                <div
                  key={component}
                  className={`p-3 rounded-lg ${componentStatus.bg} border ${componentStatus.border}`}
                >
                  <div className="flex items-center justify-between">
                    <span className="text-sm font-medium text-gray-700 capitalize">
                      {component.replace('_', ' ')}
                    </span>
                    <span className={`text-sm font-bold ${componentStatus.color}`}>
                      {componentScore.toFixed(0)}
                    </span>
                  </div>
                </div>
              )
            })}
          </div>
        </div>
      )}

      {/* Last Updated */}
      {healthData.timestamp && (
        <div className="mt-4 pt-4 border-t border-gray-200">
          <p className="text-xs text-gray-500 text-center">
            Last updated: {new Date(healthData.timestamp).toLocaleTimeString()}
          </p>
        </div>
      )}
    </div>
  )
}
