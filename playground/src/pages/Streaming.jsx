import { useState } from 'react'
import { FaPlay, FaStop, FaStream, FaSpinner } from 'react-icons/fa'
import CodeMirror from '@uiw/react-codemirror'
import { json } from '@codemirror/lang-json'
import { useNDJSONStream, useSSEStream } from '../hooks/useSSEStream'
import LiveProgressWidget from '../components/streaming/LiveProgressWidget'

export default function Streaming() {
  const [streamType, setStreamType] = useState('ndjson')
  const [isStreaming, setIsStreaming] = useState(false)
  const [showLiveWidget, setShowLiveWidget] = useState(false)
  const [streamResults, setStreamResults] = useState([])
  const [urls, setUrls] = useState(JSON.stringify({
    urls: ['https://example.com', 'https://example.org'],
    options: {
      concurrency: 2
    }
  }, null, 2))

  const handleStartStream = async () => {
    setIsStreaming(true)
    setShowLiveWidget(true)
    setStreamResults([])

    try {
      const body = JSON.parse(urls)

      if (streamType === 'ndjson') {
        // Start NDJSON stream
        // In production, this would use the useNDJSONStream hook
        console.log('Starting NDJSON stream with:', body)
      } else if (streamType === 'sse') {
        // Start SSE stream
        // In production, this would use the useSSEStream hook
        console.log('Starting SSE stream with:', body)
      }

      // Simulate streaming for demo
      simulateStream()
    } catch (error) {
      console.error('Failed to parse request body:', error)
      setIsStreaming(false)
    }
  }

  const simulateStream = () => {
    let count = 0
    const interval = setInterval(() => {
      count++

      setStreamResults(prev => [...prev, {
        timestamp: new Date().toISOString(),
        url: `https://example.com/page-${count}`,
        status: Math.random() > 0.1 ? 'success' : 'failed',
        duration: Math.round(Math.random() * 2000),
        size: Math.round(Math.random() * 50000)
      }])

      if (count >= 10) {
        clearInterval(interval)
        setIsStreaming(false)
      }
    }, 1000)
  }

  const handleStopStream = () => {
    setIsStreaming(false)
    setShowLiveWidget(false)
  }

  const getStatusBadge = (status) => {
    if (status === 'success') {
      return <span className="px-2 py-1 bg-green-100 text-green-800 rounded-full text-xs font-medium">Success</span>
    }
    return <span className="px-2 py-1 bg-red-100 text-red-800 rounded-full text-xs font-medium">Failed</span>
  }

  return (
    <div className="container mx-auto px-4 py-8">
      <div className="mb-6">
        <h2 className="text-3xl font-bold text-gray-800 mb-2">Real-Time Streaming</h2>
        <p className="text-gray-600">
          Test streaming endpoints with live progress updates (NDJSON, SSE, WebSocket)
        </p>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Configuration Panel */}
        <div className="space-y-6">
          <div className="card">
            <h3 className="text-xl font-semibold mb-4 flex items-center">
              <FaStream className="text-riptide-blue mr-2" />
              Stream Configuration
            </h3>

            {/* Stream Type Selection */}
            <div className="mb-4">
              <label className="block text-sm font-medium text-gray-700 mb-2">
                Stream Type
              </label>
              <div className="grid grid-cols-3 gap-2">
                <button
                  onClick={() => setStreamType('ndjson')}
                  className={`px-4 py-2 rounded-lg border-2 transition-colors ${
                    streamType === 'ndjson'
                      ? 'border-riptide-blue bg-riptide-blue text-white'
                      : 'border-gray-300 bg-white text-gray-700 hover:border-riptide-blue'
                  }`}
                >
                  NDJSON
                </button>
                <button
                  onClick={() => setStreamType('sse')}
                  className={`px-4 py-2 rounded-lg border-2 transition-colors ${
                    streamType === 'sse'
                      ? 'border-riptide-blue bg-riptide-blue text-white'
                      : 'border-gray-300 bg-white text-gray-700 hover:border-riptide-blue'
                  }`}
                >
                  SSE
                </button>
                <button
                  onClick={() => setStreamType('websocket')}
                  className={`px-4 py-2 rounded-lg border-2 transition-colors ${
                    streamType === 'websocket'
                      ? 'border-riptide-blue bg-riptide-blue text-white'
                      : 'border-gray-300 bg-white text-gray-700 hover:border-riptide-blue'
                  }`}
                >
                  WebSocket
                </button>
              </div>
            </div>

            {/* Endpoint Info */}
            <div className="mb-4 bg-blue-50 border border-blue-200 rounded-lg p-3">
              <p className="text-sm text-blue-900">
                <strong>Endpoint:</strong>{' '}
                {streamType === 'ndjson' && 'POST /crawl/stream'}
                {streamType === 'sse' && 'POST /crawl/sse'}
                {streamType === 'websocket' && 'GET /crawl/ws'}
              </p>
            </div>

            {/* Request Body */}
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-2">
                Request Body (JSON)
              </label>
              <div className="border border-gray-300 rounded-lg overflow-hidden">
                <CodeMirror
                  value={urls}
                  height="200px"
                  extensions={[json()]}
                  onChange={(value) => setUrls(value)}
                  theme="light"
                  basicSetup={{
                    lineNumbers: true,
                    foldGutter: true,
                    bracketMatching: true,
                  }}
                />
              </div>
            </div>

            {/* Control Buttons */}
            <div className="mt-6 flex space-x-3">
              {!isStreaming ? (
                <button
                  onClick={handleStartStream}
                  className="btn-primary flex items-center space-x-2"
                >
                  <FaPlay />
                  <span>Start Stream</span>
                </button>
              ) : (
                <button
                  onClick={handleStopStream}
                  className="bg-red-600 hover:bg-red-700 text-white px-4 py-2 rounded-lg font-medium transition-colors flex items-center space-x-2"
                >
                  <FaStop />
                  <span>Stop Stream</span>
                </button>
              )}
            </div>
          </div>

          {/* Info Card */}
          <div className="card bg-gradient-to-br from-blue-50 to-blue-100 border-blue-200">
            <h4 className="font-semibold text-blue-900 mb-2">About Streaming</h4>
            <div className="text-sm text-blue-800 space-y-2">
              <p>
                <strong>NDJSON:</strong> Newline-delimited JSON for efficient streaming of multiple results.
              </p>
              <p>
                <strong>SSE:</strong> Server-Sent Events for real-time updates over HTTP.
              </p>
              <p>
                <strong>WebSocket:</strong> Bi-directional communication for interactive streaming.
              </p>
            </div>
          </div>
        </div>

        {/* Results Panel */}
        <div className="space-y-6">
          <div className="card">
            <div className="flex items-center justify-between mb-4">
              <h3 className="text-xl font-semibold flex items-center">
                <FaSpinner className={`mr-2 ${isStreaming ? 'animate-spin text-riptide-blue' : 'text-gray-400'}`} />
                Stream Results
              </h3>
              <span className="text-sm text-gray-500">
                {streamResults.length} results
              </span>
            </div>

            {streamResults.length === 0 ? (
              <div className="text-center py-12 text-gray-500">
                <FaStream className="mx-auto text-4xl mb-2 opacity-50" />
                <p>No stream data yet. Start a stream to see results.</p>
              </div>
            ) : (
              <div className="space-y-2 max-h-[600px] overflow-y-auto">
                {streamResults.map((result, idx) => (
                  <div
                    key={idx}
                    className="border border-gray-200 rounded-lg p-3 hover:border-riptide-blue transition-colors"
                  >
                    <div className="flex items-center justify-between mb-2">
                      <span className="text-sm font-mono text-gray-600">
                        #{idx + 1}
                      </span>
                      {getStatusBadge(result.status)}
                    </div>
                    <p className="text-sm text-gray-800 truncate mb-1">
                      <strong>URL:</strong> {result.url}
                    </p>
                    <div className="flex items-center space-x-4 text-xs text-gray-600">
                      <span>Duration: {result.duration}ms</span>
                      <span>Size: {(result.size / 1024).toFixed(1)}KB</span>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>
        </div>
      </div>

      {/* Live Progress Widget */}
      {showLiveWidget && (
        <LiveProgressWidget
          crawlId="demo-crawl-123"
          onClose={() => setShowLiveWidget(false)}
        />
      )}
    </div>
  )
}
