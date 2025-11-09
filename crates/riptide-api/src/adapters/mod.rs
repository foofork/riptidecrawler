//! Adapters for integrating external facades with internal infrastructure.
//!
//! These adapters bridge the gap between riptide-api's current architecture
//! and the port-based interfaces required by riptide-facade.

pub mod resource_pool_adapter;

pub use resource_pool_adapter::{ResourceManagerPoolAdapter, ResourceSlot};
