import { useState } from 'react'
import { FaCode, FaCopy, FaPlay } from 'react-icons/fa'
import CodeMirror from '@uiw/react-codemirror'
import { json } from '@codemirror/lang-json'
import { javascript } from '@codemirror/lang-javascript'
import { python } from '@codemirror/lang-python'

const exampleCategories = [
  {
    id: 'getting-started',
    name: 'Getting Started',
    examples: [
      {
        id: 'basic-crawl',
        title: 'Basic URL Crawl',
        description: 'Extract content from a single URL',
        language: 'javascript',
        code: `const response = await fetch('http://localhost:8080/crawl', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    urls: ['https://example.com'],
    options: {
      concurrency: 1,
      cache_mode: 'read_write'
    }
  })
})

const result = await response.json()
console.log(result.results[0])`,
      },
      {
        id: 'health-check',
        title: 'Health Check',
        description: 'Check if the API is healthy',
        language: 'curl',
        code: `curl http://localhost:8080/healthz | jq '.'

# Expected output:
# {
#   "status": "healthy",
#   "version": "1.0.0",
#   "dependencies": {...}
# }`,
      },
    ]
  },
  {
    id: 'advanced',
    name: 'Advanced Use Cases',
    examples: [
      {
        id: 'batch-crawl',
        title: 'Batch Crawling',
        description: 'Crawl multiple URLs with concurrency',
        language: 'python',
        code: `import requests

urls_to_crawl = [
    'https://example.com',
    'https://example.org',
    'https://example.net'
]

response = requests.post(
    'http://localhost:8080/crawl',
    json={
        'urls': urls_to_crawl,
        'options': {
            'concurrency': 3,
            'cache_mode': 'read_write'
        }
    }
)

results = response.json()
for result in results['results']:
    print(f"Title: {result['document']['title']}")
    print(f"URL: {result['url']}")
    print("---")`,
      },
      {
        id: 'streaming',
        title: 'Streaming Results',
        description: 'Stream crawl results in real-time',
        language: 'javascript',
        code: `const response = await fetch('http://localhost:8080/crawl/stream', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    urls: ['https://example.com', 'https://example.org'],
    options: { concurrency: 2 }
  })
})

const reader = response.body.getReader()
const decoder = new TextDecoder()

while (true) {
  const { done, value } = await reader.read()
  if (done) break

  const chunk = decoder.decode(value)
  const lines = chunk.split('\\n').filter(Boolean)

  for (const line of lines) {
    const result = JSON.parse(line)
    console.log('Received:', result.url)
  }
}`,
      },
      {
        id: 'deep-search',
        title: 'Deep Search',
        description: 'Search and extract content',
        language: 'python',
        code: `import requests

response = requests.post(
    'http://localhost:8080/deepsearch',
    json={
        'query': 'machine learning tutorials',
        'limit': 20,
        'include_content': True,
        'crawl_options': {
            'extract_mode': 'article'
        }
    }
)

results = response.json()
for article in results['results']:
    print(f"Title: {article['title']}")
    print(f"URL: {article['url']}")
    print(f"Summary: {article['content'][:200]}...")
    print("---")`,
      },
    ]
  },
  {
    id: 'production',
    name: 'Production Patterns',
    examples: [
      {
        id: 'error-handling',
        title: 'Error Handling',
        description: 'Robust error handling with retries',
        language: 'javascript',
        code: `async function crawlWithRetry(url, maxRetries = 3) {
  for (let attempt = 1; attempt <= maxRetries; attempt++) {
    try {
      const response = await fetch('http://localhost:8080/crawl', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          urls: [url],
          options: { concurrency: 1 }
        })
      })

      if (!response.ok) {
        throw new Error(\`HTTP \${response.status}\`)
      }

      return await response.json()
    } catch (error) {
      console.log(\`Attempt \${attempt} failed: \${error.message}\`)

      if (attempt === maxRetries) {
        throw error
      }

      // Exponential backoff
      await new Promise(r => setTimeout(r, Math.pow(2, attempt) * 1000))
    }
  }
}

// Usage
try {
  const result = await crawlWithRetry('https://example.com')
  console.log('Success:', result)
} catch (error) {
  console.error('Failed after retries:', error)
}`,
      },
      {
        id: 'session-management',
        title: 'Session Management',
        description: 'Using sessions for authentication',
        language: 'python',
        code: `import requests

class RipTideSession:
    def __init__(self, base_url='http://localhost:8080'):
        self.base_url = base_url
        self.session_id = None

    def create_session(self, name, config):
        """Create a new session with cookies/auth"""
        response = requests.post(
            f'{self.base_url}/sessions',
            json={'name': name, 'config': config}
        )
        session = response.json()
        self.session_id = session['id']
        return session

    def crawl(self, urls, options=None):
        """Crawl with session context"""
        payload = {
            'urls': urls,
            'options': options or {},
            'session_id': self.session_id
        }

        response = requests.post(
            f'{self.base_url}/crawl',
            json=payload
        )
        return response.json()

# Usage
session = RipTideSession()
session.create_session('my-session', {
    'user_agent': 'MyBot/1.0',
    'cookies': [{'name': 'token', 'value': 'abc123'}]
})

results = session.crawl(['https://protected-site.com'])`,
      },
      {
        id: 'monitoring',
        title: 'Health Monitoring',
        description: 'Monitor API health and performance',
        language: 'javascript',
        code: `class RipTideMonitor {
  constructor(apiUrl = 'http://localhost:8080') {
    this.apiUrl = apiUrl
    this.healthHistory = []
  }

  async checkHealth() {
    const response = await fetch(\`\${this.apiUrl}/healthz\`)
    const health = await response.json()

    this.healthHistory.push({
      timestamp: new Date(),
      status: health.status,
      dependencies: health.dependencies
    })

    return health
  }

  async getMetrics() {
    const response = await fetch(\`\${this.apiUrl}/metrics\`)
    return await response.text()
  }

  async getHealthScore() {
    const response = await fetch(
      \`\${this.apiUrl}/monitoring/health-score\`
    )
    return await response.json()
  }

  async monitor(intervalMs = 30000) {
    console.log('Starting monitoring...')

    setInterval(async () => {
      try {
        const health = await this.checkHealth()
        const score = await this.getHealthScore()

        console.log(\`Health: \${health.status}, Score: \${score.score}\`)

        if (health.status !== 'healthy' || score.score < 80) {
          console.warn('⚠️  Service degraded!')
          // Send alert here
        }
      } catch (error) {
        console.error('❌ Monitoring error:', error)
      }
    }, intervalMs)
  }
}

// Usage
const monitor = new RipTideMonitor()
monitor.monitor(30000) // Check every 30 seconds`,
      },
    ]
  },
  {
    id: 'integrations',
    name: 'Integrations',
    examples: [
      {
        id: 'python-sdk',
        title: 'Python SDK',
        description: 'Using the official Python SDK',
        language: 'python',
        code: `# Install: pip install riptide-client
from riptide import RipTide

# Initialize client
client = RipTide('http://localhost:8080')

# Basic crawl
result = client.crawl(['https://example.com'])

# Advanced options
result = client.crawl(
    urls=['https://example.com'],
    options={
        'concurrency': 5,
        'cache_mode': 'read_write',
        'extract_mode': 'article'
    }
)

# Deep search
results = client.search(
    query='web scraping',
    limit=10,
    include_content=True
)

# Streaming
for result in client.stream_crawl(['https://example.com']):
    print(f"Received: {result['url']}")

# Session management
session = client.create_session('my-session', {
    'user_agent': 'MyBot/1.0'
})

result = client.crawl(['https://site.com'], session_id=session['id'])`,
      },
      {
        id: 'nodejs-integration',
        title: 'Node.js Integration',
        description: 'Full-featured Node.js integration',
        language: 'javascript',
        code: `// app.js
import RipTide from './riptide-client.js'

const riptide = new RipTide('http://localhost:8080')

// Express.js endpoint
app.post('/scrape', async (req, res) => {
  try {
    const { urls } = req.body

    const results = await riptide.crawl(urls, {
      concurrency: 3,
      cache_mode: 'read_write'
    })

    res.json({ success: true, data: results })
  } catch (error) {
    res.status(500).json({
      success: false,
      error: error.message
    })
  }
})

// Background job with Bull queue
queue.process('crawl-job', async (job) => {
  const { urls } = job.data

  const results = await riptide.crawl(urls)

  // Process results
  await saveToDatabase(results)

  return { processed: results.length }
})`,
      },
    ]
  }
]

export default function Examples() {
  const [selectedCategory, setSelectedCategory] = useState('getting-started')
  const [selectedExample, setSelectedExample] = useState(exampleCategories[0].examples[0])

  const copyCode = () => {
    navigator.clipboard.writeText(selectedExample.code)
  }

  const loadInPlayground = () => {
    // TODO: Implement loading example into playground
    window.location.href = '/'
  }

  return (
    <div className="container mx-auto px-4 py-8">
      <div className="mb-6">
        <h2 className="text-3xl font-bold text-gray-800 mb-2">Code Examples</h2>
        <p className="text-gray-600">
          Ready-to-use code snippets for common use cases
        </p>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-4 gap-6">
        {/* Sidebar - Categories and Examples */}
        <div className="lg:col-span-1">
          <div className="card sticky top-4">
            {exampleCategories.map(category => (
              <div key={category.id} className="mb-4">
                <h3 className="font-semibold text-gray-700 mb-2">{category.name}</h3>
                <div className="space-y-1">
                  {category.examples.map(example => (
                    <button
                      key={example.id}
                      onClick={() => {
                        setSelectedCategory(category.id)
                        setSelectedExample(example)
                      }}
                      className={`w-full text-left px-3 py-2 rounded-lg text-sm transition-colors ${
                        selectedExample.id === example.id
                          ? 'bg-riptide-blue text-white'
                          : 'hover:bg-gray-100 text-gray-700'
                      }`}
                    >
                      {example.title}
                    </button>
                  ))}
                </div>
              </div>
            ))}
          </div>
        </div>

        {/* Main Content - Example Display */}
        <div className="lg:col-span-3">
          <div className="card">
            <div className="mb-6">
              <div className="flex items-start justify-between mb-2">
                <div>
                  <h3 className="text-2xl font-bold text-gray-800">
                    {selectedExample.title}
                  </h3>
                  <p className="text-gray-600 mt-1">{selectedExample.description}</p>
                </div>
                <span className="px-3 py-1 bg-gray-100 text-gray-700 rounded-lg text-sm font-mono">
                  {selectedExample.language}
                </span>
              </div>
            </div>

            <div className="relative">
              <div className="absolute top-4 right-4 flex space-x-2 z-10">
                <button
                  onClick={copyCode}
                  className="p-2 bg-white hover:bg-gray-100 rounded-lg shadow-md transition-colors"
                  title="Copy code"
                >
                  <FaCopy className="text-gray-700" />
                </button>
                <button
                  onClick={loadInPlayground}
                  className="p-2 bg-white hover:bg-gray-100 rounded-lg shadow-md transition-colors"
                  title="Try in playground"
                >
                  <FaPlay className="text-gray-700" />
                </button>
              </div>

              <div className="border border-gray-300 rounded-lg overflow-hidden">
                <CodeMirror
                  value={selectedExample.code}
                  height="500px"
                  extensions={[
                    selectedExample.language === 'javascript' ? javascript() :
                    selectedExample.language === 'python' ? python() :
                    json()
                  ]}
                  editable={false}
                  theme="light"
                />
              </div>
            </div>

            <div className="mt-6 p-4 bg-blue-50 border border-blue-200 rounded-lg">
              <h4 className="font-semibold text-blue-900 mb-2 flex items-center">
                <FaCode className="mr-2" />
                Try This Example
              </h4>
              <p className="text-sm text-blue-800">
                Copy the code above and run it in your environment, or click the play button to load it in the playground.
              </p>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}
