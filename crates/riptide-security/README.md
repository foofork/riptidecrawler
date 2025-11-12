# 🛠️ RipTide Security - Security & Authentication

**Category:** Security & Authentication
**Purpose:** API key authentication, JWT support, rate limiting, input validation, and security middleware

## Quick Overview

`riptide-security` provides comprehensive security features for the RipTide web scraping framework. It includes API key authentication, JWT token validation, per-key rate limiting, request validation, CORS configuration, and security headers for production deployments.

## Why This Exists

Production web scraping APIs need robust security:
- Authenticate and authorize API requests
- Prevent abuse with rate limiting
- Validate and sanitize inputs
- Configure CORS properly
- Set security headers (HSTS, CSP, etc.)
- Audit security events

This crate centralizes all security concerns in one tested, auditable module.

## Key Features

- **API Key Authentication**: Secure API key validation with hashing
- **JWT Support**: JSON Web Token authentication and validation
- **Rate Limiting**: Per-key and IP-based rate limiting
- **Input Validation**: Request payload validation and sanitization
- **CORS**: Cross-origin resource sharing configuration
- **Security Headers**: HSTS, CSP, X-Frame-Options, etc.
- **Request Filtering**: Block malicious requests
- **Audit Logging**: Security event logging

## Quick Start

```rust
use riptide_security::{ApiKeyAuth, RateLimiter};
use std::time::Duration;

// Create authenticator
let auth = ApiKeyAuth::new();

// Validate API key
if auth.validate_key(&request_key).await? {
    // Request is authorized
} else {
    return Err("Unauthorized");
}

// Create rate limiter
let limiter = RateLimiter::new(100, Duration::from_secs(60));

// Check rate limit
if limiter.check_limit(&client_id).await? {
    // Allow request
} else {
    return Err("Rate limit exceeded");
}
```

## API Key Authentication

Secure API key management and validation:

### Creating API Keys

```rust
use riptide_security::{ApiKeyAuth, ApiKey};

let auth = ApiKeyAuth::new();

// Generate new API key
let api_key = auth.create_key("user_123").await?;

println!("API Key: {}", api_key.key);
println!("Secret: {}", api_key.secret);

// Store key (hashed) in database
// The secret should be shown to user only once
```

### Validating API Keys

```rust
use riptide_security::ApiKeyAuth;

let auth = ApiKeyAuth::new();

// Validate from request header
let key = extract_api_key_from_header(&request)?;

match auth.validate_key(&key).await {
    Ok(user_id) => {
        // Key is valid, proceed with request
        println!("Authenticated as: {}", user_id);
    }
    Err(_) => {
        // Invalid key, return 401 Unauthorized
        return Err("Invalid API key");
    }
}
```

### Revoking API Keys

```rust
use riptide_security::ApiKeyAuth;

let auth = ApiKeyAuth::new();

// Revoke a key
auth.revoke_key(&key_id).await?;

// List all keys for a user
let keys = auth.list_keys("user_123").await?;
for key in keys {
    println!("Key: {}, Created: {}", key.id, key.created_at);
}
```

## JWT Authentication

JSON Web Token support for stateless authentication:

### Creating JWT Tokens

```rust
use riptide_security::{JwtAuth, Claims};
use std::time::Duration;

let jwt = JwtAuth::new("your-secret-key");

// Create claims
let claims = Claims {
    sub: "user_123".to_string(),
    exp: (chrono::Utc::now() + Duration::from_secs(3600)).timestamp(),
    iat: chrono::Utc::now().timestamp(),
};

// Generate token
let token = jwt.create_token(&claims)?;
println!("Token: {}", token);
```

### Validating JWT Tokens

```rust
use riptide_security::JwtAuth;

let jwt = JwtAuth::new("your-secret-key");

// Extract token from Authorization header
let token = extract_bearer_token(&request)?;

// Validate and decode
match jwt.validate_token(&token) {
    Ok(claims) => {
        println!("User: {}", claims.sub);
        println!("Expires: {}", claims.exp);
    }
    Err(e) => {
        return Err(format!("Invalid token: {}", e));
    }
}
```

### Refreshing Tokens

```rust
use riptide_security::{JwtAuth, Claims};

let jwt = JwtAuth::new("your-secret-key");

// Validate old token
let old_claims = jwt.validate_token(&old_token)?;

// Issue new token with same claims
let new_claims = Claims {
    sub: old_claims.sub,
    exp: (chrono::Utc::now() + Duration::from_secs(3600)).timestamp(),
    iat: chrono::Utc::now().timestamp(),
};

let new_token = jwt.create_token(&new_claims)?;
```

## Rate Limiting

Prevent API abuse with flexible rate limiting:

### Per-Client Rate Limiting

```rust
use riptide_security::{RateLimiter, RateLimitConfig};
use std::time::Duration;

// Create rate limiter
// 100 requests per minute
let limiter = RateLimiter::new(100, Duration::from_secs(60));

// Check rate limit for client
let client_id = get_client_id(&request);

match limiter.check_limit(&client_id).await {
    Ok(true) => {
        // Request allowed
    }
    Ok(false) => {
        // Rate limit exceeded
        return Err("Rate limit exceeded, try again later");
    }
    Err(e) => {
        return Err(format!("Rate limiter error: {}", e));
    }
}
```

### Per-API-Key Rate Limiting

```rust
use riptide_security::{RateLimiter, ApiKeyAuth};

let auth = ApiKeyAuth::new();
let limiter = RateLimiter::new(1000, Duration::from_secs(3600)); // 1000/hour

// Validate key and check rate limit
let api_key = extract_api_key(&request)?;
let user_id = auth.validate_key(&api_key).await?;

if !limiter.check_limit(&user_id).await? {
    return Err("Rate limit exceeded for this API key");
}
```

### Custom Rate Limit Rules

```rust
use riptide_security::{RateLimiter, RateLimitRule};

let mut limiter = RateLimiter::with_rules(vec![
    // 10 requests per second burst
    RateLimitRule::new(10, Duration::from_secs(1)),
    // 100 requests per minute sustained
    RateLimitRule::new(100, Duration::from_secs(60)),
    // 1000 requests per hour maximum
    RateLimitRule::new(1000, Duration::from_secs(3600)),
]);

// All rules must pass
if !limiter.check_all_rules(&client_id).await? {
    return Err("Rate limit exceeded");
}
```

### Getting Rate Limit Status

```rust
use riptide_security::RateLimiter;

let limiter = RateLimiter::new(100, Duration::from_secs(60));

// Get status for client
let status = limiter.get_status(&client_id).await?;

println!("Requests used: {}/{}", status.count, status.limit);
println!("Resets in: {} seconds", status.reset_in_secs);
println!("Remaining: {}", status.remaining);

// Add to response headers
response.headers_mut().insert(
    "X-RateLimit-Limit",
    status.limit.to_string().parse().unwrap(),
);
response.headers_mut().insert(
    "X-RateLimit-Remaining",
    status.remaining.to_string().parse().unwrap(),
);
```

## Input Validation

Validate and sanitize request inputs:

### URL Validation

```rust
use riptide_security::Validator;

let validator = Validator::new();

// Validate URL
match validator.validate_url(&url) {
    Ok(_) => println!("URL is valid"),
    Err(e) => return Err(format!("Invalid URL: {}", e)),
}

// Check for dangerous schemes
if url.starts_with("javascript:") || url.starts_with("data:") {
    return Err("Dangerous URL scheme");
}
```

### Payload Validation

```rust
use riptide_security::{Validator, ValidationRules};

let rules = ValidationRules {
    max_body_size: 10 * 1024 * 1024, // 10MB
    allowed_content_types: vec![
        "application/json".to_string(),
        "application/x-www-form-urlencoded".to_string(),
    ],
    ..Default::default()
};

let validator = Validator::with_rules(rules);

// Validate request
if let Err(e) = validator.validate_request(&request) {
    return Err(format!("Invalid request: {}", e));
}
```

### Input Sanitization

```rust
use riptide_security::sanitize;

// Sanitize HTML
let safe_html = sanitize::html(&untrusted_html);

// Sanitize SQL (escape)
let safe_sql = sanitize::sql_string(&user_input);

// Sanitize URL parameters
let safe_params = sanitize::url_params(&params);

// Strip dangerous characters
let safe_filename = sanitize::filename(&user_filename);
```

## CORS Configuration

Configure Cross-Origin Resource Sharing:

```rust
use riptide_security::{CorsConfig, CorsMiddleware};

// Create CORS config
let cors = CorsConfig {
    allowed_origins: vec![
        "https://example.com".to_string(),
        "https://app.example.com".to_string(),
    ],
    allowed_methods: vec!["GET", "POST", "PUT", "DELETE"],
    allowed_headers: vec!["Content-Type", "Authorization"],
    expose_headers: vec!["X-Total-Count"],
    max_age: 3600,
    allow_credentials: true,
};

// Apply as middleware
let middleware = CorsMiddleware::new(cors);

// In your HTTP server
// app.layer(middleware)
```

### Wildcard Origins (Development Only)

```rust
use riptide_security::CorsConfig;

// Allow all origins (development only!)
let cors = CorsConfig::allow_all();

// Or specific patterns
let cors = CorsConfig {
    allowed_origins: vec!["*.example.com".to_string()],
    ..Default::default()
};
```

## Security Headers

Set recommended security headers:

```rust
use riptide_security::SecurityHeaders;

let headers = SecurityHeaders::default();

// Apply to response
headers.apply_to_response(&mut response);

// Headers set:
// - Strict-Transport-Security
// - X-Content-Type-Options
// - X-Frame-Options
// - X-XSS-Protection
// - Content-Security-Policy
// - Referrer-Policy
// - Permissions-Policy
```

### Custom Security Headers

```rust
use riptide_security::SecurityHeaders;

let headers = SecurityHeaders {
    hsts_max_age: 31536000, // 1 year
    hsts_include_subdomains: true,
    hsts_preload: true,
    csp_policy: Some("default-src 'self'; script-src 'self' 'unsafe-inline'".to_string()),
    frame_options: "DENY".to_string(),
    ..Default::default()
};
```

## Request Filtering

Block malicious requests:

```rust
use riptide_security::{RequestFilter, FilterRule};

let filter = RequestFilter::new(vec![
    // Block known bad IPs
    FilterRule::BlockIp("192.0.2.0/24".to_string()),

    // Block user agents
    FilterRule::BlockUserAgent("BadBot".to_string()),

    // Require specific headers
    FilterRule::RequireHeader("X-API-Version".to_string()),

    // Block by path pattern
    FilterRule::BlockPath("/admin/*".to_string()),
]);

// Apply filter
if let Err(reason) = filter.check_request(&request).await {
    return Err(format!("Request blocked: {}", reason));
}
```

## Audit Logging

Log security events for auditing:

```rust
use riptide_security::{AuditLogger, AuditEvent};

let logger = AuditLogger::new();

// Log authentication events
logger.log(AuditEvent::Authentication {
    user_id: "user_123".to_string(),
    success: true,
    ip_address: "192.0.2.1".to_string(),
    timestamp: chrono::Utc::now(),
}).await;

// Log authorization failures
logger.log(AuditEvent::AuthorizationFailure {
    user_id: "user_123".to_string(),
    resource: "/admin/users".to_string(),
    ip_address: "192.0.2.1".to_string(),
    timestamp: chrono::Utc::now(),
}).await;

// Log rate limit exceeded
logger.log(AuditEvent::RateLimitExceeded {
    client_id: "api_key_123".to_string(),
    endpoint: "/api/extract".to_string(),
    timestamp: chrono::Utc::now(),
}).await;
```

## Complete Middleware Example

Combining all security features:

```rust
use riptide_security::{
    ApiKeyAuth, RateLimiter, Validator, CorsMiddleware,
    SecurityHeaders, RequestFilter, AuditLogger,
};
use axum::{
    middleware::{self, Next},
    http::{Request, Response},
};

// Security middleware
async fn security_middleware<B>(
    req: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    // 1. Apply CORS
    let cors = CorsMiddleware::default();
    cors.handle(&req)?;

    // 2. Filter malicious requests
    let filter = RequestFilter::default();
    filter.check_request(&req).await?;

    // 3. Validate API key
    let auth = ApiKeyAuth::new();
    let api_key = extract_api_key(&req)?;
    let user_id = auth.validate_key(&api_key).await?;

    // 4. Check rate limit
    let limiter = RateLimiter::new(100, Duration::from_secs(60));
    if !limiter.check_limit(&user_id).await? {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    // 5. Validate input
    let validator = Validator::default();
    validator.validate_request(&req)?;

    // 6. Log event
    let logger = AuditLogger::new();
    logger.log_access(&user_id, req.uri().path()).await;

    // 7. Process request
    let mut response = next.run(req).await;

    // 8. Add security headers
    let headers = SecurityHeaders::default();
    headers.apply_to_response(&mut response);

    Ok(response)
}
```

## Integration with Axum

Example integration with Axum web framework:

```rust
use axum::{
    routing::post,
    Router,
    middleware,
};
use riptide_security::{ApiKeyAuth, SecurityHeaders};

async fn extract_handler(/* ... */) -> Result<Json<Response>, StatusCode> {
    // Handler logic
}

let auth = ApiKeyAuth::new();
let headers = SecurityHeaders::default();

let app = Router::new()
    .route("/api/extract", post(extract_handler))
    .layer(middleware::from_fn(move |req, next| {
        security_middleware(req, next, auth.clone(), headers.clone())
    }));
```

## Testing

```bash
# Run all tests
cargo test -p riptide-security

# Test authentication
cargo test -p riptide-security auth

# Test rate limiting
cargo test -p riptide-security rate_limit

# Test validation
cargo test -p riptide-security validation

# Test with output
cargo test -p riptide-security -- --nocapture
```

### Example Tests

```rust
use riptide_security::{ApiKeyAuth, RateLimiter};

#[tokio::test]
async fn test_api_key_auth() {
    let auth = ApiKeyAuth::new();

    // Create key
    let key = auth.create_key("user_123").await.unwrap();

    // Validate key
    let user_id = auth.validate_key(&key.secret).await.unwrap();
    assert_eq!(user_id, "user_123");

    // Revoke key
    auth.revoke_key(&key.id).await.unwrap();

    // Should fail after revocation
    assert!(auth.validate_key(&key.secret).await.is_err());
}

#[tokio::test]
async fn test_rate_limiter() {
    let limiter = RateLimiter::new(5, Duration::from_secs(60));

    // First 5 requests should succeed
    for _ in 0..5 {
        assert!(limiter.check_limit("client_1").await.unwrap());
    }

    // 6th request should fail
    assert!(!limiter.check_limit("client_1").await.unwrap());
}
```

## Best Practices

1. **Rotate Secrets**: Rotate JWT secrets and API keys regularly
2. **Use HTTPS**: Always use HTTPS in production
3. **Hash API Keys**: Never store API keys in plaintext
4. **Log Security Events**: Maintain audit logs for compliance
5. **Rate Limit Aggressively**: Protect against abuse and DDoS
6. **Validate All Inputs**: Never trust client input
7. **Set Security Headers**: Use all recommended security headers
8. **Monitor**: Set up alerts for security events

## Security Checklist

- [ ] API keys are hashed before storage
- [ ] JWT secrets are strong and rotated
- [ ] HTTPS is enforced
- [ ] Rate limiting is configured
- [ ] Input validation is comprehensive
- [ ] CORS is properly configured
- [ ] Security headers are set
- [ ] Audit logging is enabled
- [ ] Secrets are in environment variables
- [ ] Dependencies are regularly updated

## Performance Considerations

- **Rate Limiter**: Uses in-memory cache, consider Redis for distributed systems
- **API Key Validation**: Hash lookups are O(1)
- **JWT Validation**: Stateless, no database lookup needed
- **Request Filtering**: Checks run in order, put most common blocks first

## Dependencies

- `tokio` - Async runtime
- `bcrypt` - Password hashing
- `jsonwebtoken` - JWT support
- `chrono` - Date/time handling
- `anyhow` - Error handling
- `thiserror` - Error derive macros
- `tracing` - Logging

## License

Apache-2.0

## Related Crates

- `riptide-api` - Uses security middleware
- `riptide-config` - Security configuration
- `riptide-stealth` - Anti-detection (different concern)
- `riptide-utils` - Shared utilities
