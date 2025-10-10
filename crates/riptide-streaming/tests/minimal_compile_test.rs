//! Minimal test to verify compilation of riptide-streaming test dependencies

use futures::stream::StreamExt;
use httpmock::prelude::*;
use serde_json::Value;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio_test;
use tracing_test::traced_test;

#[tokio::test]
async fn test_dependencies_compile() {
    // Test that all test dependencies are available and compile
    let _server = MockServer::start();
    let _map: HashMap<String, String> = HashMap::new();
    let _value: Value = serde_json::json!({"test": "value"});
    let _dur = Duration::from_millis(100);
    let _instant = Instant::now();

    // Test StreamExt is available
    let stream = futures::stream::iter(vec![1, 2, 3]);
    let collected: Vec<i32> = stream.collect().await;
    assert_eq!(collected, vec![1, 2, 3]);

    println!("All test dependencies compile successfully!");
}

#[traced_test]
#[tokio::test]
async fn test_tracing_test_works() {
    tracing::info!("Test tracing works");
    assert!(true);
}
