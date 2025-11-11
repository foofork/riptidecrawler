   Compiling riptide-api v0.9.0 (/workspaces/riptidecrawler/crates/riptide-api)
warning: use of deprecated type alias `riptide_api::AppState`: Use context::ApplicationContext instead. See docs/architecture/ADR-001-appstate-elimination.md
  --> crates/riptide-api/tests/auth_middleware_tests.rs:24:12
   |
24 |     state::AppState,
   |            ^^^^^^^^
   |
   = note: `#[warn(deprecated)]` on by default

warning: use of deprecated type alias `riptide_api::AppState`: Use context::ApplicationContext instead. See docs/architecture/ADR-001-appstate-elimination.md
  --> crates/riptide-api/tests/auth_middleware_tests.rs:41:21
   |
41 |     let mut state = AppState::new_test_minimal().await;
   |                     ^^^^^^^^

warning: use of deprecated type alias `riptide_api::AppState`: Use context::ApplicationContext instead. See docs/architecture/ADR-001-appstate-elimination.md
   --> crates/riptide-api/tests/auth_middleware_tests.rs:101:21
    |
101 |     let mut state = AppState::new_test_minimal().await;
    |                     ^^^^^^^^

warning: use of deprecated type alias `riptide_api::AppState`: Use context::ApplicationContext instead. See docs/architecture/ADR-001-appstate-elimination.md
   --> crates/riptide-api/tests/auth_middleware_tests.rs:158:21
    |
158 |     let mut state = AppState::new_test_minimal().await;
    |                     ^^^^^^^^

warning: use of deprecated type alias `riptide_api::AppState`: Use context::ApplicationContext instead. See docs/architecture/ADR-001-appstate-elimination.md
   --> crates/riptide-api/tests/auth_middleware_tests.rs:193:21
    |
193 |     let mut state = AppState::new_test_minimal().await;
    |                     ^^^^^^^^

warning: use of deprecated type alias `riptide_api::AppState`: Use context::ApplicationContext instead. See docs/architecture/ADR-001-appstate-elimination.md
   --> crates/riptide-api/tests/auth_middleware_tests.rs:247:21
    |
247 |     let mut state = AppState::new_test_minimal().await;
    |                     ^^^^^^^^

warning: use of deprecated type alias `riptide_api::AppState`: Use context::ApplicationContext instead. See docs/architecture/ADR-001-appstate-elimination.md
   --> crates/riptide-api/tests/auth_middleware_tests.rs:289:21
    |
289 |     let mut state = AppState::new_test_minimal().await;
    |                     ^^^^^^^^

warning: use of deprecated type alias `riptide_api::AppState`: Use context::ApplicationContext instead. See docs/architecture/ADR-001-appstate-elimination.md
   --> crates/riptide-api/tests/auth_middleware_tests.rs:335:21
    |
335 |     let mut state = AppState::new_test_minimal().await;
    |                     ^^^^^^^^

warning: use of deprecated type alias `riptide_api::AppState`: Use context::ApplicationContext instead. See docs/architecture/ADR-001-appstate-elimination.md
   --> crates/riptide-api/tests/auth_middleware_tests.rs:396:21
    |
396 |     let mut state = AppState::new_test_minimal().await;
    |                     ^^^^^^^^

warning: use of deprecated type alias `riptide_api::AppState`: Use context::ApplicationContext instead. See docs/architecture/ADR-001-appstate-elimination.md
   --> crates/riptide-api/tests/auth_middleware_tests.rs:479:21
    |
479 |     let mut state = AppState::new_test_minimal().await;
    |                     ^^^^^^^^

warning: use of deprecated type alias `riptide_api::AppState`: Use context::ApplicationContext instead. See docs/architecture/ADR-001-appstate-elimination.md
   --> crates/riptide-api/tests/auth_middleware_tests.rs:508:21
    |
508 |     let mut state = AppState::new_test_minimal().await;
    |                     ^^^^^^^^

warning: use of deprecated type alias `riptide_api::AppState`: Use context::ApplicationContext instead. See docs/architecture/ADR-001-appstate-elimination.md
   --> crates/riptide-api/tests/auth_middleware_tests.rs:593:21
    |
593 |     let mut state = AppState::new_test_minimal().await;
    |                     ^^^^^^^^

warning: use of deprecated type alias `riptide_api::AppState`: Use context::ApplicationContext instead. See docs/architecture/ADR-001-appstate-elimination.md
   --> crates/riptide-api/tests/auth_middleware_tests.rs:624:21
    |
624 |     let mut state = AppState::new_test_minimal().await;
    |                     ^^^^^^^^

warning: use of deprecated type alias `riptide_api::AppState`: Use context::ApplicationContext instead. See docs/architecture/ADR-001-appstate-elimination.md
 --> crates/riptide-api/tests/test_helpers.rs:9:59
  |
9 | use riptide_api::{handlers, health::HealthChecker, state::AppState};
  |                                                           ^^^^^^^^
  |
  = note: `#[warn(deprecated)]` on by default

warning: use of deprecated type alias `riptide_api::AppState`: Use context::ApplicationContext instead. See docs/architecture/ADR-001-appstate-elimination.md
  --> crates/riptide-api/tests/test_helpers.rs:23:37
   |
23 | pub async fn create_test_state() -> AppState {
   |                                     ^^^^^^^^

warning: use of deprecated type alias `riptide_api::AppState`: Use context::ApplicationContext instead. See docs/architecture/ADR-001-appstate-elimination.md
  --> crates/riptide-api/tests/test_helpers.rs:41:5
   |
41 |     AppState::new(config, health_checker)
   |     ^^^^^^^^

warning: use of deprecated type alias `riptide_api::AppState`: Use context::ApplicationContext instead. See docs/architecture/ADR-001-appstate-elimination.md
  --> crates/riptide-api/tests/test_helpers.rs:71:11
   |
71 |     match AppState::new(config, health_checker).await {
   |           ^^^^^^^^

warning: use of deprecated type alias `riptide_api::AppState`: Use context::ApplicationContext instead. See docs/architecture/ADR-001-appstate-elimination.md
  --> crates/riptide-api/tests/test_helpers.rs:82:34
   |
82 | pub fn create_test_router(state: AppState) -> Router {
   |                                  ^^^^^^^^

warning: use of deprecated type alias `riptide_api::AppState`: Use context::ApplicationContext instead. See docs/architecture/ADR-001-appstate-elimination.md
  --> crates/riptide-api/tests/auth_rate_limiting_tests.rs:25:12
   |
25 |     state::AppState,
   |            ^^^^^^^^
   |
   = note: `#[warn(deprecated)]` on by default

warning: use of deprecated type alias `riptide_api::AppState`: Use context::ApplicationContext instead. See docs/architecture/ADR-001-appstate-elimination.md
  --> crates/riptide-api/tests/auth_rate_limiting_tests.rs:33:21
   |
33 |     let mut state = AppState::new_test_minimal().await;
   |                     ^^^^^^^^

warning: use of deprecated type alias `riptide_api::AppState`: Use context::ApplicationContext instead. See docs/architecture/ADR-001-appstate-elimination.md
   --> crates/riptide-api/tests/pdf_integration_tests.rs:257:28
    |
257 |         state::{AppConfig, AppState},
    |                            ^^^^^^^^
    |
    = note: `#[warn(deprecated)]` on by default

warning: use of deprecated type alias `riptide_api::AppState`: Use context::ApplicationContext instead. See docs/architecture/ADR-001-appstate-elimination.md
   --> crates/riptide-api/tests/pdf_integration_tests.rs:265:25
    |
265 |         let app_state = AppState::new(config, health_checker)
    |                         ^^^^^^^^

warning: use of deprecated type alias `riptide_api::AppState`: Use context::ApplicationContext instead. See docs/architecture/ADR-001-appstate-elimination.md
  --> crates/riptide-api/tests/streaming_endpoints_integration.rs:18:37
   |
18 | use riptide_api::state::{AppConfig, AppState};
   |                                     ^^^^^^^^
   |
   = note: `#[warn(deprecated)]` on by default

warning: use of deprecated type alias `riptide_api::AppState`: Use context::ApplicationContext instead. See docs/architecture/ADR-001-appstate-elimination.md
  --> crates/riptide-api/tests/streaming_endpoints_integration.rs:27:37
   |
27 | async fn create_test_app_state() -> AppState {
   |                                     ^^^^^^^^

warning: use of deprecated type alias `riptide_api::AppState`: Use context::ApplicationContext instead. See docs/architecture/ADR-001-appstate-elimination.md
  --> crates/riptide-api/tests/streaming_endpoints_integration.rs:31:5
   |
31 |     AppState::new(config, health_checker)
   |     ^^^^^^^^

warning: use of deprecated type alias `riptide_api::AppState`: Use context::ApplicationContext instead. See docs/architecture/ADR-001-appstate-elimination.md
 --> crates/riptide-api/tests/test_helpers.rs:9:59
  |
9 | use riptide_api::{handlers, health::HealthChecker, state::AppState};
  |                                                           ^^^^^^^^

warning: use of deprecated type alias `riptide_api::AppState`: Use context::ApplicationContext instead. See docs/architecture/ADR-001-appstate-elimination.md
  --> crates/riptide-api/tests/enhanced_pipeline_tests.rs:11:25
   |
11 | use riptide_api::state::AppState;
   |                         ^^^^^^^^
   |
   = note: `#[warn(deprecated)]` on by default

warning: use of deprecated type alias `riptide_api::AppState`: Use context::ApplicationContext instead. See docs/architecture/ADR-001-appstate-elimination.md
  --> crates/riptide-api/tests/enhanced_pipeline_tests.rs:19:31
   |
19 |     fn create_test_state() -> AppState {
   |                               ^^^^^^^^

warning: `riptide-api` (test "enhanced_pipeline_tests") generated 2 warnings
warning: `riptide-api` (test "auth_middleware_tests") generated 13 warnings
warning: use of deprecated type alias `riptide_api::AppState`: Use context::ApplicationContext instead. See docs/architecture/ADR-001-appstate-elimination.md
  --> crates/riptide-api/tests/phase4b_integration_tests.rs:29:28
   |
29 |         state::{AppConfig, AppState},
   |                            ^^^^^^^^
   |
   = note: `#[warn(deprecated)]` on by default

warning: use of deprecated type alias `riptide_api::AppState`: Use context::ApplicationContext instead. See docs/architecture/ADR-001-appstate-elimination.md
  --> crates/riptide-api/tests/phase4b_integration_tests.rs:35:49
   |
35 |     pub async fn create_test_app_state() -> Arc<AppState> {
   |                                                 ^^^^^^^^

warning: use of deprecated type alias `riptide_api::AppState`: Use context::ApplicationContext instead. See docs/architecture/ADR-001-appstate-elimination.md
  --> crates/riptide-api/tests/phase4b_integration_tests.rs:40:13
   |
40 |             AppState::new(config, health_checker)
   |             ^^^^^^^^

warning: `riptide-api` (test "auth_rate_limiting_tests") generated 2 warnings
warning: `riptide-api` (test "stress_tests") generated 5 warnings
warning: `riptide-api` (test "pdf_integration_tests") generated 2 warnings
warning: use of deprecated type alias `riptide_api::AppState`: Use context::ApplicationContext instead. See docs/architecture/ADR-001-appstate-elimination.md
  --> crates/riptide-api/tests/profiling_integration_tests.rs:11:37
   |
11 | use riptide_api::state::{AppConfig, AppState};
   |                                     ^^^^^^^^
   |
   = note: `#[warn(deprecated)]` on by default

warning: use of deprecated type alias `riptide_api::AppState`: Use context::ApplicationContext instead. See docs/architecture/ADR-001-appstate-elimination.md
  --> crates/riptide-api/tests/profiling_integration_tests.rs:21:17
   |
21 |     let state = AppState::new(config, health_checker)
   |                 ^^^^^^^^

warning: use of deprecated type alias `riptide_api::AppState`: Use context::ApplicationContext instead. See docs/architecture/ADR-001-appstate-elimination.md
  --> crates/riptide-api/tests/profiling_integration_tests.rs:49:17
   |
49 |     let state = AppState::new(config, health_checker)
   |                 ^^^^^^^^

warning: use of deprecated type alias `riptide_api::AppState`: Use context::ApplicationContext instead. See docs/architecture/ADR-001-appstate-elimination.md
  --> crates/riptide-api/tests/profiling_integration_tests.rs:74:17
   |
74 |     let state = AppState::new(config, health_checker)
   |                 ^^^^^^^^

warning: use of deprecated type alias `riptide_api::AppState`: Use context::ApplicationContext instead. See docs/architecture/ADR-001-appstate-elimination.md
   --> crates/riptide-api/tests/profiling_integration_tests.rs:106:17
    |
106 |     let state = AppState::new(config, health_checker)
    |                 ^^^^^^^^

warning: use of deprecated type alias `riptide_api::AppState`: Use context::ApplicationContext instead. See docs/architecture/ADR-001-appstate-elimination.md
   --> crates/riptide-api/tests/profiling_integration_tests.rs:137:17
    |
137 |     let state = AppState::new(config, health_checker)
    |                 ^^^^^^^^

warning: use of deprecated type alias `riptide_api::AppState`: Use context::ApplicationContext instead. See docs/architecture/ADR-001-appstate-elimination.md
   --> crates/riptide-api/tests/profiling_integration_tests.rs:180:17
    |
180 |     let state = AppState::new(config, health_checker)
    |                 ^^^^^^^^

warning: use of deprecated type alias `riptide_api::AppState`: Use context::ApplicationContext instead. See docs/architecture/ADR-001-appstate-elimination.md
   --> crates/riptide-api/tests/profiling_integration_tests.rs:200:17
    |
200 |     let state = AppState::new(config, health_checker)
    |                 ^^^^^^^^

warning: use of deprecated type alias `riptide_api::AppState`: Use context::ApplicationContext instead. See docs/architecture/ADR-001-appstate-elimination.md
   --> crates/riptide-api/tests/profiling_integration_tests.rs:229:9
    |
229 |         AppState::new(config, health_checker)
    |         ^^^^^^^^

warning: use of deprecated type alias `riptide_api::AppState`: Use context::ApplicationContext instead. See docs/architecture/ADR-001-appstate-elimination.md
   --> crates/riptide-api/tests/profiling_integration_tests.rs:266:17
    |
266 |     let state = AppState::new(config, health_checker)
    |                 ^^^^^^^^

warning: use of deprecated type alias `riptide_api::AppState`: Use context::ApplicationContext instead. See docs/architecture/ADR-001-appstate-elimination.md
   --> crates/riptide-api/tests/profiling_integration_tests.rs:294:17
    |
294 |     let state = AppState::new(config, health_checker)
    |                 ^^^^^^^^

warning: use of deprecated type alias `riptide_api::AppState`: Use context::ApplicationContext instead. See docs/architecture/ADR-001-appstate-elimination.md
   --> crates/riptide-api/tests/profiling_integration_tests.rs:335:21
    |
335 |         let state = AppState::new(config, health_checker)
    |                     ^^^^^^^^

warning: `riptide-api` (test "memory_profile_tests") generated 5 warnings (5 duplicates)
warning: `riptide-api` (test "streaming_endpoints_integration") generated 8 warnings (4 duplicates)
warning: use of deprecated type alias `riptide_api::AppState`: Use context::ApplicationContext instead. See docs/architecture/ADR-001-appstate-elimination.md
  --> crates/riptide-api/tests/auth_integration_tests.rs:21:12
   |
21 |     state::AppState,
   |            ^^^^^^^^
   |
   = note: `#[warn(deprecated)]` on by default

warning: use of deprecated type alias `riptide_api::AppState`: Use context::ApplicationContext instead. See docs/architecture/ADR-001-appstate-elimination.md
  --> crates/riptide-api/tests/auth_integration_tests.rs:29:21
   |
29 |     let mut state = AppState::new_test_minimal().await;
   |                     ^^^^^^^^

warning: `riptide-api` (test "cross_module_integration") generated 5 warnings (5 duplicates)
error[E0433]: failed to resolve: use of undeclared type `Arc`
  --> crates/riptide-facade/tests/crawl_facade_integration_tests.rs:27:9
   |
27 |         Arc::strong_count(pipeline_ref) >= 1,
   |         ^^^ use of undeclared type `Arc`
   |
help: consider importing this struct
   |
 8 + use std::sync::Arc;
   |

error[E0433]: failed to resolve: use of undeclared type `Arc`
  --> crates/riptide-facade/tests/crawl_facade_integration_tests.rs:31:9
   |
31 |         Arc::strong_count(strategies_ref) >= 1,
   |         ^^^ use of undeclared type `Arc`
   |
help: consider importing this struct
   |
 8 + use std::sync::Arc;
   |

error[E0433]: failed to resolve: use of undeclared type `Arc`
   --> crates/riptide-facade/tests/crawl_facade_integration_tests.rs:131:13
    |
131 |     assert!(Arc::strong_count(facade.pipeline_executor()) >= 1);
    |             ^^^ use of undeclared type `Arc`
    |
help: consider importing this struct
    |
  8 + use std::sync::Arc;
    |

error[E0433]: failed to resolve: use of undeclared type `Arc`
   --> crates/riptide-facade/tests/crawl_facade_integration_tests.rs:132:13
    |
132 |     assert!(Arc::strong_count(facade.strategies_executor()) >= 1);
    |             ^^^ use of undeclared type `Arc`
    |
help: consider importing this struct
    |
  8 + use std::sync::Arc;
    |

error[E0433]: failed to resolve: use of undeclared type `Arc`
   --> crates/riptide-facade/tests/crawl_facade_integration_tests.rs:143:13
    |
143 |     assert!(Arc::strong_count(facade.pipeline_executor()) >= 1);
    |             ^^^ use of undeclared type `Arc`
    |
help: consider importing this struct
    |
  8 + use std::sync::Arc;
    |

error[E0433]: failed to resolve: use of undeclared type `Arc`
   --> crates/riptide-facade/tests/crawl_facade_integration_tests.rs:144:13
    |
144 |     assert!(Arc::strong_count(facade.strategies_executor()) >= 1);
    |             ^^^ use of undeclared type `Arc`
    |
help: consider importing this struct
    |
  8 + use std::sync::Arc;
    |

error[E0433]: failed to resolve: use of undeclared type `Arc`
   --> crates/riptide-facade/tests/crawl_facade_integration_tests.rs:180:34
    |
180 |     let initial_pipeline_count = Arc::strong_count(pipeline1);
    |                                  ^^^ use of undeclared type `Arc`
    |
help: consider importing this struct
    |
  8 + use std::sync::Arc;
    |

error[E0433]: failed to resolve: use of undeclared type `Arc`
   --> crates/riptide-facade/tests/crawl_facade_integration_tests.rs:181:36
    |
181 |     let initial_strategies_count = Arc::strong_count(strategies1);
    |                                    ^^^ use of undeclared type `Arc`
    |
help: consider importing this struct
    |
  8 + use std::sync::Arc;
    |

error[E0433]: failed to resolve: use of undeclared type `Arc`
   --> crates/riptide-facade/tests/crawl_facade_integration_tests.rs:184:22
    |
184 |     let _pipeline2 = Arc::clone(pipeline1);
    |                      ^^^ use of undeclared type `Arc`
    |
help: consider importing this struct
    |
  8 + use std::sync::Arc;
    |

error[E0433]: failed to resolve: use of undeclared type `Arc`
   --> crates/riptide-facade/tests/crawl_facade_integration_tests.rs:185:24
    |
185 |     let _strategies2 = Arc::clone(strategies1);
    |                        ^^^ use of undeclared type `Arc`
    |
help: consider importing this struct
    |
  8 + use std::sync::Arc;
    |

error[E0433]: failed to resolve: use of undeclared type `Arc`
   --> crates/riptide-facade/tests/crawl_facade_integration_tests.rs:188:16
    |
188 |     assert_eq!(Arc::strong_count(pipeline1), initial_pipeline_count + 1);
    |                ^^^ use of undeclared type `Arc`
    |
help: consider importing this struct
    |
  8 + use std::sync::Arc;
    |

error[E0433]: failed to resolve: use of undeclared type `Arc`
   --> crates/riptide-facade/tests/crawl_facade_integration_tests.rs:189:16
    |
189 |     assert_eq!(Arc::strong_count(strategies1), initial_strategies_count + 1);
    |                ^^^ use of undeclared type `Arc`
    |
help: consider importing this struct
    |
  8 + use std::sync::Arc;
    |

error[E0433]: failed to resolve: use of undeclared type `Arc`
   --> crates/riptide-facade/tests/crawl_facade_integration_tests.rs:210:9
    |
210 |         Arc::strong_count(pipeline_arc) >= 1,
    |         ^^^ use of undeclared type `Arc`
    |
help: consider importing this struct
    |
  8 + use std::sync::Arc;
    |

error[E0433]: failed to resolve: use of undeclared type `Arc`
   --> crates/riptide-facade/tests/crawl_facade_integration_tests.rs:214:9
    |
214 |         Arc::strong_count(strategies_arc) >= 1,
    |         ^^^ use of undeclared type `Arc`
    |
help: consider importing this struct
    |
  8 + use std::sync::Arc;
    |

warning: unused imports: `create_test_pipeline_orchestrator`, `create_test_state`, and `create_test_strategies_orchestrator`
  --> crates/riptide-facade/tests/crawl_facade_integration_tests.rs:9:32
   |
 9 |     create_test_orchestrators, create_test_pipeline_orchestrator, create_test_state,
   |                                ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^  ^^^^^^^^^^^^^^^^^
10 |     create_test_strategies_orchestrator,
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: `#[warn(unused_imports)]` (part of `#[warn(unused)]`) on by default

warning: use of deprecated type alias `riptide_api::AppState`: Use context::ApplicationContext instead. See docs/architecture/ADR-001-appstate-elimination.md
  --> crates/riptide-api/tests/profiling_endpoints_live.rs:20:37
   |
20 | use riptide_api::state::{AppConfig, AppState};
   |                                     ^^^^^^^^
   |
   = note: `#[warn(deprecated)]` on by default

warning: use of deprecated type alias `riptide_api::AppState`: Use context::ApplicationContext instead. See docs/architecture/ADR-001-appstate-elimination.md
  --> crates/riptide-api/tests/profiling_endpoints_live.rs:23:37
   |
23 | async fn create_test_state() -> Arc<AppState> {
   |                                     ^^^^^^^^

warning: use of deprecated type alias `riptide_api::AppState`: Use context::ApplicationContext instead. See docs/architecture/ADR-001-appstate-elimination.md
  --> crates/riptide-api/tests/profiling_endpoints_live.rs:28:9
   |
28 |         AppState::new(config, health_checker)
   |         ^^^^^^^^

warning: use of deprecated type alias `riptide_api::AppState`: Use context::ApplicationContext instead. See docs/architecture/ADR-001-appstate-elimination.md
  --> crates/riptide-api/tests/profiling_endpoints_live.rs:35:33
   |
35 | fn build_test_router(state: Arc<AppState>) -> axum::Router {
   |                                 ^^^^^^^^

warning: use of deprecated type alias `riptide_api::AppState`: Use context::ApplicationContext instead. See docs/architecture/ADR-001-appstate-elimination.md
 --> crates/riptide-facade/tests/common/mod.rs:8:37
  |
8 | use riptide_api::state::{AppConfig, AppState};
  |                                     ^^^^^^^^
  |
  = note: `#[warn(deprecated)]` on by default

warning: use of deprecated type alias `riptide_api::AppState`: Use context::ApplicationContext instead. See docs/architecture/ADR-001-appstate-elimination.md
  --> crates/riptide-facade/tests/common/mod.rs:17:37
   |
17 | pub async fn create_test_state() -> AppState {
   |                                     ^^^^^^^^

warning: use of deprecated type alias `riptide_api::AppState`: Use context::ApplicationContext instead. See docs/architecture/ADR-001-appstate-elimination.md
  --> crates/riptide-facade/tests/common/mod.rs:20:5
   |
20 |     AppState::new(config, health_checker)
   |     ^^^^^^^^

warning: use of deprecated type alias `riptide_api::AppState`: Use context::ApplicationContext instead. See docs/architecture/ADR-001-appstate-elimination.md
  --> crates/riptide-facade/tests/common/mod.rs:27:12
   |
27 |     state: AppState,
   |            ^^^^^^^^

warning: use of deprecated type alias `riptide_api::AppState`: Use context::ApplicationContext instead. See docs/architecture/ADR-001-appstate-elimination.md
  --> crates/riptide-facade/tests/common/mod.rs:35:12
   |
35 |     state: AppState,
   |            ^^^^^^^^

For more information about this error, try `rustc --explain E0433`.
warning: `riptide-facade` (test "crawl_facade_integration_tests") generated 6 warnings
error: could not compile `riptide-facade` (test "crawl_facade_integration_tests") due to 14 previous errors; 6 warnings emitted
warning: build failed, waiting for other jobs to finish...
warning: `riptide-api` (test "auth_integration_tests") generated 2 warnings
warning: `riptide-api` (test "profiling_integration_tests") generated 12 warnings
warning: `riptide-api` (test "phase4b_integration_tests") generated 3 warnings
warning: `riptide-api` (test "profiling_endpoints_live") generated 4 warnings
warning: `riptide-api` (test "e2e_full_stack") generated 5 warnings (5 duplicates)
warning: `riptide-api` (test "spider_respect_robots_tests") generated 5 warnings (5 duplicates)
warning: `riptide-api` (test "api_tests") generated 5 warnings (5 duplicates)
warning: `riptide-api` (test "table_extraction_integration_tests") generated 5 warnings (5 duplicates)
warning: `riptide-api` (test "browser_pool_integration") generated 5 warnings (5 duplicates)
warning: `riptide-api` (test "performance_regression") generated 5 warnings (5 duplicates)
✓ Tests passed

=========================================
All quality checks passed! ✓
Time taken: 12m 48s