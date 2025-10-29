"""
Spider crawling API endpoint implementation

Provides deep crawling operations using the Spider engine with:
- Frontier-based URL queue management
- Multiple crawling strategies (BFS, DFS, Best-First)
- Adaptive stopping based on content analysis
- Budget controls and rate limiting
- Session persistence for authenticated crawling
"""

from typing import List, Optional, Dict, Any, Literal
import httpx

from ..models import (
    SpiderConfig,
    SpiderResult,
    SpiderStatus,
    SpiderControlResponse,
    ResultMode,
)
from ..exceptions import APIError, ValidationError, ConfigError


class SpiderAPI:
    """
    API for deep crawling operations using Spider engine

    The Spider API provides advanced crawling capabilities beyond simple
    batch operations, including multi-page crawling with intelligent
    frontier management and adaptive stopping conditions.
    """

    def __init__(self, client: httpx.AsyncClient, base_url: str):
        """
        Initialize SpiderAPI

        Args:
            client: Async HTTP client
            base_url: Base URL for the API
        """
        self.client = client
        self.base_url = base_url

    async def crawl(
        self,
        seed_urls: List[str],
        config: Optional[SpiderConfig] = None,
        result_mode: ResultMode = ResultMode.STATS,
    ) -> SpiderResult:
        """
        Start a deep crawl from seed URLs using the Spider engine

        This endpoint performs multi-page crawling with intelligent URL frontier
        management, multiple strategies, and adaptive stopping conditions.

        Args:
            seed_urls: List of starting URLs for the crawl
            config: Optional spider configuration (defaults, depth, strategy, etc.)
            result_mode: Result mode - STATS (default) or URLS to include discovered URLs

        Returns:
            SpiderResult with crawl summary, state, and performance metrics.
            If result_mode=URLS, also includes discovered_urls list.

        Raises:
            ValidationError: If seed URLs are invalid or empty
            ConfigError: If SpiderFacade is not enabled on the server
            APIError: If the API returns an error

        Example:
            Basic crawl (stats only):
            >>> result = await client.spider.crawl(
            ...     seed_urls=["https://example.com"],
            ... )
            >>> print(f"Crawled {result.pages_crawled} pages")
            >>> print(f"Stop reason: {result.stop_reason}")

            Crawl with URLs mode (for discovery):
            >>> from riptide_sdk import ResultMode
            >>> result = await client.spider.crawl(
            ...     seed_urls=["https://example.com"],
            ...     result_mode=ResultMode.URLS,
            ... )
            >>> print(f"Discovered {len(result.discovered_urls)} URLs")
            >>> for url in result.discovered_urls[:10]:
            ...     print(f"  - {url}")

            Advanced configuration:
            >>> from riptide_sdk import SpiderConfig, ResultMode
            >>> config = SpiderConfig(
            ...     max_depth=3,
            ...     max_pages=100,
            ...     strategy="breadth_first",
            ...     concurrency=10,
            ...     delay_ms=500,
            ...     respect_robots=True,
            ... )
            >>> result = await client.spider.crawl(
            ...     seed_urls=["https://example.com"],
            ...     config=config,
            ...     result_mode=ResultMode.URLS,
            ... )
            >>> print(result.to_summary())
            >>> print(f"Discovered URLs: {len(result.discovered_urls)}")

            Discover → Extract workflow:
            >>> # Step 1: Discover URLs
            >>> discovery = await client.spider.crawl(
            ...     seed_urls=["https://example.com/blog"],
            ...     config=SpiderConfig(max_depth=2),
            ...     result_mode=ResultMode.URLS,
            ... )
            >>> # Step 2: Extract content from discovered URLs
            >>> for url in discovery.discovered_urls:
            ...     content = await client.extract.extract(url)
            ...     print(f"Extracted: {content.title}")
        """
        if not seed_urls:
            raise ValidationError("seed_urls list cannot be empty")

        if len(seed_urls) > 50:
            raise ValidationError("Maximum 50 seed URLs per crawl request")

        # Validate seed URLs
        for url in seed_urls:
            if not url.startswith(("http://", "https://")):
                raise ValidationError(f"Invalid seed URL: {url}")

        # Build request body
        body = {"seed_urls": seed_urls}
        if config:
            body.update(config.to_dict())

        # Build query parameters
        params = {}
        if result_mode != ResultMode.STATS:
            params["result_mode"] = result_mode.value

        # Make request
        try:
            response = await self.client.post(
                f"{self.base_url}/api/v1/spider/crawl",
                json=body,
                params=params,
            )
        except httpx.RequestError as e:
            raise APIError(
                message=f"Request failed: {str(e)}",
                status_code=0,
            )

        if response.status_code == 500:
            error_data = response.json() if response.text else {}
            error_msg = error_data.get("error", {}).get("message", "")
            if "SpiderFacade is not enabled" in error_msg:
                raise ConfigError(
                    "SpiderFacade is not enabled on the server. "
                    "Please enable spider functionality in server configuration."
                )
            raise APIError(
                message=error_msg or "Spider crawl failed",
                status_code=response.status_code,
                response_data=error_data,
            )

        if response.status_code != 200:
            error_data = response.json() if response.text else {}
            raise APIError(
                message=error_data.get("error", {}).get("message", "Spider crawl failed"),
                status_code=response.status_code,
                response_data=error_data,
            )

        return SpiderResult.from_dict(response.json())

    async def status(
        self,
        include_metrics: bool = False,
    ) -> SpiderStatus:
        """
        Get current spider status and metrics

        Retrieves the current state of the spider engine, including active
        crawl status, pages processed, and optionally detailed performance metrics.

        Args:
            include_metrics: Whether to include detailed performance metrics

        Returns:
            SpiderStatus with current state and optional metrics

        Raises:
            ConfigError: If SpiderFacade is not enabled
            APIError: If the API returns an error

        Example:
            Basic status check:
            >>> status = await client.spider.status()
            >>> print(f"Active: {status.state.active}")
            >>> print(f"Pages crawled: {status.state.pages_crawled}")

            With detailed metrics:
            >>> status = await client.spider.status(include_metrics=True)
            >>> if status.performance:
            ...     print(f"Pages/sec: {status.performance.pages_per_second:.2f}")
            ...     print(f"Error rate: {status.performance.error_rate:.2%}")
        """
        body = {"include_metrics": include_metrics}

        try:
            response = await self.client.post(
                f"{self.base_url}/api/v1/spider/status",
                json=body,
            )
        except httpx.RequestError as e:
            raise APIError(
                message=f"Request failed: {str(e)}",
                status_code=0,
            )

        if response.status_code == 500:
            error_data = response.json() if response.text else {}
            error_msg = error_data.get("error", {}).get("message", "")
            if "SpiderFacade is not enabled" in error_msg:
                raise ConfigError(
                    "SpiderFacade is not enabled on the server. "
                    "Please enable spider functionality in server configuration."
                )
            raise APIError(
                message=error_msg or "Failed to get spider status",
                status_code=response.status_code,
                response_data=error_data,
            )

        if response.status_code != 200:
            error_data = response.json() if response.text else {}
            raise APIError(
                message=error_data.get("error", {}).get("message", "Failed to get spider status"),
                status_code=response.status_code,
                response_data=error_data,
            )

        return SpiderStatus.from_dict(response.json())

    async def control(
        self,
        action: Literal["stop", "reset"],
    ) -> SpiderControlResponse:
        """
        Control spider crawling operations

        Allows controlling the spider engine state through various actions:
        - stop: Stop the current crawl operation
        - reset: Reset the spider state and clear frontier

        Args:
            action: Control action to perform ("stop" or "reset")

        Returns:
            SpiderControlResponse with action result status

        Raises:
            ValidationError: If action is invalid
            ConfigError: If SpiderFacade is not enabled
            APIError: If the API returns an error

        Example:
            Stop current crawl:
            >>> response = await client.spider.control(action="stop")
            >>> print(response.status)  # "stopped"

            Reset spider state:
            >>> response = await client.spider.control(action="reset")
            >>> print(response.status)  # "reset"

            Error handling:
            >>> try:
            ...     await client.spider.control(action="pause")
            ... except ValidationError as e:
            ...     print(f"Invalid action: {e}")
        """
        if action not in ("stop", "reset"):
            raise ValidationError(
                f"Invalid action '{action}'. Must be 'stop' or 'reset'"
            )

        body = {"action": action}

        try:
            response = await self.client.post(
                f"{self.base_url}/api/v1/spider/control",
                json=body,
            )
        except httpx.RequestError as e:
            raise APIError(
                message=f"Request failed: {str(e)}",
                status_code=0,
            )

        if response.status_code == 500:
            error_data = response.json() if response.text else {}
            error_msg = error_data.get("error", {}).get("message", "")
            if "SpiderFacade is not enabled" in error_msg:
                raise ConfigError(
                    "SpiderFacade is not enabled on the server. "
                    "Please enable spider functionality in server configuration."
                )
            raise APIError(
                message=error_msg or f"Spider {action} failed",
                status_code=response.status_code,
                response_data=error_data,
            )

        if response.status_code != 200:
            error_data = response.json() if response.text else {}
            raise APIError(
                message=error_data.get("error", {}).get("message", f"Spider {action} failed"),
                status_code=response.status_code,
                response_data=error_data,
            )

        result = response.json()
        return SpiderControlResponse(status=result.get("status", action))

    async def crawl_with_status_polling(
        self,
        seed_urls: List[str],
        config: Optional[SpiderConfig] = None,
        poll_interval: float = 2.0,
        callback: Optional[callable] = None,
    ) -> SpiderResult:
        """
        Start a crawl and poll status until completion

        This is a convenience method that starts a crawl and periodically
        checks the status until the crawl completes. Useful for long-running
        crawls where you want progress updates.

        Args:
            seed_urls: List of starting URLs for the crawl
            config: Optional spider configuration
            poll_interval: Seconds between status checks (default: 2.0)
            callback: Optional callback function called with each status update

        Returns:
            SpiderResult with final crawl summary

        Example:
            With progress callback:
            >>> def on_progress(status):
            ...     print(f"Crawled: {status.state.pages_crawled}")
            >>>
            >>> result = await client.spider.crawl_with_status_polling(
            ...     seed_urls=["https://example.com"],
            ...     config=SpiderConfig(max_pages=100),
            ...     poll_interval=1.0,
            ...     callback=on_progress,
            ... )
        """
        import asyncio

        # Start the crawl (non-blocking on server side)
        result = await self.crawl(seed_urls, config)

        # If callback provided, poll for status updates
        if callback:
            while True:
                status = await self.status(include_metrics=True)
                callback(status)

                if not status.state.active:
                    break

                await asyncio.sleep(poll_interval)

        return result
