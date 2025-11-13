# Phase 1: Configuration Architecture Design

**Version**: 1.0
**Date**: 2025-11-12
**Status**: Design/Specification
**Phase**: Phase 1 - Make Redis Optional

---

## Executive Summary

This document specifies the configuration architecture for making Redis optional in RipTide, enabling three progressive deployment modes: **Minimal** (no dependencies), **Enhanced** (Redis cache), and **Distributed** (full Redis with workers). The design leverages existing hexagonal architecture from Phase 0 and adds configuration-driven backend selection.

### Key Decisions

| Decision | Rationale |
|----------|-----------|
| Default to in-memory cache | Zero-friction onboarding, CI/CD friendly |
| Three deployment modes | Clear upgrade path, specific use cases |
| Environment variable overrides | 12-factor app compliance, Docker-friendly |
| Backward compatibility | Existing Redis deployments continue working |

---

## 1. Configuration Schema

### 1.1 Cache Backend Configuration

```rust
/// Cache backend selection
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CacheBackend {
    /// In-memory cache (no external dependencies)
    Memory,
    /// Redis-backed cache (requires Redis server)
    Redis,
}

impl Default for CacheBackend {
    fn default() -> Self {
        CacheBackend::Memory // Zero-friction default
    }
}

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Cache backend to use
    #[serde(default)]
    pub backend: CacheBackend,

    /// Redis URL (required only if backend = Redis)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redis_url: Option<String>,

    /// Memory cache TTL in seconds (for in-memory backend)
    #[serde(default = "default_memory_ttl")]
    pub memory_ttl: u64,

    /// Max memory cache entries (LRU eviction)
    #[serde(default = "default_max_entries")]
    pub max_memory_entries: usize,
}

fn default_memory_ttl() -> u64 { 3600 }
fn default_max_entries() -> usize { 10_000 }

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            backend: CacheBackend::Memory,
            redis_url: None,
            memory_ttl: 3600,
            max_memory_entries: 10_000,
        }
    }
}
```

### 1.2 Worker Configuration

```rust
/// Worker service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerConfig {
    /// Enable worker service (requires Redis)
    #[serde(default)]
    pub enabled: bool,

    /// Redis URL for job queue (required if enabled = true)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redis_url: Option<String>,

    /// Worker pool size
    #[serde(default = "default_worker_count")]
    pub worker_count: usize,

    /// Job timeout in seconds
    #[serde(default = "default_job_timeout")]
    pub job_timeout: u64,

    /// Max retry attempts for failed jobs
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
}

fn default_worker_count() -> usize { 4 }
fn default_job_timeout() -> u64 { 300 }
fn default_max_retries() -> u32 { 3 }

impl Default for WorkerConfig {
    fn default() -> Self {
        Self {
            enabled: false, // Disabled by default
            redis_url: None,
            worker_count: 4,
            job_timeout: 300,
            max_retries: 3,
        }
    }
}
```

### 1.3 Integration with RiptideConfig

```rust
/// Main application configuration (existing, enhanced)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiptideConfig {
    // Existing fields (server, logging, etc.)

    /// Cache configuration
    #[serde(default)]
    pub cache: CacheConfig,

    /// Worker service configuration
    #[serde(default)]
    pub workers: WorkerConfig,
}
```

---

## 2. Deployment Configurations

### 2.1 Minimal Mode (`config/deployment/minimal.toml`)

**Target Users**: Local development, CI/CD, simple extraction, learning

**Configuration**:
```toml
# Minimal configuration - No Redis required
# Perfect for: Local development, CI/CD, simple extraction

[cache]
backend = "memory"
memory_ttl = 3600              # 1 hour cache TTL
max_memory_entries = 10000     # 10k entries max

[workers]
enabled = false                # No background workers

[server]
host = "0.0.0.0"
port = 8080

[extraction]
timeout_ms = 30000
max_concurrent = 50

[spider]
enabled = true                 # Spider works without Redis
max_depth = 3
concurrency = 4
```

**Features**:
- ✅ Fast extraction and crawling
- ✅ In-memory cache (3600s TTL)
- ✅ Single-process synchronous execution
- ✅ Zero external dependencies
- ⚠️ Cache clears on restart
- ⚠️ No persistent sessions
- ⚠️ No async job queue

**Resource Profile**:
- Memory: ~440MB (API only)
- CPU: 0.5-2.0 cores
- Disk: None required
- Network: None required

### 2.2 Enhanced Mode (`config/deployment/enhanced.toml`)

**Target Users**: Production single-instance, persistent cache, session management

**Configuration**:
```toml
# Enhanced configuration - Redis for caching
# Perfect for: Production single-instance, persistent cache

[cache]
backend = "redis"
redis_url = "redis://localhost:6379/0"
# memory_ttl and max_memory_entries ignored with redis backend

[workers]
enabled = false                # Still single-process

[server]
host = "0.0.0.0"
port = 8080

[extraction]
timeout_ms = 30000
max_concurrent = 100           # Higher with Redis

[spider]
enabled = true
max_depth = 5                  # Deeper crawls with persistent cache
concurrency = 8

[session]
# Sessions persist in Redis
ttl = 86400                    # 24 hour session TTL
```

**Features**:
- ✅ Everything in Minimal
- ✅ Persistent cache across restarts
- ✅ Session management (24h TTL)
- ✅ Better performance for repeated requests
- ⚠️ Single-instance only (no horizontal scaling)
- ⚠️ Jobs execute synchronously

**Resource Profile**:
- Memory: ~700MB (API + Redis)
- CPU: 1.0-2.0 cores
- Disk: Redis persistence (~1GB)
- Network: Redis connection

**Redis Configuration**:
```bash
redis-server --appendonly yes --appendfsync everysec \
  --maxmemory 512mb --maxmemory-policy allkeys-lru
```

### 2.3 Distributed Mode (`config/deployment/distributed.toml`)

**Target Users**: Enterprise scale, multi-instance, background jobs, horizontal scaling

**Configuration**:
```toml
# Distributed configuration - Full Redis features
# Perfect for: Enterprise scale, multi-instance, job queue

[cache]
backend = "redis"
redis_url = "redis://localhost:6379/0"

[workers]
enabled = true
redis_url = "redis://localhost:6379/1"  # Separate DB for job queue
worker_count = 8
job_timeout = 300              # 5 minute timeout
max_retries = 3

[server]
host = "0.0.0.0"
port = 8080

[extraction]
timeout_ms = 60000             # Longer timeout with workers
max_concurrent = 200           # High concurrency

[spider]
enabled = true
max_depth = 10                 # Deep crawls with workers
concurrency = 16

[session]
ttl = 86400

[distributed]
# Multi-instance coordination
instance_id = "${HOSTNAME}"    # Unique per instance
coordination_ttl = 60
```

**Features**:
- ✅ Everything in Enhanced
- ✅ Distributed job queue
- ✅ Multi-instance scaling
- ✅ Background worker pool
- ✅ Horizontal scalability
- ✅ Job retry and error recovery
- ✅ Multi-tenancy support

**Resource Profile**:
- Memory: ~1.2GB per API instance + workers
- CPU: 2.0-4.0 cores (API + workers)
- Disk: Redis persistence (~5GB)
- Network: Redis + multi-instance coordination

**Scaling Example**:
```bash
# 3 API instances + 8 workers
docker-compose up --scale riptide-api=3 --scale riptide-worker=2
```

---

## 3. Environment Variable Mapping

### 3.1 Cache Configuration

| TOML Config | Environment Variable | Default | Required |
|-------------|---------------------|---------|----------|
| `cache.backend` | `CACHE_BACKEND` | `memory` | No |
| `cache.redis_url` | `REDIS_URL` | - | If backend=redis |
| `cache.memory_ttl` | `CACHE_MEMORY_TTL` | `3600` | No |
| `cache.max_memory_entries` | `CACHE_MAX_ENTRIES` | `10000` | No |

### 3.2 Worker Configuration

| TOML Config | Environment Variable | Default | Required |
|-------------|---------------------|---------|----------|
| `workers.enabled` | `WORKERS_ENABLED` | `false` | No |
| `workers.redis_url` | `WORKERS_REDIS_URL` | - | If enabled=true |
| `workers.worker_count` | `WORKER_COUNT` | `4` | No |
| `workers.job_timeout` | `JOB_TIMEOUT` | `300` | No |
| `workers.max_retries` | `MAX_RETRIES` | `3` | No |

### 3.3 Precedence Order

1. **Environment variables** (highest priority)
2. **Config file** (TOML)
3. **Defaults** (lowest priority)

Example:
```bash
# Override config file with environment
CACHE_BACKEND=redis REDIS_URL=redis://prod-redis:6379/0 ./riptide-api
```

### 3.4 Configuration Loading Logic

```rust
impl RiptideConfig {
    pub fn load() -> Result<Self> {
        // 1. Load from file (if specified)
        let mut config = if let Ok(path) = std::env::var("CONFIG_PATH") {
            Self::from_file(&path)?
        } else {
            Self::default()
        };

        // 2. Apply environment variable overrides
        if let Ok(backend) = std::env::var("CACHE_BACKEND") {
            config.cache.backend = backend.parse()?;
        }
        if let Ok(url) = std::env::var("REDIS_URL") {
            config.cache.redis_url = Some(url);
        }
        if let Ok(enabled) = std::env::var("WORKERS_ENABLED") {
            config.workers.enabled = enabled.parse()?;
        }

        // 3. Validate configuration
        config.validate()?;

        Ok(config)
    }
}
```

---

## 4. Validation Rules

### 4.1 Cache Validation

```rust
impl CacheConfig {
    pub fn validate(&self) -> Result<()> {
        match self.backend {
            CacheBackend::Redis => {
                if self.redis_url.is_none() {
                    return Err(anyhow!(
                        "cache.redis_url is required when cache.backend = 'redis'"
                    ));
                }
                // Validate Redis URL format
                if let Some(url) = &self.redis_url {
                    if !url.starts_with("redis://") && !url.starts_with("rediss://") {
                        return Err(anyhow!(
                            "Invalid Redis URL format. Expected redis:// or rediss://"
                        ));
                    }
                }
            }
            CacheBackend::Memory => {
                if self.memory_ttl == 0 {
                    return Err(anyhow!("memory_ttl must be > 0"));
                }
                if self.max_memory_entries == 0 {
                    return Err(anyhow!("max_memory_entries must be > 0"));
                }
            }
        }
        Ok(())
    }
}
```

### 4.2 Worker Validation

```rust
impl WorkerConfig {
    pub fn validate(&self) -> Result<()> {
        if self.enabled {
            if self.redis_url.is_none() {
                return Err(anyhow!(
                    "workers.redis_url is required when workers.enabled = true"
                ));
            }
            if self.worker_count == 0 {
                return Err(anyhow!("worker_count must be > 0 when workers enabled"));
            }
            if self.job_timeout == 0 {
                return Err(anyhow!("job_timeout must be > 0"));
            }
        }
        Ok(())
    }
}
```

### 4.3 Cross-Field Validation

```rust
impl RiptideConfig {
    pub fn validate(&self) -> Result<()> {
        // Validate individual sections
        self.cache.validate()?;
        self.workers.validate()?;

        // Cross-field validation: workers require Redis
        if self.workers.enabled && self.cache.backend != CacheBackend::Redis {
            return Err(anyhow!(
                "Workers require Redis cache backend. Set cache.backend = 'redis'"
            ));
        }

        Ok(())
    }
}
```

---

## 5. Migration Guide

### 5.1 Existing Deployments (No Changes Required)

**Current users with Redis** continue working without any changes:

```toml
# Existing config (still works)
[cache]
backend = "redis"  # Or omit and set REDIS_URL env var
redis_url = "redis://localhost:6379"
```

**Environment variable migration**:
- `REDIS_URL` → Maps to `cache.redis_url` (backward compatible)
- No breaking changes

### 5.2 Upgrading Minimal → Enhanced

**Scenario**: You started with minimal mode, now need persistent cache

**Steps**:
1. Install Redis:
   ```bash
   docker run -d -p 6379:6379 redis:7-alpine
   ```

2. Update configuration:
   ```toml
   [cache]
   backend = "redis"
   redis_url = "redis://localhost:6379/0"
   ```

3. Restart RipTide:
   ```bash
   systemctl restart riptide-api
   # Or: docker-compose restart riptide-api
   ```

4. Verify capability:
   ```bash
   curl http://localhost:8080/health/capabilities
   # Should show: "cache_backend": "Redis"
   ```

**What Changes**:
- ✅ Cache persists across restarts
- ✅ Sessions survive API restarts
- ✅ ~1-2ms latency increase per cache operation
- ⚠️ Requires Redis maintenance

### 5.3 Upgrading Enhanced → Distributed

**Scenario**: Single-instance production, need to scale horizontally

**Steps**:
1. Update configuration:
   ```toml
   [workers]
   enabled = true
   redis_url = "redis://localhost:6379/1"  # Separate DB
   worker_count = 8
   ```

2. Start worker service:
   ```bash
   # Option 1: Separate binary
   cargo run --bin riptide-worker --config config/deployment/distributed.toml

   # Option 2: Docker Compose
   docker-compose up --scale riptide-worker=2
   ```

3. Scale API instances:
   ```bash
   docker-compose up --scale riptide-api=3
   ```

4. Verify:
   ```bash
   curl http://localhost:8080/health/capabilities
   # Should show: "distributed": true, "async_jobs": true
   ```

**What Changes**:
- ✅ Jobs execute asynchronously
- ✅ Horizontal scaling enabled
- ✅ Better resource utilization
- ⚠️ Requires worker monitoring
- ⚠️ Job queue management needed

### 5.4 Downgrading Distributed → Enhanced

**Scenario**: Simplify deployment, remove workers

**Steps**:
1. Stop worker services:
   ```bash
   docker-compose stop riptide-worker
   ```

2. Update configuration:
   ```toml
   [workers]
   enabled = false
   ```

3. Restart API:
   ```bash
   docker-compose restart riptide-api
   ```

**Data Migration**:
- Redis cache remains intact
- Job queue is preserved (jobs will timeout if not processed)
- Sessions continue working

**Recommendations**:
- Wait for job queue to drain before disabling workers
- Check pending jobs: `redis-cli LLEN job:queue`

---

## 6. Progressive Enhancement Strategy

### 6.1 Decision Matrix

| Use Case | Recommended Mode | Rationale |
|----------|-----------------|-----------|
| Learning RipTide | **Minimal** | Zero setup, immediate start |
| Local development | **Minimal** | Fast iteration, no dependencies |
| CI/CD integration | **Minimal** | Fast, reliable, no services |
| Single-server production | **Enhanced** | Persistent cache, better performance |
| High-traffic single-server | **Enhanced** | Redis handles traffic spikes |
| Multi-instance production | **Distributed** | Horizontal scaling, job queue |
| Background processing | **Distributed** | Async jobs, worker pool |
| Enterprise deployment | **Distributed** | Full features, scalability |

### 6.2 When to Upgrade

**Minimal → Enhanced** when:
- ❌ Cache keeps clearing on deploys
- ❌ Session management needed
- ❌ Repeated requests are slow
- ❌ Need better performance

**Enhanced → Distributed** when:
- ❌ Single instance at capacity
- ❌ Need background job processing
- ❌ Require horizontal scaling
- ❌ Processing takes too long (>30s)

### 6.3 Performance Characteristics

| Operation | Minimal (Memory) | Enhanced (Redis) | Distributed |
|-----------|-----------------|------------------|-------------|
| Cache write | ~50ns | ~1-2ms | ~1-2ms |
| Cache read | ~50ns | ~0.5-1ms | ~0.5-1ms |
| Job submit | Synchronous | Synchronous | Async (~5ms) |
| Session load | ~100ns | ~1ms | ~1ms |
| Startup time | <2s | <5s | <10s |
| Memory usage | ~440MB | ~700MB | ~1.2GB/instance |

---

## 7. Feature Availability Matrix

| Feature | Minimal | Enhanced | Distributed |
|---------|---------|----------|-------------|
| **Core Extraction** | ✅ | ✅ | ✅ |
| HTML extraction | ✅ | ✅ | ✅ |
| JavaScript rendering | ✅ | ✅ | ✅ |
| AI schema generation | ✅ | ✅ | ✅ |
| Spider crawling | ✅ | ✅ | ✅ |
| **Caching** | | | |
| In-memory cache | ✅ | ❌ | ❌ |
| Persistent cache | ❌ | ✅ | ✅ |
| Cache TTL | ✅ (runtime) | ✅ (persistent) | ✅ (persistent) |
| Cross-instance cache | ❌ | ❌ | ✅ |
| **Sessions** | | | |
| Browser contexts | ✅ (transient) | ✅ (persistent) | ✅ (persistent) |
| Session persistence | ❌ | ✅ (24h) | ✅ (24h) |
| Cross-instance sessions | ❌ | ❌ | ✅ |
| **Job Processing** | | | |
| Synchronous jobs | ✅ | ✅ | ✅ |
| Asynchronous jobs | ❌ | ❌ | ✅ |
| Job queue | ❌ | ❌ | ✅ |
| Job retry | ❌ | ❌ | ✅ |
| **Scaling** | | | |
| Single instance | ✅ | ✅ | ✅ |
| Horizontal scaling | ❌ | ❌ | ✅ |
| Load balancing | ❌ | ❌ | ✅ |
| **Operational** | | | |
| Zero dependencies | ✅ | ❌ | ❌ |
| Redis required | ❌ | ✅ | ✅ |
| Worker service | ❌ | ❌ | ✅ |
| Health monitoring | ✅ | ✅ | ✅ |
| Metrics/observability | ✅ | ✅ | ✅ |

---

## 8. Error Handling & Fallback Strategy

### 8.1 Configuration Errors

**Scenario**: Invalid configuration
```rust
// Example: Redis backend without URL
[cache]
backend = "redis"
# Missing: redis_url
```

**Behavior**:
```
ERROR: cache.redis_url is required when cache.backend = 'redis'
APPLICATION FAILED TO START
```

**Resolution**: Add `redis_url` or change to `backend = "memory"`

### 8.2 Runtime Failures

**Scenario**: Redis connection fails in Enhanced mode

**Option 1 - Fail Fast** (default):
```rust
let cache = CacheFactory::create(&config.cache)
    .await
    .context("Failed to initialize cache backend")?;
// Application exits with error
```

**Option 2 - Graceful Fallback** (optional):
```rust
let cache = CacheFactory::create_with_fallback(&config.cache).await;
// Falls back to memory cache with warning
```

**Configuration**:
```toml
[cache]
backend = "redis"
redis_url = "redis://localhost:6379/0"
fallback_to_memory = false  # Default: fail fast
```

### 8.3 Worker Failures

**Scenario**: Worker service crashes

**Behavior**:
- Jobs remain in Redis queue
- API continues serving requests
- Worker restarts and resumes processing
- Jobs timeout after `job_timeout` seconds

**Monitoring**:
```bash
# Check job queue depth
redis-cli LLEN job:queue

# Check failed jobs
redis-cli LLEN job:failed
```

---

## 9. Implementation Checklist

### Phase 1.1: Configuration Infrastructure ✓
- [x] Add `CacheBackend` enum to `riptide-config`
- [x] Add `CacheConfig` struct
- [x] Add `WorkerConfig` struct
- [x] Integrate with `RiptideConfig`
- [x] Add validation methods
- [x] Environment variable parsing
- [x] Create three example configs

### Phase 1.2: Architecture Documentation ✓
- [x] Configuration schema specification
- [x] Deployment mode descriptions
- [x] Environment variable mapping
- [x] Validation rules
- [x] Migration guide
- [x] Feature availability matrix

### Future Phases (Not in Scope)
- [ ] Cache factory implementation (Phase 2)
- [ ] Worker abstraction (Phase 3)
- [ ] Integration tests (Phase 7)
- [ ] Docker configurations (Phase 5)

---

## 10. Security Considerations

### 10.1 Redis URL Security

**Risk**: Redis URL contains credentials in plaintext

**Mitigation**:
```toml
# DON'T: Hardcode credentials
redis_url = "redis://user:password@host:6379"

# DO: Use environment variables
redis_url = "${REDIS_URL}"
```

**Best Practice**:
```bash
# Set via secrets management
export REDIS_URL=$(vault read -field=url secret/redis)
```

### 10.2 Memory Cache Security

**Risk**: In-memory cache not encrypted at rest

**Mitigation**:
- Don't cache sensitive data (PII, credentials)
- Use Redis for sensitive data (supports TLS/SSL)
- Implement cache entry encryption if needed

### 10.3 Configuration Validation

**Risk**: Invalid configuration crashes production

**Mitigation**:
- Validate on startup (fail fast)
- Configuration dry-run mode
- Health check includes config validation

```bash
# Test configuration before deploy
riptide-api --config config/production.toml --validate
```

---

## 11. Monitoring & Observability

### 11.1 Configuration Metrics

**Expose via `/health/capabilities` endpoint**:
```json
{
  "cache_backend": "Redis",
  "async_jobs": true,
  "distributed": true,
  "persistent_cache": true,
  "session_persistence": true,
  "uptime_seconds": 3600,
  "config_mode": "distributed"
}
```

### 11.2 Cache Metrics

**Memory Backend**:
```rust
pub struct CacheStats {
    pub hits: usize,
    pub misses: usize,
    pub entries: usize,
    pub memory_bytes: usize,
    pub evictions: usize,
}
```

**Redis Backend**:
```bash
# Monitor via Redis INFO
redis-cli INFO stats
# keyspace_hits, keyspace_misses, evicted_keys
```

### 11.3 Worker Metrics

**When workers enabled**:
```json
{
  "worker_pool_size": 8,
  "jobs_pending": 42,
  "jobs_processing": 3,
  "jobs_completed": 1250,
  "jobs_failed": 12,
  "avg_processing_time_ms": 2500
}
```

---

## 12. Testing Strategy

### 12.1 Unit Tests

```rust
#[test]
fn test_minimal_config_validates() {
    let config = CacheConfig {
        backend: CacheBackend::Memory,
        redis_url: None,
        memory_ttl: 3600,
        max_memory_entries: 10000,
    };
    assert!(config.validate().is_ok());
}

#[test]
fn test_redis_config_requires_url() {
    let config = CacheConfig {
        backend: CacheBackend::Redis,
        redis_url: None,  // Invalid
        ..Default::default()
    };
    assert!(config.validate().is_err());
}
```

### 12.2 Integration Tests

**Test matrix**:
- ✅ API starts with `backend = memory`
- ✅ API starts with `backend = redis` (Redis running)
- ✅ API fails gracefully if Redis unavailable
- ✅ Extraction works identically in both modes
- ✅ Environment variables override config file

### 12.3 Load Tests

**Compare performance**:
```bash
# Minimal mode
ab -n 10000 -c 100 http://localhost:8080/api/extract

# Enhanced mode (same test)
ab -n 10000 -c 100 http://localhost:8080/api/extract
```

**Expected**: <5% difference between modes for first-time requests

---

## 13. Documentation Updates

### 13.1 README Quick Start

Update README with three deployment options:
- 🚀 **Minimal**: `cargo run --config config/deployment/minimal.toml`
- ⚡ **Enhanced**: `docker-compose -f docker-compose.enhanced.yml up`
- 🏢 **Distributed**: `docker-compose up --scale riptide-api=3`

### 13.2 FAQ Updates

Add new FAQ entry:
**"Do I need Redis?"**
- No for development/CI/CD
- Recommended for production
- Required for distributed mode

### 13.3 Architecture Documentation

Update `docs/architecture/cache-layer.md` with backend comparison.

---

## 14. Success Criteria

### Technical Metrics
- ✅ API starts in <5s without Redis (currently panics)
- ✅ Zero breaking changes to existing Redis users
- ✅ All tests pass in both memory and Redis modes
- ✅ Docker images work in all three modes
- ✅ <5% performance difference between backends (first-time requests)

### User Experience Metrics
- ✅ New users can run `cargo run` immediately (no setup)
- ✅ Clear error messages if Redis misconfigured
- ✅ Migration path documented for all transitions
- ✅ Health endpoint shows current configuration

### Operational Metrics
- ✅ Three deployment configs with clear use cases
- ✅ Configuration validation prevents invalid states
- ✅ Environment variables override config files
- ✅ Backward compatibility with existing deployments

---

## 15. Risk Assessment

| Risk | Impact | Probability | Mitigation |
|------|--------|------------|------------|
| Breaking existing deployments | High | Low | Backward compatible defaults |
| Performance regression | Medium | Low | Benchmark both modes |
| Incomplete feature parity | Medium | Low | Feature matrix, clear docs |
| Increased support burden | Low | Medium | Clear modes, health endpoint |
| Redis connection failures | Medium | Medium | Fail fast with clear errors |

---

## Appendix A: Configuration File Locations

| Mode | Config File | Docker Compose | Use Case |
|------|-------------|----------------|----------|
| Minimal | `config/deployment/minimal.toml` | `docker-compose.minimal.yml` | Dev, CI/CD |
| Enhanced | `config/deployment/enhanced.toml` | `docker-compose.simple.yml` | Prod single |
| Distributed | `config/deployment/distributed.toml` | `docker-compose.yml` | Enterprise |

---

## Appendix B: Environment Variable Reference

Complete list of all configuration environment variables:

```bash
# Cache Configuration
CACHE_BACKEND=memory|redis
REDIS_URL=redis://localhost:6379/0
CACHE_MEMORY_TTL=3600
CACHE_MAX_ENTRIES=10000

# Worker Configuration
WORKERS_ENABLED=false|true
WORKERS_REDIS_URL=redis://localhost:6379/1
WORKER_COUNT=4
JOB_TIMEOUT=300
MAX_RETRIES=3

# Server Configuration
RIPTIDE_API_HOST=0.0.0.0
RIPTIDE_API_PORT=8080

# General
CONFIG_PATH=config/deployment/minimal.toml
RUST_LOG=info
```

---

**Document Version**: 1.0
**Last Updated**: 2025-11-12
**Next Review**: After Phase 1 implementation
