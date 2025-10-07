import { useState, useEffect } from 'react'
import { FaPlay, FaSpinner, FaCheckCircle, FaTimesCircle, FaClock, FaSync } from 'react-icons/fa'
import axios from 'axios'
import JobSubmitForm from '../components/workers/JobSubmitForm'
import JobsList from '../components/workers/JobsList'
import WorkerStats from '../components/workers/WorkerStats'

export default function Workers() {
  const [jobs, setJobs] = useState([])
  const [stats, setStats] = useState(null)
  const [isLoading, setIsLoading] = useState(false)
  const [activeTab, setActiveTab] = useState('all')
  const [refreshInterval, setRefreshInterval] = useState(5000)
  const [autoRefresh, setAutoRefresh] = useState(true)

  const fetchJobs = async () => {
    try {
      setIsLoading(true)
      const response = await axios.get('/api/workers/jobs')
      setJobs(response.data.jobs || [])
    } catch (error) {
      console.error('Failed to fetch jobs:', error)
    } finally {
      setIsLoading(false)
    }
  }

  const fetchStats = async () => {
    try {
      const [queueStats, workerStats, metrics] = await Promise.all([
        axios.get('/api/workers/stats/queue'),
        axios.get('/api/workers/stats/workers'),
        axios.get('/api/workers/metrics')
      ])

      setStats({
        queue: queueStats.data,
        workers: workerStats.data,
        metrics: metrics.data
      })
    } catch (error) {
      console.error('Failed to fetch stats:', error)
    }
  }

  useEffect(() => {
    fetchJobs()
    fetchStats()
  }, [])

  useEffect(() => {
    if (!autoRefresh) return

    const interval = setInterval(() => {
      fetchJobs()
      fetchStats()
    }, refreshInterval)

    return () => clearInterval(interval)
  }, [autoRefresh, refreshInterval])

  const handleJobSubmitted = () => {
    fetchJobs()
    fetchStats()
  }

  const filterJobs = (jobs) => {
    if (activeTab === 'all') return jobs

    return jobs.filter(job => {
      if (activeTab === 'active') return job.status === 'running' || job.status === 'pending'
      if (activeTab === 'completed') return job.status === 'completed'
      if (activeTab === 'failed') return job.status === 'failed'
      return true
    })
  }

  const filteredJobs = filterJobs(jobs)

  const getStatusCounts = () => {
    return {
      all: jobs.length,
      active: jobs.filter(j => j.status === 'running' || j.status === 'pending').length,
      completed: jobs.filter(j => j.status === 'completed').length,
      failed: jobs.filter(j => j.status === 'failed').length
    }
  }

  const counts = getStatusCounts()

  return (
    <div className="container mx-auto px-4 py-8">
      <div className="mb-6">
        <div className="flex items-center justify-between">
          <div>
            <h2 className="text-3xl font-bold text-gray-800 mb-2">Workers & Tasks</h2>
            <p className="text-gray-600">
              Monitor and manage background jobs and worker queue
            </p>
          </div>

          <div className="flex items-center space-x-4">
            <div className="flex items-center space-x-2">
              <label className="text-sm text-gray-600">Auto-refresh</label>
              <button
                onClick={() => setAutoRefresh(!autoRefresh)}
                className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors ${
                  autoRefresh ? 'bg-riptide-blue' : 'bg-gray-300'
                }`}
              >
                <span
                  className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${
                    autoRefresh ? 'translate-x-6' : 'translate-x-1'
                  }`}
                />
              </button>
            </div>

            <button
              onClick={() => {
                fetchJobs()
                fetchStats()
              }}
              disabled={isLoading}
              className="btn-secondary flex items-center space-x-2"
            >
              <FaSync className={isLoading ? 'animate-spin' : ''} />
              <span>Refresh</span>
            </button>
          </div>
        </div>
      </div>

      {/* Worker Stats */}
      {stats && <WorkerStats stats={stats} />}

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6 mt-6">
        {/* Job Submission Form */}
        <div className="lg:col-span-1">
          <div className="card sticky top-4">
            <h3 className="text-xl font-semibold mb-4 flex items-center">
              <FaPlay className="text-riptide-blue mr-2" />
              Submit Job
            </h3>
            <JobSubmitForm onJobSubmitted={handleJobSubmitted} />
          </div>
        </div>

        {/* Jobs List */}
        <div className="lg:col-span-2">
          <div className="card">
            <div className="flex items-center justify-between mb-4">
              <h3 className="text-xl font-semibold flex items-center">
                <FaClock className="text-green-500 mr-2" />
                Job Queue
              </h3>
              <span className="text-sm text-gray-500">
                {isLoading ? 'Loading...' : `${filteredJobs.length} jobs`}
              </span>
            </div>

            {/* Status Tabs */}
            <div className="flex border-b border-gray-200 mb-4">
              <button
                onClick={() => setActiveTab('all')}
                className={`px-4 py-2 font-medium transition-colors ${
                  activeTab === 'all'
                    ? 'border-b-2 border-riptide-blue text-riptide-blue'
                    : 'text-gray-600 hover:text-gray-800'
                }`}
              >
                All ({counts.all})
              </button>
              <button
                onClick={() => setActiveTab('active')}
                className={`px-4 py-2 font-medium transition-colors ${
                  activeTab === 'active'
                    ? 'border-b-2 border-riptide-blue text-riptide-blue'
                    : 'text-gray-600 hover:text-gray-800'
                }`}
              >
                Active ({counts.active})
              </button>
              <button
                onClick={() => setActiveTab('completed')}
                className={`px-4 py-2 font-medium transition-colors ${
                  activeTab === 'completed'
                    ? 'border-b-2 border-riptide-blue text-riptide-blue'
                    : 'text-gray-600 hover:text-gray-800'
                }`}
              >
                Completed ({counts.completed})
              </button>
              <button
                onClick={() => setActiveTab('failed')}
                className={`px-4 py-2 font-medium transition-colors ${
                  activeTab === 'failed'
                    ? 'border-b-2 border-riptide-blue text-riptide-blue'
                    : 'text-gray-600 hover:text-gray-800'
                }`}
              >
                Failed ({counts.failed})
              </button>
            </div>

            {/* Jobs List */}
            <JobsList jobs={filteredJobs} onRefresh={fetchJobs} />
          </div>
        </div>
      </div>
    </div>
  )
}
