//! Factory function tests
//!
//! Tests for browser engine creation and factory error handling

use riptide_browser::abstraction::{create_engine, EngineType};

#[tokio::test]
async fn test_create_chromiumoxide_engine_returns_error() {
    // Factory requires actual Browser instance, should return error
    let result = create_engine(EngineType::Chromiumoxide).await;

    assert!(result.is_err());
    if let Err(err) = result {
        assert!(err
            .to_string()
            .contains("Factory function requires Browser instance parameter"));
    }
}

#[tokio::test]
async fn test_create_spider_chrome_engine_returns_error() {
    // Factory requires actual Browser instance, should return error
    let result = create_engine(EngineType::SpiderChrome).await;

    assert!(result.is_err());
    if let Err(err) = result {
        assert!(err
            .to_string()
            .contains("Factory function requires Browser instance parameter"));
    }
}

#[test]
fn test_engine_type_chromiumoxide() {
    let engine_type = EngineType::Chromiumoxide;
    assert_eq!(format!("{:?}", engine_type), "Chromiumoxide");
}

#[test]
fn test_engine_type_spider_chrome() {
    let engine_type = EngineType::SpiderChrome;
    assert_eq!(format!("{:?}", engine_type), "SpiderChrome");
}

#[test]
fn test_engine_type_equality() {
    let engine1 = EngineType::Chromiumoxide;
    let engine2 = EngineType::Chromiumoxide;
    let engine3 = EngineType::SpiderChrome;

    assert_eq!(engine1, engine2);
    assert_ne!(engine1, engine3);
}

#[test]
fn test_engine_type_clone() {
    let original = EngineType::Chromiumoxide;
    let cloned = original;

    assert_eq!(original, cloned);
}

#[test]
fn test_engine_type_copy() {
    let original = EngineType::SpiderChrome;
    let copied = original;

    // Both should still be usable (Copy trait)
    assert_eq!(original, copied);
    assert_eq!(format!("{:?}", original), "SpiderChrome");
}

#[test]
fn test_engine_type_serialization() {
    let engine = EngineType::Chromiumoxide;
    let serialized = serde_json::to_string(&engine).unwrap();

    assert!(serialized.contains("Chromiumoxide") || serialized.contains("chromiumoxide"));
}

#[test]
fn test_engine_type_deserialization() {
    let json = r#""Chromiumoxide""#;
    let engine: EngineType = serde_json::from_str(json).unwrap();

    assert_eq!(engine, EngineType::Chromiumoxide);
}

#[test]
fn test_spider_chrome_serialization() {
    let engine = EngineType::SpiderChrome;
    let serialized = serde_json::to_string(&engine).unwrap();

    assert!(serialized.contains("SpiderChrome") || serialized.contains("spiderchrome"));
}

#[test]
fn test_spider_chrome_deserialization() {
    let json = r#""SpiderChrome""#;
    let engine: EngineType = serde_json::from_str(json).unwrap();

    assert_eq!(engine, EngineType::SpiderChrome);
}

#[test]
fn test_engine_type_round_trip() {
    let original = EngineType::Chromiumoxide;
    let serialized = serde_json::to_string(&original).unwrap();
    let deserialized: EngineType = serde_json::from_str(&serialized).unwrap();

    assert_eq!(original, deserialized);
}

#[test]
fn test_all_engine_types() {
    let types = [EngineType::Chromiumoxide, EngineType::SpiderChrome];

    assert_eq!(types.len(), 2);
    assert!(types.contains(&EngineType::Chromiumoxide));
    assert!(types.contains(&EngineType::SpiderChrome));
}
