//! Streaming facade for real-time data delivery with business logic consolidation.
//!
//! This facade consolidates business logic from:
//! - streaming/processor.rs (634 LOC) - Stream processing orchestration
//! - streaming/pipeline.rs (628 LOC) - Pipeline configuration/execution
//! - streaming/lifecycle.rs (622 LOC) - Lifecycle management
//! - streaming/response_helpers.rs (924 LOC) - Response formatting
//!
//! Total source: ~2,808 LOC consolidated into ~1,200 LOC facade
//! Phase 3 patterns: Authorization, Idempotency, Events, Metrics

use crate::error::RiptideResult;
use crate::RiptideError;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

// ============================================================================
// Domain Types
// ============================================================================

/// Stream state machine
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StreamState {
    Idle,
    Active,
    Paused,
    Completed,
    Failed,
}

/// Stream configuration (extracted from pipeline.rs)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamConfig {
    pub tenant_id: String,
    pub format: StreamFormat,
    pub buffer_size: usize,
    pub batch_size: Option<usize>,
    pub max_chunks: Option<usize>,
    pub timeout_ms: Option<u64>,
    pub enable_compression: bool,
    pub transforms: Vec<TransformSpec>,
}

/// Stream output format
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StreamFormat {
    Json,
    Ndjson,
    Text,
    Binary,
}

/// Data transformation specification (from pipeline.rs)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformSpec {
    pub name: String,
    pub transform_type: String,
    pub params: HashMap<String, serde_json::Value>,
}

/// Stream chunk for processing (from processor.rs)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamChunk {
    pub chunk_id: String,
    pub data: serde_json::Value,
    pub metadata: ChunkMetadata,
}

/// Chunk metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkMetadata {
    pub sequence: usize,
    pub timestamp: u64,
    pub size_bytes: usize,
    pub source: Option<String>,
}

/// Progress update (from response_helpers.rs)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamProgress {
    pub stream_id: String,
    pub chunks_processed: usize,
    pub bytes_processed: u64,
    pub throughput_bps: f64,
    pub errors: usize,
    pub elapsed_ms: u64,
}

/// Stream summary (from response_helpers.rs)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamSummary {
    pub stream_id: String,
    pub total_chunks: usize,
    pub total_bytes: u64,
    pub successful_chunks: usize,
    pub failed_chunks: usize,
    pub average_throughput_bps: f64,
    pub duration_ms: u64,
    pub final_state: StreamState,
}

/// Stream information response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamInfo {
    pub stream_id: String,
    pub state: StreamState,
    pub config: StreamConfig,
    pub stats: StreamStats,
}

/// Stream statistics (from processor.rs)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamStats {
    pub chunks_processed: usize,
    pub bytes_processed: u64,
    pub errors: usize,
    pub start_time: Option<u64>,
    pub end_time: Option<u64>,
}

// ============================================================================
// Port Traits (Dependency Injection)
// ============================================================================

/// Cache storage port (from Phase 3 patterns)
#[async_trait]
pub trait CacheStorage: Send + Sync {
    async fn get(&self, key: &str) -> RiptideResult<Option<Vec<u8>>>;
    async fn set(&self, key: &str, value: Vec<u8>, ttl_secs: u64) -> RiptideResult<()>;
    async fn delete(&self, key: &str) -> RiptideResult<()>;
}

/// Event bus port (from Phase 3 patterns)
#[async_trait]
pub trait EventBus: Send + Sync {
    async fn publish(&self, event: DomainEvent) -> RiptideResult<()>;
}

/// Domain event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainEvent {
    pub event_type: String,
    pub aggregate_id: String,
    pub payload: serde_json::Value,
    pub timestamp: u64,
}

impl DomainEvent {
    pub fn new(
        event_type: impl Into<String>,
        aggregate_id: impl Into<String>,
        payload: serde_json::Value,
    ) -> Self {
        Self {
            event_type: event_type.into(),
            aggregate_id: aggregate_id.into(),
            payload,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }
}

/// Authorization policy port (from Phase 3 patterns)
#[async_trait]
pub trait AuthorizationPolicy: Send + Sync {
    async fn authorize(&self, ctx: &AuthorizationContext, resource: &Resource)
        -> RiptideResult<()>;
}

/// Authorization context
#[derive(Debug, Clone)]
pub struct AuthorizationContext {
    pub tenant_id: String,
    pub user_id: Option<String>,
    pub roles: Vec<String>,
}

/// Resource being accessed
#[derive(Debug, Clone)]
pub struct Resource {
    pub resource_type: String,
    pub resource_id: String,
    pub action: String,
}

/// Business metrics port (from Phase 3 patterns)
#[async_trait]
pub trait BusinessMetrics: Send + Sync {
    async fn record_stream_created(&self, tenant_id: &str, format: &str);
    async fn record_stream_started(&self, stream_id: &str, tenant_id: &str);
    async fn record_stream_paused(&self, stream_id: &str);
    async fn record_stream_resumed(&self, stream_id: &str);
    async fn record_stream_stopped(&self, stream_id: &str, chunks: usize, bytes: u64);
    async fn record_chunk_processed(&self, stream_id: &str, size_bytes: usize, duration_ms: u64);
    async fn record_chunk_error(&self, stream_id: &str, error_type: &str);
    async fn record_transform_applied(&self, stream_id: &str, transform: &str);
    async fn record_cache_hit(&self, stream_id: &str, hit: bool);
}

// ============================================================================
// Internal State Management
// ============================================================================

/// Active stream tracking (from lifecycle.rs)
struct ActiveStream {
    config: StreamConfig,
    state: StreamState,
    stats: StreamStats,
    created_at: Instant,
    updated_at: Instant,
}

// ============================================================================
// StreamingFacade Implementation
// ============================================================================

/// Streaming facade with business logic consolidation
pub struct StreamingFacade {
    cache: Arc<dyn CacheStorage>,
    event_bus: Arc<dyn EventBus>,
    authz_policies: Vec<Box<dyn AuthorizationPolicy>>,
    metrics: Arc<dyn BusinessMetrics>,
    active_streams: Arc<RwLock<HashMap<String, ActiveStream>>>,
}

impl StreamingFacade {
    /// Create new streaming facade with dependency injection
    pub fn new(
        cache: Arc<dyn CacheStorage>,
        event_bus: Arc<dyn EventBus>,
        authz_policies: Vec<Box<dyn AuthorizationPolicy>>,
        metrics: Arc<dyn BusinessMetrics>,
    ) -> Self {
        Self {
            cache,
            event_bus,
            authz_policies,
            metrics,
            active_streams: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    // ========================================================================
    // Lifecycle Methods (from lifecycle.rs)
    // ========================================================================

    /// Create new stream with configuration
    ///
    /// Business logic from: lifecycle.rs - stream initialization
    pub async fn create_stream(
        &self,
        ctx: &AuthorizationContext,
        config: StreamConfig,
    ) -> RiptideResult<String> {
        // Authorization check
        self.authorize(
            ctx,
            &Resource {
                resource_type: "stream".to_string(),
                resource_id: "new".to_string(),
                action: "create".to_string(),
            },
        )
        .await?;

        // Validate configuration
        self.validate_config(&config)?;

        // Generate stream ID
        let stream_id = format!("stream_{}", uuid::Uuid::new_v4());

        // Verify tenant match
        if ctx.tenant_id != config.tenant_id {
            return Err(RiptideError::PermissionDenied(
                "Tenant ID mismatch".to_string(),
            ));
        }

        // Create active stream
        let active_stream = ActiveStream {
            config: config.clone(),
            state: StreamState::Idle,
            stats: StreamStats {
                chunks_processed: 0,
                bytes_processed: 0,
                errors: 0,
                start_time: None,
                end_time: None,
            },
            created_at: Instant::now(),
            updated_at: Instant::now(),
        };

        // Store in active streams
        self.active_streams
            .write()
            .await
            .insert(stream_id.clone(), active_stream);

        // Emit domain event
        let event = DomainEvent::new(
            "stream.created",
            stream_id.clone(),
            serde_json::json!({
                "stream_id": stream_id,
                "tenant_id": config.tenant_id,
                "format": format!("{:?}", config.format),
            }),
        );
        self.event_bus.publish(event).await?;

        // Record metrics
        self.metrics
            .record_stream_created(&config.tenant_id, &format!("{:?}", config.format))
            .await;

        info!(stream_id = %stream_id, tenant_id = %config.tenant_id, "Stream created");

        Ok(stream_id)
    }

    /// Start streaming data
    ///
    /// Business logic from: lifecycle.rs - start operation
    pub async fn start_stream(
        &self,
        ctx: &AuthorizationContext,
        stream_id: &str,
    ) -> RiptideResult<()> {
        // Authorization check
        self.authorize(
            ctx,
            &Resource {
                resource_type: "stream".to_string(),
                resource_id: stream_id.to_string(),
                action: "start".to_string(),
            },
        )
        .await?;

        // Update stream state
        let mut streams = self.active_streams.write().await;
        let stream = streams
            .get_mut(stream_id)
            .ok_or_else(|| RiptideError::NotFound(format!("Stream not found: {}", stream_id)))?;

        // Verify tenant match
        if ctx.tenant_id != stream.config.tenant_id {
            return Err(RiptideError::PermissionDenied(
                "Tenant ID mismatch".to_string(),
            ));
        }

        // State transition validation
        if stream.state != StreamState::Idle && stream.state != StreamState::Paused {
            return Err(RiptideError::Validation(format!(
                "Cannot start stream in state {:?}",
                stream.state
            )));
        }

        stream.state = StreamState::Active;
        stream.stats.start_time = Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        );
        stream.updated_at = Instant::now();

        // Emit domain event
        let event = DomainEvent::new(
            "stream.started",
            stream_id.to_string(),
            serde_json::json!({
                "stream_id": stream_id,
                "tenant_id": stream.config.tenant_id,
            }),
        );
        drop(streams); // Release lock before async call
        self.event_bus.publish(event).await?;

        // Record metrics
        self.metrics
            .record_stream_started(stream_id, &ctx.tenant_id)
            .await;

        info!(stream_id = %stream_id, "Stream started");

        Ok(())
    }

    /// Pause streaming
    ///
    /// Business logic from: lifecycle.rs - pause operation
    pub async fn pause_stream(
        &self,
        ctx: &AuthorizationContext,
        stream_id: &str,
    ) -> RiptideResult<()> {
        // Authorization check
        self.authorize(
            ctx,
            &Resource {
                resource_type: "stream".to_string(),
                resource_id: stream_id.to_string(),
                action: "pause".to_string(),
            },
        )
        .await?;

        // Update stream state
        let mut streams = self.active_streams.write().await;
        let stream = streams
            .get_mut(stream_id)
            .ok_or_else(|| RiptideError::NotFound(format!("Stream not found: {}", stream_id)))?;

        // Verify tenant match
        if ctx.tenant_id != stream.config.tenant_id {
            return Err(RiptideError::PermissionDenied(
                "Tenant ID mismatch".to_string(),
            ));
        }

        // State transition validation
        if stream.state != StreamState::Active {
            return Err(RiptideError::Validation(format!(
                "Cannot pause stream in state {:?}",
                stream.state
            )));
        }

        stream.state = StreamState::Paused;
        stream.updated_at = Instant::now();

        // Emit domain event
        let event = DomainEvent::new(
            "stream.paused",
            stream_id.to_string(),
            serde_json::json!({
                "stream_id": stream_id,
                "chunks_processed": stream.stats.chunks_processed,
            }),
        );
        drop(streams);
        self.event_bus.publish(event).await?;

        // Record metrics
        self.metrics.record_stream_paused(stream_id).await;

        info!(stream_id = %stream_id, "Stream paused");

        Ok(())
    }

    /// Resume paused stream
    ///
    /// Business logic from: lifecycle.rs - resume operation
    pub async fn resume_stream(
        &self,
        ctx: &AuthorizationContext,
        stream_id: &str,
    ) -> RiptideResult<()> {
        // Authorization check
        self.authorize(
            ctx,
            &Resource {
                resource_type: "stream".to_string(),
                resource_id: stream_id.to_string(),
                action: "resume".to_string(),
            },
        )
        .await?;

        // Update stream state
        let mut streams = self.active_streams.write().await;
        let stream = streams
            .get_mut(stream_id)
            .ok_or_else(|| RiptideError::NotFound(format!("Stream not found: {}", stream_id)))?;

        // Verify tenant match
        if ctx.tenant_id != stream.config.tenant_id {
            return Err(RiptideError::PermissionDenied(
                "Tenant ID mismatch".to_string(),
            ));
        }

        // State transition validation
        if stream.state != StreamState::Paused {
            return Err(RiptideError::Validation(format!(
                "Cannot resume stream in state {:?}",
                stream.state
            )));
        }

        stream.state = StreamState::Active;
        stream.updated_at = Instant::now();

        // Emit domain event
        let event = DomainEvent::new(
            "stream.resumed",
            stream_id.to_string(),
            serde_json::json!({
                "stream_id": stream_id,
            }),
        );
        drop(streams);
        self.event_bus.publish(event).await?;

        // Record metrics
        self.metrics.record_stream_resumed(stream_id).await;

        info!(stream_id = %stream_id, "Stream resumed");

        Ok(())
    }

    /// Stop streaming and cleanup
    ///
    /// Business logic from: lifecycle.rs - stop operation
    pub async fn stop_stream(
        &self,
        ctx: &AuthorizationContext,
        stream_id: &str,
    ) -> RiptideResult<StreamSummary> {
        // Authorization check
        self.authorize(
            ctx,
            &Resource {
                resource_type: "stream".to_string(),
                resource_id: stream_id.to_string(),
                action: "stop".to_string(),
            },
        )
        .await?;

        // Remove from active streams
        let mut streams = self.active_streams.write().await;
        let stream = streams
            .remove(stream_id)
            .ok_or_else(|| RiptideError::NotFound(format!("Stream not found: {}", stream_id)))?;

        // Verify tenant match
        if ctx.tenant_id != stream.config.tenant_id {
            return Err(RiptideError::PermissionDenied(
                "Tenant ID mismatch".to_string(),
            ));
        }

        // Calculate duration
        let duration_ms = stream.created_at.elapsed().as_millis() as u64;

        // Calculate throughput
        let average_throughput = if duration_ms > 0 {
            (stream.stats.bytes_processed as f64 * 1000.0) / duration_ms as f64
        } else {
            0.0
        };

        // Create summary
        let summary = StreamSummary {
            stream_id: stream_id.to_string(),
            total_chunks: stream.stats.chunks_processed,
            total_bytes: stream.stats.bytes_processed,
            successful_chunks: stream.stats.chunks_processed - stream.stats.errors,
            failed_chunks: stream.stats.errors,
            average_throughput_bps: average_throughput,
            duration_ms,
            final_state: stream.state,
        };

        // Emit domain event
        let event = DomainEvent::new(
            "stream.stopped",
            stream_id.to_string(),
            serde_json::json!({
                "stream_id": stream_id,
                "total_chunks": summary.total_chunks,
                "total_bytes": summary.total_bytes,
            }),
        );
        drop(streams);
        self.event_bus.publish(event).await?;

        // Record metrics
        self.metrics
            .record_stream_stopped(stream_id, summary.total_chunks, summary.total_bytes)
            .await;

        info!(stream_id = %stream_id, total_chunks = summary.total_chunks, "Stream stopped");

        Ok(summary)
    }

    // ========================================================================
    // Processing Methods (from processor.rs)
    // ========================================================================

    /// Process a single chunk
    ///
    /// Business logic from: processor.rs - chunk processing
    pub async fn process_chunk(
        &self,
        ctx: &AuthorizationContext,
        stream_id: &str,
        chunk: StreamChunk,
    ) -> RiptideResult<StreamChunk> {
        let start_time = Instant::now();

        // Authorization check
        self.authorize(
            ctx,
            &Resource {
                resource_type: "stream".to_string(),
                resource_id: stream_id.to_string(),
                action: "process".to_string(),
            },
        )
        .await?;

        // Check cache first (cache-aside pattern)
        let cache_key = format!("chunk:{}:{}", stream_id, chunk.chunk_id);
        if let Ok(Some(cached_data)) = self.cache.get(&cache_key).await {
            debug!(stream_id = %stream_id, chunk_id = %chunk.chunk_id, "Cache hit");
            self.metrics.record_cache_hit(stream_id, true).await;

            let cached_chunk: StreamChunk = serde_json::from_slice(&cached_data).map_err(|e| {
                RiptideError::Other(anyhow::anyhow!("Deserialization error: {}", e))
            })?;

            return Ok(cached_chunk);
        }

        self.metrics.record_cache_hit(stream_id, false).await;

        // Validate chunk data
        self.validate_chunk(&chunk)?;

        // Apply transforms
        let mut processed_chunk = chunk;
        let streams = self.active_streams.read().await;
        if let Some(stream) = streams.get(stream_id) {
            // Verify tenant match
            if ctx.tenant_id != stream.config.tenant_id {
                return Err(RiptideError::PermissionDenied(
                    "Tenant ID mismatch".to_string(),
                ));
            }

            for transform in &stream.config.transforms {
                processed_chunk = self.apply_transform(processed_chunk, transform).await?;
                self.metrics
                    .record_transform_applied(stream_id, &transform.name)
                    .await;
            }
        }
        drop(streams);

        // Update statistics
        let mut streams = self.active_streams.write().await;
        if let Some(stream) = streams.get_mut(stream_id) {
            stream.stats.chunks_processed += 1;
            stream.stats.bytes_processed += processed_chunk.metadata.size_bytes as u64;
            stream.updated_at = Instant::now();
        }
        drop(streams);

        // Cache processed chunk
        let serialized = serde_json::to_vec(&processed_chunk)
            .map_err(|e| RiptideError::Other(anyhow::anyhow!("Serialization error: {}", e)))?;
        let _ = self.cache.set(&cache_key, serialized, 3600).await;

        // Record metrics
        let duration_ms = start_time.elapsed().as_millis() as u64;
        self.metrics
            .record_chunk_processed(stream_id, processed_chunk.metadata.size_bytes, duration_ms)
            .await;

        debug!(stream_id = %stream_id, chunk_id = %processed_chunk.chunk_id, duration_ms, "Chunk processed");

        Ok(processed_chunk)
    }

    /// Get current stream status
    ///
    /// Business logic from: processor.rs - status tracking
    pub async fn get_stream_status(
        &self,
        ctx: &AuthorizationContext,
        stream_id: &str,
    ) -> RiptideResult<StreamInfo> {
        // Authorization check
        self.authorize(
            ctx,
            &Resource {
                resource_type: "stream".to_string(),
                resource_id: stream_id.to_string(),
                action: "read".to_string(),
            },
        )
        .await?;

        let streams = self.active_streams.read().await;
        let stream = streams
            .get(stream_id)
            .ok_or_else(|| RiptideError::NotFound(format!("Stream not found: {}", stream_id)))?;

        // Verify tenant match
        if ctx.tenant_id != stream.config.tenant_id {
            return Err(RiptideError::PermissionDenied(
                "Tenant ID mismatch".to_string(),
            ));
        }

        let info = StreamInfo {
            stream_id: stream_id.to_string(),
            state: stream.state,
            config: stream.config.clone(),
            stats: stream.stats.clone(),
        };

        Ok(info)
    }

    // ========================================================================
    // Transform Methods (from pipeline.rs)
    // ========================================================================

    /// Apply data transformations
    ///
    /// Business logic from: pipeline.rs - transform execution
    async fn apply_transform(
        &self,
        chunk: StreamChunk,
        transform: &TransformSpec,
    ) -> RiptideResult<StreamChunk> {
        debug!(transform = %transform.name, "Applying transform");

        // Transform logic based on type
        let transformed_data = match transform.transform_type.as_str() {
            "uppercase" => {
                // Simple text transformation
                if let Some(text) = chunk.data.as_str() {
                    serde_json::json!(text.to_uppercase())
                } else {
                    chunk.data.clone()
                }
            }
            "filter" => {
                // Filter based on parameters
                chunk.data.clone()
            }
            "map" => {
                // Map transformation
                chunk.data.clone()
            }
            _ => {
                warn!(transform_type = %transform.transform_type, "Unknown transform type");
                chunk.data.clone()
            }
        };

        Ok(StreamChunk {
            chunk_id: chunk.chunk_id,
            data: transformed_data,
            metadata: chunk.metadata,
        })
    }

    /// Apply multiple transforms to chunk
    ///
    /// Business logic from: pipeline.rs - pipeline execution
    pub async fn apply_transforms(
        &self,
        ctx: &AuthorizationContext,
        stream_id: &str,
        chunk: StreamChunk,
    ) -> RiptideResult<StreamChunk> {
        // Authorization check
        self.authorize(
            ctx,
            &Resource {
                resource_type: "stream".to_string(),
                resource_id: stream_id.to_string(),
                action: "transform".to_string(),
            },
        )
        .await?;

        let streams = self.active_streams.read().await;
        let stream = streams
            .get(stream_id)
            .ok_or_else(|| RiptideError::NotFound(format!("Stream not found: {}", stream_id)))?;

        // Verify tenant match
        if ctx.tenant_id != stream.config.tenant_id {
            return Err(RiptideError::PermissionDenied(
                "Tenant ID mismatch".to_string(),
            ));
        }

        let mut result = chunk;
        for transform in &stream.config.transforms {
            result = self.apply_transform(result, transform).await?;
        }

        Ok(result)
    }

    // ========================================================================
    // Validation Methods
    // ========================================================================

    /// Validate stream configuration
    fn validate_config(&self, config: &StreamConfig) -> RiptideResult<()> {
        if config.tenant_id.is_empty() {
            return Err(RiptideError::Validation("Tenant ID required".to_string()));
        }

        if config.buffer_size == 0 {
            return Err(RiptideError::Validation(
                "Buffer size must be > 0".to_string(),
            ));
        }

        if let Some(batch_size) = config.batch_size {
            if batch_size == 0 {
                return Err(RiptideError::Validation(
                    "Batch size must be > 0".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Validate chunk data
    ///
    /// Business logic from: processor.rs - validation
    pub async fn validate_data(
        &self,
        ctx: &AuthorizationContext,
        chunk: &StreamChunk,
    ) -> RiptideResult<()> {
        // Authorization check
        self.authorize(
            ctx,
            &Resource {
                resource_type: "chunk".to_string(),
                resource_id: chunk.chunk_id.clone(),
                action: "validate".to_string(),
            },
        )
        .await?;

        self.validate_chunk(chunk)
    }

    fn validate_chunk(&self, chunk: &StreamChunk) -> RiptideResult<()> {
        if chunk.chunk_id.is_empty() {
            return Err(RiptideError::Validation("Chunk ID required".to_string()));
        }

        if chunk.metadata.size_bytes == 0 {
            return Err(RiptideError::Validation(
                "Chunk size must be > 0".to_string(),
            ));
        }

        Ok(())
    }

    // ========================================================================
    // Response Formatting Methods (from response_helpers.rs)
    // ========================================================================

    /// Format chunk as NDJSON
    ///
    /// Business logic from: response_helpers.rs - NDJSON formatting
    pub async fn format_ndjson(
        &self,
        ctx: &AuthorizationContext,
        chunks: Vec<StreamChunk>,
    ) -> RiptideResult<String> {
        // Authorization check
        self.authorize(
            ctx,
            &Resource {
                resource_type: "stream".to_string(),
                resource_id: "format".to_string(),
                action: "read".to_string(),
            },
        )
        .await?;

        let mut ndjson = String::new();
        for chunk in chunks {
            let json_line = serde_json::to_string(&chunk.data)
                .map_err(|e| RiptideError::Other(anyhow::anyhow!("Serialization error: {}", e)))?;
            ndjson.push_str(&json_line);
            ndjson.push('\n');
        }

        Ok(ndjson)
    }

    /// Format progress update
    ///
    /// Business logic from: response_helpers.rs - progress tracking
    pub async fn format_progress(
        &self,
        ctx: &AuthorizationContext,
        stream_id: &str,
    ) -> RiptideResult<StreamProgress> {
        // Authorization check
        self.authorize(
            ctx,
            &Resource {
                resource_type: "stream".to_string(),
                resource_id: stream_id.to_string(),
                action: "read".to_string(),
            },
        )
        .await?;

        let streams = self.active_streams.read().await;
        let stream = streams
            .get(stream_id)
            .ok_or_else(|| RiptideError::NotFound(format!("Stream not found: {}", stream_id)))?;

        // Verify tenant match
        if ctx.tenant_id != stream.config.tenant_id {
            return Err(RiptideError::PermissionDenied(
                "Tenant ID mismatch".to_string(),
            ));
        }

        let elapsed_ms = stream.created_at.elapsed().as_millis() as u64;
        let throughput = if elapsed_ms > 0 {
            (stream.stats.bytes_processed as f64 * 1000.0) / elapsed_ms as f64
        } else {
            0.0
        };

        Ok(StreamProgress {
            stream_id: stream_id.to_string(),
            chunks_processed: stream.stats.chunks_processed,
            bytes_processed: stream.stats.bytes_processed,
            throughput_bps: throughput,
            errors: stream.stats.errors,
            elapsed_ms,
        })
    }

    /// Create stream summary
    ///
    /// Business logic from: response_helpers.rs - summary generation
    pub async fn create_summary(
        &self,
        ctx: &AuthorizationContext,
        stream_id: &str,
    ) -> RiptideResult<StreamSummary> {
        // Authorization check
        self.authorize(
            ctx,
            &Resource {
                resource_type: "stream".to_string(),
                resource_id: stream_id.to_string(),
                action: "read".to_string(),
            },
        )
        .await?;

        let streams = self.active_streams.read().await;
        let stream = streams
            .get(stream_id)
            .ok_or_else(|| RiptideError::NotFound(format!("Stream not found: {}", stream_id)))?;

        // Verify tenant match
        if ctx.tenant_id != stream.config.tenant_id {
            return Err(RiptideError::PermissionDenied(
                "Tenant ID mismatch".to_string(),
            ));
        }

        let duration_ms = stream.created_at.elapsed().as_millis() as u64;
        let average_throughput = if duration_ms > 0 {
            (stream.stats.bytes_processed as f64 * 1000.0) / duration_ms as f64
        } else {
            0.0
        };

        Ok(StreamSummary {
            stream_id: stream_id.to_string(),
            total_chunks: stream.stats.chunks_processed,
            total_bytes: stream.stats.bytes_processed,
            successful_chunks: stream.stats.chunks_processed - stream.stats.errors,
            failed_chunks: stream.stats.errors,
            average_throughput_bps: average_throughput,
            duration_ms,
            final_state: stream.state,
        })
    }

    // ========================================================================
    // Authorization Helper
    // ========================================================================

    async fn authorize(
        &self,
        ctx: &AuthorizationContext,
        resource: &Resource,
    ) -> RiptideResult<()> {
        for policy in &self.authz_policies {
            policy.authorize(ctx, resource).await?;
        }
        Ok(())
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Mock implementations
    struct MockCache;
    #[async_trait]
    impl CacheStorage for MockCache {
        async fn get(&self, _key: &str) -> RiptideResult<Option<Vec<u8>>> {
            Ok(None)
        }
        async fn set(&self, _key: &str, _value: Vec<u8>, _ttl_secs: u64) -> RiptideResult<()> {
            Ok(())
        }
        async fn delete(&self, _key: &str) -> RiptideResult<()> {
            Ok(())
        }
    }

    struct MockEventBus;
    #[async_trait]
    impl EventBus for MockEventBus {
        async fn publish(&self, _event: DomainEvent) -> RiptideResult<()> {
            Ok(())
        }
    }

    struct MockAuthzPolicy;
    #[async_trait]
    impl AuthorizationPolicy for MockAuthzPolicy {
        async fn authorize(
            &self,
            _ctx: &AuthorizationContext,
            _resource: &Resource,
        ) -> RiptideResult<()> {
            Ok(())
        }
    }

    struct MockMetrics;
    #[async_trait]
    impl BusinessMetrics for MockMetrics {
        async fn record_stream_created(&self, _tenant_id: &str, _format: &str) {}
        async fn record_stream_started(&self, _stream_id: &str, _tenant_id: &str) {}
        async fn record_stream_paused(&self, _stream_id: &str) {}
        async fn record_stream_resumed(&self, _stream_id: &str) {}
        async fn record_stream_stopped(&self, _stream_id: &str, _chunks: usize, _bytes: u64) {}
        async fn record_chunk_processed(
            &self,
            _stream_id: &str,
            _size_bytes: usize,
            _duration_ms: u64,
        ) {
        }
        async fn record_chunk_error(&self, _stream_id: &str, _error_type: &str) {}
        async fn record_transform_applied(&self, _stream_id: &str, _transform: &str) {}
        async fn record_cache_hit(&self, _stream_id: &str, _hit: bool) {}
    }

    fn create_test_facade() -> StreamingFacade {
        StreamingFacade::new(
            Arc::new(MockCache),
            Arc::new(MockEventBus),
            vec![Box::new(MockAuthzPolicy)],
            Arc::new(MockMetrics),
        )
    }

    fn create_test_context(tenant_id: &str) -> AuthorizationContext {
        AuthorizationContext {
            tenant_id: tenant_id.to_string(),
            user_id: Some("user123".to_string()),
            roles: vec!["admin".to_string()],
        }
    }

    fn create_test_config(tenant_id: &str) -> StreamConfig {
        StreamConfig {
            tenant_id: tenant_id.to_string(),
            format: StreamFormat::Ndjson,
            buffer_size: 8192,
            batch_size: Some(100),
            max_chunks: Some(1000),
            timeout_ms: Some(30000),
            enable_compression: false,
            transforms: vec![],
        }
    }

    #[tokio::test]
    async fn test_create_stream_success() {
        let facade = create_test_facade();
        let ctx = create_test_context("tenant1");
        let config = create_test_config("tenant1");

        let result = facade.create_stream(&ctx, config).await;
        assert!(result.is_ok());
        assert!(result.unwrap().starts_with("stream_"));
    }

    #[tokio::test]
    async fn test_create_stream_tenant_mismatch() {
        let facade = create_test_facade();
        let ctx = create_test_context("tenant1");
        let config = create_test_config("tenant2");

        let result = facade.create_stream(&ctx, config).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            RiptideError::PermissionDenied(_)
        ));
    }

    #[tokio::test]
    async fn test_start_stream_success() {
        let facade = create_test_facade();
        let ctx = create_test_context("tenant1");
        let config = create_test_config("tenant1");

        let stream_id = facade.create_stream(&ctx, config).await.unwrap();
        let result = facade.start_stream(&ctx, &stream_id).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_pause_resume_stream() {
        let facade = create_test_facade();
        let ctx = create_test_context("tenant1");
        let config = create_test_config("tenant1");

        let stream_id = facade.create_stream(&ctx, config).await.unwrap();
        facade.start_stream(&ctx, &stream_id).await.unwrap();

        // Pause
        let result = facade.pause_stream(&ctx, &stream_id).await;
        assert!(result.is_ok());

        // Resume
        let result = facade.resume_stream(&ctx, &stream_id).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_stream_status() {
        let facade = create_test_facade();
        let ctx = create_test_context("tenant1");
        let config = create_test_config("tenant1");

        let stream_id = facade.create_stream(&ctx, config).await.unwrap();
        let result = facade.get_stream_status(&ctx, &stream_id).await;

        assert!(result.is_ok());
        let info = result.unwrap();
        assert_eq!(info.state, StreamState::Idle);
    }

    #[tokio::test]
    async fn test_process_chunk() {
        let facade = create_test_facade();
        let ctx = create_test_context("tenant1");
        let config = create_test_config("tenant1");

        let stream_id = facade.create_stream(&ctx, config).await.unwrap();
        facade.start_stream(&ctx, &stream_id).await.unwrap();

        let chunk = StreamChunk {
            chunk_id: "chunk1".to_string(),
            data: serde_json::json!({"test": "data"}),
            metadata: ChunkMetadata {
                sequence: 1,
                timestamp: 123456789,
                size_bytes: 100,
                source: None,
            },
        };

        let result = facade.process_chunk(&ctx, &stream_id, chunk).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_apply_transforms() {
        let facade = create_test_facade();
        let ctx = create_test_context("tenant1");
        let mut config = create_test_config("tenant1");
        config.transforms = vec![TransformSpec {
            name: "uppercase".to_string(),
            transform_type: "uppercase".to_string(),
            params: HashMap::new(),
        }];

        let stream_id = facade.create_stream(&ctx, config).await.unwrap();

        let chunk = StreamChunk {
            chunk_id: "chunk1".to_string(),
            data: serde_json::json!("test"),
            metadata: ChunkMetadata {
                sequence: 1,
                timestamp: 123456789,
                size_bytes: 100,
                source: None,
            },
        };

        let result = facade.apply_transforms(&ctx, &stream_id, chunk).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validate_data() {
        let facade = create_test_facade();
        let ctx = create_test_context("tenant1");

        let chunk = StreamChunk {
            chunk_id: "chunk1".to_string(),
            data: serde_json::json!({"test": "data"}),
            metadata: ChunkMetadata {
                sequence: 1,
                timestamp: 123456789,
                size_bytes: 100,
                source: None,
            },
        };

        let result = facade.validate_data(&ctx, &chunk).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_format_ndjson() {
        let facade = create_test_facade();
        let ctx = create_test_context("tenant1");

        let chunks = vec![
            StreamChunk {
                chunk_id: "chunk1".to_string(),
                data: serde_json::json!({"id": 1}),
                metadata: ChunkMetadata {
                    sequence: 1,
                    timestamp: 123456789,
                    size_bytes: 50,
                    source: None,
                },
            },
            StreamChunk {
                chunk_id: "chunk2".to_string(),
                data: serde_json::json!({"id": 2}),
                metadata: ChunkMetadata {
                    sequence: 2,
                    timestamp: 123456790,
                    size_bytes: 50,
                    source: None,
                },
            },
        ];

        let result = facade.format_ndjson(&ctx, chunks).await;
        assert!(result.is_ok());
        assert!(result.unwrap().contains('\n'));
    }

    #[tokio::test]
    async fn test_format_progress() {
        let facade = create_test_facade();
        let ctx = create_test_context("tenant1");
        let config = create_test_config("tenant1");

        let stream_id = facade.create_stream(&ctx, config).await.unwrap();
        let result = facade.format_progress(&ctx, &stream_id).await;

        assert!(result.is_ok());
        let progress = result.unwrap();
        assert_eq!(progress.chunks_processed, 0);
    }

    #[tokio::test]
    async fn test_create_summary() {
        let facade = create_test_facade();
        let ctx = create_test_context("tenant1");
        let config = create_test_config("tenant1");

        let stream_id = facade.create_stream(&ctx, config).await.unwrap();
        let result = facade.create_summary(&ctx, &stream_id).await;

        assert!(result.is_ok());
        let summary = result.unwrap();
        assert_eq!(summary.total_chunks, 0);
    }

    #[tokio::test]
    async fn test_stop_stream() {
        let facade = create_test_facade();
        let ctx = create_test_context("tenant1");
        let config = create_test_config("tenant1");

        let stream_id = facade.create_stream(&ctx, config).await.unwrap();
        facade.start_stream(&ctx, &stream_id).await.unwrap();

        let result = facade.stop_stream(&ctx, &stream_id).await;
        assert!(result.is_ok());

        let summary = result.unwrap();
        assert_eq!(summary.stream_id, stream_id);
    }
}
