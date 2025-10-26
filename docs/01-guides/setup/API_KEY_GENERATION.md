# API Key Generation and Management

This guide explains how to generate, manage, and use API keys in the RipTide system.

## Overview

RipTide uses a secure API key system for authentication and authorization. API keys are:
- **Cryptographically secure**: Generated using 64 random alphanumeric characters
- **Hashed**: Stored as SHA-256 hashes, never in plaintext
- **Rate-limited**: Support per-minute, per-hour, and per-day limits
- **Tenant-isolated**: Each key is associated with a specific tenant
- **Scoped**: Can be restricted to specific permissions
- **Expirable**: Optional expiration dates for enhanced security

## API Key Format

All RipTide API keys follow this format:
```
rpt_<64_random_alphanumeric_characters>
```

Example:
```
rpt_AbCd1234EfGh5678IjKl9012MnOp3456QrSt7890UvWx1234YzAb5678CdEf9012
```

## Generating API Keys

### Programmatic Generation

```rust
use riptide_core::security::api_keys::ApiKeyManager;
use riptide_core::security::types::TenantId;

#[tokio::main]
async fn main() -> Result<()> {
    let manager = ApiKeyManager::new();

    // Create a new API key
    let (api_key, raw_key) = manager.create_api_key(
        TenantId::from("your-tenant-id"),
        "Production API Key".to_string(),
        Some("Key for production environment".to_string()),
        vec!["read".to_string(), "write".to_string()],
        None, // Use default rate limits
        None, // No expiration
    ).await?;

    // IMPORTANT: Store the raw_key securely - it's only shown once!
    println!("API Key: {}", raw_key);
    println!("Key ID: {}", api_key.id);

    Ok(())
}
```

### Development/Testing Keys

For development and testing, you can disable authentication:

**Environment Variable:**
```bash
export REQUIRE_AUTH=false
```

**In Code:**
```rust
std::env::set_var("REQUIRE_AUTH", "false");
```

**In Test Environment File:**
```bash
# tests/.env.test
REQUIRE_AUTH=false
```

## API Key Configuration

### Rate Limiting

Control request rates with `RateLimitConfig`:

```rust
use riptide_core::security::types::RateLimitConfig;

let rate_limits = RateLimitConfig {
    requests_per_minute: 60,
    requests_per_hour: 1000,
    requests_per_day: 10000,
    burst_allowance: 10, // Allow burst of 10 requests
    enable_adaptive_limits: false,
};

let (api_key, raw_key) = manager.create_api_key(
    tenant_id,
    "Rate Limited Key".to_string(),
    None,
    vec![],
    Some(rate_limits),
    None,
).await?;
```

### Scopes and Permissions

Restrict API key permissions using scopes:

```rust
let scopes = vec![
    "read:content".to_string(),
    "write:content".to_string(),
    "admin:cache".to_string(),
];

let (api_key, raw_key) = manager.create_api_key(
    tenant_id,
    "Scoped Key".to_string(),
    None,
    scopes,
    None,
    None,
).await?;
```

### Expiration

Set an expiration date for temporary keys:

```rust
use chrono::{Utc, Duration};

let expires_at = Utc::now() + Duration::days(30);

let (api_key, raw_key) = manager.create_api_key(
    tenant_id,
    "Temporary Key".to_string(),
    Some("Expires in 30 days".to_string()),
    vec![],
    None,
    Some(expires_at),
).await?;
```

## Using API Keys

### HTTP Requests

Include the API key in the `X-API-Key` header:

```bash
curl -H "X-API-Key: rpt_YourApiKeyHere" \
     https://api.riptide.example.com/api/v1/extract
```

### CLI Tool

```bash
riptide --api-key "rpt_YourApiKeyHere" extract --url "https://example.com"
```

Or set as environment variable:

```bash
export RIPTIDE_API_KEY="rpt_YourApiKeyHere"
riptide extract --url "https://example.com"
```

## API Key Management

### Validating Keys

```rust
let result = manager.validate_api_key(&raw_key).await;

match result {
    Ok((api_key, security_context)) => {
        println!("Valid key for tenant: {}", api_key.tenant_id);
    }
    Err(e) => {
        eprintln!("Invalid key: {}", e);
    }
}
```

### Rotating Keys

Generate a new key while preserving metadata:

```rust
let new_raw_key = manager.rotate_api_key(&api_key.id).await?;
println!("New API key: {}", new_raw_key);
```

### Revoking Keys

Disable an API key without deleting it:

```rust
manager.revoke_api_key(&api_key.id).await?;
println!("Key revoked successfully");
```

### Listing Tenant Keys

Get all keys for a specific tenant:

```rust
let keys = manager.get_tenant_keys(&tenant_id).await;
for key in keys {
    println!("Key: {} - Active: {}", key.name, key.is_active);
}
```

### Cleanup Expired Keys

Automatically remove expired keys:

```rust
let removed_count = manager.cleanup_expired_keys().await;
println!("Removed {} expired keys", removed_count);
```

## Security Best Practices

1. **Never commit API keys to version control**
   - Use environment variables or secure key management systems
   - Add `.env*` files to `.gitignore`

2. **Rotate keys regularly**
   - Implement automated key rotation for production systems
   - Rotate immediately if a key is compromised

3. **Use appropriate scopes**
   - Grant minimum necessary permissions
   - Create separate keys for different services

4. **Set expiration dates**
   - Use temporary keys for short-lived access
   - Force renewal for long-term keys

5. **Monitor key usage**
   - Track usage counts and last used timestamps
   - Alert on unusual patterns or rate limit violations

6. **Secure key storage**
   - Store raw keys in secure vaults (e.g., HashiCorp Vault, AWS Secrets Manager)
   - Never log or display full keys in production

## Environment Variables

Key environment variables for API key configuration:

```bash
# Disable authentication (development/testing only)
REQUIRE_AUTH=false

# API key for client applications
RIPTIDE_API_KEY=rpt_YourApiKeyHere

# Logging level
RUST_LOG=info

# Backtrace for debugging
RUST_BACKTRACE=1
```

## Testing with API Keys

### Integration Tests

Integration tests should disable authentication:

```rust
#[tokio::test]
async fn test_api_endpoint() {
    // Disable authentication for tests
    std::env::set_var("REQUIRE_AUTH", "false");

    // Your test code here
}
```

Or use the test environment file:

```bash
# tests/.env.test
REQUIRE_AUTH=false
```

### Unit Tests

The API key manager includes comprehensive unit tests:

```bash
# Run API key tests
cargo test -p riptide-core --test api_keys

# Run all security tests
cargo test -p riptide-core security::
```

## Troubleshooting

### Invalid API Key Error

**Problem:** Receiving "Invalid API key" errors

**Solutions:**
- Verify the key format starts with `rpt_`
- Check if the key has been revoked or expired
- Ensure the key is being sent in the `X-API-Key` header
- Verify the key belongs to the correct tenant

### Rate Limit Exceeded

**Problem:** Receiving "Rate limit exceeded" errors

**Solutions:**
- Review rate limit configuration for the key
- Check current usage with `get_api_key()` to see usage counts
- Implement exponential backoff in client applications
- Consider increasing limits or creating additional keys

### Key Rotation Issues

**Problem:** Old key still works after rotation

**Solutions:**
- Rotation generates a new key but doesn't revoke the old one
- Explicitly revoke the old key after verifying the new one works
- Implement graceful rotation with overlap period

## Production Deployment

### Key Generation Workflow

1. **Generate key programmatically**
   ```bash
   # Using a custom admin tool
   riptide-admin generate-key --tenant "production" --name "Main API Key"
   ```

2. **Store in secret management**
   ```bash
   # Example with AWS Secrets Manager
   aws secretsmanager create-secret \
     --name riptide/production/api-key \
     --secret-string "rpt_YourGeneratedKey"
   ```

3. **Configure application**
   ```bash
   # Kubernetes secret
   kubectl create secret generic riptide-api-key \
     --from-literal=key=$(aws secretsmanager get-secret-value \
       --secret-id riptide/production/api-key \
       --query SecretString --output text)
   ```

4. **Monitor and rotate**
   - Set up monitoring for key usage
   - Schedule regular key rotation (e.g., quarterly)
   - Maintain audit logs of key generation and usage

## API Reference

For detailed API documentation, see:
- `/workspaces/eventmesh/crates/riptide-core/src/security/api_keys.rs`
- `/workspaces/eventmesh/crates/riptide-core/src/security/types.rs`

## Support

For issues or questions about API key management:
- Check the security module documentation
- Review the integration tests for examples
- Consult the main RipTide documentation
