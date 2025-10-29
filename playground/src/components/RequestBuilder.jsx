import { useEffect, useState } from 'react'
import { FaChevronDown, FaChevronUp } from 'react-icons/fa'
import CodeMirror from '@uiw/react-codemirror'
import { json } from '@codemirror/lang-json'
import { usePlaygroundStore } from '../hooks/usePlaygroundStore'
import Tooltip from './Tooltip'

/**
 * Enhanced Request Builder Component
 * Features: tooltips, collapsible advanced options, validation
 */
export default function RequestBuilder() {
  const { selectedEndpoint, requestBody, setRequestBody, pathParameters, setPathParameters } = usePlaygroundStore()
  const [showAdvanced, setShowAdvanced] = useState(false)
  const [jsonError, setJsonError] = useState(null)

  useEffect(() => {
    if (selectedEndpoint?.defaultBody) {
      setRequestBody(JSON.stringify(selectedEndpoint.defaultBody, null, 2))
    }
  }, [selectedEndpoint, setRequestBody])

  useEffect(() => {
    // Validate JSON as user types
    if (requestBody) {
      try {
        JSON.parse(requestBody)
        setJsonError(null)
      } catch (e) {
        setJsonError(e.message)
      }
    }
  }, [requestBody])

  if (!selectedEndpoint) {
    return (
      <div className="text-center py-12 text-gray-500">
        <div className="text-5xl mb-4">🔍</div>
        <p className="font-medium">Select an endpoint to get started</p>
        <p className="text-sm mt-2">Choose from the dropdown above to configure your request</p>
      </div>
    )
  }

  const handleParameterChange = (paramName, value) => {
    setPathParameters({
      ...pathParameters,
      [paramName]: value
    })
  }

  // Check if endpoint has path parameters
  const hasParameters = selectedEndpoint.parameters && Object.keys(selectedEndpoint.parameters).length > 0

  return (
    <div className="space-y-6">
      {/* Path Parameters Section */}
      {hasParameters && (
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-3 flex items-center">
            Path Parameters
            <Tooltip content="Values that replace placeholders in the URL path" />
          </label>
          <div className="space-y-3">
            {Object.entries(selectedEndpoint.parameters).map(([paramName, paramInfo]) => (
              <div key={paramName}>
                <label
                  className="block text-xs font-medium text-gray-600 mb-1 flex items-center"
                  htmlFor={`param-${paramName}`}
                >
                  {paramName}
                  {paramInfo.required && <span className="text-red-500 ml-1" aria-label="required">*</span>}
                  {paramInfo.description && <Tooltip content={paramInfo.description} />}
                </label>
                <input
                  id={`param-${paramName}`}
                  type="text"
                  value={pathParameters[paramName] || ''}
                  onChange={(e) => handleParameterChange(paramName, e.target.value)}
                  placeholder={paramInfo.description || `Enter ${paramName}`}
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-riptide-blue focus:border-transparent text-sm"
                  required={paramInfo.required}
                  aria-required={paramInfo.required}
                  aria-describedby={paramInfo.description ? `param-${paramName}-desc` : undefined}
                />
                {paramInfo.description && (
                  <p id={`param-${paramName}-desc`} className="text-xs text-gray-500 mt-1 sr-only">
                    {paramInfo.description}
                  </p>
                )}
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Request Body Section */}
      {selectedEndpoint.method !== 'GET' && selectedEndpoint.method !== 'DELETE' ? (
        <div>
          <div className="flex items-center justify-between mb-2">
            <label className="block text-sm font-medium text-gray-700 flex items-center">
              Request Body (JSON)
              <Tooltip content="The JSON data to send in the request body" />
            </label>

            {/* Advanced Options Toggle */}
            <button
              onClick={() => setShowAdvanced(!showAdvanced)}
              className="text-xs text-riptide-blue hover:text-blue-700 flex items-center transition-colors"
              aria-expanded={showAdvanced}
              aria-controls="advanced-options"
            >
              Advanced Options
              {showAdvanced ? <FaChevronUp className="ml-1" /> : <FaChevronDown className="ml-1" />}
            </button>
          </div>

          {/* JSON Editor */}
          <div className={`border rounded-lg overflow-hidden ${jsonError ? 'border-red-300' : 'border-gray-300'}`}>
            <CodeMirror
              value={requestBody}
              height="300px"
              extensions={[json()]}
              onChange={(value) => setRequestBody(value)}
              theme="light"
              basicSetup={{
                lineNumbers: true,
                highlightActiveLineGutter: true,
                highlightSpecialChars: true,
                foldGutter: true,
                bracketMatching: true,
                syntaxHighlighting: true,
                autocompletion: true,
              }}
              aria-label="JSON request body editor"
            />
          </div>

          {/* JSON Validation Feedback */}
          {jsonError ? (
            <div className="mt-2 p-2 bg-red-50 border border-red-200 rounded text-xs text-red-700 flex items-start" role="alert">
              <span className="font-bold mr-1">❌</span>
              <span>JSON Error: {jsonError}</span>
            </div>
          ) : (
            <p className="text-xs text-gray-500 mt-2 flex items-center">
              <span className="text-green-500 mr-1">✓</span>
              Valid JSON. Edit the request body above.
            </p>
          )}

          {/* Advanced Options Panel */}
          {showAdvanced && (
            <div id="advanced-options" className="mt-4 p-4 bg-gray-50 border border-gray-200 rounded-lg space-y-3">
              <h4 className="text-sm font-semibold text-gray-700 mb-2">Advanced Request Options</h4>

              <div>
                <label className="flex items-center text-sm text-gray-700">
                  <input
                    type="checkbox"
                    className="mr-2 rounded border-gray-300 text-riptide-blue focus:ring-riptide-blue"
                    defaultChecked={false}
                  />
                  <span>Pretty print response</span>
                  <Tooltip content="Automatically format the response JSON for readability" />
                </label>
              </div>

              <div>
                <label className="flex items-center text-sm text-gray-700">
                  <input
                    type="checkbox"
                    className="mr-2 rounded border-gray-300 text-riptide-blue focus:ring-riptide-blue"
                    defaultChecked={true}
                  />
                  <span>Follow redirects</span>
                  <Tooltip content="Automatically follow HTTP redirects" />
                </label>
              </div>

              <div>
                <label className="block text-xs font-medium text-gray-600 mb-1">
                  Request Timeout (ms)
                  <Tooltip content="Maximum time to wait for a response" />
                </label>
                <input
                  type="number"
                  defaultValue={30000}
                  min={1000}
                  max={300000}
                  step={1000}
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-riptide-blue focus:border-transparent text-sm"
                  aria-label="Request timeout in milliseconds"
                />
              </div>
            </div>
          )}
        </div>
      ) : (
        <div className="bg-gray-50 rounded-lg p-4 text-sm text-gray-600 flex items-center">
          <span className="text-2xl mr-3">ℹ️</span>
          <span>This is a {selectedEndpoint.method} request. No request body required.</span>
        </div>
      )}
    </div>
  )
}
