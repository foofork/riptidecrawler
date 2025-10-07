import { FaServer, FaClock, FaCheckCircle, FaChartLine } from 'react-icons/fa'

export default function WorkerStats({ stats }) {
  const { queue, workers, metrics } = stats

  return (
    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
      {/* Queue Depth */}
      <div className="card bg-gradient-to-br from-blue-50 to-blue-100 border-blue-200">
        <div className="flex items-center justify-between">
          <div>
            <p className="text-sm text-blue-600 font-medium">Queue Depth</p>
            <p className="text-3xl font-bold text-blue-900">
              {queue?.pending || 0}
            </p>
            <p className="text-xs text-blue-600 mt-1">
              {queue?.total || 0} total jobs
            </p>
          </div>
          <FaClock className="text-4xl text-blue-400" />
        </div>
      </div>

      {/* Active Workers */}
      <div className="card bg-gradient-to-br from-green-50 to-green-100 border-green-200">
        <div className="flex items-center justify-between">
          <div>
            <p className="text-sm text-green-600 font-medium">Active Workers</p>
            <p className="text-3xl font-bold text-green-900">
              {workers?.active || 0}
            </p>
            <p className="text-xs text-green-600 mt-1">
              {workers?.total || 0} total workers
            </p>
          </div>
          <FaServer className="text-4xl text-green-400" />
        </div>
      </div>

      {/* Completed Jobs */}
      <div className="card bg-gradient-to-br from-purple-50 to-purple-100 border-purple-200">
        <div className="flex items-center justify-between">
          <div>
            <p className="text-sm text-purple-600 font-medium">Completed</p>
            <p className="text-3xl font-bold text-purple-900">
              {metrics?.completed_count || 0}
            </p>
            <p className="text-xs text-purple-600 mt-1">
              Last hour
            </p>
          </div>
          <FaCheckCircle className="text-4xl text-purple-400" />
        </div>
      </div>

      {/* Throughput */}
      <div className="card bg-gradient-to-br from-orange-50 to-orange-100 border-orange-200">
        <div className="flex items-center justify-between">
          <div>
            <p className="text-sm text-orange-600 font-medium">Throughput</p>
            <p className="text-3xl font-bold text-orange-900">
              {metrics?.jobs_per_second?.toFixed(2) || '0.00'}
            </p>
            <p className="text-xs text-orange-600 mt-1">
              jobs/second
            </p>
          </div>
          <FaChartLine className="text-4xl text-orange-400" />
        </div>
      </div>
    </div>
  )
}
