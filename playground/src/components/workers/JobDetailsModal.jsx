import { FaTimes } from 'react-icons/fa'
import CodeMirror from '@uiw/react-codemirror'
import { json } from '@codemirror/lang-json'

export default function JobDetailsModal({ job, onClose, onRefresh }) {
  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-white rounded-lg shadow-xl max-w-4xl w-full mx-4 max-h-[90vh] overflow-hidden flex flex-col">
        {/* Header */}
        <div className="flex items-center justify-between p-6 border-b border-gray-200">
          <div>
            <h3 className="text-2xl font-bold text-gray-800">Job Details</h3>
            <p className="text-sm text-gray-600 mt-1">ID: {job.id}</p>
          </div>
          <button
            onClick={onClose}
            className="p-2 text-gray-600 hover:text-gray-800 transition-colors"
          >
            <FaTimes />
          </button>
        </div>

        {/* Content */}
        <div className="flex-1 overflow-y-auto p-6">
          <div className="space-y-6">
            {/* Basic Info */}
            <div className="grid grid-cols-2 gap-4">
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">Job Type</label>
                <p className="text-gray-900">{job.job_type}</p>
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">Status</label>
                <p className="text-gray-900">{job.status}</p>
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">Priority</label>
                <p className="text-gray-900">{job.priority || 'normal'}</p>
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">Duration</label>
                <p className="text-gray-900">{job.duration ? `${(job.duration / 1000).toFixed(2)}s` : 'N/A'}</p>
              </div>
            </div>

            {/* Timestamps */}
            <div className="grid grid-cols-2 gap-4">
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">Created At</label>
                <p className="text-gray-900 text-sm">
                  {job.created_at ? new Date(job.created_at).toLocaleString() : 'N/A'}
                </p>
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">Completed At</label>
                <p className="text-gray-900 text-sm">
                  {job.completed_at ? new Date(job.completed_at).toLocaleString() : 'N/A'}
                </p>
              </div>
            </div>

            {/* Progress */}
            {job.progress !== undefined && (
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-2">Progress</label>
                <div className="w-full bg-gray-200 rounded-full h-3">
                  <div
                    className="bg-riptide-blue h-3 rounded-full transition-all"
                    style={{ width: `${job.progress * 100}%` }}
                  />
                </div>
                <p className="text-sm text-gray-600 mt-1">{Math.round(job.progress * 100)}%</p>
              </div>
            )}

            {/* Configuration */}
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-2">Configuration</label>
              <div className="border border-gray-300 rounded-lg overflow-hidden">
                <CodeMirror
                  value={JSON.stringify(job.config || {}, null, 2)}
                  height="200px"
                  extensions={[json()]}
                  readOnly={true}
                  theme="light"
                  basicSetup={{
                    lineNumbers: true,
                    foldGutter: true,
                  }}
                />
              </div>
            </div>

            {/* Result */}
            {job.result && (
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-2">Result</label>
                <div className="border border-gray-300 rounded-lg overflow-hidden">
                  <CodeMirror
                    value={JSON.stringify(job.result, null, 2)}
                    height="300px"
                    extensions={[json()]}
                    readOnly={true}
                    theme="light"
                    basicSetup={{
                      lineNumbers: true,
                      foldGutter: true,
                    }}
                  />
                </div>
              </div>
            )}

            {/* Error */}
            {job.error && (
              <div>
                <label className="block text-sm font-medium text-red-700 mb-2">Error</label>
                <div className="bg-red-50 border border-red-200 rounded-lg p-4">
                  <pre className="text-sm text-red-800 whitespace-pre-wrap">
                    {typeof job.error === 'string' ? job.error : JSON.stringify(job.error, null, 2)}
                  </pre>
                </div>
              </div>
            )}
          </div>
        </div>

        {/* Footer */}
        <div className="flex items-center justify-end space-x-3 p-6 border-t border-gray-200">
          <button
            onClick={onClose}
            className="btn-secondary"
          >
            Close
          </button>
        </div>
      </div>
    </div>
  )
}
