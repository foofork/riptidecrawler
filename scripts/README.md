# RipTide Phase 1 Load Testing

This directory contains comprehensive load testing tools for validating RipTide Phase 1 performance requirements.

## 🎯 Success Criteria

The load testing validates the following Phase 1 requirements:
- **P95 response time < 2000ms** for 100 concurrent requests
- **Error rate < 1%** under load
- **Successful handling** of 100 concurrent users

## 📁 Files

- `load-test.sh` - Main load testing script
- `test-data/` - Generated test data files
- `load-test-results/` - Test results and reports

## 🚀 Quick Start

### Prerequisites

Install required dependencies:

```bash
# Ubuntu/Debian
sudo apt-get install curl jq bc

# macOS
brew install curl jq bc

# Install hey for better load testing (recommended)
go install github.com/rakyll/hey@latest
```

### Basic Usage

```bash
# Run complete load test with defaults
./load-test.sh

# Test specific endpoint
./load-test.sh test-health
./load-test.sh test-crawl

# Generate test data
./load-test.sh generate-data

# Generate performance report
./load-test.sh report
```

### Advanced Usage

```bash
# Custom configuration
./load-test.sh --host http://127.0.0.1:8080 --concurrent 50 --duration 30

# Start/stop server automatically
./load-test.sh --start-server --stop-server test-all

# Verbose output
./load-test.sh --verbose test-all
```

## 🔧 Configuration

### Environment Variables

```bash
export RIPTIDE_HOST="http://localhost:8080"
export RIPTIDE_CONCURRENT="100"
export RIPTIDE_REQUESTS="1000"
export RIPTIDE_DURATION="60"
```

### Command Line Options

```bash
Options:
    -h, --host HOST     Target host (default: http://localhost:8080)
    -c, --concurrent N  Number of concurrent requests (default: 100)
    -n, --requests N    Total number of requests (default: 1000)
    -d, --duration N    Test duration in seconds (default: 60)
    -v, --verbose       Enable verbose output
    --no-cleanup        Don't cleanup temp files
    --start-server      Start RipTide server before testing
    --stop-server       Stop RipTide server after testing
```

## 🧪 Test Scenarios

### 1. Health Check Load Test

Tests the `/healthz` endpoint under load:

```bash
./load-test.sh test-health
```

**What it tests:**
- Server responsiveness under concurrent load
- Health check latency
- System stability

### 2. Crawl Endpoint Load Test

Tests the `/crawl` endpoint with realistic data:

```bash
./load-test.sh test-crawl
```

**What it tests:**
- Crawling pipeline performance
- Concurrent URL processing
- Memory and resource usage
- Cache effectiveness

### 3. Comprehensive Test Suite

Runs all tests and generates a detailed report:

```bash
./load-test.sh test-all
```

## 📊 Test Data

The script generates three types of test data:

### Small Batch (`small-batch.json`)
```json
{
  "urls": [
    "https://httpbin.org/json",
    "https://httpbin.org/html",
    "https://example.com"
  ],
  "options": {
    "concurrency": 3,
    "cache_mode": "force_fresh"
  }
}
```

### Large Batch (`large-batch.json`)
- 50 diverse URLs for stress testing
- Mix of static and dynamic content
- Various response types (HTML, JSON, XML)

### Test URLs (`test-urls.json`)
- 20 carefully selected URLs
- Includes delay endpoints for timeout testing
- Mix of fast and slow responding sites

## 📈 Performance Metrics

The load test measures and reports:

### Response Time Percentiles
- **P50**: Median response time
- **P75**: 75th percentile
- **P90**: 90th percentile
- **P95**: 95th percentile (success criteria: <2000ms)
- **P99**: 99th percentile

### Throughput Metrics
- **Requests per second**
- **Total requests processed**
- **Successful vs failed requests**
- **Error rate percentage**

### System Metrics
- **Concurrent user handling**
- **Cache hit rates**
- **Processing time breakdown**

## 📋 Example Output

```bash
$ ./load-test.sh test-all

🔍 Checking dependencies...
✅ All dependencies satisfied

🔍 Checking server health...
✅ Server is healthy (status: healthy)

🔄 Generating test data...
✅ Test data generated in ./test-data

⚡ Running load test: health
  Endpoint: http://localhost:8080/healthz
  Concurrent: 100
  Total Requests: 1000
✅ Load test completed: health

Health endpoint results:
  P95 response time: 45.2ms
  Error rate: 0.0%
✅ P95 response time meets criteria (<2000ms)
✅ Error rate meets criteria (<1%)

⚡ Running load test: crawl
  Endpoint: http://localhost:8080/crawl
  Concurrent: 100
  Total Requests: 1000
✅ Load test completed: crawl

Crawl endpoint results:
  P95 response time: 1250.5ms
  Error rate: 0.2%
✅ P95 response time meets criteria (<2000ms)
✅ Error rate meets criteria (<1%)

📊 Generating performance report...
✅ Performance report generated: ./load-test-results/performance-report-20240322-143022.md
```

## 📊 Performance Reports

The script generates detailed markdown reports including:

- **Executive Summary** with pass/fail criteria
- **Detailed Metrics** for each endpoint
- **Response Time Analysis** with percentile breakdown
- **Error Analysis** and recommendations
- **System Resource Usage**
- **Recommendations** for optimization

### Sample Report Structure

```markdown
# RipTide Phase 1 Load Testing Report

**Generated:** 2024-03-22T14:30:22Z
**Test Configuration:**
- Host: http://localhost:8080
- Concurrent Users: 100
- Total Requests: 1000
- Test Duration: 60s

## Success Criteria
- ✅ P95 response time < 2000ms
- ✅ Error rate < 1%
- ✅ 100 concurrent requests handled successfully

## Test Results

### Health Endpoint (/healthz)
**Metrics:**
- Total Requests: 1000
- Successful Requests: 1000
- Error Rate: 0.0%
- P95 Response Time: 45.2ms ✅

### Crawl Endpoint (/crawl)
**Metrics:**
- Total Requests: 1000
- Successful Requests: 998
- Error Rate: 0.2%
- P95 Response Time: 1250.5ms ✅
```

## 🔧 Troubleshooting

### Common Issues

1. **Server not responding**
   ```bash
   # Check if server is running
   curl http://localhost:8080/healthz

   # Start server automatically
   ./load-test.sh --start-server test-all
   ```

2. **Hey tool not found**
   ```bash
   # Install hey for better performance
   go install github.com/rakyll/hey@latest

   # Or use curl fallback (automatic)
   ./load-test.sh --verbose test-all
   ```

3. **Permission denied**
   ```bash
   # Make script executable
   chmod +x load-test.sh
   ```

4. **Dependencies missing**
   ```bash
   # Install on Ubuntu/Debian
   sudo apt-get install curl jq bc

   # Install on macOS
   brew install curl jq bc
   ```

### Debug Mode

Enable verbose output for troubleshooting:

```bash
./load-test.sh --verbose --no-cleanup test-all
```

This will:
- Show detailed progress information
- Keep temporary files for inspection
- Display raw test output

## 🎯 Integration with CI/CD

### GitHub Actions Example

```yaml
name: RipTide Load Test

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

jobs:
  load-test:
    runs-on: ubuntu-latest

    steps:
    - uses: actions/checkout@v3

    - name: Install dependencies
      run: |
        sudo apt-get update
        sudo apt-get install -y curl jq bc
        go install github.com/rakyll/hey@latest

    - name: Build RipTide
      run: cargo build --release --bin riptide-api

    - name: Run Load Tests
      run: |
        cd scripts
        ./load-test.sh --start-server --stop-server test-all

    - name: Upload Test Results
      uses: actions/upload-artifact@v3
      with:
        name: load-test-results
        path: scripts/load-test-results/
```

### Jenkins Pipeline Example

```groovy
pipeline {
    agent any

    stages {
        stage('Build') {
            steps {
                sh 'cargo build --release --bin riptide-api'
            }
        }

        stage('Load Test') {
            steps {
                sh '''
                cd scripts
                ./load-test.sh --start-server --stop-server test-all
                '''
            }

            post {
                always {
                    archiveArtifacts artifacts: 'scripts/load-test-results/**/*', fingerprint: true
                    publishHTML([
                        allowMissing: false,
                        alwaysLinkToLastBuild: true,
                        keepAll: true,
                        reportDir: 'scripts/load-test-results',
                        reportFiles: '*.md',
                        reportName: 'Load Test Report'
                    ])
                }
            }
        }
    }
}
```

## 🔍 Performance Optimization

### Tips for Meeting Success Criteria

1. **Database Optimization**
   - Ensure Redis is properly configured
   - Monitor Redis memory usage
   - Use appropriate cache TTL settings

2. **Application Tuning**
   - Adjust `max_concurrency` in config
   - Tune timeout settings
   - Monitor heap memory usage

3. **Infrastructure**
   - Ensure adequate CPU/memory resources
   - Use SSD storage for better I/O
   - Configure appropriate file descriptor limits

4. **Network**
   - Test on same network as production
   - Account for network latency in measurements
   - Use connection pooling

## 📚 Additional Resources

- [RipTide Architecture Documentation](../docs/architecture.md)
- [Performance Tuning Guide](../docs/performance.md)
- [Monitoring and Observability](../docs/monitoring.md)
- [Production Deployment Guide](../docs/deployment.md)

## 🤝 Contributing

To add new test scenarios or improve the load testing suite:

1. Fork the repository
2. Create a feature branch
3. Add your test scenarios to `load-test.sh`
4. Update this documentation
5. Submit a pull request

## 📄 License

This load testing suite is part of the RipTide project and follows the same license terms.