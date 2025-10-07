import { useState } from 'react'
import RequestBuilder from '../components/RequestBuilder'
import ResponseViewer from '../components/ResponseViewer'
import EndpointSelector from '../components/EndpointSelector'
import { FaPlay, FaCopy, FaDownload } from 'react-icons/fa'
import { usePlaygroundStore } from '../hooks/usePlaygroundStore'

export default function Playground() {
  const {
    selectedEndpoint,
    requestBody,
    response,
    isLoading,
    setResponse,
    setIsLoading,
    executeRequest
  } = usePlaygroundStore()

  const [activeTab, setActiveTab] = useState('response')

  const handleExecute = async () => {
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

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Left Panel - Request Builder */}
        <div className="space-y-6">
          <div className="card">
            <h3 className="text-xl font-semibold mb-4 flex items-center">
              <span className="text-riptide-blue mr-2">→</span>
              Request
            </h3>

            <EndpointSelector />
            <RequestBuilder />

            <div className="mt-6">
              <button
                onClick={handleExecute}
                disabled={isLoading}
                className="btn-primary w-full flex items-center justify-center space-x-2"
              >
                <FaPlay />
                <span>{isLoading ? 'Executing...' : 'Execute Request'}</span>
              </button>
            </div>
          </div>

          {/* Quick Info */}
          {selectedEndpoint && (
            <div className="bg-blue-50 border border-blue-200 rounded-lg p-4">
              <h4 className="font-semibold text-blue-900 mb-2">Endpoint Info</h4>
              <div className="text-sm text-blue-800 space-y-1">
                <p><strong>Method:</strong> {selectedEndpoint.method}</p>
                <p><strong>Path:</strong> {selectedEndpoint.path}</p>
                <p><strong>Category:</strong> {selectedEndpoint.category}</p>
                <p><strong>Description:</strong> {selectedEndpoint.description}</p>
                {selectedEndpoint.parameters && Object.keys(selectedEndpoint.parameters).length > 0 && (
                  <div className="mt-2 pt-2 border-t border-blue-200">
                    <p className="font-semibold mb-1">Parameters:</p>
                    <ul className="list-disc list-inside pl-2 space-y-0.5">
                      {Object.entries(selectedEndpoint.parameters).map(([name, info]) => (
                        <li key={name}>
                          <code className="bg-blue-100 px-1 rounded">{name}</code>: {info.description}
                        </li>
                      ))}
                    </ul>
                  </div>
                )}
              </div>
            </div>
          )}
        </div>

        {/* Right Panel - Response Viewer */}
        <div className="space-y-6">
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
                  >
                    <FaCopy />
                  </button>
                  <button
                    onClick={downloadResponse}
                    className="p-2 text-gray-600 hover:text-riptide-blue transition-colors"
                    title="Download response"
                  >
                    <FaDownload />
                  </button>
                </div>
              )}
            </div>

            {/* Tabs */}
            <div className="flex border-b border-gray-200 mb-4">
              <button
                onClick={() => setActiveTab('response')}
                className={`px-4 py-2 font-medium transition-colors ${
                  activeTab === 'response'
                    ? 'border-b-2 border-riptide-blue text-riptide-blue'
                    : 'text-gray-600 hover:text-gray-800'
                }`}
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
              >
                Code
              </button>
            </div>

            <ResponseViewer activeTab={activeTab} />
          </div>
        </div>
      </div>
    </div>
  )
}
