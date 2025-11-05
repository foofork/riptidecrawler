/*!
Integration tests for riptide-schemas crate
*/

use chrono::Utc;
use riptide_schemas::{
    extraction::{select_strategy, ExtractionStrategy},
    formatters::{EventCollection, EventFormatter, OutputFormatter},
    Event, EventV2Adapter, Location, Organizer, SchemaAdapter, SchemaVersion,
};

// Helper function to create a test event
fn create_test_event() -> Event {
    Event {
        schema_version: SchemaVersion::V1,
        title: "Test Event".to_string(),
        description: Some("A test event for integration testing".to_string()),
        start_date: Utc::now(),
        end_date: None,
        location: Some(Location {
            name: "Test Venue".to_string(),
            address: Some("123 Test St".to_string()),
            city: Some("Test City".to_string()),
            country: Some("Test Country".to_string()),
            lat_lon: Some((37.7749, -122.4194)),
        }),
        url: "https://test.com/event".to_string(),
        organizer: Some(Organizer {
            name: "Test Organizer".to_string(),
            email: Some("test@example.com".to_string()),
            url: Some("https://test.com".to_string()),
        }),
        confidence: Some(0.95),
        extraction_strategy: Some("json_ld".to_string()),
    }
}

#[test]
fn test_event_round_trip_json() {
    let event = create_test_event();
    let json = event.to_json().unwrap();
    let parsed: Event = serde_json::from_str(&json).unwrap();

    assert_eq!(event.title, parsed.title);
    assert_eq!(event.url, parsed.url);
    assert_eq!(event.schema_version, parsed.schema_version);
}

#[test]
fn test_event_markdown_generation() {
    let event = create_test_event();
    let markdown = event.to_markdown().unwrap();

    // Verify key sections are present
    assert!(markdown.contains("# Test Event"));
    assert!(markdown.contains("A test event for integration testing"));
    assert!(markdown.contains("## Event Details"));
    assert!(markdown.contains("Test Venue"));
    assert!(markdown.contains("Test City, Test Country"));
    assert!(markdown.contains("Test Organizer"));
    assert!(markdown.contains("95.0%"));
}

#[test]
fn test_event_collection_json() {
    let events = vec![create_test_event(), create_test_event()];
    let collection = EventCollection::new(events);

    let json = collection.to_json().unwrap();
    let parsed: Vec<Event> = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed.len(), 2);
}

#[test]
fn test_event_collection_markdown() {
    let events = vec![create_test_event(), create_test_event()];
    let collection = EventCollection::new(events);

    let markdown = collection.to_markdown().unwrap();

    assert!(markdown.contains("Events (2 total)"));
    assert!(markdown.contains("## 1. Test Event"));
    assert!(markdown.contains("## 2. Test Event"));
}

#[test]
fn test_extraction_strategy_auto_select_icalendar() {
    let content = "BEGIN:VCALENDAR\nVERSION:2.0\nBEGIN:VEVENT\nEND:VEVENT\nEND:VCALENDAR";
    let strategy = select_strategy(content, "text/calendar");

    assert_eq!(strategy, ExtractionStrategy::ICS);
}

#[test]
fn test_extraction_strategy_auto_select_jsonld() {
    let content = r#"
        <html>
        <head>
            <script type="application/ld+json">
            {
                "@context": "https://schema.org",
                "@type": "Event",
                "name": "Test Event"
            }
            </script>
        </head>
        </html>
    "#;
    let strategy = select_strategy(content, "text/html");

    assert_eq!(strategy, ExtractionStrategy::JsonLd);
}

#[test]
fn test_extraction_strategy_auto_select_microformat() {
    let content = r#"<div class="h-event"><span class="p-name">Event</span></div>"#;
    let strategy = select_strategy(content, "text/html");

    match strategy {
        ExtractionStrategy::CSS(selector) => {
            assert!(selector.contains("h-event"));
        }
        _ => panic!("Expected CSS strategy"),
    }
}

#[test]
fn test_extraction_strategy_serialization() {
    let strategies = vec![
        ExtractionStrategy::ICS,
        ExtractionStrategy::JsonLd,
        ExtractionStrategy::CSS(".selector".to_string()),
        ExtractionStrategy::Regex(r"\d+".to_string()),
        ExtractionStrategy::Browser,
    ];

    for strategy in strategies {
        let json = serde_json::to_string(&strategy).unwrap();
        let parsed: ExtractionStrategy = serde_json::from_str(&json).unwrap();
        assert_eq!(strategy, parsed);
    }
}

#[test]
fn test_schema_versioning() {
    let v1_event = create_test_event();
    assert_eq!(v1_event.schema_version, SchemaVersion::V1);

    // Test adapter (identity for v1.0)
    let converted = EventV2Adapter::from_v1(v1_event.clone()).unwrap();
    assert_eq!(v1_event.title, converted.title);

    let back = EventV2Adapter::to_v1(&converted);
    assert_eq!(v1_event.title, back.title);
}

#[test]
fn test_event_minimal() {
    let event = Event {
        schema_version: SchemaVersion::V1,
        title: "Minimal Event".to_string(),
        description: None,
        start_date: Utc::now(),
        end_date: None,
        location: None,
        url: "https://example.com".to_string(),
        organizer: None,
        confidence: None,
        extraction_strategy: None,
    };

    let json = event.to_json().unwrap();
    assert!(json.contains("Minimal Event"));

    let markdown = event.to_markdown().unwrap();
    assert!(markdown.contains("# Minimal Event"));
}

#[test]
fn test_location_full_address() {
    let location = Location {
        name: "Full Venue".to_string(),
        address: Some("456 Main St".to_string()),
        city: Some("San Francisco".to_string()),
        country: Some("USA".to_string()),
        lat_lon: Some((37.7749, -122.4194)),
    };

    let event = Event {
        location: Some(location),
        ..create_test_event()
    };

    let markdown = event.to_markdown().unwrap();
    assert!(markdown.contains("Full Venue"));
    assert!(markdown.contains("456 Main St"));
    assert!(markdown.contains("San Francisco, USA"));
    assert!(markdown.contains("37.7749"));
}

#[test]
fn test_organizer_full_details() {
    let organizer = Organizer {
        name: "Rust Foundation".to_string(),
        email: Some("info@rust-lang.org".to_string()),
        url: Some("https://foundation.rust-lang.org".to_string()),
    };

    let event = Event {
        organizer: Some(organizer),
        ..create_test_event()
    };

    let markdown = event.to_markdown().unwrap();
    assert!(markdown.contains("Rust Foundation"));
    assert!(markdown.contains("<info@rust-lang.org>"));
    assert!(markdown.contains("[Website]"));
}

#[test]
fn test_extraction_strategy_display() {
    assert_eq!(ExtractionStrategy::ICS.to_string(), "ics");
    assert_eq!(ExtractionStrategy::JsonLd.to_string(), "json_ld");
    assert_eq!(
        ExtractionStrategy::CSS(".test".to_string()).to_string(),
        "css(.test)"
    );
    assert_eq!(ExtractionStrategy::Browser.to_string(), "browser");
    assert_eq!(
        ExtractionStrategy::LLM("openai".to_string()).to_string(),
        "llm(openai)"
    );
}

#[test]
fn test_event_default() {
    let event = Event::default();
    assert_eq!(event.schema_version, SchemaVersion::V1);
    assert!(event.title.is_empty());
    assert!(event.url.is_empty());
}

#[test]
fn test_schema_version_default() {
    let version = SchemaVersion::default();
    assert_eq!(version, SchemaVersion::V1);
    assert_eq!(version.to_string(), "v1");
}
