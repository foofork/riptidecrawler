# Session Cleanup System

## Overview

The session cleanup system automatically removes expired sessions to prevent memory leaks in production. It runs as a background task and can be configured via environment variables.

## Features

- **Automated Cleanup**: Background task runs periodically to remove expired sessions
- **Detailed Metrics**: Tracks sessions removed, memory freed, and cleanup duration
- **Thread-Safe**: Concurrent cleanup operations are safe and don't cause race conditions
- **Graceful Shutdown**: Cleanup task performs final cleanup before server shutdown
- **Configurable**: All parameters can be configured via environment variables

## Configuration

### Environment Variables

| Variable | Description | Default | Example |
|----------|-------------|---------|---------|
| `SESSION_TTL_SECS` | Session time-to-live in seconds | 86400 (24 hours) | 3600 |
| `SESSION_CLEANUP_INTERVAL_SECS` | Cleanup interval in seconds | 300 (5 minutes) | 600 |
| `SESSION_MAX_SESSIONS` | Maximum concurrent sessions | 1000 | 500 |
| `SESSION_BASE_DIR` | Base directory for session data | `/tmp/riptide-sessions` | `/var/lib/sessions` |
| `SESSION_PERSIST_COOKIES` | Enable cookie persistence | `true` | `false` |

### Example Configuration

```bash
# Set session TTL to 1 hour
export SESSION_TTL_SECS=3600

# Run cleanup every 10 minutes
export SESSION_CLEANUP_INTERVAL_SECS=600

# Limit to 500 concurrent sessions
export SESSION_MAX_SESSIONS=500

# Use custom data directory
export SESSION_BASE_DIR=/var/lib/riptide/sessions
```

## Architecture

### Background Cleanup Task

The cleanup task runs in a separate tokio task and:
1. Wakes up every `cleanup_interval` seconds
2. Scans all sessions for expired ones
3. Removes expired sessions from memory and disk
4. Tracks cleanup metrics
5. Logs cleanup operations

### Graceful Shutdown

On `SIGTERM` or `SIGINT`:
1. Server receives shutdown signal
2. Session cleanup task is notified via `CancellationToken`
3. Final cleanup is performed
4. Cleanup task exits cleanly
5. Server completes shutdown

## Cleanup Statistics

Each cleanup operation tracks:

- **Sessions Removed**: Number of expired sessions deleted
- **Sessions Remaining**: Number of active sessions after cleanup
- **Memory Freed**: Estimated memory freed in bytes
- **Cleanup Duration**: How long the cleanup took in milliseconds
- **Timestamp**: When the cleanup occurred

### Accessing Statistics

```rust
use riptide_api::sessions::{SessionManager, SessionConfig};

let config = SessionConfig::default();
let manager = SessionManager::new(config).await?;

// Get overall session statistics
let stats = manager.get_stats().await?;
println!("Total sessions: {}", stats.total_sessions);
println!("Expired cleaned: {}", stats.expired_sessions_cleaned);

// Get last cleanup statistics
if let Some(cleanup_stats) = manager.get_last_cleanup_stats().await {
    println!("Last cleanup removed: {}", cleanup_stats.sessions_removed);
    println!("Memory freed: {} KB", cleanup_stats.memory_freed_bytes / 1024);
    println!("Duration: {} ms", cleanup_stats.cleanup_duration_ms);
}
```

## API Endpoints

### Manual Cleanup

Trigger manual cleanup:

```bash
curl -X POST http://localhost:8080/sessions/cleanup
```

Response:
```json
{
  "sessions_removed": 5,
  "sessions_remaining": 10,
  "memory_freed_bytes": 40960,
  "cleanup_duration_ms": 12,
  "timestamp": "2025-11-02T10:30:00Z"
}
```

### Session Statistics

Get session statistics:

```bash
curl http://localhost:8080/sessions/stats
```

Response:
```json
{
  "total_sessions": 10,
  "expired_sessions_cleaned": 25,
  "total_disk_usage_bytes": 81920,
  "avg_session_age_seconds": 3600.5,
  "sessions_created_last_hour": 5
}
```

## Monitoring

### Health Checks

The session cleanup system integrates with the health check endpoint:

```bash
curl http://localhost:8080/healthz
```

### Metrics

Cleanup metrics are exposed via Prometheus:

- `riptide_sessions_total`: Total number of active sessions
- `riptide_sessions_expired_total`: Total expired sessions cleaned
- `riptide_sessions_cleanup_duration_seconds`: Cleanup operation duration

## Memory Management

### Memory Estimation

Each session is estimated to use approximately **8KB** of memory:
- Session metadata: ~500 bytes
- Cookie jar: ~2KB (varies by cookie count)
- File handles: ~500 bytes
- Overhead: ~5KB

### Memory Leak Prevention

The cleanup system prevents memory leaks by:
1. **Automatic Expiry**: Sessions expire after TTL
2. **Periodic Cleanup**: Background task removes expired sessions
3. **Bounded Storage**: `max_sessions` limit prevents unbounded growth
4. **Disk Cleanup**: Both memory and disk storage are cleaned

## Performance

### Cleanup Performance

Typical cleanup performance:
- **10 sessions**: <1ms
- **100 sessions**: 5-10ms
- **1000 sessions**: 50-100ms

### Concurrency

- Multiple concurrent cleanups are safe
- Read-write locks prevent race conditions
- Cleanup doesn't block session operations

## Troubleshooting

### High Memory Usage

If memory usage is high despite cleanup:

1. Check cleanup interval:
   ```bash
   # Reduce cleanup interval to 1 minute
   export SESSION_CLEANUP_INTERVAL_SECS=60
   ```

2. Reduce session TTL:
   ```bash
   # Reduce TTL to 30 minutes
   export SESSION_TTL_SECS=1800
   ```

3. Check logs for cleanup failures:
   ```bash
   grep "cleanup task failed" /var/log/riptide-api.log
   ```

### Sessions Not Being Cleaned

1. Verify cleanup task is running:
   ```bash
   grep "Started background session cleanup task" /var/log/riptide-api.log
   ```

2. Check for errors:
   ```bash
   grep "Background cleanup task failed" /var/log/riptide-api.log
   ```

3. Manually trigger cleanup:
   ```bash
   curl -X POST http://localhost:8080/sessions/cleanup
   ```

### Cleanup Taking Too Long

If cleanup operations are slow:

1. Reduce max sessions:
   ```bash
   export SESSION_MAX_SESSIONS=500
   ```

2. Increase cleanup interval:
   ```bash
   export SESSION_CLEANUP_INTERVAL_SECS=900  # 15 minutes
   ```

3. Check disk I/O performance (cleanup involves disk operations)

## Testing

### Running Tests

```bash
# Run all session cleanup tests
cargo test --package riptide-api --lib sessions::tests

# Run specific test
cargo test --package riptide-api --lib sessions::tests::test_cleanup_removes_expired_sessions

# Run with output
cargo test --package riptide-api --lib sessions::tests -- --nocapture
```

### Test Coverage

The test suite covers:
- ✅ Expired session removal
- ✅ Active session preservation
- ✅ Mixed expired and active sessions
- ✅ Thread safety (concurrent cleanup)
- ✅ Statistics tracking
- ✅ Graceful shutdown
- ✅ Empty storage
- ✅ Environment variable configuration
- ✅ Memory freed estimation

## Production Recommendations

### Configuration

For production deployments:

```bash
# 2 hour session TTL
export SESSION_TTL_SECS=7200

# Cleanup every 10 minutes
export SESSION_CLEANUP_INTERVAL_SECS=600

# Support 2000 concurrent sessions
export SESSION_MAX_SESSIONS=2000

# Use persistent storage
export SESSION_BASE_DIR=/var/lib/riptide/sessions

# Enable cookie persistence
export SESSION_PERSIST_COOKIES=true
```

### Monitoring

Monitor these metrics:
- Sessions created per hour
- Sessions expired per cleanup
- Cleanup duration
- Memory usage
- Disk usage

### Alerting

Set up alerts for:
- Cleanup failures (error logs)
- High cleanup duration (>5 seconds)
- Sessions approaching max limit (>80% of max)
- High memory usage (>80% of limit)

## Implementation Details

### CleanupStats Structure

```rust
pub struct CleanupStats {
    /// Number of sessions removed
    pub sessions_removed: usize,

    /// Number of sessions remaining
    pub sessions_remaining: usize,

    /// Estimated memory freed in bytes
    pub memory_freed_bytes: u64,

    /// Duration of cleanup operation
    pub cleanup_duration_ms: u64,

    /// Timestamp of cleanup
    pub timestamp: SystemTime,
}
```

### Cleanup Algorithm

1. Acquire read lock on sessions
2. Identify expired sessions (expires_at <= now)
3. Release read lock
4. For each expired session:
   - Acquire write lock
   - Remove from memory
   - Release write lock
   - Remove from disk
5. Calculate statistics
6. Update internal metrics
7. Log results

### Thread Safety

- Uses `Arc<RwLock<HashMap>>` for session storage
- Read locks for checking expiry
- Write locks for removal
- No deadlocks possible (short-lived locks)
- Concurrent cleanups are safe

## See Also

- [Session Management](./SESSION_MANAGEMENT.md)
- [API Documentation](./API.md)
- [Configuration Guide](./CONFIGURATION.md)
