// Re-export adaptive timeout functionality from riptide-reliability
//
// This module has been migrated to the riptide-reliability crate for better
// code organization and reusability across the Riptide ecosystem.
//
// All adaptive timeout functionality is now available at:
// riptide_reliability::timeout

pub use riptide_reliability::timeout::{
    get_global_timeout_manager, AdaptiveTimeoutManager, TimeoutConfig, TimeoutProfile,
    TimeoutStats, BACKOFF_MULTIPLIER, DEFAULT_TIMEOUT_SECS, MAX_TIMEOUT_SECS, MIN_TIMEOUT_SECS,
    SUCCESS_REDUCTION,
};
