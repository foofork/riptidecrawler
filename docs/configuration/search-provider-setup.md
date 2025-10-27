# Search Provider Configuration Guide

**Status:** ✅ Complete
**Version:** 1.0.0
**Last Updated:** 2025-10-27

---

## 📋 Overview

RipTide API supports multiple search providers through a flexible backend system with graceful degradation. This guide covers configuration, deployment scenarios, and troubleshooting.

**Key Features:**
- ✅ **3 Search Backends**: Serper, None (URL parsing), SearXNG
- ✅ **Graceful Degradation**: Automatic fallback when dependencies missing
- ✅ **Zero Dependencies**: Works out-of-the-box with "none" backend
- ✅ **Circuit Breaker**: Built-in reliability and fault tolerance

---

## 🚀 Quick Start

### Option 1: No External Dependencies (Default)

Perfect for development and direct URL extraction:

```bash
# .env configuration
RIPTIDE_SEARCH_BACKEND=none
SEARCH_BACKEND=none  # Legacy compatibility
```

**Features:**
- ✅ No API key required
- ✅ Zero external dependencies
- ✅ Instant startup
- ✅ Direct URL parsing from queries

**Use Cases:**
- Local development
- CI/CD testing
- Direct URL extraction
- When you don't need web search

---

### Option 2: Google Search via Serper.dev

Production-grade Google search results:

```bash
# .env configuration
RIPTIDE_SEARCH_BACKEND=serper
SEARCH_BACKEND=serper  # Legacy compatibility
SERPER_API_KEY=your_serper_api_key_here
```

**Requirements:**
1. Sign up at [serper.dev](https://serper.dev)
2. Get your API key
3. Set `SERPER_API_KEY` environment variable

**Features:**
- ✅ Real Google search results
- ✅ Fast and reliable
- ✅ 5,000 free searches/month
- ✅ Production-ready

**Pricing:**
- Free tier: 5,000 searches/month
- Paid plans: Starting at $50/month for 10,000 searches

---

### Option 3: Self-Hosted SearXNG

Privacy-focused meta-search engine:

```bash
# .env configuration
RIPTIDE_SEARCH_BACKEND=searxng
SEARCH_BACKEND=searxng  # Legacy compatibility
SEARXNG_BASE_URL=http://localhost:8888
```

**Requirements:**
1. Deploy SearXNG instance ([docs](https://docs.searxng.org/))
2. Set `SEARXNG_BASE_URL` to your instance URL

**Features:**
- ✅ Privacy-focused
- ✅ No external API dependencies
- ✅ Self-hosted control
- ✅ Free (infrastructure costs only)

**Setup:**
```bash
# Docker deployment
docker run -d \
  --name searxng \
  -p 8888:8080 \
  searxng/searxng:latest
```

---

## 📚 Configuration Reference

### Environment Variables

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `RIPTIDE_SEARCH_BACKEND` | Enum | `none` | Search backend: `serper`, `none`, or `searxng` |
| `SEARCH_BACKEND` | Enum | `none` | Legacy compatibility (use `RIPTIDE_SEARCH_BACKEND` instead) |
| `SERPER_API_KEY` | String | - | Serper.dev API key (required for `serper` backend) |
| `SEARXNG_BASE_URL` | URL | - | SearXNG instance URL (required for `searxng` backend) |
| `RIPTIDE_SEARCH_TIMEOUT_SECS` | Integer | `30` | Search operation timeout in seconds |
| `RIPTIDE_SEARCH_ENABLE_URL_PARSING` | Boolean | `true` | Enable URL parsing for None provider |
| `RIPTIDE_SEARCH_CIRCUIT_BREAKER_FAILURE_THRESHOLD` | Integer | `50` | Circuit breaker failure threshold (0-100) |
| `RIPTIDE_SEARCH_CIRCUIT_BREAKER_MIN_REQUESTS` | Integer | `5` | Minimum requests before circuit opens |
| `RIPTIDE_SEARCH_CIRCUIT_BREAKER_RECOVERY_TIMEOUT_SECS` | Integer | `60` | Circuit breaker recovery timeout |

---

## 🔄 Graceful Degradation

RipTide API **never fails to start** due to missing search configuration. Instead:

### Scenario 1: Missing Serper API Key

```bash
# Configuration
RIPTIDE_SEARCH_BACKEND=serper
# SERPER_API_KEY not set (missing!)
```

**Behavior:**
1. ⚠️  Warning logged during startup
2. ✅ System automatically falls back to `none` backend
3. ✅ API starts successfully
4. ℹ️  `/api/search` endpoint returns helpful error message

**Error Response:**
```json
{
  "error": "dependency_unavailable",
  "message": "Search provider not configured. Set SERPER_API_KEY environment variable to enable search, or use SEARCH_BACKEND=none for URL parsing.",
  "dependency": "search_provider",
  "suggested_fix": "Set SERPER_API_KEY or change SEARCH_BACKEND to 'none'"
}
```

---

### Scenario 2: Missing SearXNG URL

```bash
# Configuration
RIPTIDE_SEARCH_BACKEND=searxng
# SEARXNG_BASE_URL not set (missing!)
```

**Behavior:**
1. ⚠️  Warning logged during startup
2. ✅ System automatically falls back to `none` backend
3. ✅ API starts successfully

---

### Scenario 3: Invalid API Key

```bash
# Configuration
RIPTIDE_SEARCH_BACKEND=serper
SERPER_API_KEY=invalid_key_12345
```

**Behavior:**
1. ✅ API starts successfully (no validation at startup)
2. ⚠️  First search request fails with authentication error
3. 🔴 Circuit breaker opens after threshold failures
4. ℹ️  Helpful error message returned to client

**Error Response:**
```json
{
  "error": "search_failed",
  "message": "Search provider authentication failed. Check SERPER_API_KEY.",
  "provider": "serper",
  "suggested_fix": "Verify your SERPER_API_KEY at https://serper.dev"
}
```

---

## 🌍 Deployment Scenarios

### Local Development

**Recommended:** No external dependencies

```bash
# .env.development
RIPTIDE_SEARCH_BACKEND=none
RIPTIDE_SEARCH_ENABLE_URL_PARSING=true
```

**Why:**
- ✅ Zero setup required
- ✅ Fast startup
- ✅ No API costs
- ✅ Perfect for testing URL extraction

---

### Staging Environment

**Recommended:** Serper.dev with free tier

```bash
# .env.staging
RIPTIDE_SEARCH_BACKEND=serper
SERPER_API_KEY=your_staging_api_key
RIPTIDE_SEARCH_TIMEOUT_SECS=30
RIPTIDE_SEARCH_CIRCUIT_BREAKER_FAILURE_THRESHOLD=50
```

**Why:**
- ✅ Real Google search results
- ✅ Free tier sufficient for testing
- ✅ Production-like behavior
- ✅ Circuit breaker protection

---

### Production (Public SaaS)

**Recommended:** Serper.dev with paid plan

```bash
# .env.production
RIPTIDE_SEARCH_BACKEND=serper
SERPER_API_KEY=your_production_api_key
RIPTIDE_SEARCH_TIMEOUT_SECS=30
RIPTIDE_SEARCH_CIRCUIT_BREAKER_FAILURE_THRESHOLD=30
RIPTIDE_SEARCH_CIRCUIT_BREAKER_MIN_REQUESTS=10
RIPTIDE_SEARCH_CIRCUIT_BREAKER_RECOVERY_TIMEOUT_SECS=120
```

**Configuration Notes:**
- Lower failure threshold (30%) for faster circuit opening
- Higher min requests (10) to avoid false positives
- Longer recovery timeout (120s) for stability

---

### Production (Privacy-Focused)

**Recommended:** Self-hosted SearXNG

```bash
# .env.production
RIPTIDE_SEARCH_BACKEND=searxng
SEARXNG_BASE_URL=https://searx.internal.company.com
RIPTIDE_SEARCH_TIMEOUT_SECS=45
```

**Infrastructure:**
```yaml
# docker-compose.yml
services:
  searxng:
    image: searxng/searxng:latest
    ports:
      - "8888:8080"
    volumes:
      - ./searxng:/etc/searxng
    restart: unless-stopped
```

---

### CI/CD Testing

**Recommended:** None backend for zero dependencies

```bash
# .env.ci
RIPTIDE_SEARCH_BACKEND=none
RIPTIDE_SEARCH_ENABLE_URL_PARSING=true
# No external dependencies = faster, more reliable tests
```

---

## 🔍 Backend Comparison

| Feature | None | Serper | SearXNG |
|---------|------|--------|---------|
| **External Dependencies** | ❌ None | ✅ API Key | ✅ Self-hosted |
| **Setup Complexity** | ⭐ Trivial | ⭐⭐ Easy | ⭐⭐⭐ Moderate |
| **Cost** | 🆓 Free | 💰 Paid (free tier) | 🆓 Free (hosting costs) |
| **Search Quality** | N/A (URL only) | ⭐⭐⭐⭐⭐ Excellent | ⭐⭐⭐⭐ Good |
| **Privacy** | 🔒 Local | 🔓 Third-party API | 🔒 Self-hosted |
| **Speed** | ⚡⚡⚡ Instant | ⚡⚡ Fast | ⚡⚡ Fast |
| **Reliability** | ⭐⭐⭐⭐⭐ Always available | ⭐⭐⭐⭐ API dependent | ⭐⭐⭐ Self-managed |
| **Best For** | Development, URLs | Production, SaaS | Privacy, Enterprise |

---

## 🧪 Testing Your Configuration

### 1. Verify Environment Variables

```bash
# Check current configuration
env | grep SEARCH
env | grep SERPER
env | grep SEARXNG
```

### 2. Test API Health Check

```bash
# Check if API started successfully
curl http://localhost:8080/health

# Should return:
# {
#   "status": "healthy",
#   "search_backend": "none" | "serper" | "searxng"
# }
```

### 3. Test Search Endpoint

```bash
# Test with None backend (URL parsing)
curl "http://localhost:8080/api/search?q=https://example.com"

# Test with Serper backend (real search)
curl "http://localhost:8080/api/search?q=rust+programming&limit=10"

# Expected response:
# {
#   "query": "rust programming",
#   "results": [...],
#   "total_results": 10,
#   "provider_used": "Serper",
#   "search_time_ms": 234
# }
```

### 4. Monitor Logs

```bash
# Watch for search-related logs
RUST_LOG=info cargo run --bin riptide-api 2>&1 | grep -i search

# Expected startup logs:
# [INFO] Initializing SearchFacade with backend: none
# [INFO] SearchFacade initialized successfully
```

---

## ❌ Troubleshooting

### Issue: "SearchFacade not initialized"

**Symptoms:**
```json
{
  "error": "dependency_unavailable",
  "message": "SearchFacade not initialized - no search backend configured"
}
```

**Causes:**
1. Missing required environment variable
2. Invalid backend configuration
3. Backend initialization failure

**Solutions:**

#### Solution 1: Check Environment Variables
```bash
# Verify variables are set
echo $RIPTIDE_SEARCH_BACKEND
echo $SERPER_API_KEY  # If using serper
echo $SEARXNG_BASE_URL  # If using searxng

# If not set, add to .env
cp .env.example .env
nano .env
```

#### Solution 2: Use None Backend (Fallback)
```bash
# .env
RIPTIDE_SEARCH_BACKEND=none
SEARCH_BACKEND=none
```

#### Solution 3: Check Application Logs
```bash
# Look for initialization errors
RUST_LOG=debug cargo run --bin riptide-api 2>&1 | grep SearchFacade

# Common errors:
# - "Failed to initialize SearchFacade: API key required"
# - "Failed to initialize SearchFacade: Invalid base URL"
```

---

### Issue: "Search provider authentication failed"

**Symptoms:**
```json
{
  "error": "search_failed",
  "message": "Search provider authentication failed"
}
```

**Causes:**
1. Invalid Serper API key
2. Expired API key
3. Exceeded API quota

**Solutions:**

#### Solution 1: Verify API Key
```bash
# Test API key directly
curl -X POST https://google.serper.dev/search \
  -H "X-API-KEY: $SERPER_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"q": "test"}'

# Should return search results, not authentication error
```

#### Solution 2: Check Serper Dashboard
1. Log in to [serper.dev](https://serper.dev)
2. Check API key status
3. Verify quota limits
4. Generate new API key if needed

#### Solution 3: Fallback to None Backend
```bash
# Temporary workaround
RIPTIDE_SEARCH_BACKEND=none
```

---

### Issue: Circuit Breaker Opens

**Symptoms:**
```json
{
  "error": "search_unavailable",
  "message": "Search provider circuit breaker is open"
}
```

**Causes:**
1. Too many failures (>50% by default)
2. Backend service unavailable
3. Network issues

**Solutions:**

#### Solution 1: Wait for Recovery
- Circuit breaker auto-recovers after 60 seconds (default)
- Monitor logs for recovery message

#### Solution 2: Adjust Circuit Breaker Settings
```bash
# More lenient settings for unreliable networks
RIPTIDE_SEARCH_CIRCUIT_BREAKER_FAILURE_THRESHOLD=75  # Allow more failures
RIPTIDE_SEARCH_CIRCUIT_BREAKER_MIN_REQUESTS=10  # More samples before opening
RIPTIDE_SEARCH_CIRCUIT_BREAKER_RECOVERY_TIMEOUT_SECS=30  # Faster recovery
```

#### Solution 3: Check Backend Health
```bash
# For SearXNG
curl http://localhost:8888/healthz

# For Serper
curl -I https://google.serper.dev
```

---

### Issue: SearXNG Connection Refused

**Symptoms:**
```
Failed to initialize SearchFacade: Connection refused
```

**Causes:**
1. SearXNG not running
2. Wrong URL/port
3. Network policy blocking

**Solutions:**

#### Solution 1: Start SearXNG
```bash
# Check if running
docker ps | grep searxng

# Start if not running
docker-compose up -d searxng

# Check logs
docker logs searxng
```

#### Solution 2: Verify URL
```bash
# Test connectivity
curl http://localhost:8888

# Update .env with correct URL
SEARXNG_BASE_URL=http://searxng:8080  # Docker network
# or
SEARXNG_BASE_URL=http://localhost:8888  # Local
```

---

## 📊 Monitoring and Metrics

### Circuit Breaker Metrics

```bash
# Check circuit breaker status
curl http://localhost:8080/metrics | grep search_circuit_breaker

# Metrics available:
# - search_circuit_breaker_state (0=closed, 1=open, 2=half_open)
# - search_circuit_breaker_failures_total
# - search_circuit_breaker_successes_total
# - search_circuit_breaker_requests_total
```

### Search Performance Metrics

```bash
# Query search performance
curl http://localhost:8080/metrics | grep search_duration

# Metrics available:
# - search_duration_seconds (histogram)
# - search_requests_total (counter)
# - search_errors_total (counter)
# - search_results_total (counter)
```

---

## 🔐 Security Best Practices

### 1. Protect API Keys

```bash
# ❌ DON'T commit to git
# .env
SERPER_API_KEY=sk-abc123  # NEVER commit this!

# ✅ DO use environment variables
export SERPER_API_KEY=$(cat /secrets/serper-key)

# ✅ DO use secret management
# Kubernetes secret, AWS Secrets Manager, HashiCorp Vault, etc.
```

### 2. Use Read-Only API Keys

- Create separate API keys for different environments
- Use minimum required permissions
- Rotate keys regularly

### 3. Rate Limiting

```bash
# Protect against abuse
RIPTIDE_RATE_LIMIT_ENABLED=true
RIPTIDE_RATE_LIMIT_RPS=1.5  # 1.5 requests per second
```

---

## 📚 Additional Resources

### Related Documentation
- [Environment Variables Guide](ENVIRONMENT-VARIABLES.md)
- [API Reference](../02-api-reference/README.md)
- [Error Handling](../02-api-reference/error-handling.md)

### External Resources
- [Serper.dev Documentation](https://serper.dev/docs)
- [SearXNG Documentation](https://docs.searxng.org/)
- [Circuit Breaker Pattern](https://learn.microsoft.com/en-us/azure/architecture/patterns/circuit-breaker)

---

## 🎯 Success Checklist

- [ ] Environment variables configured in `.env`
- [ ] Search backend set (`none`, `serper`, or `searxng`)
- [ ] API key configured (if using `serper`)
- [ ] SearXNG deployed (if using `searxng`)
- [ ] API health check returns `healthy`
- [ ] Search endpoint returns results
- [ ] Circuit breaker metrics available
- [ ] Logs show successful initialization
- [ ] Error handling tested
- [ ] Fallback behavior verified

---

**Last Updated:** 2025-10-27
**Maintained By:** RipTide Core Team
**Questions?** See [Contributing Guide](../../CONTRIBUTING.md)
