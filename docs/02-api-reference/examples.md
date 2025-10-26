# Comprehensive Code Examples

## Overview

This guide provides complete, production-ready code examples for all RipTide API endpoints and features. Examples are provided in multiple programming languages and cover common use cases, error handling, and optimization patterns.

## Table of Contents

- [Basic Crawling Examples](#basic-crawling-examples)
- [Streaming Examples](#streaming-examples)
- [Dynamic Rendering Examples](#dynamic-rendering-examples)
- [Deep Search Examples](#deep-search-examples)
- [Session Management Examples](#session-management-examples)
- [Error Handling Examples](#error-handling-examples)
- [Performance Optimization Examples](#performance-optimization-examples)
- [Integration Examples](#integration-examples)

## Basic Crawling Examples

### JavaScript/Node.js

#### Simple URL Crawling

```javascript
const RipTideClient = require('@riptide/api-client');

class BasicCrawler {
    constructor(apiUrl = 'http://localhost:8080') {
        this.apiUrl = apiUrl;
        this.sessionId = this.generateSessionId();
    }

    generateSessionId() {
        const timestamp = Date.now();
        const random = Math.random().toString(36).substring(2, 8);
        return `session-${timestamp}-${random}`;
    }

    async crawlSingle(url, options = {}) {
        const response = await fetch(`${this.apiUrl}/crawl`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'X-Session-ID': this.sessionId
            },
            body: JSON.stringify({
                urls: [url],
                options: {
                    concurrency: 1,
                    cache_mode: 'read_write',
                    extract_mode: 'article',
                    timeout_seconds: 30,
                    ...options
                }
            })
        });

        if (!response.ok) {
            const error = await response.json();
            throw new Error(`Crawl failed: ${error.error.message}`);
        }

        const result = await response.json();
        return result.results[0];
    }

    async crawlBatch(urls, options = {}) {
        const response = await fetch(`${this.apiUrl}/crawl`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'X-Session-ID': this.sessionId
            },
            body: JSON.stringify({
                urls,
                options: {
                    concurrency: Math.min(urls.length, 5),
                    cache_mode: 'read_write',
                    extract_mode: 'article',
                    quality_threshold: 0.5,
                    ...options
                }
            })
        });

        if (!response.ok) {
            const error = await response.json();
            throw new Error(`Batch crawl failed: ${error.error.message}`);
        }

        return await response.json();
    }

    async crawlWithRetry(url, maxRetries = 3) {
        let lastError;

        for (let attempt = 1; attempt <= maxRetries; attempt++) {
            try {
                return await this.crawlSingle(url);
            } catch (error) {
                lastError = error;

                if (attempt < maxRetries) {
                    const delay = Math.pow(2, attempt) * 1000;
                    console.log(`Attempt ${attempt} failed, retrying in ${delay}ms...`);
                    await new Promise(resolve => setTimeout(resolve, delay));
                }
            }
        }

        throw lastError;
    }
}

// Usage examples
async function basicExamples() {
    const crawler = new BasicCrawler();

    // Single URL
    try {
        const result = await crawler.crawlSingle('https://example.com');
        console.log('Title:', result.document?.title);
        console.log('Content preview:', result.document?.text.substring(0, 200));
        console.log('Processing time:', result.processing_time_ms, 'ms');
        console.log('Quality score:', result.quality_score);
    } catch (error) {
        console.error('Single crawl failed:', error.message);
    }

    // Batch crawling
    try {
        const batchResult = await crawler.crawlBatch([
            'https://example.com/page1',
            'https://example.com/page2',
            'https://example.com/page3'
        ]);

        console.log(`Crawled ${batchResult.total_urls} URLs`);
        console.log(`Success rate: ${(batchResult.successful / batchResult.total_urls * 100).toFixed(1)}%`);
        console.log(`Cache hit rate: ${(batchResult.statistics.cache_hit_rate * 100).toFixed(1)}%`);

        batchResult.results.forEach((result, index) => {
            if (result.document) {
                console.log(`${index + 1}. ${result.document.title} (${result.processing_time_ms}ms)`);
            } else {
                console.log(`${index + 1}. Failed: ${result.error?.message}`);
            }
        });
    } catch (error) {
        console.error('Batch crawl failed:', error.message);
    }

    // Crawl with retry
    try {
        const resilientResult = await crawler.crawlWithRetry('https://unreliable-site.com');
        console.log('Resilient crawl succeeded:', resilientResult.document?.title);
    } catch (error) {
        console.error('All retry attempts failed:', error.message);
    }
}

// Run examples
basicExamples().catch(console.error);
```

### Python

#### Complete Python Client Implementation

```python
import requests
import json
import time
import asyncio
import aiohttp
from typing import List, Dict, Optional, Union
from dataclasses import dataclass

@dataclass
class CrawlOptions:
    concurrency: int = 3
    cache_mode: str = 'read_write'
    extract_mode: str = 'article'
    timeout_seconds: int = 30
    quality_threshold: float = 0.5
    user_agent: Optional[str] = None
    follow_redirects: bool = True

class RipTidePythonClient:
    def __init__(self, api_url: str = 'http://localhost:8080'):
        self.api_url = api_url
        self.session_id = self._generate_session_id()
        self.session = requests.Session()

        # Configure session with optimization
        self.session.headers.update({
            'Content-Type': 'application/json',
            'X-Session-ID': self.session_id,
            'User-Agent': 'RipTide-Python-Client/1.0'
        })

    def _generate_session_id(self) -> str:
        timestamp = int(time.time())
        import random
        random_str = ''.join(random.choices('abcdefghijklmnopqrstuvwxyz0123456789', k=6))
        return f"python-session-{timestamp}-{random_str}"

    def crawl_single(self, url: str, options: Optional[CrawlOptions] = None) -> Dict:
        """Crawl a single URL with the RipTide API."""
        if options is None:
            options = CrawlOptions()

        payload = {
            'urls': [url],
            'options': {
                'concurrency': 1,
                'cache_mode': options.cache_mode,
                'extract_mode': options.extract_mode,
                'timeout_seconds': options.timeout_seconds,
                'quality_threshold': options.quality_threshold
            }
        }

        if options.user_agent:
            payload['options']['user_agent'] = options.user_agent

        response = self.session.post(f'{self.api_url}/crawl', json=payload)

        if not response.ok:
            error_data = response.json()
            raise Exception(f"Crawl failed: {error_data['error']['message']}")

        result = response.json()
        return result['results'][0] if result['results'] else None

    def crawl_batch(self, urls: List[str], options: Optional[CrawlOptions] = None) -> Dict:
        """Crawl multiple URLs in batch."""
        if options is None:
            options = CrawlOptions()

        # Optimize concurrency based on batch size
        optimal_concurrency = min(len(urls), options.concurrency)

        payload = {
            'urls': urls,
            'options': {
                'concurrency': optimal_concurrency,
                'cache_mode': options.cache_mode,
                'extract_mode': options.extract_mode,
                'timeout_seconds': options.timeout_seconds,
                'quality_threshold': options.quality_threshold
            }
        }

        response = self.session.post(f'{self.api_url}/crawl', json=payload)

        if not response.ok:
            error_data = response.json()
            raise Exception(f"Batch crawl failed: {error_data['error']['message']}")

        return response.json()

    def crawl_with_retry(self, url: str, max_retries: int = 3, options: Optional[CrawlOptions] = None) -> Dict:
        """Crawl with exponential backoff retry logic."""
        last_error = None

        for attempt in range(1, max_retries + 1):
            try:
                return self.crawl_single(url, options)
            except Exception as error:
                last_error = error

                if attempt < max_retries:
                    delay = 2 ** attempt
                    print(f"Attempt {attempt} failed, retrying in {delay}s...")
                    time.sleep(delay)

        raise last_error

    async def crawl_async(self, urls: List[str], options: Optional[CrawlOptions] = None) -> Dict:
        """Asynchronous crawling using aiohttp."""
        if options is None:
            options = CrawlOptions()

        payload = {
            'urls': urls,
            'options': {
                'concurrency': min(len(urls), options.concurrency),
                'cache_mode': options.cache_mode,
                'extract_mode': options.extract_mode,
                'timeout_seconds': options.timeout_seconds
            }
        }

        headers = {
            'Content-Type': 'application/json',
            'X-Session-ID': self.session_id
        }

        async with aiohttp.ClientSession() as session:
            async with session.post(
                f'{self.api_url}/crawl',
                json=payload,
                headers=headers
            ) as response:

                if not response.ok:
                    error_data = await response.json()
                    raise Exception(f"Async crawl failed: {error_data['error']['message']}")

                return await response.json()

# Usage examples
def python_examples():
    client = RipTidePythonClient()

    # Single URL crawling
    try:
        result = client.crawl_single('https://example.com')
        if result and result.get('document'):
            print(f"Title: {result['document'].get('title')}")
            print(f"Word count: {result['document'].get('word_count')}")
            print(f"Quality score: {result.get('quality_score')}")
            print(f"Processing time: {result.get('processing_time_ms')}ms")
        else:
            print("No content extracted")
    except Exception as e:
        print(f"Single crawl error: {e}")

    # Batch crawling with custom options
    try:
        options = CrawlOptions(
            concurrency=5,
            cache_mode='read_write',
            extract_mode='article',
            quality_threshold=0.7
        )

        batch_result = client.crawl_batch([
            'https://example.com/article1',
            'https://example.com/article2',
            'https://example.com/article3'
        ], options)

        print(f"\nBatch Results:")
        print(f"Total URLs: {batch_result['total_urls']}")
        print(f"Successful: {batch_result['successful']}")
        print(f"Failed: {batch_result['failed']}")
        print(f"Cache hit rate: {batch_result['statistics']['cache_hit_rate']:.2%}")

        for i, result in enumerate(batch_result['results']):
            if result.get('document'):
                title = result['document'].get('title', 'No title')
                time_ms = result.get('processing_time_ms', 0)
                print(f"  {i+1}. {title} ({time_ms}ms)")
            else:
                error_msg = result.get('error', {}).get('message', 'Unknown error')
                print(f"  {i+1}. Error: {error_msg}")

    except Exception as e:
        print(f"Batch crawl error: {e}")

    # Resilient crawling with retries
    try:
        resilient_result = client.crawl_with_retry(
            'https://sometimes-unreliable.com',
            max_retries=3
        )
        print(f"\nResilient crawl succeeded: {resilient_result['document']['title']}")
    except Exception as e:
        print(f"All retry attempts failed: {e}")

# Async example
async def async_example():
    client = RipTidePythonClient()

    urls = [
        'https://example.com/page1',
        'https://example.com/page2',
        'https://example.com/page3'
    ]

    try:
        result = await client.crawl_async(urls)
        print(f"Async crawl completed: {result['successful']}/{result['total_urls']} successful")
    except Exception as e:
        print(f"Async crawl error: {e}")

# Run examples
if __name__ == "__main__":
    python_examples()

    # Run async example
    asyncio.run(async_example())
```

### Go

#### Production-Ready Go Client

```go
package main

import (
    "bytes"
    "context"
    "encoding/json"
    "fmt"
    "io"
    "net/http"
    "time"
    "math/rand"
    "strconv"
    "strings"
)

type CrawlOptions struct {
    Concurrency      int     `json:"concurrency,omitempty"`
    CacheMode       string  `json:"cache_mode,omitempty"`
    ExtractMode     string  `json:"extract_mode,omitempty"`
    TimeoutSeconds  int     `json:"timeout_seconds,omitempty"`
    QualityThreshold float64 `json:"quality_threshold,omitempty"`
    UserAgent       string  `json:"user_agent,omitempty"`
    FollowRedirects bool    `json:"follow_redirects,omitempty"`
}

type CrawlRequest struct {
    URLs    []string     `json:"urls"`
    Options *CrawlOptions `json:"options,omitempty"`
}

type Document struct {
    URL          string   `json:"url"`
    Title        *string  `json:"title"`
    Byline       *string  `json:"byline"`
    PublishedISO *string  `json:"published_iso"`
    Markdown     string   `json:"markdown"`
    Text         string   `json:"text"`
    Links        []string `json:"links"`
    Media        []string `json:"media"`
    Language     *string  `json:"language"`
    ReadingTime  *int     `json:"reading_time"`
    QualityScore *int     `json:"quality_score"`
    WordCount    *int     `json:"word_count"`
    Categories   []string `json:"categories"`
    SiteName     *string  `json:"site_name"`
    Description  *string  `json:"description"`
}

type ErrorInfo struct {
    ErrorType string `json:"error_type"`
    Message   string `json:"message"`
    Retryable bool   `json:"retryable"`
}

type CrawlResult struct {
    URL            string     `json:"url"`
    Status         int        `json:"status"`
    FromCache      bool       `json:"from_cache"`
    GateDecision   string     `json:"gate_decision"`
    QualityScore   float64    `json:"quality_score"`
    ProcessingTime int64      `json:"processing_time_ms"`
    Document       *Document  `json:"document"`
    Error          *ErrorInfo `json:"error"`
    CacheKey       string     `json:"cache_key"`
}

type CrawlStatistics struct {
    TotalProcessingTime int64                   `json:"total_processing_time_ms"`
    AvgProcessingTime   float64                 `json:"avg_processing_time_ms"`
    GateDecisions      GateDecisionBreakdown   `json:"gate_decisions"`
    CacheHitRate       float64                 `json:"cache_hit_rate"`
}

type GateDecisionBreakdown struct {
    Raw        int `json:"raw"`
    ProbesFirst int `json:"probes_first"`
    Headless   int `json:"headless"`
    Cached     int `json:"cached"`
}

type CrawlResponse struct {
    TotalURLs  int              `json:"total_urls"`
    Successful int              `json:"successful"`
    Failed     int              `json:"failed"`
    FromCache  int              `json:"from_cache"`
    Results    []CrawlResult    `json:"results"`
    Statistics CrawlStatistics  `json:"statistics"`
}

type RipTideClient struct {
    baseURL   string
    sessionID string
    client    *http.Client
}

func NewRipTideClient(baseURL string) *RipTideClient {
    return &RipTideClient{
        baseURL:   baseURL,
        sessionID: generateSessionID(),
        client: &http.Client{
            Timeout: 60 * time.Second,
            Transport: &http.Transport{
                MaxIdleConns:       100,
                IdleConnTimeout:    90 * time.Second,
                DisableCompression: false,
            },
        },
    }
}

func generateSessionID() string {
    timestamp := time.Now().Unix()
    randStr := generateRandomString(6)
    return fmt.Sprintf("go-session-%d-%s", timestamp, randStr)
}

func generateRandomString(length int) string {
    const charset = "abcdefghijklmnopqrstuvwxyz0123456789"
    b := make([]byte, length)
    for i := range b {
        b[i] = charset[rand.Intn(len(charset))]
    }
    return string(b)
}

func (c *RipTideClient) CrawlSingle(ctx context.Context, url string, options *CrawlOptions) (*CrawlResult, error) {
    if options == nil {
        options = &CrawlOptions{
            Concurrency:      1,
            CacheMode:       "read_write",
            ExtractMode:     "article",
            TimeoutSeconds:  30,
            QualityThreshold: 0.5,
        }
    }

    response, err := c.CrawlBatch(ctx, []string{url}, options)
    if err != nil {
        return nil, err
    }

    if len(response.Results) == 0 {
        return nil, fmt.Errorf("no results returned")
    }

    return &response.Results[0], nil
}

func (c *RipTideClient) CrawlBatch(ctx context.Context, urls []string, options *CrawlOptions) (*CrawlResponse, error) {
    if options == nil {
        options = &CrawlOptions{
            Concurrency:      min(len(urls), 5),
            CacheMode:       "read_write",
            ExtractMode:     "article",
            TimeoutSeconds:  30,
            QualityThreshold: 0.5,
        }
    }

    // Optimize concurrency for batch size
    options.Concurrency = min(len(urls), options.Concurrency)

    request := CrawlRequest{
        URLs:    urls,
        Options: options,
    }

    jsonData, err := json.Marshal(request)
    if err != nil {
        return nil, fmt.Errorf("failed to marshal request: %w", err)
    }

    req, err := http.NewRequestWithContext(ctx, "POST", c.baseURL+"/crawl", bytes.NewBuffer(jsonData))
    if err != nil {
        return nil, fmt.Errorf("failed to create request: %w", err)
    }

    req.Header.Set("Content-Type", "application/json")
    req.Header.Set("X-Session-ID", c.sessionID)
    req.Header.Set("User-Agent", "RipTide-Go-Client/1.0")

    resp, err := c.client.Do(req)
    if err != nil {
        return nil, fmt.Errorf("request failed: %w", err)
    }
    defer resp.Body.Close()

    body, err := io.ReadAll(resp.Body)
    if err != nil {
        return nil, fmt.Errorf("failed to read response: %w", err)
    }

    if resp.StatusCode != http.StatusOK {
        var errorResp struct {
            Error ErrorInfo `json:"error"`
        }
        if json.Unmarshal(body, &errorResp) == nil {
            return nil, fmt.Errorf("API error (%d): %s", resp.StatusCode, errorResp.Error.Message)
        }
        return nil, fmt.Errorf("HTTP error %d: %s", resp.StatusCode, string(body))
    }

    var response CrawlResponse
    if err := json.Unmarshal(body, &response); err != nil {
        return nil, fmt.Errorf("failed to unmarshal response: %w", err)
    }

    return &response, nil
}

func (c *RipTideClient) CrawlWithRetry(ctx context.Context, url string, maxRetries int, options *CrawlOptions) (*CrawlResult, error) {
    var lastErr error

    for attempt := 1; attempt <= maxRetries; attempt++ {
        result, err := c.CrawlSingle(ctx, url, options)
        if err == nil {
            return result, nil
        }

        lastErr = err

        if attempt < maxRetries {
            delay := time.Duration(1<<uint(attempt)) * time.Second
            fmt.Printf("Attempt %d failed, retrying in %v...\n", attempt, delay)

            select {
            case <-ctx.Done():
                return nil, ctx.Err()
            case <-time.After(delay):
                continue
            }
        }
    }

    return nil, fmt.Errorf("all %d attempts failed, last error: %w", maxRetries, lastErr)
}

func min(a, b int) int {
    if a < b {
        return a
    }
    return b
}

// Usage examples
func main() {
    client := NewRipTideClient("http://localhost:8080")
    ctx := context.Background()

    // Single URL crawling
    fmt.Println("=== Single URL Crawling ===")
    result, err := client.CrawlSingle(ctx, "https://example.com", nil)
    if err != nil {
        fmt.Printf("Single crawl error: %v\n", err)
    } else {
        if result.Document != nil {
            fmt.Printf("Title: %s\n", stringValue(result.Document.Title))
            fmt.Printf("Word count: %d\n", intValue(result.Document.WordCount))
            fmt.Printf("Quality score: %.2f\n", result.QualityScore)
            fmt.Printf("Processing time: %dms\n", result.ProcessingTime)
        } else {
            fmt.Printf("No content extracted: %v\n", result.Error)
        }
    }

    // Batch crawling
    fmt.Println("\n=== Batch Crawling ===")
    urls := []string{
        "https://example.com/page1",
        "https://example.com/page2",
        "https://example.com/page3",
    }

    options := &CrawlOptions{
        Concurrency:      3,
        CacheMode:       "read_write",
        ExtractMode:     "article",
        QualityThreshold: 0.7,
    }

    batchResponse, err := client.CrawlBatch(ctx, urls, options)
    if err != nil {
        fmt.Printf("Batch crawl error: %v\n", err)
    } else {
        fmt.Printf("Total URLs: %d\n", batchResponse.TotalURLs)
        fmt.Printf("Successful: %d\n", batchResponse.Successful)
        fmt.Printf("Failed: %d\n", batchResponse.Failed)
        fmt.Printf("Cache hit rate: %.1f%%\n", batchResponse.Statistics.CacheHitRate*100)

        for i, result := range batchResponse.Results {
            if result.Document != nil {
                title := stringValue(result.Document.Title)
                fmt.Printf("  %d. %s (%dms)\n", i+1, title, result.ProcessingTime)
            } else {
                errorMsg := "Unknown error"
                if result.Error != nil {
                    errorMsg = result.Error.Message
                }
                fmt.Printf("  %d. Error: %s\n", i+1, errorMsg)
            }
        }
    }

    // Resilient crawling with retries
    fmt.Println("\n=== Resilient Crawling ===")
    resilientResult, err := client.CrawlWithRetry(ctx, "https://unreliable-site.com", 3, nil)
    if err != nil {
        fmt.Printf("All retry attempts failed: %v\n", err)
    } else {
        fmt.Printf("Resilient crawl succeeded: %s\n", stringValue(resilientResult.Document.Title))
    }
}

// Helper functions for safe pointer dereferencing
func stringValue(s *string) string {
    if s == nil {
        return ""
    }
    return *s
}

func intValue(i *int) int {
    if i == nil {
        return 0
    }
    return *i
}
```

## Streaming Examples

### JavaScript Streaming Implementation

```javascript
class StreamingCrawler {
    constructor(baseUrl = 'http://localhost:8080') {
        this.baseUrl = baseUrl;
        this.sessionId = this.generateSessionId();
    }

    generateSessionId() {
        const timestamp = Date.now();
        const random = Math.random().toString(36).substring(2, 8);
        return `streaming-session-${timestamp}-${random}`;
    }

    async streamCrawl(urls, options = {}, callbacks = {}) {
        const response = await fetch(`${this.baseUrl}/crawl/stream`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'X-Session-ID': this.sessionId,
                'X-Buffer-Size': options.bufferSize || '256'
            },
            body: JSON.stringify({
                urls,
                options: {
                    concurrency: Math.min(urls.length, options.concurrency || 3),
                    cache_mode: options.cacheMode || 'read_write',
                    extract_mode: options.extractMode || 'article'
                }
            })
        });

        if (!response.ok) {
            const error = await response.json();
            throw new Error(`Stream failed: ${error.error.message}`);
        }

        return this.processStream(response.body, callbacks);
    }

    async processStream(stream, callbacks) {
        const reader = stream.getReader();
        const decoder = new TextDecoder();
        const results = [];
        let summary = null;

        try {
            while (true) {
                const { done, value } = await reader.read();
                if (done) break;

                const chunk = decoder.decode(value, { stream: true });
                const lines = chunk.split('\n');

                for (const line of lines) {
                    if (!line.trim()) continue;

                    try {
                        const event = JSON.parse(line);

                        switch (event.event) {
                            case 'start':
                                callbacks.onStart?.(event);
                                break;

                            case 'progress':
                                callbacks.onProgress?.(event);
                                break;

                            case 'result':
                                results.push(event);
                                callbacks.onResult?.(event);
                                break;

                            case 'summary':
                                summary = event;
                                callbacks.onSummary?.(event);
                                break;

                            case 'error':
                                callbacks.onError?.(event);
                                break;

                            case 'ping':
                                callbacks.onPing?.(event);
                                break;
                        }
                    } catch (error) {
                        console.warn('Failed to parse stream event:', error);
                        callbacks.onParseError?.(error, line);
                    }
                }
            }
        } finally {
            reader.releaseLock();
        }

        return { results, summary };
    }

    async streamDeepSearch(query, options = {}, callbacks = {}) {
        const response = await fetch(`${this.baseUrl}/deepsearch/stream`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'X-Session-ID': this.sessionId
            },
            body: JSON.stringify({
                query,
                limit: options.limit || 10,
                include_content: options.includeContent !== false,
                crawl_options: options.crawlOptions || {}
            })
        });

        if (!response.ok) {
            const error = await response.json();
            throw new Error(`Deep search stream failed: ${error.error.message}`);
        }

        return this.processStream(response.body, callbacks);
    }

    // Server-Sent Events implementation
    streamCrawlSSE(urls, options = {}, callbacks = {}) {
        return new Promise((resolve, reject) => {
            // First, initiate the crawl
            fetch(`${this.baseUrl}/crawl/sse`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                    'X-Session-ID': this.sessionId
                },
                body: JSON.stringify({ urls, options })
            }).then(response => {
                if (!response.ok) {
                    throw new Error(`SSE crawl initiation failed: ${response.status}`);
                }

                // Create EventSource for the stream
                const eventSource = new EventSource(
                    `${this.baseUrl}/crawl/sse?session_id=${this.sessionId}`
                );

                const results = [];
                let summary = null;

                eventSource.onopen = () => {
                    callbacks.onConnect?.();
                };

                eventSource.addEventListener('start', (event) => {
                    const data = JSON.parse(event.data);
                    callbacks.onStart?.(data);
                });

                eventSource.addEventListener('progress', (event) => {
                    const data = JSON.parse(event.data);
                    callbacks.onProgress?.(data);
                });

                eventSource.addEventListener('result', (event) => {
                    const data = JSON.parse(event.data);
                    results.push(data);
                    callbacks.onResult?.(data);
                });

                eventSource.addEventListener('summary', (event) => {
                    const data = JSON.parse(event.data);
                    summary = data;
                    callbacks.onSummary?.(data);
                    eventSource.close();
                    resolve({ results, summary });
                });

                eventSource.addEventListener('error', (event) => {
                    const data = JSON.parse(event.data);
                    callbacks.onError?.(data);
                });

                eventSource.onerror = (error) => {
                    console.error('EventSource failed:', error);
                    eventSource.close();
                    reject(new Error('SSE connection failed'));
                };

            }).catch(reject);
        });
    }

    // WebSocket implementation
    async streamCrawlWebSocket(urls, options = {}, callbacks = {}) {
        return new Promise((resolve, reject) => {
            const wsUrl = `ws://localhost:8080/crawl/ws?session_id=${this.sessionId}`;
            const ws = new WebSocket(wsUrl);

            const results = [];
            let summary = null;

            ws.onopen = () => {
                callbacks.onConnect?.();

                // Send crawl request
                ws.send(JSON.stringify({
                    action: 'crawl',
                    urls,
                    options: {
                        concurrency: Math.min(urls.length, options.concurrency || 3),
                        cache_mode: options.cacheMode || 'read_write',
                        extract_mode: options.extractMode || 'article'
                    }
                }));
            };

            ws.onmessage = (event) => {
                try {
                    const data = JSON.parse(event.data);

                    switch (data.event || data.action) {
                        case 'start':
                            callbacks.onStart?.(data);
                            break;
                        case 'progress':
                            callbacks.onProgress?.(data);
                            break;
                        case 'result':
                            results.push(data);
                            callbacks.onResult?.(data);
                            break;
                        case 'summary':
                            summary = data;
                            callbacks.onSummary?.(data);
                            ws.close();
                            resolve({ results, summary });
                            break;
                        case 'error':
                            callbacks.onError?.(data);
                            break;
                        case 'ping':
                            callbacks.onPing?.(data);
                            // Send pong response
                            ws.send(JSON.stringify({ action: 'pong' }));
                            break;
                    }
                } catch (error) {
                    console.error('Failed to parse WebSocket message:', error);
                    callbacks.onParseError?.(error, event.data);
                }
            };

            ws.onerror = (error) => {
                console.error('WebSocket error:', error);
                reject(new Error('WebSocket connection failed'));
            };

            ws.onclose = (event) => {
                if (event.code !== 1000) {
                    console.warn('WebSocket closed unexpectedly:', event.code, event.reason);
                }
                callbacks.onDisconnect?.(event);
            };
        });
    }
}

// Usage examples
async function streamingExamples() {
    const crawler = new StreamingCrawler();

    // NDJSON streaming with progress tracking
    console.log('=== NDJSON Streaming ===');

    const urls = [
        'https://example.com/page1',
        'https://example.com/page2',
        'https://example.com/page3',
        'https://example.com/page4',
        'https://example.com/page5'
    ];

    try {
        const { results, summary } = await crawler.streamCrawl(urls, {
            concurrency: 3,
            bufferSize: 128
        }, {
            onStart: (event) => {
                console.log(`Starting crawl of ${event.total_urls} URLs`);
            },
            onProgress: (event) => {
                const percent = (event.completed / event.total * 100).toFixed(1);
                console.log(`Progress: ${event.completed}/${event.total} (${percent}%)`);
            },
            onResult: (event) => {
                const title = event.document?.title || 'No title';
                console.log(`✓ ${event.url}: ${title} (${event.processing_time_ms}ms)`);
            },
            onSummary: (event) => {
                console.log(`Completed: ${event.successful}/${event.total_urls} successful`);
                console.log(`Total time: ${event.total_time_ms}ms`);
            },
            onError: (event) => {
                console.error(`Error: ${event.error.message}`);
            }
        });

        console.log(`Final results: ${results.length} processed`);
    } catch (error) {
        console.error('Streaming failed:', error);
    }

    // Deep search streaming
    console.log('\n=== Deep Search Streaming ===');

    try {
        await crawler.streamDeepSearch('web scraping best practices', {
            limit: 5,
            includeContent: true
        }, {
            onStart: (event) => {
                console.log(`Searching for: "${event.query}"`);
            },
            onProgress: (event) => {
                console.log(`Search progress: ${event.completed}/${event.total}`);
            },
            onResult: (event) => {
                if (event.search_title) {
                    console.log(`🔍 Found: ${event.search_title}`);
                }
                if (event.content) {
                    console.log(`📄 Extracted: ${event.content.title}`);
                }
            }
        });
    } catch (error) {
        console.error('Deep search streaming failed:', error);
    }

    // Server-Sent Events example
    console.log('\n=== Server-Sent Events ===');

    try {
        await crawler.streamCrawlSSE(urls.slice(0, 3), {}, {
            onConnect: () => {
                console.log('SSE connection established');
            },
            onProgress: (event) => {
                console.log(`SSE Progress: ${event.completed}/${event.total}`);
            },
            onResult: (event) => {
                console.log(`SSE Result: ${event.url}`);
            }
        });
    } catch (error) {
        console.error('SSE streaming failed:', error);
    }

    // WebSocket example
    console.log('\n=== WebSocket Streaming ===');

    try {
        await crawler.streamCrawlWebSocket(urls.slice(0, 2), {}, {
            onConnect: () => {
                console.log('WebSocket connection established');
            },
            onProgress: (event) => {
                console.log(`WS Progress: ${event.completed}/${event.total}`);
            },
            onResult: (event) => {
                console.log(`WS Result: ${event.url}`);
            },
            onDisconnect: (event) => {
                console.log('WebSocket disconnected');
            }
        });
    } catch (error) {
        console.error('WebSocket streaming failed:', error);
    }
}

// Run streaming examples
streamingExamples().catch(console.error);
```

This comprehensive examples collection demonstrates real-world usage patterns for all major features of the RipTide API, providing production-ready code that can be adapted for specific use cases.