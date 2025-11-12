//! Adapters for implementing port traits with concrete event infrastructure
//!
//! This module contains adapter implementations that bridge the concrete
//! EventBus implementation with abstract port trait interfaces.

pub mod event_bus_adapter;

pub use event_bus_adapter::EventBusAdapter;
