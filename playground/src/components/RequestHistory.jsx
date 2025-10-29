import { FaHistory, FaClock, FaCheck, FaTimes, FaTrash } from 'react-icons/fa'
import { usePlaygroundStore } from '../hooks/usePlaygroundStore'

/**
 * Request History Component
 * Shows recent API requests with ability to replay them
 */
export default function RequestHistory() {
  const { requestHistory, loadFromHistory, clearHistory } = usePlaygroundStore()

  if (!requestHistory || requestHistory.length === 0) {
    return (
      <div className="card">
        <h3 className="text-lg font-semibold mb-3 flex items-center">
          <FaHistory className="mr-2 text-gray-400" />
          Request History
        </h3>
        <div className="text-center py-6 text-gray-500 text-sm">
          No requests yet. Execute a request to see it here.
        </div>
      </div>
    )
  }

  return (
    <div className="card">
      <div className="flex items-center justify-between mb-3">
        <h3 className="text-lg font-semibold flex items-center">
          <FaHistory className="mr-2 text-riptide-blue" />
          Request History
        </h3>
        <button
          onClick={clearHistory}
          className="text-xs text-gray-500 hover:text-red-600 transition-colors flex items-center"
          title="Clear history"
        >
          <FaTrash className="mr-1" />
          Clear
        </button>
      </div>

      <div className="space-y-2 max-h-64 overflow-y-auto">
        {requestHistory.map((item, index) => (
          <button
            key={item.timestamp}
            onClick={() => loadFromHistory(item)}
            className="w-full text-left p-3 border border-gray-200 rounded-lg hover:border-riptide-blue hover:bg-blue-50 transition-all group"
          >
            <div className="flex items-start justify-between mb-1">
              <div className="flex items-center space-x-2">
                <span className={`px-2 py-0.5 text-xs font-mono rounded ${
                  item.method === 'GET' ? 'bg-green-100 text-green-700' :
                  item.method === 'POST' ? 'bg-blue-100 text-blue-700' :
                  item.method === 'DELETE' ? 'bg-red-100 text-red-700' :
                  'bg-gray-100 text-gray-700'
                }`}>
                  {item.method}
                </span>
                <span className="text-sm font-medium text-gray-800 truncate max-w-[200px]">
                  {item.endpoint.name}
                </span>
              </div>

              <div className="flex items-center space-x-2">
                {item.response?.status && (
                  <span className={`flex items-center text-xs ${
                    item.response.status >= 200 && item.response.status < 300
                      ? 'text-green-600'
                      : 'text-red-600'
                  }`}>
                    {item.response.status >= 200 && item.response.status < 300 ? (
                      <FaCheck className="mr-1" />
                    ) : (
                      <FaTimes className="mr-1" />
                    )}
                    {item.response.status}
                  </span>
                )}
              </div>
            </div>

            <div className="flex items-center justify-between text-xs text-gray-500">
              <div className="flex items-center">
                <FaClock className="mr-1" />
                {new Date(item.timestamp).toLocaleTimeString()}
              </div>
              {item.response?.duration && (
                <span>{item.response.duration}ms</span>
              )}
            </div>
          </button>
        ))}
      </div>
    </div>
  )
}
