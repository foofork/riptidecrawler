# 🎯 RipTide v1 Quick Reference - UPDATED
## REFACTOR Don't Rewrite!

## 🚨 BEFORE YOU CODE - ALWAYS CHECK

```bash
# 1. Does this functionality already exist?
rg "function_name" --type rust
grep -r "impl.*StructName" crates/

# 2. Where is it currently?
find crates -name "*.rs" -exec grep -l "redis::Pool" {} \;

# 3. What depends on it?
cargo tree -p riptide-persistence -i

# 4. Can I just move it?
# If yes -> MOVE it, don't rewrite it!
```

## 📍 Where Things Already Live

| What You Need | Where It Already Exists | Action |
|---------------|------------------------|---------|
| Redis pool | `riptide-persistence/src/redis.rs` | MOVE to utils |
| HTTP client | `riptide-fetch/src/client.rs` | MOVE to utils |
| Error types | `riptide-types/src/error.rs` | RE-EXPORT |
| Config types | `riptide-config/src/` | ENHANCE in place |
| Extraction | `riptide-extraction/src/` | USE as-is |
| Crawling | `riptide-spider/src/` | USE as-is |
| Browser | `riptide-browser/src/` | USE as-is |
| Caching | `riptide-cache/src/` | USE as-is |
| Monitoring | `riptide-monitoring/src/` | ENHANCE in place |

## ✅ RIGHT Way - Moving Code

```rust
// Step 1: Find existing code
// crates/riptide-persistence/src/redis.rs
pub fn create_pool() -> RedisPool { /* existing */ }

// Step 2: Move to utils (copy the EXACT code)
// crates/riptide-utils/src/redis.rs  
pub fn create_pool() -> RedisPool { /* SAME CODE */ }

// Step 3: Update original to import from utils
// crates/riptide-persistence/src/lib.rs
use riptide_utils::redis::create_pool;

// Step 4: Verify nothing broke
// cargo test -p riptide-persistence
```

## ❌ WRONG Way - Rewriting

```rust
// DON'T DO THIS - Creating new implementation
pub fn new_redis_connection() -> Connection {
    // Writing from scratch - NO!
}

// When this already exists in riptide-persistence!
```

## 🔄 v1 API Pattern - Thin Wrappers Only

```rust
// RIGHT - v1 route just converts and delegates
async fn extract_v1(Json(dto): Json<ExtractRequestV1>) {
    // 1. Convert DTO to existing type
    let internal = ExtractionRequest::from(dto);
    
    // 2. Call EXISTING logic
    let result = riptide_extraction::extract(internal).await?;
    
    // 3. Convert back to DTO
    Ok(Json(ExtractResponseV1::from(result)))
}

// WRONG - Reimplementing logic
async fn extract_v1(Json(dto): Json<ExtractRequestV1>) {
    // DON'T reimplement extraction here!
}
```

## 📋 Consolidation Checklist

When creating `riptide-utils`:

- [ ] Check `riptide-persistence` for Redis code → MOVE IT
- [ ] Check `riptide-fetch` for HTTP client → MOVE IT
- [ ] Check `riptide-types` for errors → RE-EXPORT
- [ ] Check multiple crates for duplicate helpers → CONSOLIDATE
- [ ] Update all imports after moving
- [ ] Run tests to verify nothing broke

## 🔍 Quick Commands

```bash
# Find duplicate implementations
for func in "create_pool" "http_client" "parse_config"; do
  echo "=== $func ==="
  rg "$func" --type rust -l
done

# Check what a crate exports
cargo doc -p riptide-persistence --open

# See existing API routes
ls -la crates/riptide-api/src/routes/

# Verify imports after moving code
cargo check --workspace 2>&1 | grep "unresolved import"

# Test that nothing broke
cargo test --workspace
```

## 🗺️ Task Priority Map

```
1. ENHANCE riptide-config (add precedence)
   └── Don't rewrite existing ConfigBuilder
   
2. CREATE riptide-utils (by MOVING code)
   ├── Move redis from riptide-persistence
   ├── Move http from riptide-fetch
   └── Re-export from riptide-types

3. CREATE riptide-api-types (NEW - just DTOs)
   └── Thin types that convert to/from existing

4. ADD v1 routes to riptide-api
   └── Call existing logic, don't reimplement

5. ENHANCE riptide-facade  
   └── Add run_pipeline() that calls existing logic
```

## 🚦 Gate Checks After Each Step

```bash
# After moving code to utils
cargo test -p riptide-utils
cargo test -p riptide-persistence  # Should still work!

# After adding v1 routes  
curl localhost:8080/api/v1/healthz
# Old routes should still work:
curl localhost:8080/health  

# After config changes
cargo test -p riptide-config
```

## ⚡ Speed Tips

### Finding Code Fast
```bash
# Find struct definitions
rg "^pub struct" --type rust

# Find public functions
rg "^pub fn" --type rust  

# Find trait implementations
rg "^impl.*for" --type rust

# See what's exported
rg "^pub use" --type rust
```

### Moving Code Workflow
```bash
# 1. Copy the file
cp crates/riptide-persistence/src/redis.rs \
   crates/riptide-utils/src/redis.rs

# 2. Update imports in the new location
sed -i 's/crate::/riptide_utils::/g' \
   crates/riptide-utils/src/redis.rs

# 3. Replace implementation with import in original
echo "pub use riptide_utils::redis::*;" > \
   crates/riptide-persistence/src/redis.rs

# 4. Fix compilation errors
cargo check --workspace
```

## 🎯 Success Metrics

### You're doing it RIGHT if:
- ✅ Existing tests still pass
- ✅ No new implementations of existing functionality
- ✅ Code is moved, not duplicated
- ✅ v1 routes are thin wrappers
- ✅ Imports are updated everywhere

### You're doing it WRONG if:
- ❌ Writing new Redis/HTTP/Config from scratch
- ❌ Tests that passed now fail
- ❌ Same code exists in multiple places
- ❌ v1 routes contain business logic
- ❌ Can't find where functionality went

## 🆘 Panic Commands

```bash
# Everything is broken!
git stash
cargo clean
cargo build --workspace

# Can't find where code went
git diff --name-status HEAD~1

# Tests failing after refactor
git diff HEAD~1 crates/*/Cargo.toml  # Check deps
cargo test --workspace 2>&1 | head -20

# Rollback a crate
cd crates/riptide-persistence
git checkout HEAD~1 .
```

---

**Remember:** You're REFACTORING a working system, not building a new one. When in doubt, find and use existing code!
