
test backpressure::tests::test_backpressure_limits has been running for over 60 seconds
test backpressure::tests::test_memory_limits has been running for over 60 seconds
test backpressure::tests::test_metrics_calculation has been running for over 60 seconds
test backpressure::tests::test_resource_acquisition has been running for over 60 seconds




     Running tests/error_recovery.rs (target/debug/deps/error_recovery-b836d993d9290232)

running 9 tests

thread 'error_recovery_tests::test_invalid_data_recovery' panicked at crates/riptide-api/tests/error_recovery.rs:384:9:
assertion `left == right` failed
  left: 200
 right: 400
stack backtrace:
   0: __rustc::rust_begin_unwind
             at /rustc/1159e78c4747b02ef996e55082b704c09b970588/library/std/src/panicking.rs:697:5
   1: core::panicking::panic_fmt
             at /rustc/1159e78c4747b02ef996e55082b704c09b970588/library/core/src/panicking.rs:75:14
   2: core::panicking::assert_failed_inner
             at /rustc/1159e78c4747b02ef996e55082b704c09b970588/library/core/src/panicking.rs:448:17
   3: core::panicking::assert_failed
             at /rustc/1159e78c4747b02ef996e55082b704c09b970588/library/core/src/panicking.rs:403:5
   4: error_recovery::error_recovery_tests::test_invalid_data_recovery::{{closure}}
             at ./tests/error_recovery.rs:384:9
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

error: test failed, to rerun pass `-p riptide-api --test error_recovery`
Error: Process completed with exit code 101.