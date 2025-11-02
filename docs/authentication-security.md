# Authentication Security Strategy for Riptide API

**Version:** 1.0
**Date:** 2025-11-02
**Status:** Design Phase
**Author:** Security Architect

## Security Objectives

1. **Confidentiality:** API keys remain secret, never exposed in logs or errors
2. **Integrity:** API keys cannot be forged or tampered with
3. **Availability:** Authentication system has minimal performance impact
4. **Auditability:** All authentication events are logged for security monitoring
5. **Defense in Depth:** Multiple layers of security controls

## Threat Model

### Identified Threats

| Threat | Severity | Mitigation |
|--------|----------|------------|
| API key theft via network interception | HIGH | Require HTTPS in production |
| Brute force key guessing | HIGH | 256-bit key space, rate limiting |
| API key leakage in logs | MEDIUM | Redact keys in all log outputs |
| Compromised Redis instance | HIGH | Key hashing, network isolation |
| Replay attacks | MEDIUM | Optional timestamp validation |
| Denial of service via auth | MEDIUM | Rate limiting, circuit breakers |
| Credential stuffing | LOW | No password-based auth |
| Social engineering | MEDIUM | Key rotation, audit logging |

### Attack Vectors

1. **Network-based:**
   - Man-in-the-middle attacks (MITM)
   - Packet sniffing on unencrypted connections

2. **Application-based:**
   - Log file harvesting
   - Error message information disclosure
   - Debug endpoint exposure

3. **Infrastructure-based:**
   - Redis database compromise
   - Container escape
   - Memory dumping

4. **Social-based:**
   - Phishing for API keys
   - Insider threats
   - Third-party compromise

## Security Controls

### 1. API Key Generation

#### Requirements

- **Entropy:** 256 bits of cryptographic randomness
- **Format:** Base64URL encoding for URL-safe transmission
- **Prefix:** `rtk_` prefix for easy identification (Riptide Key)
- **Length:** 43 characters (32 bytes base64-encoded + 4 char prefix)

#### Implementation

```rust
use rand::RngCore;
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};

/// Generate a cryptographically secure API key
pub fn generate_api_key() -> String {
    let mut key_bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut key_bytes);

    let encoded = URL_SAFE_NO_PAD.encode(key_bytes);
    format!("rtk_{}", encoded)
}
```

**Security Properties:**
- Uses OS-provided CSPRNG via `rand::thread_rng()`
- 2^256 possible keys (computationally infeasible to brute force)
- URL-safe encoding prevents transmission issues
- Prefix prevents accidental confusion with other tokens

### 2. Key Storage and Validation

#### Hashing Strategy

**Never store raw API keys.** Only store SHA-256 hashes for validation.

```rust
use sha2::{Sha256, Digest};

/// Hash an API key for storage
pub fn hash_api_key(key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(key.as_bytes());
    hex::encode(hasher.finalize())
}
```

**Why SHA-256:**
- One-way function: Cannot derive original key from hash
- Fast validation: < 1ms computation time
- Collision resistant: Computationally infeasible to find two keys with same hash
- Industry standard: Well-tested and audited

#### Redis Storage Security

**Storage Pattern:**

```
# Key-value pairs
api_key:{hash} -> {
    "key_hash": "abc123...",
    "description": "Production API key for service X",
    "created_at": "2025-11-02T12:00:00Z",
    "last_used": "2025-11-02T12:30:00Z",
    "scopes": ["read", "write"],
    "is_active": true,
    "expires_at": null
}

# Active keys set (for quick lookup)
api_keys:active -> Set<hash1, hash2, hash3, ...>
```

**Security Measures:**

1. **No Raw Keys:** Only hashes stored, original keys destroyed after generation
2. **TTL-based Expiration:** Optional expiration for temporary keys
3. **Active Flag:** Instant key revocation without deletion
4. **Redis AUTH:** Require password for Redis connection (via connection URL)
5. **Network Isolation:** Redis should not be publicly accessible

#### Key Validation Process

```rust
pub async fn validate_api_key(
    key: &str,
    storage: &ApiKeyStore,
) -> Result<ApiKey, AuthError> {
    // 1. Hash the provided key
    let key_hash = hash_api_key(key);

    // 2. Lookup in Redis
    let api_key = storage.get(&key_hash).await?;

    // 3. Check active flag
    if !api_key.is_active {
        return Err(AuthError::KeyInactive);
    }

    // 4. Check expiration
    if let Some(expires_at) = api_key.expires_at {
        if chrono::Utc::now() > expires_at {
            return Err(AuthError::KeyExpired);
        }
    }

    // 5. Update last_used (async, don't block validation)
    tokio::spawn(async move {
        let _ = storage.update_last_used(&key_hash).await;
    });

    Ok(api_key)
}
```

### 3. Transport Security

#### HTTPS Enforcement

**Production Requirement:** ALL API requests MUST use HTTPS.

**Implementation Options:**

1. **Application Layer (Axum Middleware):**

```rust
/// Middleware to enforce HTTPS in production
pub async fn https_enforcement_middleware(
    request: Request,
    next: Next,
) -> Result<Response, Response> {
    // Only enforce in production
    if std::env::var("ENVIRONMENT") == Ok("production".to_string()) {
        // Check X-Forwarded-Proto header (for reverse proxy setups)
        if let Some(proto) = request.headers().get("X-Forwarded-Proto") {
            if proto != "https" {
                return Err(ApiError::validation(
                    "HTTPS required in production"
                ).into_response());
            }
        } else if request.uri().scheme_str() != Some("https") {
            return Err(ApiError::validation(
                "HTTPS required in production"
            ).into_response());
        }
    }

    Ok(next.run(request).await)
}
```

2. **Infrastructure Layer (Recommended):**
   - Use reverse proxy (nginx, Traefik) for TLS termination
   - Let infrastructure handle HTTPS enforcement
   - Application trusts `X-Forwarded-Proto` header

**Security Headers:**

Add security headers to all responses:

```rust
// In main.rs middleware stack
.layer(
    tower_http::set_header::SetResponseHeaderLayer::overriding(
        header::STRICT_TRANSPORT_SECURITY,
        HeaderValue::from_static("max-age=31536000; includeSubDomains"),
    )
)
```

### 4. Audit Logging

#### Security Event Types

```rust
pub enum AuthEvent {
    KeyGenerated { key_hash: String, description: String },
    KeyValidationSuccess { key_hash: String, endpoint: String },
    KeyValidationFailure { reason: String, ip: String },
    KeyRevoked { key_hash: String, reason: String },
    KeyRotated { old_hash: String, new_hash: String },
    SuspiciousActivity { key_hash: String, reason: String },
}
```

#### Logging Requirements

1. **Success Events:** Log at INFO level
2. **Failure Events:** Log at WARN level
3. **Suspicious Activity:** Log at ERROR level
4. **Log Redaction:** NEVER log raw API keys

**Example Log Entry:**

```json
{
  "timestamp": "2025-11-02T12:34:56.789Z",
  "event_type": "KeyValidationSuccess",
  "key_hash": "abc123...",
  "endpoint": "/api/v1/extract",
  "ip": "192.168.1.100",
  "user_agent": "RiptideClient/1.0",
  "response_time_ms": 45
}
```

#### Suspicious Activity Detection

Monitor for:
- Multiple failed validations from same IP (> 10/min)
- Key used from multiple IPs simultaneously
- Unusual request patterns (sudden spike in traffic)
- Access to admin endpoints with non-admin keys

**Alerting Integration:**

```rust
// In auth middleware
if failed_attempts > THRESHOLD {
    state.telemetry.emit_alert(
        AlertSeverity::High,
        "Potential brute force attack detected",
        metadata,
    );
}
```

### 5. Rate Limiting Integration

#### Per-Key Rate Limits

Current rate limiting uses client IDs extracted from headers. Enhance to use authenticated API key:

```rust
// In rate_limit middleware
let client_id = if let Some(api_key_hash) = request.extensions().get::<ApiKeyHash>() {
    // Use authenticated key hash for rate limiting
    Some(api_key_hash.to_string())
} else {
    // Fall back to IP-based identification
    extract_client_id(&request)
};
```

**Benefits:**
- Consistent rate limiting across different IPs
- Fine-grained per-customer limits
- Better abuse prevention

#### Brute Force Protection

```rust
pub struct BruteForceProtection {
    failed_attempts: Arc<DashMap<String, (u32, Instant)>>,
}

impl BruteForceProtection {
    pub async fn check_and_record(&self, ip: &str) -> Result<(), AuthError> {
        let mut entry = self.failed_attempts.entry(ip.to_string())
            .or_insert((0, Instant::now()));

        // Reset counter if last attempt was > 1 minute ago
        if entry.1.elapsed() > Duration::from_secs(60) {
            *entry = (1, Instant::now());
            return Ok(());
        }

        // Increment counter
        entry.0 += 1;

        // Block if too many attempts
        if entry.0 > 10 {
            Err(AuthError::TooManyAttempts)
        } else {
            Ok(())
        }
    }
}
```

### 6. Key Rotation

#### Rotation Strategy

**Recommended Rotation Policy:**
- Automatic rotation every 90 days
- Manual rotation on security incidents
- Zero-downtime rotation with grace period

**Implementation:**

```rust
pub async fn rotate_api_key(
    old_key_hash: &str,
    storage: &ApiKeyStore,
) -> Result<String, AuthError> {
    // 1. Generate new key
    let new_key = generate_api_key();
    let new_key_hash = hash_api_key(&new_key);

    // 2. Copy metadata from old key
    let old_metadata = storage.get(old_key_hash).await?;
    let new_metadata = ApiKey {
        key_hash: new_key_hash.clone(),
        description: old_metadata.description,
        created_at: chrono::Utc::now().to_rfc3339(),
        last_used: None,
        scopes: old_metadata.scopes,
        is_active: true,
        expires_at: None,
    };

    // 3. Store new key
    storage.save(&new_key_hash, &new_metadata).await?;

    // 4. Mark old key for deletion after grace period
    storage.schedule_deletion(old_key_hash, Duration::from_secs(86400)).await?;

    // 5. Audit log
    audit_log(AuthEvent::KeyRotated {
        old_hash: old_key_hash.to_string(),
        new_hash: new_key_hash,
    });

    // Return new key (only time it's visible)
    Ok(new_key)
}
```

### 7. Secret Management

#### Environment Variables

**Never commit API keys to git.**

```bash
# .env (git-ignored)
API_KEYS=rtk_abc123...,rtk_def456...
REQUIRE_AUTH=true
REDIS_URL=redis://:password@localhost:6379
```

**Production Deployment:**

Use secret management services:
- **Kubernetes:** Sealed Secrets, External Secrets Operator
- **AWS:** AWS Secrets Manager, Parameter Store
- **Docker:** Docker Secrets
- **HashiCorp Vault:** Centralized secret management

#### Redis Connection Security

```bash
# Use authenticated Redis connection
REDIS_URL=redis://:strong_password@redis:6379

# Or with TLS
REDIS_URL=rediss://:strong_password@redis:6380
```

### 8. Error Handling

#### Information Disclosure Prevention

**Never reveal implementation details in error messages.**

❌ **BAD:**
```json
{
  "error": "API key 'rtk_abc123' not found in Redis at key 'api_key:hash123'"
}
```

✅ **GOOD:**
```json
{
  "error": "Unauthorized",
  "message": "Invalid API key"
}
```

**Implementation:**

```rust
fn unauthorized_response(message: &str) -> Response {
    Response::builder()
        .status(StatusCode::UNAUTHORIZED)
        .header("Content-Type", "application/json")
        .header("WWW-Authenticate", "Bearer realm=\"Riptide API\"")
        .body(Body::from(
            serde_json::json!({
                "error": "Unauthorized",
                "message": message,
            })
            .to_string(),
        ))
        .unwrap()
        .into_response()
}
```

#### Log Sanitization

```rust
// Redact API keys in logs
fn sanitize_for_logging(key: &str) -> String {
    if key.len() > 8 {
        format!("{}...{}", &key[..4], &key[key.len()-4..])
    } else {
        "***".to_string()
    }
}

// Usage
tracing::warn!(
    api_key = %sanitize_for_logging(&api_key),
    "Invalid API key attempted"
);
```

### 9. Authorization (Future Enhancement)

#### Scope-Based Access Control

While not required for Phase 1, the architecture supports future authorization:

```rust
pub struct ApiKey {
    // ... existing fields ...

    /// Authorization scopes
    pub scopes: Vec<String>,
}

// Example scopes:
// - "read": Read-only access
// - "write": Create/update operations
// - "admin": Key management and admin functions
```

**Middleware Integration:**

```rust
pub fn require_scope(required_scope: &str) -> impl Fn(Request, Next) -> Future<Response> {
    move |request, next| async move {
        let api_key = request.extensions().get::<ApiKey>()?;

        if !api_key.scopes.contains(&required_scope.to_string()) {
            return Err(ApiError::forbidden("Insufficient permissions"));
        }

        Ok(next.run(request).await)
    }
}

// Usage in routes
.route("/admin/keys", post(create_key).layer(middleware::from_fn(require_scope("admin"))))
```

### 10. Compliance Considerations

#### Data Protection

- **GDPR:** API keys may be considered personal data if tied to individuals
- **Retention:** Implement configurable retention policies for audit logs
- **Right to Deletion:** Support key deletion requests

#### Security Standards

- **OWASP API Security Top 10:** Address all relevant items
- **PCI DSS:** If handling payment data, ensure compliance
- **SOC 2:** Implement controls for audit, logging, and access management

### 11. Incident Response

#### Security Incident Procedures

1. **Key Compromise:**
   - Immediately revoke compromised key
   - Generate and distribute new key
   - Review audit logs for unauthorized access
   - Notify affected customers

2. **Brute Force Attack:**
   - Enable IP-based blocking
   - Increase rate limiting threshold
   - Review and enhance monitoring

3. **Redis Compromise:**
   - Rotate all API keys immediately
   - Audit all recent authentication events
   - Investigate extent of data exposure
   - Enhance infrastructure security

## Security Testing

### Testing Requirements

1. **Unit Tests:**
   - Key generation randomness
   - Hash collision resistance
   - Validation logic correctness

2. **Integration Tests:**
   - End-to-end authentication flow
   - Redis failure scenarios
   - Rate limiting effectiveness

3. **Security Tests:**
   - Brute force resistance
   - Information disclosure checks
   - Error handling validation

4. **Penetration Testing:**
   - Periodic security audits
   - Third-party assessment
   - Vulnerability scanning

## Security Checklist

- [ ] API keys are 256-bit random values
- [ ] Keys are hashed (SHA-256) before storage
- [ ] Raw keys never logged or exposed in errors
- [ ] HTTPS enforced in production
- [ ] Rate limiting per API key implemented
- [ ] Brute force protection active
- [ ] Audit logging for all auth events
- [ ] Redis connection authenticated
- [ ] Redis not publicly accessible
- [ ] Key rotation mechanism implemented
- [ ] Secure secret management in production
- [ ] Error messages don't leak information
- [ ] Security headers configured
- [ ] Monitoring and alerting active
- [ ] Incident response procedures documented

## References

- **OWASP API Security Project:** https://owasp.org/www-project-api-security/
- **NIST Password Guidelines:** https://pages.nist.gov/800-63-3/
- **Redis Security:** https://redis.io/topics/security
- **Rust Crypto Guidelines:** https://cryptography.rs/

---

**Last Updated:** 2025-11-02
**Next Review:** 2025-12-02
**Owner:** Security Team
