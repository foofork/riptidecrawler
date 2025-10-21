# Configuration Recommendations Summary

**Executive Summary for RipTide Event Mesh Project**

---

## The Golden Rules

### 1. Secrets → Environment Variables
**Always use environment variables for sensitive data**

```bash
# ✅ Correct
ANTHROPIC_API_KEY=sk-ant-actual-key

# ❌ Wrong
const ANTHROPIC_API_KEY: &str = "sk-ant-...";  // Never!
```

### 2. Complex Config → TOML Files
**Use TOML for hierarchical, operational settings**

```toml
# ✅ Correct
[circuit_breaker]
failure_threshold = 50
min_requests = 5
recovery_timeout_secs = 60
```

### 3. Invariants → Code Constants
**Use constants for values that never change**

```rust
// ✅ Correct
pub const MAX_URL_LENGTH: usize = 2048;
pub const WASM_MAX_MEMORY_PAGES: u32 = 256;
```

---

## Current Codebase Assessment

### ✅ What's Working Well

1. **Excellent TOML usage** in `config/gate_thresholds.toml.example`
   - Complex hierarchical configuration
   - A/B testing support
   - Domain-specific overrides
   - Well-documented with comments

2. **Good environment variable patterns** in intelligence config
   - Proper use of `env::var()` for API keys
   - Environment-based provider discovery
   - Fallback to defaults

3. **Strong constant definitions** in validation module
   - Clear, documented constants
   - Type-safe system limits
   - Good use of `pub const`

### ⚠️ Areas for Improvement

1. **No unified configuration system**
   - Configuration logic scattered across crates
   - Manual environment variable parsing
   - No centralized validation

2. **Missing .env support**
   - No `.env.example` file
   - No `dotenvy` integration for local development
   - Secrets management undocumented

3. **Inconsistent patterns**
   - Some configs use manual env parsing
   - Others use structured approach
   - No standard configuration loading

---

## Recommended Architecture

### Layered Configuration System

```
┌─────────────────────────────────────────┐
│   Environment Variables (Highest)       │ ← Runtime, per-instance
│   RIPTIDE_SERVER_PORT=8080              │
└─────────────────────────────────────────┘
                  ↓
┌─────────────────────────────────────────┐
│   Local Config (Developer-specific)     │ ← Gitignored
│   config/local.toml                     │
└─────────────────────────────────────────┘
                  ↓
┌─────────────────────────────────────────┐
│   Environment Config (Per-deployment)   │ ← Versioned
│   config/production.toml                │
└─────────────────────────────────────────┘
                  ↓
┌─────────────────────────────────────────┐
│   Default Config (Base)                 │ ← Versioned
│   config/default.toml                   │
└─────────────────────────────────────────┘
                  ↓
┌─────────────────────────────────────────┐
│   Code Defaults (Fallback)              │ ← Built-in
│   impl Default for Config               │
└─────────────────────────────────────────┘
```

---

## Specific Recommendations

### For Event Mesh / Message Broker

**Environment Variables:**
- `KAFKA_BROKERS` - Broker connection strings
- `REDIS_URL` - Persistence connection
- `NODE_ID` - Unique instance identifier

**TOML Configuration:**
```toml
[broker]
max_partitions = 100
message_retention_hours = 24
compression_codec = "lz4"
delivery_guarantee = "at_least_once"

[scaling]
min_workers = 2
max_workers = 16
scale_threshold = 0.8
```

### For WebAssembly Services

**Code Constants:**
```rust
pub const WASM_MAX_MEMORY_PAGES: u32 = 256;  // 16MB
pub const WASM_MAX_TABLE_SIZE: u32 = 1000;
pub const WASM_MAX_STACK_SIZE: usize = 1024 * 1024;
```

**TOML Configuration:**
```toml
[wasm]
instances_per_worker = 1
module_timeout_secs = 10
enable_aot_cache = true
cache_size_mb = 100
```

**Environment Variables:**
```bash
WASM_CACHE_DIR=/var/cache/riptide/wasm
```

### For Distributed Systems

**Environment Variables:**
```bash
NODE_ID=node-prod-01
NODE_REGION=us-east-1
CONSUL_ADDR=http://consul:8500
CLUSTER_JOIN_ADDRESSES=node1:8080,node2:8080
```

**TOML Configuration:**
```toml
[cluster]
name = "riptide-production"
topology = "mesh"

[consensus]
algorithm = "raft"
election_timeout_ms = 1000
heartbeat_interval_ms = 100
min_quorum_size = 3
```

### For Performance-Critical Configs

**Compile-Time (Cargo Features):**
```toml
[features]
default = ["wasm-extraction", "simd-optimizations"]
wasm-extraction = ["wasmtime"]
simd-optimizations = []
```

**Runtime (TOML):**
```toml
[performance]
worker_threads = 8
max_connections = 1000
connection_timeout_ms = 5000

[caching]
enable_hot_cache = true
hot_cache_size_mb = 512
```

---

## Implementation Plan

### Phase 1: Foundation (Week 1) - PRIORITY

**Goals**: Security and basic infrastructure

**Tasks:**
1. Add `dotenvy` and `config` crates to workspace
2. Create `.env.example` with all secrets documented
3. Update `.gitignore` to exclude `.env` and `secrets/`
4. Audit codebase for hardcoded secrets
5. Move all secrets to environment variables
6. Add `Debug` redaction for secret types

**Deliverables:**
```
.env.example
.gitignore (updated)
docs/configuration-guide.md
```

**Code Changes:**
```rust
// Before
const API_KEY: &str = "hardcoded-key";  // ❌

// After
let api_key = env::var("ANTHROPIC_API_KEY")
    .map_err(|_| ConfigError::MissingSecret("ANTHROPIC_API_KEY"))?;  // ✅
```

### Phase 2: Unified Config (Week 2-3)

**Goals**: Centralized configuration system

**Tasks:**
1. Enhance `riptide-config` crate with unified loader
2. Create configuration structure:
   ```
   config/
   ├── default.toml
   ├── development.toml
   ├── production.toml
   ├── eventmesh.toml
   └── local.toml.example
   ```
3. Implement config validation
4. Add configuration tests
5. Document configuration options

**Deliverables:**
```rust
// Unified configuration loader
pub struct RiptideConfig { /* ... */ }

impl RiptideConfig {
    pub fn load() -> Result<Self, ConfigError> {
        // Load from all sources with proper precedence
    }
}
```

### Phase 3: Advanced Features (Week 4)

**Goals**: Operational excellence

**Tasks:**
1. Hot-reload support for non-critical configs
2. Configuration profiles (dev, staging, prod)
3. Configuration documentation generation
4. Monitoring dashboard for config values
5. Configuration versioning

---

## Quick Reference

### Decision Matrix

| Type | Location | Example | Override? |
|------|----------|---------|-----------|
| API Keys | `.env` | `ANTHROPIC_API_KEY=sk-...` | No |
| Credentials | `.env` | `REDIS_URL=redis://...` | No |
| Feature Flags | `.env` | `ENABLE_TELEMETRY=true` | Yes |
| Timeouts | TOML | `timeout_secs = 30` | Via env |
| Thread Pools | TOML | `workers = 8` | Via env |
| Rate Limits | TOML | `max_rps = 1000` | Via env |
| System Limits | Code | `const MAX_SIZE = 2048` | No |
| Algorithms | Code | `const THRESHOLD = 0.8` | No |

### Environment Variable Pattern

```bash
# Format: {SERVICE}_{COMPONENT}_{SETTING}
RIPTIDE_SERVER_PORT=3000
RIPTIDE_REDIS_URL=redis://localhost:6379
RIPTIDE_WASM_MAX_MEMORY_MB=128

# Secrets: Use common names
ANTHROPIC_API_KEY=sk-ant-...
OPENAI_API_KEY=sk-...
```

### TOML Override Pattern

```bash
# Setting in config/default.toml
[server]
port = 3000

# Override via environment variable
export RIPTIDE_SERVER_PORT=8080
```

---

## Security Checklist

### Development
- [ ] `.env` file for local secrets
- [ ] `.env.example` committed with placeholders
- [ ] `.env` in `.gitignore`
- [ ] Secrets never in source code

### Production
- [ ] Use Kubernetes Secrets / Vault / Cloud provider
- [ ] No secrets in container images
- [ ] Secrets rotated regularly
- [ ] Audit logging for secret access

### Code
- [ ] Use `env::var()` for all secrets
- [ ] Implement `Debug` redaction
- [ ] Never log secret values
- [ ] Fail fast if required secrets missing

---

## Crate Recommendations

### Primary: config-rs

```toml
[dependencies]
config = { version = "0.14", features = ["toml"] }
```

**Why?**
- Industry standard for Rust web services
- Supports TOML, YAML, JSON, environment variables
- Layered configuration with overrides
- Excellent error messages
- Active maintenance

### Secondary: dotenvy

```toml
[dependencies]
dotenvy = "0.15"
```

**Why?**
- Simple `.env` file loading
- Zero configuration
- Maintained fork of `dotenv`
- Essential for local development

### Optional: figment

```toml
[dependencies]
figment = { version = "0.10", features = ["toml", "env"] }
```

**Why?**
- Profile-based configuration
- Excellent for Rocket framework users
- Best-in-class error messages
- Strong type safety

---

## Benefits Summary

### Security
✅ Secrets never committed to git
✅ Environment-based credential management
✅ Production-ready secret handling
✅ Audit trail for configuration changes

### Maintainability
✅ Single source of truth for configuration
✅ Clear separation of concerns
✅ Version-controlled configuration
✅ Easy to review and update

### Operations
✅ Environment-specific configurations
✅ Hot-reload support (non-critical settings)
✅ Validation before deployment
✅ Configuration monitoring

### Development
✅ Local overrides without git conflicts
✅ Easy onboarding with `.env.example`
✅ Type-safe configuration
✅ Fast iteration

---

## Next Steps

1. **Read** full guide: `docs/research/rust-configuration-best-practices.md`
2. **Review** quick reference: `docs/research/configuration-quick-reference.md`
3. **Implement** Phase 1 (security foundation)
4. **Test** in development environment
5. **Deploy** to staging
6. **Monitor** for issues
7. **Iterate** based on feedback

---

## Resources

- **Full Research**: `/workspaces/eventmesh/docs/research/rust-configuration-best-practices.md`
- **Quick Reference**: `/workspaces/eventmesh/docs/research/configuration-quick-reference.md`
- **12-Factor App**: https://12factor.net/config
- **config-rs**: https://docs.rs/config/
- **OWASP Secrets**: https://cheatsheetseries.owasp.org/

---

**Prepared by**: Research Agent (Researcher Role)
**Date**: 2025-10-20
**Status**: ✅ Ready for Implementation
**Priority**: 🔥 High (Security & Maintainability)
