# RipTide Fetch

HTTP/network layer for the RipTide web scraping framework, providing high-performance HTTP client operations with caching, compression, and rate limiting.

## Overview

`riptide-fetch` implements the network foundation of RipTide, delivering optimized HTTP/2 operations, intelligent caching, compression handling, and polite rate limiting for respectful web scraping at scale.

## Features

- **HTTP/2 Support**: Prior knowledge and connection multiplexing
- **Compression**: Automatic Gzip/Brotli decompression
- **Connection Pooling**: Reusable connections with keep-alive
- **Rate Limiting**: Per-host and global rate limiting with jitter
- **Retry Logic**: Exponential backoff with configurable policies
- **Timeout Management**: Connect, read, and total timeouts
- **Robots.txt Compliance**: Automatic robots.txt parsing and enforcement
- **Cookie Management**: Session cookie persistence
- **Custom Headers**: User-agent rotation and custom headers
- **Progress Tracking**: Download progress callbacks
- **Proxy Support**: HTTP/SOCKS5 proxy configuration

## Architecture

```
┌──────────────────────────────────────────────────────────┐
│                  RipTide Fetch Layer                     │
├──────────────────────────────────────────────────────────┤
│                                                          │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐     │
│  │   HTTP/2    │  │    Rate     │  │   Robots    │     │
│  │   Client    │  │   Limiter   │  │   Parser    │     │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘     │
│         │                │                 │            │
│         └────────────────┼─────────────────┘            │
│                          ▼                              │
│                  ┌───────────────┐                      │
│                  │    Reqwest    │                      │
│                  │ (HTTP Client) │                      │
│                  └───────┬───────┘                      │
└────────────────────────────┼─────────────────────────────┘
                            │
                            ▼
                    ┌──────────────┐
                    │  Web Server  │
                    └──────────────┘
```

## Usage

### Basic HTTP Requests

```rust
use riptide_fetch::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Create fetch client
    let client = FetchClient::new();

    // GET request
    let response = client.get("https://example.com").await?;
    println!("Status: {}", response.status());
    println!("Body: {}", response.text().await?);

    // POST request
    let body = json!({
        "key": "value"
    });
    let response = client.post("https://api.example.com")
        .json(&body)
        .send()
        .await?;

    Ok(())
}
```

### Advanced Configuration

```rust
use riptide_fetch::*;
use std::time::Duration;

let client = FetchClient::builder()
    .timeout(Duration::from_secs(30))
    .connect_timeout(Duration::from_secs(10))
    .pool_max_idle_per_host(10)
    .http2_prior_knowledge()
    .gzip(true)
    .brotli(true)
    .user_agent("RipTide/1.0")
    .build()?;
```

### Rate Limiting

```rust
use riptide_fetch::*;

// Per-host rate limiting
let client = FetchClient::builder()
    .rate_limit_per_host(1.5) // 1.5 requests per second
    .rate_limit_jitter(0.1)   // ±10% jitter
    .burst_capacity(3)        // Allow bursts of 3
    .build()?;

// Requests automatically throttled
for url in urls {
    let response = client.get(&url).await?;
    // Automatically waits to respect rate limit
}
```

### Robots.txt Compliance

```rust
use riptide_fetch::*;

let client = FetchClient::builder()
    .respect_robots_txt(true)
    .user_agent("MyBot/1.0")
    .build()?;

// Automatically checks robots.txt before fetching
let response = client.get("https://example.com/path").await?;

// Check manually
if client.is_allowed("https://example.com/admin").await? {
    let response = client.get("https://example.com/admin").await?;
}
```

### Retry Logic

```rust
use riptide_fetch::*;

let client = FetchClient::builder()
    .retry_policy(RetryPolicy {
        max_retries: 3,
        initial_backoff: Duration::from_millis(100),
        max_backoff: Duration::from_secs(10),
        multiplier: 2.0,
        retry_on_status: vec![429, 500, 502, 503, 504],
    })
    .build()?;

// Automatically retries on transient failures
let response = client.get("https://flaky-api.com").await?;
```

### Custom Headers

```rust
use riptide_fetch::*;

let mut headers = HeaderMap::new();
headers.insert("X-Custom-Header", "value".parse()?);
headers.insert("Authorization", "Bearer token".parse()?);

let response = client.get("https://api.example.com")
    .headers(headers)
    .send()
    .await?;
```

### Cookie Management

```rust
use riptide_fetch::*;

// Enable cookie jar
let client = FetchClient::builder()
    .cookie_store(true)
    .build()?;

// Cookies automatically managed
client.get("https://example.com/login").await?;
// Cookies sent automatically on subsequent requests
client.get("https://example.com/dashboard").await?;
```

### Proxy Configuration

```rust
use riptide_fetch::*;

// HTTP proxy
let client = FetchClient::builder()
    .proxy("http://proxy.example.com:8080")
    .build()?;

// SOCKS5 proxy
let client = FetchClient::builder()
    .proxy("socks5://proxy.example.com:1080")
    .build()?;

// Proxy with authentication
let client = FetchClient::builder()
    .proxy("http://user:pass@proxy.example.com:8080")
    .build()?;
```

### Progress Tracking

```rust
use riptide_fetch::*;

let response = client.get("https://example.com/large-file.zip")
    .on_progress(|downloaded, total| {
        let percentage = (downloaded as f64 / total.unwrap_or(1) as f64) * 100.0;
        println!("Downloaded: {:.2}%", percentage);
    })
    .send()
    .await?;
```

### Compression Handling

```rust
use riptide_fetch::*;

// Automatic decompression
let client = FetchClient::builder()
    .gzip(true)
    .brotli(true)
    .deflate(true)
    .build()?;

// Accept-Encoding header automatically added
let response = client.get("https://example.com").await?;
let text = response.text().await?; // Automatically decompressed
```

## Response Handling

### Text Content

```rust
let response = client.get("https://example.com").await?;
let text = response.text().await?;
let body = response.body_bytes().await?;
```

### JSON Content

```rust
#[derive(Deserialize)]
struct ApiResponse {
    data: String,
}

let response = client.get("https://api.example.com").await?;
let data: ApiResponse = response.json().await?;
```

### Headers and Metadata

```rust
let response = client.get("https://example.com").await?;

// Status
println!("Status: {}", response.status());

// Headers
if let Some(content_type) = response.headers().get("content-type") {
    println!("Content-Type: {:?}", content_type);
}

// URL (after redirects)
println!("Final URL: {}", response.url());

// Content length
if let Some(length) = response.content_length() {
    println!("Content-Length: {}", length);
}
```

## Configuration

### Environment Variables

```bash
# Timeouts
export FETCH_TIMEOUT=30
export FETCH_CONNECT_TIMEOUT=10
export FETCH_READ_TIMEOUT=30

# Rate limiting
export RATE_LIMIT_RPS=1.5
export RATE_LIMIT_JITTER=0.1
export RATE_LIMIT_BURST=3

# Connection pooling
export HTTP_POOL_MAX_IDLE=10
export HTTP_POOL_IDLE_TIMEOUT=90

# Robots.txt
export RESPECT_ROBOTS_TXT=true
export USER_AGENT="RipTide/1.0"

# Compression
export ENABLE_GZIP=true
export ENABLE_BROTLI=true

# Proxy
export HTTP_PROXY="http://proxy:8080"
export HTTPS_PROXY="https://proxy:8080"
```

### Programmatic Configuration

```rust
use riptide_fetch::*;

let config = FetchConfig {
    timeout: Duration::from_secs(30),
    connect_timeout: Duration::from_secs(10),
    rate_limit: RateLimitConfig {
        requests_per_second: 1.5,
        jitter_factor: 0.1,
        burst_capacity: 3,
    },
    compression: CompressionConfig {
        gzip: true,
        brotli: true,
        deflate: true,
    },
    retry: RetryConfig {
        max_retries: 3,
        initial_backoff: Duration::from_millis(100),
        max_backoff: Duration::from_secs(10),
    },
    user_agent: "RipTide/1.0".to_string(),
    respect_robots_txt: true,
};

let client = FetchClient::with_config(config)?;
```

## Performance Optimization

### Connection Pooling

```rust
// Reuse connections across requests
let client = FetchClient::builder()
    .pool_max_idle_per_host(20)
    .pool_idle_timeout(Duration::from_secs(90))
    .build()?;

// Connections automatically reused
for url in urls {
    let response = client.get(&url).await?;
}
```

### HTTP/2 Prior Knowledge

```rust
// Skip HTTP/1.1 upgrade negotiation
let client = FetchClient::builder()
    .http2_prior_knowledge()
    .build()?;

// Faster for known HTTP/2 servers
```

### Concurrent Requests

```rust
use futures::future::join_all;

let client = FetchClient::new();
let urls = vec!["url1", "url2", "url3"];

// Fetch concurrently
let futures = urls.iter().map(|url| client.get(url));
let responses = join_all(futures).await;
```

## Integration with RipTide

This crate is used by:

- **riptide-core**: Core HTTP operations
- **riptide-api**: API client requests
- **riptide-spider**: Web crawling
- **riptide-search**: Search API integration

## Testing

```bash
# Run tests
cargo test -p riptide-fetch

# Run with mock HTTP server
cargo test -p riptide-fetch --features mock-server

# Integration tests
cargo test -p riptide-fetch --test '*'

# Test rate limiting
cargo test -p riptide-fetch rate_limit_tests -- --nocapture
```

## License

Apache-2.0

## Related Crates

- **riptide-core**: Core framework
- **riptide-cache**: Response caching
- **riptide-spider**: Web crawler
- **reqwest**: Underlying HTTP client
