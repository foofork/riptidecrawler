import { describe, it, expect } from 'vitest'

// Test validators for playground inputs
describe('Input Validators', () => {
  const isValidURL = (url) => {
    try {
      new URL(url)
      return true
    } catch {
      return false
    }
  }

  const isValidJSON = (str) => {
    try {
      JSON.parse(str)
      return true
    } catch {
      return false
    }
  }

  const isValidHTTPMethod = (method) => {
    const validMethods = ['GET', 'POST', 'PUT', 'DELETE', 'PATCH', 'HEAD', 'OPTIONS']
    return validMethods.includes(method?.toUpperCase())
  }

  describe('URL Validation', () => {
    it('should validate correct HTTP URLs', () => {
      expect(isValidURL('http://example.com')).toBe(true)
      expect(isValidURL('https://example.com')).toBe(true)
    })

    it('should validate URLs with paths', () => {
      expect(isValidURL('https://example.com/path/to/resource')).toBe(true)
    })

    it('should validate URLs with query parameters', () => {
      expect(isValidURL('https://example.com?param=value')).toBe(true)
    })

    it('should reject invalid URLs', () => {
      expect(isValidURL('not a url')).toBe(false)
      expect(isValidURL('')).toBe(false)
      expect(isValidURL('//invalid')).toBe(false)
    })
  })

  describe('JSON Validation', () => {
    it('should validate correct JSON', () => {
      expect(isValidJSON('{}')).toBe(true)
      expect(isValidJSON('{"key": "value"}')).toBe(true)
      expect(isValidJSON('[]')).toBe(true)
    })

    it('should validate JSON with nested objects', () => {
      expect(isValidJSON('{"nested": {"key": "value"}}')).toBe(true)
    })

    it('should reject invalid JSON', () => {
      expect(isValidJSON('not json')).toBe(false)
      expect(isValidJSON('{key: value}')).toBe(false)
      expect(isValidJSON('{invalid,}')).toBe(false)
    })
  })

  describe('HTTP Method Validation', () => {
    it('should validate standard HTTP methods', () => {
      expect(isValidHTTPMethod('GET')).toBe(true)
      expect(isValidHTTPMethod('POST')).toBe(true)
      expect(isValidHTTPMethod('PUT')).toBe(true)
      expect(isValidHTTPMethod('DELETE')).toBe(true)
    })

    it('should be case insensitive', () => {
      expect(isValidHTTPMethod('get')).toBe(true)
      expect(isValidHTTPMethod('Post')).toBe(true)
    })

    it('should reject invalid methods', () => {
      expect(isValidHTTPMethod('INVALID')).toBe(false)
      expect(isValidHTTPMethod('')).toBe(false)
    })
  })
})
