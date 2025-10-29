"""
RipTide SDK main client

Provides the primary interface for interacting with the RipTide API.

For fluent configuration, use RipTideClientBuilder:
    >>> from riptide_sdk import RipTideClientBuilder
    >>> client = (RipTideClientBuilder()
    ...     .with_base_url("http://localhost:8080")
    ...     .with_api_key("your-key")
    ...     .with_timeout(60.0)
    ...     .build())
"""

from typing import Optional, Dict, Any, List
import httpx
import asyncio

from .endpoints import CrawlAPI, ProfilesAPI, EngineSelectionAPI, StreamingAPI
from .exceptions import ConfigError


class RipTideClient:
    """
    Main client for RipTide API

    This client provides access to all RipTide API endpoints with support for
    async/await, streaming, and comprehensive error handling.

    Example:
        Basic usage:
        >>> async with RipTideClient(base_url="http://localhost:8080") as client:
        ...     # Batch crawl
        ...     result = await client.crawl.batch(["https://example.com"])
        ...     print(result.to_summary())
        ...
        ...     # Domain profiles
        ...     profile = await client.profiles.create("example.com")
        ...     print(profile.to_markdown())
        ...
        ...     # Engine selection
        ...     stats = await client.engine.get_stats()
        ...
        ...     # Streaming
        ...     async for item in client.streaming.crawl_ndjson(urls):
        ...         print(item.data)

        Fluent builder pattern (recommended):
        >>> from riptide_sdk import RipTideClientBuilder
        >>> client = (RipTideClientBuilder()
        ...     .with_base_url("http://localhost:8080")
        ...     .with_api_key("your-api-key")
        ...     .with_timeout(60.0)
        ...     .with_max_connections(200)
        ...     .with_retry_config(max_retries=3, backoff_factor=2.0)
        ...     .build())
        >>> async with client:
        ...     result = await client.crawl.batch(urls)
        ...     print(result.to_markdown())
    """

    def __init__(
        self,
        base_url: str = "http://localhost:8080",
        api_key: Optional[str] = None,
        timeout: float = 30.0,
        max_connections: int = 100,
        **kwargs,
    ):
        """
        Initialize RipTide client

        Args:
            base_url: Base URL for the RipTide API (default: http://localhost:8080)
            api_key: Optional API key for authentication
            timeout: Request timeout in seconds (default: 30.0)
            max_connections: Maximum number of concurrent connections (default: 100)
            **kwargs: Additional arguments passed to httpx.AsyncClient
        """
        if not base_url:
            raise ConfigError("base_url cannot be empty")

        # Remove trailing slash
        self.base_url = base_url.rstrip("/")
        self.api_key = api_key

        # Build headers
        headers = {
            "User-Agent": "riptide-python-sdk/0.1.0",
            "Content-Type": "application/json",
        }
        if api_key:
            headers["Authorization"] = f"Bearer {api_key}"

        # Create HTTP client with connection pooling
        self._client = httpx.AsyncClient(
            base_url=self.base_url,
            headers=headers,
            timeout=timeout,
            limits=httpx.Limits(
                max_connections=max_connections,
                max_keepalive_connections=20,
            ),
            **kwargs,
        )

        # Initialize API endpoints
        self.crawl = CrawlAPI(self._client, self.base_url)
        self.profiles = ProfilesAPI(self._client, self.base_url)
        self.engine = EngineSelectionAPI(self._client, self.base_url)
        self.streaming = StreamingAPI(self._client, self.base_url)

        # Retry config (set by builder if used)
        self._retry_config = None

    async def __aenter__(self):
        """Context manager entry"""
        return self

    async def __aexit__(self, exc_type, exc_val, exc_tb):
        """Context manager exit - closes HTTP client"""
        await self.close()

    async def close(self):
        """Close the HTTP client and clean up resources"""
        await self._client.aclose()

    async def health_check(self) -> Dict[str, Any]:
        """
        Perform a health check on the API

        Returns:
            Dictionary with health status

        Example:
            >>> health = await client.health_check()
            >>> print(health["status"])
        """
        response = await self._client.get("/health")
        response.raise_for_status()
        return response.json()

    async def batch_crawl_parallel(
        self,
        urls: List[str],
        batch_size: int = 10,
        max_concurrent: int = 5,
    ) -> List[Any]:
        """
        Crawl multiple URLs in parallel batches for maximum throughput

        This method automatically splits URLs into batches and processes them
        concurrently using asyncio.gather() for optimal performance.

        Args:
            urls: List of URLs to crawl
            batch_size: Number of URLs per batch (default: 10)
            max_concurrent: Maximum concurrent batch requests (default: 5)

        Returns:
            List of CrawlResponse objects

        Example:
            >>> urls = [f"https://example.com/page{i}" for i in range(100)]
            >>> results = await client.batch_crawl_parallel(urls, batch_size=10)
            >>> total_successful = sum(r.successful for r in results)
        """
        from .models import CrawlOptions

        # Split into batches
        batches = [urls[i:i + batch_size] for i in range(0, len(urls), batch_size)]

        # Process in concurrent groups
        all_results = []
        for i in range(0, len(batches), max_concurrent):
            batch_group = batches[i:i + max_concurrent]
            tasks = [
                self.crawl.batch(batch, CrawlOptions())
                for batch in batch_group
            ]
            results = await asyncio.gather(*tasks, return_exceptions=True)
            all_results.extend(results)

        return all_results

    def __repr__(self) -> str:
        """String representation of client"""
        return (
            f"RipTideClient(base_url={self.base_url!r}, "
            f"has_api_key={bool(self.api_key)})"
        )
