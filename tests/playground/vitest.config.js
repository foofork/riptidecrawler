import { defineConfig } from 'vitest/config'
import react from '@vitejs/plugin-react'
import path from 'path'

export default defineConfig({
  plugins: [react()],
  test: {
    globals: true,
    environment: 'jsdom',
    setupFiles: ['./setupTests.js'],
    coverage: {
      provider: 'v8',
      reporter: ['text', 'json', 'html', 'lcov'],
      exclude: [
        'node_modules/',
        'tests/',
        '**/*.test.{js,jsx}',
        '**/*.spec.{js,jsx}',
        '**/setupTests.js',
        '**/vitest.config.js',
        '**/playwright.config.js'
      ],
      statements: 82,
      branches: 75,
      functions: 80,
      lines: 82
    },
    include: ['**/*.test.{js,jsx}'],
    testTimeout: 10000
  },
  resolve: {
    alias: {
      '@': path.resolve(__dirname, '../../playground/src')
    }
  }
})
