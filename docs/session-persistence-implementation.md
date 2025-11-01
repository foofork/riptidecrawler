# Session Persistence Implementation

## Overview

This document describes the implementation of session persistence for stateful rendering in the Riptide API, enabling multi-step authenticated workflows with persistent browser state.

## Architecture

### Components

1. **Session Manager** (`crates/riptide-api/src/sessions/manager.rs`)
   - High-level API for session operations
   - Manages session lifecycle (create, get, update, remove)
   - Handles cookie storage and retrieval
   - Provides session expiration and cleanup

2. **Session Storage** (`crates/riptide-api/src/sessions/storage.rs`)
   - Thread-safe in-memory cache with disk persistence
   - Background cleanup task for expired sessions
   - Disk synchronization for session state

3. **Session Types** (`crates/riptide-api/src/sessions/types.rs`)
   - Core data structures: `Session`, `Cookie`, `CookieJar`
   - Configuration: `SessionConfig`
   - Metadata: `SessionMetadata`, `SessionBrowserConfig`

4. **RPC Client** (`crates/riptide-api/src/rpc_client.rs`)
   - Extended with `render_dynamic_with_session()` method
   - Passes session context to headless browser service
   - Supports both stateless and stateful rendering

5. **Render Processors** (`crates/riptide-api/src/handlers/render/processors.rs`)
   - Integrated session context into dynamic rendering pipeline
   - Retrieves user data directory from session manager
   - Passes session information to RPC client

## Implementation Details

### Session Persistence Flow

```
1. Client creates session via /api/sessions
   ↓
2. Session Manager creates session with unique ID
   ↓
3. Session storage creates user data directory
   ↓
4. Client makes render request with session_id header
   ↓
5. Render processor retrieves session from manager
   ↓
6. RPC client passes session_id + user_data_dir to headless service
   ↓
7. Headless service launches browser with persistent profile
   ↓
8. Browser maintains cookies, localStorage, auth state
   ↓
9. Subsequent requests reuse the same browser profile
```

### Key Features

#### 1. Cookie Management
- Store and retrieve cookies per domain
- Support for HttpOnly, Secure, SameSite attributes
- Cookie expiration handling
- Netscape format import/export

#### 2. Session Lifecycle
- Auto-generated session IDs (UUID-based)
- Configurable TTL (Time To Live)
- Last accessed timestamp tracking
- Automatic session extension on use
- Background cleanup of expired sessions

#### 3. Browser State Persistence
- User data directory per session
- Persistent browser profiles
- Cookie storage on disk
- Session metadata (user agent, viewport, locale)

#### 4. Session Expiration
- Configurable default TTL (default: 24 hours)
- Background cleanup task (default: every 5 minutes)
- Automatic removal of expired sessions
- Grace period for session extension

### API Changes

#### RPC Client

**New Method:**
```rust
pub async fn render_dynamic_with_session(
    &self,
    url: &str,
    config: &DynamicConfig,
    stealth_config: Option<&riptide_stealth::StealthConfig>,
    session_id: Option<&str>,
    user_data_dir: Option<&str>,
) -> Result<DynamicRenderResult>
```

**Backward Compatible:**
The original `render_dynamic()` method now delegates to `render_dynamic_with_session()` with `None` values for session parameters, maintaining full backward compatibility.

#### Headless Request Format

**Extended Fields:**
```rust
struct HeadlessRenderRequest {
    url: String,
    session_id: Option<String>,        // NEW: Session identifier
    user_data_dir: Option<String>,     // NEW: Browser profile directory
    actions: Option<Vec<HeadlessPageAction>>,
    timeouts: Option<HeadlessTimeouts>,
    artifacts: Option<HeadlessArtifacts>,
    stealth_config: Option<riptide_stealth::StealthConfig>,
}
```

### Configuration

```rust
SessionConfig {
    base_data_dir: PathBuf::from("/tmp/riptide-sessions"),
    default_ttl: Duration::from_secs(3600 * 24),  // 24 hours
    max_sessions: 1000,
    cleanup_interval: Duration::from_secs(300),   // 5 minutes
    persist_cookies: true,
    encrypt_session_data: false,  // Reserved for future use
}
```

## Testing

### Test Coverage

1. **Integration Tests** (`tests/session_persistence_tests.rs`)
   - Session persistence with RPC client
   - Cookie storage and retrieval
   - Session expiration handling
   - Session state restoration after restart
   - Session extension
   - User data directory creation

### Test Scenarios

#### 1. Session Persistence Integration
```rust
test_session_persistence_integration()
```
- Creates session with SessionManager
- Retrieves user data directory
- Calls RPC client with session context
- Verifies API contract (may fail if headless service not running)

#### 2. Cookie Persistence
```rust
test_session_cookie_persistence()
```
- Stores cookies with various attributes
- Retrieves cookies by domain
- Verifies disk persistence
- Checks cookie metadata (secure, httpOnly, etc.)

#### 3. Session Expiration
```rust
test_session_expiration()
```
- Creates session with short TTL
- Waits for expiration
- Runs cleanup task
- Verifies session removal

#### 4. State Restoration
```rust
test_session_state_restoration()
```
- Creates session and stores cookies
- Simulates restart by dropping SessionManager
- Creates new SessionManager
- Verifies sessions and cookies are restored from disk

#### 5. Session Extension
```rust
test_session_extension()
```
- Creates session
- Extends expiry time
- Verifies new expiration timestamp

#### 6. User Data Directory
```rust
test_user_data_directory()
```
- Creates session
- Retrieves user data directory
- Verifies directory structure
- Checks session.json file exists

## Usage Examples

### Basic Session Creation

```rust
// Create session manager
let config = SessionConfig::default();
let session_manager = SessionManager::new(config).await?;

// Create a new session
let session = session_manager.create_session().await?;
let session_id = session.session_id;

// Use session for rendering
let rpc_client = RpcClient::new();
let user_data_dir = session_manager.get_user_data_dir(&session_id).await?;

let result = rpc_client.render_dynamic_with_session(
    "https://example.com",
    &dynamic_config,
    None,
    Some(&session_id),
    Some(&user_data_dir.to_string_lossy()),
).await?;
```

### Cookie Management

```rust
// Store a cookie
let cookie = Cookie::new("auth_token".to_string(), "secret123".to_string())
    .with_domain("example.com".to_string())
    .with_path("/".to_string())
    .secure()
    .http_only();

session_manager.set_cookie(&session_id, "example.com", cookie).await?;

// Retrieve cookies
let cookies = session_manager
    .get_cookies_for_domain(&session_id, "example.com")
    .await?;

// Remove a cookie
session_manager.remove_cookie(&session_id, "example.com", "auth_token").await?;
```

### Session Extension

```rust
// Extend session by 1 hour
session_manager.extend_session(
    &session_id,
    Duration::from_secs(3600)
).await?;
```

### Session Cleanup

```rust
// Manually trigger cleanup
let cleaned_count = session_manager.cleanup_expired().await?;
println!("Cleaned up {} expired sessions", cleaned_count);
```

## Implementation Status

### Completed ✅

1. ✅ Session data structures and types
2. ✅ Session storage with disk persistence
3. ✅ Session manager API
4. ✅ Cookie jar implementation
5. ✅ RPC client extension for session support
6. ✅ Render processor integration
7. ✅ Comprehensive test suite
8. ✅ Background cleanup task
9. ✅ Session expiration handling
10. ✅ User data directory management

### TODO Comments Removed ✅

- ✅ `rpc_client.rs:56` - Session persistence implemented
- ✅ `processors.rs:111` - Session context passed to RPC

### Headless Service Requirements

**Note:** The headless browser service needs to support:
1. `session_id` parameter for identifying browser sessions
2. `user_data_dir` parameter for browser profile directories
3. Launching browsers with persistent profiles
4. Maintaining cookies and localStorage across requests

The current implementation provides the API contract and will work once the headless service implements these features.

## Performance Considerations

### Memory Management
- In-memory session cache for fast access
- Configurable maximum session limit (default: 1000)
- Background cleanup prevents memory leaks

### Disk I/O
- Sessions persisted to disk on creation/update
- Lazy loading: sessions loaded from disk only when accessed
- Batch cleanup during background task

### Concurrency
- Thread-safe using `Arc<RwLock<>>` for shared state
- Minimal lock contention through read-heavy access patterns
- Async/await for non-blocking I/O

## Security Considerations

### Current Implementation
- Session IDs use UUID v4 for uniqueness and unpredictability
- Cookies support Secure and HttpOnly flags
- File-based storage with OS-level permissions

### Future Enhancements
- Session data encryption (infrastructure in place)
- Rate limiting for session creation
- Session fingerprinting for security
- Configurable session token rotation

## Monitoring and Debugging

### Statistics Available

```rust
let stats = session_manager.get_stats().await?;
println!("Total sessions: {}", stats.total_sessions);
println!("Expired cleaned: {}", stats.expired_sessions_cleaned);
println!("Disk usage: {} bytes", stats.total_disk_usage_bytes);
println!("Avg age: {} seconds", stats.avg_session_age_seconds);
println!("Created last hour: {}", stats.sessions_created_last_hour);
```

### Logging

All session operations are logged with structured tracing:
- Session creation: `INFO` level
- Cookie operations: `DEBUG` level
- Cleanup operations: `INFO` level (when sessions cleaned)
- Errors: `ERROR` level with context

## Migration Guide

### For Existing Code

**No changes required!** The implementation is fully backward compatible:

```rust
// Old code continues to work
rpc_client.render_dynamic(url, config, stealth).await?;

// New code with session support
rpc_client.render_dynamic_with_session(
    url, config, stealth,
    Some(&session_id),
    Some(&user_data_dir)
).await?;
```

### For New Features

To add session support to new endpoints:

1. Extract session_id from request headers
2. Get user_data_dir from session_manager
3. Pass both to render processor
4. Processor automatically uses session-aware rendering

## Related Files

- `crates/riptide-api/src/rpc_client.rs` - RPC client with session support
- `crates/riptide-api/src/handlers/render/processors.rs` - Render processors
- `crates/riptide-api/src/sessions/manager.rs` - Session manager
- `crates/riptide-api/src/sessions/storage.rs` - Session storage
- `crates/riptide-api/src/sessions/types.rs` - Session types
- `crates/riptide-api/tests/session_persistence_tests.rs` - Test suite

## Verification

Run the following to verify the implementation:

```bash
# Check compilation
cargo check --package riptide-api

# Run tests
cargo test --package riptide-api session_persistence

# Run all session tests
cargo test --package riptide-api session
```

## Summary

The session persistence implementation provides a robust foundation for stateful browser rendering with:
- Persistent browser profiles across requests
- Cookie storage and retrieval
- Automatic session lifecycle management
- Comprehensive test coverage
- Full backward compatibility
- Production-ready error handling and logging

All TODO comments have been resolved, and the feature is ready for integration with a headless browser service that supports persistent profiles.
