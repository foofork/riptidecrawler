import { useEffect, useState } from 'react'
import { FaEye, FaCopy } from 'react-icons/fa'
import CodeMirror from '@uiw/react-codemirror'
import { json } from '@codemirror/lang-json'
import { usePlaygroundStore } from '../hooks/usePlaygroundStore'
import { config } from '../config/environment'

/**
 * Request Preview Component
 * Shows real-time preview of the HTTP request that will be sent
 */
export default function RequestPreview() {
  const { selectedEndpoint, requestBody, pathParameters } = usePlaygroundStore()
  const [preview, setPreview] = useState('')

  useEffect(() => {
    if (!selectedEndpoint) {
      setPreview('// Select an endpoint to preview the request')
      return
    }

    // Build URL with path parameters
    let url = selectedEndpoint.path
    if (selectedEndpoint.parameters) {
      Object.entries(pathParameters).forEach(([key, value]) => {
        url = url.replace(`:${key}`, value || `:${key}`)
      })
    }

    // Build the full request preview
    const fullUrl = `${config.api.baseUrl}${url}`

    let previewText = `${selectedEndpoint.method} ${fullUrl}\n`
    previewText += `Content-Type: application/json\n`

    if (selectedEndpoint.method !== 'GET' && selectedEndpoint.method !== 'DELETE') {
      previewText += `\n${requestBody || '{}'}`
    }

    setPreview(previewText)
  }, [selectedEndpoint, requestBody, pathParameters])

  const copyPreview = () => {
    navigator.clipboard.writeText(preview)
  }

  return (
    <div className="card">
      <div className="flex items-center justify-between mb-3">
        <h3 className="text-lg font-semibold flex items-center">
          <FaEye className="mr-2 text-riptide-blue" />
          Request Preview
        </h3>
        <button
          onClick={copyPreview}
          className="text-sm text-gray-500 hover:text-riptide-blue transition-colors flex items-center"
          title="Copy preview"
        >
          <FaCopy className="mr-1" />
          Copy
        </button>
      </div>

      <div className="border border-gray-300 rounded-lg overflow-hidden">
        <CodeMirror
          value={preview}
          height="150px"
          extensions={[json()]}
          editable={false}
          theme="light"
          basicSetup={{
            lineNumbers: false,
            highlightActiveLineGutter: false,
            foldGutter: false,
          }}
        />
      </div>

      <p className="text-xs text-gray-500 mt-2">
        This is the actual HTTP request that will be sent to the API
      </p>
    </div>
  )
}
