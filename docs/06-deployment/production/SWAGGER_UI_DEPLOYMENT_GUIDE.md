# Swagger UI Deployment Guide

## What is Swagger UI?

Swagger UI is an interactive API documentation tool that automatically generates a web interface from your OpenAPI specification. It allows developers to:

- 📖 **Browse API endpoints** with detailed descriptions
- 🧪 **Test APIs directly** in the browser (no Postman needed!)
- 📝 **See request/response examples** for each endpoint
- 🔐 **Test authentication** with API keys
- 📊 **View schemas** and data models

## Live Demo Preview

Your RipTide API would look like this:
```
┌─────────────────────────────────────────────────────┐
│  RipTide API - Comprehensive Specification v1.0.0  │
│  59 endpoints across 12 categories                  │
├─────────────────────────────────────────────────────┤
│                                                      │
│  📁 Health (2 endpoints)                            │
│    GET  /healthz          System health check       │
│    GET  /metrics          Prometheus metrics        │
│                                                      │
│  📁 Crawling (5 endpoints)                          │
│    POST /crawl            Batch crawl URLs     [Try]│
│    POST /crawl/stream     Stream results       [Try]│
│    ...                                               │
│                                                      │
│  📁 Sessions (12 endpoints)                         │
│    POST   /sessions       Create session       [Try]│
│    GET    /sessions       List sessions        [Try]│
│    DELETE /sessions/{id}  Delete session       [Try]│
│    ...                                               │
└─────────────────────────────────────────────────────┘
```

When you click **[Try]**, you get:
- Input fields for parameters
- Request body editor
- Execute button
- Live response with status code, headers, body

---

## Option 1: Docker Deployment (Recommended)

### Quick Start - Docker Compose

Create `docker-compose.swagger.yml`:

```yaml
version: '3.8'

services:
  swagger-ui:
    image: swaggerapi/swagger-ui:latest
    container_name: riptide-swagger-ui
    ports:
      - "8081:8080"
    environment:
      # Point to your OpenAPI spec
      SWAGGER_JSON_URL: http://localhost:8080/docs/api/openapi.yaml
      # OR use local file
      SWAGGER_JSON: /openapi.yaml
    volumes:
      # Mount your OpenAPI spec
      - ./docs/api/openapi.yaml:/openapi.yaml:ro
    networks:
      - riptide-network

  # Your RipTide API (for CORS support)
  riptide-api:
    build: .
    ports:
      - "8080:8080"
    # Serve OpenAPI spec as static file
    volumes:
      - ./docs/api:/usr/local/share/docs/api:ro
    networks:
      - riptide-network

networks:
  riptide-network:
    driver: bridge
```

**Start Swagger UI:**
```bash
docker-compose -f docker-compose.swagger.yml up -d

# Access at: http://localhost:8081
```

### Standalone Docker

```bash
# Quick one-liner
docker run -p 8081:8080 \
  -e SWAGGER_JSON=/openapi.yaml \
  -v $(pwd)/docs/api/openapi.yaml:/openapi.yaml \
  swaggerapi/swagger-ui

# Access at: http://localhost:8081
```

---

## Option 2: Add to RipTide API (Integrated)

### Add Swagger UI endpoint to your API

**File: `crates/riptide-api/src/routes/docs.rs`** (new file)

```rust
use axum::{
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use std::sync::Arc;
use crate::state::AppState;

pub fn docs_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/docs", get(swagger_ui))
        .route("/docs/openapi.yaml", get(serve_openapi_spec))
}

async fn swagger_ui() -> impl IntoResponse {
    Html(r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>RipTide API Documentation</title>
    <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/swagger-ui-dist@5/swagger-ui.css">
    <style>
        body { margin: 0; padding: 0; }
    </style>
</head>
<body>
    <div id="swagger-ui"></div>
    <script src="https://cdn.jsdelivr.net/npm/swagger-ui-dist@5/swagger-ui-bundle.js"></script>
    <script>
        window.onload = () => {
            window.ui = SwaggerUIBundle({
                url: '/docs/openapi.yaml',
                dom_id: '#swagger-ui',
                deepLinking: true,
                presets: [
                    SwaggerUIBundle.presets.apis,
                    SwaggerUIBundle.SwaggerUIStandalonePreset
                ],
                layout: "BaseLayout",
                defaultModelsExpandDepth: 1,
                defaultModelExpandDepth: 1,
                docExpansion: 'list',
                filter: true,
                tryItOutEnabled: true
            });
        };
    </script>
</body>
</html>
    "#)
}

async fn serve_openapi_spec() -> impl IntoResponse {
    let openapi_content = include_str!("../../../docs/api/openapi.yaml");
    (
        axum::http::StatusCode::OK,
        [("content-type", "application/yaml")],
        openapi_content,
    )
}
```

**Update `main.rs`:**
```rust
mod routes;

// In main():
let app = Router::new()
    // ... existing routes
    .nest("/", routes::docs::docs_routes())
    // ... rest of the app
```

**Access at:** `http://localhost:8080/docs`

---

## Option 3: Static HTML File

Create `docs/api/index.html`:

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>RipTide API Documentation</title>
    <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/swagger-ui-dist@5/swagger-ui.css">
</head>
<body>
    <div id="swagger-ui"></div>

    <script src="https://cdn.jsdelivr.net/npm/swagger-ui-dist@5/swagger-ui-bundle.js"></script>
    <script src="https://cdn.jsdelivr.net/npm/swagger-ui-dist@5/swagger-ui-standalone-preset.js"></script>
    <script>
        window.onload = function() {
            const ui = SwaggerUIBundle({
                url: './openapi.yaml',
                dom_id: '#swagger-ui',
                deepLinking: true,
                presets: [
                    SwaggerUIBundle.presets.apis,
                    SwaggerUIStandalonePreset
                ],
                plugins: [
                    SwaggerUIBundle.plugins.DownloadUrl
                ],
                layout: "StandaloneLayout",
                // Custom configuration
                displayRequestDuration: true,
                filter: true,
                showExtensions: true,
                showCommonExtensions: true,
                tryItOutEnabled: true,
                requestSnippetsEnabled: true,
                requestSnippets: {
                    generators: {
                        curl_bash: {
                            title: "cURL (bash)",
                            syntax: "bash"
                        },
                        curl_powershell: {
                            title: "cURL (PowerShell)",
                            syntax: "powershell"
                        }
                    }
                }
            });

            window.ui = ui;
        };
    </script>
</body>
</html>
```

**Serve with any web server:**
```bash
# Python
cd docs/api && python -m http.server 8081

# Node.js
cd docs/api && npx http-server -p 8081

# Access at: http://localhost:8081
```

---

## 2. 🧪 Automated API Testing

### Testing Tools That Use OpenAPI

#### A. **Dredd** - OpenAPI Contract Testing

Tests that your API matches the OpenAPI spec:

```bash
npm install -g dredd

# Test API against OpenAPI spec
dredd docs/api/openapi.yaml http://localhost:8080
```

**What it does:**
- Reads OpenAPI spec
- Makes real API calls to your server
- Verifies responses match the spec
- Reports any mismatches

#### B. **Postman** - Import OpenAPI Spec

```bash
# Convert OpenAPI to Postman collection
npx openapi-to-postman -s docs/api/openapi.yaml -o riptide-api.postman.json

# Run automated tests
newman run riptide-api.postman.json
```

#### C. **Schemathesis** - Property-Based Testing

```bash
pip install schemathesis

# Automatically generate test cases
schemathesis run docs/api/openapi.yaml --base-url http://localhost:8080
```

**What it does:**
- Generates 1000s of test cases automatically
- Fuzzes your API with edge cases
- Finds crashes, validation errors, security issues

#### D. **Hurl** - API Testing with `.hurl` Files

Create `tests/api/crawl.hurl`:
```hurl
# Test batch crawl endpoint
POST http://localhost:8080/crawl
Content-Type: application/json
{
  "urls": ["https://example.com"],
  "options": {
    "concurrency": 1,
    "cache_mode": "read_write"
  }
}

HTTP 200
[Asserts]
jsonpath "$.successful" == 1
jsonpath "$.results[0].status" == 200
jsonpath "$.results[0].document.markdown" exists
```

```bash
hurl --test tests/api/*.hurl
```

---

## 3. 🌐 API Gateway Integration

### What is an API Gateway?

An API gateway sits in front of your API and provides:
- **Rate limiting** - Prevent abuse
- **Authentication** - JWT, API keys, OAuth
- **Analytics** - Track usage, performance
- **Caching** - Response caching
- **Load balancing** - Distribute traffic
- **API versioning** - Manage multiple versions

### Popular API Gateways (All support OpenAPI import)

#### A. **Kong Gateway** (Open Source)

```bash
# Install Kong
docker run -d --name kong \
  -e "KONG_DATABASE=off" \
  -e "KONG_DECLARATIVE_CONFIG=/kong.yml" \
  -p 8000:8000 \
  -p 8001:8001 \
  kong/kong-gateway:latest

# Import OpenAPI spec
curl -X POST http://localhost:8001/services \
  -F "name=riptide-api" \
  -F "url=http://localhost:8080"

# Upload OpenAPI spec
curl -X POST http://localhost:8001/services/riptide-api/routes \
  -F "paths[]=/api" \
  -F "spec=@docs/api/openapi.yaml"
```

**Features:**
- Rate limiting per endpoint
- API key authentication
- Request/response transformation
- Analytics dashboard

#### B. **Tyk Gateway** (Open Source)

```bash
# Import OpenAPI spec to Tyk
curl -X POST http://localhost:3000/tyk/apis/oas/import \
  -H "Authorization: Bearer your-secret" \
  -F "file=@docs/api/openapi.yaml"
```

**Auto-generates:**
- Rate limits based on OpenAPI tags
- Authentication policies
- Mock responses for testing
- API documentation portal

#### C. **AWS API Gateway**

```bash
# Import OpenAPI to AWS
aws apigateway import-rest-api \
  --body file://docs/api/openapi.yaml \
  --fail-on-warnings

# Deploy to production
aws apigateway create-deployment \
  --rest-api-id <api-id> \
  --stage-name production
```

**AWS adds:**
- CloudWatch metrics
- Lambda integration
- Cognito authentication
- Auto-scaling
- API throttling

#### D. **KrakenD** (Open Source, Fast)

```json
{
  "version": 3,
  "endpoints": [
    {
      "endpoint": "/api/{path}",
      "method": "GET",
      "backend": [
        {
          "url_pattern": "/{path}",
          "host": ["http://localhost:8080"],
          "extra_config": {
            "qos/ratelimit/router": {
              "max_rate": 100,
              "client_max_rate": 10
            }
          }
        }
      ]
    }
  ]
}
```

### What You Get from API Gateway:

1. **Rate Limiting**
   ```yaml
   # Automatic from OpenAPI
   /crawl:
     rate_limit: 100/minute
     burst: 20
   ```

2. **Authentication**
   ```yaml
   # Auto-configured
   security:
     - api_key: []
   ```

3. **Analytics Dashboard**
   - Requests per endpoint
   - Response times
   - Error rates
   - Top consumers

4. **Documentation Portal**
   - Auto-generated from OpenAPI
   - Interactive API explorer
   - Code samples in multiple languages

---

## Complete Setup Example

### Step 1: Add Swagger UI to RipTide

```rust
// crates/riptide-api/src/main.rs
.route("/docs", get(|| async {
    Html(include_str!("../../docs/api/swagger.html"))
}))
.route("/docs/openapi.yaml", get(|| async {
    (
        StatusCode::OK,
        [("content-type", "application/yaml")],
        include_str!("../../docs/api/openapi.yaml")
    )
}))
```

### Step 2: Add Automated Testing

```yaml
# .github/workflows/api-tests.yml
name: API Tests

on: [push, pull_request]

jobs:
  contract-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Start RipTide API
        run: |
          docker-compose up -d
          sleep 10

      - name: Run Dredd contract tests
        run: |
          npm install -g dredd
          dredd docs/api/openapi.yaml http://localhost:8080

      - name: Run Schemathesis fuzzing
        run: |
          pip install schemathesis
          schemathesis run docs/api/openapi.yaml \
            --base-url http://localhost:8080 \
            --checks all
```

### Step 3: Deploy with Kong Gateway

```yaml
# docker-compose.gateway.yml
version: '3.8'

services:
  kong:
    image: kong/kong-gateway:latest
    environment:
      KONG_DATABASE: "off"
      KONG_PROXY_LISTEN: "0.0.0.0:8000"
      KONG_ADMIN_LISTEN: "0.0.0.0:8001"
    ports:
      - "8000:8000"  # API Gateway
      - "8001:8001"  # Admin API
    volumes:
      - ./docs/api/openapi.yaml:/openapi.yaml

  riptide-api:
    build: .
    ports:
      - "8080:8080"

  swagger-ui:
    image: swaggerapi/swagger-ui
    ports:
      - "8081:8080"
    environment:
      SWAGGER_JSON: /openapi.yaml
    volumes:
      - ./docs/api/openapi.yaml:/openapi.yaml
```

**Access:**
- API Gateway: `http://localhost:8000`
- Admin Dashboard: `http://localhost:8001`
- Swagger UI: `http://localhost:8081`

---

## Benefits Summary

### With Swagger UI:
✅ Interactive API documentation
✅ No Postman needed - test in browser
✅ Always up-to-date (generated from OpenAPI)
✅ Beautiful, professional docs

### With Automated Testing:
✅ Verify API matches spec
✅ Find bugs automatically
✅ Regression testing
✅ Fuzzing for edge cases
✅ CI/CD integration

### With API Gateway:
✅ Rate limiting & throttling
✅ Authentication & authorization
✅ Analytics & monitoring
✅ Load balancing
✅ API versioning
✅ Response caching
✅ Developer portal

---

## Quick Start Commands

```bash
# 1. Start Swagger UI (Docker)
docker run -p 8081:8080 \
  -v $(pwd)/docs/api/openapi.yaml:/openapi.yaml \
  -e SWAGGER_JSON=/openapi.yaml \
  swaggerapi/swagger-ui

# 2. Run automated tests
npm install -g dredd
dredd docs/api/openapi.yaml http://localhost:8080

# 3. Set up Kong Gateway
docker-compose -f docker-compose.gateway.yml up -d

# 4. Import to Kong
curl -X POST http://localhost:8001/config \
  -F "config=@docs/api/openapi.yaml"
```

---

## Next Steps

1. **Choose deployment method** (Docker recommended)
2. **Set up automated testing** in CI/CD
3. **Deploy API gateway** for production
4. **Enable analytics** to track usage

Would you like me to create any of these configurations for your RipTide API?
