# ADR-001: Distributed Coordination Port Trait

**Status**: Proposed
**Date**: 2025-11-12
**Deciders**: System Architect, Development Team
**Related**: Phase 0 Hexagonal Refactoring, riptide-persistence crate

---

## Context

The `riptide-persistence` crate's `sync.rs` module currently has direct dependencies on Redis for distributed cache synchronization and cluster coordination. This creates several problems:

### Current Architecture Violations

1. **Direct Infrastructure Coupling**
   - Line 10: `use redis::aio::MultiplexedConnection;`
   - Line 11: `use redis::{AsyncCommands, Client};`
   - Line 183: `conn.publish::<_, _, ()>(&channel, &message).await?;`
   - Business logic directly calls Redis methods

2. **Hexagonal Architecture Violations**
   - No port abstraction between domain and infrastructure
   - Cannot swap Redis for alternative messaging systems
   - Violates Dependency Inversion Principle

3. **Testing Challenges**
   - Unit tests require Redis to be running
   - Cannot mock distributed operations
   - Slow test execution
   - Complex CI/CD setup

4. **Backend Lock-in**
   - Impossible to use NATS, Kafka, RabbitMQ, or other messaging systems
   - Cannot use in-memory coordination for local development
   - Difficult to implement custom coordination strategies

### Example of Current Violation

```rust
// sync.rs:172-184 - Business logic directly coupled to Redis
pub async fn notify_operation(&self, operation: SyncOperation) -> PersistenceResult<()> {
    let channel = format!("riptide:sync:{}", "operations");
    let message = serde_json::to_string(&operation)?;

    let mut conn = self.pool.lock().await;  // ❌ Direct Redis access
    conn.publish::<_, _, ()>(&channel, &message).await?;  // ❌ Redis-specific method

    Ok(())
}
```

**Problems**:
- `DistributedSync` knows about Redis connection pools
- `notify_operation` uses Redis-specific pub/sub commands
- Cannot test without Redis infrastructure
- Cannot switch to alternative messaging backends

---

## Decision

We will introduce a **`DistributedCoordination` port trait** in `riptide-types` that abstracts distributed primitives needed for cache synchronization and cluster coordination.

### Trait Responsibilities

The trait will provide:

1. **Pub/Sub Messaging**
   - `publish(channel, message)` - Fire-and-forget messaging
   - `subscribe(pattern)` - Stream-based message consumption
   - `publish_with_ack(channel, message, timeout)` - Confirmed delivery

2. **Distributed Key Operations**
   - `keys(pattern)` - Pattern-based key discovery
   - `delete_many(keys)` - Bulk key deletion
   - `flush_all()` - Clear all keys

3. **Cluster Coordination**
   - `register_node(id, metadata)` - Node registration
   - `heartbeat(id, ttl)` - Liveness signaling
   - `list_nodes()` - Active node discovery
   - `get_node_metadata(id)` - Node information retrieval

4. **Leader Election**
   - `try_acquire_leadership(id, ttl)` - Leadership acquisition
   - `release_leadership(id)` - Voluntary step-down
   - `get_leader()` - Current leader query

5. **Health & Diagnostics**
   - `health_check()` - Backend availability check
   - `stats()` - Coordination metrics

### Trait Location

```
crates/riptide-types/src/ports/coordination.rs
```

### Adapter Implementations

1. **RedisCoordination** - Production Redis backend
   - Location: `crates/riptide-persistence/src/adapters/redis_coordination.rs`
   - Uses Redis pub/sub, KEYS, DEL, SET NX EX patterns

2. **MemoryCoordination** - Testing/development backend
   - Location: `crates/riptide-persistence/src/adapters/memory_coordination.rs`
   - In-memory HashMap with broadcast channels
   - Zero external dependencies

### Dependency Injection Pattern

```rust
// Before: Direct Redis coupling
impl DistributedSync {
    pub async fn new(redis_url: &str, config: DistributedConfig) -> PersistenceResult<Self> {
        let client = Client::open(redis_url)?;  // ❌ Direct Redis
        let conn = client.get_multiplexed_tokio_connection().await?;
        // ...
    }
}

// After: Port trait injection
impl DistributedSync {
    pub fn new(
        config: DistributedConfig,
        coordination: Arc<dyn DistributedCoordination>,  // ✅ Injected dependency
    ) -> Self {
        Self {
            config,
            coordination,
            consensus: Arc::new(ConsensusManager::new(config.clone())),
            // ...
        }
    }
}
```

### Composition Root

```rust
// Production: main.rs or application initialization
async fn initialize_production() -> anyhow::Result<DistributedSync> {
    let redis_url = std::env::var("REDIS_URL")?;
    let node_id = std::env::var("NODE_ID")?;

    // Create Redis adapter
    let coordination = Arc::new(
        RedisCoordination::new(&redis_url, node_id.clone(), Some("riptide".to_string())).await?
    ) as Arc<dyn DistributedCoordination>;

    // Inject into sync manager
    let config = DistributedConfig { /* ... */ };
    Ok(DistributedSync::new(config, coordination))
}

// Testing: Unit tests
async fn create_test_sync() -> DistributedSync {
    // Create in-memory adapter
    let coordination = Arc::new(MemoryCoordination::new()) as Arc<dyn DistributedCoordination>;

    let config = DistributedConfig { /* ... */ };
    DistributedSync::new(config, coordination)
}
```

---

## Rationale

### Why This Approach?

1. **Hexagonal Architecture Compliance**
   - Clean separation between domain logic and infrastructure
   - Business logic depends on abstractions, not implementations
   - Infrastructure adapters implement domain-defined ports

2. **Testability**
   - Unit tests run in milliseconds with `MemoryCoordination`
   - No external dependencies for fast test feedback
   - Mockable operations for behavioral testing

3. **Flexibility**
   - Can swap backends without changing business logic
   - Support for multiple messaging systems (Redis, NATS, Kafka)
   - Custom coordination strategies for specific use cases

4. **Maintainability**
   - Explicit contracts via trait methods
   - Documentation lives with the interface
   - Clear boundaries between layers

5. **Performance**
   - Trait object dispatch overhead is negligible (<1% for I/O-bound operations)
   - Backend implementations can optimize for their specific systems
   - No unnecessary abstractions

### Why Not Alternatives?

**Alternative 1: Keep Direct Redis Dependencies**
- ❌ Violates hexagonal architecture
- ❌ Impossible to test without infrastructure
- ❌ Backend lock-in

**Alternative 2: Generic Type Parameters**
```rust
pub struct DistributedSync<C: Coordination> {
    coordination: C,
}
```
- ❌ Forces monomorphization (code size bloat)
- ❌ Less flexible than trait objects
- ❌ Cannot mix implementations at runtime

**Alternative 3: Conditional Compilation**
```rust
#[cfg(feature = "redis")]
use redis::...;
```
- ❌ Feature explosion
- ❌ Doesn't solve testability
- ❌ Complex build configuration

---

## Consequences

### Positive

✅ **Clean Architecture**
- `sync.rs` has zero infrastructure dependencies
- Clear separation of concerns
- Follows Dependency Inversion Principle

✅ **Testability**
- Unit tests run in-memory without external services
- Fast test execution (<100ms per test suite)
- Simplified CI/CD pipeline

✅ **Flexibility**
- Can use Redis in production
- Can use NATS for edge deployments
- Can use in-memory for local development
- Can implement custom backends for specific needs

✅ **Maintainability**
- Explicit contracts via trait documentation
- Clear boundaries between components
- Easier to understand and modify

✅ **Type Safety**
- Compile-time guarantees via trait bounds
- Cannot accidentally use wrong backend methods
- Rust's type system prevents misuse

### Negative

⚠️ **Initial Complexity**
- Requires implementing multiple adapters
- Additional abstraction layer to understand
- Team needs to learn dependency injection patterns

⚠️ **Runtime Overhead**
- Trait object dispatch via vtable (negligible for I/O operations)
- Additional `Arc` indirection (one pointer deref)
- Estimated <1% performance impact

⚠️ **Migration Effort**
- Need to refactor existing `sync.rs` code
- Update all call sites to use dependency injection
- Write comprehensive tests for both adapters

### Neutral

📝 **Documentation Requirements**
- Need to document port trait usage
- Need to document adapter implementation guidelines
- Need migration guide for existing code

📝 **Testing Strategy Change**
- Unit tests now use `MemoryCoordination`
- Integration tests use `RedisCoordination`
- Need separate test suites for each

---

## Alternatives Considered

### Alternative 1: Keep Current Architecture
**Description**: Continue using direct Redis dependencies in `sync.rs`

**Pros**:
- No refactoring required
- Familiar to team
- Zero migration cost

**Cons**:
- Violates hexagonal architecture
- Cannot test without Redis
- Backend lock-in
- Poor maintainability

**Verdict**: ❌ **Rejected** - Architecture violations are unacceptable

---

### Alternative 2: Generic Type Parameters
**Description**: Use generics instead of trait objects

```rust
pub struct DistributedSync<C: DistributedCoordination> {
    coordination: C,
}
```

**Pros**:
- Zero-cost abstraction (no vtable dispatch)
- Compiler can inline methods
- Slightly better performance

**Cons**:
- Code size bloat from monomorphization
- Cannot mix implementations at runtime
- Complex generic bounds throughout codebase
- Cannot use `Arc<dyn Trait>` pattern

**Verdict**: ❌ **Rejected** - Flexibility more important than micro-optimization

---

### Alternative 3: Conditional Compilation
**Description**: Use feature flags for different backends

```rust
#[cfg(feature = "redis-backend")]
mod redis_impl;

#[cfg(feature = "memory-backend")]
mod memory_impl;
```

**Pros**:
- Can optimize for each backend
- No runtime overhead

**Cons**:
- Feature explosion
- Cannot test multiple backends in same build
- Complex conditional compilation logic
- Doesn't solve testability issues

**Verdict**: ❌ **Rejected** - Too complex, doesn't achieve goals

---

### Alternative 4: Repository Pattern with Connection Trait
**Description**: Create a more granular trait hierarchy

```rust
pub trait Connection { /* ... */ }
pub trait PubSubProvider { /* ... */ }
pub trait KeyValueStore { /* ... */ }
```

**Pros**:
- More fine-grained abstractions
- Interface Segregation Principle

**Cons**:
- Over-engineering for current needs
- More complex trait bounds
- Harder to implement adapters

**Verdict**: ❌ **Rejected** - Single cohesive trait is sufficient

---

## Implementation Plan

### Phase 0: Trait Definition ✅ COMPLETE
- [x] Define `DistributedCoordination` trait in `riptide-types`
- [x] Document method contracts and error handling
- [x] Create specification document
- [x] Write this ADR

### Phase 1: Adapter Implementation
- [ ] Implement `RedisCoordination` adapter
  - [ ] Pub/sub methods
  - [ ] Key operations
  - [ ] Cluster coordination
  - [ ] Leader election
  - [ ] Unit tests

- [ ] Implement `MemoryCoordination` adapter
  - [ ] In-memory pub/sub with broadcast channels
  - [ ] HashMap-based key operations
  - [ ] In-memory leader election
  - [ ] Unit tests

### Phase 2: DistributedSync Refactoring
- [ ] Add `coordination: Arc<dyn DistributedCoordination>` field
- [ ] Update constructor for dependency injection
- [ ] Refactor `notify_operation` to use `publish`
- [ ] Refactor `apply_invalidate_operation` to use `keys` + `delete_many`
- [ ] Refactor `apply_clear_operation` to use `flush_all`
- [ ] Remove direct Redis dependencies
- [ ] Update all method signatures

### Phase 3: Testing & Validation
- [ ] Unit tests with `MemoryCoordination`
  - [ ] Test pub/sub operations
  - [ ] Test leader election
  - [ ] Test cluster coordination
  - [ ] Achieve 100% coverage

- [ ] Integration tests with `RedisCoordination`
  - [ ] Test against real Redis instance
  - [ ] Test multi-node scenarios
  - [ ] Test failure modes

- [ ] Performance benchmarks
  - [ ] Compare vs. direct Redis
  - [ ] Measure trait dispatch overhead
  - [ ] Validate <5% performance regression

### Phase 4: Documentation & Cleanup
- [ ] Update architecture documentation
- [ ] Write usage examples
- [ ] Create migration guide
- [ ] Remove legacy Redis dependencies
- [ ] Update CI/CD configuration

---

## Validation Criteria

This decision will be considered successful when:

1. ✅ **Zero Direct Redis Dependencies in sync.rs**
   - No `use redis::*` statements
   - All coordination via trait methods

2. ✅ **100% Unit Test Coverage**
   - All tests pass with `MemoryCoordination`
   - No external dependencies for unit tests

3. ✅ **Performance Parity**
   - <5% performance regression vs. direct Redis
   - Measured via benchmarks

4. ✅ **Type Safety**
   - All implementations are `Send + Sync`
   - Compile-time guarantees via trait bounds

5. ✅ **Documentation Complete**
   - Trait methods fully documented
   - Usage examples provided
   - Migration guide available

6. ✅ **Backend Flexibility Demonstrated**
   - Can swap Redis for alternatives in <50 LOC
   - Both adapters fully functional

---

## Related Decisions

- **ADR-002**: TBD - CacheStorage Port Refactoring
- **ADR-003**: TBD - SessionStorage Port Refactoring
- **ADR-004**: TBD - Repository Port Refactoring

---

## References

- [Hexagonal Architecture (Ports and Adapters)](https://alistair.cockburn.us/hexagonal-architecture/)
- [Clean Architecture by Robert C. Martin](https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html)
- [Dependency Inversion Principle](https://en.wikipedia.org/wiki/Dependency_inversion_principle)
- [Trait Objects in Rust](https://doc.rust-lang.org/book/ch17-02-trait-objects.html)

---

## Appendix: Code Examples

### Before: Direct Redis Coupling

```rust
// sync.rs - BEFORE REFACTORING
use redis::aio::MultiplexedConnection;
use redis::{AsyncCommands, Client};

pub struct DistributedSync {
    pool: Arc<Mutex<MultiplexedConnection>>,  // ❌ Direct Redis
    config: DistributedConfig,
    node_id: String,
}

impl DistributedSync {
    pub async fn new(redis_url: &str, config: DistributedConfig) -> PersistenceResult<Self> {
        let client = Client::open(redis_url)?;  // ❌ Infrastructure in domain
        let conn = client.get_multiplexed_tokio_connection().await?;

        Ok(Self {
            pool: Arc::new(Mutex::new(conn)),
            config,
            node_id: config.node_id.clone(),
        })
    }

    pub async fn notify_operation(&self, operation: SyncOperation) -> PersistenceResult<()> {
        let channel = format!("riptide:sync:operations");
        let message = serde_json::to_string(&operation)?;

        let mut conn = self.pool.lock().await;  // ❌ Redis-specific
        conn.publish::<_, _, ()>(&channel, &message).await?;  // ❌ Redis command
        Ok(())
    }
}
```

### After: Port Trait Abstraction

```rust
// sync.rs - AFTER REFACTORING
use riptide_types::ports::coordination::DistributedCoordination;
use std::sync::Arc;

pub struct DistributedSync {
    coordination: Arc<dyn DistributedCoordination>,  // ✅ Abstract dependency
    config: DistributedConfig,
    node_id: String,
}

impl DistributedSync {
    pub fn new(
        config: DistributedConfig,
        coordination: Arc<dyn DistributedCoordination>,  // ✅ Injected
    ) -> Self {
        Self {
            coordination,
            config,
            node_id: config.node_id.clone(),
        }
    }

    pub async fn notify_operation(&self, operation: SyncOperation) -> PersistenceResult<()> {
        let channel = format!("riptide:sync:operations");
        let message = serde_json::to_vec(&operation)?;

        self.coordination.publish(&channel, &message).await?;  // ✅ Port method
        Ok(())
    }
}
```

### Usage Example

```rust
// Application initialization
async fn main() -> anyhow::Result<()> {
    // Production: Use Redis
    let redis_coord = Arc::new(
        RedisCoordination::new("redis://localhost", "node-1".to_string(), None).await?
    ) as Arc<dyn DistributedCoordination>;

    let config = DistributedConfig { /* ... */ };
    let sync = DistributedSync::new(config, redis_coord);

    // Testing: Use in-memory
    let memory_coord = Arc::new(MemoryCoordination::new()) as Arc<dyn DistributedCoordination>;
    let test_sync = DistributedSync::new(config, memory_coord);

    Ok(())
}
```

---

**END OF ADR**
