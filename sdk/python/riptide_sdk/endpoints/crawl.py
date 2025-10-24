"""
Crawl API endpoint implementation

Provides batch crawling operations with support for caching, concurrency,
and various extraction modes.
"""

from typing import List, Optional
import httpx

from ..models import CrawlResponse, CrawlOptions
from ..exceptions import APIError, ValidationError


class CrawlAPI:
    """API for batch crawl operations"""

    def __init__(self, client: httpx.AsyncClient, base_url: str):
        """
        Initialize CrawlAPI

        Args:
            client: Async HTTP client
            base_url: Base URL for the API
        """
        self.client = client
        self.base_url = base_url

    async def batch(
        self,
        urls: List[str],
        options: Optional[CrawlOptions] = None,
    ) -> CrawlResponse:
        """
        Perform batch crawl of multiple URLs

        Args:
            urls: List of URLs to crawl
            options: Optional crawl options

        Returns:
            CrawlResponse with results and statistics

        Raises:
            ValidationError: If URLs are invalid
            APIError: If the API returns an error

        Example:
            >>> result = await client.crawl.batch(
            ...     ["https://example.com", "https://example.org"],
            ...     options=CrawlOptions(concurrency=10)
            ... )
            >>> print(f"Successful: {result.successful}/{result.total_urls}")
        """
        if not urls:
            raise ValidationError("URLs list cannot be empty")

        if len(urls) > 100:
            raise ValidationError("Maximum 100 URLs per batch request")

        # Validate URLs
        for url in urls:
            if not url.startswith(("http://", "https://")):
                raise ValidationError(f"Invalid URL: {url}")

        # Build request body
        body = {"urls": urls}
        if options:
            body["options"] = options.to_dict()

        # Make request
        response = await self.client.post(
            f"{self.base_url}/api/v1/crawl",
            json=body,
        )

        if response.status_code != 200:
            error_data = response.json() if response.text else {}
            raise APIError(
                message=error_data.get("error", {}).get("message", "Crawl failed"),
                status_code=response.status_code,
                response_data=error_data,
            )

        return CrawlResponse.from_dict(response.json())

    async def single(
        self,
        url: str,
        options: Optional[CrawlOptions] = None,
    ) -> CrawlResponse:
        """
        Crawl a single URL (convenience wrapper around batch)

        Args:
            url: URL to crawl
            options: Optional crawl options

        Returns:
            CrawlResponse with single result

        Example:
            >>> result = await client.crawl.single("https://example.com")
            >>> doc = result.results[0].document
        """
        return await self.batch([url], options)
