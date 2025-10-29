import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest'
import { renderHook, act, waitFor } from '@testing-library/react'
import { useSSEStream, useNDJSONStream } from '@/hooks/useSSEStream'

describe('useSSEStream', () => {
  let mockEventSource

  beforeEach(() => {
    vi.useFakeTimers()
    mockEventSource = null
  })

  afterEach(() => {
    vi.runOnlyPendingTimers()
    vi.useRealTimers()
    if (mockEventSource) {
      mockEventSource.close()
    }
  })

  describe('Connection Management', () => {
    it('should initialize with disconnected state', () => {
      const { result } = renderHook(() => useSSEStream('/api/stream', { enabled: false }))

      expect(result.current.isConnected).toBe(false)
      expect(result.current.connectionStatus).toBe('disconnected')
      expect(result.current.data).toBeNull()
      expect(result.current.error).toBeNull()
    })

    it('should connect when enabled', async () => {
      const { result } = renderHook(() => useSSEStream('/api/stream'))

      await act(async () => {
        vi.runAllTimers()
      })

      expect(result.current.connectionStatus).toBe('connecting')
    })

    it('should handle successful connection', async () => {
      const onOpen = vi.fn()
      const { result } = renderHook(() =>
        useSSEStream('/api/stream', { onOpen })
      )

      await act(async () => {
        vi.runAllTimers()
      })

      // EventSource mock should trigger onopen
      await waitFor(() => {
        expect(result.current.connectionStatus).toBe('connected')
      })
    })

    it('should disconnect on unmount', () => {
      const onClose = vi.fn()
      const { unmount } = renderHook(() =>
        useSSEStream('/api/stream', { onClose })
      )

      unmount()

      expect(onClose).toHaveBeenCalled()
    })

    it('should support manual disconnect', async () => {
      const { result } = renderHook(() => useSSEStream('/api/stream'))

      await act(async () => {
        vi.runAllTimers()
      })

      act(() => {
        result.current.disconnect()
      })

      expect(result.current.connectionStatus).toBe('disconnected')
      expect(result.current.isConnected).toBe(false)
    })
  })

  describe('Message Handling', () => {
    it('should parse and handle JSON messages', async () => {
      const onMessage = vi.fn()
      const { result } = renderHook(() =>
        useSSEStream('/api/stream', { onMessage })
      )

      await act(async () => {
        vi.runAllTimers()
      })

      // Simulate receiving a message
      const testData = { type: 'progress', value: 50 }
      const mockES = new EventSource('/api/stream')

      act(() => {
        mockES._triggerMessage(testData)
      })

      await waitFor(() => {
        expect(result.current.data).toEqual(testData)
        expect(onMessage).toHaveBeenCalledWith(testData)
      })
    })

    it('should handle non-JSON messages', async () => {
      const onMessage = vi.fn()
      const { result } = renderHook(() =>
        useSSEStream('/api/stream', { onMessage })
      )

      await act(async () => {
        vi.runAllTimers()
      })

      const mockES = new EventSource('/api/stream')

      act(() => {
        if (mockES.onmessage) {
          mockES.onmessage({ data: 'plain text message' })
        }
      })

      await waitFor(() => {
        expect(result.current.data).toBe('plain text message')
      })
    })

    it('should handle multiple messages in sequence', async () => {
      const messages = []
      const onMessage = vi.fn((msg) => messages.push(msg))

      renderHook(() => useSSEStream('/api/stream', { onMessage }))

      await act(async () => {
        vi.runAllTimers()
      })

      const mockES = new EventSource('/api/stream')

      act(() => {
        mockES._triggerMessage({ count: 1 })
        mockES._triggerMessage({ count: 2 })
        mockES._triggerMessage({ count: 3 })
      })

      await waitFor(() => {
        expect(messages).toHaveLength(3)
        expect(messages[2]).toEqual({ count: 3 })
      })
    })
  })

  describe('Error Handling and Reconnection', () => {
    it('should handle connection errors', async () => {
      const onError = vi.fn()
      const { result } = renderHook(() =>
        useSSEStream('/api/stream', { onError, reconnect: false })
      )

      await act(async () => {
        vi.runAllTimers()
      })

      const mockES = new EventSource('/api/stream')

      act(() => {
        mockES._triggerError(new Error('Connection failed'))
      })

      await waitFor(() => {
        expect(result.current.connectionStatus).toBe('error')
        expect(result.current.isConnected).toBe(false)
        expect(onError).toHaveBeenCalled()
      })
    })

    it('should attempt to reconnect on error', async () => {
      const { result } = renderHook(() =>
        useSSEStream('/api/stream', {
          reconnect: true,
          maxReconnectAttempts: 3,
          reconnectInterval: 1000
        })
      )

      await act(async () => {
        vi.runAllTimers()
      })

      const mockES = new EventSource('/api/stream')

      act(() => {
        mockES._triggerError(new Error('Connection lost'))
      })

      await waitFor(() => {
        expect(result.current.connectionStatus).toBe('reconnecting')
      })

      act(() => {
        vi.advanceTimersByTime(1000)
      })

      await waitFor(() => {
        expect(result.current.connectionStatus).toBe('connecting')
      })
    })

    it('should stop reconnecting after max attempts', async () => {
      const { result } = renderHook(() =>
        useSSEStream('/api/stream', {
          reconnect: true,
          maxReconnectAttempts: 2,
          reconnectInterval: 500
        })
      )

      await act(async () => {
        vi.runAllTimers()
      })

      const mockES = new EventSource('/api/stream')

      // Trigger multiple errors
      for (let i = 0; i < 3; i++) {
        act(() => {
          mockES._triggerError(new Error('Connection lost'))
          vi.advanceTimersByTime(500)
        })
      }

      await waitFor(() => {
        expect(result.current.connectionStatus).toBe('failed')
      })
    })

    it('should support manual reconnection', async () => {
      const { result } = renderHook(() =>
        useSSEStream('/api/stream', { reconnect: false })
      )

      await act(async () => {
        vi.runAllTimers()
      })

      act(() => {
        result.current.disconnect()
      })

      expect(result.current.connectionStatus).toBe('disconnected')

      act(() => {
        result.current.reconnect()
      })

      await waitFor(() => {
        expect(result.current.connectionStatus).toBe('connecting')
      })
    })
  })

  describe('Configuration Options', () => {
    it('should respect enabled option', () => {
      const { result } = renderHook(() =>
        useSSEStream('/api/stream', { enabled: false })
      )

      expect(result.current.connectionStatus).toBe('disconnected')
    })

    it('should use custom reconnect interval', async () => {
      const { result } = renderHook(() =>
        useSSEStream('/api/stream', {
          reconnectInterval: 2000,
          maxReconnectAttempts: 1
        })
      )

      await act(async () => {
        vi.runAllTimers()
      })

      const mockES = new EventSource('/api/stream')

      act(() => {
        mockES._triggerError(new Error('Error'))
      })

      expect(result.current.connectionStatus).toBe('reconnecting')

      act(() => {
        vi.advanceTimersByTime(1000)
      })

      expect(result.current.connectionStatus).toBe('reconnecting')

      act(() => {
        vi.advanceTimersByTime(1000)
      })

      await waitFor(() => {
        expect(result.current.connectionStatus).toBe('connecting')
      })
    })
  })
})

describe('useNDJSONStream', () => {
  beforeEach(() => {
    global.fetch = vi.fn()
  })

  describe('Stream Initialization', () => {
    it('should initialize with correct state', () => {
      const { result } = renderHook(() =>
        useNDJSONStream('/api/stream', { enabled: false })
      )

      expect(result.current.data).toEqual([])
      expect(result.current.isStreaming).toBe(false)
      expect(result.current.isComplete).toBe(false)
      expect(result.current.error).toBeNull()
    })

    it('should start streaming when enabled', async () => {
      const mockReader = {
        read: vi.fn()
          .mockResolvedValueOnce({ done: false, value: new TextEncoder().encode('{"test": 1}\n') })
          .mockResolvedValueOnce({ done: true })
      }

      global.fetch.mockResolvedValueOnce({
        ok: true,
        body: { getReader: () => mockReader }
      })

      const { result } = renderHook(() => useNDJSONStream('/api/stream'))

      await waitFor(() => {
        expect(result.current.isStreaming).toBe(false)
        expect(result.current.isComplete).toBe(true)
      })
    })
  })

  describe('Data Processing', () => {
    it('should parse NDJSON data correctly', async () => {
      const testData = [
        { id: 1, message: 'First' },
        { id: 2, message: 'Second' },
        { id: 3, message: 'Third' }
      ]

      const ndjsonData = testData.map(d => JSON.stringify(d)).join('\n') + '\n'

      const mockReader = {
        read: vi.fn()
          .mockResolvedValueOnce({ done: false, value: new TextEncoder().encode(ndjsonData) })
          .mockResolvedValueOnce({ done: true })
      }

      global.fetch.mockResolvedValueOnce({
        ok: true,
        body: { getReader: () => mockReader }
      })

      const onChunk = vi.fn()
      const { result } = renderHook(() =>
        useNDJSONStream('/api/stream', { onChunk })
      )

      await waitFor(() => {
        expect(result.current.isComplete).toBe(true)
        expect(result.current.data).toHaveLength(3)
        expect(onChunk).toHaveBeenCalledTimes(3)
      })
    })

    it('should handle incomplete lines correctly', async () => {
      const mockReader = {
        read: vi.fn()
          .mockResolvedValueOnce({
            done: false,
            value: new TextEncoder().encode('{"test"')
          })
          .mockResolvedValueOnce({
            done: false,
            value: new TextEncoder().encode(': 1}\n')
          })
          .mockResolvedValueOnce({ done: true })
      }

      global.fetch.mockResolvedValueOnce({
        ok: true,
        body: { getReader: () => mockReader }
      })

      const { result } = renderHook(() => useNDJSONStream('/api/stream'))

      await waitFor(() => {
        expect(result.current.isComplete).toBe(true)
        expect(result.current.data).toHaveLength(1)
        expect(result.current.data[0]).toEqual({ test: 1 })
      })
    })
  })

  describe('Stream Control', () => {
    it('should support stopping the stream', async () => {
      const mockReader = {
        read: vi.fn().mockResolvedValue({ done: false, value: new TextEncoder().encode('{"test": 1}\n') })
      }

      global.fetch.mockResolvedValueOnce({
        ok: true,
        body: { getReader: () => mockReader }
      })

      const { result } = renderHook(() => useNDJSONStream('/api/stream'))

      await waitFor(() => {
        expect(result.current.isStreaming).toBe(true)
      })

      act(() => {
        result.current.stopStream()
      })

      expect(result.current.isStreaming).toBe(false)
    })
  })
})
