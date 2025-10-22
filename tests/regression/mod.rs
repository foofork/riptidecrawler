// Regression Tests Module
//
// Regression and golden tests to prevent feature breakage
// and ensure backward compatibility.

pub mod adaptive_timeout_tests;
pub mod wasm_aot_cache_tests;

pub mod golden {
    pub mod golden_test_cli;
    pub mod golden_tests;
}
