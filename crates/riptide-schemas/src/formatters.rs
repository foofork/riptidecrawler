/*!
# Output Formatters

Universal format conversion for extracted data.

## Overview

Provides formatters for converting extracted events and documents to various output formats:
- **JSON**: Machine-readable structured data
- **Markdown**: Human-readable documentation format
- **CSV, iCal, YAML**: Deferred to v1.1

## Example

```rust
use riptide_schemas::{Event, formatters::OutputFormatter};

let event = Event::default();

// Convert to JSON
let json = event.to_json()?;

// Convert to Markdown
let markdown = event.to_markdown()?;
# Ok::<(), anyhow::Error>(())
```
*/

use crate::events::{Event, Location, Organizer};
use anyhow::Result;
use chrono::{DateTime, Utc};

/// Universal output formatter trait
///
/// Provides methods for converting data to common output formats.
/// v1.0 supports JSON and Markdown; CSV, YAML deferred to v1.1.
pub trait OutputFormatter {
    /// Convert to JSON format
    ///
    /// Produces structured JSON output suitable for APIs and data processing.
    ///
    /// # Example
    ///
    /// ```rust
    /// use riptide_schemas::{Event, formatters::OutputFormatter};
    ///
    /// let event = Event::default();
    /// let json = event.to_json()?;
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    fn to_json(&self) -> Result<String>;

    /// Convert to Markdown format
    ///
    /// Produces human-readable Markdown documentation.
    ///
    /// # Example
    ///
    /// ```rust
    /// use riptide_schemas::{Event, formatters::OutputFormatter};
    ///
    /// let event = Event::default();
    /// let markdown = event.to_markdown()?;
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    fn to_markdown(&self) -> Result<String>;

    // CSV, YAML deferred to v1.1
    // fn to_yaml(&self) -> Result<String>;
}

/// Event-specific formatter trait
///
/// Extends OutputFormatter with event-specific format conversions.
/// v1.0 supports JSON and Markdown; iCalendar, CSV deferred to v1.1.
pub trait EventFormatter: OutputFormatter {
    // iCalendar and CSV deferred to v1.1
    // fn to_icalendar(&self) -> Result<String>;
    // fn to_csv(&self) -> Result<String>;
}

impl OutputFormatter for Event {
    fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    fn to_markdown(&self) -> Result<String> {
        let mut md = String::new();

        // Title
        md.push_str(&format!("# {}\n\n", self.title));

        // Description
        if let Some(description) = &self.description {
            md.push_str(&format!("{}\n\n", description));
        }

        // Metadata section
        md.push_str("## Event Details\n\n");

        // Date/time
        md.push_str(&format!(
            "- **Start**: {}\n",
            format_datetime(&self.start_date)
        ));
        if let Some(end_date) = &self.end_date {
            md.push_str(&format!("- **End**: {}\n", format_datetime(end_date)));
        }

        // Location
        if let Some(location) = &self.location {
            md.push_str(&format!("- **Location**: {}\n", format_location(location)));
        }

        // URL
        md.push_str(&format!("- **URL**: [{}]({})\n", self.url, self.url));

        // Organizer
        if let Some(organizer) = &self.organizer {
            md.push_str(&format!(
                "- **Organizer**: {}\n",
                format_organizer(organizer)
            ));
        }

        // Technical metadata
        if self.confidence.is_some() || self.extraction_strategy.is_some() {
            md.push_str("\n## Extraction Metadata\n\n");

            if let Some(confidence) = self.confidence {
                md.push_str(&format!("- **Confidence**: {:.1}%\n", confidence * 100.0));
            }

            if let Some(strategy) = &self.extraction_strategy {
                md.push_str(&format!("- **Strategy**: {}\n", strategy));
            }

            md.push_str(&format!("- **Schema Version**: {}\n", self.schema_version));
        }

        Ok(md)
    }
}

impl EventFormatter for Event {
    // Future: iCalendar, CSV in v1.1
}

/// Format a datetime for Markdown display
fn format_datetime(dt: &DateTime<Utc>) -> String {
    dt.format("%B %d, %Y at %I:%M %p UTC").to_string()
}

/// Format a location for Markdown display
fn format_location(location: &Location) -> String {
    let mut parts = vec![location.name.clone()];

    if let Some(address) = &location.address {
        parts.push(address.clone());
    }

    if let Some(city) = &location.city {
        if let Some(country) = &location.country {
            parts.push(format!("{}, {}", city, country));
        } else {
            parts.push(city.clone());
        }
    } else if let Some(country) = &location.country {
        parts.push(country.clone());
    }

    if let Some((lat, lon)) = location.lat_lon {
        parts.push(format!("({:.4}, {:.4})", lat, lon));
    }

    parts.join(", ")
}

/// Format an organizer for Markdown display
fn format_organizer(organizer: &Organizer) -> String {
    let mut parts = vec![organizer.name.clone()];

    if let Some(email) = &organizer.email {
        parts.push(format!("<{}>", email));
    }

    if let Some(url) = &organizer.url {
        parts.push(format!("[Website]({})", url));
    }

    parts.join(" ")
}

/// Collection of events with batch formatting support
#[derive(Debug, Clone)]
pub struct EventCollection {
    pub events: Vec<Event>,
}

impl EventCollection {
    pub fn new(events: Vec<Event>) -> Self {
        Self { events }
    }
}

impl OutputFormatter for EventCollection {
    fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string_pretty(&self.events)?)
    }

    fn to_markdown(&self) -> Result<String> {
        let mut md = String::new();

        md.push_str(&format!("# Events ({} total)\n\n", self.events.len()));

        for (i, event) in self.events.iter().enumerate() {
            md.push_str(&format!("## {}. {}\n\n", i + 1, event.title));

            if let Some(description) = &event.description {
                md.push_str(&format!("{}\n\n", description));
            }

            md.push_str(&format!(
                "- **Start**: {}\n",
                format_datetime(&event.start_date)
            ));

            if let Some(location) = &event.location {
                md.push_str(&format!("- **Location**: {}\n", format_location(location)));
            }

            md.push_str(&format!("- **URL**: [Link]({})\n\n", event.url));

            md.push_str("---\n\n");
        }

        Ok(md)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::SchemaVersion;

    fn create_test_event() -> Event {
        Event {
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
                url: Some("https://foundation.rust-lang.org".to_string()),
            }),
            confidence: Some(0.95),
            extraction_strategy: Some("json_ld".to_string()),
        }
    }

    #[test]
    fn test_event_to_json() {
        let event = create_test_event();
        let json = event.to_json().unwrap();

        assert!(json.contains("Rust Conference 2025"));
        assert!(json.contains("rustconf.com"));
        assert!(json.contains("Convention Center"));
    }

    #[test]
    fn test_event_to_markdown() {
        let event = create_test_event();
        let markdown = event.to_markdown().unwrap();

        assert!(markdown.contains("# Rust Conference 2025"));
        assert!(markdown.contains("Annual Rust conference"));
        assert!(markdown.contains("Convention Center"));
        assert!(markdown.contains("San Francisco, USA"));
        assert!(markdown.contains("Rust Foundation"));
        assert!(markdown.contains("95.0%"));
        assert!(markdown.contains("json_ld"));
    }

    #[test]
    fn test_event_collection_to_json() {
        let events = vec![create_test_event(), create_test_event()];
        let collection = EventCollection::new(events);

        let json = collection.to_json().unwrap();
        assert!(json.contains("Rust Conference 2025"));
    }

    #[test]
    fn test_event_collection_to_markdown() {
        let events = vec![create_test_event(), create_test_event()];
        let collection = EventCollection::new(events);

        let markdown = collection.to_markdown().unwrap();
        assert!(markdown.contains("Events (2 total)"));
        assert!(markdown.contains("## 1. Rust Conference 2025"));
        assert!(markdown.contains("## 2. Rust Conference 2025"));
    }

    #[test]
    fn test_format_datetime() {
        let dt = Utc::now();
        let formatted = format_datetime(&dt);
        assert!(formatted.contains("UTC"));
    }

    #[test]
    fn test_format_location_full() {
        let location = Location {
            name: "Test Venue".to_string(),
            address: Some("123 St".to_string()),
            city: Some("SF".to_string()),
            country: Some("USA".to_string()),
            lat_lon: Some((37.0, -122.0)),
        };

        let formatted = format_location(&location);
        assert!(formatted.contains("Test Venue"));
        assert!(formatted.contains("123 St"));
        assert!(formatted.contains("SF, USA"));
        assert!(formatted.contains("37."));
    }

    #[test]
    fn test_format_organizer() {
        let organizer = Organizer {
            name: "Test Org".to_string(),
            email: Some("test@example.com".to_string()),
            url: Some("https://example.com".to_string()),
        };

        let formatted = format_organizer(&organizer);
        assert!(formatted.contains("Test Org"));
        assert!(formatted.contains("<test@example.com>"));
        assert!(formatted.contains("[Website]"));
    }
}
