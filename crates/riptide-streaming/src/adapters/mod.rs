//! Adapters implementing port traits for streaming infrastructure
//!
//! This module contains adapter implementations that bridge concrete
//! streaming infrastructure with abstract port traits, enabling
//! dependency inversion in the hexagonal architecture.

pub mod streaming_module_adapter;

pub use streaming_module_adapter::StreamingModuleAdapter;
