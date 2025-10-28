# RipTide Deployment Guide

## Quick Start (Recommended for Production)

```bash
# 1. Clone repository
git clone <repo-url>
cd riptide  # Or whatever directory name you chose

# 2. Copy environment template
cp .env.example .env

# 3. Configure API keys (required)
nano .env  # Set SERPER_API_KEY and optionally OPENAI_API_KEY

# 4. Start all services
docker-compose -f docker-compose.production.yml up -d

# 5. Verify deployment
curl http://localhost:8080/health
```

That's it! You now have:
- **API** at http://localhost:8080
- **Swagger UI** at http://localhost:8081
- **Redis** for caching
- **Headless Service** (internal, automatic)

---

## 🎯 Deployment Modes

### Mode 1: Microservices (Default)

**What it is:** API and Chrome run in separate containers

**Configuration:**
```yaml
# docker-compose.production.yml (default)
riptide-api:
  environment:
    - HEADLESS_URL=http://riptide-headless:9123  # Enabled by default
```

**Provides:**
- ✅ Full JavaScript rendering (via remote Chrome)
- ✅ Complex page scraping (SPAs, dynamic content)
- ✅ PDF generation
- ✅ WASM extraction (17MB module)
- ✅ Redis caching
- ✅ Automatic scaling

**When to use:**
- Production deployments
- Cloud hosting (AWS, GCP, Azure)
- Multi-instance scaling
- High-traffic scenarios

---

### Mode 2: Monolithic (Simplified)

**What it is:** API and Chrome run in the same container

**Configuration:**
```yaml
# docker-compose.simple.yml (create this)
riptide-api:
  environment:
    # Comment out or remove HEADLESS_URL
    # - HEADLESS_URL=...

# Remove riptide-headless service entirely
```

**Provides:**
- ✅ Same features as Mode 1
- ✅ Simpler setup (one container)
- ⚠️ Larger container (951MB vs 168MB)
- ⚠️ Can't scale Chrome independently

**When to use:**
- Local development
- Small-scale deployments
- Single-server hosting
- Testing

---

## 🔧 Configuration Details

### Environment Variables

**Required:**
```bash
SERPER_API_KEY=your_serper_key_here  # For web search functionality
```

**Optional:**
```bash
# LLM Integration (for AI-powered extraction)
OPENAI_API_KEY=your_openai_key
ANTHROPIC_API_KEY=your_anthropic_key

# Chrome Mode Selection
HEADLESS_URL=http://riptide-headless:9123  # Comment out for local Chrome mode

# Performance Tuning
RIPTIDE_MAX_CONCURRENT_REQUESTS=100
RIPTIDE_POOL_SIZE=10
RUST_LOG=info
```

---

## 🎭 How Chrome Mode Selection Works

### At Startup (Configuration Time)

```rust
// The API checks HEADLESS_URL environment variable:

if HEADLESS_URL is set:
    ✅ Use Remote Chrome Mode
    - Skip local browser pool creation
    - No Chrome needed in API container
    - Send render requests via HTTP to headless service

else:
    ✅ Use Local Chrome Mode
    - Create local browser pool
    - Chrome MUST be installed in container
    - Render directly using local Chrome
```

### Per Request (Automatic)

```rust
// When a render request comes in:

if config.headless_url.is_some():
    // Send to remote Chrome
    let client = RpcClient::with_url(headless_url);
    let result = client.render_dynamic(url).await?;
else:
    // Use local Chrome
    let browser = resource_manager.browser_pool.checkout()?;
    let result = browser.goto(url).await?;
```

**The user doesn't choose per-request** - it's determined by startup configuration.

---

## 📦 What's Included (Full Functionality)

Both modes provide **100% feature parity**:

| Feature | Microservices | Monolithic |
|---------|---------------|------------|
| REST API endpoints | ✅ | ✅ |
| JavaScript rendering | ✅ | ✅ |
| WASM extraction (17MB) | ✅ | ✅ |
| PDF generation | ✅ | ✅ |
| Redis caching | ✅ | ✅ |
| Rate limiting (1.5 RPS/host) | ✅ | ✅ |
| WebSocket support | ✅ | ✅ |
| Stealth mode (anti-bot) | ✅ | ✅ |
| Search integration (Serper) | ✅ | ✅ |
| LLM integration | ✅ | ✅ |

**Difference:** Only deployment architecture, not features.

---

## 🚦 Health Checks

### Verify Deployment

```bash
# Check API health
curl http://localhost:8080/health

# Expected response:
{
  "status": "healthy",
  "version": "0.1.0",
  "uptime_seconds": 42,
  "services": {
    "redis": "connected",
    "headless": "available"  # or "not_configured" for local mode
  }
}
```

### Check Logs

```bash
# Microservices mode - expect these logs:
docker logs riptide-api
# INFO riptide_api::state: Headless service URL configured - skipping local browser launcher initialization
# INFO riptide_api::resource_manager: Headless service URL configured - skipping local browser pool initialization
# INFO riptide_api::state: WASM extractor loaded: /opt/riptide/extractor/extractor.wasm

docker logs riptide-headless
# INFO riptide_headless: Headless browser service starting on 0.0.0.0:9123
# INFO riptide_headless: Browser pool initialized with 3 instances
```

---

## 🔄 Switching Between Modes

### From Microservices → Monolithic

1. Edit docker-compose.yml:
```yaml
riptide-api:
  environment:
    # - HEADLESS_URL=http://riptide-headless:9123  # Comment out
```

2. Remove headless service or build all-in-one image
3. Restart: `docker-compose up -d`

### From Monolithic → Microservices

1. Uncomment HEADLESS_URL
2. Ensure headless service is defined
3. Restart: `docker-compose up -d`

No code changes needed - purely configuration!

---

## 🎯 Recommended Setup for New Users

**Use the default: Microservices mode (docker-compose.production.yml)**

**Why?**
- Production-ready out of the box
- Better resource management
- Easier to scale later
- Chrome issues don't crash API
- Modern microservices architecture

**Setup:**
```bash
# Just run this - everything is pre-configured:
docker-compose -f docker-compose.production.yml up -d
```

---

## 📊 Resource Requirements

### Microservices Mode
```
API Container:     168MB, 512MB RAM, 0.5 CPU
Headless Container: 783MB, 512MB RAM, 0.5 CPU
Redis Container:    ~30MB,  256MB RAM, 0.25 CPU
---------------------------------------------------
Total:             ~981MB, 1.28GB RAM, 1.25 CPU
```

### Monolithic Mode
```
API Container:     951MB, 1GB RAM, 1 CPU
Redis Container:   ~30MB, 256MB RAM, 0.25 CPU
---------------------------------------------------
Total:             ~981MB, 1.28GB RAM, 1.25 CPU
```

**Same resources** - microservices just separates concerns.

---

## 🆘 Troubleshooting

### "Could not auto detect a chrome executable"

**Problem:** API trying to use local Chrome but it's not installed

**Solution:** Set `HEADLESS_URL` environment variable:
```yaml
environment:
  - HEADLESS_URL=http://riptide-headless:9123
```

### "Connection refused to headless service"

**Problem:** Headless service not running or wrong URL

**Check:**
```bash
docker ps | grep headless  # Is it running?
docker logs riptide-headless  # Any errors?
```

**Fix:**
```bash
docker-compose up -d riptide-headless
```

### "WASM extractor not found"

**Problem:** WASM module not at expected path

**Check:**
```bash
docker exec riptide-api ls -lh /opt/riptide/extractor/
```

**Fix:** Rebuild API image with correct WASM path

---

## 🔐 Security Considerations

### Headless Service

- Runs Chrome with `--no-sandbox` (required for Docker)
- Isolated in separate container
- Not exposed to internet (internal network only)
- Port 9123 only accessible to API container

### API Service

- No Chrome execution (microservices mode)
- Validates all inputs
- Rate limits per host (1.5 RPS)
- Redis for session management
- Environment variables for secrets

---

## 📚 Next Steps

1. ✅ Start with microservices mode (default)
2. ✅ Test with sample requests
3. ✅ Monitor logs and metrics
4. ✅ Scale as needed
5. ✅ Configure custom rate limits
6. ✅ Add LLM integration (optional)

**Documentation:**
- API Reference: http://localhost:8081 (Swagger UI)
- Architecture: `/docs/deployment/CHROME_DETECTION_FIX.md`
- Implementation: `/docs/DEPLOYMENT_COMPLETE_SUCCESS.md`
