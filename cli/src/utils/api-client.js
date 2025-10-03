/**
 * RipTide API Client
 * Wrapper around the RipTide API for CLI usage
 */

import axios from 'axios';
import { getConfig } from './config.js';

export class RipTideClient {
  constructor(options = {}) {
    const config = getConfig();

    this.baseURL = options.url || config.get('api-url') || 'http://localhost:8080';
    this.apiKey = options.apiKey || config.get('api-key');
    this.timeout = options.timeout || 30000;

    this.client = axios.create({
      baseURL: this.baseURL,
      timeout: this.timeout,
      headers: {
        'Content-Type': 'application/json',
        ...(this.apiKey && { 'Authorization': `Bearer ${this.apiKey}` })
      }
    });

    // Response interceptor for error handling
    this.client.interceptors.response.use(
      response => response.data,
      error => {
        if (error.response) {
          const message = error.response.data?.error || error.response.statusText;
          throw new Error(`API Error (${error.response.status}): ${message}`);
        } else if (error.request) {
          throw new Error(`Network Error: Unable to reach ${this.baseURL}`);
        } else {
          throw new Error(`Request Error: ${error.message}`);
        }
      }
    );
  }

  /**
   * Health check
   */
  async health() {
    return this.client.get('/healthz');
  }

  /**
   * Get Prometheus metrics
   */
  async metrics() {
    const response = await axios.get(`${this.baseURL}/metrics`);
    return response.data;
  }

  /**
   * Crawl URLs
   */
  async crawl(urls, options = {}) {
    return this.client.post('/crawl', {
      urls,
      options: {
        concurrency: options.concurrency || 3,
        cache_mode: options.cacheMode || 'auto',
        extract_mode: options.extractMode || 'article',
        ...(options.sessionId && { session_id: options.sessionId })
      }
    });
  }

  /**
   * Stream crawl results
   */
  async streamCrawl(urls, options = {}, onData) {
    const response = await axios({
      method: 'POST',
      url: `${this.baseURL}/crawl/stream`,
      data: {
        urls,
        options: {
          concurrency: options.concurrency || 3,
          cache_mode: options.cacheMode || 'auto'
        }
      },
      responseType: 'stream',
      timeout: 0 // No timeout for streams
    });

    return new Promise((resolve, reject) => {
      const results = [];

      response.data.on('data', (chunk) => {
        const lines = chunk.toString().split('\n').filter(Boolean);

        lines.forEach(line => {
          try {
            const data = JSON.parse(line);
            results.push(data);
            if (onData) onData(data);
          } catch (e) {
            // Skip invalid JSON
          }
        });
      });

      response.data.on('end', () => resolve(results));
      response.data.on('error', reject);
    });
  }

  /**
   * Deep search
   */
  async search(query, options = {}) {
    return this.client.post('/deepsearch', {
      query,
      limit: options.limit || 10,
      include_content: options.includeContent || false,
      crawl_options: options.crawlOptions || {}
    });
  }

  /**
   * Render URL with headless browser
   */
  async render(url, options = {}) {
    return this.client.post('/render', {
      url,
      wait_time: options.waitTime || 2000,
      screenshot: options.screenshot || false
    });
  }

  /**
   * List sessions
   */
  async listSessions() {
    return this.client.get('/sessions');
  }

  /**
   * Create session
   */
  async createSession(name, config = {}) {
    return this.client.post('/sessions', { name, config });
  }

  /**
   * Get session
   */
  async getSession(sessionId) {
    return this.client.get(`/sessions/${sessionId}`);
  }

  /**
   * Delete session
   */
  async deleteSession(sessionId) {
    return this.client.delete(`/sessions/${sessionId}`);
  }

  /**
   * Get worker status
   */
  async workerStatus() {
    return this.client.get('/workers/status');
  }

  /**
   * Get health score
   */
  async healthScore() {
    return this.client.get('/monitoring/health-score');
  }

  /**
   * Get performance report
   */
  async performanceReport() {
    return this.client.get('/monitoring/performance-report');
  }

  /**
   * Start spider
   */
  async startSpider(url, options = {}) {
    return this.client.post('/spider/start', {
      url,
      max_depth: options.maxDepth || 2,
      max_pages: options.maxPages || 10
    });
  }

  /**
   * Get extraction strategies
   */
  async getStrategies() {
    return this.client.get('/strategies/info');
  }
}
