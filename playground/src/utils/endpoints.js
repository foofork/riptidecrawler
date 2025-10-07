export const endpoints = [
  // ========================================
  // Health & Metrics (5 endpoints)
  // ========================================
  {
    id: 'health',
    category: 'Health & Metrics',
    name: 'Health Check',
    method: 'GET',
    path: '/healthz',
    description: 'Check system health and dependency status',
  },
  {
    id: 'health-detailed',
    category: 'Health & Metrics',
    name: 'Detailed Health',
    method: 'GET',
    path: '/api/health/detailed',
    description: 'Get detailed health information for all components',
  },
  {
    id: 'health-component',
    category: 'Health & Metrics',
    name: 'Component Health',
    method: 'GET',
    path: '/health/:component',
    description: 'Check health of specific component (redis, wasm, http, etc.)',
    parameters: {
      component: { type: 'string', required: true, description: 'Component name' }
    }
  },
  {
    id: 'health-metrics',
    category: 'Health & Metrics',
    name: 'Health Metrics',
    method: 'GET',
    path: '/health/metrics',
    description: 'Get health metrics for all components',
  },
  {
    id: 'metrics',
    category: 'Health & Metrics',
    name: 'Prometheus Metrics',
    method: 'GET',
    path: '/metrics',
    description: 'Get Prometheus-formatted metrics for monitoring',
  },

  // ========================================
  // Crawling (2 endpoints)
  // ========================================
  {
    id: 'crawl',
    category: 'Crawling',
    name: 'Batch Crawl',
    method: 'POST',
    path: '/crawl',
    description: 'Crawl multiple URLs with adaptive routing and caching',
    defaultBody: {
      urls: ['https://example.com'],
      options: {
        concurrency: 1,
        cache_mode: 'read_write',
        extract_links: true,
        follow_redirects: true
      }
    }
  },
  {
    id: 'render',
    category: 'Crawling',
    name: 'Headless Render',
    method: 'POST',
    path: '/render',
    description: 'Render JavaScript-heavy pages using headless browser',
    defaultBody: {
      url: 'https://example.com',
      wait_time: 2000,
      screenshot: false,
      viewport: { width: 1920, height: 1080 }
    }
  },

  // ========================================
  // Streaming (4 endpoints)
  // ========================================
  {
    id: 'crawl-stream',
    category: 'Streaming',
    name: 'Stream Crawl (NDJSON)',
    method: 'POST',
    path: '/crawl/stream',
    description: 'Stream crawl results in real-time as NDJSON',
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
    description: 'Stream crawl results using SSE protocol',
    defaultBody: {
      urls: ['https://example.com'],
      options: {
        concurrency: 1
      }
    }
  },
  {
    id: 'crawl-ws',
    category: 'Streaming',
    name: 'WebSocket Stream',
    method: 'GET',
    path: '/crawl/ws',
    description: 'Stream crawl results via WebSocket connection',
  },
  {
    id: 'deepsearch-stream',
    category: 'Streaming',
    name: 'Deep Search Stream',
    method: 'POST',
    path: '/deepsearch/stream',
    description: 'Stream deep search results in real-time',
    defaultBody: {
      query: 'web scraping',
      limit: 10
    }
  },

  // ========================================
  // Search (1 endpoint)
  // ========================================
  {
    id: 'deepsearch',
    category: 'Search',
    name: 'Deep Search',
    method: 'POST',
    path: '/deepsearch',
    description: 'Search and extract content with AI-powered analysis',
    defaultBody: {
      query: 'web scraping best practices',
      limit: 10,
      include_content: true,
      search_depth: 2
    }
  },

  // ========================================
  // PDF Processing (3 endpoints)
  // ========================================
  {
    id: 'pdf-process',
    category: 'PDF',
    name: 'Process PDF',
    method: 'POST',
    path: '/pdf/process',
    description: 'Process PDF document and extract text, images, tables',
    defaultBody: {
      url: 'https://example.com/document.pdf',
      options: {
        extract_images: false,
        extract_tables: true,
        extract_metadata: true
      }
    }
  },
  {
    id: 'pdf-process-stream',
    category: 'PDF',
    name: 'Stream PDF Processing',
    method: 'POST',
    path: '/pdf/process-stream',
    description: 'Process PDF with real-time progress updates',
    defaultBody: {
      url: 'https://example.com/document.pdf',
      options: {
        extract_images: true
      }
    }
  },
  {
    id: 'pdf-health',
    category: 'PDF',
    name: 'PDF Health Check',
    method: 'GET',
    path: '/pdf/health',
    description: 'Check PDF processing capabilities and status',
  },

  // ========================================
  // Stealth (4 endpoints)
  // ========================================
  {
    id: 'stealth-configure',
    category: 'Stealth',
    name: 'Configure Stealth',
    method: 'POST',
    path: '/stealth/configure',
    description: 'Configure stealth settings for crawling',
    defaultBody: {
      preset: 'Medium',
      user_agent_rotation: true,
      timing_jitter: true,
      fingerprint_evasion: true
    }
  },
  {
    id: 'stealth-test',
    category: 'Stealth',
    name: 'Test Stealth',
    method: 'POST',
    path: '/stealth/test',
    description: 'Test stealth configuration effectiveness',
    defaultBody: {
      url: 'https://bot-detection-test.com',
      preset: 'High'
    }
  },
  {
    id: 'stealth-capabilities',
    category: 'Stealth',
    name: 'Stealth Capabilities',
    method: 'GET',
    path: '/stealth/capabilities',
    description: 'Get available stealth capabilities and presets',
  },
  {
    id: 'stealth-health',
    category: 'Stealth',
    name: 'Stealth Health',
    method: 'GET',
    path: '/stealth/health',
    description: 'Check stealth features health status',
  },

  // ========================================
  // Table Extraction (2 endpoints)
  // ========================================
  {
    id: 'tables-extract',
    category: 'Tables',
    name: 'Extract Tables',
    method: 'POST',
    path: '/api/v1/tables/extract',
    description: 'Extract tables from HTML or URL',
    defaultBody: {
      url: 'https://example.com/data-page',
      format: 'json',
      include_headers: true
    }
  },
  {
    id: 'tables-export',
    category: 'Tables',
    name: 'Export Table',
    method: 'GET',
    path: '/api/v1/tables/:id/export',
    description: 'Export extracted table in various formats',
    parameters: {
      id: { type: 'string', required: true, description: 'Table ID' }
    }
  },

  // ========================================
  // LLM Provider Management (4 endpoints)
  // ========================================
  {
    id: 'llm-providers',
    category: 'LLM',
    name: 'List Providers',
    method: 'GET',
    path: '/api/v1/llm/providers',
    description: 'List available LLM providers and their status',
  },
  {
    id: 'llm-switch',
    category: 'LLM',
    name: 'Switch Provider',
    method: 'POST',
    path: '/api/v1/llm/providers/switch',
    description: 'Switch active LLM provider',
    defaultBody: {
      provider: 'openai',
      model: 'gpt-4'
    }
  },
  {
    id: 'llm-config-get',
    category: 'LLM',
    name: 'Get Configuration',
    method: 'GET',
    path: '/api/v1/llm/config',
    description: 'Get current LLM configuration',
  },
  {
    id: 'llm-config-update',
    category: 'LLM',
    name: 'Update Configuration',
    method: 'POST',
    path: '/api/v1/llm/config',
    description: 'Update LLM configuration',
    defaultBody: {
      temperature: 0.7,
      max_tokens: 2000,
      timeout: 30
    }
  },

  // ========================================
  // Strategies (2 endpoints)
  // ========================================
  {
    id: 'strategies-crawl',
    category: 'Strategies',
    name: 'Execute Strategy',
    method: 'POST',
    path: '/strategies/crawl',
    description: 'Execute crawl with specific extraction strategy',
    defaultBody: {
      url: 'https://example.com',
      strategy: 'full_extraction',
      options: {}
    }
  },
  {
    id: 'strategies-info',
    category: 'Strategies',
    name: 'Strategy Info',
    method: 'GET',
    path: '/strategies/info',
    description: 'Get available extraction strategies and their capabilities',
  },

  // ========================================
  // Spider (3 endpoints)
  // ========================================
  {
    id: 'spider-crawl',
    category: 'Spider',
    name: 'Start Spider',
    method: 'POST',
    path: '/spider/crawl',
    description: 'Start deep crawling from seed URL',
    defaultBody: {
      url: 'https://example.com',
      max_depth: 2,
      max_pages: 100,
      follow_external: false
    }
  },
  {
    id: 'spider-status',
    category: 'Spider',
    name: 'Spider Status',
    method: 'POST',
    path: '/spider/status',
    description: 'Get status of running spider crawl',
    defaultBody: {
      crawl_id: 'spider-12345'
    }
  },
  {
    id: 'spider-control',
    category: 'Spider',
    name: 'Spider Control',
    method: 'POST',
    path: '/spider/control',
    description: 'Control spider crawl (pause, resume, stop)',
    defaultBody: {
      crawl_id: 'spider-12345',
      action: 'pause'
    }
  },

  // ========================================
  // Session Management (12 endpoints)
  // ========================================
  {
    id: 'sessions-create',
    category: 'Sessions',
    name: 'Create Session',
    method: 'POST',
    path: '/sessions',
    description: 'Create a new session with custom configuration',
    defaultBody: {
      name: 'my-session',
      config: {
        user_agent: 'Mozilla/5.0...',
        cookies: [],
        ttl: 3600
      }
    }
  },
  {
    id: 'sessions-list',
    category: 'Sessions',
    name: 'List Sessions',
    method: 'GET',
    path: '/sessions',
    description: 'List all active sessions',
  },
  {
    id: 'sessions-stats',
    category: 'Sessions',
    name: 'Session Statistics',
    method: 'GET',
    path: '/sessions/stats',
    description: 'Get session statistics and usage metrics',
  },
  {
    id: 'sessions-cleanup',
    category: 'Sessions',
    name: 'Cleanup Sessions',
    method: 'POST',
    path: '/sessions/cleanup',
    description: 'Clean up expired sessions',
  },
  {
    id: 'sessions-get',
    category: 'Sessions',
    name: 'Get Session',
    method: 'GET',
    path: '/sessions/:session_id',
    description: 'Get session details by ID',
    parameters: {
      session_id: { type: 'string', required: true, description: 'Session ID' }
    }
  },
  {
    id: 'sessions-delete',
    category: 'Sessions',
    name: 'Delete Session',
    method: 'DELETE',
    path: '/sessions/:session_id',
    description: 'Delete session by ID',
    parameters: {
      session_id: { type: 'string', required: true, description: 'Session ID' }
    }
  },
  {
    id: 'sessions-extend',
    category: 'Sessions',
    name: 'Extend Session',
    method: 'POST',
    path: '/sessions/:session_id/extend',
    description: 'Extend session TTL',
    parameters: {
      session_id: { type: 'string', required: true, description: 'Session ID' }
    },
    defaultBody: {
      ttl: 3600
    }
  },
  {
    id: 'sessions-set-cookie',
    category: 'Sessions',
    name: 'Set Cookie',
    method: 'POST',
    path: '/sessions/:session_id/cookies',
    description: 'Set cookies for session',
    parameters: {
      session_id: { type: 'string', required: true, description: 'Session ID' }
    },
    defaultBody: {
      cookies: [
        {
          name: 'session_token',
          value: 'abc123',
          domain: 'example.com'
        }
      ]
    }
  },
  {
    id: 'sessions-clear-cookies',
    category: 'Sessions',
    name: 'Clear Cookies',
    method: 'DELETE',
    path: '/sessions/:session_id/cookies',
    description: 'Clear all cookies from session',
    parameters: {
      session_id: { type: 'string', required: true, description: 'Session ID' }
    }
  },
  {
    id: 'sessions-get-cookies-domain',
    category: 'Sessions',
    name: 'Get Domain Cookies',
    method: 'GET',
    path: '/sessions/:session_id/cookies/:domain',
    description: 'Get cookies for specific domain',
    parameters: {
      session_id: { type: 'string', required: true, description: 'Session ID' },
      domain: { type: 'string', required: true, description: 'Domain name' }
    }
  },
  {
    id: 'sessions-get-cookie',
    category: 'Sessions',
    name: 'Get Specific Cookie',
    method: 'GET',
    path: '/sessions/:session_id/cookies/:domain/:name',
    description: 'Get specific cookie by domain and name',
    parameters: {
      session_id: { type: 'string', required: true, description: 'Session ID' },
      domain: { type: 'string', required: true, description: 'Domain name' },
      name: { type: 'string', required: true, description: 'Cookie name' }
    }
  },
  {
    id: 'sessions-delete-cookie',
    category: 'Sessions',
    name: 'Delete Cookie',
    method: 'DELETE',
    path: '/sessions/:session_id/cookies/:domain/:name',
    description: 'Delete specific cookie',
    parameters: {
      session_id: { type: 'string', required: true, description: 'Session ID' },
      domain: { type: 'string', required: true, description: 'Domain name' },
      name: { type: 'string', required: true, description: 'Cookie name' }
    }
  },

  // ========================================
  // Workers (10 endpoints)
  // ========================================
  {
    id: 'workers-submit',
    category: 'Workers',
    name: 'Submit Job',
    method: 'POST',
    path: '/workers/jobs',
    description: 'Submit a new job to worker queue',
    defaultBody: {
      job_type: 'crawl',
      config: {
        url: 'https://example.com'
      },
      priority: 'normal'
    }
  },
  {
    id: 'workers-list',
    category: 'Workers',
    name: 'List Jobs',
    method: 'GET',
    path: '/workers/jobs',
    description: 'List all jobs in worker queue',
  },
  {
    id: 'workers-get-job',
    category: 'Workers',
    name: 'Get Job Status',
    method: 'GET',
    path: '/workers/jobs/:job_id',
    description: 'Get job status and progress',
    parameters: {
      job_id: { type: 'string', required: true, description: 'Job ID' }
    }
  },
  {
    id: 'workers-get-result',
    category: 'Workers',
    name: 'Get Job Result',
    method: 'GET',
    path: '/workers/jobs/:job_id/result',
    description: 'Get completed job result',
    parameters: {
      job_id: { type: 'string', required: true, description: 'Job ID' }
    }
  },
  {
    id: 'workers-queue-stats',
    category: 'Workers',
    name: 'Queue Statistics',
    method: 'GET',
    path: '/workers/stats/queue',
    description: 'Get worker queue statistics',
  },
  {
    id: 'workers-worker-stats',
    category: 'Workers',
    name: 'Worker Statistics',
    method: 'GET',
    path: '/workers/stats/workers',
    description: 'Get worker pool statistics',
  },
  {
    id: 'workers-metrics',
    category: 'Workers',
    name: 'Worker Metrics',
    method: 'GET',
    path: '/workers/metrics',
    description: 'Get detailed worker performance metrics',
  },
  {
    id: 'workers-schedule-create',
    category: 'Workers',
    name: 'Schedule Job',
    method: 'POST',
    path: '/workers/schedule',
    description: 'Create scheduled job with cron expression',
    defaultBody: {
      name: 'daily-crawl',
      job_type: 'crawl',
      schedule: '0 0 * * *',
      config: {}
    }
  },
  {
    id: 'workers-schedule-list',
    category: 'Workers',
    name: 'List Scheduled Jobs',
    method: 'GET',
    path: '/workers/schedule',
    description: 'List all scheduled jobs',
  },
  {
    id: 'workers-schedule-delete',
    category: 'Workers',
    name: 'Delete Scheduled Job',
    method: 'DELETE',
    path: '/workers/schedule/:job_id',
    description: 'Delete scheduled job',
    parameters: {
      job_id: { type: 'string', required: true, description: 'Scheduled Job ID' }
    }
  },

  // ========================================
  // Resources (6 endpoints)
  // ========================================
  {
    id: 'resources-status',
    category: 'Resources',
    name: 'Resource Status',
    method: 'GET',
    path: '/resources/status',
    description: 'Get overall resource utilization status',
  },
  {
    id: 'resources-browser-pool',
    category: 'Resources',
    name: 'Browser Pool Status',
    method: 'GET',
    path: '/resources/browser-pool',
    description: 'Get browser pool status and availability',
  },
  {
    id: 'resources-rate-limiter',
    category: 'Resources',
    name: 'Rate Limiter Status',
    method: 'GET',
    path: '/resources/rate-limiter',
    description: 'Get rate limiter statistics',
  },
  {
    id: 'resources-memory',
    category: 'Resources',
    name: 'Memory Status',
    method: 'GET',
    path: '/resources/memory',
    description: 'Get memory usage and pressure status',
  },
  {
    id: 'resources-performance',
    category: 'Resources',
    name: 'Performance Status',
    method: 'GET',
    path: '/resources/performance',
    description: 'Get performance metrics and degradation status',
  },
  {
    id: 'resources-pdf-semaphore',
    category: 'Resources',
    name: 'PDF Semaphore Status',
    method: 'GET',
    path: '/resources/pdf/semaphore',
    description: 'Get PDF processing semaphore status',
  },

  // ========================================
  // Fetch Metrics (1 endpoint)
  // ========================================
  {
    id: 'fetch-metrics',
    category: 'Fetch',
    name: 'Fetch Metrics',
    method: 'GET',
    path: '/fetch/metrics',
    description: 'Get fetch engine performance metrics',
  },

  // ========================================
  // Monitoring System (9 endpoints)
  // ========================================
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
    description: 'Get detailed performance metrics report',
  },
  {
    id: 'monitoring-current-metrics',
    category: 'Monitoring',
    name: 'Current Metrics',
    method: 'GET',
    path: '/monitoring/metrics/current',
    description: 'Get current real-time metrics',
  },
  {
    id: 'monitoring-alert-rules',
    category: 'Monitoring',
    name: 'Alert Rules',
    method: 'GET',
    path: '/monitoring/alerts/rules',
    description: 'Get configured alert rules',
  },
  {
    id: 'monitoring-active-alerts',
    category: 'Monitoring',
    name: 'Active Alerts',
    method: 'GET',
    path: '/monitoring/alerts/active',
    description: 'Get currently active alerts',
  },
  {
    id: 'monitoring-memory-profiling',
    category: 'Monitoring',
    name: 'Memory Profiling',
    method: 'GET',
    path: '/monitoring/profiling/memory',
    description: 'Get detailed memory profiling data',
  },
  {
    id: 'monitoring-leak-analysis',
    category: 'Monitoring',
    name: 'Leak Analysis',
    method: 'GET',
    path: '/monitoring/profiling/leaks',
    description: 'Analyze potential memory leaks',
  },
  {
    id: 'monitoring-allocations',
    category: 'Monitoring',
    name: 'Allocation Metrics',
    method: 'GET',
    path: '/monitoring/profiling/allocations',
    description: 'Get memory allocation metrics',
  },
  {
    id: 'monitoring-resource-status',
    category: 'Monitoring',
    name: 'Resource Management Status',
    method: 'GET',
    path: '/api/resources/status',
    description: 'Get resource management status',
  },

  // ========================================
  // Pipeline (1 endpoint)
  // ========================================
  {
    id: 'pipeline-phases',
    category: 'Pipeline',
    name: 'Pipeline Phases',
    method: 'GET',
    path: '/pipeline/phases',
    description: 'Get pipeline phase metrics and visualization data',
  },

  // ========================================
  // Telemetry (3 endpoints)
  // ========================================
  {
    id: 'telemetry-status',
    category: 'Telemetry',
    name: 'Telemetry Status',
    method: 'GET',
    path: '/api/telemetry/status',
    description: 'Get telemetry system status',
  },
  {
    id: 'telemetry-traces',
    category: 'Telemetry',
    name: 'List Traces',
    method: 'GET',
    path: '/api/telemetry/traces',
    description: 'List available traces',
  },
  {
    id: 'telemetry-trace-tree',
    category: 'Telemetry',
    name: 'Trace Tree',
    method: 'GET',
    path: '/api/telemetry/traces/:trace_id',
    description: 'Get trace tree visualization',
    parameters: {
      trace_id: { type: 'string', required: true, description: 'Trace ID' }
    }
  },
]
