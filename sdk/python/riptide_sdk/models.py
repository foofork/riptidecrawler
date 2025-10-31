"""
Data models for RipTide SDK

Provides type-safe data classes for all API requests and responses.
"""

from dataclasses import dataclass, field, asdict
from typing import Optional, List, Dict, Any, Literal
from datetime import datetime
from enum import Enum


# ============================================================================
# Enumerations
# ============================================================================

class CacheMode(str, Enum):
    """Cache mode for crawl operations"""
    NONE = "none"
    READ = "read"
    WRITE = "write"
    READ_WRITE = "read_write"


class StealthLevel(str, Enum):
    """Stealth level for domain profiles"""
    LOW = "low"
    MEDIUM = "medium"
    HIGH = "high"


class UAStrategy(str, Enum):
    """User-Agent rotation strategy"""
    FIXED = "fixed"
    ROTATE = "rotate"
    RANDOM = "random"


class ResultMode(str, Enum):
    """Result mode for spider crawl operations"""
    STATS = "stats"
    URLS = "urls"


# ============================================================================
# Configuration Models
# ============================================================================

@dataclass
class ChunkingConfig:
    """Configuration for content chunking"""
    enabled: bool = False
    max_chunk_size: Optional[int] = None
    overlap: Optional[int] = None
    strategy: Optional[str] = None

    def to_dict(self) -> Dict[str, Any]:
        return {k: v for k, v in asdict(self).items() if v is not None}


@dataclass
class CrawlOptions:
    """Options for crawl operations"""
    cache_mode: CacheMode = CacheMode.READ_WRITE
    concurrency: int = 5
    use_spider: Optional[bool] = None
    chunking_config: Optional[ChunkingConfig] = None
    timeout_secs: Optional[int] = None

    def to_dict(self) -> Dict[str, Any]:
        data = {
            "cache_mode": self.cache_mode.value,
            "concurrency": self.concurrency,
        }
        if self.use_spider is not None:
            data["use_spider"] = self.use_spider
        if self.chunking_config:
            data["chunking_config"] = self.chunking_config.to_dict()
        if self.timeout_secs:
            data["timeout_secs"] = self.timeout_secs
        return data


# ============================================================================
# Domain Profile Models (Phase 10.4)
# ============================================================================

@dataclass
class ProfileConfig:
    """Domain profile configuration"""
    stealth_level: Optional[StealthLevel] = None
    rate_limit: Optional[float] = None
    respect_robots_txt: Optional[bool] = None
    ua_strategy: Optional[UAStrategy] = None
    confidence_threshold: Optional[float] = None
    enable_javascript: Optional[bool] = None
    request_timeout_secs: Optional[int] = None

    def to_dict(self) -> Dict[str, Any]:
        data = {}
        if self.stealth_level:
            data["stealth_level"] = self.stealth_level.value
        if self.rate_limit is not None:
            data["rate_limit"] = self.rate_limit
        if self.respect_robots_txt is not None:
            data["respect_robots_txt"] = self.respect_robots_txt
        if self.ua_strategy:
            data["ua_strategy"] = self.ua_strategy.value
        if self.confidence_threshold is not None:
            data["confidence_threshold"] = self.confidence_threshold
        if self.enable_javascript is not None:
            data["enable_javascript"] = self.enable_javascript
        if self.request_timeout_secs is not None:
            data["request_timeout_secs"] = self.request_timeout_secs
        return data


@dataclass
class ProfileMetadata:
    """Domain profile metadata"""
    description: Optional[str] = None
    tags: Optional[List[str]] = None
    author: Optional[str] = None

    def to_dict(self) -> Dict[str, Any]:
        data = {}
        if self.description:
            data["description"] = self.description
        if self.tags:
            data["tags"] = self.tags
        if self.author:
            data["author"] = self.author
        return data


@dataclass
class DomainProfile:
    """Domain profile with configuration and metadata"""
    domain: str
    config: Optional[ProfileConfig] = None
    metadata: Optional[ProfileMetadata] = None
    created_at: Optional[str] = None
    updated_at: Optional[str] = None

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'DomainProfile':
        """Create DomainProfile from API response"""
        config = None
        if 'config' in data and data['config']:
            config = ProfileConfig(**data['config'])

        metadata = None
        if 'metadata' in data and data['metadata']:
            metadata = ProfileMetadata(**data['metadata'])

        return cls(
            domain=data['domain'],
            config=config,
            metadata=metadata,
            created_at=data.get('created_at'),
            updated_at=data.get('updated_at'),
        )

    def to_dict(self) -> Dict[str, Any]:
        data = {"domain": self.domain}
        if self.config:
            data["config"] = self.config.to_dict()
        if self.metadata:
            data["metadata"] = self.metadata.to_dict()
        return data


@dataclass
class ProfileStats:
    """Statistics for a domain profile"""
    domain: str
    total_requests: int
    cache_hits: int
    cache_misses: int
    avg_response_time_ms: float
    last_used: Optional[str] = None

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'ProfileStats':
        return cls(**data)


# ============================================================================
# Engine Selection Models (Phase 10)
# ============================================================================

@dataclass
class EngineDecision:
    """Engine selection decision result"""
    engine: str
    confidence: float
    reasoning: str
    flags: Dict[str, bool]

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'EngineDecision':
        return cls(**data)


@dataclass
class EngineStats:
    """Engine usage statistics"""
    total_decisions: int
    raw_count: int
    probes_first_count: int
    headless_count: int
    probe_first_enabled: bool

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'EngineStats':
        return cls(**data)


# ============================================================================
# Crawl Result Models
# ============================================================================

@dataclass
class ErrorInfo:
    """Error information for failed crawls"""
    error_type: str
    message: str
    retryable: bool

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'ErrorInfo':
        return cls(**data)


@dataclass
class Document:
    """Extracted document content"""
    html: Optional[str] = None
    text: Optional[str] = None
    markdown: Optional[str] = None
    metadata: Optional[Dict[str, Any]] = None
    links: Optional[List[str]] = None

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'Document':
        return cls(**data)


@dataclass
class CrawlResult:
    """Result for a single URL crawl"""
    url: str
    status: int
    from_cache: bool
    gate_decision: str
    quality_score: float
    processing_time_ms: int
    document: Optional[Document] = None
    error: Optional[ErrorInfo] = None
    cache_key: str = ""

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'CrawlResult':
        document = None
        if data.get('document'):
            document = Document.from_dict(data['document'])

        error = None
        if data.get('error'):
            error = ErrorInfo.from_dict(data['error'])

        return cls(
            url=data['url'],
            status=data['status'],
            from_cache=data['from_cache'],
            gate_decision=data['gate_decision'],
            quality_score=data['quality_score'],
            processing_time_ms=data['processing_time_ms'],
            document=document,
            error=error,
            cache_key=data.get('cache_key', ''),
        )


@dataclass
class GateDecisionBreakdown:
    """Breakdown of gate decisions"""
    raw: int
    probes_first: int
    headless: int
    cached: int

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'GateDecisionBreakdown':
        return cls(**data)


@dataclass
class CrawlStatistics:
    """Statistics for crawl operations"""
    total_processing_time_ms: int
    avg_processing_time_ms: float
    gate_decisions: GateDecisionBreakdown
    cache_hit_rate: float

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'CrawlStatistics':
        return cls(
            total_processing_time_ms=data['total_processing_time_ms'],
            avg_processing_time_ms=data['avg_processing_time_ms'],
            gate_decisions=GateDecisionBreakdown.from_dict(data['gate_decisions']),
            cache_hit_rate=data['cache_hit_rate'],
        )


@dataclass
class CrawlResponse:
    """Response for batch crawl operations"""
    total_urls: int
    successful: int
    failed: int
    from_cache: int
    results: List[CrawlResult]
    statistics: CrawlStatistics

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'CrawlResponse':
        return cls(
            total_urls=data['total_urls'],
            successful=data['successful'],
            failed=data['failed'],
            from_cache=data['from_cache'],
            results=[CrawlResult.from_dict(r) for r in data['results']],
            statistics=CrawlStatistics.from_dict(data['statistics']),
        )


# ============================================================================
# Streaming Models
# ============================================================================

@dataclass
class StreamingResult:
    """Result from streaming operations"""
    event_type: str
    data: Dict[str, Any]
    timestamp: Optional[str] = None

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'StreamingResult':
        return cls(**data)


# ============================================================================
# Extraction Models
# ============================================================================

@dataclass
class ExtractOptions:
    """Options for content extraction.

    Attributes:
        strategy: Extraction strategy to use (v0.9.0+ default changed to "native").
            - "native" (default): Fast pure-Rust extraction (2-5ms, always available)
            - "wasm": WASM-based extraction (10-20ms, requires server --features wasm-extractor)
            - "multi": Server auto-selects best available strategy
            Defaults to "native" for best performance.

        quality_threshold: Minimum quality score (0.0-1.0) for extraction.
        timeout_ms: Maximum time to wait for extraction in milliseconds.

    Server Compatibility:
        Native-only server (default build):
            ✅ strategy="native" - Works
            ✅ strategy="multi" - Falls back to native
            ❌ strategy="wasm" - Error (WASM not available)

        WASM-enabled server (--features wasm-extractor):
            ✅ strategy="native" - Works
            ✅ strategy="wasm" - Works
            ✅ strategy="multi" - Prefers WASM, falls back to native

    Migration from v0.8.x:
        The default strategy changed from "multi" to "native" in v0.9.0.
        To restore old behavior (use WASM if available):
            options = ExtractOptions(strategy="wasm")

        Or use multi-strategy auto-selection:
            options = ExtractOptions(strategy="multi")

    Example:
        >>> # Default: native extraction (fastest, recommended)
        >>> options = ExtractOptions()
        >>> result = await client.extract.extract(url, options=options)
        >>> print(result.strategy_used)  # "native"

        >>> # Explicit WASM (only if server supports it)
        >>> options = ExtractOptions(strategy="wasm")
        >>> try:
        ...     result = await client.extract.extract(url, options=options)
        ... except ExtractionError as e:
        ...     print(f"WASM not available: {e}")

        >>> # Auto-select (server decides based on availability)
        >>> options = ExtractOptions(strategy="multi")
        >>> result = await client.extract.extract(url, options=options)

    Note:
        WASM strategy requires the server to be built with the wasm-extractor
        feature flag and WASM_EXTRACTOR_PATH environment variable set. Most
        deployments use native extraction for better performance (4x faster).
    """
    strategy: str = "native"  # Changed from "multi" to "native" in v0.9.0
    quality_threshold: float = 0.7
    timeout_ms: int = 30000

    def to_dict(self) -> Dict[str, Any]:
        return {
            "strategy": self.strategy,
            "quality_threshold": self.quality_threshold,
            "timeout_ms": self.timeout_ms,
        }


@dataclass
class ContentMetadata:
    """Metadata for extracted content"""
    author: Optional[str] = None
    publish_date: Optional[str] = None
    word_count: int = 0
    language: Optional[str] = None

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'ContentMetadata':
        return cls(
            author=data.get('author'),
            publish_date=data.get('publish_date'),
            word_count=data.get('word_count', 0),
            language=data.get('language'),
        )


@dataclass
class ParserMetadata:
    """Parser metadata for observability"""
    parser_used: str
    confidence_score: float
    fallback_occurred: bool
    parse_time_ms: int
    extraction_path: Optional[str] = None
    primary_error: Optional[str] = None

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'ParserMetadata':
        return cls(
            parser_used=data['parser_used'],
            confidence_score=data['confidence_score'],
            fallback_occurred=data['fallback_occurred'],
            parse_time_ms=data['parse_time_ms'],
            extraction_path=data.get('extraction_path'),
            primary_error=data.get('primary_error'),
        )


@dataclass
class ExtractionResult:
    """Result of content extraction operation.

    Attributes:
        url: The URL that was extracted from.
        title: Extracted page title.
        content: Main content text extracted from the page.
        metadata: Additional metadata about the content (author, date, etc).
        strategy_used: Which extraction strategy was actually used ("native" or "wasm").
            This may differ from the requested strategy if fallback occurred.
        quality_score: Extraction quality score (0.0-1.0).
        extraction_time_ms: Time taken for extraction in milliseconds.
        parser_metadata: Optional detailed parser information for observability.

    Note:
        The strategy_used field indicates which extraction method was employed.
        The server may fall back to native even if WASM was requested, depending
        on server configuration and availability.

        In v0.9.0+, you'll typically see strategy_used="native" as that's the
        new default. If the server has WASM enabled and you request it explicitly,
        you'll see strategy_used="wasm".

    Example:
        >>> result = await client.extract.extract("https://example.com")
        >>> print(f"Used: {result.strategy_used}")  # "native" in v0.9.0+
        >>> print(f"Quality: {result.quality_score:.2f}")
        >>> print(f"Time: {result.extraction_time_ms}ms")
    """
    url: str
    title: Optional[str]
    content: str
    metadata: ContentMetadata
    strategy_used: str
    quality_score: float
    extraction_time_ms: int
    parser_metadata: Optional[ParserMetadata] = None

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'ExtractionResult':
        parser_metadata = None
        if data.get('parser_metadata'):
            parser_metadata = ParserMetadata.from_dict(data['parser_metadata'])

        return cls(
            url=data['url'],
            title=data.get('title'),
            content=data['content'],
            metadata=ContentMetadata.from_dict(data['metadata']),
            strategy_used=data['strategy_used'],
            quality_score=data['quality_score'],
            extraction_time_ms=data['extraction_time_ms'],
            parser_metadata=parser_metadata,
        )

    def to_summary(self) -> str:
        """
        Generate a human-readable summary of the extraction

        Returns:
            Formatted summary string
        """
        lines = [
            f"URL: {self.url}",
            f"Title: {self.title or 'N/A'}",
            f"Strategy: {self.strategy_used}",
            f"Quality: {self.quality_score:.2f}",
            f"Word Count: {self.metadata.word_count}",
            f"Extraction Time: {self.extraction_time_ms}ms",
        ]

        if self.metadata.author:
            lines.append(f"Author: {self.metadata.author}")
        if self.metadata.publish_date:
            lines.append(f"Published: {self.metadata.publish_date}")
        if self.metadata.language:
            lines.append(f"Language: {self.metadata.language}")

        if self.parser_metadata:
            lines.extend([
                f"Parser: {self.parser_metadata.parser_used}",
                f"Confidence: {self.parser_metadata.confidence_score:.2f}",
                f"Fallback: {'Yes' if self.parser_metadata.fallback_occurred else 'No'}",
            ])

        return "\n".join(lines)



# ============================================================================
# Search Models
# ============================================================================

@dataclass
class SearchOptions:
    """Options for search operations"""
    country: str = "us"
    language: str = "en"
    provider: Optional[str] = None

    def to_dict(self) -> Dict[str, Any]:
        data = {
            "country": self.country,
            "language": self.language,
        }
        if self.provider:
            data["provider"] = self.provider
        return data


@dataclass
class SearchResultItem:
    """Individual search result"""
    title: str
    url: str
    snippet: str
    position: int

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'SearchResultItem':
        return cls(
            title=data.get('title', ''),
            url=data['url'],
            snippet=data.get('snippet', ''),
            position=data.get('position', 0),
        )


@dataclass
class SearchResponse:
    """Response for search operations"""
    query: str
    results: List[SearchResultItem]
    total_results: int
    provider_used: str
    search_time_ms: int

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'SearchResponse':
        return cls(
            query=data['query'],
            results=[SearchResultItem.from_dict(r) for r in data['results']],
            total_results=data['total_results'],
            provider_used=data['provider_used'],
            search_time_ms=data['search_time_ms'],
        )


@dataclass
class SearchResult:
    """Search result item (legacy - for backward compatibility)"""
    url: str
    title: Optional[str] = None
    snippet: Optional[str] = None
    rank: Optional[int] = None

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'SearchResult':
        return cls(**data)


@dataclass
class DeepSearchResponse:
    """Response for deep search operations"""
    query: str
    total_results: int
    results: List[SearchResult]
    processing_time_ms: int

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'DeepSearchResponse':
        return cls(
            query=data['query'],
            total_results=data['total_results'],
            results=[SearchResult.from_dict(r) for r in data['results']],
            processing_time_ms=data['processing_time_ms'],
        )


# ============================================================================
# Spider Crawling Models (Phase 11 - Deep Crawling)
# ============================================================================

@dataclass
class SpiderConfig:
    """
    Configuration for spider crawling operations

    Provides fine-grained control over deep crawling behavior including
    depth limits, page budgets, crawling strategies, and rate limiting.
    """
    max_depth: Optional[int] = None
    max_pages: Optional[int] = None
    strategy: Optional[Literal["breadth_first", "depth_first", "best_first"]] = None
    timeout_seconds: Optional[int] = None
    delay_ms: Optional[int] = None
    concurrency: Optional[int] = None
    respect_robots: Optional[bool] = None
    follow_redirects: Optional[bool] = None

    def to_dict(self) -> Dict[str, Any]:
        """Convert to API request format"""
        data = {}
        if self.max_depth is not None:
            data["max_depth"] = self.max_depth
        if self.max_pages is not None:
            data["max_pages"] = self.max_pages
        if self.strategy:
            data["strategy"] = self.strategy
        if self.timeout_seconds is not None:
            data["timeout_seconds"] = self.timeout_seconds
        if self.delay_ms is not None:
            data["delay_ms"] = self.delay_ms
        if self.concurrency is not None:
            data["concurrency"] = self.concurrency
        if self.respect_robots is not None:
            data["respect_robots"] = self.respect_robots
        if self.follow_redirects is not None:
            data["follow_redirects"] = self.follow_redirects
        return data


@dataclass
class CrawlState:
    """Current state of spider crawl operation"""
    active: bool
    pages_crawled: int
    pages_failed: int
    frontier_size: int
    domains_seen: int

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'CrawlState':
        return cls(
            active=data['active'],
            pages_crawled=data['pages_crawled'],
            pages_failed=data['pages_failed'],
            frontier_size=data['frontier_size'],
            domains_seen=data['domains_seen'],
        )


@dataclass
class PerformanceMetrics:
    """Performance metrics for spider operations"""
    pages_per_second: float
    avg_response_time_ms: float
    memory_usage: int
    error_rate: float

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'PerformanceMetrics':
        # Handle duration format from Rust (e.g., {"secs": 0, "nanos": 150000000})
        avg_response = data.get('avg_response_time', {})
        if isinstance(avg_response, dict):
            secs = avg_response.get('secs', 0)
            nanos = avg_response.get('nanos', 0)
            avg_response_ms = (secs * 1000.0) + (nanos / 1_000_000.0)
        else:
            avg_response_ms = float(avg_response) if avg_response else 0.0

        return cls(
            pages_per_second=data['pages_per_second'],
            avg_response_time_ms=avg_response_ms,
            memory_usage=data['memory_usage'],
            error_rate=data['error_rate'],
        )


@dataclass
class FrontierMetrics:
    """Metrics for URL frontier management"""
    total_requests: int
    average_depth: float
    memory_usage: int

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'FrontierMetrics':
        return cls(
            total_requests=data['total_requests'],
            average_depth=data['average_depth'],
            memory_usage=data['memory_usage'],
        )


@dataclass
class AdaptiveStopStats:
    """Statistics for adaptive stopping mechanism"""
    similarity_threshold: float
    consecutive_similar: int
    total_comparisons: int

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'AdaptiveStopStats':
        return cls(
            similarity_threshold=data['similarity_threshold'],
            consecutive_similar=data['consecutive_similar'],
            total_comparisons=data['total_comparisons'],
        )


@dataclass
class SpiderApiResult:
    """Results from spider crawl operation"""
    pages_crawled: int
    pages_failed: int
    duration_seconds: float
    stop_reason: str
    domains: List[str]

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'SpiderApiResult':
        return cls(
            pages_crawled=data['pages_crawled'],
            pages_failed=data['pages_failed'],
            duration_seconds=data['duration_seconds'],
            stop_reason=data['stop_reason'],
            domains=data['domains'],
        )

    def to_summary(self) -> str:
        """Generate a human-readable summary"""
        return (
            f"Spider Crawl Summary:\n"
            f"  Pages Crawled: {self.pages_crawled}\n"
            f"  Pages Failed: {self.pages_failed}\n"
            f"  Duration: {self.duration_seconds:.2f}s\n"
            f"  Stop Reason: {self.stop_reason}\n"
            f"  Domains: {', '.join(self.domains)}"
        )


@dataclass
class SpiderResult:
    """Complete spider crawl result with state and metrics"""
    result: SpiderApiResult
    state: CrawlState
    performance: PerformanceMetrics
    discovered_urls: Optional[List[str]] = None

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'SpiderResult':
        return cls(
            result=SpiderApiResult.from_dict(data['result']),
            state=CrawlState.from_dict(data['state']),
            performance=PerformanceMetrics.from_dict(data['performance']),
            discovered_urls=data.get('discovered_urls'),
        )

    @property
    def pages_crawled(self) -> int:
        """Convenience accessor for pages crawled"""
        return self.result.pages_crawled

    @property
    def pages_failed(self) -> int:
        """Convenience accessor for pages failed"""
        return self.result.pages_failed

    @property
    def duration_seconds(self) -> float:
        """Convenience accessor for duration"""
        return self.result.duration_seconds

    @property
    def stop_reason(self) -> str:
        """Convenience accessor for stop reason"""
        return self.result.stop_reason

    @property
    def domains(self) -> List[str]:
        """Convenience accessor for domains"""
        return self.result.domains

    def to_summary(self) -> str:
        """Generate a comprehensive summary"""
        summary = self.result.to_summary()
        summary += f"\n\nCurrent State:\n"
        summary += f"  Active: {self.state.active}\n"
        summary += f"  Frontier Size: {self.state.frontier_size}\n"
        summary += f"  Domains Seen: {self.state.domains_seen}\n"
        summary += f"\nPerformance:\n"
        summary += f"  Pages/Second: {self.performance.pages_per_second:.2f}\n"
        summary += f"  Avg Response Time: {self.performance.avg_response_time_ms:.2f}ms\n"
        summary += f"  Error Rate: {self.performance.error_rate:.2%}"
        return summary


@dataclass
class SpiderStatus:
    """Spider status with optional detailed metrics"""
    state: CrawlState
    performance: Optional[PerformanceMetrics] = None
    frontier_stats: Optional[FrontierMetrics] = None
    adaptive_stop_stats: Optional[AdaptiveStopStats] = None

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'SpiderStatus':
        performance = None
        if data.get('performance'):
            performance = PerformanceMetrics.from_dict(data['performance'])

        frontier_stats = None
        if data.get('frontier_stats'):
            frontier_stats = FrontierMetrics.from_dict(data['frontier_stats'])

        adaptive_stop_stats = None
        if data.get('adaptive_stop_stats'):
            adaptive_stop_stats = AdaptiveStopStats.from_dict(data['adaptive_stop_stats'])

        return cls(
            state=CrawlState.from_dict(data['state']),
            performance=performance,
            frontier_stats=frontier_stats,
            adaptive_stop_stats=adaptive_stop_stats,
        )

    def to_summary(self) -> str:
        """Generate a summary of spider status"""
        summary = f"Spider Status:\n"
        summary += f"  Active: {self.state.active}\n"
        summary += f"  Pages Crawled: {self.state.pages_crawled}\n"
        summary += f"  Pages Failed: {self.state.pages_failed}\n"
        summary += f"  Frontier Size: {self.state.frontier_size}\n"
        summary += f"  Domains Seen: {self.state.domains_seen}"

        if self.performance:
            summary += f"\n\nPerformance:\n"
            summary += f"  Pages/Second: {self.performance.pages_per_second:.2f}\n"
            summary += f"  Avg Response Time: {self.performance.avg_response_time_ms:.2f}ms\n"
            summary += f"  Memory Usage: {self.performance.memory_usage:,} bytes\n"
            summary += f"  Error Rate: {self.performance.error_rate:.2%}"

        if self.frontier_stats:
            summary += f"\n\nFrontier:\n"
            summary += f"  Total Requests: {self.frontier_stats.total_requests}\n"
            summary += f"  Average Depth: {self.frontier_stats.average_depth:.2f}\n"
            summary += f"  Memory Usage: {self.frontier_stats.memory_usage:,} bytes"

        if self.adaptive_stop_stats:
            summary += f"\n\nAdaptive Stop:\n"
            summary += f"  Similarity Threshold: {self.adaptive_stop_stats.similarity_threshold:.2f}\n"
            summary += f"  Consecutive Similar: {self.adaptive_stop_stats.consecutive_similar}\n"
            summary += f"  Total Comparisons: {self.adaptive_stop_stats.total_comparisons}"

        return summary


@dataclass
class SpiderControlResponse:
    """Response from spider control operations"""
    status: str

    def __str__(self) -> str:
        return f"Spider {self.status}"


# ============================================================================
# Worker/Job Models
# ============================================================================

class JobPriority(str, Enum):
    """Job priority levels"""
    LOW = "low"
    NORMAL = "normal"
    HIGH = "high"
    CRITICAL = "critical"


class JobStatus(str, Enum):
    """Job status values"""
    PENDING = "pending"
    DELAYED = "delayed"
    PROCESSING = "processing"
    COMPLETED = "completed"
    FAILED = "failed"
    RETRY = "retry"
    DEAD_LETTER = "dead_letter"


@dataclass
class RetryConfig:
    """Retry configuration for jobs"""
    max_attempts: int = 3
    initial_delay_secs: int = 1
    backoff_multiplier: float = 2.0
    max_delay_secs: int = 300
    use_jitter: bool = True

    def to_dict(self) -> Dict[str, Any]:
        return {
            "max_attempts": self.max_attempts,
            "initial_delay_secs": self.initial_delay_secs,
            "backoff_multiplier": self.backoff_multiplier,
            "max_delay_secs": self.max_delay_secs,
            "use_jitter": self.use_jitter,
        }


@dataclass
class JobType:
    """Base class for job types"""
    type: str
    data: Dict[str, Any]

    @staticmethod
    def batch_crawl(urls: List[str], options: Optional[CrawlOptions] = None) -> 'JobType':
        """Create a batch crawl job type"""
        data = {"urls": urls}
        if options:
            data["options"] = options.to_dict()
        return JobType(type="batch_crawl", data=data)

    @staticmethod
    def single_crawl(url: str, options: Optional[CrawlOptions] = None) -> 'JobType':
        """Create a single crawl job type"""
        data = {"url": url}
        if options:
            data["options"] = options.to_dict()
        return JobType(type="single_crawl", data=data)

    @staticmethod
    def maintenance(task_type: str, parameters: Dict[str, Any]) -> 'JobType':
        """Create a maintenance job type"""
        return JobType(
            type="maintenance",
            data={"task_type": task_type, "parameters": parameters}
        )

    @staticmethod
    def custom(job_name: str, payload: Any) -> 'JobType':
        """Create a custom job type"""
        return JobType(type="custom", data={"job_name": job_name, "payload": payload})

    def to_dict(self) -> Dict[str, Any]:
        return {"type": self.type, **self.data}


@dataclass
class JobConfig:
    """Configuration for job submission"""
    job_type: JobType
    priority: JobPriority = JobPriority.NORMAL
    retry_config: Optional[RetryConfig] = None
    metadata: Optional[Dict[str, Any]] = None
    scheduled_at: Optional[str] = None
    timeout_secs: Optional[int] = None

    def to_dict(self) -> Dict[str, Any]:
        data = {"job_type": self.job_type.to_dict()}
        if self.priority != JobPriority.NORMAL:
            data["priority"] = self.priority.value
        if self.retry_config:
            data["retry_config"] = self.retry_config.to_dict()
        if self.metadata:
            data["metadata"] = self.metadata
        if self.scheduled_at:
            data["scheduled_at"] = self.scheduled_at
        if self.timeout_secs:
            data["timeout_secs"] = self.timeout_secs
        return data


@dataclass
class Job:
    """Job object"""
    job_id: str
    status: JobStatus
    created_at: str
    started_at: Optional[str] = None
    completed_at: Optional[str] = None
    worker_id: Optional[str] = None
    retry_count: int = 0
    last_error: Optional[str] = None
    processing_time_ms: Optional[int] = None
    metadata: Dict[str, Any] = field(default_factory=dict)

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'Job':
        return cls(
            job_id=data['job_id'],
            status=JobStatus(data['status']),
            created_at=data['created_at'],
            started_at=data.get('started_at'),
            completed_at=data.get('completed_at'),
            worker_id=data.get('worker_id'),
            retry_count=data.get('retry_count', 0),
            last_error=data.get('last_error'),
            processing_time_ms=data.get('processing_time_ms'),
            metadata=data.get('metadata', {}),
        )


@dataclass
class JobResult:
    """Job execution result"""
    job_id: str
    success: bool
    data: Optional[Any] = None
    error: Optional[str] = None
    processing_time_ms: int = 0
    worker_id: str = ""
    completed_at: str = ""

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'JobResult':
        return cls(
            job_id=data['job_id'],
            success=data['success'],
            data=data.get('data'),
            error=data.get('error'),
            processing_time_ms=data.get('processing_time_ms', 0),
            worker_id=data.get('worker_id', ''),
            completed_at=data.get('completed_at', ''),
        )


@dataclass
class QueueStats:
    """Queue statistics"""
    pending: int
    processing: int
    completed: int
    failed: int
    retry: int
    delayed: int
    total: int

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'QueueStats':
        return cls(
            pending=data['pending'],
            processing=data['processing'],
            completed=data['completed'],
            failed=data['failed'],
            retry=data['retry'],
            delayed=data['delayed'],
            total=data['total'],
        )


@dataclass
class WorkerStats:
    """Worker pool statistics"""
    total_workers: int
    healthy_workers: int
    total_jobs_processed: int
    total_jobs_failed: int
    is_running: bool

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'WorkerStats':
        return cls(
            total_workers=data['total_workers'],
            healthy_workers=data['healthy_workers'],
            total_jobs_processed=data['total_jobs_processed'],
            total_jobs_failed=data['total_jobs_failed'],
            is_running=data['is_running'],
        )


@dataclass
class ScheduledJobConfig:
    """Configuration for scheduled job creation"""
    name: str
    cron_expression: str
    job_template: JobType
    priority: JobPriority = JobPriority.NORMAL
    enabled: bool = True
    retry_config: Optional[RetryConfig] = None
    metadata: Optional[Dict[str, Any]] = None

    def to_dict(self) -> Dict[str, Any]:
        data = {
            "name": self.name,
            "cron_expression": self.cron_expression,
            "job_template": self.job_template.to_dict(),
        }
        if self.priority != JobPriority.NORMAL:
            data["priority"] = self.priority.value
        if not self.enabled:
            data["enabled"] = self.enabled
        if self.retry_config:
            data["retry_config"] = self.retry_config.to_dict()
        if self.metadata:
            data["metadata"] = self.metadata
        return data


@dataclass
class ScheduledJob:
    """Scheduled job object"""
    id: str
    name: str
    cron_expression: str
    enabled: bool
    priority: JobPriority
    created_at: str
    last_executed_at: Optional[str] = None
    next_execution_at: Optional[str] = None
    execution_count: int = 0

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'ScheduledJob':
        return cls(
            id=data['id'],
            name=data['name'],
            cron_expression=data['cron_expression'],
            enabled=data['enabled'],
            priority=JobPriority(data['priority']),
            created_at=data['created_at'],
            last_executed_at=data.get('last_executed_at'),
            next_execution_at=data.get('next_execution_at'),
            execution_count=data.get('execution_count', 0),
        )


@dataclass
class JobListItem:
    """Item in job list response"""
    job_id: str
    job_type: str
    status: JobStatus
    priority: JobPriority
    created_at: str
    started_at: Optional[str] = None
    completed_at: Optional[str] = None
    worker_id: Optional[str] = None
    retry_count: int = 0

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'JobListItem':
        return cls(
            job_id=data['job_id'],
            job_type=data['job_type'],
            status=JobStatus(data['status']),
            priority=JobPriority(data['priority']),
            created_at=data['created_at'],
            started_at=data.get('started_at'),
            completed_at=data.get('completed_at'),
            worker_id=data.get('worker_id'),
            retry_count=data.get('retry_count', 0),
        )


@dataclass
class JobListResponse:
    """Response for job listing"""
    jobs: List[JobListItem]
    total: int
    limit: int
    offset: int

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'JobListResponse':
        return cls(
            jobs=[JobListItem.from_dict(j) for j in data['jobs']],
            total=data['total'],
            limit=data['limit'],
            offset=data['offset'],
        )


# ============================================================================
# PDF Processing Models
# ============================================================================

@dataclass
class PdfExtractionOptions:
    """Options for PDF extraction"""
    extract_text: bool = True
    extract_metadata: bool = True
    extract_images: bool = False
    include_page_numbers: bool = True

    def to_dict(self) -> Dict[str, Any]:
        return asdict(self)


@dataclass
class PdfCapabilities:
    """PDF processing capabilities"""
    text_extraction: bool
    image_extraction: bool
    metadata_extraction: bool
    table_extraction: bool
    form_extraction: bool
    encrypted_pdfs: bool
    max_file_size_mb: int
    supported_versions: List[str]

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'PdfCapabilities':
        return cls(**data)


@dataclass
class PdfFeatures:
    """PDF processing features"""
    progress_streaming: bool
    concurrent_processing: bool
    memory_monitoring: bool
    performance_metrics: bool

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'PdfFeatures':
        return cls(**data)


@dataclass
class PdfProcessingStats:
    """Statistics for PDF processing"""
    processing_time_ms: int
    file_size: int
    pages_processed: int
    memory_used: int
    pages_per_second: float
    progress_overhead_us: Optional[int] = None

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'PdfProcessingStats':
        return cls(**data)


@dataclass
class ExtractedDocument:
    """Extracted document from PDF"""
    url: Optional[str] = None
    title: Optional[str] = None
    text: Optional[str] = None
    quality_score: Optional[int] = None
    links: Optional[List[str]] = None
    byline: Optional[str] = None
    published_iso: Optional[str] = None
    markdown: Optional[str] = None
    media: Optional[List[str]] = None
    language: Optional[str] = None
    reading_time: Optional[int] = None
    word_count: Optional[int] = None
    categories: Optional[List[str]] = None
    site_name: Optional[str] = None
    parser_metadata: Optional[Dict[str, Any]] = None
    description: Optional[str] = None
    html: Optional[str] = None

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'ExtractedDocument':
        return cls(**data)


@dataclass
class PdfExtractionResult:
    """Result from PDF extraction"""
    success: bool
    document: Optional[ExtractedDocument] = None
    error: Optional[str] = None
    stats: Optional[PdfProcessingStats] = None

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'PdfExtractionResult':
        document = None
        if data.get('document'):
            document = ExtractedDocument.from_dict(data['document'])

        stats = None
        if data.get('stats'):
            stats = PdfProcessingStats.from_dict(data['stats'])

        return cls(
            success=data['success'],
            document=document,
            error=data.get('error'),
            stats=stats,
        )


@dataclass
class PdfStreamProgress:
    """Progress update from streaming PDF extraction"""
    event_type: str
    current_page: Optional[int] = None
    total_pages: Optional[int] = None
    percentage: Optional[float] = None
    estimated_remaining_ms: Optional[int] = None
    stage: Optional[str] = None
    pages_per_second: Optional[float] = None
    average_progress_overhead_us: Optional[int] = None
    memory_usage_mb: Optional[float] = None
    document: Optional[ExtractedDocument] = None
    error: Optional[str] = None

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'PdfStreamProgress':
        # Handle different event types
        event_type = "unknown"
        if "Progress" in data:
            event_type = "progress"
            progress_data = data["Progress"]
            return cls(
                event_type=event_type,
                current_page=progress_data.get("current_page"),
                total_pages=progress_data.get("total_pages"),
                percentage=progress_data.get("percentage"),
                estimated_remaining_ms=progress_data.get("estimated_remaining_ms"),
                stage=str(progress_data.get("stage", "")),
                pages_per_second=data.get("pages_per_second"),
                average_progress_overhead_us=data.get("average_progress_overhead_us"),
                memory_usage_mb=data.get("memory_usage_mb"),
            )
        elif "Completed" in data:
            event_type = "completed"
            completed_data = data["Completed"]
            document = None
            if completed_data.get("document"):
                document = ExtractedDocument.from_dict(completed_data["document"])
            return cls(
                event_type=event_type,
                document=document,
                pages_per_second=data.get("pages_per_second"),
            )
        elif "Failed" in data:
            event_type = "failed"
            failed_data = data["Failed"]
            return cls(
                event_type=event_type,
                error=failed_data.get("error"),
            )
        elif "KeepAlive" in data:
            return cls(event_type="keepalive")
        else:
            # Flat structure for enhanced updates
            document = None
            if data.get("document"):
                document = ExtractedDocument.from_dict(data["document"])
            return cls(
                event_type=data.get("type", "unknown"),
                current_page=data.get("current_page"),
                total_pages=data.get("total_pages"),
                percentage=data.get("percentage"),
                estimated_remaining_ms=data.get("estimated_remaining_ms"),
                stage=data.get("stage"),
                pages_per_second=data.get("pages_per_second"),
                average_progress_overhead_us=data.get("average_progress_overhead_us"),
                memory_usage_mb=data.get("memory_usage_mb"),
                document=document,
                error=data.get("error"),
            )


@dataclass
class PdfJobStatus:
    """Status of an asynchronous PDF extraction job"""
    job_id: str
    status: Literal["pending", "processing", "completed", "failed"]
    created_at: str
    updated_at: str
    result: Optional[PdfExtractionResult] = None
    error: Optional[str] = None
    progress: Optional[float] = None

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'PdfJobStatus':
        result = None
        if data.get('result'):
            result = PdfExtractionResult.from_dict(data['result'])

        return cls(
            job_id=data['job_id'],
            status=data['status'],
            created_at=data['created_at'],
            updated_at=data['updated_at'],
            result=result,
            error=data.get('error'),
            progress=data.get('progress'),
        )


@dataclass
class PdfMetrics:
    """PDF processing metrics and capabilities"""
    status: str
    pdf_processing_available: bool
    capabilities: PdfCapabilities
    features: PdfFeatures

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'PdfMetrics':
        return cls(
            status=data['status'],
            pdf_processing_available=data['pdf_processing_available'],
            capabilities=PdfCapabilities.from_dict(data['capabilities']),
            features=PdfFeatures.from_dict(data['features']),
        )


# ============================================================================
# Browser Automation Models
# ============================================================================

@dataclass
class BrowserSessionConfig:
    """Configuration for creating a browser session"""
    stealth_preset: Optional[Literal["none", "low", "medium", "high"]] = None
    initial_url: Optional[str] = None
    timeout_secs: Optional[int] = None

    def to_dict(self) -> Dict[str, Any]:
        data = {}
        if self.stealth_preset is not None:
            data["stealth_preset"] = self.stealth_preset
        if self.initial_url is not None:
            data["initial_url"] = self.initial_url
        if self.timeout_secs is not None:
            data["timeout_secs"] = self.timeout_secs
        return data


@dataclass
class PoolStatusInfo:
    """Browser pool status information"""
    available: int
    in_use: int
    total_capacity: int
    utilization_percent: float

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'PoolStatusInfo':
        return cls(
            available=data['available'],
            in_use=data['in_use'],
            total_capacity=data['total_capacity'],
            utilization_percent=data['utilization_percent'],
        )


@dataclass
class LauncherStatsInfo:
    """Browser launcher statistics"""
    total_requests: int
    successful_requests: int
    failed_requests: int
    avg_response_time_ms: float
    stealth_requests: int
    non_stealth_requests: int

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'LauncherStatsInfo':
        return cls(
            total_requests=data['total_requests'],
            successful_requests=data['successful_requests'],
            failed_requests=data['failed_requests'],
            avg_response_time_ms=data['avg_response_time_ms'],
            stealth_requests=data['stealth_requests'],
            non_stealth_requests=data['non_stealth_requests'],
        )

    @property
    def success_rate(self) -> float:
        """Calculate success rate as a decimal (0-1)"""
        if self.total_requests == 0:
            return 0.0
        return self.successful_requests / self.total_requests


@dataclass
class BrowserSession:
    """Browser session information"""
    session_id: str
    pool_stats: PoolStatusInfo
    created_at: str
    expires_at: str

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'BrowserSession':
        return cls(
            session_id=data['session_id'],
            pool_stats=PoolStatusInfo.from_dict(data['pool_stats']),
            created_at=data['created_at'],
            expires_at=data['expires_at'],
        )


@dataclass
class BrowserAction:
    """Browser action to execute"""
    action_type: str
    url: Optional[str] = None
    wait_for_load: Optional[bool] = None
    script: Optional[str] = None
    timeout_ms: Optional[int] = None
    full_page: Optional[bool] = None
    selector: Optional[str] = None
    text: Optional[str] = None
    landscape: Optional[bool] = None
    print_background: Optional[bool] = None

    @staticmethod
    def navigate(
        session_id: str,
        url: str,
        wait_for_load: bool = True,
    ) -> 'BrowserAction':
        """Create a navigate action"""
        action = BrowserAction(action_type="navigate")
        action.url = url
        action.wait_for_load = wait_for_load
        return action

    @staticmethod
    def execute_script(
        session_id: str,
        script: str,
        timeout_ms: Optional[int] = None,
    ) -> 'BrowserAction':
        """Create an execute script action"""
        action = BrowserAction(action_type="execute_script")
        action.script = script
        action.timeout_ms = timeout_ms
        return action

    @staticmethod
    def screenshot(
        session_id: str,
        full_page: bool = False,
    ) -> 'BrowserAction':
        """Create a screenshot action"""
        action = BrowserAction(action_type="screenshot")
        action.full_page = full_page
        return action

    @staticmethod
    def get_content(session_id: str) -> 'BrowserAction':
        """Create a get content action"""
        return BrowserAction(action_type="get_content")

    @staticmethod
    def wait_for_element(
        session_id: str,
        selector: str,
        timeout_ms: int = 5000,
    ) -> 'BrowserAction':
        """Create a wait for element action"""
        action = BrowserAction(action_type="wait_for_element")
        action.selector = selector
        action.timeout_ms = timeout_ms
        return action

    @staticmethod
    def click(session_id: str, selector: str) -> 'BrowserAction':
        """Create a click action"""
        action = BrowserAction(action_type="click")
        action.selector = selector
        return action

    @staticmethod
    def type_text(
        session_id: str,
        selector: str,
        text: str,
    ) -> 'BrowserAction':
        """Create a type text action"""
        action = BrowserAction(action_type="type_text")
        action.selector = selector
        action.text = text
        return action

    @staticmethod
    def render_pdf(
        session_id: str,
        landscape: bool = False,
        print_background: bool = False,
    ) -> 'BrowserAction':
        """Create a render PDF action"""
        action = BrowserAction(action_type="render_pdf")
        action.landscape = landscape
        action.print_background = print_background
        return action

    def to_dict(self) -> Dict[str, Any]:
        """Convert to API request format"""
        data = {"action_type": self.action_type}
        if self.url is not None:
            data["url"] = self.url
        if self.wait_for_load is not None:
            data["wait_for_load"] = self.wait_for_load
        if self.script is not None:
            data["script"] = self.script
        if self.timeout_ms is not None:
            data["timeout_ms"] = self.timeout_ms
        if self.full_page is not None:
            data["full_page"] = self.full_page
        if self.selector is not None:
            data["selector"] = self.selector
        if self.text is not None:
            data["text"] = self.text
        if self.landscape is not None:
            data["landscape"] = self.landscape
        if self.print_background is not None:
            data["print_background"] = self.print_background
        return data


@dataclass
class BrowserActionResult:
    """Result from browser action execution"""
    success: bool
    result: Dict[str, Any]
    duration_ms: int
    messages: List[str]

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'BrowserActionResult':
        return cls(
            success=data['success'],
            result=data['result'],
            duration_ms=data['duration_ms'],
            messages=data['messages'],
        )


@dataclass
class BrowserPoolStatus:
    """Browser pool status with detailed metrics"""
    stats: PoolStatusInfo
    launcher_stats: LauncherStatsInfo
    health: str

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'BrowserPoolStatus':
        return cls(
            stats=PoolStatusInfo.from_dict(data['stats']),
            launcher_stats=LauncherStatsInfo.from_dict(data['launcher_stats']),
            health=data['health'],
        )

    def to_summary(self) -> str:
        """Generate a human-readable summary"""
        return (
            f"Browser Pool Status:\n"
            f"  Health: {self.health}\n"
            f"  Available: {self.stats.available}\n"
            f"  In Use: {self.stats.in_use}\n"
            f"  Total Capacity: {self.stats.total_capacity}\n"
            f"  Utilization: {self.stats.utilization_percent:.1f}%\n"
            f"\n"
            f"Launcher Stats:\n"
            f"  Total Requests: {self.launcher_stats.total_requests}\n"
            f"  Successful: {self.launcher_stats.successful_requests}\n"
            f"  Failed: {self.launcher_stats.failed_requests}\n"
            f"  Success Rate: {self.launcher_stats.success_rate:.1%}\n"
            f"  Avg Response Time: {self.launcher_stats.avg_response_time_ms:.2f}ms\n"
            f"  Stealth Requests: {self.launcher_stats.stealth_requests}\n"
            f"  Non-Stealth Requests: {self.launcher_stats.non_stealth_requests}"
        )


# ============================================================================
# Session Management Models
# ============================================================================

@dataclass
class SessionConfig:
    """Configuration for creating a session"""
    ttl_seconds: Optional[int] = None

    def to_dict(self) -> Dict[str, Any]:
        data = {}
        if self.ttl_seconds is not None:
            data["ttl_seconds"] = self.ttl_seconds
        return data


@dataclass
class Session:
    """Browser session with persistent state"""
    session_id: str
    user_data_dir: str
    created_at: str
    last_accessed: Optional[str] = None
    expires_at: Optional[str] = None
    cookie_count: Optional[int] = None
    total_domains: Optional[int] = None

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'Session':
        """Create Session from API response"""
        return cls(
            session_id=data['session_id'],
            user_data_dir=data['user_data_dir'],
            created_at=data['created_at'],
            last_accessed=data.get('last_accessed'),
            expires_at=data.get('expires_at'),
            cookie_count=data.get('cookie_count'),
            total_domains=data.get('total_domains'),
        )


@dataclass
class SessionStats:
    """Statistics for session management"""
    total_sessions: int
    expired_sessions_cleaned: int
    last_cleanup_time: Optional[str] = None

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'SessionStats':
        """Create SessionStats from API response"""
        return cls(
            total_sessions=data['total_sessions'],
            expired_sessions_cleaned=data['expired_sessions_cleaned'],
            last_cleanup_time=data.get('last_cleanup_time'),
        )


@dataclass
class Cookie:
    """HTTP cookie"""
    name: str
    value: str
    domain: Optional[str] = None
    path: Optional[str] = None
    expires: Optional[str] = None
    secure: bool = False
    http_only: bool = False

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'Cookie':
        """Create Cookie from API response"""
        return cls(
            name=data['name'],
            value=data['value'],
            domain=data.get('domain'),
            path=data.get('path'),
            expires=data.get('expires'),
            secure=data.get('secure', False),
            http_only=data.get('http_only', False),
        )


@dataclass
class SetCookieRequest:
    """Request body for setting a cookie"""
    domain: str
    name: str
    value: str
    path: Optional[str] = None
    expires_in_seconds: Optional[int] = None
    secure: Optional[bool] = None
    http_only: Optional[bool] = None

    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary for API request"""
        data = {
            "domain": self.domain,
            "name": self.name,
            "value": self.value,
        }
        if self.path is not None:
            data["path"] = self.path
        if self.expires_in_seconds is not None:
            data["expires_in_seconds"] = self.expires_in_seconds
        if self.secure is not None:
            data["secure"] = self.secure
        if self.http_only is not None:
            data["http_only"] = self.http_only
        return data
