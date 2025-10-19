//! WASM validation module
//!
//! This module provides validation for WASM components.

pub mod wasm_validation;

pub use wasm_validation::{
    validate_before_instantiation, ComponentMetadata, TypeMismatch, TypeSignature,
    ValidationReport, WitValidator,
};
