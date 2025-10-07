import { useState, useEffect, useRef } from 'react'
import { FaSpinner, FaCheckCircle, FaTimesCircle, FaChevronDown, FaChevronUp, FaTimes } from 'react-icons/fa'

export default function LiveProgressWidget({ crawlId, onClose }) {
  const [isCollapsed, setIsCollapsed] = useState(false)
  const [progress, setProgress] = useState({
    total: 0,
    completed: 0,
    succeeded: 0,
    failed: 0,
    currentUrl: '',
    status: 'running',
    startTime: Date.now(),
    errors: []
  })
  const eventSourceRef = useRef(null)

  useEffect(() => {
    // Connect to SSE endpoint for live updates
    const connectToStream = () => {
      try {
        // For demo purposes, we'll simulate progress
        // In production, this would connect to /crawl/sse endpoint

        // Simulated progress updates
        const interval = setInterval(() => {
          setProgress(prev => {
            if (prev.completed >= prev.total && prev.total > 0) {
              clearInterval(interval)
              return { ...prev, status: 'completed' }
            }

            const newCompleted = Math.min(prev.completed + 1, prev.total || 50)
            const newSucceeded = prev.succeeded + (Math.random() > 0.1 ? 1 : 0)
            const newFailed = newCompleted - newSucceeded

            return {
              ...prev,
              total: prev.total || 50,
              completed: newCompleted,
              succeeded: newSucceeded,
              failed: newFailed,
              currentUrl: `https://example.com/page-${newCompleted}`
            }
          })
        }, 1000)

        return () => clearInterval(interval)
      } catch (error) {
        console.error('Failed to connect to progress stream:', error)
      }
    }

    const cleanup = connectToStream()
    return cleanup
  }, [crawlId])

  const getProgressPercentage = () => {
    if (!progress.total) return 0
    return Math.round((progress.completed / progress.total) * 100)
  }

  const getElapsedTime = () => {
    const elapsed = Date.now() - progress.startTime
    const seconds = Math.floor(elapsed / 1000)
    const minutes = Math.floor(seconds / 60)
    if (minutes > 0) {
      return `${minutes}m ${seconds % 60}s`
    }
    return `${seconds}s`
  }

  const getETA = () => {
    if (progress.completed === 0) return 'Calculating...'
    if (progress.completed >= progress.total) return 'Complete'

    const elapsed = Date.now() - progress.startTime
    const rate = progress.completed / (elapsed / 1000)
    const remaining = progress.total - progress.completed
    const etaSeconds = Math.round(remaining / rate)

    const minutes = Math.floor(etaSeconds / 60)
    if (minutes > 0) {
      return `${minutes}m ${etaSeconds % 60}s`
    }
    return `${etaSeconds}s`
  }

  const getCrawlRate = () => {
    const elapsed = (Date.now() - progress.startTime) / 1000
    if (elapsed < 1) return '0.0'
    return (progress.completed / elapsed).toFixed(1)
  }

  const getSuccessRate = () => {
    if (progress.completed === 0) return 100
    return Math.round((progress.succeeded / progress.completed) * 100)
  }

  if (isCollapsed) {
    return (
      <div className="fixed bottom-4 right-4 bg-white rounded-lg shadow-lg border border-gray-200 p-3 max-w-sm z-50">
        <div className="flex items-center justify-between">
          <div className="flex items-center space-x-2">
            <FaSpinner className="animate-spin text-riptide-blue" />
            <span className="font-medium text-gray-800">
              Crawling... {getProgressPercentage()}%
            </span>
          </div>
          <div className="flex items-center space-x-2">
            <button
              onClick={() => setIsCollapsed(false)}
              className="p-1 text-gray-600 hover:text-gray-800 transition-colors"
              title="Expand"
            >
              <FaChevronUp />
            </button>
            <button
              onClick={onClose}
              className="p-1 text-gray-600 hover:text-red-600 transition-colors"
              title="Close"
            >
              <FaTimes />
            </button>
          </div>
        </div>
      </div>
    )
  }

  return (
    <div className="fixed bottom-4 right-4 bg-white rounded-lg shadow-xl border border-gray-300 max-w-md w-full z-50">
      {/* Header */}
      <div className="flex items-center justify-between p-4 border-b border-gray-200 bg-gradient-to-r from-riptide-blue to-blue-600 text-white rounded-t-lg">
        <div className="flex items-center space-x-2">
          {progress.status === 'running' ? (
            <>
              <FaSpinner className="animate-spin" />
              <h3 className="font-semibold">Crawling in Progress...</h3>
            </>
          ) : progress.status === 'completed' ? (
            <>
              <FaCheckCircle />
              <h3 className="font-semibold">Crawl Completed</h3>
            </>
          ) : (
            <>
              <FaTimesCircle />
              <h3 className="font-semibold">Crawl Failed</h3>
            </>
          )}
        </div>
        <div className="flex items-center space-x-2">
          <button
            onClick={() => setIsCollapsed(true)}
            className="p-1 hover:bg-white hover:bg-opacity-20 rounded transition-colors"
            title="Collapse"
          >
            <FaChevronDown />
          </button>
          <button
            onClick={onClose}
            className="p-1 hover:bg-white hover:bg-opacity-20 rounded transition-colors"
            title="Close"
          >
            <FaTimes />
          </button>
        </div>
      </div>

      {/* Progress Bar */}
      <div className="p-4">
        <div className="mb-2 flex items-center justify-between text-sm">
          <span className="font-medium text-gray-700">
            Progress: {progress.completed}/{progress.total} URLs
          </span>
          <span className="font-bold text-riptide-blue">
            {getProgressPercentage()}%
          </span>
        </div>
        <div className="w-full bg-gray-200 rounded-full h-3 mb-4">
          <div
            className="bg-gradient-to-r from-riptide-blue to-blue-600 h-3 rounded-full transition-all duration-500"
            style={{ width: `${getProgressPercentage()}%` }}
          />
        </div>

        {/* Statistics Grid */}
        <div className="grid grid-cols-3 gap-3 mb-4">
          <div className="bg-green-50 rounded-lg p-3 text-center border border-green-200">
            <div className="flex items-center justify-center space-x-1 mb-1">
              <FaCheckCircle className="text-green-600 text-sm" />
              <span className="text-xs text-green-600 font-medium">Success</span>
            </div>
            <p className="text-xl font-bold text-green-700">{progress.succeeded}</p>
            <p className="text-xs text-green-600">{getSuccessRate()}%</p>
          </div>

          <div className="bg-red-50 rounded-lg p-3 text-center border border-red-200">
            <div className="flex items-center justify-center space-x-1 mb-1">
              <FaTimesCircle className="text-red-600 text-sm" />
              <span className="text-xs text-red-600 font-medium">Failed</span>
            </div>
            <p className="text-xl font-bold text-red-700">{progress.failed}</p>
            <p className="text-xs text-red-600">{100 - getSuccessRate()}%</p>
          </div>

          <div className="bg-blue-50 rounded-lg p-3 text-center border border-blue-200">
            <div className="flex items-center justify-center space-x-1 mb-1">
              <FaSpinner className="text-blue-600 text-sm" />
              <span className="text-xs text-blue-600 font-medium">Rate</span>
            </div>
            <p className="text-xl font-bold text-blue-700">{getCrawlRate()}</p>
            <p className="text-xs text-blue-600">URLs/sec</p>
          </div>
        </div>

        {/* Timing Information */}
        <div className="grid grid-cols-2 gap-3 mb-4">
          <div className="bg-gray-50 rounded-lg p-2 border border-gray-200">
            <p className="text-xs text-gray-600">Elapsed Time</p>
            <p className="text-sm font-semibold text-gray-800">{getElapsedTime()}</p>
          </div>
          <div className="bg-gray-50 rounded-lg p-2 border border-gray-200">
            <p className="text-xs text-gray-600">ETA Remaining</p>
            <p className="text-sm font-semibold text-gray-800">{getETA()}</p>
          </div>
        </div>

        {/* Current URL */}
        {progress.currentUrl && (
          <div className="bg-blue-50 border border-blue-200 rounded-lg p-3">
            <p className="text-xs text-blue-600 font-medium mb-1">Current URL:</p>
            <p className="text-sm text-blue-900 truncate" title={progress.currentUrl}>
              {progress.currentUrl}
            </p>
          </div>
        )}

        {/* Errors List */}
        {progress.errors.length > 0 && (
          <div className="mt-3 bg-red-50 border border-red-200 rounded-lg p-3 max-h-32 overflow-y-auto">
            <p className="text-xs text-red-600 font-medium mb-2">Recent Errors:</p>
            <ul className="space-y-1">
              {progress.errors.slice(-5).map((error, idx) => (
                <li key={idx} className="text-xs text-red-800">
                  • {error}
                </li>
              ))}
            </ul>
          </div>
        )}
      </div>
    </div>
  )
}
