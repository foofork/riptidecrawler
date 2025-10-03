"""RipTide API Client implementation."""

from typing import Any, Dict, List, Optional, Iterator
import requests
from requests.adapters import HTTPAdapter
from urllib3.util.retry import Retry

from .exceptions import APIError, RateLimitError, TimeoutError, ValidationError
from .types import CrawlOptions, SearchOptions, SessionConfig


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

    # Spider

    def start_spider(
        self,
        url: str,
        max_depth: int = 2,
        max_pages: int = 10,
    ) -> Dict[str, Any]:
        """Start deep crawling from a URL.

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
