//! Comprehensive test suite for RipTide API.

#[cfg(test)]
mod event_bus_integration_tests;

#[cfg(test)]
mod facade_integration_tests;

#[cfg(test)]
mod middleware_validation_tests;

#[cfg(test)]
mod resource_controls;

#[cfg(test)]
pub mod test_helpers;

#[cfg(test)]
mod strategy_selection_tests;
// mod appstate_migration_tests; // Removed - will be created during migration
