# Development Mode - Authentication Bypass

## Overview

For local development and testing, RipTide API supports a **development mode** that bypasses authentication requirements. This allows CLI commands and API testing without needing to configure API keys.

## Configuration

### Environment Variables

The authentication system is controlled by the `REQUIRE_AUTH` environment variable:

```bash
# Disable authentication (development mode)
REQUIRE_AUTH=false

# Enable authentication (production mode - default)
REQUIRE_AUTH=true
```

### .env File Configuration

The `.env` file at the project root already has dev mode configured:

```bash
# Authentication & Security (Line 125)
REQUIRE_AUTH=false
```

## Starting API Server in Dev Mode

### Option 1: Using Environment File (Recommended)

The API server automatically loads `.env` on startup:

```bash
# Start API server (reads REQUIRE_AUTH=false from .env)
cargo run --release --bin riptide-api
```

### Option 2: Explicit Environment Variable

Override the environment variable when starting:

```bash
# Start with auth disabled
REQUIRE_AUTH=false cargo run --release --bin riptide-api

# Or using the built binary
REQUIRE_AUTH=false target/release/riptide-api --bind 0.0.0.0:8080
```

### Option 3: Restart Existing Server

If the server is already running, restart it to pick up configuration changes:

```bash
# Find and kill existing process
pkill riptide-api

# Start with dev mode
REQUIRE_AUTH=false cargo run --release --bin riptide-api
```

## Authentication Middleware Behavior

### Code Location
- **Middleware**: `/workspaces/eventmesh/crates/riptide-api/src/middleware/auth.rs`
- **Lines 36-39**: Reads `REQUIRE_AUTH` environment variable
- **Lines 141-144**: Bypasses authentication when disabled

### Public Endpoints (Always Accessible)
These endpoints never require authentication:
- `/health`
- `/metrics`
- `/api/v1/health`
- `/api/v1/metrics`

### Protected Endpoints (Require Auth When Enabled)
All other endpoints require authentication when `REQUIRE_AUTH=true`:
- `/api/v1/tables/extract`
- `/api/v1/search`
- `/crawl`
- `/pdf/*`
- And all other API endpoints

## Testing Dev Mode

### Quick Test Script

```bash
# Test without authentication
curl http://localhost:8080/api/v1/health
curl -X POST http://localhost:8080/api/v1/tables/extract \
  -H "Content-Type: application/json" \
  -d '{"html_content":"<table><tr><th>A</th></tr></table>"}'
```

### Expected Behavior

**With Dev Mode (`REQUIRE_AUTH=false`):**
```json
{
  "tables": [...],
  "total_tables": 1
}
```

**Without Dev Mode (`REQUIRE_AUTH=true`):**
```json
{
  "error": "Unauthorized",
  "message": "Missing API key"
}
```

## CLI Usage in Dev Mode

When `REQUIRE_AUTH=false`, CLI commands work without API key configuration:

```bash
# Tables command
riptide-cli tables extract --html "<table>...</table>"

# Search command
riptide-cli search "query"

# Crawl command
riptide-cli crawl https://example.com

# PDF extraction
riptide-cli pdf extract file.pdf
```

## Production Configuration

### Enabling Authentication for Production

1. Update `.env`:
```bash
REQUIRE_AUTH=true
API_KEYS=production-key-1,production-key-2
```

2. Restart the API server:
```bash
cargo run --release --bin riptide-api
```

3. Configure clients with API key:
```bash
# Via environment variable
export RIPTIDE_API_KEY=production-key-1

# Or via CLI flag
riptide-cli --api-key production-key-1 crawl https://example.com
```

## Security Considerations

### Development Mode
- ✅ **Use for**: Local testing, development, CI/CD pipelines
- ✅ **Safe when**: API is not exposed to external networks
- ❌ **Never use**: In production or public-facing deployments

### Production Mode
- ✅ **Always enable**: For production deployments
- ✅ **Use strong keys**: Generate cryptographically secure API keys
- ✅ **Rotate keys**: Regularly update API keys
- ✅ **Monitor access**: Track API key usage via logs

## Troubleshooting

### Issue: "401 Unauthorized" in Dev Mode

**Solution**: Verify environment variable is set:
```bash
# Check if server sees the variable
grep REQUIRE_AUTH .env

# Restart server with explicit override
REQUIRE_AUTH=false cargo run --release --bin riptide-api
```

### Issue: Server Not Reading .env

**Solution**: Ensure `.env` is in the working directory:
```bash
# Run from project root
cd /workspaces/eventmesh
cargo run --release --bin riptide-api
```

### Issue: Changes Not Taking Effect

**Solution**: Kill old process and restart:
```bash
# Find process
ps aux | grep riptide-api

# Kill it
pkill riptide-api

# Start fresh
REQUIRE_AUTH=false cargo run --release --bin riptide-api
```

## Implementation Details

### AuthConfig Structure

```rust
pub struct AuthConfig {
    valid_api_keys: Arc<RwLock<HashSet<String>>>,
    require_auth: bool,  // Controlled by REQUIRE_AUTH env var
    public_paths: Arc<Vec<String>>,
}
```

### Authentication Flow

```rust
pub async fn auth_middleware(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, Response> {
    // 1. Check if path is public
    if state.auth_config.is_public_path(path) {
        return Ok(next.run(request).await);
    }

    // 2. Check if auth is required (DEV MODE CHECK)
    if !state.auth_config.requires_auth() {
        return Ok(next.run(request).await);  // ✅ Bypass auth
    }

    // 3. Validate API key (only when auth required)
    validate_api_key(&request)?;
    Ok(next.run(request).await)
}
```

## Summary

- **Dev Mode**: Set `REQUIRE_AUTH=false` in `.env` or environment
- **Restart Required**: Changes take effect after server restart
- **Already Configured**: Project `.env` has dev mode enabled by default
- **Production**: Set `REQUIRE_AUTH=true` and configure `API_KEYS`

For more information, see:
- Authentication middleware: `crates/riptide-api/src/middleware/auth.rs`
- Environment configuration: `.env`
- API documentation: `docs/API.md`
