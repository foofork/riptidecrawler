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
# Search Models
# ============================================================================

@dataclass
class SearchResult:
    """Search result item"""
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
