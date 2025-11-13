/// Command modules for the RipTide CLI
///
/// Each module corresponds to a top-level command and implements:
/// - Args struct with clap derives
/// - execute() function that uses ApiClient
/// - Result formatting and output handling
pub mod crawl;
pub mod doctor;
pub mod extract;
pub mod render;
pub mod search;
pub mod session; // API-based session management
pub mod session_api;
pub mod spider;
pub mod strategies;

#[cfg(feature = "riptide-pdf")]
pub mod pdf;
