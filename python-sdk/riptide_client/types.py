"""Type definitions for RipTide client."""

from typing import TypedDict, Optional, List, Dict, Any


class CrawlOptions(TypedDict, total=False):
    """Options for crawl requests."""

    concurrency: int
    cache_mode: str  # 'auto', 'read_write', 'read_only', 'write_only', 'disabled'
    use_spider: bool
    chunking_config: Optional[Dict[str, Any]]
    extract_mode: Optional[str]  # 'article', 'full', etc.


class SearchOptions(TypedDict, total=False):
    """Options for search requests."""

    limit: int
    include_content: bool
    crawl_options: Optional[CrawlOptions]


class SessionConfig(TypedDict, total=False):
    """Session configuration."""

    user_agent: str
    cookies: List[Dict[str, str]]
    headers: Dict[str, str]
