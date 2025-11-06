/*!
# Event Schema

Event schema definitions with versioning support for forward compatibility.

## Schema Versioning

The event schema uses a simple string-based versioning system to allow for
future evolution without breaking existing code:

```rust
use riptide_schemas::{Event, SchemaVersion};

let event = Event {
    schema_version: SchemaVersion::V1,
    // ... other fields
    ..Default::default()
};
```

## Adapter Pattern

The `SchemaAdapter` trait enables conversion between schema versions:

```rust
use riptide_schemas::{Event, EventV2Adapter, SchemaAdapter};

let v1_event = Event::default();
let v2_event = EventV2Adapter::from_v1(v1_event)?;
```
*/

use chrono::{DateTime, Utc};
use riptide_types::Result;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Event schema version for forward compatibility
///
/// Using a simple string-based versioning system allows for easy evolution
/// without breaking existing code. Future versions (V2, V3) can be added
/// without modifying existing V1 code.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum SchemaVersion {
    /// Version 1.0 schema (initial release)
    V1,
    // V2 will be added in future versions without breaking existing code
}

impl Default for SchemaVersion {
    fn default() -> Self {
        SchemaVersion::V1
    }
}

impl std::fmt::Display for SchemaVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SchemaVersion::V1 => write!(f, "v1"),
        }
    }
}

/// Event extracted from web content
///
/// Represents a single event with temporal, location, and organizational metadata.
/// Includes schema versioning for forward compatibility.
///
/// # Example
///
/// ```rust
/// use riptide_schemas::{Event, SchemaVersion, Location};
/// use chrono::Utc;
///
/// let event = Event {
///     schema_version: SchemaVersion::V1,
///     title: "Rust Meetup".to_string(),
///     description: Some("Monthly Rust community meetup".to_string()),
///     start_date: Utc::now(),
///     end_date: None,
///     location: Some(Location {
///         name: "Tech Hub".to_string(),
///         address: Some("456 Market St".to_string()),
///         city: Some("San Francisco".to_string()),
///         country: Some("USA".to_string()),
///         lat_lon: Some((37.7749, -122.4194)),
///     }),
///     url: "https://meetup.com/rust-sf".to_string(),
///     organizer: None,
///     confidence: Some(0.92),
///     extraction_strategy: Some("css".to_string()),
/// };
/// ```
#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct Event {
    /// Schema version for evolution path
    #[serde(default)]
    pub schema_version: SchemaVersion,

    /// Event title (required)
    pub title: String,

    /// Event description (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Event start date/time (required)
    pub start_date: DateTime<Utc>,

    /// Event end date/time (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_date: Option<DateTime<Utc>>,

    /// Event location (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<Location>,

    /// Event URL (required)
    pub url: String,

    /// Event organizer (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organizer: Option<Organizer>,

    /// Confidence score (0.0-1.0) for extraction quality
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f32>,

    /// Extraction strategy used (e.g., "json_ld", "css", "ics")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extraction_strategy: Option<String>,
}

impl Default for Event {
    fn default() -> Self {
        Self {
            schema_version: SchemaVersion::default(),
            title: String::new(),
            description: None,
            start_date: Utc::now(),
            end_date: None,
            location: None,
            url: String::new(),
            organizer: None,
            confidence: None,
            extraction_strategy: None,
        }
    }
}

/// Physical or virtual location for an event
///
/// Supports both physical addresses and virtual locations (URLs).
/// Includes optional geocoding (lat/lon) for mapping integration.
#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct Location {
    /// Location name (e.g., "Convention Center", "Virtual")
    pub name: String,

    /// Street address (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,

    /// City (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub city: Option<String>,

    /// Country (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,

    /// Latitude and longitude (optional)
    /// Format: (latitude, longitude)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lat_lon: Option<(f64, f64)>,
}

impl Default for Location {
    fn default() -> Self {
        Self {
            name: String::new(),
            address: None,
            city: None,
            country: None,
            lat_lon: None,
        }
    }
}

/// Event organizer information
///
/// Represents the individual or organization hosting the event.
#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct Organizer {
    /// Organizer name (required)
    pub name: String,

    /// Contact email (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,

    /// Organizer website URL (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

impl Default for Organizer {
    fn default() -> Self {
        Self {
            name: String::new(),
            email: None,
            url: None,
        }
    }
}

/// Adapter pattern for schema evolution
///
/// Enables conversion between different schema versions without breaking
/// existing code. Future versions can implement this trait to provide
/// backward and forward compatibility.
///
/// # Example
///
/// ```rust
/// use riptide_schemas::{Event, EventV2Adapter, SchemaAdapter};
///
/// let v1_event = Event::default();
/// // In the future, this will convert to V2 format
/// let converted = EventV2Adapter::from_v1(v1_event)?;
/// # Ok::<(), anyhow::Error>(())
/// ```
pub trait SchemaAdapter<T> {
    /// Convert from V1 schema to target schema
    fn from_v1(event: Event) -> Result<T>;

    /// Convert to V1 schema from source schema
    fn to_v1(value: &T) -> Event;
}

/// V2 adapter (stub for future use)
///
/// This is a placeholder for future schema evolution. In v1.0, it simply
/// returns the V1 event unchanged. In future versions, it will perform
/// actual schema transformations.
pub struct EventV2Adapter;

impl SchemaAdapter<Event> for EventV2Adapter {
    fn from_v1(event: Event) -> Result<Event> {
        // Identity for now, will evolve in v1.1+
        Ok(event)
    }

    fn to_v1(event: &Event) -> Event {
        event.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_default() {
        let event = Event::default();
        assert_eq!(event.schema_version, SchemaVersion::V1);
        assert!(event.title.is_empty());
        assert!(event.description.is_none());
    }

    #[test]
    fn test_schema_version_display() {
        assert_eq!(SchemaVersion::V1.to_string(), "v1");
    }

    #[test]
    fn test_event_serialization() {
        let event = Event {
            schema_version: SchemaVersion::V1,
            title: "Test Event".to_string(),
            description: Some("Description".to_string()),
            start_date: Utc::now(),
            end_date: None,
            location: None,
            url: "https://example.com".to_string(),
            organizer: None,
            confidence: Some(0.95),
            extraction_strategy: Some("json_ld".to_string()),
        };

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: Event = serde_json::from_str(&json).unwrap();

        assert_eq!(event.title, deserialized.title);
        assert_eq!(event.schema_version, deserialized.schema_version);
    }

    #[test]
    fn test_location_creation() {
        let location = Location {
            name: "Convention Center".to_string(),
            address: Some("123 Main St".to_string()),
            city: Some("San Francisco".to_string()),
            country: Some("USA".to_string()),
            lat_lon: Some((37.7749, -122.4194)),
        };

        assert_eq!(location.name, "Convention Center");
        assert_eq!(location.city, Some("San Francisco".to_string()));
    }

    #[test]
    fn test_organizer_creation() {
        let organizer = Organizer {
            name: "Rust Foundation".to_string(),
            email: Some("info@rust-lang.org".to_string()),
            url: Some("https://foundation.rust-lang.org".to_string()),
        };

        assert_eq!(organizer.name, "Rust Foundation");
        assert!(organizer.email.is_some());
    }

    #[test]
    fn test_schema_adapter_v2() {
        let event = Event::default();
        let converted = EventV2Adapter::from_v1(event.clone()).unwrap();
        assert_eq!(event.title, converted.title);

        let back = EventV2Adapter::to_v1(&converted);
        assert_eq!(event.title, back.title);
    }
}
