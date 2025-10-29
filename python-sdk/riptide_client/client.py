"""RipTide API Client implementation."""

from typing import Any, Dict, List, Optional, Iterator
import requests
from requests.adapters import HTTPAdapter
from urllib3.util.retry import Retry

from .exceptions import APIError, RateLimitError, TimeoutError, ValidationError
from .types import (
    CrawlOptions,
    SearchOptions,
    SessionConfig,
    SpiderOptions,
    SpiderResultStats,
    SpiderResultUrls,
    SpiderResultPages,
    SpiderJobResponse,
    JobResultsResponse,
    ExtractBatchRequest,
    ExtractBatchResult,
)


class RipTide:
    """RipTide API Client.

    Args:
        base_url: Base URL of the RipTide API (e.g., 'http://localhost:8080')
        api_key: Optional API key for authentication
        timeout: Default timeout for requests in seconds (default: 30)
        max_retries: Maximum number of retries for failed requests (default: 3)
    """

    def __init__(
        self,
        base_url: str = "http://localhost:8080",
        api_key: Optional[str] = None,
        timeout: int = 30,
        max_retries: int = 3,
    ):
        self.base_url = base_url.rstrip("/")
        self.api_key = api_key
        self.timeout = timeout

        # Setup session with retry logic
        self.session = requests.Session()
        retry_strategy = Retry(
            total=max_retries,
            backoff_factor=1,
            status_forcelist=[429, 500, 502, 503, 504],
            allowed_methods=["GET", "POST"],
        )
        adapter = HTTPAdapter(max_retries=retry_strategy)
        self.session.mount("http://", adapter)
        self.session.mount("https://", adapter)

        if api_key:
            self.session.headers.update({"Authorization": f"Bearer {api_key}"})

    def _request(
        self,
        method: str,
        endpoint: str,
        data: Optional[Dict[str, Any]] = None,
        params: Optional[Dict[str, Any]] = None,
        stream: bool = False,
    ) -> Any:
        """Make HTTP request to API."""
        url = f"{self.base_url}{endpoint}"

        try:
            response = self.session.request(
                method=method,
                url=url,
                json=data,
                params=params,
                timeout=self.timeout,
                stream=stream,
            )

            if response.status_code == 429:
                raise RateLimitError("Rate limit exceeded")
            elif response.status_code >= 400:
                error_data = response.json() if response.content else {}
                raise APIError(
                    f"API error {response.status_code}: {error_data.get('error', response.text)}"
                )

            if stream:
                return response

            return response.json() if response.content else {}

        except requests.exceptions.Timeout:
            raise TimeoutError(f"Request to {endpoint} timed out after {self.timeout}s")
        except requests.exceptions.RequestException as e:
            raise APIError(f"Request failed: {str(e)}")

    def health(self) -> Dict[str, Any]:
        """Check API health status.

        Returns:
            Health status including dependencies and metrics
        """
        return self._request("GET", "/healthz")

    def metrics(self) -> str:
        """Get Prometheus metrics.

        Returns:
            Prometheus-formatted metrics as string
        """
        response = self.session.get(f"{self.base_url}/metrics", timeout=self.timeout)
        return response.text

    def crawl(
        self,
        urls: List[str],
        options: Optional[CrawlOptions] = None,
        session_id: Optional[str] = None,
    ) -> Dict[str, Any]:
        """Crawl one or more URLs.

        Args:
            urls: List of URLs to crawl
            options: Crawl options (concurrency, cache mode, etc.)
            session_id: Optional session ID for stateful crawling

        Returns:
            Crawl results with extracted content

        Example:
            >>> result = client.crawl(['https://example.com'])
            >>> print(result['results'][0]['document']['title'])
        """
        payload: Dict[str, Any] = {"urls": urls}

        if options:
            payload["options"] = options

        if session_id:
            payload["session_id"] = session_id

        return self._request("POST", "/crawl", data=payload)

    def stream_crawl(
        self,
        urls: List[str],
        options: Optional[CrawlOptions] = None,
    ) -> Iterator[Dict[str, Any]]:
        """Stream crawl results in real-time (NDJSON).

        Args:
            urls: List of URLs to crawl
            options: Crawl options

        Yields:
            Individual crawl results as they complete

        Example:
            >>> for result in client.stream_crawl(['https://example.com']):
            ...     print(f"Got: {result['url']}")
        """
        payload: Dict[str, Any] = {"urls": urls}

        if options:
            payload["options"] = options

        response = self._request("POST", "/crawl/stream", data=payload, stream=True)

        for line in response.iter_lines():
            if line:
                import json
                yield json.loads(line)

    def search(
        self,
        query: str,
        options: Optional[SearchOptions] = None,
    ) -> Dict[str, Any]:
        """Perform deep search with content extraction.

        Args:
            query: Search query string
            options: Search options (limit, include_content, etc.)

        Returns:
            Search results with extracted content

        Example:
            >>> results = client.search('python tutorials', {'limit': 10})
            >>> for item in results['results']:
            ...     print(item['title'])
        """
        payload: Dict[str, Any] = {"query": query}

        if options:
            payload.update(options)

        return self._request("POST", "/deepsearch", data=payload)

    def render(
        self,
        url: str,
        wait_time: int = 2000,
        screenshot: bool = False,
    ) -> Dict[str, Any]:
        """Render JavaScript-heavy pages using headless browser.

        Args:
            url: URL to render
            wait_time: Wait time in milliseconds
            screenshot: Capture screenshot

        Returns:
            Rendered content
        """
        payload = {
            "url": url,
            "wait_time": wait_time,
            "screenshot": screenshot,
        }

        return self._request("POST", "/render", data=payload)

    # Session Management

    def list_sessions(self) -> Dict[str, Any]:
        """List all active sessions."""
        return self._request("GET", "/sessions")

    def create_session(
        self,
        name: str,
        config: Optional[SessionConfig] = None,
    ) -> Dict[str, Any]:
        """Create a new session.

        Args:
            name: Session name
            config: Session configuration (user agent, cookies, etc.)

        Returns:
            Created session details including session_id
        """
        payload: Dict[str, Any] = {"name": name}

        if config:
            payload["config"] = config

        return self._request("POST", "/sessions", data=payload)

    def get_session(self, session_id: str) -> Dict[str, Any]:
        """Get session details."""
        return self._request("GET", f"/sessions/{session_id}")

    def delete_session(self, session_id: str) -> Dict[str, Any]:
        """Delete a session."""
        return self._request("DELETE", f"/sessions/{session_id}")

    # Monitoring

    def health_score(self) -> Dict[str, Any]:
        """Get overall health score (0-100)."""
        return self._request("GET", "/monitoring/health-score")

    def performance_report(self) -> Dict[str, Any]:
        """Get detailed performance metrics."""
        return self._request("GET", "/monitoring/performance-report")

    # Workers

    def worker_status(self) -> Dict[str, Any]:
        """Get worker queue status."""
        return self._request("GET", "/workers/status")

    # Strategies

    def get_strategies(self) -> Dict[str, Any]:
        """Get available extraction strategies."""
        return self._request("GET", "/strategies/info")

    # Spider - Phase 2 Implementation

    def spider(
        self,
        seeds: List[str],
        result_mode: str = "stats",
        max_pages: int = 100,
        max_depth: int = 3,
        include: Optional[str] = None,
        exclude: Optional[str] = None,
        scope: Optional[Dict[str, Any]] = None,
        max_content_bytes: Optional[int] = None,
    ) -> Dict[str, Any]:
        """Execute spider crawl with configurable result mode.

        Args:
            seeds: List of starting URLs
            result_mode: Output format - 'stats' (lightweight), 'urls' (discovered URLs),
                        'pages' (full page objects), 'stream' (use spider_stream instead),
                        'store' (async job, returns job_id)
            max_pages: Maximum number of pages to crawl (default: 100)
            max_depth: Maximum crawl depth (default: 3)
            include: Comma-separated fields to include (e.g., 'title,links,markdown')
            exclude: Comma-separated fields to exclude (e.g., 'content')
            scope: Crawl scope restrictions (optional)
            max_content_bytes: Maximum content size per page (optional)

        Returns:
            Dict with results based on result_mode:
            - 'stats': SpiderResultStats (pages_crawled, duration, etc.)
            - 'urls': SpiderResultUrls (stats + discovered_urls list)
            - 'pages': SpiderResultPages (stats + full page objects)
            - 'store': SpiderJobResponse (job_id for async fetch)

        Example:
            >>> # Discover URLs only
            >>> result = client.spider(
            ...     seeds=['https://example.com'],
            ...     result_mode='urls',
            ...     max_pages=500
            ... )
            >>> for url in result['discovered_urls']:
            ...     print(url)

            >>> # Get full page data with selective fields
            >>> result = client.spider(
            ...     seeds=['https://docs.example.com'],
            ...     result_mode='pages',
            ...     include='title,markdown,links',
            ...     max_pages=100
            ... )
            >>> for page in result['pages']:
            ...     print(f"{page['title']}: {len(page.get('markdown', ''))} chars")

            >>> # Large crawl - store for async fetch
            >>> job = client.spider(
            ...     seeds=['https://example.com'],
            ...     result_mode='store',
            ...     max_pages=10000
            ... )
            >>> job_id = job['job_id']
        """
        params: Dict[str, Any] = {
            "result_mode": result_mode,
            "max_pages": max_pages,
            "max_depth": max_depth,
        }

        if include:
            params["include"] = include
        if exclude:
            params["exclude"] = exclude
        if scope:
            params["scope"] = scope
        if max_content_bytes:
            params["max_content_bytes"] = max_content_bytes

        payload = {"seeds": seeds}

        return self._request("POST", "/spider", data=payload, params=params)

    def spider_stream(
        self,
        seeds: List[str],
        max_pages: int = 100,
        max_depth: int = 3,
        include: Optional[str] = None,
        exclude: Optional[str] = None,
    ) -> Iterator[Dict[str, Any]]:
        """Stream spider results in real-time (NDJSON format).

        Args:
            seeds: List of starting URLs
            max_pages: Maximum number of pages to crawl
            max_depth: Maximum crawl depth
            include: Comma-separated fields to include
            exclude: Comma-separated fields to exclude

        Yields:
            Dicts with 'type' and 'data' keys:
            - {"type": "page", "data": CrawledPage}
            - {"type": "stats", "data": SpiderResultStats} (final message)

        Example:
            >>> for event in client.spider_stream(
            ...     seeds=['https://example.com'],
            ...     include='title,links,markdown'
            ... ):
            ...     if event['type'] == 'page':
            ...         page = event['data']
            ...         print(f"Crawled: {page['url']}")
            ...     elif event['type'] == 'stats':
            ...         print(f"Complete: {event['data']['pages_crawled']} pages")
        """
        params: Dict[str, Any] = {
            "result_mode": "stream",
            "max_pages": max_pages,
            "max_depth": max_depth,
        }

        if include:
            params["include"] = include
        if exclude:
            params["exclude"] = exclude

        payload = {"seeds": seeds}

        # Accept NDJSON format
        headers = {"Accept": "application/x-ndjson"}
        original_headers = self.session.headers.copy()
        self.session.headers.update(headers)

        try:
            response = self._request("POST", "/spider", data=payload, params=params, stream=True)

            for line in response.iter_lines():
                if line:
                    import json
                    yield json.loads(line)
        finally:
            # Restore original headers
            self.session.headers = original_headers

    def spider_store(
        self,
        seeds: List[str],
        max_pages: int = 1000,
        max_depth: int = 5,
        include: Optional[str] = None,
        exclude: Optional[str] = None,
    ) -> str:
        """Start a stored spider job for async result fetching.

        Args:
            seeds: List of starting URLs
            max_pages: Maximum number of pages to crawl
            max_depth: Maximum crawl depth
            include: Comma-separated fields to include
            exclude: Comma-separated fields to exclude

        Returns:
            Job ID string for fetching results

        Example:
            >>> job_id = client.spider_store(
            ...     seeds=['https://example.com'],
            ...     max_pages=5000,
            ...     include='title,markdown'
            ... )
            >>> # Fetch results in pages
            >>> results = client.get_results(job_id, limit=200)
        """
        result = self.spider(
            seeds=seeds,
            result_mode="store",
            max_pages=max_pages,
            max_depth=max_depth,
            include=include,
            exclude=exclude,
        )
        return result["job_id"]

    def get_results(
        self,
        job_id: str,
        cursor: Optional[str] = None,
        limit: int = 100,
        include: Optional[str] = None,
    ) -> JobResultsResponse:
        """Fetch paginated results from a stored spider job.

        Args:
            job_id: Job ID from spider_store()
            cursor: Pagination cursor (optional, for subsequent pages)
            limit: Number of results per page (default: 100)
            include: Comma-separated fields to include

        Returns:
            JobResultsResponse with 'pages', 'next_cursor', and 'done' fields

        Example:
            >>> cursor = None
            >>> while True:
            ...     batch = client.get_results(job_id, cursor=cursor, limit=200)
            ...     for page in batch['pages']:
            ...         process_page(page)
            ...     if batch['done']:
            ...         break
            ...     cursor = batch['next_cursor']
        """
        params: Dict[str, Any] = {"limit": limit}

        if cursor:
            params["cursor"] = cursor
        if include:
            params["include"] = include

        return self._request("GET", f"/jobs/{job_id}/results", params=params)

    def get_stats(self, job_id: str) -> SpiderResultStats:
        """Get statistics for a spider job.

        Args:
            job_id: Job ID from spider_store()

        Returns:
            SpiderResultStats with pages_crawled, duration, etc.

        Example:
            >>> stats = client.get_stats(job_id)
            >>> print(f"Crawled {stats['pages_crawled']} pages in {stats['duration_seconds']}s")
        """
        return self._request("GET", f"/jobs/{job_id}/stats")

    def extract(
        self,
        url: str,
        format: str = "markdown",
    ) -> Dict[str, Any]:
        """Extract content from a single URL.

        Args:
            url: URL to extract content from
            format: Output format ('markdown', 'html', 'text')

        Returns:
            Extracted content with metadata

        Example:
            >>> result = client.extract('https://example.com/article', format='markdown')
            >>> print(result['markdown'])
        """
        payload = {
            "url": url,
            "format": format,
        }

        return self._request("POST", "/extract", data=payload)

    def extract_batch(
        self,
        urls: List[str],
        format: str = "markdown",
    ) -> List[ExtractBatchResult]:
        """Extract content from multiple URLs in batch.

        Args:
            urls: List of URLs to extract
            format: Output format ('markdown', 'html', 'text')

        Returns:
            List of extraction results with url, content, and metadata

        Example:
            >>> urls = ['https://example.com/page1', 'https://example.com/page2']
            >>> results = client.extract_batch(urls, format='markdown')
            >>> for result in results:
            ...     if result.get('markdown'):
            ...         print(f"{result['url']}: {len(result['markdown'])} chars")
        """
        payload: ExtractBatchRequest = {
            "urls": urls,
            "format": format,
        }

        return self._request("POST", "/extract/batch", data=payload)

    # Legacy spider method for backward compatibility
    def start_spider(
        self,
        url: str,
        max_depth: int = 2,
        max_pages: int = 10,
    ) -> Dict[str, Any]:
        """Start deep crawling from a URL (legacy method).

        DEPRECATED: Use spider() method instead for Phase 2 features.

        Args:
            url: Starting URL
            max_depth: Maximum crawl depth
            max_pages: Maximum pages to crawl

        Returns:
            Spider job details
        """
        payload = {
            "url": url,
            "max_depth": max_depth,
            "max_pages": max_pages,
        }

        return self._request("POST", "/spider/start", data=payload)

    def close(self) -> None:
        """Close the HTTP session."""
        self.session.close()

    def __enter__(self) -> "RipTide":
        """Context manager entry."""
        return self

    def __exit__(self, *args: Any) -> None:
        """Context manager exit."""
        self.close()
