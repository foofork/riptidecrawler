//! WASM validation module
//!
//! This module provides validation for WASM components.
//! Only available when the `wasm-extractor` feature is enabled.

#[cfg(feature = "wasm-extractor")]
pub mod wasm_validation;

#[cfg(feature = "wasm-extractor")]
pub use wasm_validation::{
    validate_before_instantiation, ComponentMetadata, TypeMismatch, TypeSignature,
    ValidationReport, WitValidator,
};
