export const endpoints = [
  // Health & Metrics
  {
    id: 'health',
    category: 'Health & Metrics',
    name: 'Health Check',
    method: 'GET',
    path: '/healthz',
    description: 'Check system health and dependency status',
  },
  {
    id: 'metrics',
    category: 'Health & Metrics',
    name: 'Prometheus Metrics',
    method: 'GET',
    path: '/metrics',
    description: 'Get Prometheus-formatted metrics',
  },

  // Crawling
  {
    id: 'crawl',
    category: 'Crawling',
    name: 'Batch Crawl',
    method: 'POST',
    path: '/crawl',
    description: 'Crawl multiple URLs with adaptive routing',
    defaultBody: {
      urls: ['https://example.com'],
      options: {
        concurrency: 1,
        cache_mode: 'read_write',
      }
    }
  },
  {
    id: 'render',
    category: 'Crawling',
    name: 'Headless Render',
    method: 'POST',
    path: '/render',
    description: 'Render JavaScript-heavy pages',
    defaultBody: {
      url: 'https://example.com',
      wait_time: 2000,
      screenshot: false
    }
  },

  // Streaming
  {
    id: 'crawl-stream',
    category: 'Streaming',
    name: 'Stream Crawl (NDJSON)',
    method: 'POST',
    path: '/crawl/stream',
    description: 'Stream crawl results in real-time',
    defaultBody: {
      urls: ['https://example.com', 'https://example.org'],
      options: {
        concurrency: 2
      }
    }
  },
  {
    id: 'crawl-sse',
    category: 'Streaming',
    name: 'Server-Sent Events',
    method: 'POST',
    path: '/crawl/sse',
    description: 'Stream using SSE protocol',
    defaultBody: {
      urls: ['https://example.com'],
      options: {
        concurrency: 1
      }
    }
  },

  // Search
  {
    id: 'deepsearch',
    category: 'Search',
    name: 'Deep Search',
    method: 'POST',
    path: '/deepsearch',
    description: 'Search and extract content',
    defaultBody: {
      query: 'web scraping best practices',
      limit: 10,
      include_content: true
    }
  },

  // Spider
  {
    id: 'spider-start',
    category: 'Spider',
    name: 'Start Spider',
    method: 'POST',
    path: '/spider/start',
    description: 'Start deep crawling',
    defaultBody: {
      url: 'https://example.com',
      max_depth: 2,
      max_pages: 10
    }
  },

  // Strategies
  {
    id: 'strategies-info',
    category: 'Strategies',
    name: 'Strategy Info',
    method: 'GET',
    path: '/strategies/info',
    description: 'Get available extraction strategies',
  },

  // Sessions
  {
    id: 'sessions-list',
    category: 'Sessions',
    name: 'List Sessions',
    method: 'GET',
    path: '/sessions',
    description: 'List all active sessions',
  },
  {
    id: 'sessions-create',
    category: 'Sessions',
    name: 'Create Session',
    method: 'POST',
    path: '/sessions',
    description: 'Create a new session',
    defaultBody: {
      name: 'my-session',
      config: {
        user_agent: 'Mozilla/5.0...',
        cookies: []
      }
    }
  },

  // Workers
  {
    id: 'workers-status',
    category: 'Workers',
    name: 'Worker Status',
    method: 'GET',
    path: '/workers/status',
    description: 'Get worker queue status',
  },

  // Monitoring
  {
    id: 'monitoring-health-score',
    category: 'Monitoring',
    name: 'Health Score',
    method: 'GET',
    path: '/monitoring/health-score',
    description: 'Get overall health score (0-100)',
  },
  {
    id: 'monitoring-performance',
    category: 'Monitoring',
    name: 'Performance Report',
    method: 'GET',
    path: '/monitoring/performance-report',
    description: 'Get detailed performance metrics',
  },

  // Pipeline
  {
    id: 'pipeline-phases',
    category: 'Pipeline',
    name: 'Pipeline Phases',
    method: 'GET',
    path: '/pipeline/phases',
    description: 'Get pipeline phase metrics',
  },
]
