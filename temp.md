
test test_multiple_health_endpoints ... ok
test test_overall_status_determination ... ok
test test_unhealthy_connection_failure ... ok
test test_unhealthy_timeout ... ok
test test_health_check_timeout_enforcement ... ok

test result: ok. 30 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s

     Running tests/health_tests.rs (target/debug/deps/health_tests-d610df7626dd1100)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/integration_phase4a_tests.rs (target/debug/deps/integration_phase4a_tests-f8cc463f94efbec0)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/integration_tests.rs (target/debug/deps/integration_tests-7986c6337479c2a6)

running 24 tests

    table_extraction_tests::test_table_markdown_export

test result: FAILED. 0 passed; 24 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.58s

error: test failed, to rerun pass `-p riptide-api --test integration_tests`
Error: Process completed with exit code 101.




test result: ok. 98 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.11s

     Running unittests src/lib.rs (target/debug/deps/riptide_streaming-70a5b4009891af72)

running 39 tests
test api_handlers::tests::test_parse_format ... ok
test api_handlers::tests::test_parse_theme ... ok
test api_handlers::tests::test_get_default_config ... ok
test backpressure::tests::test_backpressure_controller_creation ... ok
test api_handlers::tests::test_health_check ... ok
test api_handlers::tests::test_list_formats ... ok

thread 'backpressure::tests::test_resource_acquisition' panicked at crates/riptide-streaming/src/backpressure.rs:496:14:
Test should complete within 30 seconds: Elapsed(())
stack backtrace:
   0: __rustc::rust_begin_unwind
             at /rustc/1159e78c4747b02ef996e55082b704c09b970588/library/std/src/panicking.rs:697:5
   1: core::panicking::panic_fmt
             at /rustc/1159e78c4747b02ef996e55082b704c09b970588/library/core/src/panicking.rs:75:14
   2: core::result::unwrap_failed
             at /rustc/1159e78c4747b02ef996e55082b704c09b970588/library/core/src/result.rs:1765:5
   3: core::result::Result<T,E>::expect
             at /rustc/1159e78c4747b02ef996e55082b704c09b970588/library/core/src/result.rs:1119:23
   4: riptide_streaming::backpressure::tests::test_resource_acquisition::{{closure}}
             at ./src/backpressure.rs:496:14
   5: <core::pin::Pin<P> as core::future::future::Future>::poll
             at /rustc/1159e78c4747b02ef996e55082b704c09b970588/library/core/src/future/future.rs:133:9
   6: tokio::runtime::park::CachedParkThread::block_on::{{closure}}
             at /home/runner/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-1.48.0/src/runtime/park.rs:285:71
   7: tokio::task::coop::with_budget
             at /home/runner/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-1.48.0/src/task/coop/mod.rs:167:5
   8: tokio::task::coop::budget
             at /home/runner/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-1.48.0/src/task/coop/mod.rs:133:5
   9: tokio::runtime::park::CachedParkThread::block_on
             at /home/runner/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-1.48.0/src/runtime/park.rs:285:31
  10: tokio::runtime::context::blocking::BlockingRegionGuard::block_on
             at /home/runner/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-1.48.0/src/runtime/context/blocking.rs:66:14
  11: tokio::runtime::scheduler::multi_thread::MultiThread::block_on::{{closure}}
             at /home/runner/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-1.48.0/src/runtime/scheduler/multi_thread/mod.rs:87:22
  12: tokio::runtime::context::runtime::enter_runtime
             at /home/runner/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-1.48.0/src/runtime/park.rs:285:71
   7: tokio::task::coop::with_budget
             at /home/runner/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-1.48.0/src/task/coop/mod.rs:167:5
   8: tokio::task::coop::budget
             at /home/runner/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-1.48.0/src/task/coop/mod.rs:133:5
   9: tokio::runtime::park::CachedParkThread::block_on
             at /home/runner/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-1.48.0/src/runtime/park.rs:285:31
  10: tokio::runtime::context::blocking::BlockingRegionGuard::block_on
             at /home/runner/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-1.48.0/src/runtime/context/blocking.rs:66:14
  11: tokio::runtime::scheduler::multi_thread::MultiThread::block_on::{{closure}}
             at /home/runner/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-1.48.0/src/runtime/scheduler/multi_thread/mod.rs:87:22
  12: tokio::runtime::context::runtime::enter_runtime
             at /home/runner/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-1.48.0/src/runtime/context/runtime.rs:65:16
  13: tokio::runtime::scheduler::multi_thread::MultiThread::block_on
             at /home/runner/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-1.48.0/src/runtime/scheduler/multi_thread/mod.rs:86:9
  14: tokio::runtime::runtime::Runtime::block_on_inner
             at /home/runner/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-1.48.0/src/runtime/runtime.rs:370:50
  15: tokio::runtime::runtime::Runtime::block_on
             at /home/runner/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-1.48.0/src/runtime/runtime.rs:342:18
  16: riptide_streaming::backpressure::tests::test_stream_registration
             at ./src/backpressure.rs:466:62
  17: riptide_streaming::backpressure::tests::test_stream_registration::{{closure}}
             at ./src/backpressure.rs:452:40
  18: core::ops::function::FnOnce::call_once
             at /rustc/1159e78c4747b02ef996e55082b704c09b970588/library/core/src/ops/function.rs:253:5
  19: core::ops::function::FnOnce::call_once
             at /rustc/1159e78c4747b02ef996e55082b704c09b970588/library/core/src/ops/function.rs:253:5
note: Some details are omitted, run with `RUST_BACKTRACE=full` for a verbose backtrace.
test backpressure::tests::test_stream_registration ... FAILED

failures:

failures:
    backpressure::tests::test_backpressure_limits
    backpressure::tests::test_memory_limits
    backpressure::tests::test_metrics_calculation
    backpressure::tests::test_resource_acquisition
    backpressure::tests::test_stream_registration

test result: FAILED. 34 passed; 5 failed; 0 ignored; 0 measured; 0 filtered out; finished in 60.09s

error: test failed, to rerun pass `-p riptide-streaming --lib`
Error: Process completed with exit code 101.