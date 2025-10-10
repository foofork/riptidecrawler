# LLM Provider Setup Guide

This guide provides complete setup instructions for all supported LLM providers in Riptide Intelligence.

## Table of Contents

1. [OpenAI](#openai)
2. [Anthropic (Claude)](#anthropic-claude)
3. [Google Vertex AI](#google-vertex-ai)
4. [AWS Bedrock](#aws-bedrock)
5. [Ollama (Local)](#ollama-local)
6. [LocalAI](#localai)
7. [Azure OpenAI](#azure-openai)
8. [API Usage](#api-usage)
9. [Troubleshooting](#troubleshooting)

---

## OpenAI

### Requirements
- OpenAI API key
- Model selection (e.g., `gpt-4`, `gpt-3.5-turbo`, `gpt-4-turbo`)

### Configuration

```rust
use riptide_intelligence::{LlmRegistry, ProviderConfig};
use serde_json::json;

let registry = LlmRegistry::new();
let config = ProviderConfig::new("openai-main", "openai")
    .with_config("api_key", json!("sk-..."))
    .with_config("model", json!("gpt-4"))
    .with_fallback_order(1);

registry.load_provider(config)?;
```

### Environment Variables

```bash
export OPENAI_API_KEY="sk-..."
export OPENAI_MODEL="gpt-4"
```

### Pricing (as of 2024)
- **GPT-4**: $0.03/1K input tokens, $0.06/1K output tokens
- **GPT-3.5 Turbo**: $0.001/1K input tokens, $0.002/1K output tokens

### Common Issues

**Issue**: `401 Unauthorized`
- **Solution**: Verify API key is valid and has not expired

**Issue**: Rate limit exceeded
- **Solution**: Implement retry logic with exponential backoff or upgrade to higher tier

---

## Anthropic (Claude)

### Requirements
- Anthropic API key
- Model selection (e.g., `claude-3-opus`, `claude-3-sonnet`, `claude-3-haiku`)

### Configuration

```rust
let config = ProviderConfig::new("anthropic-main", "anthropic")
    .with_config("api_key", json!("sk-ant-..."))
    .with_config("model", json!("claude-3-sonnet-20240229"))
    .with_fallback_order(2);

registry.load_provider(config)?;
```

### Environment Variables

```bash
export ANTHROPIC_API_KEY="sk-ant-..."
export ANTHROPIC_MODEL="claude-3-sonnet-20240229"
```

### Pricing (as of 2024)
- **Claude 3 Opus**: $0.015/1K input tokens, $0.075/1K output tokens
- **Claude 3 Sonnet**: $0.003/1K input tokens, $0.015/1K output tokens
- **Claude 3 Haiku**: $0.00025/1K input tokens, $0.00125/1K output tokens

### Features
- Large context window (200K tokens)
- Strong reasoning capabilities
- No function calling support (yet)

---

## Google Vertex AI

### Requirements
- Google Cloud Project ID
- Service account with Vertex AI permissions
- Location/region selection (e.g., `us-central1`)
- OAuth2 access token or service account credentials

### Setup Steps

1. **Enable Vertex AI API**
   ```bash
   gcloud services enable aiplatform.googleapis.com
   ```

2. **Create Service Account**
   ```bash
   gcloud iam service-accounts create vertex-ai-sa \
       --display-name="Vertex AI Service Account"
   ```

3. **Grant Permissions**
   ```bash
   gcloud projects add-iam-policy-binding PROJECT_ID \
       --member="serviceAccount:vertex-ai-sa@PROJECT_ID.iam.gserviceaccount.com" \
       --role="roles/aiplatform.user"
   ```

4. **Generate Access Token**
   ```bash
   gcloud auth application-default login
   gcloud auth application-default print-access-token
   ```

### Configuration

```rust
let config = ProviderConfig::new("vertex-ai-main", "google_vertex")
    .with_config("project_id", json!("my-gcp-project"))
    .with_config("location", json!("us-central1"))
    .with_config("model", json!("gemini-1.5-pro"));

registry.load_provider(config)?;

// Set access token at runtime
let provider = VertexAIProvider::new("my-project", "us-central1")?
    .with_access_token(access_token);
```

### Environment Variables

```bash
export GCP_PROJECT_ID="my-gcp-project"
export GCP_LOCATION="us-central1"
export GOOGLE_APPLICATION_CREDENTIALS="/path/to/service-account-key.json"
```

### Available Models
- **Gemini 1.5 Pro**: Advanced multimodal model
- **Gemini 1.5 Flash**: Fast, efficient model
- **Gemini 1.0 Pro**: Stable production model
- **PaLM 2**: Legacy text generation

### Pricing (approximate, varies by region)
- **Gemini 1.5 Pro**: $0.00125/1K input tokens, $0.00375/1K output tokens
- **Gemini 1.5 Flash**: $0.000075/1K input tokens, $0.0003/1K output tokens

### Common Issues

**Issue**: `Authentication failed`
- **Solution**: Run `gcloud auth application-default login` and ensure service account has correct roles

**Issue**: `Model not found in region`
- **Solution**: Check model availability in your selected region, try `us-central1` or `europe-west4`

---

## AWS Bedrock

### Requirements
- AWS Account with Bedrock access
- IAM credentials (Access Key + Secret Key)
- AWS region with Bedrock availability
- Model access request approved

### Setup Steps

1. **Request Model Access**
   - Navigate to AWS Bedrock Console
   - Request access to desired models (Claude, Titan, Llama)
   - Wait for approval (usually instant for some models)

2. **Create IAM User/Role**
   ```bash
   aws iam create-user --user-name bedrock-user
   aws iam attach-user-policy --user-name bedrock-user \
       --policy-arn arn:aws:iam::aws:policy/AmazonBedrockFullAccess
   aws iam create-access-key --user-name bedrock-user
   ```

### Configuration

```rust
let config = ProviderConfig::new("bedrock-main", "aws_bedrock")
    .with_config("region", json!("us-east-1"))
    .with_config("access_key", json!("AKIA..."))
    .with_config("secret_key", json!("..."))
    .with_config("model", json!("anthropic.claude-3-sonnet-20240229-v1:0"));

registry.load_provider(config)?;
```

### Environment Variables

```bash
export AWS_REGION="us-east-1"
export AWS_ACCESS_KEY_ID="AKIA..."
export AWS_SECRET_ACCESS_KEY="..."
export AWS_SESSION_TOKEN="..."  # Optional for temporary credentials
```

### Available Models

#### Anthropic Claude (via Bedrock)
- `anthropic.claude-3-opus-20240229-v1:0`
- `anthropic.claude-3-sonnet-20240229-v1:0`
- `anthropic.claude-3-haiku-20240307-v1:0`

#### Amazon Titan
- `amazon.titan-text-express-v1`
- `amazon.titan-text-lite-v1`
- `amazon.titan-embed-text-v1` (embeddings)

#### Meta Llama
- `meta.llama2-70b-chat-v1`
- `meta.llama2-13b-chat-v1`

### Pricing (approximate, varies by region)
- **Claude 3 Sonnet**: $0.003/1K input tokens, $0.015/1K output tokens
- **Claude 3 Haiku**: $0.00025/1K input tokens, $0.00125/1K output tokens
- **Titan Express**: $0.0008/1K input tokens, $0.0016/1K output tokens

### Current Implementation Status

**Note**: The current implementation is a **mock/placeholder** for development purposes. Full AWS Bedrock integration requires:

1. Adding the AWS SDK for Rust:
   ```toml
   [dependencies]
   aws-config = "1.0"
   aws-sdk-bedrockruntime = "1.0"
   ```

2. Implementing actual API calls in `complete()` method

See the code comments in `aws_bedrock.rs` for integration examples.

---

## Ollama (Local)

### Requirements
- Ollama installed locally or on network
- Downloaded models

### Setup Steps

1. **Install Ollama**
   ```bash
   # macOS/Linux
   curl https://ollama.ai/install.sh | sh

   # Or download from https://ollama.ai/download
   ```

2. **Download Models**
   ```bash
   ollama pull llama3.2
   ollama pull codellama
   ollama pull mistral
   ollama pull nomic-embed-text  # For embeddings
   ```

3. **Start Ollama Server**
   ```bash
   ollama serve  # Runs on http://localhost:11434 by default
   ```

### Configuration

```rust
let config = ProviderConfig::new("ollama-local", "ollama")
    .with_config("base_url", json!("http://localhost:11434"))
    .with_config("model", json!("llama3.2"));

registry.load_provider(config)?;

// Optionally fetch available models
let mut provider = OllamaProvider::new("http://localhost:11434")?;
provider.fetch_available_models().await?;
```

### Environment Variables

```bash
export OLLAMA_HOST="http://localhost:11434"
export OLLAMA_MODEL="llama3.2"
```

### Available Models
- **Llama 3.2**: Latest Meta model
- **Code Llama**: Specialized for code generation
- **Mistral**: High-performance open model
- **Phi-2**: Lightweight but capable

### Features
- **Free**: No API costs
- **Privacy**: Runs locally
- **Offline**: No internet required after download
- **Fast**: With GPU acceleration

### Performance Tips

1. **GPU Acceleration**
   - NVIDIA GPU: Automatically detected
   - AMD GPU: Set `HSA_OVERRIDE_GFX_VERSION`
   - Apple Silicon: Uses Metal acceleration

2. **Model Quantization**
   - Use quantized models (e.g., `llama3.2:7b-q4`) for better performance
   - Trade-off between quality and speed

3. **Memory Management**
   - Monitor with `ollama ps`
   - Unload models with `ollama stop <model>`

---

## LocalAI

### Requirements
- LocalAI server running
- Compatible models deployed

### Setup Steps

1. **Install LocalAI**
   ```bash
   # Using Docker
   docker run -p 8080:8080 \
       -v $PWD/models:/models \
       quay.io/go-skynet/local-ai:latest
   ```

2. **Deploy Models**
   - Place GGUF model files in `/models` directory
   - Or use model gallery: https://localai.io/models/

### Configuration

```rust
let config = ProviderConfig::new("localai-main", "localai")
    .with_config("base_url", json!("http://localhost:8080"))
    .with_config("model", json!("gpt-3.5-turbo"));  // Model alias

registry.load_provider(config)?;
```

### Environment Variables

```bash
export LOCALAI_BASE_URL="http://localhost:8080"
export LOCALAI_MODEL="gpt-3.5-turbo"
```

### Features
- OpenAI-compatible API
- Supports multiple model formats (GGUF, GPTQ, etc.)
- Built-in embeddings support
- Function calling support

---

## Azure OpenAI

### Requirements
- Azure subscription
- Azure OpenAI resource created
- Deployment of OpenAI models
- API key and endpoint

### Setup Steps

1. **Create Azure OpenAI Resource**
   - Navigate to Azure Portal
   - Create new Azure OpenAI resource
   - Wait for deployment

2. **Deploy Models**
   - In Azure OpenAI Studio, create deployment
   - Select model (GPT-4, GPT-3.5, etc.)
   - Choose deployment name

3. **Get Credentials**
   - Navigate to Keys and Endpoint
   - Copy Key 1 and Endpoint URL

### Configuration

```rust
let config = ProviderConfig::new("azure-openai", "azure_openai")
    .with_config("api_key", json!("..."))
    .with_config("endpoint", json!("https://your-resource.openai.azure.com"))
    .with_config("api_version", json!("2023-12-01-preview"))
    .with_config("deployment", json!("gpt-4-deployment"));

registry.load_provider(config)?;
```

### Environment Variables

```bash
export AZURE_OPENAI_API_KEY="..."
export AZURE_OPENAI_ENDPOINT="https://your-resource.openai.azure.com"
export AZURE_OPENAI_API_VERSION="2023-12-01-preview"
export AZURE_OPENAI_DEPLOYMENT="gpt-4-deployment"
```

---

## API Usage

### List All Providers

```bash
curl http://localhost:3000/api/v1/llm/providers?include_models=true&include_cost=true
```

**Response:**
```json
{
  "providers": [
    {
      "name": "openai-main",
      "provider_type": "openai",
      "status": "healthy",
      "capabilities": ["text-generation", "embedding", "chat", "function-calling", "streaming"],
      "config_required": ["api_key", "model"],
      "available": true,
      "cost_info": {
        "input_token_cost": 0.001,
        "output_token_cost": 0.002,
        "currency": "USD"
      },
      "models": [
        {
          "name": "gpt-4",
          "context_window": 8192,
          "max_output_tokens": 4096,
          "supports_functions": true
        }
      ]
    }
  ],
  "current_provider": "openai-main",
  "total_providers": 7
}
```

### Switch Provider

```bash
curl -X POST http://localhost:3000/api/v1/llm/providers/switch \
  -H "Content-Type: application/json" \
  -d '{
    "provider_name": "anthropic-main",
    "gradual_rollout": false
  }'
```

### Update Configuration

```bash
curl -X POST http://localhost:3000/api/v1/llm/config \
  -H "Content-Type: application/json" \
  -d '{
    "provider_configs": {
      "openai-main": {
        "api_key": "sk-new-key",
        "model": "gpt-4-turbo"
      }
    },
    "validate": true
  }'
```

---

## Troubleshooting

### Health Check Failures

All providers implement health checks. Common issues:

**Google Vertex AI**
- Error: `Authentication failed`
- Solution: Refresh access token or check service account permissions

**AWS Bedrock**
- Error: `Model not available`
- Solution: Request model access in AWS Console

**Ollama/LocalAI**
- Error: `Connection refused`
- Solution: Ensure server is running and accessible

### Performance Issues

**Slow Response Times**
- Check network latency to API endpoints
- Consider using local providers (Ollama/LocalAI)
- Enable streaming for better perceived performance

**High Costs**
- Monitor token usage with built-in metrics
- Use cheaper models for simple tasks (e.g., Haiku, GPT-3.5)
- Implement caching for repeated requests

### Rate Limiting

**OpenAI/Anthropic**
- Implement exponential backoff
- Use multiple API keys with failover
- Upgrade to higher tier

**Local Providers**
- No rate limits, but consider:
  - CPU/GPU capacity
  - Memory constraints
  - Concurrent request limits

---

## Best Practices

### 1. Multi-Provider Setup

Configure multiple providers with fallback:

```rust
// Primary: OpenAI
let openai_config = ProviderConfig::new("openai", "openai")
    .with_fallback_order(1);

// Fallback 1: Anthropic
let anthropic_config = ProviderConfig::new("anthropic", "anthropic")
    .with_fallback_order(2);

// Fallback 2: Local Ollama
let ollama_config = ProviderConfig::new("ollama", "ollama")
    .with_fallback_order(3);

registry.load_providers(vec![
    openai_config,
    anthropic_config,
    ollama_config,
])?;
```

### 2. Cost Optimization

- Use embeddings from cheaper providers
- Route simple queries to fast, cheap models
- Cache results when possible

### 3. Security

- Never commit API keys to source control
- Use environment variables or secret management
- Rotate keys regularly
- Monitor usage for anomalies

### 4. Monitoring

```rust
// Check health of all providers
let health_status = registry.health_check().await;
for (name, is_healthy) in health_status {
    println!("{}: {}", name, if is_healthy { "✓" } else { "✗" });
}
```

---

## Support

For issues or questions:
- GitHub Issues: https://github.com/ruvnet/riptide/issues
- Documentation: https://riptide.dev/docs
- API Reference: https://riptide.dev/api
