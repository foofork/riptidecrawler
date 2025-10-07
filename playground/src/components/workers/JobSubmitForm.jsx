import { useState } from 'react'
import { FaPlay, FaSpinner } from 'react-icons/fa'
import axios from 'axios'
import CodeMirror from '@uiw/react-codemirror'
import { json } from '@codemirror/lang-json'

export default function JobSubmitForm({ onJobSubmitted }) {
  const [jobType, setJobType] = useState('crawl')
  const [priority, setPriority] = useState('normal')
  const [config, setConfig] = useState(JSON.stringify({
    url: 'https://example.com'
  }, null, 2))
  const [isSubmitting, setIsSubmitting] = useState(false)
  const [error, setError] = useState(null)
  const [success, setSuccess] = useState(null)

  const jobTypes = [
    { value: 'crawl', label: 'Crawl' },
    { value: 'render', label: 'Render' },
    { value: 'pdf', label: 'PDF Processing' },
    { value: 'spider', label: 'Spider Crawl' },
    { value: 'deepsearch', label: 'Deep Search' }
  ]

  const priorities = [
    { value: 'low', label: 'Low', color: 'text-gray-600' },
    { value: 'normal', label: 'Normal', color: 'text-blue-600' },
    { value: 'high', label: 'High', color: 'text-orange-600' },
    { value: 'urgent', label: 'Urgent', color: 'text-red-600' }
  ]

  const handleSubmit = async (e) => {
    e.preventDefault()
    setError(null)
    setSuccess(null)
    setIsSubmitting(true)

    try {
      const configObj = JSON.parse(config)

      const response = await axios.post('/api/workers/jobs', {
        job_type: jobType,
        priority,
        config: configObj
      })

      setSuccess(`Job submitted successfully! ID: ${response.data.job_id}`)

      // Reset form
      setConfig(JSON.stringify({ url: 'https://example.com' }, null, 2))

      // Notify parent
      if (onJobSubmitted) {
        onJobSubmitted(response.data)
      }

      // Clear success message after 3 seconds
      setTimeout(() => setSuccess(null), 3000)
    } catch (err) {
      setError(err.response?.data?.error || err.message || 'Failed to submit job')
    } finally {
      setIsSubmitting(false)
    }
  }

  return (
    <form onSubmit={handleSubmit} className="space-y-4">
      {/* Job Type */}
      <div>
        <label className="block text-sm font-medium text-gray-700 mb-2">
          Job Type
        </label>
        <select
          value={jobType}
          onChange={(e) => setJobType(e.target.value)}
          className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-riptide-blue focus:border-transparent"
        >
          {jobTypes.map(type => (
            <option key={type.value} value={type.value}>
              {type.label}
            </option>
          ))}
        </select>
      </div>

      {/* Priority */}
      <div>
        <label className="block text-sm font-medium text-gray-700 mb-2">
          Priority
        </label>
        <select
          value={priority}
          onChange={(e) => setPriority(e.target.value)}
          className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-riptide-blue focus:border-transparent"
        >
          {priorities.map(p => (
            <option key={p.value} value={p.value}>
              {p.label}
            </option>
          ))}
        </select>
      </div>

      {/* Config */}
      <div>
        <label className="block text-sm font-medium text-gray-700 mb-2">
          Configuration (JSON)
        </label>
        <div className="border border-gray-300 rounded-lg overflow-hidden">
          <CodeMirror
            value={config}
            height="200px"
            extensions={[json()]}
            onChange={(value) => setConfig(value)}
            theme="light"
            basicSetup={{
              lineNumbers: true,
              foldGutter: true,
              bracketMatching: true,
            }}
          />
        </div>
        <p className="text-xs text-gray-500 mt-1">
          Job configuration depends on job type
        </p>
      </div>

      {/* Success/Error Messages */}
      {success && (
        <div className="bg-green-50 border border-green-200 rounded-lg p-3 text-sm text-green-800">
          {success}
        </div>
      )}

      {error && (
        <div className="bg-red-50 border border-red-200 rounded-lg p-3 text-sm text-red-800">
          {error}
        </div>
      )}

      {/* Submit Button */}
      <button
        type="submit"
        disabled={isSubmitting}
        className="btn-primary w-full flex items-center justify-center space-x-2"
      >
        {isSubmitting ? (
          <>
            <FaSpinner className="animate-spin" />
            <span>Submitting...</span>
          </>
        ) : (
          <>
            <FaPlay />
            <span>Submit Job</span>
          </>
        )}
      </button>
    </form>
  )
}
