=== CLIPPY ANALYSIS REPORT ===
Generated: Fri Oct 10 20:12:45 UTC 2025

    Checking riptide-api v0.1.0 (/workspaces/eventmesh/crates/riptide-api)
    Checking riptide-streaming v0.1.0 (/workspaces/eventmesh/crates/riptide-streaming)
    Checking riptide-intelligence v0.1.0 (/workspaces/eventmesh/crates/riptide-intelligence)
    Checking riptide-persistence v0.1.0 (/workspaces/eventmesh/crates/riptide-persistence)
    Checking riptide-extractor-wasm v0.1.0 (/workspaces/eventmesh/wasm/riptide-extractor-wasm)
error[E0432]: unresolved import `riptide_persistence::storage`
 --> crates/riptide-persistence/tests/persistence_tests.rs:9:30
  |
9 |     use riptide_persistence::storage::{DatabaseStorage, FileStorage, StorageBackend};
  |                              ^^^^^^^ could not find `storage` in `riptide_persistence`

error[E0432]: unresolved import `riptide_persistence::queue`
   --> crates/riptide-persistence/tests/persistence_tests.rs:106:30
    |
106 |     use riptide_persistence::queue::{PersistentQueue, QueueConfig};
    |                              ^^^^^ could not find `queue` in `riptide_persistence`

error[E0432]: unresolved import `riptide_persistence::checkpoint`
   --> crates/riptide-persistence/tests/persistence_tests.rs:148:30
    |
148 |     use riptide_persistence::checkpoint::{CheckpointManager, CrawlState};
    |                              ^^^^^^^^^^ could not find `checkpoint` in `riptide_persistence`

error[E0422]: cannot find struct, variant or union type `CrawlRecord` in this scope
  --> crates/riptide-persistence/tests/persistence_tests.rs:37:22
   |
37 |                     &CrawlRecord {
   |                      ^^^^^^^^^^^ not found in this scope

error[E0422]: cannot find struct, variant or union type `CacheConfig` in this scope
  --> crates/riptide-persistence/tests/persistence_tests.rs:59:42
   |
59 |         let cache = PersistentCache::new(CacheConfig {
   |                                          ^^^^^^^^^^^ not found in this scope
   |
help: consider importing one of these structs
   |
 8 +     use crate::config::CacheConfig;
   |
 8 +     use riptide_core::cache::CacheConfig;
   |
 8 +     use riptide_persistence::config::CacheConfig;
   |

error[E0422]: cannot find struct, variant or union type `CacheConfig` in this scope
  --> crates/riptide-persistence/tests/persistence_tests.rs:71:43
   |
71 |         let cache2 = PersistentCache::new(CacheConfig {
   |                                           ^^^^^^^^^^^ not found in this scope
   |
help: consider importing one of these structs
   |
 8 +     use crate::config::CacheConfig;
   |
 8 +     use riptide_core::cache::CacheConfig;
   |
 8 +     use riptide_persistence::config::CacheConfig;
   |

error: unused import: `super::*`
   --> crates/riptide-persistence/tests/persistence_tests.rs:105:9
    |
105 |     use super::*;
    |         ^^^^^^^^
    |
    = note: `-D unused-imports` implied by `-D warnings`
    = help: to override `-D warnings` add `#[allow(unused_imports)]`

error[E0433]: failed to resolve: use of undeclared type `PersistentCache`
  --> crates/riptide-persistence/tests/persistence_tests.rs:59:21
   |
59 |         let cache = PersistentCache::new(CacheConfig {
   |                     ^^^^^^^^^^^^^^^ use of undeclared type `PersistentCache`

error[E0433]: failed to resolve: use of undeclared type `PersistentCache`
  --> crates/riptide-persistence/tests/persistence_tests.rs:71:22
   |
71 |         let cache2 = PersistentCache::new(CacheConfig {
   |                      ^^^^^^^^^^^^^^^ use of undeclared type `PersistentCache`

error[E0433]: failed to resolve: use of undeclared type `PriorityQueue`
   --> crates/riptide-persistence/tests/persistence_tests.rs:132:21
    |
132 |         let queue = PriorityQueue::new("/tmp/riptide_pqueue").await.unwrap();
    |                     ^^^^^^^^^^^^^ use of undeclared type `PriorityQueue`

error[E0624]: associated function `new` is private
   --> crates/riptide-persistence/tests/persistence_tests.rs:152:42
    |
152 |         let manager = CheckpointManager::new("/tmp/riptide_checkpoints")
    |                                          ^^^ private associated function
    |
   ::: /workspaces/eventmesh/crates/riptide-persistence/src/state.rs:858:5
    |
858 |     async fn new(config: StateConfig) -> PersistenceResult<Self> {
    |     ------------------------------------------------------------ private associated function defined here

    Checking riptide-search v0.1.0 (/workspaces/eventmesh/crates/riptide-search)
error: wildcard pattern covers any other pattern as it will match anyway
   --> crates/riptide-streaming/src/api_handlers.rs:114:9
    |
114 |         "modern" | _ => ReportTheme::Modern,
    |         ^^^^^^^^^^^^
    |
    = help: consider handling `_` separately
    = help: for further information visit https://rust-lang.github.io/rust-clippy/master/index.html#wildcard_in_or_patterns
    = note: `-D clippy::wildcard-in-or-patterns` implied by `-D warnings`
    = help: to override `-D warnings` add `#[allow(clippy::wildcard_in_or_patterns)]`

error[E0308]: mismatched types
   --> crates/riptide-persistence/tests/persistence_tests.rs:152:46
    |
152 |         let manager = CheckpointManager::new("/tmp/riptide_checkpoints")
    |                       ---------------------- ^^^^^^^^^^^^^^^^^^^^^^^^^^ expected `StateConfig`, found `&str`
    |                       |
    |                       arguments to this function are incorrect
    |
note: associated function defined here
   --> /workspaces/eventmesh/crates/riptide-persistence/src/state.rs:858:14
    |
858 |     async fn new(config: StateConfig) -> PersistenceResult<Self> {
    |              ^^^

error[E0599]: no method named `save_checkpoint` found for struct `riptide_persistence::CheckpointManager` in the current scope
   --> crates/riptide-persistence/tests/persistence_tests.rs:165:17
    |
165 |         manager.save_checkpoint(&state).await.unwrap();
    |                 ^^^^^^^^^^^^^^^ method not found in `riptide_persistence::CheckpointManager`

error[E0599]: no method named `restore_checkpoint` found for struct `riptide_persistence::CheckpointManager` in the current scope
   --> crates/riptide-persistence/tests/persistence_tests.rs:168:32
    |
168 |         let restored = manager.restore_checkpoint("job123").await.unwrap();
    |                                ^^^^^^^^^^^^^^^^^^ method not found in `riptide_persistence::CheckpointManager`

error[E0624]: associated function `new` is private
   --> crates/riptide-persistence/tests/persistence_tests.rs:175:42
    |
175 |         let manager = CheckpointManager::new("/tmp/riptide_checkpoints")
    |                                          ^^^ private associated function
    |
   ::: /workspaces/eventmesh/crates/riptide-persistence/src/state.rs:858:5
    |
858 |     async fn new(config: StateConfig) -> PersistenceResult<Self> {
    |     ------------------------------------------------------------ private associated function defined here

error[E0308]: mismatched types
   --> crates/riptide-persistence/tests/persistence_tests.rs:175:46
    |
175 |         let manager = CheckpointManager::new("/tmp/riptide_checkpoints")
    |                       ---------------------- ^^^^^^^^^^^^^^^^^^^^^^^^^^ expected `StateConfig`, found `&str`
    |                       |
    |                       arguments to this function are incorrect
    |
note: associated function defined here
   --> /workspaces/eventmesh/crates/riptide-persistence/src/state.rs:858:14
    |
858 |     async fn new(config: StateConfig) -> PersistenceResult<Self> {
    |              ^^^

error[E0599]: no method named `save_checkpoint` found for struct `riptide_persistence::CheckpointManager` in the current scope
   --> crates/riptide-persistence/tests/persistence_tests.rs:188:21
    |
188 |             manager.save_checkpoint(&state).await.unwrap();
    |                     ^^^^^^^^^^^^^^^ method not found in `riptide_persistence::CheckpointManager`

error[E0624]: method `cleanup_old_checkpoints` is private
   --> crates/riptide-persistence/tests/persistence_tests.rs:193:14
    |
193 |             .cleanup_old_checkpoints(Duration::from_secs(86400 * 7))
    |              ^^^^^^^^^^^^^^^^^^^^^^^ private method
    |
   ::: /workspaces/eventmesh/crates/riptide-persistence/src/state.rs:899:5
    |
899 |     async fn cleanup_old_checkpoints(&self) -> PersistenceResult<()> {
    |     ---------------------------------------------------------------- private method defined here

Some errors have detailed explanations: E0308, E0422, E0432, E0433, E0599, E0624.
For more information about an error, try `rustc --explain E0308`.
error: could not compile `riptide-persistence` (test "persistence_tests") due to 18 previous errors
warning: build failed, waiting for other jobs to finish...
error: useless use of `vec!`
   --> crates/riptide-streaming/src/reports.rs:396:31
    |
396 |             let mut buckets = vec![0; 10];
    |                               ^^^^^^^^^^^ help: you can use an array directly: `[0; 10]`
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/master/index.html#useless_vec
    = note: `-D clippy::useless-vec` implied by `-D warnings`
    = help: to override `-D warnings` add `#[allow(clippy::useless_vec)]`

error: useless use of `vec!`
   --> crates/riptide-streaming/src/reports.rs:515:31
    |
515 |             let mut buckets = vec![0; 10];
    |                               ^^^^^^^^^^^ help: you can use an array directly: `[0; 10]`
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/master/index.html#useless_vec

error: unused imports: `CircuitBreakerWrapper` and `create_search_provider_from_env`
 --> crates/riptide-search/tests/riptide_search_providers_tests.rs:8:29
  |
8 |     create_search_provider, create_search_provider_from_env, AdvancedSearchConfig,
  |                             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
9 |     CircuitBreakerWrapper, NoneProvider, SearchBackend, SearchConfig, SearchHit, SearchProvider,
  |     ^^^^^^^^^^^^^^^^^^^^^
  |
  = note: `-D unused-imports` implied by `-D warnings`
  = help: to override `-D warnings` add `#[allow(unused_imports)]`

error: could not compile `riptide-streaming` (lib) due to 3 previous errors
error: field assignment outside of initializer for an instance created with Default::default()
   --> crates/riptide-search/tests/riptide_search_providers_tests.rs:375:9
    |
375 |         config.backend = SearchBackend::None;
    |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    |
note: consider initializing the variable with `riptide_search::AdvancedSearchConfig { backend: SearchBackend::None, ..Default::default() }` and removing relevant reassignments
   --> crates/riptide-search/tests/riptide_search_providers_tests.rs:374:9
    |
374 |         let mut config = AdvancedSearchConfig::default();
    |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    = help: for further information visit https://rust-lang.github.io/rust-clippy/master/index.html#field_reassign_with_default
    = note: `-D clippy::field-reassign-with-default` implied by `-D warnings`
    = help: to override `-D warnings` add `#[allow(clippy::field_reassign_with_default)]`

error: could not compile `riptide-search` (test "riptide_search_providers_tests") due to 2 previous errors
error: unused import: `Deserialize`
  --> crates/riptide-api/src/handlers/profiling.rs:29:13
   |
29 | use serde::{Deserialize, Serialize};
   |             ^^^^^^^^^^^
   |
   = note: `-D unused-imports` implied by `-D warnings`
   = help: to override `-D warnings` add `#[allow(unused_imports)]`

error: unused import: `std::collections::HashMap`
  --> crates/riptide-api/src/handlers/profiling.rs:30:5
   |
30 | use std::collections::HashMap;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^

error: could not compile `riptide-streaming` (lib test) due to 2 previous errors
error: manual implementation of `Option::map`
   --> crates/riptide-api/src/handlers/llm.rs:695:9
    |
695 | /         if let Some(model) = llm_capabilities.models.first() {
696 | |             Some(CostInfo {
697 | |                 input_token_cost: Some(model.cost_per_1k_prompt_tokens),
698 | |                 output_token_cost: Some(model.cost_per_1k_completion_tokens),
...   |
702 | |             None
703 | |         }
    | |_________^
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/master/index.html#manual_map
    = note: `-D clippy::manual-map` implied by `-D warnings`
    = help: to override `-D warnings` add `#[allow(clippy::manual_map)]`
help: try
    |
695 ~         llm_capabilities.models.first().map(|model| CostInfo {
696 +                 input_token_cost: Some(model.cost_per_1k_prompt_tokens),
697 +                 output_token_cost: Some(model.cost_per_1k_completion_tokens),
698 +                 currency: "USD".to_string(),
699 +             })
    |

error: these `if` branches have the same condition
   --> crates/riptide-api/src/handlers/profiling.rs:621:16
    |
621 |             if 500.0 > 700.0 {
    |                ^^^^^^^^^^^^^
622 |                 "critical"
623 |             } else if 500.0 > 650.0 {
    |                       ^^^^^^^^^^^^^
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/master/index.html#ifs_same_cond
    = note: `#[deny(clippy::ifs_same_cond)]` on by default

error: could not compile `riptide-api` (lib) due to 3 previous errors
error: could not compile `riptide-api` (lib test) due to 4 previous errors
