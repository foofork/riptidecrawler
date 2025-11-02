# Authentication Implementation Plan

**Version:** 1.0
**Date:** 2025-11-02
**Status:** Ready for Implementation
**Estimated Effort:** 3-5 days

## Overview

This document provides a detailed implementation plan for enhancing the Riptide API authentication system based on the architecture defined in `authentication-architecture.md` and security strategy in `authentication-security.md`.

## Implementation Phases

### Phase 1: Core Storage Infrastructure (Day 1)

**Objective:** Create the foundational storage layer for API keys.

#### Tasks

1. **Update Dependencies**
   - File: `crates/riptide-api/Cargo.toml`
   - Add: `sha2 = "0.10"`
   - Verify: `cargo check` passes

2. **Create Module Structure**
   - Create directory: `crates/riptide-api/src/middleware/auth/`
   - Create files:
     - `mod.rs` - Module exports
     - `types.rs` - Data structures
     - `storage.rs` - Redis storage backend
     - `generator.rs` - Key generation
     - `audit.rs` - Audit logging

3. **Implement Types** (`types.rs`)
   ```rust
   pub struct ApiKey {
       pub key_hash: String,
       pub description: String,
       pub created_at: String,
       pub last_used: Option<String>,
       pub scopes: Vec<String>,
       pub is_active: bool,
       pub expires_at: Option<String>,
   }

   pub struct ApiKeyHash(pub String);
   ```

4. **Implement Key Generator** (`generator.rs`)
   ```rust
   pub fn generate_api_key() -> String;
   pub fn hash_api_key(key: &str) -> String;
   ```

5. **Implement Storage Backend** (`storage.rs`)
   ```rust
   pub struct ApiKeyStore {
       cache: Arc<tokio::sync::Mutex<CacheManager>>,
   }

   impl ApiKeyStore {
       pub async fn get(&self, hash: &str) -> Result<ApiKey>;
       pub async fn save(&self, hash: &str, key: &ApiKey) -> Result<()>;
       pub async fn delete(&self, hash: &str) -> Result<()>;
       pub async fn list_active(&self) -> Result<Vec<ApiKey>>;
       pub async fn update_last_used(&self, hash: &str) -> Result<()>;
   }
   ```

6. **Write Unit Tests**
   - Test key generation randomness
   - Test hash consistency
   - Test storage CRUD operations
   - Test expiration logic

**Deliverables:**
- [x] Updated Cargo.toml
- [x] Complete auth module structure
- [x] All functions with documentation
- [x] Unit tests with >80% coverage

**Estimated Time:** 6-8 hours

---

### Phase 2: Middleware Enhancement (Day 2)

**Objective:** Integrate Redis storage into existing auth middleware.

#### Tasks

1. **Update AppState**
   - File: `crates/riptide-api/src/state.rs`
   - Modify `AppState::new()` to include `ApiKeyStore`
   - Wire up Redis connection from existing `CacheManager`

2. **Refactor AuthConfig**
   - File: `crates/riptide-api/src/middleware/auth.rs`
   - Replace `Arc<RwLock<HashSet<String>>>` with `Arc<ApiKeyStore>`
   - Update `is_valid_key()` to use storage backend
   - Keep environment variable support for backward compatibility

3. **Enhance auth_middleware**
   - Add brute force protection
   - Add audit logging for all events
   - Attach `ApiKeyHash` to request extensions
   - Implement async `last_used` updates

4. **Implement Audit Logger** (`audit.rs`)
   ```rust
   pub enum AuthEvent {
       ValidationSuccess { key_hash: String, endpoint: String },
       ValidationFailure { reason: String, ip: String },
       SuspiciousActivity { details: String },
   }

   pub async fn log_auth_event(
       event: AuthEvent,
       telemetry: Option<&TelemetrySystem>,
   );
   ```

5. **Write Integration Tests**
   - Test end-to-end authentication flow
   - Test Redis failure scenarios
   - Test brute force protection
   - Test audit logging

**Deliverables:**
- [x] Updated auth.rs with storage integration
- [x] Audit logging implemented
- [x] Brute force protection active
- [x] Integration tests passing

**Estimated Time:** 6-8 hours

---

### Phase 3: Admin Endpoints (Day 3)

**Objective:** Create management endpoints for API keys.

#### Tasks

1. **Create Admin Handlers**
   - File: `crates/riptide-api/src/handlers/admin.rs`
   - Implement handlers:
     ```rust
     pub async fn create_api_key(
         State(state): State<AppState>,
         Json(payload): Json<CreateKeyRequest>,
     ) -> Result<Json<CreateKeyResponse>, ApiError>;

     pub async fn list_api_keys(
         State(state): State<AppState>,
     ) -> Result<Json<Vec<ApiKey>>, ApiError>;

     pub async fn revoke_api_key(
         State(state): State<AppState>,
         Path(key_hash): Path<String>,
     ) -> Result<StatusCode, ApiError>;

     pub async fn rotate_api_key(
         State(state): State<AppState>,
         Path(key_hash): Path<String>,
     ) -> Result<Json<RotateKeyResponse>, ApiError>;
     ```

2. **Create Admin Routes**
   - File: `crates/riptide-api/src/routes/admin.rs`
   - Define routes:
     ```rust
     pub fn admin_routes() -> Router<AppState> {
         Router::new()
             .route("/api/v1/admin/keys", post(create_api_key))
             .route("/api/v1/admin/keys", get(list_api_keys))
             .route("/api/v1/admin/keys/:hash", delete(revoke_api_key))
             .route("/api/v1/admin/keys/:hash/rotate", post(rotate_api_key))
             // Future: Add scope-based authorization
     }
     ```

3. **Update Route Module**
   - File: `crates/riptide-api/src/routes/mod.rs`
   - Add: `pub mod admin;`

4. **Wire Routes in Main**
   - File: `crates/riptide-api/src/main.rs`
   - Add admin routes to router

5. **Write API Tests**
   - Test key creation flow
   - Test key listing
   - Test revocation
   - Test rotation with grace period

**Deliverables:**
- [x] Complete admin handlers
- [x] Routes configured
- [x] Request/response DTOs
- [x] API tests passing

**Estimated Time:** 4-6 hours

---

### Phase 4: Rate Limiting Integration (Day 4)

**Objective:** Tie rate limiting to authenticated API keys.

#### Tasks

1. **Update Rate Limit Middleware**
   - File: `crates/riptide-api/src/middleware/rate_limit.rs`
   - Modify `extract_client_id()`:
     ```rust
     fn extract_client_id(request: &Request) -> Option<String> {
         // Prioritize authenticated key hash
         if let Some(api_key_hash) = request.extensions().get::<ApiKeyHash>() {
             return Some(api_key_hash.0.clone());
         }

         // Fall back to existing logic
         // ... existing code ...
     }
     ```

2. **Add Per-Key Metrics**
   - Update `RipTideMetrics` to track per-key usage
   - Add metric: `api_requests_per_key{key_hash}`

3. **Write Load Tests**
   - Test rate limiting with authenticated keys
   - Test rate limiting with unauthenticated requests
   - Verify consistent client ID across IPs

**Deliverables:**
- [x] Rate limiting uses API key hash
- [x] Metrics updated
- [x] Load tests passing (>1000 req/s)

**Estimated Time:** 3-4 hours

---

### Phase 5: Security Hardening (Day 4-5)

**Objective:** Implement security best practices.

#### Tasks

1. **Add Security Headers**
   - File: `crates/riptide-api/src/main.rs`
   - Add `Strict-Transport-Security` header
   - Add `X-Content-Type-Options: nosniff`
   - Add `X-Frame-Options: DENY`

2. **Implement Log Sanitization**
   - Create utility function to redact keys in logs
   - Apply to all tracing calls with API keys

3. **Add Suspicious Activity Detection**
   - Monitor failed authentication attempts
   - Alert on brute force patterns
   - Track concurrent key usage across IPs

4. **Document Secret Management**
   - Create deployment guide for production
   - Document Redis AUTH configuration
   - Document environment variable security

5. **Security Testing**
   - Test brute force protection
   - Test information disclosure prevention
   - Test error message sanitization
   - Penetration testing (if applicable)

**Deliverables:**
- [x] Security headers configured
- [x] Log sanitization implemented
- [x] Monitoring alerts configured
- [x] Security documentation complete

**Estimated Time:** 4-6 hours

---

### Phase 6: Testing & Documentation (Day 5)

**Objective:** Comprehensive testing and documentation.

#### Tasks

1. **Unit Tests**
   - Coverage > 80% for all auth modules
   - Edge case testing
   - Error handling validation

2. **Integration Tests**
   - End-to-end authentication flows
   - Redis failure scenarios
   - Middleware interaction

3. **Performance Tests**
   - Load testing (target: 10k+ req/s)
   - Latency testing (target: < 10ms auth overhead)
   - Concurrent request testing

4. **Update API Documentation**
   - Document authentication headers
   - Document admin endpoints
   - Document error responses
   - Create Postman collection

5. **Create Migration Guide**
   - Guide for migrating from env-var keys to Redis
   - Backward compatibility notes
   - Rollback procedures

**Deliverables:**
- [x] All tests passing
- [x] Performance benchmarks met
- [x] Complete API documentation
- [x] Migration guide ready

**Estimated Time:** 6-8 hours

---

## File Locations Reference

### New Files

```
crates/riptide-api/src/
├── middleware/
│   └── auth/                      # NEW: Auth submodule
│       ├── mod.rs                 # Module exports
│       ├── types.rs               # ApiKey, ApiKeyHash structs
│       ├── storage.rs             # Redis storage backend
│       ├── generator.rs           # Key generation utilities
│       └── audit.rs               # Audit logging
├── handlers/
│   └── admin.rs                   # NEW: Key management handlers
└── routes/
    └── admin.rs                   # NEW: Admin routes
```

### Modified Files

```
crates/riptide-api/
├── Cargo.toml                     # Add sha2 dependency
├── src/
│   ├── main.rs                    # Wire admin routes, security headers
│   ├── state.rs                   # Add ApiKeyStore to AppState
│   ├── middleware/
│   │   ├── mod.rs                 # Export auth submodule
│   │   ├── auth.rs                # Integrate storage backend
│   │   └── rate_limit.rs          # Extract ApiKeyHash from extensions
│   ├── routes/
│   │   └── mod.rs                 # Export admin routes
│   └── handlers/
│       └── mod.rs                 # Export admin handlers
```

## Dependencies

### Existing Dependencies (Already in Cargo.toml)
- `axum` - Web framework
- `tokio` - Async runtime
- `redis` - Redis client (via riptide-cache)
- `serde` - Serialization
- `tracing` - Logging
- `rand` - Random number generation

### New Dependencies
- `sha2 = "0.10"` - SHA-256 hashing

## Integration Points

### 1. AppState

Current structure in `state.rs`:
```rust
pub struct AppState {
    pub auth_config: AuthConfig,
    pub cache: Arc<tokio::sync::Mutex<CacheManager>>,
    // ... other fields ...
}
```

Enhancement:
```rust
pub struct AppState {
    pub auth_config: AuthConfig,
    pub api_key_store: Arc<ApiKeyStore>,  // NEW
    pub cache: Arc<tokio::sync::Mutex<CacheManager>>,
    // ... other fields ...
}
```

### 2. Middleware Stack (main.rs)

Current order (no changes needed):
```rust
.layer(middleware::from_fn_with_state(state.clone(), auth_middleware))
.layer(middleware::from_fn_with_state(state.clone(), rate_limit_middleware))
```

### 3. Error Handling (errors.rs)

Already defined:
```rust
#[error("Authentication failed: {message}")]
AuthenticationError { message: String },
```

### 4. Rate Limiting (rate_limit.rs)

Modify `extract_client_id()` to prioritize API key hash from request extensions.

## Configuration

### Environment Variables

```bash
# Authentication
REQUIRE_AUTH=true                  # Enable/disable authentication
API_KEYS=rtk_legacy1,rtk_legacy2   # Legacy: Comma-separated keys (for migration)

# Redis
REDIS_URL=redis://:password@localhost:6379

# Security
ENVIRONMENT=production             # Enable HTTPS enforcement
```

### Redis Schema

```
# API key storage
api_key:{hash} -> JSON(ApiKey)

# Active keys set
api_keys:active -> Set<hash1, hash2, hash3>

# Audit logs
audit:auth:2025-11-02 -> List<AuthEvent>

# Rate limiting (integrated)
rate_limit:{key_hash} -> Counter
```

## Testing Strategy

### Unit Tests
- `auth/generator.rs` - Key generation, hashing
- `auth/storage.rs` - CRUD operations, expiration
- `auth/types.rs` - Serialization, validation

### Integration Tests
- `auth_middleware` - Full auth flow
- Admin endpoints - Key management
- Rate limiting - Per-key limits

### Performance Tests
- Load testing: 10,000+ req/s
- Latency: < 10ms auth overhead
- Concurrency: 100+ parallel requests

### Security Tests
- Brute force protection
- Information disclosure
- Error handling
- Log sanitization

## Rollout Plan

### Development
1. Implement all phases
2. Run full test suite
3. Code review

### Staging
1. Deploy to staging environment
2. Test with real Redis instance
3. Performance testing
4. Security scanning

### Production
1. Deploy Redis changes first
2. Enable authentication (REQUIRE_AUTH=true)
3. Migrate existing env-var keys to Redis
4. Monitor authentication metrics
5. Gradual rollout to production traffic

### Rollback Procedure
1. Set `REQUIRE_AUTH=false`
2. Fall back to environment variable keys
3. Investigate issues
4. Fix and redeploy

## Success Criteria

- [ ] All unit tests passing (>80% coverage)
- [ ] All integration tests passing
- [ ] Performance benchmarks met (10k req/s, <10ms latency)
- [ ] Security tests passing
- [ ] Zero downtime deployment
- [ ] Documentation complete
- [ ] Migration guide tested

## Risks and Mitigation

| Risk | Impact | Mitigation |
|------|--------|------------|
| Redis unavailability | HIGH | Implement fallback to deny-all mode |
| Performance degradation | MEDIUM | Load testing, caching, optimization |
| Key compromise | HIGH | Audit logging, rotation mechanism |
| Migration issues | MEDIUM | Backward compatibility, staged rollout |
| Security vulnerabilities | HIGH | Security review, penetration testing |

## Support and Maintenance

### Monitoring
- Authentication success/failure rates
- Key usage metrics per API key
- Brute force attempt detection
- Redis connection health

### Alerts
- High authentication failure rate (>5%)
- Brute force attack detected
- Redis connection failures
- Suspicious activity patterns

### Maintenance Tasks
- Review audit logs weekly
- Rotate keys every 90 days
- Update dependencies monthly
- Security review quarterly

## References

- [Authentication Architecture](./authentication-architecture.md)
- [Authentication Security Strategy](./authentication-security.md)
- [Existing Auth Middleware](../crates/riptide-api/src/middleware/auth.rs)
- [Existing Rate Limiting](../crates/riptide-api/src/middleware/rate_limit.rs)

---

**Last Updated:** 2025-11-02
**Next Review:** After Phase 1 completion
**Owner:** Implementation Team
