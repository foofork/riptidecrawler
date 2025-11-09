//! Browser Engine Abstraction Layer
//!
//! This module provides trait-only abstractions with NO concrete CDP types.
//! All concrete implementations are in the ../cdp/ module.

mod error;
mod params;
mod traits;

// Public exports (traits and types only, NO implementations)
pub use error::{AbstractionError, AbstractionResult};
pub use params::{NavigateParams, PdfParams, ScreenshotFormat, ScreenshotParams, WaitUntil};
pub use traits::{BrowserEngine, EngineType, PageHandle};
