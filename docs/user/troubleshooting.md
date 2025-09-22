# Troubleshooting Guide

This guide helps you diagnose and resolve common issues with RipTide Crawler.

## Quick Diagnostics

### Health Check
```bash
# Check all services
curl http://localhost:8080/health

# Check specific components
curl http://localhost:8080/health?component=redis
curl http://localhost:8080/health?component=headless
curl http://localhost:8080/health?component=wasm
```

### Service Status
```bash
# Check if services are running
docker-compose ps

# Check logs
docker-compose logs api
docker-compose logs headless
docker-compose logs redis
```

### Configuration Validation
```bash
# Validate configuration file
riptide-api --config riptide.yml --check-config

# Test configuration
riptide-api --config riptide.yml --dry-run
```

## Common Issues

### 1. Service Startup Problems

#### Issue: API service won't start
```
Error: Failed to bind to 0.0.0.0:8080: address already in use
```

**Solutions:**
```bash
# Check what's using the port
lsof -i :8080
netstat -tlnp | grep :8080

# Kill the process or change port
export RIPTIDE_API_PORT=8081

# Or stop conflicting service
sudo systemctl stop apache2  # if Apache is using port 8080
```

#### Issue: Redis connection failed
```
Error: Redis connection refused (redis://localhost:6379)
```

**Solutions:**
```bash
# Check Redis status
redis-cli ping
systemctl status redis

# Start Redis if stopped
systemctl start redis
docker run -d -p 6379:6379 redis:7

# Check Redis configuration
redis-cli CONFIG GET bind
redis-cli CONFIG GET protected-mode
```

#### Issue: WASM module not found
```
Error: Failed to load WASM module at /opt/riptide/extractor.wasm
```

**Solutions:**
```bash
# Check file exists
ls -la /opt/riptide/extractor.wasm

# Build WASM module
cd wasm/riptide-extractor-wasm
cargo build --release --target wasm32-wasip2

# Copy to correct location
cp target/wasm32-wasip2/release/riptide-extractor-wasm.wasm /opt/riptide/extractor.wasm

# Update configuration
export RIPTIDE_EXTRACTION_WASM_MODULE_PATH="./target/wasm32-wasip2/release/riptide-extractor-wasm.wasm"
```

### 2. Performance Issues

#### Issue: Slow crawling performance
```
Crawling 100 URLs takes over 10 minutes
```

**Diagnosis:**
```bash
# Check current configuration
curl http://localhost:8080/health | jq '.config.crawl'

# Monitor resource usage
htop
iotop
```

**Solutions:**
```yaml
# Increase concurrency in riptide.yml
crawl:
  concurrency: 32  # Increase from default 16
  timeout_ms: 10000  # Reduce timeout

# Disable headless fallback for speed
dynamic:
  enable_headless_fallback: false

# Use faster cache mode
crawl:
  cache: "read_through"  # Instead of "bypass"
```

#### Issue: High memory usage
```
Process using 4GB+ RAM, system running out of memory
```

**Diagnosis:**
```bash
# Check memory usage
ps aux | grep riptide
free -h

# Check for memory leaks
valgrind --tool=memcheck --leak-check=yes riptide-api
```

**Solutions:**
```yaml
# Reduce concurrency
crawl:
  concurrency: 8
  max_response_mb: 5  # Reduce from 20MB

# Limit extraction resources
extraction:
  max_input_bytes: 5242880  # 5MB limit
  timeout_seconds: 15

# Reduce headless sessions
dynamic:
  max_concurrent_sessions: 1
```

#### Issue: CPU usage at 100%
```
High CPU load, system becomes unresponsive
```

**Solutions:**
```yaml
# Reduce processing intensity
crawl:
  concurrency: 4
  default_delay_ms: 1000  # Add delays

extraction:
  mode: "article"  # Instead of "full"
  timeout_seconds: 10

# Disable expensive features
dynamic:
  enable_headless_fallback: false
stealth:
  enabled: false
```

### 3. Extraction Issues

#### Issue: No content extracted
```json
{
  "content": {
    "title": null,
    "text": "",
    "markdown": ""
  }
}
```

**Diagnosis:**
```bash
# Test WASM extractor directly
echo "<html><body><h1>Test</h1><p>Content</p></body></html>" | \
  wasmtime run --env RIPTIDE_URL=test.com \
  target/wasm32-wasip2/release/riptide-extractor-wasm.wasm

# Check URL accessibility
curl -v https://problematic-url.com

# Test with different extraction modes
curl -X POST http://localhost:8080/crawl \
  -d '{"urls": ["https://site.com"], "options": {"extract_mode": "full"}}'
```

**Solutions:**
```bash
# Force headless mode for dynamic content
curl -X POST http://localhost:8080/crawl \
  -d '{"urls": ["https://spa-site.com"], "options": {"force_headless": true}}'

# Increase extraction timeout
curl -X POST http://localhost:8080/crawl \
  -d '{"urls": ["https://slow-site.com"], "options": {"timeout_seconds": 60}}'

# Try different wait conditions
curl -X POST http://localhost:8080/crawl \
  -d '{"urls": ["https://dynamic-site.com"], "options": {"dynamic_wait_for": ".main-content"}}'
```

#### Issue: Partial content extraction
```
Only extracting page headers, missing main content
```

**Solutions:**
```yaml
# Adjust extraction quality thresholds
extraction:
  quality_thresholds:
    min_text_ratio: 0.05  # Lower threshold
    min_paragraph_count: 1  # More lenient

# Enable more aggressive extraction
extraction:
  mode: "full"
  remove_elements: []  # Don't remove any elements
```

#### Issue: Incorrect language detection
```
Content detected as wrong language, affecting extraction
```

**Solutions:**
```yaml
# Override language detection
extraction:
  force_language: "en"  # Force English
  language_detection: false

# Use different tokenizer
extraction:
  tokenizer: "unicode"  # Instead of "tiktoken"
```

### 4. Network and Connectivity Issues

#### Issue: Timeouts on specific sites
```
Requests timing out for certain domains
```

**Diagnosis:**
```bash
# Test direct connectivity
curl -v --max-time 30 https://problematic-site.com
traceroute problematic-site.com
nslookup problematic-site.com
```

**Solutions:**
```yaml
# Increase timeouts for slow sites
crawl:
  timeout_ms: 60000  # 60 seconds
  connect_timeout_ms: 10000  # 10 seconds

# Per-domain configuration
crawl:
  domain_delays:
    "slow-site.com": 5000  # 5 second delay
  domain_timeouts:
    "slow-site.com": 120000  # 2 minute timeout
```

#### Issue: Rate limiting / 429 errors
```
Getting "Too Many Requests" responses
```

**Solutions:**
```yaml
# Reduce request rate
crawl:
  concurrency: 2
  default_delay_ms: 3000
  rate_limiting: true

# Implement exponential backoff
crawl:
  retry_config:
    max_attempts: 3
    base_delay_ms: 1000
    exponential_base: 2
```

#### Issue: IP blocking / access denied
```
403 Forbidden or connection refused errors
```

**Solutions:**
```yaml
# Enable stealth mode
stealth:
  enabled: true
  random_ua: true
  randomize_viewport: true

# Use proxies
proxies:
  enabled: true
  rotation_strategy: "random"
  proxy_list:
    - "http://proxy1.example.com:8080"
    - "http://proxy2.example.com:8080"

# Rotate user agents more aggressively
crawl:
  user_agent_mode: "random"
```

### 5. Docker and Container Issues

#### Issue: Container won't start
```
docker-compose up fails with various errors
```

**Diagnosis:**
```bash
# Check Docker status
docker version
docker-compose version

# Check logs
docker-compose logs api
docker-compose logs headless

# Check resource usage
docker stats
```

**Solutions:**
```bash
# Increase Docker memory limits
# In docker-compose.yml:
services:
  api:
    mem_limit: 2g
  headless:
    mem_limit: 4g

# Clean up Docker resources
docker system prune -f
docker volume prune -f

# Rebuild containers
docker-compose down
docker-compose build --no-cache
docker-compose up -d
```

#### Issue: Chrome/Chromium crashes in headless service
```
Headless service crashes with "Chrome crashed" errors
```

**Solutions:**
```bash
# Add Chrome stability flags
# In docker-compose.yml:
environment:
  - CHROME_FLAGS=--no-sandbox --disable-dev-shm-usage --disable-gpu

# Increase shared memory
services:
  headless:
    shm_size: 2gb

# Or use tmpfs
tmpfs:
  - /tmp
  - /dev/shm
```

#### Issue: Volume mount permissions
```
Permission denied errors accessing mounted volumes
```

**Solutions:**
```bash
# Fix ownership
sudo chown -R 1000:1000 ./data
sudo chmod -R 755 ./data

# Use correct user in container
# In Dockerfile:
USER 1000:1000

# Or run with specific user
docker-compose run --user 1000:1000 api
```

### 6. API and Integration Issues

#### Issue: JSON parsing errors
```
400 Bad Request: Invalid JSON in request body
```

**Solutions:**
```bash
# Validate JSON syntax
echo '{"urls": ["https://example.com"]}' | jq '.'

# Check Content-Type header
curl -X POST http://localhost:8080/crawl \
  -H "Content-Type: application/json" \
  -d '{"urls": ["https://example.com"]}'

# Use proper escaping
curl -X POST http://localhost:8080/crawl \
  -H "Content-Type: application/json" \
  -d "{\"urls\": [\"https://example.com\"]}"
```

#### Issue: Authentication failures
```
401 Unauthorized responses
```

**Solutions:**
```bash
# Check API key format
curl -H "Authorization: Bearer your-api-key" \
  http://localhost:8080/crawl

# Verify key in configuration
grep -r "api_key" configs/

# Test without authentication
# Temporarily disable auth in config
api:
  auth:
    enabled: false
```

#### Issue: CORS errors in browser
```
CORS policy blocks requests from web apps
```

**Solutions:**
```yaml
# Configure CORS in riptide.yml
api:
  cors:
    enabled: true
    origins:
      - "http://localhost:3000"  # Development
      - "https://your-app.com"   # Production
    methods: ["GET", "POST", "OPTIONS"]
    headers: ["Content-Type", "Authorization"]
```

### 7. Configuration Issues

#### Issue: Environment variables not working
```
Configuration not updating despite setting env vars
```

**Solutions:**
```bash
# Check variable names (must be uppercase)
export RIPTIDE_CRAWL_CONCURRENCY=32

# Verify variables are loaded
env | grep RIPTIDE

# Check .env file format
cat .env
# Should be: RIPTIDE_CRAWL_CONCURRENCY=32 (no spaces around =)

# Restart services after changing variables
docker-compose restart
```

#### Issue: Configuration file not found
```
Error: Could not load configuration file riptide.yml
```

**Solutions:**
```bash
# Check file exists
ls -la riptide.yml

# Use absolute path
riptide-api --config /full/path/to/riptide.yml

# Set config environment variable
export RIPTIDE_CONFIG_FILE=/path/to/riptide.yml

# Copy example configuration
cp configs/riptide.yml.example riptide.yml
```

#### Issue: Invalid YAML syntax
```
Error: Failed to parse YAML configuration
```

**Solutions:**
```bash
# Validate YAML syntax
python -c "import yaml; yaml.safe_load(open('riptide.yml'))"

# Or use yamllint
yamllint riptide.yml

# Check indentation (use spaces, not tabs)
cat -A riptide.yml  # Shows hidden characters
```

## Monitoring and Logging

### Enable Debug Logging
```bash
# Set debug log level
export RUST_LOG=debug
export RIPTIDE_LOGGING_LEVEL=debug

# Or in configuration
logging:
  level: "debug"
  format: "text"  # More readable than JSON
```

### Structured Logging Analysis
```bash
# Filter logs by component
docker-compose logs api | grep "extraction"
docker-compose logs api | grep "ERROR"

# Follow logs in real-time
docker-compose logs -f api

# Search for specific patterns
journalctl -u riptide-api | grep "timeout"
```

### Performance Monitoring
```bash
# Monitor API response times
curl -w "@curl-format.txt" -o /dev/null -s http://localhost:8080/health

# curl-format.txt:
#     time_namelookup:  %{time_namelookup}\n
#        time_connect:  %{time_connect}\n
#     time_appconnect:  %{time_appconnect}\n
#    time_pretransfer:  %{time_pretransfer}\n
#       time_redirect:  %{time_redirect}\n
#  time_starttransfer:  %{time_starttransfer}\n
#                     ----------\n
#          time_total:  %{time_total}\n

# Monitor resource usage
docker stats --format "table {{.Container}}\t{{.CPUPerc}}\t{{.MemUsage}}"
```

## Getting Help

### Collect Diagnostic Information
```bash
#!/bin/bash
# diagnostic_info.sh

echo "=== System Information ==="
uname -a
docker version
docker-compose version

echo "=== Service Status ==="
docker-compose ps
curl -s http://localhost:8080/health | jq '.'

echo "=== Resource Usage ==="
free -h
df -h
docker stats --no-stream

echo "=== Recent Logs ==="
docker-compose logs --tail=50 api
docker-compose logs --tail=50 headless

echo "=== Configuration ==="
cat riptide.yml | head -50
env | grep RIPTIDE
```

### Performance Benchmarking
```bash
# Simple performance test
time curl -X POST http://localhost:8080/crawl \
  -H "Content-Type: application/json" \
  -d '{"urls": ["https://httpbin.org/html"]}'

# Load testing with curl
seq 1 10 | xargs -I {} -P 5 curl -X POST http://localhost:8080/crawl \
  -H "Content-Type: application/json" \
  -d '{"urls": ["https://httpbin.org/delay/1"]}'

# Memory leak testing
for i in {1..100}; do
  curl -X POST http://localhost:8080/crawl \
    -H "Content-Type: application/json" \
    -d '{"urls": ["https://httpbin.org/html"]}'
  echo "Request $i completed"
  sleep 1
done
```

### Report Issues
When reporting issues, include:

1. **System Information**: OS, Docker version, available resources
2. **Configuration**: Sanitized configuration file and environment variables
3. **Error Messages**: Complete error messages and stack traces
4. **Reproduction Steps**: Minimal example to reproduce the issue
5. **Expected vs Actual Behavior**: What you expected vs what happened
6. **Logs**: Relevant log entries with debug level enabled

### Community Resources
- **GitHub Issues**: Report bugs and feature requests
- **GitHub Discussions**: Ask questions and share solutions
- **Documentation**: Check the complete docs in the `/docs` directory
- **Examples**: Look at working examples in `/examples` directory

This troubleshooting guide covers the most common issues. For specific problems not covered here, enable debug logging and check the detailed error messages for more specific guidance.