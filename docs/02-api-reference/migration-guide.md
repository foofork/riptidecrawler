# Migration Guide: From Placeholder to Production

## Overview

This guide helps migrate from placeholder implementations to the production RipTide API with full WASM extractor integration, dynamic rendering capabilities, and enhanced streaming features. It covers breaking changes, migration strategies, and step-by-step upgrade paths.

## Breaking Changes Summary

### API Changes

| Change | Version | Impact | Migration Required |
|--------|---------|--------|-------------------|
| WASM Extractor Integration | v1.0.0 | High | Update extraction calls |
| Enhanced `/render` Endpoint | v1.0.0 | Medium | Update rendering requests |
| Session Management | v1.0.0 | Low | Optional headers |
| Streaming Buffer Management | v1.0.0 | Low | Update buffer sizes |
| Error Response Format | v1.0.0 | Medium | Update error handling |

### Removed Features

- **Placeholder WASM calls**: Replaced with actual Component Model integration
- **Mock rendering**: Replaced with real headless browser rendering
- **Static extraction modes**: Enhanced with dynamic mode detection
- **Legacy cache keys**: New versioned cache key format

### New Features

- **Advanced stealth capabilities**: Anti-detection measures
- **PDF processing**: Native PDF content extraction
- **Adaptive rendering**: Intelligent mode selection
- **Enhanced streaming**: Better backpressure handling
- **Session persistence**: Cross-request state management

## Migration Strategies

### 1. Gradual Migration (Recommended)

Deploy both old and new versions side-by-side with feature flags:

```javascript
class MigrationManager {
    constructor(config) {
        this.oldApiUrl = config.oldApiUrl;
        this.newApiUrl = config.newApiUrl;
        this.migrationFlags = config.migrationFlags;
        this.fallbackEnabled = config.fallbackEnabled || true;
    }

    async crawl(urls, options = {}) {
        // Check migration flag for this feature
        if (this.migrationFlags.useNewCrawlApi) {
            try {
                return await this.newCrawl(urls, options);
            } catch (error) {
                if (this.fallbackEnabled) {
                    console.warn('New API failed, falling back to old API:', error);
                    return await this.oldCrawl(urls, options);
                }
                throw error;
            }
        }

        return await this.oldCrawl(urls, options);
    }

    async newCrawl(urls, options) {
        const response = await fetch(`${this.newApiUrl}/crawl`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'X-Migration-Source': 'v1.0.0'
            },
            body: JSON.stringify({
                urls: this.validateUrls(urls),
                options: this.migrateOptions(options)
            })
        });

        if (!response.ok) {
            throw new Error(`New API error: ${response.status}`);
        }

        const result = await response.json();
        return this.transformNewResponse(result);
    }

    async oldCrawl(urls, options) {
        // Legacy API call
        const response = await fetch(`${this.oldApiUrl}/crawl`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ urls, options })
        });

        return response.json();
    }

    validateUrls(urls) {
        // Ensure URLs meet new validation requirements
        return urls.filter(url => {
            try {
                new URL(url);
                return true;
            } catch {
                console.warn(`Invalid URL filtered out: ${url}`);
                return false;
            }
        });
    }

    migrateOptions(oldOptions) {
        const newOptions = { ...oldOptions };

        // Migrate cache mode if needed
        if (oldOptions.useCache !== undefined) {
            newOptions.cache_mode = oldOptions.useCache ? 'read_write' : 'disabled';
            delete newOptions.useCache;
        }

        // Migrate concurrency settings
        if (oldOptions.parallel !== undefined) {
            newOptions.concurrency = Math.min(oldOptions.parallel, 10);
            delete newOptions.parallel;
        }

        return newOptions;
    }

    transformNewResponse(newResponse) {
        // Transform new response format to match legacy expectations
        return {
            ...newResponse,
            // Add any legacy fields that consumers expect
            success: newResponse.successful > 0,
            total: newResponse.total_urls
        };
    }
}

// Usage with feature flags
const migrationManager = new MigrationManager({
    oldApiUrl: 'https://old-api.example.com',
    newApiUrl: 'https://api.riptide.dev',
    migrationFlags: {
        useNewCrawlApi: process.env.ENABLE_NEW_CRAWL === 'true',
        useNewRenderApi: process.env.ENABLE_NEW_RENDER === 'true',
        useNewStreaming: process.env.ENABLE_NEW_STREAMING === 'true'
    },
    fallbackEnabled: true
});
```

### 2. Blue-Green Deployment

Switch traffic gradually between old and new deployments:

```nginx
# Nginx configuration for blue-green deployment
upstream old_api {
    server old-api-1:8080;
    server old-api-2:8080;
}

upstream new_api {
    server new-api-1:8080;
    server new-api-2:8080;
}

# Split traffic: 80% old, 20% new
split_clients $remote_addr $backend {
    20%     new_api;
    *       old_api;
}

server {
    listen 443 ssl;
    server_name api.example.com;

    location / {
        proxy_pass http://$backend;

        # Add migration headers
        proxy_set_header X-Migration-Backend $backend;
        proxy_set_header X-Migration-Timestamp $time_iso8601;
    }
}
```

### 3. A/B Testing Migration

Test new features with specific user segments:

```javascript
class ABTestMigration {
    constructor(userId, experimentConfig) {
        this.userId = userId;
        this.experimentConfig = experimentConfig;
        this.userSegment = this.calculateUserSegment(userId);
    }

    calculateUserSegment(userId) {
        // Simple hash-based segmentation
        const hash = userId.split('').reduce((acc, char) => {
            return acc + char.charCodeAt(0);
        }, 0);

        return hash % 100; // 0-99
    }

    shouldUseNewFeature(featureName) {
        const experiment = this.experimentConfig[featureName];
        if (!experiment) return false;

        return this.userSegment < experiment.percentage;
    }

    async crawlWithABTest(urls, options) {
        if (this.shouldUseNewFeature('newCrawlApi')) {
            // Track experiment participation
            this.trackExperiment('newCrawlApi', 'treatment');

            try {
                const result = await this.newApiCrawl(urls, options);
                this.trackExperiment('newCrawlApi', 'success');
                return result;
            } catch (error) {
                this.trackExperiment('newCrawlApi', 'error', { error: error.message });
                throw error;
            }
        } else {
            this.trackExperiment('newCrawlApi', 'control');
            return await this.legacyApiCrawl(urls, options);
        }
    }

    trackExperiment(feature, event, metadata = {}) {
        const data = {
            userId: this.userId,
            feature,
            event,
            userSegment: this.userSegment,
            timestamp: new Date().toISOString(),
            ...metadata
        };

        // Send to analytics
        fetch('/analytics/experiment', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(data)
        }).catch(console.error);
    }
}
```

## Step-by-Step Migration

### Phase 1: Environment Preparation

#### 1.1 Update Dependencies

```bash
# Update to latest API client
npm install @riptide/api-client@latest

# Install new dependencies for enhanced features
npm install ws eventsource compression

# Update development tools
npm install @riptide/dev-tools@latest
```

#### 1.2 Environment Configuration

```bash
# .env.migration
# Old API (for fallback)
OLD_API_URL=https://old-api.example.com
OLD_API_KEY=your-old-api-key

# New API
NEW_API_URL=https://api.riptide.dev
SERPER_API_KEY=your-serper-key  # Required for deep search

# Migration settings
MIGRATION_ENABLED=true
FALLBACK_ENABLED=true
MIGRATION_PERCENTAGE=10  # Start with 10% traffic

# Performance settings
REDIS_URL=redis://localhost:6379
WASM_COMPONENT_PATH=./wasm/riptide-extractor.wasm
```

#### 1.3 Health Check Integration

```javascript
class MigrationHealthChecker {
    constructor(config) {
        this.config = config;
        this.healthStatus = {
            oldApi: 'unknown',
            newApi: 'unknown',
            migration: 'disabled'
        };
    }

    async checkHealth() {
        // Check old API
        try {
            const oldResponse = await fetch(`${this.config.oldApiUrl}/health`);
            this.healthStatus.oldApi = oldResponse.ok ? 'healthy' : 'unhealthy';
        } catch (error) {
            this.healthStatus.oldApi = 'unhealthy';
        }

        // Check new API
        try {
            const newResponse = await fetch(`${this.config.newApiUrl}/healthz`);
            this.healthStatus.newApi = newResponse.ok ? 'healthy' : 'unhealthy';
        } catch (error) {
            this.healthStatus.newApi = 'unhealthy';
        }

        // Update migration status
        this.healthStatus.migration = this.shouldEnableMigration() ? 'enabled' : 'disabled';

        return this.healthStatus;
    }

    shouldEnableMigration() {
        return this.config.migrationEnabled &&
               this.healthStatus.newApi === 'healthy';
    }
}
```

### Phase 2: API Client Migration

#### 2.1 Update Crawl Requests

**Before (Legacy):**

```javascript
// Old API format
const result = await fetch('/crawl', {
    method: 'POST',
    body: JSON.stringify({
        urls: ['https://example.com'],
        options: {
            useCache: true,
            parallel: 5,
            timeout: 30
        }
    })
});
```

**After (New API):**

```javascript
// New API format with enhanced options
const result = await fetch('/crawl', {
    method: 'POST',
    headers: {
        'Content-Type': 'application/json',
        'X-Session-ID': generateSessionId()  // Optional but recommended
    },
    body: JSON.stringify({
        urls: ['https://example.com'],
        options: {
            cache_mode: 'read_write',  // Changed from useCache
            concurrency: 5,            // Changed from parallel
            timeout_seconds: 30,       // Changed from timeout
            extract_mode: 'article',   // New: extraction strategy
            gate_mode: 'adaptive',     // New: routing strategy
            quality_threshold: 0.7     // New: quality filtering
        }
    })
});
```

#### 2.2 Update Response Handling

**Before:**

```javascript
// Legacy response format
const data = await response.json();
console.log(`Processed ${data.total} URLs`);
console.log(`Success rate: ${data.success ? '100%' : '0%'}`);

data.results.forEach(result => {
    if (result.content) {
        console.log(`Title: ${result.content.title}`);
        console.log(`Text: ${result.content.text.substring(0, 100)}...`);
    }
});
```

**After:**

```javascript
// New response format with enhanced data
const data = await response.json();
console.log(`Processed ${data.total_urls} URLs`);
console.log(`Success rate: ${(data.successful / data.total_urls * 100).toFixed(1)}%`);
console.log(`Cache hit rate: ${(data.statistics.cache_hit_rate * 100).toFixed(1)}%`);

data.results.forEach(result => {
    if (result.document) {
        console.log(`Title: ${result.document.title}`);
        console.log(`Quality Score: ${result.quality_score}`);
        console.log(`Processing Time: ${result.processing_time_ms}ms`);
        console.log(`Gate Decision: ${result.gate_decision}`);

        // New: Enhanced content data
        if (result.document.reading_time) {
            console.log(`Reading Time: ${result.document.reading_time} minutes`);
        }
        if (result.document.word_count) {
            console.log(`Word Count: ${result.document.word_count}`);
        }
    }
});
```

#### 2.3 Add Enhanced Rendering

**New rendering capabilities:**

```javascript
// Enhanced rendering with dynamic content support
async function migrateToEnhancedRendering(url, options = {}) {
    const renderRequest = {
        url,
        mode: 'adaptive',  // Let API choose optimal strategy
        dynamic_config: {
            wait_conditions: [
                {
                    type: 'element_visible',
                    selector: '.content-loaded',
                    timeout: 10000
                }
            ],
            viewport: {
                width: 1920,
                height: 1080
            },
            javascript_enabled: true,
            timeout: 30000
        },
        stealth_config: {
            user_agent_rotation: true,
            header_randomization: true,
            timing_jitter: true
        },
        output_format: 'markdown',
        capture_artifacts: false,
        timeout: 45
    };

    const response = await fetch('/render', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
            'X-Session-ID': generateSessionId()
        },
        body: JSON.stringify(renderRequest)
    });

    const result = await response.json();

    if (result.success) {
        console.log(`Rendered with ${result.mode} mode`);
        console.log(`Processing time: ${result.stats.total_time_ms}ms`);

        if (result.stealth_applied.length > 0) {
            console.log(`Stealth measures: ${result.stealth_applied.join(', ')}`);
        }

        return result.content;
    } else {
        throw new Error(`Rendering failed: ${result.error?.message}`);
    }
}
```

### Phase 3: Streaming Migration

#### 3.1 Migrate from Polling to Streaming

**Before (Polling):**

```javascript
// Legacy polling approach
async function pollForResults(urls) {
    const jobId = await startCrawlJob(urls);

    while (true) {
        const status = await checkJobStatus(jobId);

        if (status.completed) {
            return status.results;
        }

        await new Promise(resolve => setTimeout(resolve, 1000));
    }
}
```

**After (Streaming):**

```javascript
// New streaming approach
async function streamCrawlResults(urls) {
    const sessionId = generateSessionId();
    const results = [];

    const response = await fetch('/crawl/stream', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
            'X-Session-ID': sessionId,
            'X-Buffer-Size': '256'
        },
        body: JSON.stringify({ urls, options: { concurrency: 3 } })
    });

    const reader = response.body.getReader();
    const decoder = new TextDecoder();

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
                        case 'result':
                            results.push(event);
                            onProgress?.(results.length, urls.length);
                            break;
                        case 'summary':
                            return { results, summary: event };
                        case 'error':
                            console.error('Stream error:', event.error);
                            break;
                    }
                } catch (error) {
                    console.warn('Failed to parse stream event:', error);
                }
            }
        }
    } finally {
        reader.releaseLock();
    }

    return { results };
}
```

#### 3.2 Add Error Recovery

```javascript
class StreamingMigrationClient {
    constructor(baseUrl) {
        this.baseUrl = baseUrl;
        this.maxRetries = 3;
        this.retryDelay = 1000;
    }

    async streamWithRetry(endpoint, payload, options = {}) {
        let lastError;

        for (let attempt = 1; attempt <= this.maxRetries; attempt++) {
            try {
                return await this.createStream(endpoint, payload, options);
            } catch (error) {
                lastError = error;
                console.warn(`Stream attempt ${attempt} failed:`, error.message);

                if (attempt < this.maxRetries) {
                    const delay = this.retryDelay * Math.pow(2, attempt - 1);
                    console.log(`Retrying in ${delay}ms...`);
                    await new Promise(resolve => setTimeout(resolve, delay));
                }
            }
        }

        // Fall back to non-streaming API
        console.warn('All streaming attempts failed, falling back to batch API');
        return await this.fallbackToBatchApi(payload);
    }

    async fallbackToBatchApi(payload) {
        const response = await fetch(`${this.baseUrl}/crawl`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(payload)
        });

        const result = await response.json();

        // Transform batch response to streaming format
        return {
            results: result.results.map(r => ({ event: 'result', ...r })),
            summary: {
                event: 'summary',
                total_urls: result.total_urls,
                successful: result.successful,
                failed: result.failed,
                total_time_ms: result.statistics.total_processing_time_ms
            }
        };
    }
}
```

### Phase 4: Error Handling Migration

#### 4.1 Update Error Response Handling

**Before:**

```javascript
// Legacy error handling
try {
    const result = await apiCall();
} catch (error) {
    if (error.status === 500) {
        console.error('Server error');
    } else {
        console.error('Request failed:', error.message);
    }
}
```

**After:**

```javascript
// Enhanced error handling with detailed error types
try {
    const result = await apiCall();
} catch (error) {
    const errorData = await error.response?.json();

    if (errorData?.error) {
        const { type, message, retryable, status } = errorData.error;

        switch (type) {
            case 'validation_error':
                console.error('Invalid request:', message);
                // Don't retry validation errors
                break;

            case 'rate_limited':
                console.warn('Rate limited:', message);
                if (retryable) {
                    await handleRateLimit(error);
                }
                break;

            case 'fetch_error':
            case 'timeout_error':
            case 'dependency_error':
                console.warn('Retryable error:', message);
                if (retryable) {
                    await retryWithBackoff(apiCall);
                }
                break;

            default:
                console.error('API error:', message);
        }

        // Track error for monitoring
        trackError(type, message, status);
    } else {
        console.error('Unknown error:', error);
    }
}

async function handleRateLimit(error) {
    const retryAfter = error.response?.headers?.get('retry-after') || 60;
    console.log(`Waiting ${retryAfter} seconds before retry...`);
    await new Promise(resolve => setTimeout(resolve, retryAfter * 1000));
}
```

### Phase 5: Performance Optimization

#### 5.1 Implement Caching Strategy

```javascript
class MigrationCacheStrategy {
    constructor(redis, config) {
        this.redis = redis;
        this.config = config;
        this.migrationPrefix = 'migration:v1:';
    }

    async migrateExistingCache() {
        // Migrate old cache keys to new format
        const oldKeys = await this.redis.keys('crawl:*');

        for (const oldKey of oldKeys) {
            try {
                const value = await this.redis.get(oldKey);
                const newKey = this.transformCacheKey(oldKey);

                // Copy to new key with updated TTL
                await this.redis.setex(newKey, 3600, value);

                // Mark old key for deletion
                await this.redis.expire(oldKey, 86400); // 24 hours grace period
            } catch (error) {
                console.warn(`Failed to migrate cache key ${oldKey}:`, error);
            }
        }
    }

    transformCacheKey(oldKey) {
        // Transform: crawl:example.com:article -> crawl:v1:example.com:article:hash
        const parts = oldKey.split(':');
        if (parts.length >= 3) {
            const domain = parts[1];
            const mode = parts[2] || 'article';
            const hash = this.generateHash(domain + mode);
            return `${this.migrationPrefix}${domain}:${mode}:${hash}`;
        }
        return oldKey;
    }

    generateHash(input) {
        // Simple hash for cache keys
        return input.split('').reduce((hash, char) => {
            return ((hash << 5) - hash) + char.charCodeAt(0);
        }, 0).toString(36);
    }
}
```

#### 5.2 Add Performance Monitoring

```javascript
class MigrationMetrics {
    constructor() {
        this.metrics = {
            legacy: { requests: 0, errors: 0, totalTime: 0 },
            new: { requests: 0, errors: 0, totalTime: 0 },
            migration: { enabled: false, percentage: 0 }
        };
    }

    recordLegacyRequest(duration, success) {
        this.metrics.legacy.requests++;
        this.metrics.legacy.totalTime += duration;
        if (!success) this.metrics.legacy.errors++;
    }

    recordNewRequest(duration, success) {
        this.metrics.new.requests++;
        this.metrics.new.totalTime += duration;
        if (!success) this.metrics.new.errors++;
    }

    getComparisonReport() {
        const legacy = this.metrics.legacy;
        const newApi = this.metrics.new;

        return {
            legacy: {
                avgResponseTime: legacy.requests ? legacy.totalTime / legacy.requests : 0,
                errorRate: legacy.requests ? legacy.errors / legacy.requests : 0,
                totalRequests: legacy.requests
            },
            new: {
                avgResponseTime: newApi.requests ? newApi.totalTime / newApi.requests : 0,
                errorRate: newApi.requests ? newApi.errors / newApi.requests : 0,
                totalRequests: newApi.requests
            },
            improvement: this.calculateImprovement(legacy, newApi)
        };
    }

    calculateImprovement(legacy, newApi) {
        if (legacy.requests === 0 || newApi.requests === 0) {
            return null;
        }

        const legacyAvg = legacy.totalTime / legacy.requests;
        const newAvg = newApi.totalTime / newApi.requests;
        const legacyErrorRate = legacy.errors / legacy.requests;
        const newErrorRate = newApi.errors / newApi.requests;

        return {
            responseTimeImprovement: ((legacyAvg - newAvg) / legacyAvg * 100).toFixed(2),
            errorRateImprovement: ((legacyErrorRate - newErrorRate) / legacyErrorRate * 100).toFixed(2)
        };
    }
}
```

## Rollback Strategy

### Emergency Rollback

```javascript
class EmergencyRollback {
    constructor(config) {
        this.config = config;
        this.rollbackTriggers = {
            errorRateThreshold: 0.1,    // 10% error rate
            latencyThreshold: 5000,     // 5 second response time
            availabilityThreshold: 0.95 // 95% availability
        };
    }

    async checkRollbackConditions() {
        const health = await this.getCurrentHealth();

        const shouldRollback =
            health.errorRate > this.rollbackTriggers.errorRateThreshold ||
            health.avgLatency > this.rollbackTriggers.latencyThreshold ||
            health.availability < this.rollbackTriggers.availabilityThreshold;

        if (shouldRollback) {
            console.error('Rollback conditions met:', health);
            await this.executeRollback();
        }

        return shouldRollback;
    }

    async executeRollback() {
        console.log('Executing emergency rollback...');

        // 1. Disable new API traffic
        await this.disableMigration();

        // 2. Alert operations team
        await this.sendAlert('Emergency rollback executed');

        // 3. Update load balancer to route to legacy API
        await this.updateTrafficRouting('legacy');

        // 4. Scale up legacy infrastructure if needed
        await this.scaleUpLegacyServices();

        console.log('Emergency rollback completed');
    }

    async disableMigration() {
        // Update feature flags
        await this.updateFeatureFlag('migration_enabled', false);
    }

    async updateTrafficRouting(target) {
        // Update load balancer configuration
        // This would integrate with your infrastructure automation
    }
}
```

## Validation and Testing

### Migration Validation Checklist

- [ ] **API Compatibility**: All legacy endpoints work with new client
- [ ] **Data Integrity**: Response data matches expected format
- [ ] **Performance**: New API meets or exceeds performance benchmarks
- [ ] **Error Handling**: All error scenarios are properly handled
- [ ] **Fallback**: Legacy API fallback works correctly
- [ ] **Monitoring**: Metrics and logging are properly configured
- [ ] **Security**: Authentication and authorization work correctly
- [ ] **Caching**: Cache migration completed without data loss

### Automated Migration Tests

```javascript
// migration-tests.js
describe('API Migration Tests', () => {
    const migrationClient = new MigrationManager(testConfig);

    test('crawl API compatibility', async () => {
        const urls = ['https://httpbin.org/html'];
        const options = { concurrency: 1 };

        const oldResult = await migrationClient.oldCrawl(urls, options);
        const newResult = await migrationClient.newCrawl(urls, options);

        // Verify response structure compatibility
        expect(newResult.total_urls).toBe(oldResult.total || 1);
        expect(newResult.results).toHaveLength(1);
        expect(newResult.results[0].document).toBeDefined();
    });

    test('error handling compatibility', async () => {
        const invalidUrls = ['not-a-url'];

        await expect(migrationClient.newCrawl(invalidUrls))
            .rejects.toThrow();
    });

    test('streaming fallback', async () => {
        // Test that streaming falls back to batch API when needed
        const result = await migrationClient.streamWithFallback(['https://httpbin.org/html']);
        expect(result.results).toHaveLength(1);
    });
});

// Run tests
npm test -- migration-tests.js
```

This comprehensive migration guide provides structured approaches to safely transition from placeholder implementations to the full production RipTide API while maintaining backward compatibility and minimizing risks.