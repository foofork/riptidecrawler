Query-aware duration: 325.224371ms
Query-aware RPS: 3074.80
test query_aware_tests::query_aware_week7_tests::test_performance_benchmarking ... ok

thread 'session::tests::test_session_expiration' panicked at crates/riptide-spider/src/session.rs:411:9:
assertion `left != right` failed
  left: "session_example.com_70"
 right: "session_example.com_70"
stack backtrace:
test session::tests::test_session_limits ... ok
test session::tests::test_session_validation ... ok
test strategy::tests::test_adaptive_criteria ... ok
test strategy::tests::test_best_first_processing ... ok
test strategy::tests::test_breadth_first_processing ... ok
test strategy::tests::test_default_scoring ... ok
test strategy::tests::test_depth_first_processing ... ok
test strategy::tests::test_strategy_context ... ok
test tests::config_tests::test_config_validation ... ok
test tests::config_tests::test_memory_estimation ... ok
test tests::config_tests::test_preset_configurations ... ok
test tests::config_tests::test_resource_optimization ... ok
test session::tests::test_session_state ... ok
   0: __rustc::rust_begin_unwind
             at /rustc/1159e78c4747b02ef996e55082b704c09b970588/library/std/src/panicking.rs:697:5
   1: core::panicking::panic_fmt
             at /rustc/1159e78c4747b02ef996e55082b704c09b970588/library/core/src/panicking.rs:75:14
   2: core::panicking::assert_failed_inner
             at /rustc/1159e78c4747b02ef996e55082b704c09b970588/library/core/src/panicking.rs:448:17
   3: core::panicking::assert_failed
             at /rustc/1159e78c4747b02ef996e55082b704c09b970588/library/core/src/panicking.rs:403:5
   4: riptide_spider::session::tests::test_session_expiration::{{closure}}
             at ./src/session.rs:411:9
   5: <core::pin::Pin<P> as core::future::future::Future>::poll
             at /rustc/1159e78c4747b02ef996e55082b704c09b970588/library/core/src/future/future.rs:133:9
   6: <core::pin::Pin<P> as core::future::future::Future>::poll
             at /rustc/1159e78c4747b02ef996e55082b704c09b970588/library/core/src/future/future.rs:133:9
   7: tokio::runtime::scheduler::current_thread::CoreGuard::block_on::{{closure}}::{{closure}}::{{closure}}
             at /home/runner/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-1.48.0/src/runtime/scheduler/current_thread/mod.rs:742:70
   8: tokio::task::coop::with_budget
             at /home/runner/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-1.48.0/src/task/coop/mod.rs:167:5
   9: tokio::task::coop::budget
             at /home/runner/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-1.48.0/src/task/coop/mod.rs:133:5
  10: tokio::runtime::scheduler::current_thread::CoreGuard::block_on::{{closure}}::{{closure}}
             at /home/runner/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-1.48.0/src/runtime/scheduler/current_thread/mod.rs:742:25
  11: tokio::runtime::scheduler::current_thread::Context::enter
             at /home/runner/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-1.48.0/src/runtime/scheduler/current_thread/mod.rs:432:19
  12: tokio::runtime::scheduler::current_thread::CoreGuard::block_on::{{closure}}
             at /home/runner/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-1.48.0/src/runtime/scheduler/current_thread/mod.rs:741:44
  13: tokio::runtime::scheduler::current_thread::CoreGuard::enter::{{closure}}
             at /home/runner/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-1.48.0/src/runtime/scheduler/current_thread/mod.rs:829:68
  14: tokio::runtime::context::scoped::Scoped<T>::set
             at /home/runner/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-1.48.0/src/runtime/context/scoped.rs:40:9
  15: tokio::runtime::context::set_scheduler::{{closure}}
             at /home/runner/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-1.48.0/src/runtime/context.rs:176:38
  16: std::thread::local::LocalKey<T>::try_with
             at /rustc/1159e78c4747b02ef996e55082b704c09b970588/library/std/src/thread/local.rs:315:12
  17: std::thread::local::LocalKey<T>::with
             at /rustc/1159e78c4747b02ef996e55082b704c09b970588/library/std/src/thread/local.rs:279:20
  18: tokio::runtime::context::set_scheduler
             at /home/runner/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-1.48.0/src/runtime/context.rs:176:17
  19: tokio::runtime::scheduler::current_thread::CoreGuard::enter
             at /home/runner/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-1.48.0/src/runtime/scheduler/current_thread/mod.rs:829:27
  20: tokio::runtime::scheduler::current_thread::CoreGuard::block_on
             at /home/runner/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-1.48.0/src/runtime/scheduler/current_thread/mod.rs:729:24
  21: tokio::runtime::scheduler::current_thread::CurrentThread::block_on::{{closure}}
             at /home/runner/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-1.48.0/src/runtime/scheduler/current_thread/mod.rs:200:33
  22: tokio::runtime::context::runtime::enter_runtime
             at /home/runner/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-1.48.0/src/runtime/context/runtime.rs:65:16
  23: tokio::runtime::scheduler::current_thread::CurrentThread::block_on
             at /home/runner/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-1.48.0/src/runtime/scheduler/current_thread/mod.rs:188:9
  24: tokio::runtime::runtime::Runtime::block_on_inner
             at /home/runner/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-1.48.0/src/runtime/runtime.rs:368:52
  25: tokio::runtime::runtime::Runtime::block_on
             at /home/runner/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-1.48.0/src/runtime/runtime.rs:342:18
  26: riptide_spider::session::tests::test_session_expiration
             at ./src/session.rs:411:45
  27: riptide_spider::session::tests::test_session_expiration::{{closure}}
             at ./src/session.rs:391:39
  28: core::ops::function::FnOnce::call_once
             at /rustc/1159e78c4747b02ef996e55082b704c09b970588/library/core/src/ops/function.rs:253:5
  29: core::ops::function::FnOnce::call_once
             at /rustc/1159e78c4747b02ef996e55082b704c09b970588/library/core/src/ops/function.rs:253:5
note: Some details are omitted, run with `RUST_BACKTRACE=full` for a verbose backtrace.



test tests::performance::test_frontier_performance ... ok

thread 'tests::integration::test_session_lifecycle' panicked at crates/riptide-spider/src/tests.rs:356:9:
assertion `left != right` failed
  left: "session_test.example.com_80"
 right: "session_test.example.com_80"
stack backtrace:
   0: __rustc::rust_begin_unwind
             at /rustc/1159e78c4747b02ef996e55082b704c09b970588/library/std/src/panicking.rs:697:5
   1: core::panicking::panic_fmt
             at /rustc/1159e78c4747b02ef996e55082b704c09b970588/library/core/src/panicking.rs:75:14
   2: core::panicking::assert_failed_inner
             at /rustc/1159e78c4747b02ef996e55082b704c09b970588/library/core/src/panicking.rs:448:17
   3: core::panicking::assert_failed
             at /rustc/1159e78c4747b02ef996e55082b704c09b970588/library/core/src/panicking.rs:403:5
   4: riptide_spider::tests::integration::test_session_lifecycle::{{closure}}
             at ./src/tests.rs:356:9
   5: <core::pin::Pin<P> as core::future::future::Future>::poll
             at /rustc/1159e78c4747b02ef996e55082b704c09b970588/library/core/src/future/future.rs:133:9
   6: <core::pin::Pin<P> as core::future::future::Future>::poll
             at /rustc/1159e78c4747b02ef996e55082b704c09b970588/library/core/src/future/future.rs:133:9
   7: tokio::runtime::scheduler::current_thread::CoreGuard::block_on::{{closure}}::{{closure}}::{{closure}}
             at /home/runner/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-1.48.0/src/runtime/scheduler/current_thread/mod.rs:742:70
   8: tokio::task::coop::with_budget
             at /home/runner/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-1.48.0/src/task/coop/mod.rs:167:5
   9: tokio::task::coop::budget
             at /home/runner/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-1.48.0/src/task/coop/mod.rs:133:5
  10: tokio::runtime::scheduler::current_thread::CoreGuard::block_on::{{closure}}::{{closure}}
             at /home/runner/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-1.48.0/src/runtime/scheduler/current_thread/mod.rs:742:25
  11: tokio::runtime::scheduler::current_thread::Context::enter
             at /home/runner/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-1.48.0/src/runtime/scheduler/current_thread/mod.rs:432:19
  12: tokio::runtime::scheduler::current_thread::CoreGuard::block_on::{{closure}}
             at /home/runner/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-1.48.0/src/runtime/scheduler/current_thread/mod.rs:741:44
  13: tokio::runtime::scheduler::current_thread::CoreGuard::enter::{{closure}}
             at /home/runner/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-1.48.0/src/runtime/scheduler/current_thread/mod.rs:829:68
  14: tokio::runtime::context::scoped::Scoped<T>::set
             at /home/runner/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-1.48.0/src/runtime/context/scoped.rs:40:9
  15: tokio::runtime::context::set_scheduler::{{closure}}
             at /home/runner/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-1.48.0/src/runtime/context.rs:176:38
  16: std::thread::local::LocalKey<T>::try_with
             at /rustc/1159e78c4747b02ef996e55082b704c09b970588/library/std/src/thread/local.rs:315:12
  17: std::thread::local::LocalKey<T>::with
             at /rustc/1159e78c4747b02ef996e55082b704c09b970588/library/std/src/thread/local.rs:279:20
  18: tokio::runtime::context::set_scheduler
             at /home/runner/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-1.48.0/src/runtime/context.rs:176:17
  19: tokio::runtime::scheduler::current_thread::CoreGuard::enter
             at /home/runner/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-1.48.0/src/runtime/scheduler/current_thread/mod.rs:829:27
  20: tokio::runtime::scheduler::current_thread::CoreGuard::block_on
             at /home/runner/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-1.48.0/src/runtime/scheduler/current_thread/mod.rs:729:24
  21: tokio::runtime::scheduler::current_thread::CurrentThread::block_on::{{closure}}
             at /home/runner/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-1.48.0/src/runtime/scheduler/current_thread/mod.rs:200:33
  22: tokio::runtime::context::runtime::enter_runtime
             at /home/runner/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-1.48.0/src/runtime/context/runtime.rs:65:16
  23: tokio::runtime::scheduler::current_thread::CurrentThread::block_on
             at /home/runner/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-1.48.0/src/runtime/scheduler/current_thread/mod.rs:188:9
  24: tokio::runtime::runtime::Runtime::block_on_inner
             at /home/runner/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-1.48.0/src/runtime/runtime.rs:368:52
  25: tokio::runtime::runtime::Runtime::block_on
             at /home/runner/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-1.48.0/src/runtime/runtime.rs:342:18
  26: riptide_spider::tests::integration::test_session_lifecycle
             at ./src/tests.rs:356:45
  27: riptide_spider::tests::integration::test_session_lifecycle::{{closure}}
             at ./src/tests.rs:322:38
  28: core::ops::function::FnOnce::call_once
             at /rustc/1159e78c4747b02ef996e55082b704c09b970588/library/core/src/ops/function.rs:253:5
  29: core::ops::function::FnOnce::call_once
             at /rustc/1159e78c4747b02ef996e55082b704c09b970588/library/core/src/ops/function.rs:253:5
note: Some details are omitted, run with `RUST_BACKTRACE=full` for a verbose backtrace.



failures:

failures:
    session::tests::test_session_expiration
    tests::integration::test_session_lifecycle

test result: FAILED. 104 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.41s

error: test failed, to rerun pass `-p riptide-spider --lib`
Error: Process completed with exit code 101.

AND



failures:
    error_recovery_tests::test_circuit_breaker_prevents_cascade
    error_recovery_tests::test_network_timeout_recovery
    error_recovery_tests::test_redis_connection_failure_recovery
    error_recovery_tests::test_tenant_quota_exceeded_errors

test result: FAILED. 5 passed; 4 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s

error: test failed, to rerun pass `-p riptide-api --test error_recovery`
Error: Process completed with exit code 101.

     Running tests/error_recovery.rs (target/debug/deps/error_recovery-b836d993d9290232)

running 9 tests

thread 'error_recovery_tests::test_circuit_breaker_prevents_cascade' panicked at crates/riptide-api/tests/error_recovery.rs:438:9:
assertion `left == right` failed
  left: 404
 right: 503
stack backtrace:
test error_recovery_tests::test_invalid_data_recovery ... ok
   0: __rustc::rust_begin_unwind
             at /rustc/1159e78c4747b02ef996e55082b704c09b970588/library/std/src/panicking.rs:697:5
   1: core::panicking::panic_fmt
             at /rustc/1159e78c4747b02ef996e55082b704c09b970588/library/core/src/panicking.rs:75:14
   2: core::panicking::assert_failed_inner
             at /rustc/1159e78c4747b02ef996e55082b704c09b970588/library/core/src/panicking.rs:448:17
   3: core::panicking::assert_failed
             at /rustc/1159e78c4747b02ef996e55082b704c09b970588/library/core/src/panicking.rs:403:5
   4: error_recovery::error_recovery_tests::test_circuit_breaker_prevents_cascade::{{closure}}
             at ./tests/error_recovery.rs:438:9
   5: <core::pin::Pin<P> as core::future::future::Future>::poll
             at /rustc/1159e78c4747b02ef996e55082b704c09b970588/library/core/src/future/future.rs:133:9
   6: <core::pin::Pin<P> as core::future::future::Future>::poll
             at /rustc/1159e78c4747b02ef996e55082b704c09b970588/library/core/src/future/future.rs:133:9
   7: tokio::runtime::scheduler::current_thread::CoreGuard::block_on::{{closure}}::{{closure}}::{{closure}}
             at /home/runner/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-1.48.0/src/runtime/scheduler/current_thread/mod.rs:742:70
   8: tokio::task::coop::with_budget
             at /home/runner/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-1.48.0/src/task/coop/mod.rs:167:5
   9: tokio::task::coop::budget
             at /home/runner/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-1.48.0/src/task/coop/mod.rs:133:5
  10: tokio::runtime::scheduler::current_thread::CoreGuard::block_on::{{closure}}::{{closure}}
             at /home/runner/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-1.48.0/src/runtime/scheduler/current_thread/mod.rs:742:25
  11: tokio::runtime::scheduler::current_thread::Context::enter
             at /home/runner/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-1.48.0/src/runtime/scheduler/current_thread/mod.rs:432:19
  12: tokio::runtime::scheduler::current_thread::CoreGuard::block_on::{{closure}}
             at /home/runner/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-1.48.0/src/runtime/scheduler/current_thread/mod.rs:741:44
  13: tokio::runtime::scheduler::current_thread::CoreGuard::enter::{{closure}}
             at /home/runner/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-1.48.0/src/runtime/scheduler/current_thread/mod.rs:829:68
  14: tokio::runtime::context::scoped::Scoped<T>::set
             at /home/runner/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-1.48.0/src/runtime/context/scoped.rs:40:9
  15: tokio::runtime::context::set_scheduler::{{closure}}
             at /home/runner/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-1.48.0/src/runtime/context.rs:176:38
  16: std::thread::local::LocalKey<T>::try_with
             at /rustc/1159e78c4747b02ef996e55082b704c09b970588/library/std/src/thread/local.rs:315:12
  17: std::thread::local::LocalKey<T>::with
             at /rustc/1159e78c4747b02ef996e55082b704c09b970588/library/std/src/thread/local.rs:279:20
  18: tokio::runtime::context::set_scheduler
             at /home/runner/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-1.48.0/src/runtime/context.rs:176:17
  19: tokio::runtime::scheduler::current_thread::CoreGuard::enter
             at /home/runner/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-1.48.0/src/runtime/scheduler/current_thread/mod.rs:829:27
  20: tokio::runtime::scheduler::current_thread::CoreGuard::block_on
             at /home/runner/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-1.48.0/src/runtime/scheduler/current_thread/mod.rs:729:24
  21: tokio::runtime::scheduler::current_thread::CurrentThread::block_on::{{closure}}
             at /home/runner/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-1.48.0/src/runtime/scheduler/current_thread/mod.rs:200:33
  22: tokio::runtime::context::runtime::enter_runtime
             at /home/runner/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-1.48.0/src/runtime/context/runtime.rs:65:16
  23: tokio::runtime::scheduler::current_thread::CurrentThread::block_on
             at /home/runner/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-1.48.0/src/runtime/scheduler/current_thread/mod.rs:188:9
  24: tokio::runtime::runtime::Runtime::block_on_inner
             at /home/runner/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-1.48.0/src/runtime/runtime.rs:368:52
  25: tokio::runtime::runtime::Runtime::block_on
             at /home/runner/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-1.48.0/src/runtime/runtime.rs:342:18
  26: error_recovery::error_recovery_tests::test_tenant_quota_exceeded_errors
             at ./tests/error_recovery.rs:312:63
  27: error_recovery::error_recovery_tests::test_tenant_quota_exceeded_errors::{{closure}}
             at ./tests/error_recovery.rs:245:49
  28: core::ops::function::FnOnce::call_once
             at /rustc/1159e78c4747b02ef996e55082b704c09b970588/library/core/src/ops/function.rs:253:5
  29: core::ops::function::FnOnce::call_once
             at /rustc/1159e78c4747b02ef996e55082b704c09b970588/library/core/src/ops/function.rs:253:5
note: Some details are omitted, run with `RUST_BACKTRACE=full` for a verbose backtrace.
test error_recovery_tests::test_redis_connection_failure_recovery ... FAILED
test error_recovery_tests::test_tenant_quota_exceeded_errors ... FAILED

Run # Clean any stale Chrome singleton locks from previous runs
    Updating crates.io index
   Compiling riptide-types v0.9.0 (/home/runner/work/riptidecrawler/riptidecrawler/crates/riptide-types)
   Compiling riptide-stealth v0.9.0 (/home/runner/work/riptidecrawler/riptidecrawler/crates/riptide-stealth)
   Compiling riptide-browser-abstraction v0.9.0 (/home/runner/work/riptidecrawler/riptidecrawler/crates/riptide-browser-abstraction)
   Compiling riptide-browser v0.9.0 (/home/runner/work/riptidecrawler/riptidecrawler/crates/riptide-browser)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 16.61s
     Running unittests src/lib.rs (target/debug/deps/riptide_browser-a4425af831d438f9)
running 24 tests
test cdp::tests::test_batch_command ... ok
test cdp::tests::test_batch_config_disabled ... error: test failed, to rerun pass `-p riptide-browser --lib`
Caused by:
  process didn't exit successfully: `/home/runner/work/riptidecrawler/riptidecrawler/target/debug/deps/riptide_browser-a4425af831d438f9 --test-threads=1 --nocapture` (signal: 4, SIGILL: illegal instruction)
