/**
 * Environment configuration
 * Uses Vite's environment variables with sensible defaults
 */

export const config = {
  api: {
    baseUrl: import.meta.env.VITE_API_BASE_URL || 'http://localhost:8080',
    timeout: parseInt(import.meta.env.VITE_API_TIMEOUT) || 30000,
  },
  features: {
    enableRequestHistory: import.meta.env.VITE_ENABLE_REQUEST_HISTORY !== 'false',
    maxHistoryItems: parseInt(import.meta.env.VITE_MAX_HISTORY_ITEMS) || 10,
  },
  dev: {
    enableDebug: import.meta.env.VITE_ENABLE_DEBUG === 'true',
  }
}

export default config
