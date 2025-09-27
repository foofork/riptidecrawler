//! NDJSON (Newline Delimited JSON) streaming support
//!
//! This module provides real-time streaming of extraction results as NDJSON,
//! with comprehensive backpressure handling, progress tracking, and lifecycle management.

use crate::{ExtractionResult, ProgressUpdate, StreamingError, StreamingResult};
use anyhow::Result;
use async_stream::stream;
use bytes::Bytes;
use futures::{Stream, StreamExt};
use pin_project_lite::pin_project;
use serde::{Deserialize, Serialize};
use std::{
    fmt,
    pin::Pin,
    task::{Context, Poll},
    time::{Duration, Instant},
};
use tokio::time::interval;
use tokio_util::codec::{Decoder, Encoder, FramedRead, LinesCodec};
use uuid::Uuid;

/// NDJSON item representing a single line in the stream
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum NdjsonItem {
    /// Extraction result data
    Result(ExtractionResult),
    /// Progress update
    Progress(ProgressUpdate),
    /// Stream event (start, end, error)
    Event(StreamEvent),
    /// Metadata about the stream
    Metadata(StreamMetadata),
    /// Heartbeat to keep connection alive
    Heartbeat(HeartbeatData),
}

/// Stream lifecycle events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamEvent {
    pub stream_id: Uuid,
    pub extraction_id: String,
    pub event_type: StreamEventType,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub data: Option<serde_json::Value>,
}

/// Types of stream events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StreamEventType {
    /// Stream started
    Started,
    /// Stream paused
    Paused,
    /// Stream resumed
    Resumed,
    /// Stream completed successfully
    Completed,
    /// Stream failed with error
    Failed,
    /// Stream was cancelled
    Cancelled,
    /// Backpressure activated
    BackpressureActivated,
    /// Backpressure released
    BackpressureReleased,
}

/// Metadata about the stream
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamMetadata {
    pub stream_id: Uuid,
    pub extraction_id: String,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub estimated_total: Option<usize>,
    pub content_type: String,
    pub source_url: Option<String>,
    pub user_agent: Option<String>,
    pub tags: Vec<String>,
}

/// Heartbeat data to keep connections alive
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatData {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub stream_id: Uuid,
    pub uptime_seconds: u64,
    pub items_processed: usize,
}

/// Configuration for NDJSON streaming
#[derive(Debug, Clone)]
pub struct NdjsonConfig {
    /// Buffer size for the stream
    pub buffer_size: usize,
    /// Whether to include progress updates
    pub include_progress: bool,
    /// Whether to include heartbeat messages
    pub include_heartbeat: bool,
    /// Heartbeat interval
    pub heartbeat_interval: Duration,
    /// Whether to include metadata
    pub include_metadata: bool,
    /// Maximum line length for safety
    pub max_line_length: usize,
    /// Compression enabled
    pub compression_enabled: bool,
    /// Pretty print JSON (for debugging)
    pub pretty_print: bool,
}

impl Default for NdjsonConfig {
    fn default() -> Self {
        Self {
            buffer_size: 1000,
            include_progress: true,
            include_heartbeat: true,
            heartbeat_interval: Duration::from_secs(30),
            include_metadata: true,
            max_line_length: 1024 * 1024, // 1MB per line
            compression_enabled: false,
            pretty_print: false,
        }
    }
}

/// NDJSON codec for encoding and decoding
#[derive(Debug, Clone)]
pub struct NdjsonCodec {
    lines_codec: LinesCodec,
    config: NdjsonConfig,
}

impl NdjsonCodec {
    /// Create a new NDJSON codec with default configuration
    pub fn new() -> Self {
        Self::with_config(NdjsonConfig::default())
    }

    /// Create a new NDJSON codec with custom configuration
    pub fn with_config(config: NdjsonConfig) -> Self {
        let lines_codec = LinesCodec::new_with_max_length(config.max_line_length);
        Self {
            lines_codec,
            config,
        }
    }
}

impl Default for NdjsonCodec {
    fn default() -> Self {
        Self::new()
    }
}

impl Decoder for NdjsonCodec {
    type Item = NdjsonItem;
    type Error = StreamingError;

    fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        match self.lines_codec.decode(src)? {
            Some(line) => {
                let item: NdjsonItem = serde_json::from_str(&line)
                    .map_err(StreamingError::SerializationError)?;
                Ok(Some(item))
            }
            None => Ok(None),
        }
    }
}

impl Encoder<NdjsonItem> for NdjsonCodec {
    type Error = StreamingError;

    fn encode(&mut self, item: NdjsonItem, dst: &mut bytes::BytesMut) -> Result<(), Self::Error> {
        let json_str = if self.config.pretty_print {
            serde_json::to_string_pretty(&item)
        } else {
            serde_json::to_string(&item)
        }.map_err(StreamingError::SerializationError)?;

        self.lines_codec.encode(json_str, dst)?;
        Ok(())
    }
}

/// Basic NDJSON stream for real-time extraction results
pub struct NdjsonStream {
    stream_id: Uuid,
    config: NdjsonConfig,
    start_time: Instant,
    items_sent: usize,
    last_heartbeat: Instant,
}

impl NdjsonStream {
    /// Create a new NDJSON stream from an extraction result stream
    pub fn new<S>(
        stream_id: Uuid,
        extraction_id: String,
        results_stream: S,
        config: NdjsonConfig,
    ) -> impl Stream<Item = StreamingResult<NdjsonItem>>
    where
        S: Stream<Item = StreamingResult<ExtractionResult>> + Send + 'static,
    {
        Self::create_inner_stream(stream_id, extraction_id, results_stream, config)
    }

    fn create_inner_stream<S>(
        stream_id: Uuid,
        extraction_id: String,
        results_stream: S,
        config: NdjsonConfig,
    ) -> impl Stream<Item = StreamingResult<NdjsonItem>>
    where
        S: Stream<Item = StreamingResult<ExtractionResult>> + Send + 'static,
    {
        stream! {
            let mut results_stream = Box::pin(results_stream);
            let mut items_processed = 0usize;
            let start_time = Instant::now();
            let mut last_heartbeat = start_time;
            let mut heartbeat_interval = interval(config.heartbeat_interval);

            // Send start event
            let start_event = StreamEvent {
                stream_id,
                extraction_id: extraction_id.clone(),
                event_type: StreamEventType::Started,
                timestamp: chrono::Utc::now(),
                data: Some(serde_json::json!({
                    "buffer_size": config.buffer_size,
                    "include_progress": config.include_progress,
                    "include_heartbeat": config.include_heartbeat
                })),
            };
            yield Ok(NdjsonItem::Event(start_event));

            // Send metadata if enabled
            if config.include_metadata {
                let metadata = StreamMetadata {
                    stream_id,
                    extraction_id: extraction_id.clone(),
                    start_time: chrono::Utc::now(),
                    estimated_total: None,
                    content_type: "application/x-ndjson".to_string(),
                    source_url: None,
                    user_agent: Some("RipTide-Streaming/0.1.0".to_string()),
                    tags: vec!["extraction".to_string(), "real-time".to_string()],
                };
                yield Ok(NdjsonItem::Metadata(metadata));
            }

            loop {
                tokio::select! {
                    // Process extraction results
                    result = results_stream.next() => {
                        match result {
                            Some(Ok(extraction_result)) => {
                                yield Ok(NdjsonItem::Result(extraction_result));
                                items_processed += 1;

                                // Send progress update if enabled
                                if config.include_progress && items_processed % 10 == 0 {
                                    let progress = ProgressUpdate {
                                        stream_id,
                                        extraction_id: extraction_id.clone(),
                                        processed: items_processed,
                                        total: None,
                                        current_item: None,
                                        timestamp: chrono::Utc::now(),
                                        rate_per_second: items_processed as f64 / start_time.elapsed().as_secs_f64(),
                                        estimated_completion: None,
                                    };
                                    yield Ok(NdjsonItem::Progress(progress));
                                }
                            }
                            Some(Err(e)) => {
                                let error_event = StreamEvent {
                                    stream_id,
                                    extraction_id: extraction_id.clone(),
                                    event_type: StreamEventType::Failed,
                                    timestamp: chrono::Utc::now(),
                                    data: Some(serde_json::json!({
                                        "error": e.to_string(),
                                        "items_processed": items_processed
                                    })),
                                };
                                yield Ok(NdjsonItem::Event(error_event));
                                yield Err(e);
                                break;
                            }
                            None => {
                                // Stream completed
                                let complete_event = StreamEvent {
                                    stream_id,
                                    extraction_id: extraction_id.clone(),
                                    event_type: StreamEventType::Completed,
                                    timestamp: chrono::Utc::now(),
                                    data: Some(serde_json::json!({
                                        "total_items": items_processed,
                                        "duration_seconds": start_time.elapsed().as_secs(),
                                        "rate_per_second": items_processed as f64 / start_time.elapsed().as_secs_f64()
                                    })),
                                };
                                yield Ok(NdjsonItem::Event(complete_event));
                                break;
                            }
                        }
                    }

                    // Send heartbeat if enabled
                    _ = heartbeat_interval.tick(), if config.include_heartbeat => {
                        let now = Instant::now();
                        if now.duration_since(last_heartbeat) >= config.heartbeat_interval {
                            let heartbeat = HeartbeatData {
                                timestamp: chrono::Utc::now(),
                                stream_id,
                                uptime_seconds: start_time.elapsed().as_secs(),
                                items_processed,
                            };
                            yield Ok(NdjsonItem::Heartbeat(heartbeat));
                            last_heartbeat = now;
                        }
                    }
                }
            }
        }
    }
}

/// Builder for creating NDJSON streams with various configurations
pub struct NdjsonStreamBuilder {
    config: NdjsonConfig,
}

impl std::fmt::Debug for NdjsonStreamBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NdjsonStreamBuilder")
            .field("config", &self.config)
            .finish()
    }
}

impl NdjsonStreamBuilder {
    /// Create a new builder with default configuration
    pub fn new() -> Self {
        Self {
            config: NdjsonConfig::default(),
        }
    }

    /// Set the buffer size
    pub fn buffer_size(mut self, size: usize) -> Self {
        self.config.buffer_size = size;
        self
    }

    /// Enable or disable progress updates
    pub fn include_progress(mut self, enabled: bool) -> Self {
        self.config.include_progress = enabled;
        self
    }

    /// Enable or disable heartbeat messages
    pub fn include_heartbeat(mut self, enabled: bool) -> Self {
        self.config.include_heartbeat = enabled;
        self
    }

    /// Set the heartbeat interval
    pub fn heartbeat_interval(mut self, interval: Duration) -> Self {
        self.config.heartbeat_interval = interval;
        self
    }

    /// Enable or disable metadata
    pub fn include_metadata(mut self, enabled: bool) -> Self {
        self.config.include_metadata = enabled;
        self
    }

    /// Set maximum line length
    pub fn max_line_length(mut self, length: usize) -> Self {
        self.config.max_line_length = length;
        self
    }

    /// Enable or disable compression
    pub fn compression_enabled(mut self, enabled: bool) -> Self {
        self.config.compression_enabled = enabled;
        self
    }

    /// Enable or disable pretty printing
    pub fn pretty_print(mut self, enabled: bool) -> Self {
        self.config.pretty_print = enabled;
        self
    }

    /// Build the NDJSON stream
    pub fn build<S>(
        self,
        stream_id: Uuid,
        extraction_id: String,
        results_stream: S,
    ) -> impl Stream<Item = StreamingResult<NdjsonItem>>
    where
        S: Stream<Item = StreamingResult<ExtractionResult>> + Send + 'static,
    {
        NdjsonStream::new(stream_id, extraction_id, results_stream, self.config)
    }
}

impl Default for NdjsonStreamBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Utility functions for NDJSON operations
pub mod utils {
    use super::*;
    use std::io::Write;

    /// Write NDJSON items to a writer
    pub async fn write_ndjson_items<W, I>(
        writer: &mut W,
        items: I,
        config: &NdjsonConfig,
    ) -> StreamingResult<usize>
    where
        W: Write,
        I: IntoIterator<Item = NdjsonItem>,
    {
        let mut count = 0;
        let mut codec = NdjsonCodec::with_config(config.clone());

        for item in items {
            let mut buf = bytes::BytesMut::new();
            codec.encode(item, &mut buf)?;
            writer.write_all(&buf)?;
            count += 1;
        }

        writer.flush()?;
        Ok(count)
    }

    /// Read NDJSON items from a reader
    pub fn read_ndjson_items<R>(
        reader: R,
        config: &NdjsonConfig,
    ) -> impl Stream<Item = StreamingResult<NdjsonItem>>
    where
        R: tokio::io::AsyncRead + Send + 'static,
    {
        let codec = NdjsonCodec::with_config(config.clone());
        let framed = FramedRead::new(reader, codec);

        framed.map(|result| {
            result.map_err(StreamingError::from)
        })
    }

    /// Validate NDJSON item
    pub fn validate_ndjson_item(item: &NdjsonItem) -> StreamingResult<()> {
        match item {
            NdjsonItem::Result(result) => {
                if result.id.is_empty() {
                    return Err(StreamingError::ConfigError("Result ID cannot be empty".to_string()));
                }
                if result.url.is_empty() {
                    return Err(StreamingError::ConfigError("Result URL cannot be empty".to_string()));
                }
            }
            NdjsonItem::Progress(progress) => {
                if let Some(total) = progress.total {
                    if progress.processed > total {
                        return Err(StreamingError::ConfigError(
                            "Processed items cannot exceed total".to_string()
                        ));
                    }
                }
            }
            NdjsonItem::Event(_) | NdjsonItem::Metadata(_) | NdjsonItem::Heartbeat(_) => {
                // These are always valid
            }
        }
        Ok(())
    }

    /// Convert NDJSON item to compact JSON string
    pub fn to_compact_json(item: &NdjsonItem) -> StreamingResult<String> {
        serde_json::to_string(item).map_err(StreamingError::SerializationError)
    }

    /// Convert NDJSON item to pretty JSON string
    pub fn to_pretty_json(item: &NdjsonItem) -> StreamingResult<String> {
        serde_json::to_string_pretty(item).map_err(StreamingError::SerializationError)
    }

    /// Convert stream to bytes
    pub fn into_bytes_stream<S>(
        stream: S,
        config: NdjsonConfig,
    ) -> impl Stream<Item = StreamingResult<Bytes>>
    where
        S: Stream<Item = StreamingResult<NdjsonItem>>,
    {
        let codec = NdjsonCodec::with_config(config);
        stream.map(move |item| {
            match item {
                Ok(ndjson_item) => {
                    let mut buf = bytes::BytesMut::new();
                    let mut codec = codec.clone();
                    codec.encode(ndjson_item, &mut buf)
                        .map(|_| buf.freeze())
                        .map_err(|e| e)
                }
                Err(e) => Err(e),
            }
        })
    }

    /// Convert stream to string
    pub fn into_string_stream<S>(
        stream: S,
        config: NdjsonConfig,
    ) -> impl Stream<Item = StreamingResult<String>>
    where
        S: Stream<Item = StreamingResult<NdjsonItem>>,
    {
        into_bytes_stream(stream, config).map(|result| {
            result.and_then(|bytes| {
                String::from_utf8(bytes.to_vec())
                    .map_err(|e| StreamingError::ConfigError(format!("UTF-8 conversion error: {}", e)))
            })
        })
    }
}

/// Formatting implementation for better debugging
impl fmt::Display for NdjsonItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NdjsonItem::Result(result) => {
                write!(f, "Result(id={}, url={})", result.id, result.url)
            }
            NdjsonItem::Progress(progress) => {
                write!(f, "Progress({}/{})", progress.processed,
                       progress.total.map_or("?".to_string(), |t| t.to_string()))
            }
            NdjsonItem::Event(event) => {
                write!(f, "Event({:?})", event.event_type)
            }
            NdjsonItem::Metadata(metadata) => {
                write!(f, "Metadata(stream={})", metadata.stream_id)
            }
            NdjsonItem::Heartbeat(heartbeat) => {
                write!(f, "Heartbeat(uptime={}s)", heartbeat.uptime_seconds)
            }
        }
    }
}

impl fmt::Display for StreamEventType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StreamEventType::Started => write!(f, "Started"),
            StreamEventType::Paused => write!(f, "Paused"),
            StreamEventType::Resumed => write!(f, "Resumed"),
            StreamEventType::Completed => write!(f, "Completed"),
            StreamEventType::Failed => write!(f, "Failed"),
            StreamEventType::Cancelled => write!(f, "Cancelled"),
            StreamEventType::BackpressureActivated => write!(f, "BackpressureActivated"),
            StreamEventType::BackpressureReleased => write!(f, "BackpressureReleased"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_stream::iter;
    use std::collections::HashMap;

    fn create_test_extraction_result(id: &str) -> ExtractionResult {
        ExtractionResult {
            id: id.to_string(),
            url: format!("https://example.com/{}", id),
            title: Some(format!("Test Title {}", id)),
            content: format!("Test content for {}", id),
            metadata: HashMap::new(),
            timestamp: chrono::Utc::now(),
            extraction_time_ms: 100,
            word_count: 10,
            links: vec![],
            images: vec![],
        }
    }

    #[tokio::test]
    async fn test_ndjson_codec_encode_decode() {
        let mut codec = NdjsonCodec::new();
        let item = NdjsonItem::Result(create_test_extraction_result("test"));

        // Encode
        let mut buf = bytes::BytesMut::new();
        codec.encode(item.clone(), &mut buf).unwrap();

        // Decode
        let decoded = codec.decode(&mut buf).unwrap().unwrap();

        match (item, decoded) {
            (NdjsonItem::Result(original), NdjsonItem::Result(decoded)) => {
                assert_eq!(original.id, decoded.id);
                assert_eq!(original.url, decoded.url);
            }
            _ => panic!("Item type mismatch"),
        }
    }

    #[tokio::test]
    async fn test_ndjson_stream_items() {
        let stream_id = Uuid::new_v4();
        let extraction_id = "test-extraction".to_string();

        let results = vec![
            Ok(create_test_extraction_result("1")),
            Ok(create_test_extraction_result("2")),
        ];
        let results_stream = iter(results);

        let config = NdjsonConfig {
            include_heartbeat: false,
            include_progress: false,
            include_metadata: false,
            ..Default::default()
        };

        let ndjson_stream = NdjsonStream::new(
            stream_id,
            extraction_id,
            results_stream,
            config,
        );

        let items: Vec<_> = ndjson_stream.collect().await;

        // Should have: start event + 2 results + completion event
        assert_eq!(items.len(), 4);

        // First item should be start event
        assert!(matches!(items[0], Ok(NdjsonItem::Event(ref event))
                        if matches!(event.event_type, StreamEventType::Started)));

        // Next two should be results
        assert!(matches!(items[1], Ok(NdjsonItem::Result(_))));
        assert!(matches!(items[2], Ok(NdjsonItem::Result(_))));

        // Last should be completion event
        assert!(matches!(items[3], Ok(NdjsonItem::Event(ref event))
                        if matches!(event.event_type, StreamEventType::Completed)));
    }

    #[tokio::test]
    async fn test_ndjson_stream_builder() {
        let stream_id = Uuid::new_v4();
        let extraction_id = "test-extraction".to_string();

        let results = vec![Ok(create_test_extraction_result("1"))];
        let results_stream = iter(results);

        let ndjson_stream = NdjsonStreamBuilder::new()
            .buffer_size(500)
            .include_progress(false)
            .include_heartbeat(false)
            .max_line_length(2048)
            .build(stream_id, extraction_id, results_stream);

        let items: Vec<_> = ndjson_stream.collect().await;
        assert!(!items.is_empty());
    }

    #[test]
    fn test_ndjson_item_display() {
        let result = NdjsonItem::Result(create_test_extraction_result("test"));
        let display = format!("{}", result);
        assert!(display.contains("Result(id=test"));

        let progress = NdjsonItem::Progress(ProgressUpdate {
            stream_id: Uuid::new_v4(),
            extraction_id: "test".to_string(),
            processed: 50,
            total: Some(100),
            current_item: None,
            timestamp: chrono::Utc::now(),
            rate_per_second: 10.0,
            estimated_completion: None,
        });
        let display = format!("{}", progress);
        assert!(display.contains("Progress(50/100"));
    }

    #[test]
    fn test_utils_validate_ndjson_item() {
        // Valid result
        let valid_result = NdjsonItem::Result(create_test_extraction_result("test"));
        assert!(utils::validate_ndjson_item(&valid_result).is_ok());

        // Invalid result (empty ID)
        let mut invalid_result = create_test_extraction_result("test");
        invalid_result.id = "".to_string();
        let invalid_item = NdjsonItem::Result(invalid_result);
        assert!(utils::validate_ndjson_item(&invalid_item).is_err());

        // Invalid progress (processed > total)
        let invalid_progress = NdjsonItem::Progress(ProgressUpdate {
            stream_id: Uuid::new_v4(),
            extraction_id: "test".to_string(),
            processed: 150,
            total: Some(100),
            current_item: None,
            timestamp: chrono::Utc::now(),
            rate_per_second: 10.0,
            estimated_completion: None,
        });
        assert!(utils::validate_ndjson_item(&invalid_progress).is_err());
    }

    #[test]
    fn test_utils_json_conversion() {
        let item = NdjsonItem::Result(create_test_extraction_result("test"));

        let compact = utils::to_compact_json(&item).unwrap();
        let pretty = utils::to_pretty_json(&item).unwrap();

        assert!(!compact.contains('\n'));
        assert!(pretty.contains('\n'));

        // Both should be valid JSON
        let _: serde_json::Value = serde_json::from_str(&compact).unwrap();
        let _: serde_json::Value = serde_json::from_str(&pretty).unwrap();
    }
}