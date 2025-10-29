import { describe, it, expect, beforeEach, vi } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import ResponseViewer from '@/components/ResponseViewer'
import { usePlaygroundStore } from '@/hooks/usePlaygroundStore'
import { mockCrawlResponse, mockJobStatus } from '../fixtures/mockData'

vi.mock('@/hooks/usePlaygroundStore')

describe('ResponseViewer Component', () => {
  let mockStore

  beforeEach(() => {
    mockStore = {
      response: null,
      responseHeaders: null,
      selectedEndpoint: null,
      requestBody: '{}',
      isLoading: false
    }
    usePlaygroundStore.mockReturnValue(mockStore)
  })

  describe('Loading State', () => {
    it('should show loading spinner when loading', () => {
      mockStore.isLoading = true
      render(<ResponseViewer activeTab="response" />)

      expect(screen.getByRole('status', { hidden: true })).toBeInTheDocument()
    })
  })

  describe('No Response State', () => {
    it('should show empty state when no response', () => {
      render(<ResponseViewer activeTab="response" />)
      expect(screen.getByText('Execute a request to see the response')).toBeInTheDocument()
    })

    it('should not show empty state on code tab', () => {
      render(<ResponseViewer activeTab="code" />)
      expect(screen.queryByText('Execute a request to see the response')).not.toBeInTheDocument()
    })
  })

  describe('Response Tab', () => {
    beforeEach(() => {
      mockStore.response = {
        data: mockCrawlResponse,
        status: 200,
        statusText: 'OK',
        duration: 1234
      }
    })

    it('should display success status badge', () => {
      render(<ResponseViewer activeTab="response" />)
      expect(screen.getByText(/200 OK/i)).toBeInTheDocument()
    })

    it('should display response duration', () => {
      render(<ResponseViewer activeTab="response" />)
      expect(screen.getByText('1234ms')).toBeInTheDocument()
    })

    it('should show green badge for 2xx status codes', () => {
      render(<ResponseViewer activeTab="response" />)
      const badge = screen.getByText(/200 OK/i).parentElement
      expect(badge).toHaveClass('bg-green-100', 'text-green-800')
    })

    it('should show red badge for 4xx/5xx status codes', () => {
      mockStore.response.status = 404
      mockStore.response.statusText = 'Not Found'
      render(<ResponseViewer activeTab="response" />)

      const badge = screen.getByText(/404 Not Found/i).parentElement
      expect(badge).toHaveClass('bg-red-100', 'text-red-800')
    })

    it('should show yellow badge for 3xx status codes', () => {
      mockStore.response.status = 301
      mockStore.response.statusText = 'Moved Permanently'
      render(<ResponseViewer activeTab="response" />)

      const badge = screen.getByText(/301/i).parentElement
      expect(badge).toHaveClass('bg-yellow-100', 'text-yellow-800')
    })
  })

  describe('Headers Tab', () => {
    beforeEach(() => {
      mockStore.responseHeaders = {
        'content-type': 'application/json',
        'x-request-id': '12345'
      }
    })

    it('should display response headers', () => {
      render(<ResponseViewer activeTab="headers" />)
      // CodeMirror renders the content, check that component renders
      expect(screen.getByRole('textbox', { hidden: true })).toBeInTheDocument()
    })
  })

  describe('Code Tab', () => {
    beforeEach(() => {
      mockStore.selectedEndpoint = {
        id: 'test-endpoint',
        method: 'POST',
        path: '/test'
      }
    })

    it('should render language selector buttons', () => {
      render(<ResponseViewer activeTab="code" />)

      expect(screen.getByText('Javascript')).toBeInTheDocument()
      expect(screen.getByText('Python')).toBeInTheDocument()
      expect(screen.getByText('Curl')).toBeInTheDocument()
      expect(screen.getByText('Rust')).toBeInTheDocument()
    })

    it('should highlight selected language', () => {
      render(<ResponseViewer activeTab="code" />)

      const jsButton = screen.getByText('Javascript')
      expect(jsButton).toHaveClass('bg-riptide-blue', 'text-white')
    })

    it('should switch language when button clicked', () => {
      render(<ResponseViewer activeTab="code" />)

      const pythonButton = screen.getByText('Python')
      fireEvent.click(pythonButton)

      expect(pythonButton).toHaveClass('bg-riptide-blue', 'text-white')
    })
  })
})
