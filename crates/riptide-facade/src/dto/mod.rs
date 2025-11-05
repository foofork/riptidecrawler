//! Data Transfer Objects (DTOs) for public API
//!
//! This module provides DTOs that decouple internal extraction models from the public API.
//! This allows internal structures to evolve without breaking client code.

mod document;
mod mapper;
mod structured_data;

pub use document::Document;
pub use mapper::ToDto;
pub use structured_data::{Event, Product, StructuredData};
