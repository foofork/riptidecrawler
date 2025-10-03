import CodeMirror from '@uiw/react-codemirror'
import { json } from '@codemirror/lang-json'
import { javascript } from '@codemirror/lang-javascript'
import { python } from '@codemirror/lang-python'
import { usePlaygroundStore } from '../hooks/usePlaygroundStore'
import { generateCode } from '../utils/codeGenerator'
import { useState } from 'react'

export default function ResponseViewer({ activeTab }) {
  const { response, responseHeaders, selectedEndpoint, requestBody, isLoading } = usePlaygroundStore()
  const [codeLanguage, setCodeLanguage] = useState('javascript')

  if (isLoading) {
    return (
      <div className="flex items-center justify-center py-12">
        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-riptide-blue"></div>
      </div>
    )
  }

  if (!response && activeTab !== 'code') {
    return (
      <div className="text-center py-12 text-gray-500">
        Execute a request to see the response
      </div>
    )
  }

  if (activeTab === 'response') {
    return (
      <div>
        {response?.status && (
          <div className="mb-4 flex items-center space-x-2">
            <span className={`px-3 py-1 rounded-full text-sm font-medium ${
              response.status >= 200 && response.status < 300
                ? 'bg-green-100 text-green-800'
                : response.status >= 400
                ? 'bg-red-100 text-red-800'
                : 'bg-yellow-100 text-yellow-800'
            }`}>
              {response.status} {response.statusText}
            </span>
            <span className="text-sm text-gray-500">
              {response.duration}ms
            </span>
          </div>
        )}
        <div className="border border-gray-300 rounded-lg overflow-hidden">
          <CodeMirror
            value={JSON.stringify(response?.data || response, null, 2)}
            height="400px"
            extensions={[json()]}
            editable={false}
            theme="light"
          />
        </div>
      </div>
    )
  }

  if (activeTab === 'headers') {
    return (
      <div className="border border-gray-300 rounded-lg overflow-hidden">
        <CodeMirror
          value={JSON.stringify(responseHeaders || {}, null, 2)}
          height="400px"
          extensions={[json()]}
          editable={false}
          theme="light"
        />
      </div>
    )
  }

  if (activeTab === 'code') {
    const code = generateCode(selectedEndpoint, requestBody, codeLanguage)

    return (
      <div>
        <div className="mb-4 flex space-x-2">
          {['javascript', 'python', 'curl', 'rust'].map(lang => (
            <button
              key={lang}
              onClick={() => setCodeLanguage(lang)}
              className={`px-3 py-1 rounded-lg text-sm font-medium transition-colors ${
                codeLanguage === lang
                  ? 'bg-riptide-blue text-white'
                  : 'bg-gray-200 text-gray-700 hover:bg-gray-300'
              }`}
            >
              {lang.charAt(0).toUpperCase() + lang.slice(1)}
            </button>
          ))}
        </div>
        <div className="border border-gray-300 rounded-lg overflow-hidden">
          <CodeMirror
            value={code}
            height="400px"
            extensions={[
              codeLanguage === 'javascript' ? javascript() :
              codeLanguage === 'python' ? python() :
              json()
            ]}
            editable={false}
            theme="light"
          />
        </div>
      </div>
    )
  }
}
