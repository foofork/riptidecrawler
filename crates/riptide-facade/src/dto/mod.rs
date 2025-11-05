//! Data Transfer Objects (DTOs) for public API
//!
//! This module provides DTOs that decouple internal extraction models from the public API.
//! This allows internal structures to evolve without breaking client code.

mod document;
mod structured_data;
mod mapper;

pub use document::Document;
pub use structured_data::{StructuredData, Event, Product};
pub use mapper::ToDto;
