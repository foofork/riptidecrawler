# API Tooling Quick Start Guide

Your RipTide API now has **complete OpenAPI 3.0 documentation** with 100% endpoint coverage (59/59 endpoints). Here's how to use all the powerful tools that work with it.

---

## 🎨 1. Interactive API Documentation (Swagger UI)

### Option A: Docker (Fastest)

```bash
# Start Swagger UI
docker-compose -f docker-compose.swagger.yml up -d swagger-ui

# Access at: http://localhost:8081
```

**What you get:**
- 📖 Beautiful, interactive API docs
- 🧪 Test APIs directly in browser (no Postman!)
- 📝 See request/response examples
- 🔍 Search and filter endpoints

### Option B: Local HTML File

```bash
# Serve static files
cd docs/api
python -m http.server 8081

# Access at: http://localhost:8081/swagger.html
```

### Option C: Alternative - ReDoc (Cleaner UI)

```bash
# Start ReDoc
docker-compose -f docker-compose.swagger.yml up -d redoc

# Access at: http://localhost:8082
```

---

## 🧪 2. Automated API Testing

### A. Contract Testing with Dredd

Tests that your API matches the OpenAPI spec:

```bash
# Install Dredd
npm install -g dredd

# Start your API
cargo run --package riptide-api &

# Run contract tests
dredd docs/api/openapi.yaml http://localhost:8080
```

**What it tests:**
- ✅ Response status codes match spec
- ✅ Response schemas match spec
- ✅ Required fields are present
- ✅ Data types are correct

### B. Fuzzing with Schemathesis

Automatically generates 1000s of test cases:

```bash
# Install Schemathesis
pip install schemathesis

# Run fuzzing tests
schemathesis run docs/api/openapi.yaml \
  --base-url http://localhost:8080 \
  --checks all
```

**What it finds:**
- 🐛 Crashes and errors
- 🔒 Security vulnerabilities
- ⚠️ Validation issues
- 🎯 Edge cases you didn't think of

### C. Load Testing with k6

```bash
# Install k6
brew install k6  # macOS
# OR
curl https://github.com/grafana/k6/releases/download/v0.47.0/k6-v0.47.0-linux-amd64.tar.gz -L | tar xvz

# Run performance tests
k6 run - <<'EOF'
import http from 'k6/http';
import { check } from 'k6';

export let options = {
  vus: 10,
  duration: '30s',
};

export default function() {
  let res = http.get('http://localhost:8080/healthz');
  check(res, {
    'status is 200': (r) => r.status === 200,
  });
}
EOF
```

### D. CI/CD Integration

GitHub Actions workflow already created:

```bash
# File: .github/workflows/api-contract-tests.yml

# Runs automatically on:
# - Push to main/develop
# - Pull requests
# - Changes to API code or OpenAPI spec
```

**Tests run:**
1. ✅ Dredd contract tests
2. ✅ Schemathesis fuzzing
3. ✅ OpenAPI validation
4. ✅ Performance tests
5. ✅ Security scanning (OWASP ZAP)

---

## 🌐 3. API Gateway Integration

### Kong Gateway (Recommended)

Adds rate limiting, authentication, analytics:

```bash
# Start Kong Gateway + RipTide
docker-compose -f docker-compose.gateway.yml up -d

# Configure Kong to proxy RipTide API
curl -X POST http://localhost:8001/services \
  --data name=riptide-api \
  --data url=http://riptide-api:8080

curl -X POST http://localhost:8001/services/riptide-api/routes \
  --data paths[]=/api

# Add rate limiting (100 requests/minute)
curl -X POST http://localhost:8001/services/riptide-api/plugins \
  --data "name=rate-limiting" \
  --data "config.minute=100"

# Add API key authentication
curl -X POST http://localhost:8001/services/riptide-api/plugins \
  --data "name=key-auth"
```

**Access points:**
- 🚪 API Gateway: `http://localhost:8000/api`
- 🔧 Admin API: `http://localhost:8001`
- 📊 Kong Manager: `http://localhost:8002`
- 📚 Swagger UI: `http://localhost:8081`

**What you get:**
- ⏱️ Rate limiting per user/IP
- 🔐 API key authentication
- 📈 Analytics & monitoring
- 💾 Response caching
- ⚖️ Load balancing
- 🔄 API versioning

---

## 🚀 4. Generate Client SDKs

### TypeScript/JavaScript Client

```bash
# Install generator
npm install -g @openapitools/openapi-generator-cli

# Generate TypeScript client
openapi-generator-cli generate \
  -i docs/api/openapi.yaml \
  -g typescript-axios \
  -o clients/typescript

# Use in your code
import { DefaultApi } from './clients/typescript';

const api = new DefaultApi({ basePath: 'http://localhost:8080' });
const response = await api.healthCheck();
```

### Python Client

```bash
# Generate Python client
openapi-generator-cli generate \
  -i docs/api/openapi.yaml \
  -g python \
  -o clients/python

# Use in your code
from clients.python import DefaultApi, Configuration

config = Configuration(host='http://localhost:8080')
api = DefaultApi(config)
response = api.health_check()
```

### Rust Client

```bash
# Generate Rust client
openapi-generator-cli generate \
  -i docs/api/openapi.yaml \
  -g rust \
  -o clients/rust
```

### Go Client

```bash
# Generate Go client
openapi-generator-cli generate \
  -i docs/api/openapi.yaml \
  -g go \
  -o clients/go
```

---

## 📊 5. Import to Postman/Insomnia

### Postman

1. Open Postman
2. Click **Import**
3. Select `docs/api/openapi.yaml`
4. Auto-generates 59 requests organized by category

**OR via CLI:**

```bash
# Convert to Postman collection
npm install -g openapi-to-postmanv2

openapi2postmanv2 \
  -s docs/api/openapi.yaml \
  -o riptide-api.postman.json

# Import to Postman
# File → Import → riptide-api.postman.json
```

### Insomnia

1. Open Insomnia
2. Click **Import/Export**
3. Select `docs/api/openapi.yaml`
4. Creates full collection with environments

---

## 🔍 6. API Linting & Validation

### Validate OpenAPI Spec

```bash
# Install validator
npm install -g @apidevtools/swagger-cli

# Validate spec
swagger-cli validate docs/api/openapi.yaml
```

### Lint for Best Practices

```bash
# Install Spectral
npm install -g @stoplight/spectral-cli

# Lint OpenAPI spec
spectral lint docs/api/openapi.yaml
```

### Check Breaking Changes

```bash
# Install oasdiff
npm install -g oasdiff

# Compare versions
oasdiff changelog \
  docs/api/openapi.yaml.old \
  docs/api/openapi.yaml
```

---

## 📈 7. API Monitoring & Analytics

### With Kong Gateway

```bash
# View metrics
curl http://localhost:8001/status

# Export to Prometheus
curl http://localhost:8001/metrics
```

### With Prometheus + Grafana

```yaml
# prometheus.yml
scrape_configs:
  - job_name: 'riptide-api'
    static_configs:
      - targets: ['localhost:8080']
    metrics_path: '/metrics'

  - job_name: 'kong'
    static_configs:
      - targets: ['localhost:8001']
    metrics_path: '/metrics'
```

---

## 🎯 Quick Command Reference

```bash
# 1. Start Swagger UI
docker-compose -f docker-compose.swagger.yml up -d swagger-ui
# → http://localhost:8081

# 2. Run contract tests
dredd docs/api/openapi.yaml http://localhost:8080

# 3. Fuzz testing
schemathesis run docs/api/openapi.yaml --base-url http://localhost:8080

# 4. Start Kong Gateway
docker-compose -f docker-compose.gateway.yml up -d
# → http://localhost:8000 (gateway)
# → http://localhost:8002 (dashboard)

# 5. Generate TypeScript client
openapi-generator-cli generate -i docs/api/openapi.yaml -g typescript-axios -o clients/typescript

# 6. Validate spec
swagger-cli validate docs/api/openapi.yaml

# 7. Import to Postman
openapi2postmanv2 -s docs/api/openapi.yaml -o riptide.postman.json
```

---

## 🔗 Useful Links

- **Swagger UI**: https://swagger.io/tools/swagger-ui/
- **ReDoc**: https://redocly.com/redoc/
- **Dredd**: https://dredd.org/
- **Schemathesis**: https://schemathesis.readthedocs.io/
- **Kong Gateway**: https://konghq.com/
- **OpenAPI Generator**: https://openapi-generator.tech/
- **Spectral Linter**: https://stoplight.io/open-source/spectral

---

## 🎉 What You Have Now

✅ **100% API Documentation** (59/59 endpoints)
✅ **Interactive API Explorer** (Swagger UI)
✅ **Automated Contract Testing** (Dredd)
✅ **Fuzzing & Security Tests** (Schemathesis)
✅ **API Gateway Ready** (Kong, Tyk, AWS)
✅ **Client SDK Generation** (TypeScript, Python, Rust, Go)
✅ **CI/CD Integration** (GitHub Actions)
✅ **Performance Testing** (k6)
✅ **API Monitoring** (Prometheus/Grafana)

Your RipTide API is now enterprise-ready with professional tooling! 🚀
