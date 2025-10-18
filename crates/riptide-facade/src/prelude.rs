//! Convenience re-exports for common usage.
//!
//! Import this module to get access to the most commonly used types:
//!
//! ```
//! use riptide_facade::prelude::*;
//! ```

pub use crate::{
    builder::RiptideBuilder,
    config::RiptideConfig,
    error::{RiptideError, RiptideResult},
    facades::ScraperFacade,
    Riptide,
};
