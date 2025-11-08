//! Handler-level facades for business logic extraction
//!
//! This module contains facades that extract complex business logic
//! from HTTP handlers, following the thin handler pattern.
//!
//! **Sprint 3.2 Refactoring**:
//! - `CrawlHandlerFacade`: Extract crawl handler business logic (395 → 60 LOC)

pub mod crawl_handler_facade;

pub use crawl_handler_facade::CrawlHandlerFacade;
