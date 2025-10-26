# Authentication and Security Guide

## Overview

The RipTide API implements comprehensive security measures including authentication, rate limiting, input validation, and protection against common attack vectors. This guide covers security configurations, best practices, and implementation details.

## Authentication Methods

### API Key Authentication

For endpoints requiring authentication (like deep search with Serper.dev):

```http
POST /deepsearch
Content-Type: application/json
X-API-Key: your-api-key-here

{
  "query": "machine learning best practices",
  "limit": 10
}
```

### Environment Variables

Sensitive configuration is managed through environment variables:

```bash
# Required for deep search functionality
export SERPER_API_KEY="your-serper-api-key"

# Optional: Custom API authentication
export RIPTIDE_API_KEY="your-riptide-api-key"

# Redis credentials (if authentication enabled)
export REDIS_PASSWORD="your-redis-password"

# Database credentials (if applicable)
export DATABASE_URL="postgresql://user:pass@localhost/riptide"
```

### Session-Based Authentication

For extended operations, use session-based authentication:

```javascript
class AuthenticatedRipTideClient {
    constructor(apiKey) {
        this.apiKey = apiKey;
        this.sessionToken = null;
        this.tokenExpiry = null;
    }

    async authenticate() {
        const response = await fetch('/auth/login', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'X-API-Key': this.apiKey
            },
            body: JSON.stringify({
                client_id: 'riptide-client',
                grant_type: 'api_key'
            })
        });

        if (!response.ok) {
            throw new Error(`Authentication failed: ${response.status}`);
        }

        const auth = await response.json();
        this.sessionToken = auth.access_token;
        this.tokenExpiry = Date.now() + (auth.expires_in * 1000);

        return auth;
    }

    async makeAuthenticatedRequest(endpoint, options = {}) {
        // Check if token needs renewal
        if (!this.sessionToken || Date.now() >= this.tokenExpiry - 60000) {
            await this.authenticate();
        }

        const headers = {
            'Authorization': `Bearer ${this.sessionToken}`,
            'Content-Type': 'application/json',
            ...options.headers
        };

        return fetch(endpoint, {
            ...options,
            headers
        });
    }

    async crawl(urls, options = {}) {
        const response = await this.makeAuthenticatedRequest('/crawl', {
            method: 'POST',
            body: JSON.stringify({ urls, options })
        });

        return response.json();
    }
}

// Usage
const client = new AuthenticatedRipTideClient('your-api-key');
const result = await client.crawl(['https://example.com']);
```

## Rate Limiting

### Rate Limit Headers

All responses include rate limiting information:

```http
HTTP/1.1 200 OK
Content-Type: application/json
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 95
X-RateLimit-Reset: 1640995800
X-RateLimit-Window: 60
Retry-After: 5

{
  "results": [...]
}
```

**Header Descriptions:**
- `X-RateLimit-Limit`: Maximum requests per window
- `X-RateLimit-Remaining`: Remaining requests in current window
- `X-RateLimit-Reset`: Unix timestamp when window resets
- `X-RateLimit-Window`: Window duration in seconds
- `Retry-After`: Seconds to wait before retry (when rate limited)

### Rate Limit Tiers

Different endpoints have different rate limits:

| Endpoint | Limit | Window | Notes |
|----------|-------|--------|-------|
| `/crawl` | 100 requests | 1 minute | Per IP address |
| `/deepsearch` | 20 requests | 1 minute | Per API key |
| `/render` | 50 requests | 1 minute | Per session |
| `/crawl/stream` | 10 streams | 1 minute | Concurrent limit |
| `/healthz` | 1000 requests | 1 minute | Monitoring friendly |
| `/metrics` | 200 requests | 1 minute | For Prometheus scraping |

### Rate Limit Implementation

```javascript
class RateLimitHandler {
    constructor() {
        this.requests = new Map(); // IP -> timestamps
        this.apiRequests = new Map(); // API key -> timestamps
    }

    checkRateLimit(req, endpoint) {
        const identifier = this.getIdentifier(req, endpoint);
        const limits = this.getLimitsForEndpoint(endpoint);
        const now = Date.now();

        if (!this.requests.has(identifier)) {
            this.requests.set(identifier, []);
        }

        const requests = this.requests.get(identifier);

        // Remove requests outside the window
        const validRequests = requests.filter(
            timestamp => now - timestamp < limits.window * 1000
        );

        this.requests.set(identifier, validRequests);

        if (validRequests.length >= limits.limit) {
            const oldestRequest = Math.min(...validRequests);
            const resetTime = oldestRequest + (limits.window * 1000);

            throw new RateLimitError({
                limit: limits.limit,
                remaining: 0,
                reset: Math.floor(resetTime / 1000),
                retryAfter: Math.ceil((resetTime - now) / 1000)
            });
        }

        // Add current request
        validRequests.push(now);
        this.requests.set(identifier, validRequests);

        return {
            limit: limits.limit,
            remaining: limits.limit - validRequests.length,
            reset: Math.floor((now + limits.window * 1000) / 1000),
            window: limits.window
        };
    }

    getIdentifier(req, endpoint) {
        // Use API key for authenticated endpoints, IP for others
        if (endpoint.startsWith('/deepsearch') && req.headers['x-api-key']) {
            return `api:${req.headers['x-api-key']}`;
        }

        return `ip:${req.ip}`;
    }

    getLimitsForEndpoint(endpoint) {
        const limits = {
            '/crawl': { limit: 100, window: 60 },
            '/deepsearch': { limit: 20, window: 60 },
            '/render': { limit: 50, window: 60 },
            '/crawl/stream': { limit: 10, window: 60 },
            '/healthz': { limit: 1000, window: 60 },
            '/metrics': { limit: 200, window: 60 }
        };

        return limits[endpoint] || { limit: 60, window: 60 };
    }
}
```

## Input Validation and Sanitization

### URL Validation

```javascript
class URLValidator {
    constructor() {
        this.allowedProtocols = ['http:', 'https:'];
        this.blockedDomains = [
            'localhost',
            '127.0.0.1',
            '0.0.0.0',
            '::1',
            '169.254.0.0/16', // AWS metadata
            '10.0.0.0/8',     // Private networks
            '172.16.0.0/12',
            '192.168.0.0/16'
        ];
        this.maxUrlLength = 2048;
        this.maxUrls = 100;
    }

    validateUrls(urls) {
        if (!Array.isArray(urls)) {
            throw new ValidationError('URLs must be an array');
        }

        if (urls.length === 0) {
            throw new ValidationError('URLs array cannot be empty');
        }

        if (urls.length > this.maxUrls) {
            throw new ValidationError(`Maximum ${this.maxUrls} URLs allowed`);
        }

        return urls.map(url => this.validateSingleUrl(url));
    }

    validateSingleUrl(url) {
        if (typeof url !== 'string') {
            throw new ValidationError('URL must be a string');
        }

        if (url.length > this.maxUrlLength) {
            throw new ValidationError(`URL too long (max ${this.maxUrlLength} characters)`);
        }

        let parsedUrl;
        try {
            parsedUrl = new URL(url);
        } catch (error) {
            throw new ValidationError(`Invalid URL format: ${error.message}`);
        }

        // Check protocol
        if (!this.allowedProtocols.includes(parsedUrl.protocol)) {
            throw new ValidationError(`Protocol not allowed: ${parsedUrl.protocol}`);
        }

        // Check for blocked domains/IPs
        if (this.isBlockedDomain(parsedUrl.hostname)) {
            throw new ValidationError(`Domain not allowed: ${parsedUrl.hostname}`);
        }

        // Additional security checks
        this.performSecurityChecks(parsedUrl);

        return parsedUrl.href;
    }

    isBlockedDomain(hostname) {
        // Check exact matches
        if (this.blockedDomains.some(blocked => hostname === blocked)) {
            return true;
        }

        // Check IP ranges (simplified)
        if (this.isPrivateIP(hostname)) {
            return true;
        }

        return false;
    }

    isPrivateIP(hostname) {
        // Basic private IP detection
        const privateRanges = [
            /^10\./,
            /^172\.(1[6-9]|2[0-9]|3[0-1])\./,
            /^192\.168\./,
            /^127\./,
            /^169\.254\./
        ];

        return privateRanges.some(range => range.test(hostname));
    }

    performSecurityChecks(url) {
        // Check for suspicious patterns
        const suspiciousPatterns = [
            /javascript:/i,
            /data:/i,
            /vbscript:/i,
            /file:/i,
            /ftp:/i
        ];

        if (suspiciousPatterns.some(pattern => pattern.test(url.href))) {
            throw new ValidationError('URL contains suspicious patterns');
        }

        // Check for overly long paths (potential DoS)
        if (url.pathname.length > 1000) {
            throw new ValidationError('URL path too long');
        }

        // Check for excessive query parameters
        const params = new URLSearchParams(url.search);
        if (params.toString().length > 2000) {
            throw new ValidationError('URL query string too long');
        }
    }
}

// Usage in API handlers
const urlValidator = new URLValidator();

app.post('/crawl', (req, res) => {
    try {
        const validatedUrls = urlValidator.validateUrls(req.body.urls);
        // Proceed with crawling
    } catch (error) {
        res.status(400).json({
            error: {
                type: 'validation_error',
                message: error.message,
                retryable: false,
                status: 400
            }
        });
    }
});
```

### Request Sanitization

```javascript
class RequestSanitizer {
    sanitizeCrawlRequest(body) {
        const sanitized = {};

        // Sanitize URLs
        if (body.urls) {
            sanitized.urls = this.sanitizeUrls(body.urls);
        }

        // Sanitize options
        if (body.options) {
            sanitized.options = this.sanitizeCrawlOptions(body.options);
        }

        return sanitized;
    }

    sanitizeUrls(urls) {
        if (!Array.isArray(urls)) {
            throw new ValidationError('URLs must be an array');
        }

        return urls.map(url => {
            if (typeof url !== 'string') {
                throw new ValidationError('URL must be a string');
            }

            // Remove control characters and normalize
            return url.trim()
                     .replace(/[\x00-\x1F\x7F]/g, '') // Remove control chars
                     .normalize('NFC'); // Unicode normalization
        });
    }

    sanitizeCrawlOptions(options) {
        const sanitized = {};

        // Concurrency limits
        if (options.concurrency !== undefined) {
            sanitized.concurrency = Math.max(1, Math.min(10, parseInt(options.concurrency) || 3));
        }

        // Cache mode validation
        if (options.cache_mode !== undefined) {
            const validModes = ['read_only', 'write_only', 'read_write', 'disabled'];
            sanitized.cache_mode = validModes.includes(options.cache_mode)
                ? options.cache_mode
                : 'read_write';
        }

        // Timeout limits
        if (options.timeout_seconds !== undefined) {
            sanitized.timeout_seconds = Math.max(5, Math.min(300, parseInt(options.timeout_seconds) || 30));
        }

        // User agent sanitization
        if (options.user_agent !== undefined) {
            sanitized.user_agent = this.sanitizeUserAgent(options.user_agent);
        }

        // Extract mode validation
        if (options.extract_mode !== undefined) {
            const validModes = ['article', 'full', 'metadata'];
            sanitized.extract_mode = validModes.includes(options.extract_mode)
                ? options.extract_mode
                : 'article';
        }

        return sanitized;
    }

    sanitizeUserAgent(userAgent) {
        if (typeof userAgent !== 'string') {
            return 'RipTide-Crawler/1.0';
        }

        // Remove potentially dangerous characters
        const sanitized = userAgent
            .replace(/[<>'"]/g, '')
            .replace(/[\r\n]/g, ' ')
            .trim()
            .substring(0, 200); // Limit length

        return sanitized || 'RipTide-Crawler/1.0';
    }

    sanitizeQuery(query) {
        if (typeof query !== 'string') {
            throw new ValidationError('Query must be a string');
        }

        // Remove potential injection patterns
        const sanitized = query
            .replace(/[<>'"]/g, '') // Remove HTML/script chars
            .replace(/\0/g, '') // Remove null bytes
            .trim()
            .substring(0, 500); // Limit length

        if (sanitized.length === 0) {
            throw new ValidationError('Query cannot be empty');
        }

        return sanitized;
    }
}
```

## HTTPS and TLS Configuration

### TLS Settings

```rust
// Axum TLS configuration (if using HTTPS directly)
use axum_server::tls_rustls::RustlsConfig;

async fn create_tls_config() -> Result<RustlsConfig, Box<dyn std::error::Error>> {
    let cert_path = std::env::var("TLS_CERT_PATH")
        .unwrap_or_else(|_| "/etc/ssl/certs/riptide.crt".to_string());
    let key_path = std::env::var("TLS_KEY_PATH")
        .unwrap_or_else(|_| "/etc/ssl/private/riptide.key".to_string());

    let config = RustlsConfig::from_pem_file(cert_path, key_path).await?;
    Ok(config)
}

// Start server with TLS
async fn start_tls_server() -> Result<(), Box<dyn std::error::Error>> {
    let app = create_app().await?;
    let tls_config = create_tls_config().await?;

    let addr = SocketAddr::from(([0, 0, 0, 0], 8443));

    axum_server::bind_rustls(addr, tls_config)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
```

### Reverse Proxy Configuration

Most deployments use a reverse proxy for TLS termination:

```nginx
# Nginx configuration
server {
    listen 443 ssl http2;
    server_name api.riptide.dev;

    # TLS configuration
    ssl_certificate /etc/ssl/certs/riptide.crt;
    ssl_certificate_key /etc/ssl/private/riptide.key;
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers ECDHE-RSA-AES256-GCM-SHA512:DHE-RSA-AES256-GCM-SHA512:ECDHE-RSA-AES256-GCM-SHA384;
    ssl_prefer_server_ciphers off;

    # Security headers
    add_header Strict-Transport-Security "max-age=63072000" always;
    add_header X-Frame-Options DENY always;
    add_header X-Content-Type-Options nosniff always;
    add_header X-XSS-Protection "1; mode=block" always;
    add_header Referrer-Policy "strict-origin-when-cross-origin" always;

    # Rate limiting
    limit_req_zone $binary_remote_addr zone=api:10m rate=100r/m;
    limit_req zone=api burst=20 nodelay;

    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;

        # Timeouts
        proxy_connect_timeout 60s;
        proxy_send_timeout 60s;
        proxy_read_timeout 60s;

        # Buffer settings for streaming
        proxy_buffering off;
        proxy_cache off;
    }

    # WebSocket support
    location /crawl/ws {
        proxy_pass http://127.0.0.1:8080;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

## CORS Configuration

```rust
use tower_http::cors::{CorsLayer, Any};
use http::Method;

fn create_cors_layer() -> CorsLayer {
    CorsLayer::new()
        // Allow specific origins in production
        .allow_origin([
            "https://app.riptide.dev".parse().unwrap(),
            "https://dashboard.riptide.dev".parse().unwrap(),
        ])
        // For development, you might use:
        // .allow_origin(Any)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::OPTIONS,
        ])
        .allow_headers([
            "content-type",
            "authorization",
            "x-session-id",
            "x-request-id",
            "x-client-info",
            "x-api-key",
            "x-buffer-size",
        ])
        .expose_headers([
            "x-ratelimit-limit",
            "x-ratelimit-remaining",
            "x-ratelimit-reset",
            "x-session-id",
        ])
        .allow_credentials(true)
        .max_age(Duration::from_secs(86400)) // 24 hours
}

// Apply to app
let app = Router::new()
    .route("/crawl", post(handlers::crawl))
    // ... other routes
    .layer(create_cors_layer());
```

## Request/Response Security

### Security Headers Middleware

```rust
use axum::{
    middleware::{self, Next},
    response::Response,
    Request,
};
use http::HeaderValue;

async fn security_headers<B>(
    request: Request<B>,
    next: Next<B>,
) -> Response {
    let mut response = next.run(request).await;

    let headers = response.headers_mut();

    // Prevent XSS attacks
    headers.insert(
        "X-Content-Type-Options",
        HeaderValue::from_static("nosniff"),
    );

    // Prevent clickjacking
    headers.insert(
        "X-Frame-Options",
        HeaderValue::from_static("DENY"),
    );

    // XSS protection
    headers.insert(
        "X-XSS-Protection",
        HeaderValue::from_static("1; mode=block"),
    );

    // Referrer policy
    headers.insert(
        "Referrer-Policy",
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );

    // Content Security Policy
    headers.insert(
        "Content-Security-Policy",
        HeaderValue::from_static("default-src 'none'; connect-src 'self'"),
    );

    // HSTS (if using HTTPS)
    if let Some(proto) = headers.get("x-forwarded-proto") {
        if proto == "https" {
            headers.insert(
                "Strict-Transport-Security",
                HeaderValue::from_static("max-age=31536000; includeSubDomains"),
            );
        }
    }

    response
}

// Apply middleware
let app = Router::new()
    .route("/crawl", post(handlers::crawl))
    .layer(middleware::from_fn(security_headers));
```

### Request Size Limits

```rust
use axum::extract::DefaultBodyLimit;
use tower_http::limit::RequestBodyLimitLayer;

let app = Router::new()
    .route("/crawl", post(handlers::crawl))
    .layer(DefaultBodyLimit::max(1024 * 1024)) // 1MB limit
    .layer(RequestBodyLimitLayer::new(1024 * 1024)); // Additional layer
```

## Monitoring and Logging

### Security Event Logging

```javascript
class SecurityLogger {
    constructor() {
        this.events = [];
        this.alertThresholds = {
            failed_requests: 10, // per minute
            rate_limit_hits: 5,  // per minute
            validation_errors: 20 // per minute
        };
    }

    logSecurityEvent(event) {
        const securityEvent = {
            timestamp: new Date().toISOString(),
            type: event.type,
            severity: event.severity || 'info',
            source_ip: event.source_ip,
            user_agent: event.user_agent,
            endpoint: event.endpoint,
            session_id: event.session_id,
            details: event.details,
            ...event
        };

        this.events.push(securityEvent);

        // Log to system
        console.log('SECURITY_EVENT:', JSON.stringify(securityEvent));

        // Check for alert conditions
        this.checkAlertThresholds(securityEvent);

        // Clean old events (keep last hour)
        const oneHourAgo = Date.now() - 3600000;
        this.events = this.events.filter(
            e => new Date(e.timestamp).getTime() > oneHourAgo
        );
    }

    checkAlertThresholds(event) {
        const recentEvents = this.getRecentEvents(60000); // Last minute

        for (const [eventType, threshold] of Object.entries(this.alertThresholds)) {
            const count = recentEvents.filter(e => e.type === eventType).length;

            if (count >= threshold) {
                this.sendAlert({
                    type: 'threshold_exceeded',
                    event_type: eventType,
                    count,
                    threshold,
                    time_window: '1 minute'
                });
            }
        }
    }

    getRecentEvents(timeWindowMs) {
        const cutoff = Date.now() - timeWindowMs;
        return this.events.filter(
            e => new Date(e.timestamp).getTime() > cutoff
        );
    }

    sendAlert(alert) {
        console.error('SECURITY_ALERT:', JSON.stringify(alert));

        // Send to monitoring system
        // sendToSlack(alert);
        // sendToEmail(alert);
        // sendToPagerDuty(alert);
    }
}

// Usage in middleware
const securityLogger = new SecurityLogger();

function logSecurityMiddleware(req, res, next) {
    const originalSend = res.send;

    res.send = function(data) {
        // Log failed requests
        if (res.statusCode >= 400) {
            securityLogger.logSecurityEvent({
                type: 'failed_request',
                severity: res.statusCode >= 500 ? 'error' : 'warning',
                source_ip: req.ip,
                user_agent: req.get('User-Agent'),
                endpoint: req.path,
                session_id: req.get('X-Session-ID'),
                status_code: res.statusCode,
                response: typeof data === 'string' ? data.substring(0, 200) : '[object]'
            });
        }

        return originalSend.call(this, data);
    };

    next();
}
```

## Attack Prevention

### DDoS Protection

```javascript
class DDoSProtection {
    constructor() {
        this.requestCounts = new Map(); // IP -> count
        this.suspiciousIPs = new Set();
        this.bannedIPs = new Set();

        // Thresholds
        this.maxRequestsPerSecond = 10;
        this.maxRequestsPerMinute = 600;
        this.banDuration = 3600000; // 1 hour
    }

    checkRequest(req) {
        const ip = req.ip;
        const now = Date.now();

        // Check if IP is banned
        if (this.bannedIPs.has(ip)) {
            throw new SecurityError('IP address banned', 403);
        }

        // Initialize tracking for new IPs
        if (!this.requestCounts.has(ip)) {
            this.requestCounts.set(ip, {
                requests: [],
                lastRequest: now,
                suspiciousActivity: 0
            });
        }

        const ipData = this.requestCounts.get(ip);

        // Add current request
        ipData.requests.push(now);
        ipData.lastRequest = now;

        // Clean old requests (keep last minute)
        ipData.requests = ipData.requests.filter(
            timestamp => now - timestamp < 60000
        );

        // Check rate limits
        this.checkRateLimits(ip, ipData, now);

        // Check for suspicious patterns
        this.checkSuspiciousPatterns(ip, ipData, req);
    }

    checkRateLimits(ip, ipData, now) {
        const recentRequests = ipData.requests.filter(
            timestamp => now - timestamp < 1000
        );

        // Requests per second check
        if (recentRequests.length > this.maxRequestsPerSecond) {
            this.handleRateLimit(ip, 'requests_per_second', recentRequests.length);
        }

        // Requests per minute check
        if (ipData.requests.length > this.maxRequestsPerMinute) {
            this.handleRateLimit(ip, 'requests_per_minute', ipData.requests.length);
        }
    }

    checkSuspiciousPatterns(ip, ipData, req) {
        // Check for bot-like behavior
        const userAgent = req.get('User-Agent') || '';

        if (this.isSuspiciousUserAgent(userAgent)) {
            ipData.suspiciousActivity++;
        }

        // Check for rapid sequential requests
        if (ipData.requests.length >= 5) {
            const intervals = ipData.requests
                .slice(-5)
                .map((time, i, arr) => i > 0 ? time - arr[i-1] : 0)
                .slice(1);

            const avgInterval = intervals.reduce((a, b) => a + b, 0) / intervals.length;

            if (avgInterval < 100) { // Less than 100ms between requests
                ipData.suspiciousActivity++;
            }
        }

        // Check accumulated suspicious activity
        if (ipData.suspiciousActivity >= 5) {
            this.banIP(ip, 'suspicious_activity');
        }
    }

    isSuspiciousUserAgent(userAgent) {
        const suspiciousPatterns = [
            /bot/i,
            /crawler/i,
            /spider/i,
            /scraper/i,
            /python/i,
            /curl/i,
            /wget/i,
            '^$' // Empty user agent
        ];

        return suspiciousPatterns.some(pattern =>
            typeof pattern === 'string'
                ? userAgent.includes(pattern)
                : pattern.test(userAgent)
        );
    }

    handleRateLimit(ip, reason, count) {
        securityLogger.logSecurityEvent({
            type: 'rate_limit_exceeded',
            severity: 'warning',
            source_ip: ip,
            reason,
            count,
            details: `Rate limit exceeded: ${count} requests`
        });

        this.suspiciousIPs.add(ip);

        throw new SecurityError(`Rate limit exceeded: ${reason}`, 429);
    }

    banIP(ip, reason) {
        this.bannedIPs.add(ip);

        securityLogger.logSecurityEvent({
            type: 'ip_banned',
            severity: 'error',
            source_ip: ip,
            reason,
            duration: this.banDuration,
            details: `IP banned for ${reason}`
        });

        // Schedule unban
        setTimeout(() => {
            this.bannedIPs.delete(ip);
            this.suspiciousIPs.delete(ip);
            this.requestCounts.delete(ip);
        }, this.banDuration);

        throw new SecurityError(`IP banned: ${reason}`, 403);
    }
}

// Middleware integration
const ddosProtection = new DDoSProtection();

function ddosProtectionMiddleware(req, res, next) {
    try {
        ddosProtection.checkRequest(req);
        next();
    } catch (error) {
        if (error instanceof SecurityError) {
            res.status(error.status).json({
                error: {
                    type: 'security_error',
                    message: error.message,
                    retryable: false,
                    status: error.status
                }
            });
        } else {
            next(error);
        }
    }
}
```

### Input Validation Against Injection

```javascript
class InjectionProtection {
    constructor() {
        // SQL injection patterns
        this.sqlPatterns = [
            /(\b(select|union|insert|update|delete|drop|create|alter|exec|execute)\b)/gi,
            /(--|\#|\/\*|\*\/)/gi,
            /(\bor\b|\band\b).*(\=|like|in)/gi,
            /('.*'|".*")/gi
        ];

        // XSS patterns
        this.xssPatterns = [
            /<script\b[^<]*(?:(?!<\/script>)<[^<]*)*<\/script>/gi,
            /<iframe\b[^<]*(?:(?!<\/iframe>)<[^<]*)*<\/iframe>/gi,
            /javascript:/gi,
            /vbscript:/gi,
            /onload\s*=/gi,
            /onerror\s*=/gi,
            /onclick\s*=/gi
        ];

        // Command injection patterns
        this.commandPatterns = [
            /[;&|`$()]/g,
            /\.\./g,
            /\/etc\/passwd/gi,
            /\/bin\//gi,
            /cmd\.exe/gi,
            /powershell/gi
        ];
    }

    validateInput(input, context = 'general') {
        if (typeof input !== 'string') {
            return input; // Non-string inputs are generally safe
        }

        // Check for SQL injection
        if (this.containsSQLInjection(input)) {
            throw new SecurityError('Potential SQL injection detected', 400);
        }

        // Check for XSS
        if (this.containsXSS(input)) {
            throw new SecurityError('Potential XSS detected', 400);
        }

        // Check for command injection
        if (this.containsCommandInjection(input)) {
            throw new SecurityError('Potential command injection detected', 400);
        }

        // Context-specific validation
        switch (context) {
            case 'url':
                return this.validateURL(input);
            case 'query':
                return this.validateQuery(input);
            case 'user_agent':
                return this.validateUserAgent(input);
            default:
                return this.sanitizeGeneral(input);
        }
    }

    containsSQLInjection(input) {
        return this.sqlPatterns.some(pattern => pattern.test(input));
    }

    containsXSS(input) {
        return this.xssPatterns.some(pattern => pattern.test(input));
    }

    containsCommandInjection(input) {
        return this.commandPatterns.some(pattern => pattern.test(input));
    }

    validateURL(url) {
        // Already handled by URLValidator class
        return url;
    }

    validateQuery(query) {
        // Additional query-specific validation
        const dangerous = [
            'drop table',
            'delete from',
            'update set',
            'insert into',
            'create table',
            'alter table'
        ];

        const lowerQuery = query.toLowerCase();
        if (dangerous.some(pattern => lowerQuery.includes(pattern))) {
            throw new SecurityError('Dangerous query pattern detected', 400);
        }

        return query;
    }

    validateUserAgent(userAgent) {
        // Remove potential script injection from user agent
        return userAgent
            .replace(/<[^>]*>/g, '') // Remove HTML tags
            .replace(/[<>"']/g, '') // Remove dangerous characters
            .substring(0, 200); // Limit length
    }

    sanitizeGeneral(input) {
        return input
            .replace(/[\x00-\x08\x0B\x0C\x0E-\x1F\x7F]/g, '') // Remove control chars
            .replace(/[<>"'&]/g, match => {
                const entities = {
                    '<': '&lt;',
                    '>': '&gt;',
                    '"': '&quot;',
                    "'": '&#x27;',
                    '&': '&amp;'
                };
                return entities[match] || match;
            });
    }
}

// Middleware integration
const injectionProtection = new InjectionProtection();

function validateInputMiddleware(req, res, next) {
    try {
        // Validate request body
        if (req.body) {
            req.body = this.validateObject(req.body, injectionProtection);
        }

        // Validate query parameters
        if (req.query) {
            req.query = this.validateObject(req.query, injectionProtection);
        }

        next();
    } catch (error) {
        if (error instanceof SecurityError) {
            securityLogger.logSecurityEvent({
                type: 'injection_attempt',
                severity: 'error',
                source_ip: req.ip,
                user_agent: req.get('User-Agent'),
                endpoint: req.path,
                details: error.message
            });

            res.status(error.status).json({
                error: {
                    type: 'security_error',
                    message: error.message,
                    retryable: false,
                    status: error.status
                }
            });
        } else {
            next(error);
        }
    }
}
```

This comprehensive security guide covers authentication, rate limiting, input validation, HTTPS configuration, and protection against common attack vectors, providing a solid security foundation for the RipTide API.