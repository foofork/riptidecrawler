import { useState } from 'react'
import { FaCode, FaCopy, FaCheck } from 'react-icons/fa'
import CodeMirror from '@uiw/react-codemirror'
import { javascript } from '@codemirror/lang-javascript'
import { python } from '@codemirror/lang-python'
import { usePlaygroundStore } from '../hooks/usePlaygroundStore'
import { generateCode } from '../utils/codeGenerator'

/**
 * Code Exporter Component
 * Allows exporting API requests as code in multiple languages
 */
export default function CodeExporter() {
  const { selectedEndpoint, requestBody } = usePlaygroundStore()
  const [language, setLanguage] = useState('javascript')
  const [copied, setCopied] = useState(false)

  const languages = [
    { id: 'javascript', name: 'JavaScript', icon: '📜' },
    { id: 'python', name: 'Python', icon: '🐍' },
    { id: 'curl', name: 'cURL', icon: '💻' },
    { id: 'rust', name: 'Rust', icon: '🦀' }
  ]

  const code = generateCode(selectedEndpoint, requestBody, language)

  const copyCode = async () => {
    await navigator.clipboard.writeText(code)
    setCopied(true)
    setTimeout(() => setCopied(false), 2000)
  }

  return (
    <div className="card">
      <div className="flex items-center justify-between mb-4">
        <h3 className="text-lg font-semibold flex items-center">
          <FaCode className="mr-2 text-riptide-blue" />
          Export to Code
        </h3>
        <button
          onClick={copyCode}
          className={`px-3 py-1.5 rounded-lg text-sm font-medium transition-all flex items-center space-x-2 ${
            copied
              ? 'bg-green-500 text-white'
              : 'bg-gray-200 text-gray-700 hover:bg-gray-300'
          }`}
          title="Copy code"
        >
          {copied ? (
            <>
              <FaCheck />
              <span>Copied!</span>
            </>
          ) : (
            <>
              <FaCopy />
              <span>Copy</span>
            </>
          )}
        </button>
      </div>

      {/* Language Selector */}
      <div className="flex flex-wrap gap-2 mb-4">
        {languages.map(lang => (
          <button
            key={lang.id}
            onClick={() => setLanguage(lang.id)}
            className={`px-3 py-1.5 rounded-lg text-sm font-medium transition-colors flex items-center space-x-1 ${
              language === lang.id
                ? 'bg-riptide-blue text-white'
                : 'bg-gray-200 text-gray-700 hover:bg-gray-300'
            }`}
            aria-label={`Export as ${lang.name}`}
          >
            <span>{lang.icon}</span>
            <span>{lang.name}</span>
          </button>
        ))}
      </div>

      {/* Code Display */}
      <div className="border border-gray-300 rounded-lg overflow-hidden">
        <CodeMirror
          value={code}
          height="300px"
          extensions={[
            language === 'javascript' ? javascript() :
            language === 'python' ? python() :
            javascript()
          ]}
          editable={false}
          theme="light"
        />
      </div>

      <div className="mt-3 p-3 bg-blue-50 border border-blue-200 rounded-lg">
        <p className="text-xs text-blue-800">
          💡 <strong>Tip:</strong> Copy this code and use it in your application to make requests to the RipTide API.
        </p>
      </div>
    </div>
  )
}
