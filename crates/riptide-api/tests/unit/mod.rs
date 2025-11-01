/// Unit tests module
///
/// This module contains unit tests for individual components of the RipTide API.
/// Unit tests focus on testing isolated functionality without external dependencies.

pub mod test_state;
pub mod test_errors;
pub mod test_validation;
pub mod test_pipeline;
pub mod spider_crawled_page_tests;

#[cfg(test)]
mod test_runner {
    /// Test utilities and common setup for unit tests

    use std::env;

    /// Setup test environment with clean state
    pub fn setup_test_env() {
        // Clear any environment variables that might affect tests
        let test_vars = vec![
            "REDIS_URL", "WASM_EXTRACTOR_PATH", "MAX_CONCURRENCY",
            "CACHE_TTL", "GATE_HI_THRESHOLD", "GATE_LO_THRESHOLD", "HEADLESS_URL"
        ];

        for var in test_vars {
            env::remove_var(var);
        }
    }

    /// Cleanup test environment
    pub fn cleanup_test_env() {
        // Restore any necessary environment variables
        // This would be called in test cleanup if needed
    }
}

/// Run all unit tests
#[cfg(test)]
mod integration {
    use super::test_runner::*;

    #[test]
    fn run_all_unit_tests() {
        setup_test_env();

        // Unit tests are run automatically by the test framework
        // This is just a placeholder to ensure module compilation

        cleanup_test_env();
    }
}