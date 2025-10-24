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
from .models import (
    CrawlResult,
    CrawlResponse,
    StreamingResult,
    DomainProfile,
    EngineStats,
    CrawlOptions,
    ChunkingConfig,
)
from .exceptions import (
    RipTideError,
    ValidationError,
    APIError,
    NetworkError,
    TimeoutError,
)

__version__ = "0.1.0"
__all__ = [
    "RipTideClient",
    "CrawlResult",
    "CrawlResponse",
    "StreamingResult",
    "DomainProfile",
    "EngineStats",
    "CrawlOptions",
    "ChunkingConfig",
    "RipTideError",
    "ValidationError",
    "APIError",
    "NetworkError",
    "TimeoutError",
]
