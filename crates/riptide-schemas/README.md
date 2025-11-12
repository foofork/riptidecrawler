# 🛠️ RipTide Schemas - Data Schemas & Validation

**Category:** Data & Serialization
**Purpose:** Event schemas, extraction strategies, and output formatters with versioning support

## Quick Overview

`riptide-schemas` provides structured data schemas for the RipTide web scraping framework. It defines the format for extracted events, provides multiple extraction strategies, and offers universal format conversion (JSON, Markdown, etc.) with schema versioning for easy evolution.

## Why This Exists

Web scraping produces diverse data that needs:
- Consistent structure for storage and processing
- Multiple extraction strategies for different content types
- Format conversion for different consumers
- Schema versioning for forward compatibility
- Validation to ensure data quality

This crate centralizes all data schema definitions and provides tools for working with them.

## Key Features

- **Event Schema**: Structured event data with rich metadata
- **8 Extraction Strategies**: Modular strategies for different content types
- **Output Formatters**: Convert to JSON, Markdown, CSV, and more
- **Schema Validation**: Automatic validation with JSON Schema support
- **Forward Compatibility**: Schema versioning for evolution
- **Type Safety**: Strong typing with serde serialization

## Quick Start

```rust
use riptide_schemas::{Event, SchemaVersion, Location, Organizer};
use chrono::Utc;

// Create an event
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
        url: Some("https://foundation.rust-lang.org".to_string()),
    }),
    confidence: Some(0.95),
    extraction_strategy: Some("json_ld".to_string()),
};

// Convert to JSON
let json = serde_json::to_string_pretty(&event)?;
println!("{}", json);
```

## Event Schema

The core data structure for extracted events:

```rust
use riptide_schemas::{Event, SchemaVersion, Location, Organizer};
use chrono::{DateTime, Utc};

pub struct Event {
    /// Schema version for forward compatibility
    pub schema_version: SchemaVersion,

    // Core fields
    pub title: String,
    pub description: Option<String>,
    pub start_date: DateTime<Utc>,
    pub end_date: Option<DateTime<Utc>>,

    // Location & organization
    pub location: Option<Location>,
    pub organizer: Option<Organizer>,
    pub url: String,

    // Metadata
    pub confidence: Option<f32>,
    pub extraction_strategy: Option<String>,
}
```

### Location Schema

Structured location data with coordinates:

```rust
pub struct Location {
    pub name: String,
    pub address: Option<String>,
    pub city: Option<String>,
    pub country: Option<String>,
    pub lat_lon: Option<(f64, f64)>, // (latitude, longitude)
}
```

### Organizer Schema

Event organizer information:

```rust
pub struct Organizer {
    pub name: String,
    pub email: Option<String>,
    pub url: Option<String>,
}
```

## Extraction Strategies

RipTide supports 8 extraction strategies, each optimized for different content types:

### 1. ICS (iCalendar)

For extracting events from `.ics` calendar files:

```rust
use riptide_schemas::extraction::{ExtractionStrategy, select_strategy};

let strategy = ExtractionStrategy::ICS;

// Auto-detect ICS content
let ics_content = r#"BEGIN:VCALENDAR
VERSION:2.0
BEGIN:VEVENT
SUMMARY:Meeting
DTSTART:20250115T100000Z
END:VEVENT
END:VCALENDAR"#;

let auto_strategy = select_strategy(ics_content, "text/calendar");
assert_eq!(auto_strategy, ExtractionStrategy::ICS);
```

### 2. JSON-LD (Structured Data)

For extracting from JSON-LD structured data embedded in HTML:

```rust
let html = r#"<script type="application/ld+json">
{
  "@context": "https://schema.org",
  "@type": "Event",
  "name": "Rust Conference",
  "startDate": "2025-01-15"
}
</script>"#;

let strategy = select_strategy(html, "text/html");
assert_eq!(strategy, ExtractionStrategy::JsonLd);
```

### 3. CSS Selectors

For extracting using CSS selectors:

```rust
let strategy = ExtractionStrategy::CSS(".event-title, .event-date".to_string());

// Use to extract from HTML
// let title = document.select(&strategy.selector())?;
```

### 4. Regex Patterns

For pattern-based extraction:

```rust
let strategy = ExtractionStrategy::Regex(r"\d{4}-\d{2}-\d{2}".to_string());

// Extract dates from text
// let matches = regex.find_all(text);
```

### 5. Custom Rules

For complex extraction logic:

```rust
let strategy = ExtractionStrategy::Rules;

// Apply custom extraction rules
// Rules defined in configuration
```

### 6. LLM-Powered

For AI-powered extraction:

```rust
let strategy = ExtractionStrategy::LLM("openai".to_string());

// Use LLM to extract structured data
// let result = llm.extract(content, schema)?;
```

### 7. Browser Rendering

For JavaScript-heavy sites:

```rust
let strategy = ExtractionStrategy::Browser;

// Render page in headless browser first
// Then extract from rendered DOM
```

### 8. WASM Extractors

For custom WebAssembly extractors:

```rust
let strategy = ExtractionStrategy::WASM;

// Load and execute WASM module
// let result = wasm_module.extract(content)?;
```

## Auto-Detection

Automatically select the best strategy:

```rust
use riptide_schemas::extraction::select_strategy;

// Detect from content and content-type
let html = r#"<script type="application/ld+json">...</script>"#;
let strategy = select_strategy(html, "text/html");

// Returns: ExtractionStrategy::JsonLd

let ics = "BEGIN:VCALENDAR\n...";
let strategy = select_strategy(ics, "text/calendar");

// Returns: ExtractionStrategy::ICS
```

## Output Formatters

Convert events to different formats:

### JSON Output

```rust
use riptide_schemas::formatters::OutputFormatter;

let event = /* create event */;

// Compact JSON
let json = event.to_json()?;

// Pretty JSON
let pretty_json = serde_json::to_string_pretty(&event)?;
```

Example output:
```json
{
  "schema_version": "v1",
  "title": "Rust Conference 2025",
  "description": "Annual Rust conference",
  "start_date": "2025-01-15T10:00:00Z",
  "location": {
    "name": "Convention Center",
    "city": "San Francisco",
    "country": "USA",
    "lat_lon": [37.7749, -122.4194]
  },
  "url": "https://rustconf.com",
  "confidence": 0.95,
  "extraction_strategy": "json_ld"
}
```

### Markdown Output

```rust
use riptide_schemas::formatters::OutputFormatter;

let markdown = event.to_markdown()?;
println!("{}", markdown);
```

Example output:
```markdown
# Rust Conference 2025

Annual Rust conference

## Event Details

- **Start**: January 15, 2025 at 10:00 AM UTC
- **Location**: Convention Center, San Francisco, USA (37.7749, -122.4194)
- **URL**: [https://rustconf.com](https://rustconf.com)
- **Organizer**: Rust Foundation <info@rustconf.com>

## Extraction Metadata

- **Confidence**: 95.0%
- **Strategy**: json_ld
- **Schema Version**: v1
```

## Schema Versioning

Forward-compatible schema evolution:

```rust
use riptide_schemas::{SchemaVersion, Event, SchemaAdapter, EventV2Adapter};

// V1 event
let v1_event = Event {
    schema_version: SchemaVersion::V1,
    // ... fields
    ..Default::default()
};

// Future: Convert to V2 when needed
// let v2_event = EventV2Adapter::from_v1(v1_event)?;

// Check version
match event.schema_version {
    SchemaVersion::V1 => println!("Using V1 schema"),
    SchemaVersion::V2 => println!("Using V2 schema"),
}
```

## Usage Examples

### Basic Event Creation

```rust
use riptide_schemas::{Event, SchemaVersion};
use chrono::Utc;

let event = Event {
    schema_version: SchemaVersion::V1,
    title: "Tech Meetup".to_string(),
    description: Some("Monthly tech meetup".to_string()),
    start_date: Utc::now(),
    end_date: None,
    location: None,
    organizer: None,
    url: "https://meetup.com/tech".to_string(),
    confidence: Some(0.8),
    extraction_strategy: Some("css".to_string()),
};
```

### With Builder Pattern

```rust
use riptide_schemas::Event;

let event = Event::builder()
    .title("Conference")
    .start_date(Utc::now())
    .url("https://conf.com")
    .location("Convention Center", "San Francisco", "USA")
    .organizer("Tech Corp", "info@techcorp.com")
    .confidence(0.95)
    .strategy("json_ld")
    .build()?;
```

### Serialization/Deserialization

```rust
use riptide_schemas::Event;

// Serialize to JSON
let json = serde_json::to_string(&event)?;

// Deserialize from JSON
let event: Event = serde_json::from_str(&json)?;

// Serialize to YAML (requires serde_yaml)
let yaml = serde_yaml::to_string(&event)?;
```

### Validation

```rust
use riptide_schemas::Event;

fn validate_event(event: &Event) -> Result<(), String> {
    // Check required fields
    if event.title.is_empty() {
        return Err("Title is required".to_string());
    }

    if event.url.is_empty() {
        return Err("URL is required".to_string());
    }

    // Check confidence range
    if let Some(confidence) = event.confidence {
        if !(0.0..=1.0).contains(&confidence) {
            return Err("Confidence must be between 0 and 1".to_string());
        }
    }

    Ok(())
}
```

## API Reference

### Core Types

- `Event` - Main event structure
- `Location` - Location information
- `Organizer` - Organizer details
- `SchemaVersion` - Version enum (V1, V2, etc.)

### Extraction Types

- `ExtractionStrategy` - Enum of all strategies
- `select_strategy(content, content_type)` - Auto-detect strategy

### Formatter Types

- `OutputFormatter` - Trait for formatters
- `EventFormatter` - Event-specific formatter

## Integration with Other Crates

### Used By

- **riptide-extraction**: Uses schemas for extracted data
- **riptide-spider**: Stores crawled events in schema format
- **riptide-api**: Returns events in schema format
- **riptide-workers**: Processes events with schema validation

### Example Integration

```rust
// In riptide-extraction
use riptide_schemas::{Event, ExtractionStrategy, select_strategy};

pub async fn extract_events(html: &str, content_type: &str) -> Vec<Event> {
    // Auto-detect strategy
    let strategy = select_strategy(html, content_type);

    // Extract based on strategy
    match strategy {
        ExtractionStrategy::JsonLd => extract_from_json_ld(html),
        ExtractionStrategy::ICS => extract_from_ics(html),
        ExtractionStrategy::CSS(selector) => extract_from_css(html, &selector),
        _ => Vec::new(),
    }
}
```

## Testing

```bash
# Run all tests
cargo test -p riptide-schemas

# Test serialization
cargo test -p riptide-schemas serialization

# Test extraction strategies
cargo test -p riptide-schemas extraction

# Test with coverage
cargo tarpaulin -p riptide-schemas
```

### Example Tests

```rust
use riptide_schemas::{Event, SchemaVersion};
use chrono::Utc;

#[test]
fn test_event_serialization() {
    let event = Event {
        schema_version: SchemaVersion::V1,
        title: "Test Event".to_string(),
        start_date: Utc::now(),
        url: "https://test.com".to_string(),
        ..Default::default()
    };

    // Serialize
    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains("Test Event"));

    // Deserialize
    let parsed: Event = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.title, "Test Event");
}

#[test]
fn test_strategy_detection() {
    use riptide_schemas::extraction::{ExtractionStrategy, select_strategy};

    let json_ld = r#"<script type="application/ld+json">..."#;
    let strategy = select_strategy(json_ld, "text/html");
    assert_eq!(strategy, ExtractionStrategy::JsonLd);

    let ics = "BEGIN:VCALENDAR";
    let strategy = select_strategy(ics, "text/calendar");
    assert_eq!(strategy, ExtractionStrategy::ICS);
}
```

## Best Practices

1. **Use Schema Versioning**: Always set `schema_version` for forward compatibility
2. **Validate Data**: Validate events before storing or transmitting
3. **Set Confidence**: Include confidence scores when available
4. **Document Strategy**: Record which extraction strategy was used
5. **Handle Optional Fields**: Gracefully handle missing location/organizer data
6. **Use Auto-Detection**: Let `select_strategy` choose the best approach

## Common Patterns

### Event Collection

```rust
use riptide_schemas::Event;

fn collect_events(sources: Vec<String>) -> Vec<Event> {
    sources
        .into_iter()
        .flat_map(|source| extract_from_source(&source))
        .collect()
}
```

### Event Filtering

```rust
use riptide_schemas::Event;

fn filter_high_confidence(events: Vec<Event>) -> Vec<Event> {
    events
        .into_iter()
        .filter(|e| e.confidence.unwrap_or(0.0) >= 0.8)
        .collect()
}
```

### Event Transformation

```rust
use riptide_schemas::Event;

fn enrich_event(mut event: Event, timezone: &str) -> Event {
    // Add timezone info
    if let Some(location) = &mut event.location {
        location.timezone = Some(timezone.to_string());
    }
    event
}
```

## Future Enhancements (v1.1+)

- **CSV Export**: Direct CSV serialization
- **iCalendar Export**: Convert events back to .ics format
- **YAML Support**: Full YAML serialization
- **Schema Validation**: JSON Schema validation
- **More Strategies**: Additional extraction strategies
- **V2 Schema**: Enhanced schema with new fields

## Dependencies

- `serde` - Serialization framework
- `serde_json` - JSON support
- `schemars` - JSON Schema generation
- `chrono` - Date/time handling
- `anyhow` - Error handling
- `thiserror` - Error derive macros
- `riptide-types` - Shared type definitions

## License

MIT OR Apache-2.0

## Related Crates

- `riptide-extraction` - Uses schemas for extraction
- `riptide-types` - Core type definitions
- `riptide-api` - API responses in schema format
- `riptide-workers` - Background job processing
