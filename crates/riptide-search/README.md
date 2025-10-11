# riptide-search

Search provider abstraction layer for RipTide with multiple backend support.

## Overview

`riptide-search` is a flexible, vendor-agnostic search provider abstraction that enables RipTide to perform web searches through multiple backends. The crate provides a unified `SearchProvider` trait with built-in reliability features including circuit breaker protection, configurable timeouts, and comprehensive error handling.

**Key Features:**

- **Multiple Backend Support**: Easily switch between search providers (Google/Serper, URL parsing, SearXNG)
- **Circuit Breaker Pattern**: Automatic failure detection and fast-fail protection
- **Async/Await API**: Built on tokio for high-performance concurrent operations
- **Type Safety**: Strong typing with comprehensive error handling via `anyhow`
- **Zero External Dependencies**: Optional "None" backend parses URLs without API calls
- **Result Normalization**: Consistent `SearchHit` format across all providers
- **Configurable**: Environment variables and programmatic configuration support

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
riptide-search = { path = "../riptide-search" }
```

## Supported Providers

### 1. Serper (Google Search API)

High-quality Google search results via [Serper.dev](https://serper.dev) API.

**Requirements:**
- API key from serper.dev
- Environment variable: `SERPER_API_KEY`

**Features:**
- Ranked search results with titles and snippets
- Country and locale customization
- Configurable result limits (1-100)
- Rate limiting awareness

### 2. None (URL Parser)

Zero-dependency URL extraction from query strings.

**Use Cases:**
- Direct URL input without external API calls
- Fallback when no search API is configured
- Testing and development

**Supported Formats:**
- Space-separated: `https://a.com https://b.com`
- Comma-separated: `https://a.com,https://b.com`
- Mixed with text: `Check https://example.com for info`
- Newline-separated URLs

### 3. SearXNG (Self-Hosted)

Support for self-hosted [SearXNG](https://github.com/searxng/searxng) instances.

**Status:** Planned (not yet implemented)

**Requirements:**
- SearXNG instance URL
- Environment variable: `SEARXNG_BASE_URL`

## SearchProvider Trait

The core abstraction that all search providers implement:

```rust
#[async_trait]
pub trait SearchProvider: Send + Sync + std::fmt::Debug {
    /// Perform a web search with the given parameters
    async fn search(
        &self,
        query: &str,
        limit: u32,
        country: &str,
        locale: &str,
    ) -> Result<Vec<SearchHit>>;

    /// Get the backend type for this provider
    fn backend_type(&self) -> SearchBackend;

    /// Health check to verify provider readiness
    async fn health_check(&self) -> Result<()>;
}
```

### SearchHit

Normalized search result structure:

```rust
pub struct SearchHit {
    pub url: String,           // Result URL
    pub rank: u32,            // Position (1-based)
    pub title: Option<String>,    // Result title
    pub snippet: Option<String>,  // Description/excerpt
    pub metadata: HashMap<String, String>, // Provider-specific data
}
```

## Usage Examples

### Basic Usage with Serper

```rust
use riptide_search::{create_search_provider, SearchConfig, SearchBackend};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = SearchConfig {
        backend: SearchBackend::Serper,
        api_key: Some("your-serper-api-key".to_string()),
        timeout_seconds: 30,
        ..Default::default()
    };

    let provider = create_search_provider(config).await?;
    let results = provider.search("rust programming", 10, "us", "en").await?;

    for hit in results {
        println!("#{}: {}", hit.rank, hit.url);
        if let Some(title) = hit.title {
            println!("  Title: {}", title);
        }
    }

    Ok(())
}
```

### Environment-Based Configuration

```rust
use riptide_search::SearchProviderFactory;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Reads from environment variables:
    // - SEARCH_BACKEND (serper, none, searxng)
    // - SERPER_API_KEY (for Serper backend)
    // - SEARCH_TIMEOUT (optional, default 30)
    let provider = SearchProviderFactory::create_from_env().await?;

    let results = provider.search("async rust", 5, "us", "en").await?;
    println!("Found {} results", results.len());

    Ok(())
}
```

### None Provider (URL Parsing)

```rust
use riptide_search::{create_search_provider, SearchConfig, SearchBackend};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = SearchConfig {
        backend: SearchBackend::None,
        enable_url_parsing: true,
        ..Default::default()
    };

    let provider = create_search_provider(config).await?;

    // Parse URLs directly from query
    let results = provider.search(
        "https://example.com https://rust-lang.org",
        10,
        "us",
        "en"
    ).await?;

    for hit in results {
        println!("Parsed URL: {}", hit.url);
    }

    Ok(())
}
```

### Advanced Configuration with Circuit Breaker

```rust
use riptide_search::{
    SearchProviderFactory,
    AdvancedSearchConfig,
    CircuitBreakerConfigOptions,
    SearchBackend,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = AdvancedSearchConfig {
        backend: SearchBackend::Serper,
        api_key: Some("your-api-key".to_string()),
        timeout_seconds: 30,
        enable_url_parsing: false,
        circuit_breaker: CircuitBreakerConfigOptions {
            failure_threshold: 60,      // 60% failure rate triggers circuit
            min_requests: 10,            // Need 10 requests before tripping
            recovery_timeout_secs: 120,  // Wait 2 minutes before retry
        },
    };

    // Validate configuration before creating provider
    config.validate()?;

    let provider = SearchProviderFactory::create_provider(config).await?;
    let results = provider.search("tokio async", 10, "us", "en").await?;

    Ok(())
}
```

### Multi-Provider Setup with Fallback

```rust
use riptide_search::{create_search_provider, SearchConfig, SearchBackend};
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Try Serper first
    let serper_config = SearchConfig {
        backend: SearchBackend::Serper,
        api_key: std::env::var("SERPER_API_KEY").ok(),
        ..Default::default()
    };

    let provider = match create_search_provider(serper_config).await {
        Ok(p) => p,
        Err(_) => {
            // Fall back to None provider
            let none_config = SearchConfig {
                backend: SearchBackend::None,
                enable_url_parsing: true,
                ..Default::default()
            };
            create_search_provider(none_config).await?
        }
    };

    let results = provider.search("query or URLs", 10, "us", "en").await?;
    println!("Search successful using {} backend", provider.backend_type());

    Ok(())
}
```

### Concurrent Searches

```rust
use riptide_search::{create_search_provider, SearchConfig, SearchBackend};
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = SearchConfig {
        backend: SearchBackend::None,
        enable_url_parsing: true,
        ..Default::default()
    };

    let provider = Arc::new(create_search_provider(config).await?);

    let queries = vec![
        "https://example1.com",
        "https://example2.com",
        "https://example3.com",
    ];

    let mut handles = vec![];
    for query in queries {
        let provider_clone = provider.clone();
        handles.push(tokio::spawn(async move {
            provider_clone.search(query, 10, "us", "en").await
        }));
    }

    for handle in handles {
        let results = handle.await??;
        println!("Got {} results", results.len());
    }

    Ok(())
}
```

## Configuration

### SearchConfig

Basic configuration structure:

```rust
pub struct SearchConfig {
    pub backend: SearchBackend,           // serper, none, searxng
    pub api_key: Option<String>,          // API key if required
    pub base_url: Option<String>,         // Base URL for self-hosted
    pub timeout_seconds: u64,             // Request timeout (default: 30)
    pub enable_url_parsing: bool,         // Enable URL parsing (default: true)
}
```

### AdvancedSearchConfig

Enhanced configuration with circuit breaker:

```rust
pub struct AdvancedSearchConfig {
    pub backend: SearchBackend,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub timeout_seconds: u64,
    pub enable_url_parsing: bool,
    pub circuit_breaker: CircuitBreakerConfigOptions,
}

pub struct CircuitBreakerConfigOptions {
    pub failure_threshold: u32,          // Percentage (0-100) to trip circuit
    pub min_requests: u32,               // Minimum requests before tripping
    pub recovery_timeout_secs: u64,      // Wait time before retry
}
```

### Environment Variables

The crate reads configuration from these environment variables:

| Variable | Description | Default |
|----------|-------------|---------|
| `SEARCH_BACKEND` | Backend type (serper, none, searxng) | `serper` |
| `SERPER_API_KEY` | Serper.dev API key | None |
| `SEARXNG_BASE_URL` | SearXNG instance URL | None |
| `SEARCH_TIMEOUT` | Request timeout in seconds | `30` |
| `SEARCH_ENABLE_URL_PARSING` | Enable URL parsing (true/false) | `true` |
| `CIRCUIT_BREAKER_FAILURE_THRESHOLD` | Failure % to trip circuit (0-100) | `50` |
| `CIRCUIT_BREAKER_MIN_REQUESTS` | Min requests before circuit trips | `5` |
| `CIRCUIT_BREAKER_RECOVERY_TIMEOUT` | Recovery wait time (seconds) | `60` |

## Error Handling

The crate uses `anyhow::Result` for comprehensive error handling:

```rust
use riptide_search::{create_search_provider, SearchConfig, SearchBackend};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = SearchConfig {
        backend: SearchBackend::Serper,
        api_key: None, // Missing required API key
        ..Default::default()
    };

    match create_search_provider(config).await {
        Ok(provider) => {
            // Use provider
        },
        Err(e) => {
            eprintln!("Failed to create provider: {}", e);
            // Error: "API key is required for Serper backend"
        }
    }

    Ok(())
}
```

### Common Error Scenarios

1. **Configuration Errors**
   - Missing API key for Serper
   - Missing base URL for SearXNG
   - Invalid timeout values

2. **Runtime Errors**
   - Network connectivity issues
   - API rate limiting
   - Invalid query parameters
   - Circuit breaker OPEN state

3. **Provider-Specific Errors**
   - Serper: API authentication failures, quota exceeded
   - None: No URLs found in query string
   - SearXNG: Instance unavailable

## Circuit Breaker

The circuit breaker pattern protects against cascading failures:

### States

1. **Closed** (Normal): All requests pass through
2. **Open** (Failed): All requests fail immediately
3. **Half-Open** (Testing): Limited test requests to check recovery

### Behavior

```rust
use riptide_search::{
    CircuitBreakerWrapper,
    CircuitBreakerConfig,
    NoneProvider,
};
use std::time::Duration;

let provider = Box::new(NoneProvider::new(true));
let config = CircuitBreakerConfig {
    failure_threshold_percentage: 50,  // Trip at 50% failures
    minimum_request_threshold: 5,      // Need 5 requests first
    recovery_timeout: Duration::from_secs(60),
    half_open_max_requests: 3,         // Test with 3 requests
};

let circuit = CircuitBreakerWrapper::with_config(provider, config);

// Monitor circuit state
println!("State: {:?}", circuit.current_state());
println!("Failure rate: {}%", circuit.failure_rate());

// Manual reset (for testing)
circuit.reset();
```

### Benefits

- **Fast Fail**: Avoid waiting for timeouts when provider is down
- **Resource Protection**: Prevent resource exhaustion
- **Automatic Recovery**: Self-healing after recovery timeout
- **Metrics**: Built-in failure rate tracking

## Testing with Mocks

The crate supports testing with mock providers:

```rust
use riptide_search::{SearchProvider, SearchBackend, SearchHit};
use anyhow::Result;
use async_trait::async_trait;

#[derive(Debug)]
struct MockProvider {
    responses: Vec<Vec<SearchHit>>,
    call_count: std::sync::Arc<std::sync::Mutex<usize>>,
}

#[async_trait]
impl SearchProvider for MockProvider {
    async fn search(
        &self,
        _query: &str,
        _limit: u32,
        _country: &str,
        _locale: &str,
    ) -> Result<Vec<SearchHit>> {
        let mut count = self.call_count.lock().unwrap();
        let idx = *count % self.responses.len();
        *count += 1;
        Ok(self.responses[idx].clone())
    }

    fn backend_type(&self) -> SearchBackend {
        SearchBackend::None
    }

    async fn health_check(&self) -> Result<()> {
        Ok(())
    }
}

#[tokio::test]
async fn test_with_mock() {
    let mock = MockProvider {
        responses: vec![vec![
            SearchHit::new("https://example.com".to_string(), 1),
        ]],
        call_count: std::sync::Arc::new(std::sync::Mutex::new(0)),
    };

    let results = mock.search("test", 10, "us", "en").await.unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].url, "https://example.com");
}
```

## Integration with riptide-core

`riptide-search` is designed to integrate seamlessly with the RipTide core:

```rust
// In riptide-core or riptide-api
use riptide_search::{SearchProviderFactory, SearchBackend};

pub struct SearchHandler {
    provider: Box<dyn SearchProvider>,
}

impl SearchHandler {
    pub async fn new() -> anyhow::Result<Self> {
        let provider = SearchProviderFactory::create_from_env().await?;
        Ok(Self { provider })
    }

    pub async fn handle_search(
        &self,
        query: &str,
        limit: u32,
        country: &str,
        locale: &str,
    ) -> anyhow::Result<Vec<SearchHit>> {
        self.provider.search(query, limit, country, locale).await
    }
}
```

## Performance Considerations

1. **Concurrent Searches**: All providers are `Send + Sync` for safe concurrent usage
2. **Connection Pooling**: Serper uses reqwest with automatic connection pooling
3. **Timeouts**: Configurable timeouts prevent hanging requests
4. **Circuit Breaker**: Fast-fail during provider outages
5. **Zero-Copy**: None provider uses regex for efficient URL parsing

## Best Practices

1. **Always set timeouts**: Default 30s may be too long for some use cases
2. **Use circuit breaker**: Protect your application from provider failures
3. **Validate configuration**: Call `config.validate()` before creating providers
4. **Handle errors gracefully**: Provide fallbacks for search failures
5. **Monitor circuit state**: Log circuit breaker state changes
6. **Test with None backend**: Use URL parsing for development/testing

## Examples

See the `tests/` directory for comprehensive integration tests:

- **Provider creation**: Different backend configurations
- **Environment setup**: Reading from env variables
- **Circuit breaker**: Failure scenarios and recovery
- **Multi-provider**: Fallback patterns
- **Concurrent usage**: High-throughput scenarios
- **Error handling**: Various error conditions

Run tests:

```bash
cargo test -p riptide-search
```

## Crate Structure

```
riptide-search/
├── src/
│   ├── lib.rs              # Main traits and configuration
│   ├── providers.rs        # Serper provider implementation
│   ├── none_provider.rs    # URL parsing provider
│   └── circuit_breaker.rs  # Circuit breaker implementation
├── tests/
│   └── riptide_search_integration_tests.rs  # Integration tests
├── Cargo.toml
└── README.md
```

## Dependencies

- `anyhow`: Error handling
- `async-trait`: Async trait support
- `serde` / `serde_json`: Serialization
- `reqwest`: HTTP client (for Serper)
- `regex`: URL parsing (for None)
- `url`: URL validation
- `tokio`: Async runtime

## Future Enhancements

1. **SearXNG Implementation**: Complete self-hosted search support
2. **Additional Providers**: Bing, DuckDuckGo, Brave Search
3. **Caching Layer**: Optional result caching
4. **Rate Limiting**: Per-provider rate limit enforcement
5. **Metrics Export**: Prometheus/OpenTelemetry integration
6. **Retry Logic**: Configurable retry strategies
7. **Response Streaming**: Stream results as they arrive

## License

Apache-2.0

## Contributing

This crate is part of the RipTide project. See the main repository for contribution guidelines.

## Authors

RipTide Team
