import { useEffect } from 'react'
import CodeMirror from '@uiw/react-codemirror'
import { json } from '@codemirror/lang-json'
import { usePlaygroundStore } from '../hooks/usePlaygroundStore'

export default function RequestBuilder() {
  const { selectedEndpoint, requestBody, setRequestBody, pathParameters, setPathParameters } = usePlaygroundStore()

  useEffect(() => {
    if (selectedEndpoint?.defaultBody) {
      setRequestBody(JSON.stringify(selectedEndpoint.defaultBody, null, 2))
    }
  }, [selectedEndpoint, setRequestBody])

  if (!selectedEndpoint) {
    return (
      <div className="text-center py-12 text-gray-500">
        Select an endpoint to get started
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
          <label className="block text-sm font-medium text-gray-700 mb-3">
            Path Parameters
          </label>
          <div className="space-y-3">
            {Object.entries(selectedEndpoint.parameters).map(([paramName, paramInfo]) => (
              <div key={paramName}>
                <label className="block text-xs font-medium text-gray-600 mb-1">
                  {paramName}
                  {paramInfo.required && <span className="text-red-500 ml-1">*</span>}
                </label>
                <input
                  type="text"
                  value={pathParameters[paramName] || ''}
                  onChange={(e) => handleParameterChange(paramName, e.target.value)}
                  placeholder={paramInfo.description || `Enter ${paramName}`}
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-riptide-blue focus:border-transparent text-sm"
                />
                {paramInfo.description && (
                  <p className="text-xs text-gray-500 mt-1">{paramInfo.description}</p>
                )}
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Request Body Section */}
      {selectedEndpoint.method !== 'GET' && selectedEndpoint.method !== 'DELETE' ? (
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-2">
            Request Body (JSON)
          </label>
          <div className="border border-gray-300 rounded-lg overflow-hidden">
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
              }}
            />
          </div>
          <p className="text-xs text-gray-500 mt-2">
            Edit the JSON request body. Syntax errors will be highlighted.
          </p>
        </div>
      ) : (
        <div className="bg-gray-50 rounded-lg p-4 text-sm text-gray-600">
          This is a {selectedEndpoint.method} request. No request body required.
        </div>
      )}
    </div>
  )
}
