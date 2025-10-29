export const mockEndpoints = [
  {
    id: 'crawl-start',
    method: 'POST',
    path: '/crawl',
    title: 'Start Crawl',
    description: 'Initiate a new web crawling job',
    category: 'Crawling',
    defaultBody: {
      url: 'https://example.com',
      options: {
        depth: 2,
        followLinks: true
      }
    }
  },
  {
    id: 'job-status',
    method: 'GET',
    path: '/job/:jobId',
    title: 'Get Job Status',
    description: 'Retrieve status of a crawling job',
    category: 'Jobs',
    parameters: {
      jobId: {
        required: true,
        description: 'Unique job identifier'
      }
    }
  },
  {
    id: 'extract-data',
    method: 'POST',
    path: '/extract',
    title: 'Extract Data',
    description: 'Extract structured data from HTML',
    category: 'Extraction',
    defaultBody: {
      html: '<html><body>Test</body></html>',
      selectors: {
        title: 'h1'
      }
    }
  }
]

export const mockCrawlResponse = {
  jobId: 'job-12345',
  status: 'processing',
  message: 'Crawl started successfully',
  estimatedDuration: 5000
}

export const mockJobStatus = {
  jobId: 'job-12345',
  status: 'completed',
  progress: 100,
  results: {
    pages: 10,
    links: 45,
    errors: 0,
    duration: 4523
  },
  startedAt: new Date().toISOString(),
  completedAt: new Date().toISOString()
}

export const mockSSEEvents = [
  { type: 'progress', data: { progress: 25, message: 'Crawling page 1/4' } },
  { type: 'progress', data: { progress: 50, message: 'Crawling page 2/4' } },
  { type: 'progress', data: { progress: 75, message: 'Crawling page 3/4' } },
  { type: 'complete', data: { progress: 100, message: 'Crawl completed' } }
]

export const mockExtractionResult = {
  extractionId: 'extract-789',
  data: {
    title: 'Example Page',
    description: 'This is an example',
    links: ['https://example.com/page1', 'https://example.com/page2']
  },
  metadata: {
    extractedAt: new Date().toISOString(),
    duration: 125
  }
}
