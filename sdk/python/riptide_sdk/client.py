"""
RipTide SDK main client

Provides the primary interface for interacting with the RipTide API.
"""

from typing import Optional
import httpx

from .endpoints import CrawlAPI, ProfilesAPI, EngineSelectionAPI, StreamingAPI
from .exceptions import ConfigError


class RipTideClient:
    """
    Main client for RipTide API

    This client provides access to all RipTide API endpoints with support for
    async/await, streaming, and comprehensive error handling.

    Example:
        >>> async with RipTideClient(base_url="http://localhost:8080") as client:
        ...     # Batch crawl
        ...     result = await client.crawl.batch(["https://example.com"])
        ...
        ...     # Domain profiles
        ...     profile = await client.profiles.create("example.com")
        ...
        ...     # Engine selection
        ...     stats = await client.engine.get_stats()
        ...
        ...     # Streaming
        ...     async for item in client.streaming.crawl_ndjson(urls):
        ...         print(item.data)
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

    async def __aenter__(self):
        """Context manager entry"""
        return self

    async def __aexit__(self, exc_type, exc_val, exc_tb):
        """Context manager exit - closes HTTP client"""
        await self.close()

    async def close(self):
        """Close the HTTP client and clean up resources"""
        await self._client.aclose()

    async def health_check(self) -> dict:
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
