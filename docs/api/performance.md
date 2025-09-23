# Performance Optimization Guide

## Overview

The RipTide API is designed for high-performance web crawling and content extraction with sophisticated optimization strategies. This guide covers performance considerations, optimization techniques, and monitoring approaches to maximize throughput and minimize latency.

## Architecture Performance Features

### WASM-Powered Extraction
- **High-Performance Processing**: WebAssembly component provides near-native speed
- **Memory Efficiency**: Controlled memory usage with automatic cleanup
- **Concurrent Safe**: Thread-safe operations for parallel processing

### Intelligent Gate System
- **Content Routing**: Automatic selection of optimal processing path
- **Quality Assessment**: Pre-processing quality scoring
- **Resource Optimization**: Efficient resource allocation based on content type

### Advanced Caching
- **Redis-Backed**: High-performance distributed caching
- **TTL Management**: Configurable time-to-live settings
- **Cache Warming**: Proactive cache population strategies

## Performance Metrics

### Key Performance Indicators

| Metric | Target | Good | Needs Improvement |
|--------|--------|------|-------------------|
| Response Time (p95) | <2s | <5s | >5s |
| Throughput | >100 req/s | >50 req/s | <50 req/s |
| Cache Hit Rate | >70% | >50% | <50% |
| Error Rate | <1% | <5% | >5% |
| Memory Usage | <2GB | <4GB | >4GB |
| CPU Usage | <60% | <80% | >80% |

### Performance Benchmarks

```bash
# Benchmark single URL extraction
curl -w "@curl-format.txt" -X POST 'http://localhost:8080/crawl' \
  -H 'Content-Type: application/json' \
  -d '{"urls": ["https://example.com"]}'

# Benchmark batch processing
curl -w "@curl-format.txt" -X POST 'http://localhost:8080/crawl' \
  -H 'Content-Type: application/json' \
  -d '{
    "urls": [
      "https://example.com/page1",
      "https://example.com/page2",
      "https://example.com/page3"
    ],
    "options": {
      "concurrency": 3,
      "cache_mode": "read_write"
    }
  }'

# curl-format.txt content:
#     time_namelookup:  %{time_namelookup}\n
#        time_connect:  %{time_connect}\n
#     time_appconnect:  %{time_appconnect}\n
#    time_pretransfer:  %{time_pretransfer}\n
#       time_redirect:  %{time_redirect}\n
#  time_starttransfer:  %{time_starttransfer}\n
#                     ----------\n
#          time_total:  %{time_total}\n
```

## Optimization Strategies

### 1. Concurrency Optimization

#### Optimal Concurrency Configuration

```javascript
// Client-side concurrency optimization
class ConcurrencyOptimizer {
    constructor() {
        this.baseUrl = 'http://localhost:8080';
        this.maxConcurrency = 10;
        this.minConcurrency = 1;
        this.currentOptimal = 3;
        this.performanceHistory = [];
    }

    async findOptimalConcurrency(urls, testSizes = [1, 2, 3, 5, 8, 10]) {
        const results = [];

        for (const concurrency of testSizes) {
            const result = await this.benchmarkConcurrency(urls.slice(0, 20), concurrency);
            results.push({ concurrency, ...result });

            console.log(`Concurrency ${concurrency}: ${result.avgResponseTime}ms avg, ${result.throughput} req/s`);
        }

        // Find optimal based on throughput vs response time
        const optimal = results.reduce((best, current) => {
            const bestScore = best.throughput / (best.avgResponseTime / 1000);
            const currentScore = current.throughput / (current.avgResponseTime / 1000);
            return currentScore > bestScore ? current : best;
        });

        this.currentOptimal = optimal.concurrency;
        return optimal;
    }

    async benchmarkConcurrency(urls, concurrency) {
        const startTime = Date.now();

        const response = await fetch(`${this.baseUrl}/crawl`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                urls,
                options: { concurrency }
            })
        });

        const result = await response.json();
        const endTime = Date.now();
        const totalTime = endTime - startTime;

        return {
            totalTime,
            avgResponseTime: result.statistics.avg_processing_time_ms,
            throughput: urls.length / (totalTime / 1000),
            cacheHitRate: result.statistics.cache_hit_rate,
            successRate: result.successful / result.total_urls
        };
    }
}

// Usage
const optimizer = new ConcurrencyOptimizer();
const testUrls = [
    'https://example.com/page1',
    'https://example.com/page2',
    // ... more URLs
];

const optimal = await optimizer.findOptimalConcurrency(testUrls);
console.log(`Optimal concurrency: ${optimal.concurrency}`);
```

#### Server-Side Concurrency Management

```rust
// Rust configuration for optimal performance
#[derive(Clone)]
pub struct PerformanceConfig {
    pub max_concurrent_requests: usize,
    pub max_concurrent_extractions: usize,
    pub pipeline_buffer_size: usize,
    pub gate_worker_threads: usize,
    pub extractor_pool_size: usize,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        let cpu_count = num_cpus::get();

        Self {
            max_concurrent_requests: cpu_count * 4,
            max_concurrent_extractions: cpu_count * 2,
            pipeline_buffer_size: 1000,
            gate_worker_threads: cpu_count,
            extractor_pool_size: cpu_count * 2,
        }
    }
}

impl PerformanceConfig {
    pub fn for_high_throughput() -> Self {
        let cpu_count = num_cpus::get();

        Self {
            max_concurrent_requests: cpu_count * 8,
            max_concurrent_extractions: cpu_count * 4,
            pipeline_buffer_size: 2000,
            gate_worker_threads: cpu_count * 2,
            extractor_pool_size: cpu_count * 4,
        }
    }

    pub fn for_low_latency() -> Self {
        let cpu_count = num_cpus::get();

        Self {
            max_concurrent_requests: cpu_count * 2,
            max_concurrent_extractions: cpu_count,
            pipeline_buffer_size: 500,
            gate_worker_threads: cpu_count,
            extractor_pool_size: cpu_count,
        }
    }
}
```

### 2. Caching Optimization

#### Cache Strategy Selection

```javascript
class CacheStrategyOptimizer {
    constructor(redis) {
        this.redis = redis;
        this.strategies = {
            aggressive: { ttl: 3600, preload: true, compression: true },
            balanced: { ttl: 1800, preload: false, compression: true },
            conservative: { ttl: 600, preload: false, compression: false }
        };
    }

    selectOptimalStrategy(urlPattern, contentType, updateFrequency) {
        // High-update frequency content
        if (updateFrequency === 'high') {
            return this.strategies.conservative;
        }

        // Static content (images, documents)
        if (contentType.includes('pdf') || contentType.includes('image')) {
            return this.strategies.aggressive;
        }

        // News/blog content
        if (urlPattern.includes('news') || urlPattern.includes('blog')) {
            return this.strategies.balanced;
        }

        // Default to balanced
        return this.strategies.balanced;
    }

    async implementCacheWarming(popularUrls) {
        const warmingBatch = popularUrls.slice(0, 100); // Limit batch size

        for (const url of warmingBatch) {
            try {
                // Check if already cached
                const cached = await this.redis.get(`crawl:v1:${url}`);
                if (!cached) {
                    // Trigger background crawl
                    fetch('/crawl', {
                        method: 'POST',
                        headers: {
                            'Content-Type': 'application/json',
                            'X-Cache-Warming': 'true'
                        },
                        body: JSON.stringify({
                            urls: [url],
                            options: { cache_mode: 'write_only' }
                        })
                    }).catch(err => console.log(`Cache warming failed for ${url}: ${err}`));
                }
            } catch (error) {
                console.error(`Cache warming error for ${url}:`, error);
            }
        }
    }

    async optimizeCacheUsage() {
        // Analyze cache hit patterns
        const stats = await this.redis.info('stats');
        const hitRate = this.parseHitRate(stats);

        if (hitRate < 0.5) {
            console.log('Low cache hit rate detected, adjusting TTL strategies');
            // Increase TTL for better hit rates
            Object.values(this.strategies).forEach(strategy => {
                strategy.ttl = Math.min(strategy.ttl * 1.5, 7200);
            });
        }

        return { hitRate, strategies: this.strategies };
    }

    parseHitRate(stats) {
        const hits = parseInt(stats.match(/keyspace_hits:(\d+)/)?.[1] || '0');
        const misses = parseInt(stats.match(/keyspace_misses:(\d+)/)?.[1] || '0');
        return hits / (hits + misses) || 0;
    }
}
```

#### Cache Preloading Strategy

```javascript
class CachePreloader {
    constructor(apiClient, redis) {
        this.apiClient = apiClient;
        this.redis = redis;
        this.preloadQueue = [];
        this.isProcessing = false;
    }

    async analyzeTrafficPatterns() {
        // Analyze access logs to find popular URLs
        const accessLogs = await this.getRecentAccessLogs();
        const urlFrequency = new Map();

        accessLogs.forEach(log => {
            const count = urlFrequency.get(log.url) || 0;
            urlFrequency.set(log.url, count + 1);
        });

        // Sort by frequency and recency
        const popularUrls = Array.from(urlFrequency.entries())
            .sort((a, b) => b[1] - a[1])
            .slice(0, 50)
            .map(([url]) => url);

        return popularUrls;
    }

    async schedulePreloading() {
        if (this.isProcessing) return;

        this.isProcessing = true;

        try {
            const popularUrls = await this.analyzeTrafficPatterns();

            for (const url of popularUrls) {
                const cacheKey = `crawl:v1:${this.hashUrl(url)}`;
                const cached = await this.redis.exists(cacheKey);

                if (!cached) {
                    this.preloadQueue.push(url);
                }
            }

            await this.processPreloadQueue();
        } finally {
            this.isProcessing = false;
        }
    }

    async processPreloadQueue() {
        const batchSize = 5;

        while (this.preloadQueue.length > 0) {
            const batch = this.preloadQueue.splice(0, batchSize);

            try {
                await this.apiClient.crawl(batch, {
                    cache_mode: 'write_only',
                    concurrency: batchSize
                });

                console.log(`Preloaded ${batch.length} URLs`);

                // Rate limiting - wait between batches
                await new Promise(resolve => setTimeout(resolve, 1000));
            } catch (error) {
                console.error('Preloading batch failed:', error);
            }
        }
    }

    hashUrl(url) {
        // Simple hash function for cache keys
        return url.split('').reduce((hash, char) => {
            return ((hash << 5) - hash) + char.charCodeAt(0);
        }, 0).toString(36);
    }
}
```

### 3. WASM Optimization

#### Memory Management

```javascript
class WasmPerformanceMonitor {
    constructor() {
        this.extractionMetrics = [];
        this.memoryThresholds = {
            warning: 100 * 1024 * 1024, // 100MB
            critical: 500 * 1024 * 1024  // 500MB
        };
    }

    recordExtraction(stats) {
        this.extractionMetrics.push({
            timestamp: Date.now(),
            processingTime: stats.processing_time_ms,
            memoryUsed: stats.memory_used,
            nodesProcessed: stats.nodes_processed,
            success: stats.success
        });

        // Keep only last 1000 extractions
        if (this.extractionMetrics.length > 1000) {
            this.extractionMetrics.shift();
        }

        this.checkPerformanceThresholds(stats);
    }

    checkPerformanceThresholds(stats) {
        // Memory usage warnings
        if (stats.memory_used > this.memoryThresholds.critical) {
            console.error(`Critical memory usage: ${stats.memory_used} bytes`);
            this.triggerGarbageCollection();
        } else if (stats.memory_used > this.memoryThresholds.warning) {
            console.warn(`High memory usage: ${stats.memory_used} bytes`);
        }

        // Processing time warnings
        if (stats.processing_time_ms > 5000) {
            console.warn(`Slow extraction: ${stats.processing_time_ms}ms`);
        }
    }

    triggerGarbageCollection() {
        // Force garbage collection if available
        if (global.gc) {
            global.gc();
        }

        // Reset WASM component state
        fetch('/admin/wasm/reset', { method: 'POST' })
            .catch(err => console.error('WASM reset failed:', err));
    }

    getPerformanceAnalysis() {
        if (this.extractionMetrics.length === 0) {
            return null;
        }

        const recentMetrics = this.extractionMetrics.slice(-100);

        const avgProcessingTime = recentMetrics.reduce(
            (sum, m) => sum + m.processingTime, 0
        ) / recentMetrics.length;

        const avgMemoryUsage = recentMetrics.reduce(
            (sum, m) => sum + m.memoryUsed, 0
        ) / recentMetrics.length;

        const successRate = recentMetrics.filter(m => m.success).length / recentMetrics.length;

        const p95ProcessingTime = this.calculatePercentile(
            recentMetrics.map(m => m.processingTime), 0.95
        );

        return {
            avgProcessingTime,
            avgMemoryUsage,
            successRate,
            p95ProcessingTime,
            totalExtractions: this.extractionMetrics.length
        };
    }

    calculatePercentile(values, percentile) {
        const sorted = [...values].sort((a, b) => a - b);
        const index = Math.ceil(sorted.length * percentile) - 1;
        return sorted[index];
    }
}
```

### 4. Network Optimization

#### Connection Pooling

```javascript
class OptimizedHttpClient {
    constructor(options = {}) {
        this.maxConnections = options.maxConnections || 100;
        this.maxConnectionsPerHost = options.maxConnectionsPerHost || 10;
        this.timeout = options.timeout || 30000;
        this.keepAlive = options.keepAlive !== false;
        this.connectionPools = new Map();
    }

    createHttpClient() {
        return new (require('http').Agent)({
            keepAlive: this.keepAlive,
            keepAliveMsecs: 10000,
            maxSockets: this.maxConnectionsPerHost,
            maxTotalSockets: this.maxConnections,
            scheduling: 'fifo', // or 'lifo'
            timeout: this.timeout
        });
    }

    async fetchWithOptimization(url, options = {}) {
        const client = this.getOrCreateClient(url);

        const fetchOptions = {
            ...options,
            agent: client,
            timeout: this.timeout,
            headers: {
                'Connection': 'keep-alive',
                'Accept-Encoding': 'gzip, deflate, br',
                'User-Agent': 'RipTide-Crawler/1.0 (High-Performance)',
                ...options.headers
            }
        };

        try {
            const response = await fetch(url, fetchOptions);
            this.recordSuccess(url);
            return response;
        } catch (error) {
            this.recordFailure(url, error);
            throw error;
        }
    }

    getOrCreateClient(url) {
        const host = new URL(url).host;

        if (!this.connectionPools.has(host)) {
            this.connectionPools.set(host, this.createHttpClient());
        }

        return this.connectionPools.get(host);
    }

    recordSuccess(url) {
        // Track successful requests for optimization
        console.debug(`Successful request to ${url}`);
    }

    recordFailure(url, error) {
        console.warn(`Failed request to ${url}: ${error.message}`);
    }

    getPoolStats() {
        const stats = {};

        for (const [host, pool] of this.connectionPools) {
            stats[host] = {
                totalSockets: pool.totalSocketCount || 0,
                freeSockets: Object.keys(pool.freeSockets || {}).length,
                requests: Object.keys(pool.requests || {}).length
            };
        }

        return stats;
    }
}
```

### 5. Database and Cache Optimization

#### Redis Optimization

```javascript
class RedisOptimizer {
    constructor(redis) {
        this.redis = redis;
        this.compressionEnabled = true;
        this.pipelineSize = 100;
    }

    async optimizeRedisConfiguration() {
        // Set optimal Redis configuration
        const configs = {
            'maxmemory-policy': 'allkeys-lru',
            'timeout': '0',
            'tcp-keepalive': '60',
            'maxclients': '10000'
        };

        for (const [key, value] of Object.entries(configs)) {
            try {
                await this.redis.config('SET', key, value);
            } catch (error) {
                console.warn(`Failed to set Redis config ${key}: ${error.message}`);
            }
        }
    }

    async batchOperations(operations) {
        const pipeline = this.redis.pipeline();

        operations.forEach(op => {
            switch (op.type) {
                case 'set':
                    pipeline.setex(op.key, op.ttl, op.value);
                    break;
                case 'get':
                    pipeline.get(op.key);
                    break;
                case 'del':
                    pipeline.del(op.key);
                    break;
            }
        });

        return await pipeline.exec();
    }

    compressValue(value) {
        if (!this.compressionEnabled || typeof value !== 'string') {
            return value;
        }

        // Use gzip compression for large values
        if (value.length > 1024) {
            const zlib = require('zlib');
            return zlib.gzipSync(value).toString('base64');
        }

        return value;
    }

    decompressValue(value) {
        if (!this.compressionEnabled || typeof value !== 'string') {
            return value;
        }

        try {
            const zlib = require('zlib');
            const buffer = Buffer.from(value, 'base64');
            return zlib.gunzipSync(buffer).toString();
        } catch (error) {
            // Not compressed, return as-is
            return value;
        }
    }

    async cleanupExpiredKeys() {
        const cursor = '0';
        const pattern = 'crawl:v1:*';
        const count = 1000;

        let scanCursor = cursor;
        let deletedKeys = 0;

        do {
            const result = await this.redis.scan(scanCursor, 'MATCH', pattern, 'COUNT', count);
            scanCursor = result[0];
            const keys = result[1];

            if (keys.length > 0) {
                const pipeline = this.redis.pipeline();

                keys.forEach(key => {
                    pipeline.ttl(key);
                });

                const ttlResults = await pipeline.exec();
                const expiredKeys = keys.filter((key, index) => {
                    const ttl = ttlResults[index][1];
                    return ttl === -2; // Key doesn't exist or expired
                });

                if (expiredKeys.length > 0) {
                    await this.redis.del(...expiredKeys);
                    deletedKeys += expiredKeys.length;
                }
            }
        } while (scanCursor !== '0');

        console.log(`Cleaned up ${deletedKeys} expired keys`);
        return deletedKeys;
    }
}
```

## Performance Monitoring

### Real-Time Metrics Collection

```javascript
class PerformanceCollector {
    constructor() {
        this.metrics = {
            requests: new Map(),
            responses: new Map(),
            errors: new Map(),
            cache: new Map(),
            wasm: new Map()
        };

        this.intervals = {
            collect: null,
            report: null,
            cleanup: null
        };
    }

    start() {
        // Collect metrics every second
        this.intervals.collect = setInterval(() => {
            this.collectSystemMetrics();
        }, 1000);

        // Report metrics every minute
        this.intervals.report = setInterval(() => {
            this.reportMetrics();
        }, 60000);

        // Cleanup old metrics every hour
        this.intervals.cleanup = setInterval(() => {
            this.cleanupOldMetrics();
        }, 3600000);
    }

    stop() {
        Object.values(this.intervals).forEach(interval => {
            if (interval) clearInterval(interval);
        });
    }

    recordRequest(endpoint, method, duration, statusCode) {
        const key = `${method}:${endpoint}`;
        const now = Date.now();

        if (!this.metrics.requests.has(key)) {
            this.metrics.requests.set(key, []);
        }

        this.metrics.requests.get(key).push({
            timestamp: now,
            duration,
            statusCode,
            success: statusCode < 400
        });
    }

    recordCacheOperation(operation, hit, duration) {
        const now = Date.now();

        if (!this.metrics.cache.has(operation)) {
            this.metrics.cache.set(operation, []);
        }

        this.metrics.cache.get(operation).push({
            timestamp: now,
            hit,
            duration
        });
    }

    recordWasmExtraction(processingTime, memoryUsed, success) {
        const now = Date.now();

        if (!this.metrics.wasm.has('extractions')) {
            this.metrics.wasm.set('extractions', []);
        }

        this.metrics.wasm.get('extractions').push({
            timestamp: now,
            processingTime,
            memoryUsed,
            success
        });
    }

    collectSystemMetrics() {
        const memUsage = process.memoryUsage();
        const cpuUsage = process.cpuUsage();

        this.metrics.system = {
            memory: {
                rss: memUsage.rss,
                heapTotal: memUsage.heapTotal,
                heapUsed: memUsage.heapUsed,
                external: memUsage.external
            },
            cpu: {
                user: cpuUsage.user,
                system: cpuUsage.system
            },
            uptime: process.uptime(),
            timestamp: Date.now()
        };
    }

    calculateRequestMetrics(timeWindow = 60000) {
        const now = Date.now();
        const cutoff = now - timeWindow;
        const metrics = {};

        for (const [endpoint, requests] of this.metrics.requests) {
            const recentRequests = requests.filter(r => r.timestamp > cutoff);

            if (recentRequests.length === 0) continue;

            const durations = recentRequests.map(r => r.duration);
            const successCount = recentRequests.filter(r => r.success).length;

            metrics[endpoint] = {
                count: recentRequests.length,
                successRate: successCount / recentRequests.length,
                avgDuration: durations.reduce((a, b) => a + b, 0) / durations.length,
                p95Duration: this.percentile(durations, 0.95),
                p99Duration: this.percentile(durations, 0.99),
                throughput: recentRequests.length / (timeWindow / 1000)
            };
        }

        return metrics;
    }

    calculateCacheMetrics(timeWindow = 60000) {
        const now = Date.now();
        const cutoff = now - timeWindow;

        const allCacheOps = Array.from(this.metrics.cache.values())
            .flat()
            .filter(op => op.timestamp > cutoff);

        if (allCacheOps.length === 0) {
            return { hitRate: 0, avgDuration: 0, operations: 0 };
        }

        const hits = allCacheOps.filter(op => op.hit).length;
        const durations = allCacheOps.map(op => op.duration);

        return {
            hitRate: hits / allCacheOps.length,
            avgDuration: durations.reduce((a, b) => a + b, 0) / durations.length,
            operations: allCacheOps.length,
            throughput: allCacheOps.length / (timeWindow / 1000)
        };
    }

    percentile(values, p) {
        const sorted = [...values].sort((a, b) => a - b);
        const index = Math.ceil(sorted.length * p) - 1;
        return sorted[index] || 0;
    }

    reportMetrics() {
        const requestMetrics = this.calculateRequestMetrics();
        const cacheMetrics = this.calculateCacheMetrics();

        console.log('=== Performance Report ===');
        console.log('System:', this.metrics.system);
        console.log('Requests:', requestMetrics);
        console.log('Cache:', cacheMetrics);

        // Send to monitoring system
        this.sendToMonitoring({
            timestamp: Date.now(),
            system: this.metrics.system,
            requests: requestMetrics,
            cache: cacheMetrics
        });
    }

    cleanupOldMetrics() {
        const oneHourAgo = Date.now() - 3600000;

        for (const [key, requests] of this.metrics.requests) {
            this.metrics.requests.set(
                key,
                requests.filter(r => r.timestamp > oneHourAgo)
            );
        }

        for (const [key, operations] of this.metrics.cache) {
            this.metrics.cache.set(
                key,
                operations.filter(op => op.timestamp > oneHourAgo)
            );
        }
    }

    sendToMonitoring(data) {
        // Send to external monitoring service
        // fetch('https://monitoring.example.com/metrics', {
        //     method: 'POST',
        //     headers: { 'Content-Type': 'application/json' },
        //     body: JSON.stringify(data)
        // }).catch(console.error);
    }
}

// Usage
const perfCollector = new PerformanceCollector();
perfCollector.start();

// Integration with API handlers
app.use((req, res, next) => {
    const start = Date.now();

    res.on('finish', () => {
        const duration = Date.now() - start;
        perfCollector.recordRequest(req.path, req.method, duration, res.statusCode);
    });

    next();
});
```

## Load Testing

### Comprehensive Load Testing Script

```javascript
// load-test.js
const autocannon = require('autocannon');

class LoadTester {
    constructor(baseUrl = 'http://localhost:8080') {
        this.baseUrl = baseUrl;
        this.testResults = [];
    }

    async runCrawlLoadTest(options = {}) {
        const testConfig = {
            url: `${this.baseUrl}/crawl`,
            method: 'POST',
            headers: {
                'content-type': 'application/json'
            },
            body: JSON.stringify({
                urls: [
                    'https://httpbin.org/html',
                    'https://httpbin.org/json',
                    'https://httpbin.org/xml'
                ],
                options: {
                    concurrency: 3,
                    cache_mode: 'read_write'
                }
            }),
            connections: options.connections || 10,
            pipelining: options.pipelining || 1,
            duration: options.duration || 30,
            ...options
        };

        console.log(`Starting load test: ${testConfig.connections} connections for ${testConfig.duration}s`);

        const result = await autocannon(testConfig);
        this.testResults.push(result);

        return this.analyzeResults(result);
    }

    async runStreamingLoadTest(options = {}) {
        const testConfig = {
            url: `${this.baseUrl}/crawl/stream`,
            method: 'POST',
            headers: {
                'content-type': 'application/json',
                'x-session-id': `load-test-${Date.now()}`
            },
            body: JSON.stringify({
                urls: Array.from({ length: 20 }, (_, i) =>
                    `https://httpbin.org/delay/${Math.floor(Math.random() * 3) + 1}`
                ),
                options: { concurrency: 5 }
            }),
            connections: options.connections || 5,
            duration: options.duration || 60,
            ...options
        };

        console.log(`Starting streaming load test: ${testConfig.connections} connections for ${testConfig.duration}s`);

        const result = await autocannon(testConfig);
        return this.analyzeResults(result);
    }

    analyzeResults(result) {
        const analysis = {
            summary: {
                duration: result.duration,
                connections: result.connections,
                requests: result.requests,
                throughput: result.throughput,
                latency: result.latency,
                errors: result.errors
            },
            performance: {
                requestsPerSecond: result.requests.total / (result.duration / 1000),
                avgLatency: result.latency.mean,
                p95Latency: result.latency.p95,
                p99Latency: result.latency.p99,
                errorRate: result.errors / result.requests.total
            },
            recommendations: this.generateRecommendations(result)
        };

        console.log('Load Test Results:');
        console.log(`  Requests/sec: ${analysis.performance.requestsPerSecond.toFixed(2)}`);
        console.log(`  Avg Latency: ${analysis.performance.avgLatency.toFixed(2)}ms`);
        console.log(`  P95 Latency: ${analysis.performance.p95Latency.toFixed(2)}ms`);
        console.log(`  Error Rate: ${(analysis.performance.errorRate * 100).toFixed(2)}%`);

        return analysis;
    }

    generateRecommendations(result) {
        const recommendations = [];
        const rps = result.requests.total / (result.duration / 1000);
        const errorRate = result.errors / result.requests.total;

        if (rps < 50) {
            recommendations.push('Consider increasing server resources or optimizing concurrency');
        }

        if (result.latency.p95 > 5000) {
            recommendations.push('High P95 latency detected - review processing efficiency');
        }

        if (errorRate > 0.05) {
            recommendations.push('High error rate - investigate error causes and implement retries');
        }

        if (result.latency.mean > 2000) {
            recommendations.push('High average latency - consider caching and connection pooling');
        }

        return recommendations;
    }

    async runProgressiveLoadTest() {
        const connectionCounts = [1, 5, 10, 20, 50, 100];
        const results = [];

        for (const connections of connectionCounts) {
            console.log(`\nTesting with ${connections} connections...`);

            try {
                const result = await this.runCrawlLoadTest({
                    connections,
                    duration: 30
                });

                results.push({ connections, ...result });

                // Wait between tests
                await new Promise(resolve => setTimeout(resolve, 5000));
            } catch (error) {
                console.error(`Test failed with ${connections} connections:`, error.message);
                results.push({ connections, error: error.message });
            }
        }

        return this.analyzeProgressiveResults(results);
    }

    analyzeProgressiveResults(results) {
        console.log('\n=== Progressive Load Test Analysis ===');

        const validResults = results.filter(r => !r.error);

        if (validResults.length === 0) {
            console.log('All tests failed');
            return results;
        }

        // Find optimal connection count
        const optimal = validResults.reduce((best, current) => {
            const bestScore = best.performance.requestsPerSecond / best.performance.avgLatency;
            const currentScore = current.performance.requestsPerSecond / current.performance.avgLatency;
            return currentScore > bestScore ? current : best;
        });

        console.log(`Optimal configuration: ${optimal.connections} connections`);
        console.log(`Best performance: ${optimal.performance.requestsPerSecond.toFixed(2)} req/s @ ${optimal.performance.avgLatency.toFixed(2)}ms avg latency`);

        return { results, optimal };
    }
}

// Usage
async function runLoadTests() {
    const tester = new LoadTester();

    // Single load test
    await tester.runCrawlLoadTest({
        connections: 10,
        duration: 30
    });

    // Streaming load test
    await tester.runStreamingLoadTest({
        connections: 5,
        duration: 60
    });

    // Progressive load test to find optimal settings
    await tester.runProgressiveLoadTest();
}

// Run if script is executed directly
if (require.main === module) {
    runLoadTests().catch(console.error);
}

module.exports = LoadTester;
```

This comprehensive performance guide covers optimization strategies, monitoring techniques, and load testing approaches to ensure the RipTide API operates at peak efficiency.