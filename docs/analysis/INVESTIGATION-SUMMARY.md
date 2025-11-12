# Redis-Persistence Architecture Investigation - Quick Reference

**Date**: 2025-11-12  
**Investigator**: Claude Code  
**Full Report**: [`redis-persistence-architecture-violations.md`](./redis-persistence-architecture-violations.md)

---

## TL;DR - Critical Findings

**STATUS**: 🚨 **BLOCKS Redis-Optional Implementation**

### The Problem
riptide-persistence violates hexagonal architecture by:
- **4 files** directly import and use Redis (cache.rs, sync.rs, tenant.rs, state.rs)
- **4+ separate Redis pools** (wasteful, non-scalable)
- **3,442 lines** of direct Redis usage
- **100% duplication** with riptide-cache functionality
- **Zero abstraction** - cannot make Redis optional

### The Impact
- ❌ Cannot make Redis optional without this fix
- ❌ Cannot test without real Redis instance
- ❌ Cannot swap backends (DragonflyDB, in-memory, etc.)
- ❌ Wastes 13+ Redis connections per instance
- ❌ Violates dependency inversion principle

### The Solution
**MUST refactor riptide-persistence to use port traits BEFORE attempting Redis-optional.**

**Estimated Effort**: 6-7 days  
**Risk**: HIGH (core infrastructure change)  
**Benefit**: Clean architecture, testability, flexibility

---

## Evidence Summary

### Files Violating Architecture

| File | LOC | Redis Imports | Pool Size | Should Use Trait |
|------|-----|---------------|-----------|------------------|
| `cache.rs` | 718 | ✓ Line 16-17 | 10 connections | CacheStorage |
| `sync.rs` | 601 | ✓ Line 10-11 | 1 connection | CacheStorage |
| `tenant.rs` | 931 | ✓ Line 14-15 | 1 connection | CacheStorage |
| `state.rs` | 1192 | ✓ Line 14-15 | 1 connection | SessionStorage |
| **TOTAL** | **3,442** | **4 violations** | **13+ connections** | |

### Correct Architecture (Exists in riptide-cache)

✅ **RedisStorage** (`src/redis_storage.rs`)
```rust
impl CacheStorage for RedisStorage { ... }  // ✓ Correct
```

✅ **RedisSessionStorage** (`src/adapters/redis_session_storage.rs`)
```rust
impl SessionStorage for RedisSessionStorage { ... }  // ✓ Correct
```

---

## What Needs to Happen

### Phase 1: Stop Redis-Optional Work
- [ ] Halt any Redis feature flag work
- [ ] Review this investigation with team
- [ ] Create refactoring epic/stories

### Phase 2: Execute Refactoring (6-7 days)
- [ ] **Day 1**: cache.rs - Replace Redis with CacheStorage trait
- [ ] **Day 2**: state.rs - Replace Redis with SessionStorage trait
- [ ] **Day 3**: tenant.rs - Replace Redis with CacheStorage trait
- [ ] **Day 4-5**: sync.rs - Create DistributedCoordination port + adapter
- [ ] **Day 6**: Integration testing
- [ ] **Day 7**: Documentation

### Phase 3: Then Resume Redis-Optional
- [ ] Make riptide-cache Redis dependency optional
- [ ] Add alternative adapters (in-memory, DragonflyDB)
- [ ] Update initialization code
- [ ] Complete Redis-optional implementation

---

## Quick Architecture Comparison

### BEFORE (Current - WRONG)
```
riptide-persistence
    ├─ cache.rs       -> redis::Client::open() ❌
    ├─ sync.rs        -> redis::Client::open() ❌
    ├─ tenant.rs      -> redis::Client::open() ❌
    └─ state.rs       -> redis::Client::open() ❌
```

### AFTER (Correct Hexagonal Architecture)
```
riptide-persistence (domain)
    ├─ cache.rs       -> CacheStorage trait ✓
    ├─ sync.rs        -> CacheStorage trait ✓
    ├─ tenant.rs      -> CacheStorage trait ✓
    └─ state.rs       -> SessionStorage trait ✓
                              ↓
                     riptide-types (ports)
                              ↓
                     riptide-cache (adapters)
                         ├─ RedisStorage (impl CacheStorage)
                         └─ RedisSessionStorage (impl SessionStorage)
```

---

## Key Metrics

| Metric | Current | After Refactoring |
|--------|---------|-------------------|
| Redis pools | 4+ separate | 1 shared |
| Redis connections | 13+ | 3-5 (configurable) |
| Direct Redis imports | 4 files | 0 files |
| Port trait usage | 0% | 100% |
| Testability | Requires Redis | Mock adapters |
| Backend flexibility | Hardcoded | Swappable |
| LOC to refactor | 3,442 | - |

---

## Decision

**RECOMMENDATION**: 
1. ✅ **Approve** refactoring (6-7 days)
2. ✅ **Option A**: Big Bang refactoring (all 4 files in one PR)
3. ❌ **Reject** Redis-optional work until this is complete

**RATIONALE**:
- Cannot make Redis optional without this fix
- Current architecture blocks all future flexibility
- 6-7 days is acceptable for correct architecture
- Risk is HIGH but necessary - no workaround exists

---

## Next Steps

1. **Immediate**: Share this report with team
2. **Today**: Create refactoring epic in issue tracker
3. **This week**: Assign developers and start refactoring
4. **Next week**: Complete refactoring
5. **Following week**: Resume Redis-optional work

---

## References

- **Full Report**: [redis-persistence-architecture-violations.md](./redis-persistence-architecture-violations.md)
- **Architecture Docs**: [/docs/04-architecture/HEXAGONAL_ARCHITECTURE.md](../04-architecture/HEXAGONAL_ARCHITECTURE.md)
- **Port Traits**: 
  - `crates/riptide-types/src/ports/cache.rs` (CacheStorage)
  - `crates/riptide-types/src/ports/session.rs` (SessionStorage)
- **Correct Examples**:
  - `crates/riptide-cache/src/redis_storage.rs` (CacheStorage impl)
  - `crates/riptide-cache/src/adapters/redis_session_storage.rs` (SessionStorage impl)

---

**VERDICT**: CRITICAL - Must fix before Redis-optional work can proceed.
