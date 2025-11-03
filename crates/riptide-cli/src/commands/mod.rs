pub mod config;
pub mod doctor;
/// Command modules for the RipTide CLI
///
/// Each module corresponds to a top-level command and implements:
/// - Args struct with clap derives
/// - execute() function that uses ApiClient
/// - Result formatting and output handling
pub mod extract;
pub mod render;
pub mod search;
pub mod session; // Local session management (existing)
pub mod session_api;
pub mod spider; // API-based session management (new)
