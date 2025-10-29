import { describe, it, expect, beforeEach, vi } from 'vitest'
import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import RequestBuilder from '@/components/RequestBuilder'
import { usePlaygroundStore } from '@/hooks/usePlaygroundStore'
import { mockEndpoints } from '../fixtures/mockData'

vi.mock('@/hooks/usePlaygroundStore')

describe('RequestBuilder Component', () => {
  let mockStore

  beforeEach(() => {
    mockStore = {
      selectedEndpoint: null,
      requestBody: '{}',
      pathParameters: {},
      setRequestBody: vi.fn(),
      setPathParameters: vi.fn()
    }
    usePlaygroundStore.mockReturnValue(mockStore)
  })

  describe('No Endpoint Selected', () => {
    it('should show placeholder when no endpoint selected', () => {
      render(<RequestBuilder />)
      expect(screen.getByText('Select an endpoint to get started')).toBeInTheDocument()
    })
  })

  describe('POST Endpoint', () => {
    beforeEach(() => {
      mockStore.selectedEndpoint = mockEndpoints[0] // POST /crawl
      mockStore.requestBody = JSON.stringify(mockEndpoints[0].defaultBody, null, 2)
    })

    it('should render request body editor for POST requests', () => {
      render(<RequestBuilder />)
      expect(screen.getByText('Request Body (JSON)')).toBeInTheDocument()
    })

    it('should populate default request body', () => {
      render(<RequestBuilder />)
      expect(mockStore.setRequestBody).toHaveBeenCalledWith(
        expect.stringContaining('"url"')
      )
    })

    it('should show syntax error help text', () => {
      render(<RequestBuilder />)
      expect(screen.getByText(/Syntax errors will be highlighted/i)).toBeInTheDocument()
    })

    it('should not show GET message for POST requests', () => {
      render(<RequestBuilder />)
      expect(screen.queryByText(/This is a GET request/i)).not.toBeInTheDocument()
    })
  })

  describe('GET Endpoint', () => {
    beforeEach(() => {
      mockStore.selectedEndpoint = mockEndpoints[1] // GET /job/:jobId
    })

    it('should show GET message instead of body editor', () => {
      render(<RequestBuilder />)
      expect(screen.getByText(/This is a GET request/i)).toBeInTheDocument()
      expect(screen.queryByText('Request Body (JSON)')).not.toBeInTheDocument()
    })

    it('should show no request body required message', () => {
      render(<RequestBuilder />)
      expect(screen.getByText(/No request body required/i)).toBeInTheDocument()
    })
  })

  describe('Path Parameters', () => {
    beforeEach(() => {
      mockStore.selectedEndpoint = mockEndpoints[1] // Has jobId parameter
      mockStore.pathParameters = {}
    })

    it('should render path parameter inputs', () => {
      render(<RequestBuilder />)
      expect(screen.getByText('Path Parameters')).toBeInTheDocument()
      expect(screen.getByPlaceholderText(/Enter jobId/i)).toBeInTheDocument()
    })

    it('should show required indicator for required parameters', () => {
      render(<RequestBuilder />)
      expect(screen.getByText('*')).toBeInTheDocument()
    })

    it('should handle path parameter changes', async () => {
      const user = userEvent.setup()
      render(<RequestBuilder />)

      const input = screen.getByPlaceholderText(/Enter jobId/i)
      await user.type(input, 'test-123')

      expect(mockStore.setPathParameters).toHaveBeenCalled()
    })

    it('should show parameter description', () => {
      render(<RequestBuilder />)
      expect(screen.getByText(/Unique job identifier/i)).toBeInTheDocument()
    })

    it('should not show path parameters section when none exist', () => {
      mockStore.selectedEndpoint = mockEndpoints[0] // No path params
      render(<RequestBuilder />)
      expect(screen.queryByText('Path Parameters')).not.toBeInTheDocument()
    })
  })
})
