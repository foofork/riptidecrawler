//! Integration tests for riptide-streaming
//!
//! Tests for streaming coordinator, progress tracking, and backpressure control

use riptide_streaming::*;
use uuid::Uuid;

#[tokio::test]
async fn test_complete_streaming_workflow() {
    let mut coordinator = StreamingCoordinator::new();

    // Start a stream
    let extraction_id = "test-extraction-001".to_string();
    let stream_id = coordinator
        .start_stream(extraction_id.clone())
        .await
        .unwrap();

    // Verify stream is active
    let stream_info = coordinator.get_stream(&stream_id).unwrap();
    assert_eq!(stream_info.extraction_id, extraction_id);
    assert!(matches!(stream_info.status, StreamStatus::Active));

    // Update progress multiple times
    for i in 1..=10 {
        coordinator
            .update_progress(stream_id, i * 10, Some(100))
            .await
            .unwrap();
    }

    // Verify final progress
    let stream_info = coordinator.get_stream(&stream_id).unwrap();
    assert_eq!(stream_info.processed_items, 100);
    assert_eq!(stream_info.total_items, Some(100));

    // Complete the stream
    coordinator.complete_stream(stream_id).await.unwrap();

    // Verify completion
    let stream_info = coordinator.get_stream(&stream_id).unwrap();
    assert!(matches!(stream_info.status, StreamStatus::Completed));
}

#[tokio::test]
async fn test_progress_tracker_integration() {
    let tracker = ProgressTracker::new();
    let stream_id = Uuid::new_v4();

    // Start tracking - event receiver not monitored in integration test
    let _rx = tracker.start_tracking(stream_id).await.unwrap();
    // Update progress
    tracker
        .update_progress(stream_id, 50, Some(100))
        .await
        .unwrap();

    // Get progress info
    let progress = tracker.get_progress(&stream_id).await.unwrap();
    assert_eq!(progress.processed_items, 50);
    assert_eq!(progress.total_items, Some(100));

    // Complete tracking
    tracker.complete_tracking(stream_id).await.unwrap();

    let progress = tracker.get_progress(&stream_id).await.unwrap();
    assert!(matches!(progress.stage, ProgressStage::Completed));
}

#[tokio::test]
async fn test_backpressure_controller_integration() {
    let config = BackpressureConfig {
        max_in_flight: 5,
        max_total_items: 10,
        max_memory_bytes: 1024 * 1024, // 1MB
        ..Default::default()
    };

    let controller = BackpressureController::new(config);
    let stream_id = Uuid::new_v4();

    // Register stream
    controller.register_stream(stream_id).await.unwrap();

    // Acquire multiple permits
    let mut permits = Vec::new();
    for _ in 0..5 {
        let permit = controller.acquire(stream_id, 1024).await.unwrap();
        permits.push(permit);
    }

    // Verify metrics
    let metrics = controller.get_metrics().await;
    assert_eq!(metrics.total_in_flight, 5);
    assert_eq!(metrics.total_streams, 1);

    // Try to exceed limit
    let result = controller.acquire(stream_id, 1024).await;
    assert!(result.is_err());

    // Drop permits to free resources
    drop(permits);

    // Wait for cleanup
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    // Should be able to acquire again - permit held as RAII guard until end of scope
    let _permit = controller.acquire(stream_id, 1024).await.unwrap();
}

#[tokio::test]
async fn test_concurrent_streams() {
    let mut coordinator = StreamingCoordinator::new();

    // Start multiple streams concurrently
    let mut handles = vec![];

    for i in 0..5 {
        let extraction_id = format!("extraction-{}", i);
        let stream_id = coordinator.start_stream(extraction_id).await.unwrap();

        handles.push(tokio::spawn(async move { stream_id }));
    }

    // Collect stream IDs
    let mut stream_ids = vec![];
    for handle in handles {
        stream_ids.push(handle.await.unwrap());
    }

    assert_eq!(stream_ids.len(), 5);
    assert_eq!(coordinator.streams.len(), 5);

    // Complete all streams
    for stream_id in stream_ids {
        coordinator.complete_stream(stream_id).await.unwrap();
    }
}

#[tokio::test]
async fn test_error_handling() {
    let coordinator = StreamingCoordinator::new();

    // Try to get non-existent stream
    let result = coordinator.get_stream(&Uuid::new_v4());
    assert!(result.is_none());

    // Test backpressure exceeded error
    let config = BackpressureConfig {
        max_in_flight: 1,
        max_total_items: 1,
        ..Default::default()
    };
    let controller = BackpressureController::new(config);
    let stream_id = Uuid::new_v4();

    controller.register_stream(stream_id).await.unwrap();
    // RAII guard - must hold permit to maintain backpressure state during test
    let _permit = controller.acquire(stream_id, 1024).await.unwrap();
    let result = controller.acquire(stream_id, 1024).await;
    assert!(matches!(
        result.unwrap_err(),
        StreamingError::BackpressureExceeded
    ));
}

#[tokio::test]
async fn test_progress_stages() {
    let tracker = ProgressTracker::new();
    let stream_id = Uuid::new_v4();
    // Event receiver not monitored - test validates stage progression
    let _rx = tracker.start_tracking(stream_id).await.unwrap();
    // Test different stages
    tracker
        .set_stage(stream_id, ProgressStage::Discovering)
        .await
        .unwrap();
    let progress = tracker.get_progress(&stream_id).await.unwrap();
    assert!(matches!(progress.stage, ProgressStage::Discovering));

    tracker
        .set_stage(stream_id, ProgressStage::Extracting)
        .await
        .unwrap();
    let progress = tracker.get_progress(&stream_id).await.unwrap();
    assert!(matches!(progress.stage, ProgressStage::Extracting));

    tracker
        .set_stage(stream_id, ProgressStage::Processing)
        .await
        .unwrap();
    let progress = tracker.get_progress(&stream_id).await.unwrap();
    assert!(matches!(progress.stage, ProgressStage::Processing));

    tracker.complete_tracking(stream_id).await.unwrap();
    let progress = tracker.get_progress(&stream_id).await.unwrap();
    assert!(matches!(progress.stage, ProgressStage::Completed));
}
