#![allow(clippy::all, dead_code, unused)]

//! Tests for execution mode logic
//!
//! Coverage includes:
//! - Mode selection from flags
//! - Environment variable parsing
//! - Permission checks
//! - Mode descriptions

use riptide_cli::execution_mode::{get_execution_mode, ExecutionMode};
use std::env;

#[test]
fn test_direct_flag_precedence() {
    let mode = ExecutionMode::from_flags(true, true);
    assert_eq!(mode, ExecutionMode::DirectOnly);
}

#[test]
fn test_api_only_flag() {
    let mode = ExecutionMode::from_flags(false, true);
    assert_eq!(mode, ExecutionMode::ApiOnly);
}

#[test]
fn test_no_flags_default() {
    // Clear env var if set
    env::remove_var("RIPTIDE_EXECUTION_MODE");
    let mode = ExecutionMode::from_flags(false, false);
    assert_eq!(mode, ExecutionMode::ApiFirst);
}

#[test]
fn test_direct_only_mode() {
    let mode = ExecutionMode::from_flags(true, false);
    assert_eq!(mode, ExecutionMode::DirectOnly);
}

#[test]
fn test_api_first_allows_api() {
    let mode = ExecutionMode::ApiFirst;
    assert!(mode.allows_api());
}

#[test]
fn test_api_first_allows_direct() {
    let mode = ExecutionMode::ApiFirst;
    assert!(mode.allows_direct());
}

#[test]
fn test_api_first_allows_fallback() {
    let mode = ExecutionMode::ApiFirst;
    assert!(mode.allows_fallback());
}

#[test]
fn test_api_only_allows_api() {
    let mode = ExecutionMode::ApiOnly;
    assert!(mode.allows_api());
}

#[test]
fn test_api_only_denies_direct() {
    let mode = ExecutionMode::ApiOnly;
    assert!(!mode.allows_direct());
}

#[test]
fn test_api_only_denies_fallback() {
    let mode = ExecutionMode::ApiOnly;
    assert!(!mode.allows_fallback());
}

#[test]
fn test_direct_only_denies_api() {
    let mode = ExecutionMode::DirectOnly;
    assert!(!mode.allows_api());
}

#[test]
fn test_direct_only_allows_direct() {
    let mode = ExecutionMode::DirectOnly;
    assert!(mode.allows_direct());
}

#[test]
fn test_direct_only_denies_fallback() {
    let mode = ExecutionMode::DirectOnly;
    assert!(!mode.allows_fallback());
}

#[test]
fn test_mode_descriptions() {
    assert_eq!(
        ExecutionMode::ApiFirst.description(),
        "API-first with fallback"
    );
    assert_eq!(
        ExecutionMode::ApiOnly.description(),
        "API-only (no fallback)"
    );
    assert_eq!(
        ExecutionMode::DirectOnly.description(),
        "Direct execution (offline)"
    );
}

#[test]
fn test_env_var_direct() {
    env::set_var("RIPTIDE_EXECUTION_MODE", "direct");
    let mode = ExecutionMode::from_flags(false, false);
    assert_eq!(mode, ExecutionMode::DirectOnly);
    env::remove_var("RIPTIDE_EXECUTION_MODE");
}

#[test]
fn test_env_var_offline() {
    env::set_var("RIPTIDE_EXECUTION_MODE", "offline");
    let mode = ExecutionMode::from_flags(false, false);
    assert_eq!(mode, ExecutionMode::DirectOnly);
    env::remove_var("RIPTIDE_EXECUTION_MODE");
}

#[test]
fn test_env_var_api_only() {
    env::set_var("RIPTIDE_EXECUTION_MODE", "api-only");
    let mode = ExecutionMode::from_flags(false, false);
    assert_eq!(mode, ExecutionMode::ApiOnly);
    env::remove_var("RIPTIDE_EXECUTION_MODE");
}

#[test]
fn test_env_var_api_only_underscore() {
    env::set_var("RIPTIDE_EXECUTION_MODE", "api_only");
    let mode = ExecutionMode::from_flags(false, false);
    assert_eq!(mode, ExecutionMode::ApiOnly);
    env::remove_var("RIPTIDE_EXECUTION_MODE");
}

#[test]
fn test_env_var_api_first() {
    env::set_var("RIPTIDE_EXECUTION_MODE", "api-first");
    let mode = ExecutionMode::from_flags(false, false);
    assert_eq!(mode, ExecutionMode::ApiFirst);
    env::remove_var("RIPTIDE_EXECUTION_MODE");
}

#[test]
fn test_env_var_invalid_defaults_to_api_first() {
    env::set_var("RIPTIDE_EXECUTION_MODE", "invalid-mode");
    let mode = ExecutionMode::from_flags(false, false);
    assert_eq!(mode, ExecutionMode::ApiFirst);
    env::remove_var("RIPTIDE_EXECUTION_MODE");
}

#[test]
fn test_env_var_case_insensitive() {
    env::set_var("RIPTIDE_EXECUTION_MODE", "DIRECT");
    let mode = ExecutionMode::from_flags(false, false);
    assert_eq!(mode, ExecutionMode::DirectOnly);
    env::remove_var("RIPTIDE_EXECUTION_MODE");
}

#[test]
fn test_flags_override_env_var() {
    env::set_var("RIPTIDE_EXECUTION_MODE", "api-only");
    let mode = ExecutionMode::from_flags(true, false);
    assert_eq!(mode, ExecutionMode::DirectOnly);
    env::remove_var("RIPTIDE_EXECUTION_MODE");
}

#[test]
fn test_get_execution_mode_wrapper() {
    let mode = get_execution_mode(false, true);
    assert_eq!(mode, ExecutionMode::ApiOnly);
}

#[test]
fn test_mode_equality() {
    assert_eq!(ExecutionMode::ApiFirst, ExecutionMode::ApiFirst);
    assert_eq!(ExecutionMode::ApiOnly, ExecutionMode::ApiOnly);
    assert_eq!(ExecutionMode::DirectOnly, ExecutionMode::DirectOnly);
}

#[test]
fn test_mode_inequality() {
    assert_ne!(ExecutionMode::ApiFirst, ExecutionMode::ApiOnly);
    assert_ne!(ExecutionMode::ApiFirst, ExecutionMode::DirectOnly);
    assert_ne!(ExecutionMode::ApiOnly, ExecutionMode::DirectOnly);
}

#[test]
fn test_all_permission_combinations() {
    let modes = vec![
        ExecutionMode::ApiFirst,
        ExecutionMode::ApiOnly,
        ExecutionMode::DirectOnly,
    ];

    for mode in modes {
        // Test that permission methods are consistent
        if mode.allows_fallback() {
            assert!(mode.allows_api());
            assert!(mode.allows_direct());
        }
    }
}

#[test]
fn test_mode_copy_trait() {
    let mode1 = ExecutionMode::ApiFirst;
    let mode2 = mode1; // Copy
    assert_eq!(mode1, mode2);
}

#[test]
fn test_mode_clone_trait() {
    let mode1 = ExecutionMode::DirectOnly;
    let mode2 = mode1.clone();
    assert_eq!(mode1, mode2);
}

#[test]
fn test_mode_debug_trait() {
    let mode = ExecutionMode::ApiFirst;
    let debug_str = format!("{:?}", mode);
    assert!(debug_str.contains("ApiFirst"));
}
