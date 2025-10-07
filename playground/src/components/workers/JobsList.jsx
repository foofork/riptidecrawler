import { useState } from 'react'
import { FaSpinner, FaCheckCircle, FaTimesCircle, FaClock, FaEye, FaDownload } from 'react-icons/fa'
import axios from 'axios'
import JobDetailsModal from './JobDetailsModal'

export default function JobsList({ jobs, onRefresh }) {
  const [selectedJob, setSelectedJob] = useState(null)
  const [showModal, setShowModal] = useState(false)

  const getStatusIcon = (status) => {
    switch (status) {
      case 'pending':
        return <FaClock className="text-yellow-500" />
      case 'running':
        return <FaSpinner className="text-blue-500 animate-spin" />
      case 'completed':
        return <FaCheckCircle className="text-green-500" />
      case 'failed':
        return <FaTimesCircle className="text-red-500" />
      default:
        return <FaClock className="text-gray-500" />
    }
  }

  const getStatusBadge = (status) => {
    const badges = {
      pending: 'bg-yellow-100 text-yellow-800',
      running: 'bg-blue-100 text-blue-800',
      completed: 'bg-green-100 text-green-800',
      failed: 'bg-red-100 text-red-800'
    }

    return (
      <span className={`px-2 py-1 rounded-full text-xs font-medium ${badges[status] || 'bg-gray-100 text-gray-800'}`}>
        {status}
      </span>
    )
  }

  const getPriorityBadge = (priority) => {
    const badges = {
      low: 'bg-gray-100 text-gray-600',
      normal: 'bg-blue-100 text-blue-600',
      high: 'bg-orange-100 text-orange-600',
      urgent: 'bg-red-100 text-red-600'
    }

    return (
      <span className={`px-2 py-1 rounded-full text-xs font-medium ${badges[priority] || 'bg-gray-100 text-gray-600'}`}>
        {priority}
      </span>
    )
  }

  const handleViewJob = async (job) => {
    try {
      const response = await axios.get(`/api/workers/jobs/${job.id}`)
      setSelectedJob(response.data)
      setShowModal(true)
    } catch (error) {
      console.error('Failed to fetch job details:', error)
    }
  }

  const handleDownloadResult = async (jobId) => {
    try {
      const response = await axios.get(`/api/workers/jobs/${jobId}/result`)
      const blob = new Blob([JSON.stringify(response.data, null, 2)], { type: 'application/json' })
      const url = URL.createObjectURL(blob)
      const a = document.createElement('a')
      a.href = url
      a.download = `job-${jobId}-result.json`
      a.click()
      URL.revokeObjectURL(url)
    } catch (error) {
      console.error('Failed to download result:', error)
    }
  }

  const formatDuration = (ms) => {
    if (!ms) return 'N/A'
    const seconds = Math.floor(ms / 1000)
    const minutes = Math.floor(seconds / 60)
    if (minutes > 0) {
      return `${minutes}m ${seconds % 60}s`
    }
    return `${seconds}s`
  }

  const formatTimestamp = (timestamp) => {
    if (!timestamp) return 'N/A'
    return new Date(timestamp).toLocaleString()
  }

  if (jobs.length === 0) {
    return (
      <div className="text-center py-12 text-gray-500">
        <FaClock className="mx-auto text-4xl mb-2 opacity-50" />
        <p>No jobs found</p>
      </div>
    )
  }

  return (
    <>
      <div className="space-y-3">
        {jobs.map(job => (
          <div
            key={job.id}
            className="border border-gray-200 rounded-lg p-4 hover:border-riptide-blue transition-colors"
          >
            <div className="flex items-start justify-between">
              <div className="flex items-start space-x-3 flex-1">
                <div className="mt-1">
                  {getStatusIcon(job.status)}
                </div>
                <div className="flex-1">
                  <div className="flex items-center space-x-2 mb-2">
                    <h4 className="font-semibold text-gray-800">
                      {job.job_type || 'Unknown'}
                    </h4>
                    {getStatusBadge(job.status)}
                    {getPriorityBadge(job.priority || 'normal')}
                  </div>

                  <div className="text-sm text-gray-600 space-y-1">
                    <p><strong>ID:</strong> <code className="bg-gray-100 px-1 rounded">{job.id}</code></p>
                    {job.created_at && (
                      <p><strong>Created:</strong> {formatTimestamp(job.created_at)}</p>
                    )}
                    {job.duration && (
                      <p><strong>Duration:</strong> {formatDuration(job.duration)}</p>
                    )}
                    {job.progress !== undefined && (
                      <div className="mt-2">
                        <div className="flex items-center justify-between text-xs mb-1">
                          <span>Progress</span>
                          <span>{Math.round(job.progress * 100)}%</span>
                        </div>
                        <div className="w-full bg-gray-200 rounded-full h-2">
                          <div
                            className="bg-riptide-blue h-2 rounded-full transition-all"
                            style={{ width: `${job.progress * 100}%` }}
                          />
                        </div>
                      </div>
                    )}
                  </div>
                </div>
              </div>

              <div className="flex items-center space-x-2 ml-4">
                <button
                  onClick={() => handleViewJob(job)}
                  className="p-2 text-gray-600 hover:text-riptide-blue transition-colors"
                  title="View details"
                >
                  <FaEye />
                </button>
                {job.status === 'completed' && (
                  <button
                    onClick={() => handleDownloadResult(job.id)}
                    className="p-2 text-gray-600 hover:text-green-600 transition-colors"
                    title="Download result"
                  >
                    <FaDownload />
                  </button>
                )}
              </div>
            </div>
          </div>
        ))}
      </div>

      {/* Job Details Modal */}
      {showModal && selectedJob && (
        <JobDetailsModal
          job={selectedJob}
          onClose={() => {
            setShowModal(false)
            setSelectedJob(null)
          }}
          onRefresh={onRefresh}
        />
      )}
    </>
  )
}
