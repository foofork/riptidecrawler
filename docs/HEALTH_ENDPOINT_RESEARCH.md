# Health Endpoint Research Report
**Generated**: 2025-10-17
**Researcher**: Hive Mind Research Agent
**Task ID**: task-1760686440764-cyvwhhi40

---

## Executive Summary

RipTide currently implements **3 primary health endpoints** with inconsistent naming conventions and incomplete standards compliance. The system is functional but does not fully adhere to Kubernetes health check best practices or IETF draft specifications.

### Current Endpoints
1. `/healthz` - Basic health check (Kubernetes-style, primary)
2. `/api/v1/health` - REST API alias
3. `/api/health/detailed` - Enhanced diagnostics with full metrics

### Key Findings
- ✅ **Working**: All endpoints functional with comprehensive checks
- ⚠️ **Missing**: Kubernetes `/livez` and `/readyz` standards
- ⚠️ **Non-standard**: Status values ("healthy/degraded/unhealthy" vs IETF "pass/warn/fail")
- ⚠️ **Inconsistent**: Path structure across endpoints
- ✅ **Correct**: HTTP status codes (200 for healthy, 503 for unhealthy)

---

## Industry Standards Analysis

### 1. Kubernetes Health Check Standards (v1.16+)

#### Official Endpoints
```
/healthz  - DEPRECATED since v1.16 (but still widely used)
/livez    - RECOMMENDED for liveness checks
/readyz   - RECOMMENDED for readiness checks
```

#### Key Characteristics
- **HTTP Status**: 200 = healthy, non-200 = unhealthy
- **Query Parameters**: `?verbose` for detailed output, `?exclude=<check>` to skip checks
- **Purpose Separation**:
  - **Liveness**: "Is the process running?" (kills pod if fails)
  - **Readiness**: "Can it handle traffic?" (removes from load balancer if fails)

#### Best Practices
- Initial delay for liveness should allow app startup
- Readiness has no grace period (must be ready immediately)
- Different checks for liveness vs readiness (avoid killing pod unnecessarily)
- Use HTTP GET probe type (most common)
- Response time: < 100ms for liveness, < 5s for readiness

#### Source
- [Kubernetes API Health Endpoints](https://kubernetes.io/docs/reference/using-api/health-checks/)
- [Configure Liveness, Readiness and Startup Probes](https://kubernetes.io/docs/tasks/configure-pod-container/configure-liveness-readiness-startup-probes/)

---

### 2. IETF Draft Specification (RFC Draft)

#### Overview
**Draft**: `draft-inadarei-api-health-check-06`
**Status**: Expired (2022) but represents closest to standardization
**Media Type**: `application/health+json`

#### Response Format
```json
{
  "status": "pass|warn|fail",
  "version": "1.0.0",
  "releaseId": "git-sha-12345",
  "serviceId": "riptide-api",
  "description": "RipTide Web Extraction API",
  "checks": {
    "redis:connectivity": {
      "componentType": "datastore",
      "observedValue": "connected",
      "status": "pass",
      "time": "2025-10-17T07:30:00Z",
      "output": "Redis operations successful"
    }
  },
  "notes": ["Optional warnings or info"],
  "links": {
    "self": "/health"
  }
}
```

#### Status Values
- `"pass"` - Healthy (aliases: "ok", "up")
- `"fail"` - Unhealthy (aliases: "error", "down")
- `"warn"` - Healthy with concerns (degraded)

#### HTTP Status Code Requirements
- `pass` or `warn` → **2xx-3xx** range
- `fail` → **4xx-5xx** range

#### Optional Fields
- `version`, `releaseId`, `serviceId`, `description`
- `checks` - Individual component health
- `notes` - Array of informational messages
- `output` - Detailed diagnostic information
- `links` - Related URLs

#### Source
- [IETF Draft Specification](https://datatracker.ietf.org/doc/html/draft-inadarei-api-health-check-06)

---

### 3. REST API Naming Conventions

#### Common Patterns

**Kubernetes/Docker Ecosystem**:
```
/healthz  - Most common in containerized environments
/livez    - Liveness probe
/readyz   - Readiness probe
```

**Traditional REST APIs**:
```
/health           - Simple, clear
/api/health       - Namespaced
/api/v1/health    - Versioned API
```

**Detailed Diagnostics**:
```
/health/detailed  - Comprehensive diagnostics
/health/:component - Component-specific checks
/health/metrics   - System metrics only
```

#### Naming Convention Origins

**Why "/healthz" with a 'z'?**
- Origin: Google's internal "z-pages" convention
- Purpose: Reduce collision with existing `/health` endpoints
- Related endpoints: `/varz`, `/statusz`, `/rpcz`
- Adoption: Kubernetes ecosystem, Docker, cloud-native services
- Reason: If you already have a `/health` endpoint for other purposes, you don't need to rename it

#### Sources
- [StackOverflow: Where does /healthz come from?](https://stackoverflow.com/questions/43380939/where-does-the-convention-of-using-healthz-for-application-health-checks-come-f)
- [Microservices Pattern: Health Check API](https://microservices.io/patterns/observability/health-check-api.html)

---

### 4. Basic vs Detailed Health Checks

#### Basic Health Check
**Purpose**: Quick availability check
**Response Time**: < 100ms
**Content**: HTTP status code only or minimal JSON
**Use Cases**:
- Load balancer health checks
- Kubernetes liveness probes
- High-frequency monitoring (every 5-10s)

```json
{
  "status": "healthy"
}
```

#### Detailed Health Check
**Purpose**: Comprehensive diagnostics
**Response Time**: < 10s
**Content**: Full system state with metrics
**Use Cases**:
- Monitoring dashboards
- Debugging and troubleshooting
- Initial deployment validation
- Low-frequency checks (every 60s+)

```json
{
  "status": "healthy",
  "version": "0.1.0",
  "uptime": 86400,
  "dependencies": {
    "redis": {
      "status": "healthy",
      "response_time_ms": 15,
      "message": "Redis operations successful"
    },
    "database": {...},
    "cache": {...}
  },
  "metrics": {
    "memory_usage_bytes": 524288000,
    "cpu_usage_percent": 45.2,
    "active_connections": 127,
    "requests_per_second": 850.5
  }
}
```

#### Key Differences
| Aspect | Basic | Detailed |
|--------|-------|----------|
| Response Size | < 100 bytes | 1-10 KB |
| Execution Time | < 100ms | < 10s |
| Dependency Checks | None or minimal | All dependencies |
| Metrics | None | Comprehensive |
| Use Frequency | Every 5-10s | Every 60s+ |
| HTTP Route | `/healthz`, `/livez` | `/health/detailed` |

---

## Current RipTide Implementation Analysis

### Endpoint Inventory

#### Primary Endpoints
```rust
// From /workspaces/eventmesh/crates/riptide-api/src/main.rs:162-172

.route("/healthz", get(handlers::health))
.route("/api/v1/health", get(handlers::health)) // v1 alias
.route("/api/health/detailed", get(handlers::health_detailed))
.route("/health/:component", get(handlers::health::component_health_check))
.route("/health/metrics", get(handlers::health::health_metrics_check))
```

#### Nested Component Endpoints
```rust
// Stealth service health
.nest("/stealth", routes::stealth::stealth_routes())
  → /stealth/health

// PDF processing health
.nest("/pdf", routes::pdf::pdf_routes())
  → /pdf/health
```

### Response Structure

#### Current HealthResponse Model
```rust
// From /workspaces/eventmesh/crates/riptide-api/src/models.rs:182-200

pub struct HealthResponse {
    pub status: String,              // "healthy", "degraded", "unhealthy"
    pub version: String,             // Application version
    pub timestamp: String,           // RFC3339 timestamp
    pub uptime: u64,                 // Seconds since startup
    pub dependencies: DependencyStatus,
    pub metrics: Option<SystemMetrics>,
}
```

#### Dependency Status Structure
```rust
pub struct DependencyStatus {
    pub redis: ServiceHealth,
    pub extractor: ServiceHealth,       // WASM extractor
    pub http_client: ServiceHealth,
    pub headless_service: Option<ServiceHealth>,
    pub spider_engine: Option<ServiceHealth>,
}

pub struct ServiceHealth {
    pub status: String,                 // "healthy", "unhealthy", "unknown"
    pub message: Option<String>,
    pub response_time_ms: Option<u64>,
    pub last_check: String,             // RFC3339 timestamp
}
```

#### System Metrics Structure
```rust
pub struct SystemMetrics {
    pub memory_usage_bytes: u64,
    pub active_connections: u32,
    pub total_requests: u64,
    pub requests_per_second: f64,
    pub avg_response_time_ms: f64,
    pub cpu_usage_percent: Option<f32>,
    pub disk_usage_bytes: Option<u64>,
    pub file_descriptor_count: Option<u32>,
    pub thread_count: Option<u32>,
    pub load_average: Option<[f32; 3]>,  // 1min, 5min, 15min
}
```

### Health Check Implementation

#### Basic Check (/healthz)
```rust
// From /workspaces/eventmesh/crates/riptide-api/src/handlers/health.rs:34

pub async fn health(State(state): State<AppState>) -> Result<impl IntoResponse, ApiError> {
    // 5-second timeout for health check
    let health_status = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        state.health_check()
    ).await?;

    // Status determination
    let overall_status = if health_status.healthy {
        "healthy"
    } else if browser_pool_only_failure {
        "degraded"  // Special case: browser pool failure tolerated
    } else {
        "unhealthy"
    };

    // HTTP status code
    let status_code = if health_status.healthy || overall_status == "degraded" {
        StatusCode::OK  // 200
    } else {
        StatusCode::SERVICE_UNAVAILABLE  // 503
    };

    Ok((status_code, Json(response)))
}
```

#### Detailed Check (/api/health/detailed)
```rust
// From /workspaces/eventmesh/crates/riptide-api/src/handlers/health.rs:252

pub async fn health_detailed(State(state): State<AppState>) -> Result<impl IntoResponse, ApiError> {
    // 10-second timeout for comprehensive check
    let health_response = tokio::time::timeout(
        std::time::Duration::from_secs(10),
        state.health_checker.check_health(&state)
    ).await?;

    // Uses HealthChecker for comprehensive diagnostics:
    // - All dependency checks with response times
    // - Complete system metrics
    // - Build information (git SHA, timestamp)
    // - Component versions
    // - Performance bucket configuration

    Ok((status_code, Json(health_response)))
}
```

### Dependency Health Checks

#### Redis Check
```rust
// Tests: set/get/delete operations
// Includes batch performance test (10 keys)
// Response time tracking
// Status: "healthy" | "unhealthy"
```

#### HTTP Client Check
```rust
// Tests multiple endpoints:
// - https://httpbin.org/status/200
// - https://www.google.com/robots.txt
// Status: "healthy" | "degraded" | "unhealthy"
// Tolerates partial failures
```

#### WASM Extractor Check
```rust
// Checks initialization status
// Status: "healthy" (assumes healthy if initialized)
```

#### Headless Service Check (Optional)
```rust
// If configured: GET {headless_url}/health
// Response time tracking
// Status: "healthy" | "unhealthy" | "not_configured"
```

#### Spider Engine Check (Optional)
```rust
// If configured: checks spider initialization
// Status: "healthy" | "unhealthy" | "not_configured"
```

### Status Value Mapping

#### Current Status Values
```
"healthy"    - All systems operational
"degraded"   - Browser pool failure only (non-critical)
"unhealthy"  - Critical dependencies failing
```

#### HTTP Status Code Mapping
```
healthy   → 200 OK
degraded  → 200 OK (allows traffic, warns in response)
unhealthy → 503 Service Unavailable
```

---

## Identified Inconsistencies

### 1. Endpoint Naming Collision
**Issue**: Multiple paths serve similar purposes
```
/healthz              → Basic check
/api/v1/health        → Same as /healthz (alias)
/api/health/detailed  → Enhanced check
```

**Problem**:
- Unclear which endpoint to use
- `/api/v1/health` suggests versioning but no v2 exists
- `/api/health/detailed` not under `/api/v1/`

---

### 2. Status Values Non-Standard

**Current**: `"healthy"`, `"degraded"`, `"unhealthy"`
**IETF Standard**: `"pass"`, `"warn"`, `"fail"`
**Kubernetes**: HTTP status codes only

**Mapping Suggestion**:
```
healthy   → pass
degraded  → warn
unhealthy → fail
```

---

### 3. Missing Kubernetes Standards

**Missing Endpoints**:
- `/livez` - Liveness check (is process alive?)
- `/readyz` - Readiness check (can handle traffic?)

**Current Workaround**:
- `/healthz` serves as both liveness and readiness
- No separation of concerns
- Cannot configure different probe behaviors

---

### 4. Media Type Non-Standard

**Current**: `application/json`
**IETF Standard**: `application/health+json`

**Impact**:
- Monitoring tools may not recognize as health check
- Missing semantic meaning in Content-Type

---

### 5. Response Format Differences

**Current Structure** vs **IETF Structure**:

| Field | Current | IETF |
|-------|---------|------|
| Status | `status: "healthy"` | `status: "pass"` |
| Components | `dependencies: {...}` | `checks: {...}` |
| Service Info | `version` only | `version`, `releaseId`, `serviceId`, `description` |
| Component Format | Flat structure | Hierarchical with `componentType`, `observedValue` |

---

### 6. Nested Route Confusion

**Issue**: Component-specific health endpoints in different locations
```
/health/:component     - Main health components (redis, extractor, etc.)
/stealth/health        - Stealth service health
/pdf/health            - PDF service health
```

**Problem**:
- Inconsistent discovery pattern
- Hard to enumerate all health endpoints
- Should all be under `/health/` hierarchy

---

## Recommendations

### Phase 1: Add Kubernetes-Compatible Endpoints (Backward Compatible)

#### Implement New Standard Endpoints
```rust
// Liveness check - lightweight, fast
.route("/livez", get(handlers::health_liveness))
.route("/health/live", get(handlers::health_liveness))  // Alias

// Readiness check - comprehensive dependency checks
.route("/readyz", get(handlers::health_readiness))
.route("/health/ready", get(handlers::health_readiness))  // Alias
```

#### Keep Existing Endpoints (Deprecated)
```rust
.route("/healthz", get(handlers::health))  // Mark as deprecated in docs
.route("/api/v1/health", get(handlers::health))  // Redirect to /healthz
.route("/api/health/detailed", get(handlers::health))  // Redirect to /health
```

---

### Phase 2: Standardize Response Format

#### Add IETF-Compatible Response

```rust
// Support content negotiation
// Accept: application/health+json → IETF format
// Accept: application/json → Current format (default)

pub struct IETFHealthResponse {
    pub status: String,  // "pass", "warn", "fail"
    pub version: Option<String>,
    pub releaseId: Option<String>,
    pub serviceId: Option<String>,
    pub description: Option<String>,
    pub checks: HashMap<String, ComponentCheck>,
    pub notes: Option<Vec<String>>,
    pub links: Option<HashMap<String, String>>,
}

pub struct ComponentCheck {
    pub componentType: String,  // "datastore", "system", "service"
    pub observedValue: Option<String>,
    pub observedUnit: Option<String>,
    pub status: String,  // "pass", "warn", "fail"
    pub time: String,  // RFC3339
    pub output: Option<String>,
}
```

#### Example IETF Response
```json
{
  "status": "pass",
  "version": "0.1.0",
  "releaseId": "abc123git",
  "serviceId": "riptide-api",
  "description": "RipTide Web Extraction API",
  "checks": {
    "redis:connectivity": {
      "componentType": "datastore",
      "observedValue": "connected",
      "status": "pass",
      "time": "2025-10-17T07:30:00Z",
      "output": "Redis operations successful (15ms)"
    },
    "wasm:extractor": {
      "componentType": "system",
      "status": "pass",
      "time": "2025-10-17T07:30:00Z"
    },
    "http:client": {
      "componentType": "service",
      "observedValue": "2/2 endpoints reachable",
      "status": "pass",
      "time": "2025-10-17T07:30:00Z",
      "output": "All test endpoints responding"
    }
  },
  "links": {
    "self": "/health",
    "livez": "/livez",
    "readyz": "/readyz"
  }
}
```

---

### Phase 3: Consolidate Routes

#### Recommended Route Structure
```rust
// Primary health endpoints
.route("/health", get(handlers::health_comprehensive))      // Main entry point
.route("/health/live", get(handlers::health_liveness))      // Liveness (fast)
.route("/health/ready", get(handlers::health_readiness))    // Readiness (with deps)

// Kubernetes compatibility
.route("/healthz", get(handlers::health_liveness))          // Alias to /health/live
.route("/livez", get(handlers::health_liveness))            // Kubernetes standard
.route("/readyz", get(handlers::health_readiness))          // Kubernetes standard

// Component-specific checks
.route("/health/component/:name", get(handlers::component_health))

// Legacy support (deprecated, redirect)
.route("/api/v1/health", get(redirect_to("/health")))
.route("/api/health/detailed", get(redirect_to("/health")))
```

#### Health Hierarchy
```
/health                    - Comprehensive diagnostics (detailed)
  ├─ /health/live          - Liveness check (lightweight)
  ├─ /health/ready         - Readiness check (with dependencies)
  ├─ /health/metrics       - System metrics only
  └─ /health/component/:name - Component-specific health
      ├─ /health/component/redis
      ├─ /health/component/wasm
      ├─ /health/component/http-client
      ├─ /health/component/headless
      ├─ /health/component/spider
      ├─ /health/component/pdf
      └─ /health/component/stealth
```

---

### Phase 4: Implementation Specifications

#### Liveness Check (/livez, /health/live)
```rust
/// Minimal health check for Kubernetes liveness probe
/// - Response time target: < 100ms
/// - Checks: Process alive, basic memory validation
/// - No external dependencies
/// - Always returns 200 unless process is dying
pub async fn health_liveness() -> Result<impl IntoResponse, ApiError> {
    let response = json!({
        "status": "pass",
        "timestamp": Utc::now().to_rfc3339()
    });
    Ok((StatusCode::OK, Json(response)))
}
```

#### Readiness Check (/readyz, /health/ready)
```rust
/// Comprehensive readiness check for Kubernetes readiness probe
/// - Response time target: < 5s
/// - Checks: All critical dependencies (Redis, HTTP client, WASM)
/// - Returns 200 if ready, 503 if not ready
/// - Removes from load balancer if fails
pub async fn health_readiness(State(state): State<AppState>) -> Result<impl IntoResponse, ApiError> {
    let health_status = tokio::time::timeout(
        Duration::from_secs(5),
        state.health_check()
    ).await?;

    let status = if health_status.healthy { "pass" } else { "fail" };
    let http_status = if health_status.healthy {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    Ok((http_status, Json(json!({
        "status": status,
        "timestamp": Utc::now().to_rfc3339(),
        "checks": {
            "redis": health_status.redis.to_string(),
            "extractor": health_status.extractor.to_string(),
            "http_client": health_status.http_client.to_string()
        }
    }))))
}
```

#### Comprehensive Health (/health)
```rust
/// Full health diagnostics with all details
/// - Response time target: < 10s
/// - Checks: All dependencies + system metrics + build info
/// - Content negotiation: application/health+json or application/json
pub async fn health_comprehensive(
    State(state): State<AppState>,
    headers: HeaderMap
) -> Result<impl IntoResponse, ApiError> {
    let accept = headers.get("accept")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("application/json");

    let health = state.health_checker.check_health(&state).await;

    if accept.contains("application/health+json") {
        // Return IETF-compliant format
        Ok((StatusCode::OK, Json(health.to_ietf_format())))
    } else {
        // Return current format
        Ok((StatusCode::OK, Json(health)))
    }
}
```

---

### Phase 5: Update Documentation

#### OpenAPI/Swagger Specification
```yaml
paths:
  /health:
    get:
      summary: Comprehensive health check
      description: Returns detailed health status with all dependencies and metrics
      responses:
        200:
          description: Service is healthy
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/HealthResponse'
            application/health+json:
              schema:
                $ref: '#/components/schemas/IETFHealthResponse'
        503:
          description: Service is unhealthy

  /livez:
    get:
      summary: Liveness check
      description: Kubernetes liveness probe - checks if process is alive
      responses:
        200:
          description: Process is alive

  /readyz:
    get:
      summary: Readiness check
      description: Kubernetes readiness probe - checks if service can handle traffic
      responses:
        200:
          description: Ready to serve traffic
        503:
          description: Not ready to serve traffic
```

#### Kubernetes Deployment Example
```yaml
apiVersion: v1
kind: Pod
metadata:
  name: riptide-api
spec:
  containers:
  - name: riptide-api
    image: riptide-api:latest
    ports:
    - containerPort: 8080
    livenessProbe:
      httpGet:
        path: /livez
        port: 8080
      initialDelaySeconds: 15
      periodSeconds: 10
      timeoutSeconds: 1
      failureThreshold: 3
    readinessProbe:
      httpGet:
        path: /readyz
        port: 8080
      initialDelaySeconds: 5
      periodSeconds: 5
      timeoutSeconds: 5
      failureThreshold: 2
```

---

## Migration Path

### Step 1: Immediate (Week 1)
1. Add `/livez` and `/readyz` endpoints
2. Document new endpoints
3. Update Kubernetes manifests (if any)

### Step 2: Near-term (Week 2-3)
1. Implement IETF response format support
2. Add content negotiation
3. Add component hierarchy consolidation

### Step 3: Long-term (Month 2-3)
1. Deprecate `/api/v1/health` and `/api/health/detailed`
2. Add deprecation warnings to responses
3. Update client documentation
4. Monitor usage and migrate clients

### Step 4: Cleanup (Month 6+)
1. Remove deprecated endpoints (if usage is near zero)
2. Simplify codebase
3. Update all documentation

---

## Reference Implementations

### Kubernetes API Server
- Source: [kubernetes/apiserver/pkg/server/healthz.go](https://github.com/kubernetes/apiserver/blob/master/pkg/server/healthz.go)
- Implements: `/livez`, `/readyz`, `/healthz`
- Pattern: Pluggable health checks with verbose mode

### Spring Boot Actuator
- Endpoint: `/actuator/health`
- Format: Similar to RipTide detailed
- Features: Component hierarchy, status aggregation

### Node.js Terminus
- Pattern: `/health` with pass/fail
- Uses: Express middleware
- Format: Simple JSON with optional details

### HashiCorp Consul
- Endpoints: `/v1/health/node/:node`
- Status: passing, warning, critical
- Features: Service mesh health aggregation

---

## Testing Requirements

### Unit Tests
```rust
#[tokio::test]
async fn test_livez_responds_quickly() {
    let start = Instant::now();
    let response = health_liveness().await;
    assert!(start.elapsed() < Duration::from_millis(100));
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_readyz_checks_dependencies() {
    // Should fail when Redis is down
    // Should fail when HTTP client is broken
    // Should pass when all dependencies healthy
}

#[tokio::test]
async fn test_ietf_format_compliance() {
    // Validate IETF response structure
    // Check required fields
    // Verify status values
}
```

### Integration Tests
```bash
# Liveness check
curl http://localhost:8080/livez
# Should return < 100ms

# Readiness check
curl http://localhost:8080/readyz
# Should check dependencies

# IETF format
curl -H "Accept: application/health+json" http://localhost:8080/health
# Should return IETF-compliant response
```

### Load Tests
```bash
# Sustained load on /livez (should not degrade)
wrk -t4 -c100 -d30s http://localhost:8080/livez

# Readiness check under load
wrk -t2 -c10 -d30s http://localhost:8080/readyz
```

---

## Conclusion

RipTide's current health endpoint implementation is **functional and comprehensive** but lacks full standards compliance. The system performs thorough dependency checking and provides detailed metrics, which is excellent.

### Strengths
✅ Comprehensive dependency checks (Redis, WASM, HTTP client, headless, spider)
✅ Detailed system metrics (CPU, memory, disk, threads, load)
✅ Correct HTTP status codes
✅ Timeout protection (5s basic, 10s detailed)
✅ Special handling for non-critical failures (browser pool)
✅ Response time tracking for dependencies

### Gaps
⚠️ Missing Kubernetes `/livez` and `/readyz` standards
⚠️ Non-standard status values (not IETF-compliant)
⚠️ Inconsistent endpoint paths
⚠️ No IETF response format support
⚠️ Nested component health endpoints scattered

### Recommended Priority
**HIGH**: Implement `/livez` and `/readyz` for Kubernetes compatibility
**MEDIUM**: Add IETF response format support
**LOW**: Consolidate component health endpoints (can maintain backward compatibility)

---

## Appendix A: Complete Endpoint Mapping

```
Current Implementation:
├─ /healthz                          → Basic health (primary)
├─ /api/v1/health                    → Alias to /healthz
├─ /api/health/detailed              → Comprehensive health
├─ /health/:component                → Component-specific
├─ /health/metrics                   → System metrics only
├─ /stealth/health                   → Stealth service health
└─ /pdf/health                       → PDF service health

Recommended Implementation:
├─ /health                           → Comprehensive (new primary)
│   ├─ /health/live                  → Liveness check
│   ├─ /health/ready                 → Readiness check
│   ├─ /health/metrics               → System metrics only
│   └─ /health/component/:name       → Component-specific
│       ├─ /health/component/redis
│       ├─ /health/component/wasm
│       ├─ /health/component/http-client
│       ├─ /health/component/headless
│       ├─ /health/component/spider
│       ├─ /health/component/pdf
│       └─ /health/component/stealth
├─ /healthz                          → Alias to /health/live (deprecated)
├─ /livez                            → Kubernetes standard
├─ /readyz                           → Kubernetes standard
└─ /api/v1/health                    → Redirect (deprecated)
```

---

## Appendix B: HTTP Status Code Reference

### Current Implementation
```
200 OK                   - healthy or degraded
503 Service Unavailable  - unhealthy
```

### Recommended Implementation
```
200 OK                   - pass (healthy)
200 OK                   - warn (degraded but operational)
503 Service Unavailable  - fail (unhealthy, cannot serve traffic)
```

### Rationale
Both implementations follow best practices. The key is consistency:
- **2xx**: Service can handle requests
- **5xx**: Service cannot handle requests
- Load balancers and Kubernetes use status codes, not response body

---

## Appendix C: Additional Resources

### Standards
- [IETF Health Check Draft](https://datatracker.ietf.org/doc/html/draft-inadarei-api-health-check-06)
- [Kubernetes Health Checks](https://kubernetes.io/docs/reference/using-api/health-checks/)
- [Microservices Health Check Pattern](https://microservices.io/patterns/observability/health-check-api.html)

### Articles
- [Health Check Response Format for HTTP APIs](https://blog.frankel.ch/healthcheck-http-apis/)
- [A Deep Dive into Proper Health Check API Implementation](https://thinhdanggroup.github.io/health-check-api/)
- [Writing Meaningful Health Check Endpoints](https://emmer.dev/blog/writing-meaningful-health-check-endpoints/)

### Tools
- [Spring Boot Actuator](https://docs.spring.io/spring-boot/docs/current/reference/html/actuator.html#actuator.endpoints.health)
- [Node.js Terminus](https://github.com/godaddy/terminus)
- [.NET Health Checks](https://learn.microsoft.com/en-us/aspnet/core/host-and-deploy/health-checks)

---

**End of Report**
