# Riptide Schemas

Event and data schemas for the Riptide web scraping framework.

## Features

- **Event Schema**: Structured event data with versioning support
- **Extraction Strategies**: 8 modular extraction strategies for different content types
- **Output Formatters**: Universal format conversion (JSON, Markdown)
- **Schema Validation**: Automatic validation with JSON Schema support
- **Forward Compatibility**: Schema versioning for easy evolution

## Installation

```toml
[dependencies]
riptide-schemas = "1.0.0"
```

## Quick Start

### Creating an Event

```rust
use riptide_schemas::{Event, SchemaVersion, Location, Organizer};
use chrono::Utc;

let event = Event {
    schema_version: SchemaVersion::V1,
    title: "Rust Conference 2025".to_string(),
    description: Some("Annual Rust community conference".to_string()),
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
```

### Output Formatting

```rust
use riptide_schemas::formatters::OutputFormatter;

// Convert to JSON
let json = event.to_json()?;
println!("{}", json);

// Convert to Markdown
let markdown = event.to_markdown()?;
println!("{}", markdown);
```

### Extraction Strategies

```rust
use riptide_schemas::extraction::{ExtractionStrategy, select_strategy};

// Manual strategy selection
let strategy = ExtractionStrategy::JsonLd;
let css_strategy = ExtractionStrategy::CSS(".event-title".to_string());
let llm_strategy = ExtractionStrategy::LLM("openai".to_string());

// Auto-selection based on content
let html = r#"<script type="application/ld+json">{"@type": "Event"}</script>"#;
let auto_strategy = select_strategy(html, "text/html");
// Returns: ExtractionStrategy::JsonLd
```

## Extraction Strategies

Riptide supports 8 extraction strategies:

| Strategy | Use Case | Example |
|----------|----------|---------|
| **ICS** | iCalendar files | `BEGIN:VCALENDAR` |
| **JsonLd** | JSON-LD structured data | `<script type="application/ld+json">` |
| **CSS** | CSS selectors | `.event-title, .event-date` |
| **Regex** | Pattern matching | `\d{4}-\d{2}-\d{2}` |
| **Rules** | Custom extraction rules | Rule-based logic |
| **LLM** | AI-powered extraction | OpenAI (v1.0) |
| **Browser** | JavaScript-rendered content | Headless browser |
| **WASM** | Custom extractors | WebAssembly modules |

## Schema Versioning

Events include schema versioning for forward compatibility:

```rust
use riptide_schemas::{Event, SchemaVersion, SchemaAdapter, EventV2Adapter};

let v1_event = Event {
    schema_version: SchemaVersion::V1,
    // ... fields
    ..Default::default()
};

// Future: Convert between schema versions
let v2_event = EventV2Adapter::from_v1(v1_event)?;
```

## Output Formats

### v1.0 Supported

- **JSON**: Machine-readable structured data
- **Markdown**: Human-readable documentation

### v1.1 (Future)

- **CSV**: Spreadsheet export
- **iCalendar**: Calendar integration
- **YAML**: Configuration format

## Event Schema

```rust
pub struct Event {
    // Schema versioning
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

## Location Schema

```rust
pub struct Location {
    pub name: String,
    pub address: Option<String>,
    pub city: Option<String>,
    pub country: Option<String>,
    pub lat_lon: Option<(f64, f64)>,
}
```

## Organizer Schema

```rust
pub struct Organizer {
    pub name: String,
    pub email: Option<String>,
    pub url: Option<String>,
}
```

## Examples

### JSON Output

```json
{
  "schema_version": "v1",
  "title": "Rust Conference 2025",
  "description": "Annual Rust community conference",
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

```markdown
# Rust Conference 2025

Annual Rust community conference

## Event Details

- **Start**: January 15, 2025 at 10:00 AM UTC
- **Location**: Convention Center, San Francisco, USA (37.7749, -122.4194)
- **URL**: [https://rustconf.com](https://rustconf.com)
- **Organizer**: Rust Foundation <info@rustconf.com> [Website](https://foundation.rust-lang.org)

## Extraction Metadata

- **Confidence**: 95.0%
- **Strategy**: json_ld
- **Schema Version**: v1
```

## Testing

```bash
# Run tests
cargo test -p riptide-schemas

# Run with coverage
cargo tarpaulin -p riptide-schemas
```

## Documentation

```bash
# Generate documentation
cargo doc --package riptide-schemas --open
```

## License

MIT OR Apache-2.0

## Contributing

See the main Riptide repository for contribution guidelines.
