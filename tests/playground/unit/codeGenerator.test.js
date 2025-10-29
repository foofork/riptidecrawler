import { describe, it, expect } from 'vitest'
import { generateCode } from '@/utils/codeGenerator'
import { mockEndpoints } from '../fixtures/mockData'

describe('codeGenerator', () => {
  describe('No Endpoint Selected', () => {
    it('should return placeholder when no endpoint is selected', () => {
      const result = generateCode(null, '{}', 'javascript')
      expect(result).toBe('// Select an endpoint to generate code')
    })
  })

  describe('JavaScript Generation', () => {
    it('should generate JavaScript for GET requests', () => {
      const endpoint = mockEndpoints[1] // GET endpoint
      const code = generateCode(endpoint, '{}', 'javascript')

      expect(code).toContain('fetch')
      expect(code).toContain('GET')
      expect(code).toContain(endpoint.path)
      expect(code).toContain('axios')
      expect(code).not.toContain('body:')
    })

    it('should generate JavaScript for POST requests', () => {
      const endpoint = mockEndpoints[0] // POST endpoint
      const body = JSON.stringify({ url: 'https://test.com' }, null, 2)
      const code = generateCode(endpoint, body, 'javascript')

      expect(code).toContain('fetch')
      expect(code).toContain('POST')
      expect(code).toContain(endpoint.path)
      expect(code).toContain('body: JSON.stringify')
      expect(code).toContain('axios.post')
    })

    it('should include request body in JavaScript code', () => {
      const endpoint = mockEndpoints[2]
      const body = JSON.stringify({ test: 'data' })
      const code = generateCode(endpoint, body, 'javascript')

      expect(code).toContain(body)
    })
  })

  describe('Python Generation', () => {
    it('should generate Python for GET requests', () => {
      const endpoint = mockEndpoints[1]
      const code = generateCode(endpoint, '{}', 'python')

      expect(code).toContain('import requests')
      expect(code).toContain('requests.get')
      expect(code).toContain(endpoint.path)
      expect(code).toContain('from riptide import RipTide')
    })

    it('should generate Python for POST requests', () => {
      const endpoint = mockEndpoints[0]
      const body = JSON.stringify({ url: 'https://test.com' })
      const code = generateCode(endpoint, body, 'python')

      expect(code).toContain('requests.post')
      expect(code).toContain("headers={'Content-Type': 'application/json'}")
      expect(code).toContain('json=')
    })

    it('should include SDK usage in Python code', () => {
      const endpoint = mockEndpoints[0]
      const code = generateCode(endpoint, '{}', 'python')

      expect(code).toContain('from riptide import RipTide')
      expect(code).toContain('client = RipTide')
    })
  })

  describe('cURL Generation', () => {
    it('should generate cURL for GET requests', () => {
      const endpoint = mockEndpoints[1]
      const code = generateCode(endpoint, '{}', 'curl')

      expect(code).toContain('curl -X GET')
      expect(code).toContain(endpoint.path)
      expect(code).toContain('jq')
      expect(code).not.toContain('-d')
    })

    it('should generate cURL for POST requests', () => {
      const endpoint = mockEndpoints[0]
      const body = '{"url": "https://test.com"}'
      const code = generateCode(endpoint, body, 'curl')

      expect(code).toContain('curl -X POST')
      expect(code).toContain("-H 'Content-Type: application/json'")
      expect(code).toContain("-d '")
      expect(code).toContain('jq')
    })

    it('should escape body for cURL', () => {
      const endpoint = mockEndpoints[0]
      const body = JSON.stringify({ test: 'data with spaces' }, null, 2)
      const code = generateCode(endpoint, body, 'curl')

      // Should remove newlines and extra spaces
      expect(code).not.toContain('\n  ')
      expect(code).toContain('-d')
    })
  })

  describe('Rust Generation', () => {
    it('should generate Rust for GET requests', () => {
      const endpoint = mockEndpoints[1]
      const code = generateCode(endpoint, '{}', 'rust')

      expect(code).toContain('use reqwest')
      expect(code).toContain('reqwest::get')
      expect(code).toContain('#[tokio::main]')
      expect(code).toContain('async fn main')
      expect(code).toContain(endpoint.path)
    })

    it('should generate Rust for POST requests', () => {
      const endpoint = mockEndpoints[0]
      const body = '{"url": "https://test.com"}'
      const code = generateCode(endpoint, body, 'rust')

      expect(code).toContain('use serde_json::json')
      expect(code).toContain('reqwest::Client::new()')
      expect(code).toContain('.post')
      expect(code).toContain('.json(&json!')
    })

    it('should include proper error handling in Rust', () => {
      const endpoint = mockEndpoints[0]
      const code = generateCode(endpoint, '{}', 'rust')

      expect(code).toContain('Result<(), Box<dyn std::error::Error>>')
      expect(code).toContain('.await?')
    })
  })

  describe('URL Construction', () => {
    it('should use correct base URL', () => {
      const endpoint = mockEndpoints[0]
      const code = generateCode(endpoint, '{}', 'javascript')

      expect(code).toContain('http://localhost:8080')
    })

    it('should construct full URL correctly', () => {
      const endpoint = mockEndpoints[1]
      const code = generateCode(endpoint, '{}', 'curl')

      expect(code).toContain('http://localhost:8080/job/:jobId')
    })
  })

  describe('Unsupported Language', () => {
    it('should return empty string for unsupported language', () => {
      const endpoint = mockEndpoints[0]
      const code = generateCode(endpoint, '{}', 'unsupported')

      expect(code).toBe('')
    })
  })
})
