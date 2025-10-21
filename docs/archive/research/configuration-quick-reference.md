# Configuration Quick Reference Guide

**Quick decision matrix for RipTide configuration management**

---

## Quick Decision Tree

```
Is it sensitive data (API keys, passwords)?
├─ YES → Environment Variable (.env + secret manager)
└─ NO → Continue...

Will it change between deployments (dev/staging/prod)?
├─ YES → Environment Variable
└─ NO → Continue...

Is it a complex hierarchy with multiple related values?
├─ YES → TOML Configuration File
└─ NO → Continue...

Does it need to be tuned frequently in production?
├─ YES → TOML with hot-reload support
└─ NO → Continue...

Is it a system invariant that never changes?
├─ YES → Code Constant
└─ NO → Environment Variable (default choice)
```

---

## What Goes Where

### Environment Variables (.env)

**Always:**
- ✅ `ANTHROPIC_API_KEY` - API secrets
- ✅ `OPENAI_API_KEY` - API secrets
- ✅ `SERPER_API_KEY` - API secrets
- ✅ `REDIS_URL` - Connection strings
- ✅ `NODE_ID` - Deployment identifiers
- ✅ `RUST_ENV` - Environment name

**Maybe:**
- ⚠️ `RIPTIDE_SERVER_PORT` - If needs to change per deployment
- ⚠️ `ENABLE_TELEMETRY` - Feature flags
- ⚠️ `LOG_LEVEL` - Operational toggles

**Never:**
- ❌ Complex nested configuration
- ❌ Lists or arrays
- ❌ Algorithm parameters

### TOML Configuration Files

**Always:**
- ✅ Thread pool sizes
- ✅ Timeout configurations
- ✅ Circuit breaker settings
- ✅ Rate limiting rules
- ✅ Message retention policies
- ✅ Domain-specific routing rules
- ✅ Feature weights/scoring

**Current Examples:**
- ✅ `config/gate_thresholds.toml` - Excellent pattern!
- ✅ A/B testing variants
- ✅ Domain overrides

**Structure:**
```toml
[section.subsection]
setting = value
nested_setting = { key = "value" }

[[arrays]]
name = "item1"
[[arrays]]
name = "item2"
```

### Code Constants

**Always:**
- ✅ `MAX_URL_LENGTH = 2048`
- ✅ `MAX_HEADER_SIZE = 8192`
- ✅ `WASM_MAX_MEMORY_PAGES = 256`
- ✅ Protocol constants
- ✅ Magic numbers with comments

**Pattern:**
```rust
/// Maximum URL length per RFC 2616
pub const MAX_URL_LENGTH: usize = 2048;

/// WASM memory limit (16MB = 256 pages * 64KB)
pub const WASM_MAX_MEMORY_PAGES: u32 = 256;
```

---

## System-Specific Guidelines

### Event Mesh / Message Broker

| Setting | Location | Example |
|---------|----------|---------|
| Broker URLs | Environment | `KAFKA_BROKERS=broker1:9092,broker2:9092` |
| Partition count | TOML | `max_partitions = 100` |
| Retention policy | TOML | `message_retention_hours = 24` |
| Compression | TOML | `compression_codec = "lz4"` |
| Throughput limits | TOML | `max_messages_per_second = 10000` |

### WebAssembly Runtime

| Setting | Location | Example |
|---------|----------|---------|
| Memory limit | Code Constant | `const WASM_MAX_MEMORY: u32 = 256;` |
| Timeout | TOML | `module_timeout_secs = 10` |
| Instance count | TOML | `instances_per_worker = 1` |
| Cache directory | Environment | `WASM_CACHE_DIR=/var/cache/wasm` |
| Enable AOT | TOML | `enable_aot_cache = true` |

### Distributed Systems

| Setting | Location | Example |
|---------|----------|---------|
| Node ID | Environment | `NODE_ID=node-prod-01` |
| Cluster name | TOML | `cluster.name = "production"` |
| Consensus timeout | TOML | `consensus_timeout_ms = 5000` |
| Service discovery | Environment | `CONSUL_ADDR=http://consul:8500` |
| Health check | TOML | `health_check_interval_secs = 30` |

### Performance-Critical

| Setting | Location | Example |
|---------|----------|---------|
| Buffer sizes | Code Constant | `const BUFFER_SIZE: usize = 8192;` |
| Worker threads | TOML + Env | `workers = 8` + `RIPTIDE_WORKERS=16` |
| Connection pools | TOML | `max_connections = 1000` |
| SIMD features | Cargo Feature | `features = ["simd"]` |
| Cache sizes | TOML | `cache_size_mb = 512` |

---

## Security Checklist

### Development
- [ ] Create `.env` file for local secrets
- [ ] Never commit `.env` to git
- [ ] Use `.env.example` with placeholders
- [ ] Add `.env` to `.gitignore`

### Code
- [ ] Use `env::var()` for all secrets
- [ ] Implement `Debug` redaction for secret types
- [ ] Never log secret values
- [ ] Fail fast if required secrets missing

### Production
- [ ] Use Kubernetes Secrets, Vault, or cloud provider
- [ ] Rotate secrets regularly
- [ ] Use short-lived tokens when possible
- [ ] Monitor for secret access patterns

---

## Configuration Loading Priority

**Highest → Lowest Priority:**

1. **Environment Variables** (runtime, per-instance)
   ```bash
   RIPTIDE_SERVER_PORT=8080
   ```

2. **Local Config File** (gitignored, developer-specific)
   ```toml
   # config/local.toml (not in git)
   [server]
   port = 3001
   ```

3. **Environment-Specific Config** (per deployment environment)
   ```toml
   # config/production.toml
   [server]
   workers = 32
   ```

4. **Default Config** (baseline for all environments)
   ```toml
   # config/default.toml
   [server]
   port = 3000
   workers = 8
   ```

5. **Code Defaults** (fallback in struct implementation)
   ```rust
   impl Default for ServerConfig {
       fn default() -> Self {
           Self { port: 3000, workers: 8 }
       }
   }
   ```

---

## Implementation Quick Start

### Step 1: Add Dependencies (5 minutes)

```toml
[dependencies]
config = { version = "0.14", features = ["toml"] }
dotenvy = "0.15"
serde = { version = "1", features = ["derive"] }
```

### Step 2: Create Config Structure (10 minutes)

```rust
use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub redis: RedisConfig,
}

impl AppConfig {
    pub fn load() -> Result<Self, ConfigError> {
        dotenvy::dotenv().ok();

        Config::builder()
            .add_source(File::with_name("config/default"))
            .add_source(Environment::with_prefix("RIPTIDE").separator("__"))
            .build()?
            .try_deserialize()
    }
}
```

### Step 3: Create Config Files (15 minutes)

```toml
# config/default.toml
[server]
port = 3000
workers = 8

[redis]
url = "redis://localhost:6379"
pool_size = 10
```

### Step 4: Create .env.example (5 minutes)

```bash
# .env.example
ANTHROPIC_API_KEY=your_key_here
OPENAI_API_KEY=your_key_here
REDIS_URL=redis://localhost:6379
RUST_ENV=development
```

### Step 5: Update .gitignore (2 minutes)

```gitignore
.env
.env.local
config/local.toml
secrets/
```

**Total Time: ~37 minutes**

---

## Common Patterns

### Pattern: Environment Override

```toml
# config/default.toml
[server]
port = 3000

# Override in production:
# RIPTIDE_SERVER_PORT=8080
```

### Pattern: Secret with Fallback

```rust
let api_key = env::var("OPENAI_API_KEY")
    .map_err(|_| ConfigError::MissingSecret("OPENAI_API_KEY"))?;
```

### Pattern: Optional Feature

```toml
# config/default.toml
[features]
enable_telemetry = false

# Override: RIPTIDE_FEATURES_ENABLE_TELEMETRY=true
```

### Pattern: Validated Config

```rust
impl AppConfig {
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.server.port == 0 {
            return Err(ConfigError::Message("Invalid port".into()));
        }
        Ok(())
    }
}
```

---

## Environment Variable Naming

### Good Names
✅ `RIPTIDE_SERVER_PORT`
✅ `RIPTIDE_REDIS_URL`
✅ `ANTHROPIC_API_KEY`
✅ `ENABLE_TELEMETRY`

### Bad Names
❌ `PORT` (too generic)
❌ `URL` (ambiguous)
❌ `API_KEY` (which API?)
❌ `TIMEOUT` (timeout for what?)

### Convention

```
{SERVICE}_{COMPONENT}_{SETTING}
RIPTIDE_INTELLIGENCE_MAX_RETRIES

Secrets: Use common names
OPENAI_API_KEY (not RIPTIDE_OPENAI_API_KEY)
```

---

## Troubleshooting

### "Secret not found"
1. Check `.env` file exists
2. Check variable name spelling
3. Check `dotenv()` is called before reading
4. In production, check secret manager integration

### "Config validation failed"
1. Check TOML syntax with `toml-cli validate`
2. Check all required fields present
3. Check types match (string vs number)
4. Check enum values are valid

### "Environment override not working"
1. Check prefix matches (`RIPTIDE_` by default)
2. Check separator (use `__` for nested: `RIPTIDE_SERVER__PORT`)
3. Check env var is set: `echo $RIPTIDE_SERVER_PORT`
4. Check parsing succeeds (try_parsing = true)

---

## Migration Checklist

Migrating from hardcoded config to proper system:

- [ ] Audit all configuration usage
- [ ] Identify secrets and move to environment variables
- [ ] Create default TOML configuration
- [ ] Create environment-specific TOML files
- [ ] Implement unified config loader
- [ ] Add validation
- [ ] Update documentation
- [ ] Add tests for config loading
- [ ] Update deployment scripts
- [ ] Test in staging environment

---

## References

- Full guide: `docs/research/rust-configuration-best-practices.md`
- 12-Factor App: https://12factor.net/config
- config-rs docs: https://docs.rs/config/
- OWASP Secrets: https://cheatsheetseries.owasp.org/cheatsheets/Secrets_Management_Cheat_Sheet.html

---

**Last Updated**: 2025-10-20
**Maintainer**: RipTide Team
