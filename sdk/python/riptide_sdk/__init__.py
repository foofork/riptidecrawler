"""
RipTide Python SDK - Official Python client for RipTide API

This SDK provides a comprehensive interface to the RipTide web crawling and
extraction platform, with support for async/await, streaming, and all Phase 10+ features.

Example:
    >>> from riptide_sdk import RipTideClient
    >>>
    >>> async with RipTideClient(base_url="http://localhost:8080") as client:
    ...     result = await client.crawl.batch(["https://example.com"])
    ...     print(result.successful)
"""

from .client import RipTideClient
from .builder import RipTideClientBuilder, RetryConfig
from .models import (
    ExtractOptions,
    ExtractionResult,
    ContentMetadata,
    ParserMetadata,
    CrawlResult,
    CrawlResponse,
    StreamingResult,
    DomainProfile,
    EngineStats,
    CrawlOptions,
    ChunkingConfig,
    SearchOptions,
    SearchResponse,
    SearchResultItem,
    Session,
    SessionConfig,
    SessionStats,
    Cookie,
    SetCookieRequest,
    SpiderConfig,
    SpiderResult,
    SpiderStatus,
    SpiderControlResponse,
    CacheMode,
    StealthLevel,
    UAStrategy,
    ResultMode,
    PdfExtractionOptions,
    PdfExtractionResult,
    PdfJobStatus,
    PdfMetrics,
    PdfStreamProgress,
    # Worker/Job models
    Job,
    JobConfig,
    JobResult,
    JobType,
    JobPriority,
    JobStatus,
    QueueStats,
    WorkerStats,
    ScheduledJob,
    ScheduledJobConfig,
    JobListItem,
    JobListResponse,
)
from .exceptions import (
    RipTideError,
    ValidationError,
    APIError,
    NetworkError,
    TimeoutError,
    ConfigError,
    StreamingError,
)
from .formatters import (
    format_crawl_response,
    format_domain_profile,
    format_engine_stats,
)

__version__ = "0.1.0"
__all__ = [
    # Client
    "RipTideClient",
    "RipTideClientBuilder",
    # Models
    "CrawlResult",
    "CrawlResponse",
    "StreamingResult",
    "DomainProfile",
    "EngineStats",
    "CrawlOptions",
    "ChunkingConfig",
    "SearchOptions",
    "SearchResponse",
    "SearchResultItem",
    "ExtractOptions",
    "ExtractionResult",
    "ContentMetadata",
    "ParserMetadata",
    "Session",
    "SessionConfig",
    "SessionStats",
    "Cookie",
    "SetCookieRequest",
    "SpiderConfig",
    "SpiderResult",
    "SpiderStatus",
    "SpiderControlResponse",
    # Worker/Job models
    "Job",
    "JobConfig",
    "JobResult",
    "JobType",
    "QueueStats",
    "WorkerStats",
    "ScheduledJob",
    "ScheduledJobConfig",
    "JobListItem",
    "JobListResponse",
    # Enums
    "CacheMode",
    "StealthLevel",
    "UAStrategy",
    "ResultMode",
    "JobPriority",
    "JobStatus",
    # Exceptions
    "RipTideError",
    "ValidationError",
    "APIError",
    "NetworkError",
    "TimeoutError",
    "ConfigError",
    "StreamingError",
    # Formatters
    "format_crawl_response",
    "format_domain_profile",
    "format_engine_stats",
]
