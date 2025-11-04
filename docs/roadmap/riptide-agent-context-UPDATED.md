# RipTide v1 Refactoring Context - UPDATED
## Existing System Enhancement (Not Greenfield!)

## ⚠️ CRITICAL: This is a REFACTORING, not a REWRITE

You are working with an EXISTING, FUNCTIONAL system with 26 crates that already implements:
- Web crawling and extraction
- Browser automation  
- Redis persistence and caching
- HTTP client with retries
- Configuration management
- Monitoring and telemetry
- LLM integration
- And much more...

**Your mission:** Add a v1 API layer and consolidate shared code WITHOUT breaking or rewriting existing functionality.

---

## 🗂️ Current State Inventory

### Existing Crates and Their Functionality

| Crate | What It Already Does | What Needs Adding |
|-------|---------------------|-------------------|
| **riptide-types** | Core types, `RiptideError`, traits | Nothing - use as-is |
| **riptide-config** | Configuration with `ApiConfig`, `SpiderConfig` | Add precedence resolver |
| **riptide-fetch** | HTTP client with retry, rate limiting, robots.txt | Move shared client to utils |
| **riptide-extraction** | HTML/CSS/regex extraction, WASM support | Add strategy chain pattern |
| **riptide-browser** | Browser automation with CDP | Nothing - use as-is |
| **riptide-spider** | Web crawling with 4 strategies | Nothing - call from v1 |
| **riptide-cache** | Redis + local caching | Nothing - use as-is |
| **riptide-persistence** | Redis persistence, state storage | Move shared Redis to utils |
| **riptide-monitoring** | Telemetry, metrics, health checks | Add v1 endpoints |
| **riptide-reliability** | Circuit breakers, retry policies | Nothing - use as-is |
| **riptide-intelligence** | LLM providers (OpenAI, Anthropic) | Nothing - use as-is |
| **riptide-facade** | `SimpleScraper`, high-level interface | Add `run_pipeline()` method |
| **riptide-api** | Current API with 120+ routes | Add v1 routes + shims |
| **riptide-cli** | CLI commands | Update to call v1 endpoints |

### What's Actually NEW (to be created)
1. **riptide-api-types** - DTO crate for v1 API boundary
2. **riptide-validation** - Schema validation layer
3. **riptide-adapters** - Schema version adapters
4. **riptide-utils** - Consolidated utilities (by MOVING existing code)

---

## 📋 Refactoring Strategy

### Phase 1: Consolidate Without Breaking
```rust
// Example: Creating riptide-utils
// DON'T write new Redis implementation
// DO move existing implementation

// riptide-utils/src/redis.rs
pub mod redis {
    // Option 1: Re-export if no changes needed
    pub use riptide_persistence::redis::*;
    
    // Option 2: Move code here and update imports
    // (copy from riptide-persistence/src/redis.rs)
    pub fn create_pool() -> RedisPool {
        // EXISTING CODE MOVED HERE
    }
}
```

### Phase 2: Add v1 API Layer
```rust
// New v1 routes are THIN wrappers
async fn extract_v1(
    Json(dto): Json<ExtractRequestV1>  // NEW DTO type
) -> Result<Json<ExtractResponseV1>> {
    // Convert to existing internal type
    let internal_request = ExtractionRequest::from(dto);
    
    // Call EXISTING extraction logic
    let result = riptide_extraction::extract(internal_request).await?;
    
    // Convert back to DTO
    Ok(Json(result.into()))
}
```

### Phase 3: Enhance, Don't Replace
```rust
// Enhancing existing config
impl riptide_config::ConfigBuilder {
    // EXISTING methods remain
    pub fn new() -> Self { /* existing */ }
    pub fn with_api(config: ApiConfig) -> Self { /* existing */ }
    
    // ADD new capability
    pub fn with_precedence_resolution(/* ... */) -> Self {
        // NEW functionality
    }
}
```

---

## 🔍 How to Approach Each Task

### Before Starting ANY Implementation

1. **Check if it exists:**
```bash
# Search for existing functionality
rg "function_name" --type rust
grep -r "StructName" crates/
```

2. **Review existing code:**
```bash
# Look at what's already there
cat crates/riptide-persistence/src/redis.rs
cat crates/riptide-config/src/lib.rs
```

3. **Understand current patterns:**
```bash
# See how it's currently used
rg "create_pool\(" --type rust -A 3 -B 3
```

### Decision Tree for Every Task

```
Does this functionality exist?
├── YES → Should it be shared?
│   ├── YES → MOVE to riptide-utils
│   └── NO → KEEP in place, call from v1
└── NO → Is it required for v1?
    ├── YES → CREATE it (rare!)
    └── NO → SKIP for now
```

---

## 📁 Code Patterns to Follow

### Pattern: Moving Shared Code
```rust
// Step 1: Identify shared code in riptide-persistence
// crates/riptide-persistence/src/redis.rs
pub fn create_pool() -> RedisPool { /* implementation */ }

// Step 2: Move to riptide-utils
// crates/riptide-utils/src/redis.rs
pub fn create_pool() -> RedisPool { /* SAME implementation */ }

// Step 3: Update riptide-persistence to use utils
// crates/riptide-persistence/Cargo.toml
// [dependencies]
// riptide-utils = { path = "../riptide-utils" }

// crates/riptide-persistence/src/lib.rs
use riptide_utils::redis::create_pool;  // Now from utils
```

### Pattern: Adding v1 Routes
```rust
// DON'T duplicate business logic
// DO create thin API layer

// crates/riptide-api/src/routes/v1/extract.rs
pub async fn extract_handler(
    State(app_state): State<AppState>,
    Json(request): Json<ExtractRequestV1>,
) -> Result<Json<ExtractResponseV1>> {
    // Just transformation + delegation
    let internal = transform_to_internal(request);
    let result = existing_extraction_logic(internal).await?;
    Ok(Json(transform_to_v1(result)))
}
```

### Pattern: Config Enhancement
```rust
// Keep existing config structures
// Add new capabilities

// crates/riptide-config/src/lib.rs
pub struct ConfigResolver {
    // Uses EXISTING ConfigBuilder internally
    builder: ConfigBuilder,
}

impl ConfigResolver {
    pub fn resolve_with_precedence(
        &self,
        request: Option<RequestConfig>,
        profile: Profile,
    ) -> Config {
        // NEW precedence logic using EXISTING config types
        self.builder
            .with_request(request)
            .with_profile(profile)
            .with_defaults()
            .build()
    }
}
```

---

## 🚫 Anti-Patterns to Avoid

### ❌ DON'T: Rewrite Working Code
```rust
// WRONG - Writing new Redis pool implementation
pub fn new_redis_pool() -> Pool {
    // Brand new implementation
}

// RIGHT - Use existing implementation
pub use riptide_persistence::redis::create_pool;
```

### ❌ DON'T: Duplicate Business Logic
```rust
// WRONG - Reimplementing extraction
async fn extract_v1(url: String) {
    let html = fetch_html(url);  // New implementation
    let extracted = parse_html(html);  // New implementation
}

// RIGHT - Call existing extraction
async fn extract_v1(url: String) {
    riptide_extraction::extract(url).await  // Use existing
}
```

### ❌ DON'T: Break Existing Tests
```rust
// WRONG - Changing behavior that breaks tests
pub fn process(data: &str) -> Result<Output> {
    // New behavior that breaks existing tests
}

// RIGHT - Maintain compatibility
pub fn process(data: &str) -> Result<Output> {
    // Same behavior, possibly refactored location
}
```

---

## 🎯 Success Criteria

### After Each Change, Verify:
1. **Existing tests still pass:** `cargo test -p <affected-crate>`
2. **No duplicate code:** Check with `rg "function_name"`
3. **Imports updated:** `cargo check --workspace`
4. **Feature parity:** Can still do everything that worked before

### End State Requirements:
- ✅ All 26 crates still compile and test successfully
- ✅ v1 API provides access to ALL existing functionality
- ✅ Shared utilities consolidated in riptide-utils
- ✅ Legacy routes work via shims (with deprecation headers)
- ✅ No loss of functionality
- ✅ No unnecessary code duplication

---

## 🔧 Debugging When Things Break

### If Tests Fail After Moving Code:
```bash
# Check what's missing
cargo test -p riptide-persistence 2>&1 | grep "unresolved import"

# Verify the moved code location
ls crates/riptide-utils/src/

# Update imports
rg "use.*create_pool" --type rust
```

### If Functionality is Missing:
```bash
# Find where it was
git grep "function_name" $(git rev-list --all)

# See what depends on it
cargo tree -i riptide-persistence

# Check if it's just moved
rg "function_name" --type rust
```

---

## 📚 Quick Reference for Existing Code

### Where to Find Key Functionality:
- **Redis Operations**: `crates/riptide-persistence/src/` and `crates/riptide-cache/src/`
- **HTTP Client**: `crates/riptide-fetch/src/client.rs`
- **Extraction Logic**: `crates/riptide-extraction/src/`
- **Spider/Crawl**: `crates/riptide-spider/src/`
- **Browser Automation**: `crates/riptide-browser/src/`
- **Configuration**: `crates/riptide-config/src/`
- **Error Types**: `crates/riptide-types/src/error.rs`
- **Monitoring**: `crates/riptide-monitoring/src/`

### Common Imports After Refactoring:
```rust
use riptide_utils::{redis, http, error};  // Consolidated utilities
use riptide_types::{RiptideError, Result};  // Core types
use riptide_config::ConfigResolver;  // Enhanced config
use riptide_facade::run_pipeline;  // Unified pipeline
use riptide_api_types::v1::*;  // v1 DTOs (NEW)
```

---

## 🎯 Your Primary Objective

**Add a v1 API layer and consolidate shared utilities WITHOUT breaking the existing, working system.**

Remember:
- This is a REFACTORING, not a rewrite
- Most code already exists and works
- Your job is to organize and expose it better
- When in doubt, check what exists first
- Move code rather than duplicating it
- Test frequently to ensure nothing breaks

---

*End of Updated Context - Focus on ENHANCING, not REPLACING*
