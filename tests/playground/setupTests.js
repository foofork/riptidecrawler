import '@testing-library/jest-dom'
import { expect, afterEach, beforeAll, afterAll, vi } from 'vitest'
import { cleanup } from '@testing-library/react'
import { setupServer } from 'msw/node'
import { http, HttpResponse } from 'msw'

// Setup MSW server
export const handlers = [
  // Mock API endpoints
  http.post('/api/crawl', () => {
    return HttpResponse.json({
      jobId: 'test-job-123',
      status: 'processing',
      message: 'Crawl job started successfully'
    })
  }),

  http.get('/api/job/:jobId', ({ params }) => {
    return HttpResponse.json({
      jobId: params.jobId,
      status: 'completed',
      results: {
        pages: 5,
        links: 25,
        duration: 1234
      }
    })
  }),

  http.get('/api/health', () => {
    return HttpResponse.json({
      status: 'healthy',
      version: '1.0.0',
      uptime: 12345
    })
  }),

  http.post('/api/extract', () => {
    return HttpResponse.json({
      extractionId: 'extract-456',
      data: {
        title: 'Test Page',
        content: 'Test content'
      }
    })
  })
]

export const server = setupServer(...handlers)

// Start server before all tests
beforeAll(() => {
  server.listen({ onUnhandledRequest: 'warn' })
})

// Reset handlers after each test
afterEach(() => {
  cleanup()
  server.resetHandlers()
})

// Close server after all tests
afterAll(() => {
  server.close()
})

// Mock EventSource for SSE tests
global.EventSource = class EventSource {
  constructor(url) {
    this.url = url
    this.readyState = 0
    this.onopen = null
    this.onmessage = null
    this.onerror = null

    // Simulate connection opening
    setTimeout(() => {
      this.readyState = 1
      if (this.onopen) this.onopen(new Event('open'))
    }, 0)
  }

  close() {
    this.readyState = 2
  }

  // Helper method for tests to trigger events
  _triggerMessage(data) {
    if (this.onmessage) {
      this.onmessage({ data: JSON.stringify(data) })
    }
  }

  _triggerError(error) {
    if (this.onerror) {
      this.onerror(error)
    }
  }
}

// Mock window.matchMedia
Object.defineProperty(window, 'matchMedia', {
  writable: true,
  value: vi.fn().mockImplementation(query => ({
    matches: false,
    media: query,
    onchange: null,
    addListener: vi.fn(),
    removeListener: vi.fn(),
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
    dispatchEvent: vi.fn(),
  })),
})

// Mock IntersectionObserver
global.IntersectionObserver = class IntersectionObserver {
  constructor() {}
  disconnect() {}
  observe() {}
  takeRecords() {
    return []
  }
  unobserve() {}
}
