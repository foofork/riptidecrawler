/*!
# Riptide Schemas

Event and data schemas for the Riptide web scraping framework.

## Overview

This crate provides:
- Event schema definitions with versioning support
- Schema adapters for forward compatibility
- Output format conversions (JSON, Markdown)
- Schema validation

## Example

```rust
use riptide_schemas::{Event, SchemaVersion, Location, Organizer};
use chrono::Utc;

let event = Event {
    schema_version: SchemaVersion::V1,
    title: "Rust Conference 2025".to_string(),
    description: Some("Annual Rust conference".to_string()),
    start_date: Utc::now(),
    end_date: None,
    location: Some(Location {
        name: "Convention Center".to_string(),
        address: Some("123 Main St".to_string()),
        city: Some("San Francisco".to_string()),
        country: Some("USA".to_string()),
        lat_lon: Some((37.7749, -122.4194)),
    }),
    url: "https://rustconf.com".to_string(),
    organizer: Some(Organizer {
        name: "Rust Foundation".to_string(),
        email: Some("info@rustconf.com".to_string()),
        url: Some("https://rustconf.com".to_string()),
    }),
    confidence: Some(0.95),
    extraction_strategy: Some("json_ld".to_string()),
};
```
*/

pub mod events;
pub mod extraction;
pub mod formatters;

pub use events::{Event, EventV2Adapter, Location, Organizer, SchemaAdapter, SchemaVersion};
pub use extraction::{select_strategy, ExtractionStrategy};
pub use formatters::{EventFormatter, OutputFormatter};
