import { describe, it, expect, beforeEach, vi } from 'vitest'
import { renderHook, act, waitFor } from '@testing-library/react'
import { usePlaygroundStore } from '@/hooks/usePlaygroundStore'
import axios from 'axios'
import { mockEndpoints, mockCrawlResponse } from '../fixtures/mockData'

vi.mock('axios')

describe('usePlaygroundStore', () => {
  beforeEach(() => {
    // Reset store state
    const { result } = renderHook(() => usePlaygroundStore())
    act(() => {
      result.current.setSelectedEndpoint(null)
      result.current.setRequestBody('{}')
      result.current.setPathParameters({})
      result.current.setResponse(null)
      result.current.setError(null)
    })
    vi.clearAllMocks()
  })

  describe('Initial State', () => {
    it('should have correct initial state', () => {
      const { result } = renderHook(() => usePlaygroundStore())

      expect(result.current.selectedEndpoint).toBeNull()
      expect(result.current.requestBody).toBe('{}')
      expect(result.current.pathParameters).toEqual({})
      expect(result.current.response).toBeNull()
      expect(result.current.responseHeaders).toBeNull()
      expect(result.current.isLoading).toBe(false)
      expect(result.current.error).toBeNull()
    })
  })

  describe('State Setters', () => {
    it('should set selected endpoint', () => {
      const { result } = renderHook(() => usePlaygroundStore())

      act(() => {
        result.current.setSelectedEndpoint(mockEndpoints[0])
      })

      expect(result.current.selectedEndpoint).toEqual(mockEndpoints[0])
      expect(result.current.pathParameters).toEqual({})
    })

    it('should set request body', () => {
      const { result } = renderHook(() => usePlaygroundStore())
      const testBody = '{"test": "data"}'

      act(() => {
        result.current.setRequestBody(testBody)
      })

      expect(result.current.requestBody).toBe(testBody)
    })

    it('should set path parameters', () => {
      const { result } = renderHook(() => usePlaygroundStore())
      const params = { jobId: 'test-123' }

      act(() => {
        result.current.setPathParameters(params)
      })

      expect(result.current.pathParameters).toEqual(params)
    })

    it('should set response data', () => {
      const { result } = renderHook(() => usePlaygroundStore())
      const responseData = { success: true }

      act(() => {
        result.current.setResponse(responseData)
      })

      expect(result.current.response).toEqual(responseData)
    })

    it('should set error state', () => {
      const { result } = renderHook(() => usePlaygroundStore())
      const error = 'Test error'

      act(() => {
        result.current.setError(error)
      })

      expect(result.current.error).toBe(error)
    })
  })

  describe('executeRequest - POST Requests', () => {
    it('should execute POST request successfully', async () => {
      const { result } = renderHook(() => usePlaygroundStore())

      axios.mockResolvedValueOnce({
        data: mockCrawlResponse,
        status: 200,
        statusText: 'OK',
        headers: { 'content-type': 'application/json' }
      })

      act(() => {
        result.current.setSelectedEndpoint(mockEndpoints[0])
        result.current.setRequestBody(JSON.stringify(mockEndpoints[0].defaultBody))
      })

      await act(async () => {
        await result.current.executeRequest()
      })

      expect(result.current.isLoading).toBe(false)
      expect(result.current.response.data).toEqual(mockCrawlResponse)
      expect(result.current.response.status).toBe(200)
      expect(result.current.error).toBeNull()
    })

    it('should handle invalid JSON in request body', async () => {
      const { result } = renderHook(() => usePlaygroundStore())

      act(() => {
        result.current.setSelectedEndpoint(mockEndpoints[0])
        result.current.setRequestBody('invalid json')
      })

      await act(async () => {
        await result.current.executeRequest()
      })

      expect(result.current.error).toBe('Invalid JSON in request body')
    })

    it('should measure request duration', async () => {
      const { result } = renderHook(() => usePlaygroundStore())

      axios.mockImplementation(() =>
        new Promise(resolve => setTimeout(() => resolve({
          data: mockCrawlResponse,
          status: 200,
          statusText: 'OK',
          headers: {}
        }), 100))
      )

      act(() => {
        result.current.setSelectedEndpoint(mockEndpoints[0])
      })

      await act(async () => {
        await result.current.executeRequest()
      })

      expect(result.current.response.duration).toBeGreaterThanOrEqual(100)
    })
  })

  describe('executeRequest - GET Requests', () => {
    it('should execute GET request with path parameters', async () => {
      const { result } = renderHook(() => usePlaygroundStore())

      axios.mockResolvedValueOnce({
        data: { status: 'completed' },
        status: 200,
        statusText: 'OK',
        headers: {}
      })

      act(() => {
        result.current.setSelectedEndpoint(mockEndpoints[1])
        result.current.setPathParameters({ jobId: 'test-job-123' })
      })

      await act(async () => {
        await result.current.executeRequest()
      })

      expect(axios).toHaveBeenCalledWith(
        expect.objectContaining({
          method: 'GET',
          url: '/api/job/test-job-123'
        })
      )
    })

    it('should not include body in GET request', async () => {
      const { result } = renderHook(() => usePlaygroundStore())

      axios.mockResolvedValueOnce({
        data: {},
        status: 200,
        statusText: 'OK',
        headers: {}
      })

      act(() => {
        result.current.setSelectedEndpoint(mockEndpoints[1])
      })

      await act(async () => {
        await result.current.executeRequest()
      })

      expect(axios).toHaveBeenCalledWith(
        expect.not.objectContaining({
          data: expect.anything()
        })
      )
    })
  })

  describe('Error Handling', () => {
    it('should handle network errors', async () => {
      const { result } = renderHook(() => usePlaygroundStore())

      axios.mockRejectedValueOnce(new Error('Network error'))

      act(() => {
        result.current.setSelectedEndpoint(mockEndpoints[0])
      })

      await act(async () => {
        await result.current.executeRequest()
      })

      expect(result.current.error).toBe('Network error')
      expect(result.current.response.status).toBe(500)
    })

    it('should handle API errors with status code', async () => {
      const { result } = renderHook(() => usePlaygroundStore())

      axios.mockRejectedValueOnce({
        response: {
          data: { error: 'Bad request' },
          status: 400,
          statusText: 'Bad Request',
          headers: {}
        },
        message: 'Request failed'
      })

      act(() => {
        result.current.setSelectedEndpoint(mockEndpoints[0])
      })

      await act(async () => {
        await result.current.executeRequest()
      })

      expect(result.current.response.status).toBe(400)
      expect(result.current.response.data).toEqual({ error: 'Bad request' })
    })

    it('should handle no endpoint selected error', async () => {
      const { result } = renderHook(() => usePlaygroundStore())

      await act(async () => {
        await result.current.executeRequest()
      })

      expect(result.current.error).toBe('No endpoint selected')
    })
  })
})
