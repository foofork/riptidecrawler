import { useEffect } from 'react'
import CodeMirror from '@uiw/react-codemirror'
import { json } from '@codemirror/lang-json'
import { usePlaygroundStore } from '../hooks/usePlaygroundStore'

export default function RequestBuilder() {
  const { selectedEndpoint, requestBody, setRequestBody } = usePlaygroundStore()

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

  if (selectedEndpoint.method === 'GET') {
    return (
      <div className="bg-gray-50 rounded-lg p-4 text-sm text-gray-600">
        This is a GET request. No request body required.
      </div>
    )
  }

  return (
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
  )
}
