import { useState } from 'react'
import RequestBuilder from '../components/RequestBuilder'
import ResponseViewer from '../components/ResponseViewer'
import EndpointSelector from '../components/EndpointSelector'
import RequestHistory from '../components/RequestHistory'
import RequestPreview from '../components/RequestPreview'
import CodeExporter from '../components/CodeExporter'
import ErrorBoundary from '../components/ErrorBoundary'
import { FaPlay, FaCopy, FaDownload, FaRedo, FaExclamationTriangle } from 'react-icons/fa'
import { usePlaygroundStore } from '../hooks/usePlaygroundStore'

/**
 * Enhanced Playground Component
 * Features: error boundaries, request history, preview, code export, retry functionality
 */
export default function Playground() {
  const {
    selectedEndpoint,
    response,
    isLoading,
    error,
    executeRequest
  } = usePlaygroundStore()

  const [activeTab, setActiveTab] = useState('response')
  const [retryCount, setRetryCount] = useState(0)

  const handleExecute = async () => {
    setRetryCount(0)
    await executeRequest()
  }

  const handleRetry = async () => {
    setRetryCount(retryCount + 1)
    await executeRequest()
  }

  const copyResponse = () => {
    if (response) {
      navigator.clipboard.writeText(JSON.stringify(response, null, 2))
    }
  }

  const downloadResponse = () => {
    if (response) {
      const blob = new Blob([JSON.stringify(response, null, 2)], { type: 'application/json' })
      const url = URL.createObjectURL(blob)
      const a = document.createElement('a')
      a.href = url
      a.download = `riptide-response-${Date.now()}.json`
      a.click()
      URL.revokeObjectURL(url)
    }
  }

  return (
    <div className="container mx-auto px-4 py-8">
      <div className="mb-6">
        <h2 className="text-3xl font-bold text-gray-800 mb-2">API Playground</h2>
        <p className="text-gray-600">
          Test RipTide API endpoints interactively. Build requests, view responses, and generate code.
        </p>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        {/* Left Panel - Request Builder (2 columns) */}
        <div className="lg:col-span-2 space-y-6">
          <ErrorBoundary fallbackMessage="Error loading request builder">
            <div className="card">
              <h3 className="text-xl font-semibold mb-4 flex items-center">
                <span className="text-riptide-blue mr-2">→</span>
                Request
              </h3>

              <EndpointSelector />
              <RequestBuilder />

              <div className="mt-6 space-y-3">
                {/* Error Display with Retry */}
                {error && (
                  <div className="p-4 bg-red-50 border-2 border-red-200 rounded-lg" role="alert">
                    <div className="flex items-start">
                      <FaExclamationTriangle className="text-red-500 mt-0.5 mr-3 flex-shrink-0" />
                      <div className="flex-1">
                        <p className="text-sm font-medium text-red-800 mb-2">{error}</p>
                        <button
                          onClick={handleRetry}
                          disabled={isLoading}
                          className="text-sm text-red-700 hover:text-red-900 font-medium flex items-center"
                        >
                          <FaRedo className="mr-1" />
                          {retryCount > 0 ? `Retry (Attempt ${retryCount + 1})` : 'Try Again'}
                        </button>
                      </div>
                    </div>
                  </div>
                )}

                {/* Execute Button */}
                <button
                  onClick={handleExecute}
                  disabled={isLoading || !selectedEndpoint}
                  className="btn-primary w-full flex items-center justify-center space-x-2 disabled:opacity-50 disabled:cursor-not-allowed"
                  aria-label={isLoading ? 'Executing request' : 'Execute request'}
                >
                  {isLoading ? (
                    <>
                      <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-white"></div>
                      <span>Executing...</span>
                    </>
                  ) : (
                    <>
                      <FaPlay />
                      <span>Execute Request</span>
                    </>
                  )}
                </button>
              </div>
            </div>
          </ErrorBoundary>

          {/* Request Preview */}
          <ErrorBoundary fallbackMessage="Error loading request preview">
            <RequestPreview />
          </ErrorBoundary>

          {/* Quick Info */}
          {selectedEndpoint && (
            <ErrorBoundary fallbackMessage="Error loading endpoint info">
              <div className="bg-blue-50 border border-blue-200 rounded-lg p-4">
                <h4 className="font-semibold text-blue-900 mb-2 flex items-center">
                  📘 Endpoint Info
                </h4>
                <div className="text-sm text-blue-800 space-y-1">
                  <p><strong>Method:</strong> <code className="bg-blue-100 px-2 py-0.5 rounded">{selectedEndpoint.method}</code></p>
                  <p><strong>Path:</strong> <code className="bg-blue-100 px-2 py-0.5 rounded">{selectedEndpoint.path}</code></p>
                  <p><strong>Category:</strong> {selectedEndpoint.category}</p>
                  <p><strong>Description:</strong> {selectedEndpoint.description}</p>
                  {selectedEndpoint.parameters && Object.keys(selectedEndpoint.parameters).length > 0 && (
                    <div className="mt-2 pt-2 border-t border-blue-200">
                      <p className="font-semibold mb-1">Parameters:</p>
                      <ul className="list-disc list-inside pl-2 space-y-0.5">
                        {Object.entries(selectedEndpoint.parameters).map(([name, info]) => (
                          <li key={name}>
                            <code className="bg-blue-100 px-1 rounded">{name}</code>
                            {info.required && <span className="text-red-600 ml-1">*</span>}: {info.description}
                          </li>
                        ))}
                      </ul>
                    </div>
                  )}
                </div>
              </div>
            </ErrorBoundary>
          )}
        </div>

        {/* Right Panel - Response Viewer & Utilities (1 column) */}
        <div className="space-y-6">
          <ErrorBoundary fallbackMessage="Error loading response viewer">
            <div className="card">
              <div className="flex items-center justify-between mb-4">
                <h3 className="text-xl font-semibold flex items-center">
                  <span className="text-green-500 mr-2">←</span>
                  Response
                </h3>

                {response && (
                  <div className="flex space-x-2">
                    <button
                      onClick={copyResponse}
                      className="p-2 text-gray-600 hover:text-riptide-blue transition-colors"
                      title="Copy response"
                      aria-label="Copy response to clipboard"
                    >
                      <FaCopy />
                    </button>
                    <button
                      onClick={downloadResponse}
                      className="p-2 text-gray-600 hover:text-riptide-blue transition-colors"
                      title="Download response"
                      aria-label="Download response as JSON file"
                    >
                      <FaDownload />
                    </button>
                  </div>
                )}
              </div>

              {/* Tabs */}
              <div className="flex border-b border-gray-200 mb-4" role="tablist">
                <button
                  onClick={() => setActiveTab('response')}
                  className={`px-4 py-2 font-medium transition-colors ${
                    activeTab === 'response'
                      ? 'border-b-2 border-riptide-blue text-riptide-blue'
                      : 'text-gray-600 hover:text-gray-800'
                  }`}
                  role="tab"
                  aria-selected={activeTab === 'response'}
                  aria-controls="response-panel"
                >
                  Response
                </button>
                <button
                  onClick={() => setActiveTab('headers')}
                  className={`px-4 py-2 font-medium transition-colors ${
                    activeTab === 'headers'
                      ? 'border-b-2 border-riptide-blue text-riptide-blue'
                      : 'text-gray-600 hover:text-gray-800'
                  }`}
                  role="tab"
                  aria-selected={activeTab === 'headers'}
                  aria-controls="headers-panel"
                >
                  Headers
                </button>
                <button
                  onClick={() => setActiveTab('code')}
                  className={`px-4 py-2 font-medium transition-colors ${
                    activeTab === 'code'
                      ? 'border-b-2 border-riptide-blue text-riptide-blue'
                      : 'text-gray-600 hover:text-gray-800'
                  }`}
                  role="tab"
                  aria-selected={activeTab === 'code'}
                  aria-controls="code-panel"
                >
                  Code
                </button>
              </div>

              <div role="tabpanel" id={`${activeTab}-panel`}>
                <ResponseViewer activeTab={activeTab} />
              </div>
            </div>
          </ErrorBoundary>

          {/* Code Exporter */}
          <ErrorBoundary fallbackMessage="Error loading code exporter">
            <CodeExporter />
          </ErrorBoundary>

          {/* Request History */}
          <ErrorBoundary fallbackMessage="Error loading request history">
            <RequestHistory />
          </ErrorBoundary>
        </div>
      </div>
    </div>
  )
}
