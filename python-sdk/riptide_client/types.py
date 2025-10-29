"""Type definitions for RipTide client."""

from typing import TypedDict, Optional, List, Dict, Any, Literal


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


# Phase 2: Spider Types

class CrawledPage(TypedDict, total=False):
    """Represents a single crawled page with extracted content.

    Attributes:
        url: The final URL after redirects
        depth: Crawl depth from seed URL
        status_code: HTTP status code
        title: Page title
        content: Raw HTML/text content (optional, use include parameter)
        markdown: Normalized markdown content (optional, use include parameter)
        links: List of discovered links on the page
        canonical_url: Canonical URL if found
        mime: MIME type of the response
        charset: Character encoding
        fetch_time_ms: Time taken to fetch the page in milliseconds
        robots_obeyed: Whether robots.txt was respected
        disallowed: Whether the page was disallowed by robots.txt
        fetch_error: Error message if fetch failed
        parse_error: Error message if parsing failed
        truncated: Whether content was truncated due to size limits
    """

    url: str
    depth: int
    status_code: int
    title: Optional[str]
    content: Optional[str]
    markdown: Optional[str]
    links: List[str]
    canonical_url: Optional[str]
    mime: Optional[str]
    charset: Optional[str]
    fetch_time_ms: Optional[float]
    robots_obeyed: Optional[bool]
    disallowed: Optional[bool]
    fetch_error: Optional[str]
    parse_error: Optional[str]
    truncated: Optional[bool]


class SpiderResultStats(TypedDict):
    """Spider result with only statistics (lightweight)."""

    pages_crawled: int
    pages_failed: int
    duration_seconds: float
    stop_reason: str


class SpiderResultUrls(TypedDict):
    """Spider result with discovered URLs list."""

    pages_crawled: int
    pages_failed: int
    duration_seconds: float
    stop_reason: str
    domains: List[str]
    discovered_urls: List[str]


class SpiderResultPages(TypedDict):
    """Spider result with full page objects."""

    pages_crawled: int
    pages_failed: int
    duration_seconds: float
    stop_reason: str
    pages: List[CrawledPage]
    api_version: Optional[str]


class SpiderJobResponse(TypedDict):
    """Response when creating a stored spider job."""

    job_id: str


class SpiderOptions(TypedDict, total=False):
    """Options for spider requests.

    Attributes:
        result_mode: Output format ('stats', 'urls', 'pages', 'stream', 'store')
        max_pages: Maximum number of pages to crawl
        max_depth: Maximum crawl depth
        include: Comma-separated fields to include (e.g., 'title,links,markdown')
        exclude: Comma-separated fields to exclude (e.g., 'content')
        scope: Crawl scope restrictions
        max_content_bytes: Maximum content size per page
    """

    result_mode: Literal['stats', 'urls', 'pages', 'stream', 'store']
    max_pages: int
    max_depth: int
    include: str
    exclude: str
    scope: Optional[Dict[str, Any]]
    max_content_bytes: Optional[int]


class JobResultsResponse(TypedDict):
    """Response from paginated job results."""

    pages: List[CrawledPage]
    next_cursor: Optional[str]
    done: bool


class ExtractBatchRequest(TypedDict):
    """Request for batch extraction."""

    urls: List[str]
    format: Literal['markdown', 'html', 'text']


class ExtractBatchResult(TypedDict):
    """Single result from batch extraction."""

    url: str
    markdown: Optional[str]
    html: Optional[str]
    text: Optional[str]
    metadata: Optional[Dict[str, Any]]
    error: Optional[str]
