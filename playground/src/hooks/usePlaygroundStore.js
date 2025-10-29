import { create } from 'zustand'
import { persist } from 'zustand/middleware'
import axios from 'axios'
import { config } from '../config/environment'

export const usePlaygroundStore = create(
  persist(
    (set, get) => ({
      selectedEndpoint: null,
      requestBody: '{}',
      pathParameters: {},
      response: null,
      responseHeaders: null,
      isLoading: false,
      error: null,
      requestHistory: [],

      setSelectedEndpoint: (endpoint) => set({ selectedEndpoint: endpoint, pathParameters: {}, error: null }),
      setRequestBody: (body) => set({ requestBody: body }),
      setPathParameters: (params) => set({ pathParameters: params }),
      setResponse: (response) => set({ response }),
      setResponseHeaders: (headers) => set({ responseHeaders: headers }),
      setIsLoading: (isLoading) => set({ isLoading }),
      setError: (error) => set({ error }),

      addToHistory: (item) => {
        const history = get().requestHistory
        const maxItems = config.features.maxHistoryItems
        const newHistory = [item, ...history].slice(0, maxItems)
        set({ requestHistory: newHistory })
      },

      loadFromHistory: (item) => {
        set({
          selectedEndpoint: item.endpoint,
          requestBody: item.requestBody,
          pathParameters: item.pathParameters || {},
          error: null
        })
      },

      clearHistory: () => set({ requestHistory: [] }),

      executeRequest: async () => {
        const { selectedEndpoint, requestBody, pathParameters, addToHistory } = get()

        if (!selectedEndpoint) {
          set({ error: 'No endpoint selected. Please select an endpoint first.' })
          return
        }

        set({ isLoading: true, error: null })
        const startTime = Date.now()

        try {
          // Validate path parameters
          if (selectedEndpoint.parameters) {
            const missingParams = Object.entries(selectedEndpoint.parameters)
              .filter(([key, info]) => info.required && !pathParameters[key])
              .map(([key]) => key)

            if (missingParams.length > 0) {
              throw new Error(`Missing required parameters: ${missingParams.join(', ')}`)
            }
          }

          // Build URL with path parameters replaced
          let url = selectedEndpoint.path

          // Replace path parameters
          if (selectedEndpoint.parameters) {
            Object.entries(pathParameters).forEach(([key, value]) => {
              url = url.replace(`:${key}`, value || `:${key}`)
            })
          }

          const axiosConfig = {
            method: selectedEndpoint.method,
            url: `/api${url}`,
            headers: {
              'Content-Type': 'application/json',
            },
            timeout: config.api.timeout,
          }

          if (selectedEndpoint.method !== 'GET' && selectedEndpoint.method !== 'DELETE' && requestBody) {
            try {
              axiosConfig.data = JSON.parse(requestBody)
            } catch (e) {
              throw new Error('Invalid JSON in request body. Please check your JSON syntax.')
            }
          }

          const response = await axios(axiosConfig)
          const duration = Date.now() - startTime

          const responseData = {
            data: response.data,
            status: response.status,
            statusText: response.statusText,
            duration,
          }

          set({
            response: responseData,
            responseHeaders: response.headers,
            isLoading: false,
            error: null,
          })

          // Add to history
          if (config.features.enableRequestHistory) {
            addToHistory({
              endpoint: selectedEndpoint,
              requestBody,
              pathParameters,
              response: responseData,
              timestamp: Date.now(),
              method: selectedEndpoint.method,
            })
          }
        } catch (error) {
          const duration = Date.now() - startTime
          let errorMessage = error.message

          // Provide actionable error messages
          if (error.code === 'ECONNABORTED') {
            errorMessage = 'Request timeout. The API took too long to respond. Try again or check if the server is running.'
          } else if (error.code === 'ERR_NETWORK') {
            errorMessage = 'Network error. Cannot connect to the API. Please verify the API is running at ' + config.api.baseUrl
          } else if (error.response?.status === 404) {
            errorMessage = 'Endpoint not found (404). The requested resource does not exist.'
          } else if (error.response?.status === 401) {
            errorMessage = 'Unauthorized (401). Please check your authentication credentials.'
          } else if (error.response?.status === 500) {
            errorMessage = 'Server error (500). The API encountered an internal error.'
          }

          const responseData = {
            data: error.response?.data || { error: error.message },
            status: error.response?.status || 500,
            statusText: error.response?.statusText || 'Error',
            duration,
          }

          set({
            response: responseData,
            responseHeaders: error.response?.headers || {},
            isLoading: false,
            error: errorMessage,
          })

          // Add failed request to history
          if (config.features.enableRequestHistory) {
            addToHistory({
              endpoint: selectedEndpoint,
              requestBody,
              pathParameters,
              response: responseData,
              timestamp: Date.now(),
              method: selectedEndpoint.method,
              error: errorMessage,
            })
          }
        }
      },
    }),
    {
      name: 'riptide-playground-storage',
      partialize: (state) => ({
        requestHistory: state.requestHistory,
      }),
    }
  )
)
