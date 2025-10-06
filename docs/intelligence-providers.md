# Intelligence Providers - LLM Integration Guide

**Status:** ✅ Production Ready
**Last Updated:** 2025-10-06
**Providers Available:** 5 (4 production-ready, 1 experimental)

---

## Overview

RipTide Intelligence supports multiple LLM providers for intelligent content extraction, analysis, and processing. All providers implement a common `LlmProvider` trait for consistent integration.

### Available Providers

| Provider | Status | Cost | Use Case |
|----------|--------|------|----------|
| **Anthropic (Claude)** | ✅ Production | Paid | Best for complex extraction, high accuracy |
| **Ollama** | ✅ Production | Free (Local) | Best for privacy, offline usage |
| **LocalAI** | ✅ Production | Free (Local) | OpenAI-compatible local deployment |
| **Google Vertex AI** | ✅ Production | Paid | Best for Google Cloud integration |
| **AWS Bedrock** | ⚠️ Experimental | Paid | Planned feature, mock implementation |

---

## Quick Start

### 1. Anthropic (Claude)

**Best for:** Complex content extraction, highest accuracy, production workloads

**Setup:**
```bash
# Get API key from https://console.anthropic.com/
export ANTHROPIC_API_KEY="sk-ant-api03-..."
```

**Configuration:**
```toml
# config.toml
[intelligence]
provider = "anthropic"
model = "claude-3-5-sonnet-20241022"  # Recommended for balanced performance
# Or: "claude-3-5-haiku-20241022" for speed
# Or: "claude-3-opus-20240229" for maximum accuracy
```

**Rust Code:**
```rust
use riptide_intelligence::providers::AnthropicProvider;
use riptide_intelligence::LlmProvider;

// Create provider
let provider = AnthropicProvider::new(
    std::env::var("ANTHROPIC_API_KEY").expect("ANTHROPIC_API_KEY not set")
)?;

// Make completion request
let request = CompletionRequest {
    id: uuid::Uuid::new_v4(),
    model: "claude-3-5-sonnet-20241022".to_string(),
    messages: vec![
        Message {
            role: Role::User,
            content: "Extract key facts from this article...".to_string(),
        }
    ],
    max_tokens: Some(4096),
    temperature: Some(0.7),
    ..Default::default()
};

let response = provider.complete(request).await?;
println!("Response: {}", response.content);
```

**Available Models:**
- `claude-3-5-sonnet-20241022` - Best balance (recommended)
- `claude-3-5-haiku-20241022` - Fastest, most affordable
- `claude-3-opus-20240229` - Most powerful for complex tasks
- `claude-3-sonnet-20240229` - Balanced (legacy)
- `claude-3-haiku-20240307` - Fast (legacy)

**Pricing (as of 2024):**
- Haiku 3.5: $0.80/$4.00 per million tokens (input/output)
- Sonnet 3.5: $3/$15 per million tokens
- Opus 3: $15/$75 per million tokens

---

### 2. Ollama (Local)

**Best for:** Privacy-sensitive data, offline usage, development/testing

**Setup:**
```bash
# Install Ollama: https://ollama.ai/
curl https://ollama.ai/install.sh | sh

# Pull a model
ollama pull llama3.2
ollama pull codellama  # For code extraction
ollama pull nomic-embed-text  # For embeddings

# Start Ollama server (usually automatic)
ollama serve
```

**Configuration:**
```toml
# config.toml
[intelligence]
provider = "ollama"
model = "llama3.2"
base_url = "http://localhost:11434"  # Default Ollama endpoint
```

**Rust Code:**
```rust
use riptide_intelligence::providers::OllamaProvider;

// Create provider
let provider = OllamaProvider::new("http://localhost:11434".to_string())?;

// Use like any other provider
let response = provider.complete(request).await?;

// Generate embeddings (for semantic search)
let embeddings = provider.embed("text to embed").await?;
```

**Available Models:**
- `llama3.2` - Latest Meta model (recommended)
- `llama3.1` - High-performance general model
- `codellama` - Specialized for code
- `nomic-embed-text` - For embeddings

**Advantages:**
- ✅ Completely free
- ✅ No API keys needed
- ✅ Data stays on your infrastructure
- ✅ No rate limits
- ✅ Supports embeddings

---

### 3. LocalAI (OpenAI-compatible)

**Best for:** OpenAI API compatibility with local models

**Setup:**
```bash
# Docker deployment (easiest)
docker run -p 8080:8080 localai/localai:latest

# Or install locally: https://localai.io/basics/getting_started/
```

**Configuration:**
```toml
# config.toml
[intelligence]
provider = "localai"
model = "gpt-3.5-turbo"  # Model name configured in LocalAI
base_url = "http://localhost:8080"
```

**Rust Code:**
```rust
use riptide_intelligence::providers::LocalAIProvider;

// Create provider
let provider = LocalAIProvider::new("http://localhost:8080".to_string())?;

// OpenAI-compatible API
let response = provider.complete(request).await?;
```

**Advantages:**
- ✅ OpenAI API compatibility
- ✅ Free local usage
- ✅ Supports function calling
- ✅ Supports embeddings

---

### 4. Google Vertex AI (Gemini)

**Best for:** Google Cloud integration, multimodal tasks

**Setup:**
```bash
# 1. Create Google Cloud project
# 2. Enable Vertex AI API
# 3. Set up authentication

# Option A: Service account (recommended for production)
export GOOGLE_APPLICATION_CREDENTIALS="/path/to/service-account-key.json"

# Option B: User credentials (for development)
gcloud auth application-default login
```

**Get Access Token:**
```bash
# Temporary token for testing
gcloud auth print-access-token

# For production, use service account
gcloud auth activate-service-account --key-file=/path/to/key.json
VERTEX_TOKEN=$(gcloud auth print-access-token)
```

**Configuration:**
```toml
# config.toml
[intelligence]
provider = "google_vertex"
model = "gemini-1.5-pro"
project_id = "your-gcp-project-id"
location = "us-central1"
```

**Rust Code:**
```rust
use riptide_intelligence::providers::GoogleVertexProvider;

// Create provider with OAuth token
let provider = GoogleVertexProvider::new(
    "your-gcp-project-id".to_string(),
    "us-central1".to_string(),
    access_token,  // From gcloud auth print-access-token
)?;

// Completions
let response = provider.complete(request).await?;

// Embeddings (text-embedding-004 model)
let embeddings = provider.embed("text to embed").await?;
```

**OAuth Token Management:**

For production deployments, implement automatic token refresh:

```rust
use google_auth_lib::ApplicationDefaultCredentialsFlowOpts;

async fn get_vertex_token() -> Result<String> {
    let opts = ApplicationDefaultCredentialsFlowOpts::default();
    let flow = google_auth_lib::ApplicationDefaultCredentialsFlow::new(opts)?;
    let token = flow.token(&["https://www.googleapis.com/auth/cloud-platform"]).await?;
    Ok(token.access_token)
}
```

**Available Models:**
- `gemini-1.5-pro` - Most capable (recommended)
- `gemini-1.5-flash` - Fastest, cost-effective
- `text-embedding-004` - For embeddings

---

### 5. AWS Bedrock (Experimental)

**Status:** ⚠️ Mock implementation - Returns placeholder responses

**Planned for:** Q1 2025 full integration

**Current Capabilities:**
- ✅ API structure defined
- ✅ Request/response models complete
- ⚠️ Returns mock data only

**To Use (when completed):**
```toml
# config.toml (future)
[intelligence]
provider = "aws_bedrock"
model = "anthropic.claude-3-sonnet-20240229-v1:0"
region = "us-east-1"
```

**Tracking:** See `crates/riptide-intelligence/src/providers/aws_bedrock.rs`

---

## Provider Comparison

### Performance

| Provider | Latency | Throughput | Context Window |
|----------|---------|------------|----------------|
| Claude 3.5 Sonnet | 2-5s | 50 req/min | 200K tokens |
| Claude 3.5 Haiku | 1-2s | 100 req/min | 200K tokens |
| Ollama (llama3.2) | <1s | Unlimited* | 8K tokens |
| LocalAI | <1s | Unlimited* | 4-8K tokens |
| Vertex Gemini Pro | 2-4s | 60 req/min | 1M tokens |

*Limited by hardware

### Cost Comparison (per 1M tokens)

| Provider | Input | Output | Best For |
|----------|-------|--------|----------|
| Claude Haiku 3.5 | $0.80 | $4.00 | High volume, speed |
| Claude Sonnet 3.5 | $3.00 | $15.00 | Balanced workloads |
| Claude Opus 3 | $15.00 | $75.00 | Complex analysis |
| Ollama | FREE | FREE | Development, privacy |
| LocalAI | FREE | FREE | Development, privacy |
| Vertex Gemini Pro | $1.25 | $5.00 | Google Cloud users |

### Feature Matrix

| Feature | Anthropic | Ollama | LocalAI | Vertex AI | Bedrock |
|---------|-----------|--------|---------|-----------|---------|
| Completions | ✅ | ✅ | ✅ | ✅ | ⚠️ |
| Embeddings | ❌ | ✅ | ✅ | ✅ | ⚠️ |
| Function Calling | ✅ | ❌ | ✅ | ✅ | ⚠️ |
| Streaming | ✅ | ✅ | ✅ | ✅ | ⚠️ |
| Local Deployment | ❌ | ✅ | ✅ | ❌ | ❌ |

---

## Advanced Configuration

### Dynamic Provider Selection

```rust
use riptide_intelligence::create_provider_from_config;

let config = IntelligenceConfig {
    provider: "anthropic".to_string(),
    model: Some("claude-3-5-sonnet-20241022".to_string()),
    api_key: Some(env::var("ANTHROPIC_API_KEY")?),
    ..Default::default()
};

let provider = create_provider_from_config(&config)?;
```

### Cost Estimation

```rust
// Estimate cost before making request
let estimated_tokens = 1000;
let cost = provider.estimate_cost(estimated_tokens);

println!("Estimated cost: ${:.4}", cost.total_cost);
```

### Health Checks

```rust
// Verify provider is reachable
provider.health_check().await?;
```

### Error Handling

```rust
use riptide_intelligence::IntelligenceError;

match provider.complete(request).await {
    Ok(response) => println!("Success: {}", response.content),
    Err(IntelligenceError::Network(e)) => {
        eprintln!("Network error: {}", e);
        // Retry logic
    },
    Err(IntelligenceError::Provider(e)) => {
        eprintln!("Provider error: {}", e);
        // Check API key, rate limits
    },
    Err(IntelligenceError::RateLimit { retry_after }) => {
        eprintln!("Rate limited, retry after: {:?}", retry_after);
        tokio::time::sleep(retry_after).await;
    },
    Err(e) => eprintln!("Other error: {}", e),
}
```

---

## Integration with RipTide Extraction

### Intelligent Content Extraction

```rust
use riptide_api::handlers::extraction::ExtractionRequest;

let extraction_request = ExtractionRequest {
    url: "https://example.com/article".to_string(),
    intelligence_config: Some(IntelligenceConfig {
        provider: "anthropic".to_string(),
        model: Some("claude-3-5-haiku-20241022".to_string()),
        extraction_prompt: Some(
            "Extract the main points from this article in bullet format".to_string()
        ),
        ..Default::default()
    }),
    ..Default::default()
};

// RipTide will automatically use LLM for extraction
let response = client.post("/extract")
    .json(&extraction_request)
    .send()
    .await?;
```

---

## Production Best Practices

### 1. API Key Management

```bash
# Use environment variables (recommended)
export ANTHROPIC_API_KEY="sk-ant-..."

# Or use secret management (production)
# - AWS Secrets Manager
# - Google Secret Manager
# - HashiCorp Vault
```

### 2. Rate Limiting

```rust
// Implement rate limiting for paid providers
use governor::{Quota, RateLimiter};

let limiter = RateLimiter::direct(Quota::per_minute(nonzero!(50u32)));
limiter.until_ready().await;

let response = provider.complete(request).await?;
```

### 3. Fallback Strategy

```rust
// Try primary provider, fall back to secondary
async fn complete_with_fallback(request: CompletionRequest) -> Result<CompletionResponse> {
    match anthropic_provider.complete(request.clone()).await {
        Ok(response) => Ok(response),
        Err(e) => {
            warn!("Anthropic failed: {}, falling back to Ollama", e);
            ollama_provider.complete(request).await
        }
    }
}
```

### 4. Caching

```rust
// Cache responses for identical requests
use moka::future::Cache;

let cache: Cache<String, CompletionResponse> = Cache::builder()
    .max_capacity(1000)
    .time_to_live(Duration::from_hours(24))
    .build();

let cache_key = format!("{:?}", request);
if let Some(cached) = cache.get(&cache_key).await {
    return Ok(cached);
}

let response = provider.complete(request).await?;
cache.insert(cache_key, response.clone()).await;
```

---

## Troubleshooting

### Anthropic

**Error: "Invalid API key"**
- Verify key format: `sk-ant-api03-...`
- Check key hasn't expired
- Ensure correct environment variable

**Error: "Rate limit exceeded"**
- Anthropic has tiered rate limits
- Implement exponential backoff
- Consider upgrading plan

### Ollama

**Error: "Connection refused"**
- Check Ollama is running: `ollama list`
- Verify port 11434 is accessible
- Try: `ollama serve`

**Error: "Model not found"**
- Pull model first: `ollama pull llama3.2`
- List available models: `ollama list`

### Vertex AI

**Error: "Invalid authentication credentials"**
- Regenerate access token: `gcloud auth print-access-token`
- Check service account permissions
- Verify project ID is correct

**Error: "Quota exceeded"**
- Check quota in GCP Console
- Request quota increase
- Implement request batching

---

## Monitoring & Metrics

### Track Provider Performance

```rust
use riptide_intelligence::LlmProvider;

let start = Instant::now();
let response = provider.complete(request).await?;
let duration = start.elapsed();

// Log metrics
metrics.record_llm_request(
    provider.name(),
    response.model.clone(),
    duration,
    response.usage.total_tokens,
    cost.total_cost,
);
```

### Prometheus Metrics

```
riptide_llm_requests_total{provider="anthropic",model="claude-3-5-sonnet"} 1523
riptide_llm_tokens_total{provider="anthropic",model="claude-3-5-sonnet"} 45690
riptide_llm_cost_dollars{provider="anthropic",model="claude-3-5-sonnet"} 0.68
riptide_llm_latency_seconds{provider="anthropic",model="claude-3-5-sonnet"} 2.4
```

---

## Future Roadmap

- [ ] **AWS Bedrock** - Complete SDK integration (Q1 2025)
- [ ] **Azure OpenAI** - Add support (Q2 2025)
- [ ] **Cohere** - Add support (Q2 2025)
- [ ] **HuggingFace Inference** - Add support (Q3 2025)
- [ ] **Ollama Model Discovery** - Auto-detect available models (Q1 2025)
- [ ] **Provider Auto-Selection** - Choose best provider based on task
- [ ] **Multi-Provider Ensembles** - Combine responses from multiple providers

---

## Support & Resources

- **Documentation:** `/docs/intelligence-providers.md`
- **Examples:** `/examples/intelligence_provider_demo.rs`
- **Issues:** https://github.com/your-repo/riptide/issues
- **Provider Docs:**
  - [Anthropic Claude](https://docs.anthropic.com/)
  - [Ollama](https://ollama.ai/)
  - [LocalAI](https://localai.io/)
  - [Google Vertex AI](https://cloud.google.com/vertex-ai/docs)

---

**Version:** 1.0.0
**Status:** ✅ Production Ready (4/5 providers)
**Last Verified:** 2025-10-06
