# Testing Docker Deployment Modes

## Automated Testing

### Test Script
The `scripts/test-docker-modes.sh` script provides comprehensive testing for all deployment modes.

### Usage
```bash
# Test all modes
./scripts/test-docker-modes.sh all

# Test specific mode
./scripts/test-docker-modes.sh minimal
./scripts/test-docker-modes.sh simple
./scripts/test-docker-modes.sh distributed
```

### What It Tests

#### Minimal Mode
- ✅ Single container deployment
- ✅ Health endpoint responsiveness
- ✅ Static content extraction
- ✅ In-memory cache behavior
- ✅ Cache cleared on restart
- ✅ Memory usage (< 600MB)

#### Simple Mode
- ✅ Two container deployment (API + Redis)
- ✅ Redis connectivity
- ✅ Health endpoint responsiveness
- ✅ Static content extraction
- ✅ Cache persistence after restart
- ✅ Redis cache entries
- ✅ Memory usage (< 800MB)

#### Distributed Mode
- ✅ Multi-container deployment (API + Redis + Chrome)
- ✅ All services healthy
- ✅ Health endpoint responsiveness
- ✅ Static content extraction
- ✅ Cache persistence
- ✅ Browser service availability
- ✅ Memory usage (< 2000MB)

## Manual Testing

### Minimal Mode

```bash
# Start
docker-compose -f docker-compose.minimal.yml up -d

# Test health
curl http://localhost:8080/health

# Test extraction
curl "http://localhost:8080/extract?url=https://example.com"

# Test cache (second request should be faster)
time curl "http://localhost:8080/extract?url=https://example.com"

# Test cache clears on restart
docker-compose -f docker-compose.minimal.yml restart
time curl "http://localhost:8080/extract?url=https://example.com"  # Should be slow again

# Check memory
docker stats --no-stream riptide-minimal

# Stop
docker-compose -f docker-compose.minimal.yml down
```

### Simple Mode

```bash
# Start
docker-compose -f docker-compose.simple.yml up -d

# Test health
curl http://localhost:8080/health

# Test extraction
curl "http://localhost:8080/extract?url=https://example.com"

# Test cache persists
docker-compose -f docker-compose.simple.yml restart
time curl "http://localhost:8080/extract?url=https://example.com"  # Should be fast

# Check Redis
docker-compose -f docker-compose.simple.yml exec redis redis-cli DBSIZE
docker-compose -f docker-compose.simple.yml exec redis redis-cli INFO memory

# Check memory
docker stats --no-stream

# Stop
docker-compose -f docker-compose.simple.yml down
```

### Distributed Mode

```bash
# Start
docker-compose up -d

# Test health
curl http://localhost:8080/health

# Test static extraction
curl "http://localhost:8080/extract?url=https://example.com"

# Test JavaScript rendering (if Chrome is available)
curl -X POST http://localhost:8080/extract \
  -H "Content-Type: application/json" \
  -d '{"url": "https://react-app.com", "render": true}'

# Check services
docker-compose ps
docker-compose logs riptide-headless | tail -20

# Check memory
docker stats --no-stream

# Stop
docker-compose down
```

## CI/CD Integration

### GitHub Actions
The workflow file `.github/workflows/docker-modes-test.yml` automatically tests all modes on:
- Push to main/develop
- Pull requests
- Manual trigger

### Triggering Tests
```bash
# Via GitHub CLI
gh workflow run docker-modes-test.yml -f mode=all

# Or specific mode
gh workflow run docker-modes-test.yml -f mode=minimal
```

## Benchmark Testing

### Performance Comparison
```bash
# Test each mode's throughput
for mode in minimal simple distributed; do
    echo "Testing $mode mode..."
    docker-compose -f docker-compose.$mode.yml up -d 2>/dev/null || docker-compose up -d
    sleep 20
    
    # Run 100 requests
    ab -n 100 -c 10 "http://localhost:8080/extract?url=https://example.com"
    
    docker-compose -f docker-compose.$mode.yml down 2>/dev/null || docker-compose down
done
```

### Memory Profiling
```bash
# Start mode
docker-compose -f docker-compose.minimal.yml up -d

# Monitor memory over time
while true; do
    docker stats --no-stream --format "table {{.Name}}\t{{.MemUsage}}\t{{.MemPerc}}"
    sleep 5
done
```

## Load Testing

### Using Apache Bench
```bash
# Start service
docker-compose up -d

# Run load test
ab -n 1000 -c 50 "http://localhost:8080/extract?url=https://example.com"
```

### Using hey
```bash
# Install hey
go install github.com/rakyll/hey@latest

# Run load test
hey -n 1000 -c 50 "http://localhost:8080/extract?url=https://example.com"
```

## Expected Results

### Minimal Mode
```
Startup Time:     ~5 seconds
Memory Usage:     ~440MB
Cold Request:     500ms - 2s
Cache Hit:        1-5ms
Throughput:       ~30 req/min
```

### Simple Mode
```
Startup Time:     ~15 seconds
Memory Usage:     ~600MB
Cold Request:     500ms - 2s
Cache Hit:        10-20ms (Redis)
Throughput:       ~50 req/min
```

### Distributed Mode
```
Startup Time:     ~40 seconds
Memory Usage:     ~1.2GB
Cold Request:     500ms - 2s (static)
                  2s - 5s (JavaScript)
Cache Hit:        10-20ms (Redis)
Throughput:       ~200 req/min
```

## Troubleshooting Tests

### Test Failures

#### Health Check Timeout
```bash
# Check logs
docker-compose logs riptide-api

# Check if port is available
netstat -tuln | grep 8080

# Increase timeout in script
TIMEOUT=120 ./scripts/test-docker-modes.sh
```

#### Cache Test Failures
```bash
# Verify cache backend
docker-compose exec riptide-api env | grep CACHE

# For simple/distributed, check Redis
docker-compose exec redis redis-cli ping
docker-compose exec redis redis-cli KEYS '*'
```

#### Memory Test Failures
```bash
# Check actual memory usage
docker stats --no-stream

# Increase limits if needed
# Edit docker-compose.yml deploy.resources.limits.memory
```

## Continuous Monitoring

### Setup Monitoring
```bash
# Add Prometheus + Grafana
docker-compose -f docker-compose.yml -f docker-compose.monitoring.yml up -d

# Access metrics
curl http://localhost:8080/metrics
```

### Health Monitoring
```bash
# Setup periodic health checks
watch -n 5 'curl -s http://localhost:8080/health | jq'
```

---

**Last Updated**: 2025-11-12
