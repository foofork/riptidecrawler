import { useState, useEffect, useRef, useCallback } from 'react'

/**
 * Custom hook for Server-Sent Events (SSE) streaming
 * Connects to SSE endpoints and handles real-time data updates
 */
export function useSSEStream(url, options = {}) {
  const [data, setData] = useState(null)
  const [error, setError] = useState(null)
  const [isConnected, setIsConnected] = useState(false)
  const [connectionStatus, setConnectionStatus] = useState('disconnected')
  const eventSourceRef = useRef(null)
  const reconnectTimeoutRef = useRef(null)
  const reconnectAttemptsRef = useRef(0)

  const {
    enabled = true,
    reconnect = true,
    maxReconnectAttempts = 5,
    reconnectInterval = 3000,
    onMessage = null,
    onError = null,
    onOpen = null,
    onClose = null
  } = options

  const connect = useCallback(() => {
    if (!enabled || !url) return

    try {
      setConnectionStatus('connecting')
      setError(null)

      // Create EventSource connection
      const eventSource = new EventSource(url, {
        withCredentials: false
      })

      eventSource.onopen = (event) => {
        console.log('SSE connection opened:', url)
        setIsConnected(true)
        setConnectionStatus('connected')
        reconnectAttemptsRef.current = 0
        if (onOpen) onOpen(event)
      }

      eventSource.onmessage = (event) => {
        try {
          const parsedData = JSON.parse(event.data)
          setData(parsedData)
          if (onMessage) onMessage(parsedData)
        } catch (err) {
          console.error('Failed to parse SSE message:', err)
          setData(event.data) // Use raw data if JSON parsing fails
          if (onMessage) onMessage(event.data)
        }
      }

      eventSource.onerror = (err) => {
        console.error('SSE connection error:', err)
        setError(err)
        setIsConnected(false)
        setConnectionStatus('error')

        if (onError) onError(err)

        // Close the connection
        eventSource.close()

        // Attempt to reconnect
        if (reconnect && reconnectAttemptsRef.current < maxReconnectAttempts) {
          reconnectAttemptsRef.current += 1
          setConnectionStatus('reconnecting')

          reconnectTimeoutRef.current = setTimeout(() => {
            console.log(`Reconnecting... Attempt ${reconnectAttemptsRef.current}/${maxReconnectAttempts}`)
            connect()
          }, reconnectInterval)
        } else {
          setConnectionStatus('failed')
        }
      }

      eventSourceRef.current = eventSource
    } catch (err) {
      console.error('Failed to create SSE connection:', err)
      setError(err)
      setConnectionStatus('failed')
    }
  }, [url, enabled, reconnect, maxReconnectAttempts, reconnectInterval, onMessage, onError, onOpen])

  const disconnect = useCallback(() => {
    if (eventSourceRef.current) {
      eventSourceRef.current.close()
      eventSourceRef.current = null
      setIsConnected(false)
      setConnectionStatus('disconnected')
      if (onClose) onClose()
    }

    if (reconnectTimeoutRef.current) {
      clearTimeout(reconnectTimeoutRef.current)
      reconnectTimeoutRef.current = null
    }
  }, [onClose])

  const reconnectManually = useCallback(() => {
    disconnect()
    reconnectAttemptsRef.current = 0
    connect()
  }, [connect, disconnect])

  useEffect(() => {
    if (enabled) {
      connect()
    }

    return () => {
      disconnect()
    }
  }, [enabled, connect, disconnect])

  return {
    data,
    error,
    isConnected,
    connectionStatus,
    reconnect: reconnectManually,
    disconnect
  }
}

/**
 * Hook for NDJSON streaming (newline-delimited JSON)
 * Uses fetch with streaming response body
 */
export function useNDJSONStream(url, options = {}) {
  const [data, setData] = useState([])
  const [error, setError] = useState(null)
  const [isStreaming, setIsStreaming] = useState(false)
  const [isComplete, setIsComplete] = useState(false)
  const abortControllerRef = useRef(null)

  const {
    enabled = true,
    onChunk = null,
    onComplete = null,
    onError = null,
    requestBody = null
  } = options

  const startStream = useCallback(async () => {
    if (!enabled || !url) return

    try {
      setIsStreaming(true)
      setIsComplete(false)
      setError(null)
      setData([])

      abortControllerRef.current = new AbortController()

      const response = await fetch(url, {
        method: requestBody ? 'POST' : 'GET',
        headers: {
          'Content-Type': 'application/json',
          'Accept': 'application/x-ndjson'
        },
        body: requestBody ? JSON.stringify(requestBody) : undefined,
        signal: abortControllerRef.current.signal
      })

      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`)
      }

      const reader = response.body.getReader()
      const decoder = new TextDecoder()
      let buffer = ''

      while (true) {
        const { done, value } = await reader.read()

        if (done) {
          setIsComplete(true)
          setIsStreaming(false)
          if (onComplete) onComplete(data)
          break
        }

        buffer += decoder.decode(value, { stream: true })
        const lines = buffer.split('\n')
        buffer = lines.pop() || '' // Keep incomplete line in buffer

        for (const line of lines) {
          if (line.trim()) {
            try {
              const parsed = JSON.parse(line)
              setData(prev => [...prev, parsed])
              if (onChunk) onChunk(parsed)
            } catch (err) {
              console.error('Failed to parse NDJSON line:', err)
            }
          }
        }
      }
    } catch (err) {
      if (err.name !== 'AbortError') {
        console.error('NDJSON stream error:', err)
        setError(err)
        if (onError) onError(err)
      }
      setIsStreaming(false)
    }
  }, [url, enabled, requestBody, onChunk, onComplete, onError, data])

  const stopStream = useCallback(() => {
    if (abortControllerRef.current) {
      abortControllerRef.current.abort()
      abortControllerRef.current = null
      setIsStreaming(false)
    }
  }, [])

  useEffect(() => {
    if (enabled) {
      startStream()
    }

    return () => {
      stopStream()
    }
  }, [enabled, startStream, stopStream])

  return {
    data,
    error,
    isStreaming,
    isComplete,
    stopStream,
    restartStream: startStream
  }
}
