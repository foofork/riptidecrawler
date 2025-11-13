# RipTide Testing Guide

**Quick Answer:** Want to test RipTide instantly with no auth? Use the Docker quick-start below. ⚡

---

## 🚀 Method 1: Docker Quick Start (Recommended)

**Zero configuration, works everywhere, no auth required.**

```bash
# Start RipTide in test mode (no authentication)
docker compose -f docker-compose.test.yml up -d

# Wait 5 seconds for startup, then test:
curl http://localhost:8080/health

# Extract a webpage:
curl -X POST http://localhost:8080/api/extract \
  -H "Content-Type: application/json" \
  -d '{"url": "https://example.com"}'

# Stop when done:
docker compose -f docker-compose.test.yml down
```

**Why Docker?**
- ✅ Consistent environment across all platforms
- ✅ No local dependencies required
- ✅ Pre-configured with auth disabled
- ✅ Works in CI/CD, codespaces, local dev
- ✅ Clean teardown with `docker compose down`

---

## 🧪 Method 2: Quick Test Script

**One command to test everything.**

```bash
# Make executable and run:
chmod +x scripts/quick-test.sh
./scripts/quick-test.sh

# The script will:
# 1. Start services with Docker
# 2. Wait for readiness
# 3. Run comprehensive tests
# 4. Show results and cleanup
```

---

## 💻 Method 3: Local Development (Cargo)

**For developers working on the codebase.**

### Step 1: Create Test Environment File

```bash
# Copy the test environment template:
cp .env.test .env

# This sets REQUIRE_AUTH=false and other test-friendly defaults
```

### Step 2: Start Dependencies

```bash
# Start Redis (optional, but recommended):
docker run -d --name riptide-redis \
  -p 6379:6379 \
  redis:7-alpine

# Or use in-memory mode (no Redis):
export CACHE_BACKEND=memory
```

### Step 3: Run RipTide

```bash
# Option A: Use minimal config (no auth)
cargo run --release -- --config config/deployment/minimal.toml

# Option B: Use environment variables
REQUIRE_AUTH=false cargo run --release

# Option C: Use .env file (if created in Step 1)
cargo run --release
```

### Step 4: Test

```bash
# Health check:
curl http://localhost:8080/health

# Extract example.com:
curl -X POST http://localhost:8080/api/extract \
  -H "Content-Type: application/json" \
  -d '{"url": "https://example.com"}'
```

---

## 🔬 Method 4: Integration Tests

**Automated test suite with no manual setup.**

```bash
# Run all integration tests:
cargo test -p riptide-api --test integration_tests

# Run specific test categories:
cargo test -p riptide-api --test integration_tests -- extraction
cargo test -p riptide-api --test integration_tests -- spider
cargo test -p riptide-api --test integration_tests -- health
```

**Tests automatically:**
- ✅ Start test server with auth disabled
- ✅ Test all API endpoints
- ✅ Validate responses and error handling
- ✅ Clean up after completion

---

## 📋 Common Test Scenarios

### Test 1: Basic Extraction (No Auth)

```bash
curl -X POST http://localhost:8080/api/extract \
  -H "Content-Type: application/json" \
  -d '{
    "url": "https://example.com"
  }'
```

**Expected Response:**
```json
{
  "url": "https://example.com",
  "title": "Example Domain",
  "content": "This domain is for use in illustrative examples...",
  "status": "success"
}
```

---

### Test 2: Batch Crawling

```bash
curl -X POST http://localhost:8080/api/crawl \
  -H "Content-Type: application/json" \
  -d '{
    "urls": [
      "https://example.com",
      "https://docs.rs/tokio"
    ],
    "use_spider": false
  }'
```

---

### Test 3: Spider Mode (Deep Crawl)

```bash
curl -X POST http://localhost:8080/api/crawl \
  -H "Content-Type: application/json" \
  -d '{
    "urls": ["https://example.com"],
    "use_spider": true,
    "max_depth": 2,
    "max_pages": 10
  }'
```

---

### Test 4: Health Check

```bash
# Quick health check:
curl http://localhost:8080/health

# Detailed metrics:
curl http://localhost:8080/metrics
```

---

## 🔐 Testing With Authentication (Production Mode)

**When you want to test auth features:**

### Step 1: Generate API Key

```bash
# Create .env with API key:
echo 'API_KEYS=test-key-abc123,test-key-xyz789' >> .env
echo 'REQUIRE_AUTH=true' >> .env
```

### Step 2: Restart Server

```bash
cargo run --release
```

### Step 3: Test with API Key

```bash
curl -X POST http://localhost:8080/api/extract \
  -H "X-API-Key: test-key-abc123" \
  -H "Content-Type: application/json" \
  -d '{"url": "https://example.com"}'
```

**Expected Behavior:**
- ✅ With valid key → 200 OK
- ❌ Without key → 401 Unauthorized
- ❌ With invalid key → 401 Unauthorized

---

## 🐛 Troubleshooting

### Issue: Still Getting 401 Unauthorized

**Problem:** Server ignoring `REQUIRE_AUTH=false`

**Solutions:**

1. **Check environment variable is set:**
   ```bash
   # Linux/Mac:
   export REQUIRE_AUTH=false
   echo $REQUIRE_AUTH  # Should print "false"

   # Windows PowerShell:
   $env:REQUIRE_AUTH="false"
   echo $env:REQUIRE_AUTH
   ```

2. **Use config file instead:**
   ```bash
   cargo run --release -- --config config/deployment/minimal.toml
   ```

   The minimal.toml has `require_auth = false` built-in.

3. **Use Docker (guaranteed to work):**
   ```bash
   docker compose -f docker-compose.test.yml up -d
   ```

### Issue: Redis Connection Error

**Problem:** `Failed to connect to Redis`

**Solutions:**

1. **Use in-memory mode (no Redis required):**
   ```bash
   CACHE_BACKEND=memory cargo run --release
   ```

2. **Or start Redis with Docker:**
   ```bash
   docker run -d --name riptide-redis -p 6379:6379 redis:7-alpine
   ```

### Issue: Health Endpoint Returns 404

**Problem:** `/health` returns "Resource not found"

**Solution:** The correct endpoints are:
- ✅ `GET /health` (should work)
- ✅ `GET /api/health` (alternative path)
- ✅ `GET /metrics` (Prometheus metrics)

If all return 404, check server logs for routing configuration.

---

## 🎯 Testing Best Practices

### For Development

1. **Use Docker for consistency**
   - Eliminates "works on my machine" issues
   - Pre-configured test environment
   - Easy cleanup

2. **Start with minimal.toml config**
   - No Redis required
   - No authentication
   - Fast startup

3. **Run integration tests frequently**
   ```bash
   cargo test -p riptide-api
   ```

### For CI/CD

1. **Use docker-compose.test.yml**
   ```yaml
   # In your CI pipeline:
   docker compose -f docker-compose.test.yml up -d
   ./scripts/run-integration-tests.sh
   docker compose -f docker-compose.test.yml down
   ```

2. **Set explicit environment variables**
   ```bash
   export REQUIRE_AUTH=false
   export CACHE_BACKEND=memory
   export RUST_LOG=info
   ```

### For Production Testing

1. **Always use authentication**
   ```bash
   REQUIRE_AUTH=true cargo run --release
   ```

2. **Test with Redis**
   ```bash
   docker compose --profile redis up -d
   ```

3. **Enable rate limiting**
   ```toml
   [rate_limit]
   enabled = true
   requests_per_minute = 100
   ```

---

## 📊 Test Checklist

Use this checklist to validate all features:

- [ ] **Health Check** - `GET /health` returns 200
- [ ] **Basic Extraction** - Extract example.com successfully
- [ ] **Batch Processing** - Extract multiple URLs in one request
- [ ] **Spider Mode** - Deep crawl with max_depth > 1
- [ ] **PDF Extraction** - Extract PDF document
- [ ] **Authentication** - Test with API keys (if enabled)
- [ ] **Rate Limiting** - Verify rate limits work
- [ ] **Error Handling** - Test invalid URLs and timeouts
- [ ] **Metrics** - Verify `/metrics` endpoint works
- [ ] **Redis Cache** - Test cache hit/miss (if Redis enabled)

---

## 📚 Additional Resources

- **API Documentation:** See [API.md](./API.md) for endpoint reference
- **Configuration Guide:** See [CONFIGURATION.md](./CONFIGURATION.md)
- **Deployment Guide:** See [DEPLOYMENT.md](./DEPLOYMENT.md)
- **Troubleshooting:** See test findings in `/tmp/riptide-realworld-test-findings.md`

---

## 🤝 Contributing Test Cases

When adding new features, please:

1. Add integration tests in `crates/riptide-api/tests/`
2. Update this guide with test examples
3. Add test scenarios to `scripts/quick-test.sh`
4. Verify tests pass in Docker environment

---

**Questions?** Open an issue with:
- Test environment (Docker/Cargo/CI)
- Configuration used (minimal.toml/.env/docker-compose)
- Error messages and logs
- Steps to reproduce
