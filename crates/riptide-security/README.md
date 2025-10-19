# RipTide Security

Security middleware and utilities for the RipTide web scraping framework.

## Overview

`riptide-security` provides comprehensive security features including authentication, authorization, rate limiting, input validation, and request sanitization for production deployments.

## Features

- **API Key Authentication**: Secure API key validation
- **JWT Support**: JSON Web Token authentication
- **Rate Limiting**: Per-key and IP-based rate limiting
- **Input Validation**: Request payload validation and sanitization
- **CORS**: Cross-origin resource sharing configuration
- **Security Headers**: HSTS, CSP, X-Frame-Options, etc.
- **Request Filtering**: Block malicious requests
- **Audit Logging**: Security event logging

## Usage

### API Key Authentication

```rust
use riptide_security::*;

let auth = ApiKeyAuth::new();

// Validate API key
if auth.validate_key(&request_key).await? {
    // Authorized
} else {
    // Unauthorized
}
```

### Rate Limiting

```rust
use riptide_security::*;

let limiter = RateLimiter::new(100, Duration::from_secs(60));

if limiter.check_limit(&client_id).await? {
    // Allow request
} else {
    // Rate limit exceeded
}
```

## License

Apache-2.0
