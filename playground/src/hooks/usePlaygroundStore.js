import { create } from 'zustand'
import axios from 'axios'

export const usePlaygroundStore = create((set, get) => ({
  selectedEndpoint: null,
  requestBody: '{}',
  pathParameters: {},
  response: null,
  responseHeaders: null,
  isLoading: false,
  error: null,

  setSelectedEndpoint: (endpoint) => set({ selectedEndpoint: endpoint, pathParameters: {} }),
  setRequestBody: (body) => set({ requestBody: body }),
  setPathParameters: (params) => set({ pathParameters: params }),
  setResponse: (response) => set({ response }),
  setResponseHeaders: (headers) => set({ responseHeaders: headers }),
  setIsLoading: (isLoading) => set({ isLoading }),
  setError: (error) => set({ error }),

  executeRequest: async () => {
    const { selectedEndpoint, requestBody, pathParameters } = get()

    if (!selectedEndpoint) {
      set({ error: 'No endpoint selected' })
      return
    }

    set({ isLoading: true, error: null })
    const startTime = Date.now()

    try {
      // Build URL with path parameters replaced
      let url = selectedEndpoint.path

      // Replace path parameters
      if (selectedEndpoint.parameters) {
        Object.entries(pathParameters).forEach(([key, value]) => {
          url = url.replace(`:${key}`, value || `:${key}`)
        })
      }

      const config = {
        method: selectedEndpoint.method,
        url: `/api${url}`,
        headers: {
          'Content-Type': 'application/json',
        },
      }

      if (selectedEndpoint.method !== 'GET' && requestBody) {
        try {
          config.data = JSON.parse(requestBody)
        } catch (e) {
          throw new Error('Invalid JSON in request body')
        }
      }

      const response = await axios(config)
      const duration = Date.now() - startTime

      set({
        response: {
          data: response.data,
          status: response.status,
          statusText: response.statusText,
          duration,
        },
        responseHeaders: response.headers,
        isLoading: false,
      })
    } catch (error) {
      const duration = Date.now() - startTime
      set({
        response: {
          data: error.response?.data || { error: error.message },
          status: error.response?.status || 500,
          statusText: error.response?.statusText || 'Error',
          duration,
        },
        responseHeaders: error.response?.headers || {},
        isLoading: false,
        error: error.message,
      })
    }
  },
}))
