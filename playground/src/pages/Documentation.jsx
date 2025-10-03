import { FaBook, FaExternalLinkAlt } from 'react-icons/fa'

const docSections = [
  {
    title: 'Getting Started',
    links: [
      { name: 'Quick Start Guide', url: '/docs/user/installation.md' },
      { name: 'API Usage', url: '/docs/user/api-usage.md' },
      { name: 'Configuration', url: '/docs/user/configuration.md' },
    ]
  },
  {
    title: 'API Reference',
    links: [
      { name: 'OpenAPI Specification', url: '/docs/api/openapi.yaml' },
      { name: 'Endpoint Catalog', url: '/docs/api/ENDPOINT_CATALOG.md' },
      { name: 'Error Handling', url: '/docs/api/error-handling.md' },
      { name: 'Streaming API', url: '/docs/api/streaming.md' },
    ]
  },
  {
    title: 'Architecture',
    links: [
      { name: 'System Overview', url: '/docs/architecture/system-overview.md' },
      { name: 'Integration Crosswalk', url: '/docs/architecture/integration-crosswalk.md' },
      { name: 'Deployment Guide', url: '/docs/architecture/deployment-guide.md' },
    ]
  },
  {
    title: 'Advanced Topics',
    links: [
      { name: 'WASM Integration', url: '/docs/api/wasm-integration.md' },
      { name: 'Browser Pool Integration', url: '/docs/api/browser-pool-integration.md' },
      { name: 'Performance Monitoring', url: '/docs/api/performance.md' },
      { name: 'Security Guide', url: '/docs/api/security.md' },
    ]
  },
  {
    title: 'Development',
    links: [
      { name: 'Developer Guide', url: '/docs/development/getting-started.md' },
      { name: 'Testing Guide', url: '/docs/development/testing.md' },
      { name: 'Contributing', url: '/docs/development/contributing.md' },
    ]
  }
]

const quickLinks = [
  {
    title: 'Swagger UI',
    description: 'Interactive API documentation',
    url: 'http://localhost:8081',
    icon: '📚',
    external: true
  },
  {
    title: 'GitHub Repository',
    description: 'Source code and issues',
    url: 'https://github.com/your-org/riptide-api',
    icon: '💻',
    external: true
  },
  {
    title: 'Full Documentation',
    description: 'Complete docs on GitHub',
    url: 'https://github.com/your-org/riptide-api/tree/main/docs',
    icon: '📖',
    external: true
  }
]

export default function Documentation() {
  return (
    <div className="container mx-auto px-4 py-8">
      <div className="mb-8">
        <h2 className="text-3xl font-bold text-gray-800 mb-2">Documentation</h2>
        <p className="text-gray-600">
          Comprehensive guides and API reference for RipTide
        </p>
      </div>

      {/* Quick Links */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-8">
        {quickLinks.map(link => (
          <a
            key={link.title}
            href={link.url}
            target={link.external ? '_blank' : '_self'}
            rel={link.external ? 'noopener noreferrer' : ''}
            className="card hover:shadow-lg transition-shadow cursor-pointer group"
          >
            <div className="flex items-start space-x-4">
              <div className="text-4xl">{link.icon}</div>
              <div className="flex-1">
                <h3 className="font-semibold text-gray-800 group-hover:text-riptide-blue transition-colors flex items-center">
                  {link.title}
                  {link.external && <FaExternalLinkAlt className="ml-2 text-sm" />}
                </h3>
                <p className="text-sm text-gray-600 mt-1">{link.description}</p>
              </div>
            </div>
          </a>
        ))}
      </div>

      {/* Documentation Sections */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        {docSections.map(section => (
          <div key={section.title} className="card">
            <h3 className="text-xl font-bold text-gray-800 mb-4 flex items-center">
              <FaBook className="mr-2 text-riptide-blue" />
              {section.title}
            </h3>
            <ul className="space-y-2">
              {section.links.map(link => (
                <li key={link.name}>
                  <a
                    href={link.url}
                    className="text-riptide-blue hover:underline flex items-center"
                  >
                    <span className="mr-2">→</span>
                    {link.name}
                  </a>
                </li>
              ))}
            </ul>
          </div>
        ))}
      </div>

      {/* API Endpoints Overview */}
      <div className="card mt-8">
        <h3 className="text-xl font-bold text-gray-800 mb-4">API Endpoints Overview</h3>
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {[
            { category: 'Health & Metrics', count: 2 },
            { category: 'Crawling', count: 5 },
            { category: 'Streaming', count: 4 },
            { category: 'Search', count: 2 },
            { category: 'Spider', count: 3 },
            { category: 'Strategies', count: 2 },
            { category: 'PDF', count: 3 },
            { category: 'Stealth', count: 4 },
            { category: 'Tables', count: 2 },
            { category: 'LLM', count: 4 },
            { category: 'Sessions', count: 12 },
            { category: 'Workers', count: 9 },
            { category: 'Monitoring', count: 6 }
          ].map(item => (
            <div key={item.category} className="p-3 bg-gray-50 rounded-lg">
              <div className="font-semibold text-gray-800">{item.category}</div>
              <div className="text-sm text-gray-600">{item.count} endpoints</div>
            </div>
          ))}
        </div>
        <div className="mt-4 text-center">
          <p className="text-sm text-gray-600">
            <strong>59 Total Endpoints</strong> · <a href="/docs/api/ENDPOINT_CATALOG.md" className="text-riptide-blue hover:underline">View Full Catalog</a>
          </p>
        </div>
      </div>
    </div>
  )
}
