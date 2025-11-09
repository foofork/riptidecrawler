//! Facade implementations for different use cases.
//!
//! This module contains specialized facades that provide simplified
//! interfaces for common web scraping tasks.

pub mod browser;
pub mod browser_metrics;
pub mod crawl_facade;
pub mod engine;
pub mod extraction;
pub mod extraction_authz;
pub mod extraction_metrics;
pub mod extractor;
pub mod intelligence;
pub mod llm;
pub mod pdf;
pub mod pipeline;
pub mod pipeline_metrics;
#[cfg(feature = "llm")]
pub mod profile;
pub mod profiling;
pub mod render;
pub mod render_strategy;
pub mod scraper;
pub mod search;
pub mod session;
pub mod session_metrics;
pub mod spider;
pub mod table;
pub mod trace;
#[cfg(feature = "workers")]
pub mod workers;

// Sprint 3.2: Medium handler facades
pub mod chunking;
pub mod deep_search;
pub mod memory;
pub mod monitoring;
pub mod pipeline_phases;
pub mod strategies;
pub mod streaming;

// Sprint 4.4: Resource management facade
pub mod resource;

// Re-export metrics for facade use
pub use crate::metrics::BusinessMetrics;
pub use browser_metrics::MetricsBrowserFacade;
pub use extraction_metrics::{ExtractionMetricsExt, MetricsExtractionFacade};
pub use pipeline_metrics::MetricsPipelineFacade;
pub use session_metrics::MetricsSessionFacade;

pub use browser::{
    BrowserAction, BrowserFacade, BrowserSession, Cookie, ImageFormat, ScreenshotOptions,
};
pub use crawl_facade::{CrawlFacade, CrawlMode, CrawlResult};
pub use engine::{
    EngineCapability, EngineConfig, EngineFacade, EngineSelectionCriteria, EngineStats,
};
pub use extraction::{ExtractedDoc, UrlExtractionFacade, UrlExtractionOptions};
pub use extraction_authz::AuthorizedExtractionFacade;
pub use extractor::{
    ExtractedData, ExtractionFacade, ExtractionStrategy, FieldSpec, FieldType,
    HtmlExtractionOptions, PdfExtractionOptions, Schema,
};
pub use llm::{
    LlmCapabilities, LlmFacade, LlmProvider, LlmRequest, LlmResponse,
    MetricsCollector as LlmMetricsCollector, TokenUsage,
};
pub use pdf::{
    EnhancedProgressUpdate, PdfFacade, PdfInput, PdfMetadata, PdfProcessOptions, PdfProcessResult,
    ProcessingStats,
};
pub use pipeline::PipelineFacade;
#[cfg(feature = "llm")]
pub use profile::{
    BatchCreateResult, BatchFailure, BulkStatistics, ProfileConfigRequest, ProfileFacade,
    ProfileMetadataRequest,
};
pub use profiling::{
    AllocationMetrics, BottleneckAnalysis, CpuMetrics, HeapSnapshot, HotspotInfo,
    LeakDetectionResult, LeakInfo, LoadAverage, MemoryMetrics, ProfilingFacade, SizeDistribution,
};
pub use render::{
    RenderConfig, RenderFacade, RenderResult, RenderStrategy, SessionContext, SessionCookie,
};
pub use render_strategy::RenderStrategyFacade;
pub use scraper::ScraperFacade;
pub use search::SearchFacade;
pub use session::{SessionConfig, SessionEvent, SessionFacade};
pub use spider::{CrawlSummary, SpiderFacade, SpiderPreset};
pub use table::{
    TableCacheService, TableExtractionOptions as FacadeTableExtractionOptions,
    TableExtractionRequest, TableFacade, TableFormat, TableMetadata as FacadeTableMetadata,
    TableSummary as FacadeTableSummary,
};
pub use trace::{
    CompleteTrace, SpanData, SpanEvent, TelemetryBackend, TraceData, TraceFacade, TraceMetadata,
    TraceQuery,
};
#[cfg(feature = "workers")]
pub use workers::{
    AuthorizationContext, JobFilter, JobResult, QueueStats, ScheduledJobRequest, SubmitJobRequest,
    WorkerMetrics, WorkerPoolStats, WorkerService, WorkersFacade,
};

// Sprint 3.2 exports
pub use chunking::{ChunkData, ChunkParameters, ChunkRequest, ChunkResponse, ChunkingFacade};
pub use deep_search::{DeepSearchFacade, DeepSearchRequest, DeepSearchResponse, SearchResult};
pub use memory::{MemoryFacade, MemoryUsageResponse};
pub use monitoring::MonitoringFacade;
pub use pipeline_phases::{
    PhaseConfig, PhaseExecutionRequest, PhaseExecutionResponse, PhaseMetrics, PipelinePhasesFacade,
};
pub use strategies::{AlternativeStrategy, StrategiesFacade, StrategyRequest, StrategyResponse};
pub use streaming::{
    AuthorizationContext as StreamingAuthzContext, CacheStorage as StreamingCacheStorage,
    ChunkMetadata, DomainEvent as StreamingDomainEvent, EventBus as StreamingEventBus,
    Resource as StreamingResource, StreamChunk, StreamConfig, StreamFormat, StreamInfo,
    StreamProgress, StreamState, StreamStats, StreamSummary, StreamingFacade, TransformSpec,
};

// Sprint 4.4 exports
pub use resource::{ResourceConfig, ResourceFacade, ResourceResult, ResourceStatus};
