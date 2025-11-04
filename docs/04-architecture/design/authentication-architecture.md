# Authentication Architecture for Riptide API

**Version:** 1.0
**Date:** 2025-11-02
**Status:** Design Phase
**Author:** System Architect

## Executive Summary

This document outlines the authentication architecture for the Riptide API, building upon the existing `middleware/auth.rs` implementation. The design focuses on simplicity, security, and integration with the existing Axum-based infrastructure without multi-tenant complexity.

## Current State Analysis

### Existing Implementation

The codebase already has a foundational authentication system in place at `/workspaces/eventmesh/crates/riptide-api/src/middleware/auth.rs`:

**Current Features:**
- ✅ API key validation via `X-API-Key` header
- ✅ Bearer token support via `Authorization` header
- ✅ Environment-based configuration (`API_KEYS`, `REQUIRE_AUTH`)
- ✅ Public path exemptions (`/health`, `/metrics`)
- ✅ Axum middleware integration
- ✅ Integration with `AppState`
- ✅ Error handling with proper HTTP status codes

**Gaps Identified:**
- ❌ No secure API key generation mechanism
- ❌ No key rotation capability
- ❌ No persistence layer for API keys (currently env-var only)
- ❌ No audit logging for authentication events
- ❌ No authorization levels or scopes
- ❌ Rate limiting uses client ID but not tied to auth context

## Architecture Design

### 1. System Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     Client Request                           │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│              Axum Router + Middleware Stack                  │
├─────────────────────────────────────────────────────────────┤
│  1. TraceLayer (Logging)                                     │
│  2. TimeoutLayer (Request timeout)                           │
│  3. CompressionLayer (Response compression)                  │
│  4. CorsLayer (CORS handling)                                │
│  5. PayloadLimitLayer (Request size limits)                  │
│  ┌──────────────────────────────────────────────────────┐  │
│  │ 6. AUTH MIDDLEWARE (auth_middleware)                  │  │
│  │    - Extract API key from headers                     │  │
│  │    - Validate against storage                         │  │
│  │    - Check authorization scope                        │  │
│  │    - Audit log authentication event                   │  │
│  └──────────────────────────────────────────────────────┘  │
│  7. RateLimitLayer (Client-aware rate limiting)             │
│  8. RequestValidationLayer (Input validation)               │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│                   Handler Functions                          │
└─────────────────────────────────────────────────────────────┘
```

### 2. Component Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Authentication Module                     │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌──────────────────────────────────────────────────────┐  │
│  │           AuthMiddleware (middleware/auth.rs)         │  │
│  │  - Request interception                               │  │
│  │  - Header extraction (X-API-Key, Authorization)       │  │
│  │  - Public path bypass logic                           │  │
│  │  - Error response generation                          │  │
│  └────────────────┬─────────────────────────────────────┘  │
│                   │                                          │
│                   ▼                                          │
│  ┌──────────────────────────────────────────────────────┐  │
│  │        AuthConfig (middleware/auth.rs)                │  │
│  │  - Configuration management                           │  │
│  │  - Public path definitions                            │  │
│  │  - Auth requirement flag                              │  │
│  └────────────────┬─────────────────────────────────────┘  │
│                   │                                          │
│                   ▼                                          │
│  ┌──────────────────────────────────────────────────────┐  │
│  │         ApiKeyStore (NEW: auth/storage.rs)            │  │
│  │  - Redis-backed API key storage                       │  │
│  │  - Key validation and lookup                          │  │
│  │  - Key metadata (created_at, last_used, scopes)      │  │
│  │  - Key rotation support                               │  │
│  └────────────────┬─────────────────────────────────────┘  │
│                   │                                          │
│                   ▼                                          │
│  ┌──────────────────────────────────────────────────────┐  │
│  │      AuthAuditLogger (NEW: auth/audit.rs)             │  │
│  │  - Authentication event logging                       │  │
│  │  - Integration with TelemetrySystem                   │  │
│  │  - Failed attempt tracking                            │  │
│  │  - Success audit trail                                │  │
│  └──────────────────────────────────────────────────────┘  │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### 3. Data Model

#### API Key Structure

```rust
/// API key metadata stored in Redis
pub struct ApiKey {
    /// Unique key identifier (SHA-256 hash of actual key)
    pub key_hash: String,

    /// Human-readable description
    pub description: String,

    /// Timestamp when key was created (ISO 8601)
    pub created_at: String,

    /// Timestamp when key was last used (ISO 8601)
    pub last_used: Option<String>,

    /// Authorization scopes (for future use)
    /// Examples: "read", "write", "admin"
    pub scopes: Vec<String>,

    /// Whether the key is active
    pub is_active: bool,

    /// Optional expiration timestamp (ISO 8601)
    pub expires_at: Option<String>,
}
```

#### Redis Storage Schema

```
# API key hash -> metadata mapping
api_key:{hash} -> JSON serialized ApiKey

# Active keys set (for quick validation)
api_keys:active -> Set of active key hashes

# Audit log entries (time-series)
audit:auth:{date} -> List of audit events

# Rate limiting integration
rate_limit:{key_hash} -> Current rate limit counter
```

### 4. Authentication Flow

```
┌─────────────┐
│   Request   │
└──────┬──────┘
       │
       ▼
┌─────────────────────────────────────┐
│ Is path in public_paths?            │
└──────┬──────────────────────────────┘
       │
  YES  │  NO
   ┌───┴────┐
   │        │
   │        ▼
   │   ┌─────────────────────────────────────┐
   │   │ Extract API key from headers        │
   │   │ (X-API-Key or Authorization)        │
   │   └──────┬──────────────────────────────┘
   │          │
   │          ▼
   │   ┌─────────────────────────────────────┐
   │   │ API key present?                    │
   │   └──────┬──────────────────────────────┘
   │          │
   │     NO   │  YES
   │      ┌───┴────┐
   │      │        │
   │      │        ▼
   │      │   ┌─────────────────────────────────────┐
   │      │   │ Hash key (SHA-256)                  │
   │      │   └──────┬──────────────────────────────┘
   │      │          │
   │      │          ▼
   │      │   ┌─────────────────────────────────────┐
   │      │   │ Lookup in Redis storage             │
   │      │   └──────┬──────────────────────────────┘
   │      │          │
   │      │     FOUND│  NOT FOUND / EXPIRED
   │      │      ┌───┴────┐
   │      │      │        │
   │      │      │        ▼
   │      │      │   ┌─────────────────────────────────────┐
   │      │      │   │ Log failed attempt                  │
   │      │      │   │ Return 401 Unauthorized             │
   │      │      │   └─────────────────────────────────────┘
   │      │      │
   │      │      ▼
   │      │ ┌─────────────────────────────────────┐
   │      │ │ Check is_active flag                │
   │      │ └──────┬──────────────────────────────┘
   │      │        │
   │      │  ACTIVE│  INACTIVE
   │      │    ┌───┴────┐
   │      │    │        │
   │      │    │        ▼
   │      │    │   ┌─────────────────────────────────────┐
   │      │    │   │ Log inactive key attempt            │
   │      │    │   │ Return 401 Unauthorized             │
   │      │    │   └─────────────────────────────────────┘
   │      │    │
   │      │    ▼
   │      │ ┌─────────────────────────────────────┐
   │      │ │ Update last_used timestamp          │
   │      │ │ Log successful authentication       │
   │      │ │ Attach key metadata to request ext  │
   │      │ └──────┬──────────────────────────────┘
   │      │        │
   │      ▼        ▼
   │   ┌─────────────────────────────────────┐
   │   │ Return 401 Missing API key          │
   │   └─────────────────────────────────────┘
   │
   ▼
┌─────────────────────────────────────┐
│ Proceed to next middleware          │
└─────────────────────────────────────┘
```

### 5. Integration Points

#### 5.1 State Integration

The authentication system integrates with `AppState`:

```rust
pub struct AppState {
    // ... existing fields ...

    /// Authentication configuration
    pub auth_config: AuthConfig,

    // ... other fields ...
}
```

#### 5.2 Rate Limiting Integration

Current rate limiting middleware in `middleware/rate_limit.rs` extracts client IDs from:
1. `X-Client-ID` header
2. `X-API-Key` header
3. `X-Forwarded-For` header
4. `X-Real-IP` header

**Enhancement:** After authentication, the API key hash should be injected into request extensions for consistent client identification across rate limiting.

#### 5.3 Telemetry Integration

Authentication events integrate with existing `TelemetrySystem`:

```rust
// Log authentication events to OpenTelemetry if enabled
if let Some(telemetry) = &state.telemetry {
    telemetry.log_auth_event(event);
}
```

#### 5.4 Error Handling Integration

Authentication errors use existing `ApiError` enum:

```rust
// Defined in errors.rs
#[error("Authentication failed: {message}")]
AuthenticationError { message: String }
```

### 6. File Structure

```
crates/riptide-api/src/
├── middleware/
│   ├── mod.rs                  # Export auth module
│   ├── auth.rs                 # EXISTING: Core auth middleware
│   ├── auth/                   # NEW: Auth submodule
│   │   ├── mod.rs              # Module exports
│   │   ├── storage.rs          # API key storage (Redis)
│   │   ├── generator.rs        # Secure key generation
│   │   ├── audit.rs            # Authentication audit logging
│   │   └── types.rs            # ApiKey struct and types
│   ├── rate_limit.rs           # EXISTING: Rate limiting
│   └── request_validation.rs   # EXISTING: Input validation
├── handlers/
│   └── admin.rs                # NEW: Key management endpoints
└── routes/
    └── admin.rs                # NEW: Admin routes
```

## Technology Stack

### Core Dependencies

- **Axum** (existing): Web framework and middleware
- **Redis** (existing): API key storage via `riptide-cache::CacheManager`
- **SHA-256** (new: via `sha2` crate): Key hashing
- **OpenTelemetry** (existing): Audit logging via `TelemetrySystem`
- **Tracing** (existing): Debug and info logging

### New Dependencies Required

Add to `Cargo.toml`:

```toml
[dependencies]
sha2 = "0.10"           # For secure key hashing
```

## Non-Functional Requirements

### Performance

- **API Key Validation:** < 10ms (Redis lookup)
- **Key Generation:** < 50ms (cryptographic operation)
- **Middleware Overhead:** < 5ms per request
- **Redis Connection Pooling:** Reuse existing connection pool

### Scalability

- **Concurrent Validations:** 10,000+ req/s (Redis capability)
- **Key Storage:** Unlimited keys (Redis-backed)
- **Audit Log Retention:** Configurable, default 90 days

### Security

- **Key Strength:** 256-bit random keys (32 bytes)
- **Storage Security:** Keys stored as SHA-256 hashes
- **Transport Security:** HTTPS required in production
- **Audit Trail:** All auth events logged with timestamps

### Reliability

- **Redis Availability:** Fallback to deny-all if Redis unavailable
- **Graceful Degradation:** Option to disable auth for emergency access
- **Key Rotation:** Zero-downtime key rotation support

## Security Considerations

See detailed security strategy in `authentication-security.md`.

## Future Enhancements

### Phase 2 (Not in Current Scope)

1. **Authorization Scopes**
   - Fine-grained permissions (read, write, admin)
   - Endpoint-level authorization checks

2. **Multi-Tier Rate Limiting**
   - Different rate limits per API key tier
   - Burst allowance for premium keys

3. **Key Usage Analytics**
   - Dashboard for key usage metrics
   - Cost tracking per API key

4. **API Key Revocation Lists**
   - Emergency key revocation
   - Blacklist suspicious keys

5. **JWT Token Support**
   - Short-lived session tokens
   - Refresh token mechanism

## References

- **Existing Implementation:** `/workspaces/eventmesh/crates/riptide-api/src/middleware/auth.rs`
- **Rate Limiting:** `/workspaces/eventmesh/crates/riptide-api/src/middleware/rate_limit.rs`
- **Error Handling:** `/workspaces/eventmesh/crates/riptide-api/src/errors.rs`
- **Application State:** `/workspaces/eventmesh/crates/riptide-api/src/state.rs`
- **Main Router:** `/workspaces/eventmesh/crates/riptide-api/src/main.rs`

## Approval and Sign-off

- [ ] Architecture Review
- [ ] Security Review
- [ ] Implementation Team Handoff
- [ ] Documentation Complete

---

**Next Steps:** See `authentication-security.md` for security strategy and `implementation-checklist` in memory for detailed implementation plan.
