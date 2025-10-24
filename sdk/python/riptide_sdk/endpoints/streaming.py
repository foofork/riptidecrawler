"""
Streaming API endpoint implementation

Provides streaming operations for crawl and deep search with support for
NDJSON, Server-Sent Events (SSE), and WebSocket protocols.
"""

from typing import AsyncIterator, List, Optional, Dict, Any
import httpx
import json
import asyncio

from ..models import StreamingResult, CrawlOptions
from ..exceptions import APIError, StreamingError, ValidationError


class StreamingAPI:
    """API for streaming operations"""

    def __init__(self, client: httpx.AsyncClient, base_url: str):
        """
        Initialize StreamingAPI

        Args:
            client: Async HTTP client
            base_url: Base URL for the API
        """
        self.client = client
        self.base_url = base_url

    async def crawl_ndjson(
        self,
        urls: List[str],
        options: Optional[CrawlOptions] = None,
    ) -> AsyncIterator[StreamingResult]:
        """
        Stream crawl results in NDJSON format

        Args:
            urls: List of URLs to crawl
            options: Optional crawl options

        Yields:
            StreamingResult objects as they complete

        Raises:
            ValidationError: If URLs are invalid
            StreamingError: If streaming fails

        Example:
            >>> async for result in client.streaming.crawl_ndjson(urls):
            ...     print(f"Got result: {result.data['url']}")
        """
        if not urls:
            raise ValidationError("URLs list cannot be empty")

        body = {"urls": urls}
        if options:
            body["options"] = options.to_dict()

        try:
            async with self.client.stream(
                "POST",
                f"{self.base_url}/api/v1/stream/crawl",
                json=body,
            ) as response:
                if response.status_code != 200:
                    error_text = await response.aread()
                    raise StreamingError(
                        f"Streaming failed: {error_text.decode()}",
                        status_code=response.status_code,
                    )

                async for line in response.aiter_lines():
                    if line.strip():
                        try:
                            data = json.loads(line)
                            yield StreamingResult(
                                event_type="crawl_result",
                                data=data,
                            )
                        except json.JSONDecodeError as e:
                            raise StreamingError(f"Invalid JSON: {e}")

        except httpx.HTTPError as e:
            raise StreamingError(f"HTTP error during streaming: {e}")

    async def deepsearch_ndjson(
        self,
        query: str,
        limit: int = 10,
        options: Optional[Dict[str, Any]] = None,
    ) -> AsyncIterator[StreamingResult]:
        """
        Stream deep search results in NDJSON format

        Args:
            query: Search query
            limit: Maximum number of results
            options: Optional search options

        Yields:
            StreamingResult objects as results are found

        Example:
            >>> async for result in client.streaming.deepsearch_ndjson("python"):
            ...     print(f"Found: {result.data['url']}")
        """
        if not query:
            raise ValidationError("Query cannot be empty")

        body = {
            "query": query,
            "limit": limit,
        }
        if options:
            body.update(options)

        try:
            async with self.client.stream(
                "POST",
                f"{self.base_url}/api/v1/stream/deepsearch",
                json=body,
            ) as response:
                if response.status_code != 200:
                    error_text = await response.aread()
                    raise StreamingError(
                        f"Streaming failed: {error_text.decode()}",
                        status_code=response.status_code,
                    )

                async for line in response.aiter_lines():
                    if line.strip():
                        try:
                            data = json.loads(line)
                            yield StreamingResult(
                                event_type="search_result",
                                data=data,
                            )
                        except json.JSONDecodeError as e:
                            raise StreamingError(f"Invalid JSON: {e}")

        except httpx.HTTPError as e:
            raise StreamingError(f"HTTP error during streaming: {e}")

    async def crawl_sse(
        self,
        urls: List[str],
        options: Optional[CrawlOptions] = None,
    ) -> AsyncIterator[StreamingResult]:
        """
        Stream crawl results using Server-Sent Events

        Args:
            urls: List of URLs to crawl
            options: Optional crawl options

        Yields:
            StreamingResult objects

        Example:
            >>> async for event in client.streaming.crawl_sse(urls):
            ...     if event.event_type == "result":
            ...         print(event.data)
        """
        if not urls:
            raise ValidationError("URLs list cannot be empty")

        body = {"urls": urls}
        if options:
            body["options"] = options.to_dict()

        try:
            async with self.client.stream(
                "POST",
                f"{self.base_url}/api/v1/sse/crawl",
                json=body,
                headers={"Accept": "text/event-stream"},
            ) as response:
                if response.status_code != 200:
                    error_text = await response.aread()
                    raise StreamingError(
                        f"SSE streaming failed: {error_text.decode()}",
                        status_code=response.status_code,
                    )

                event_type = "message"
                event_data = []

                async for line in response.aiter_lines():
                    line = line.strip()

                    if not line:
                        # Empty line marks end of event
                        if event_data:
                            data_str = "\n".join(event_data)
                            try:
                                data = json.loads(data_str)
                                yield StreamingResult(
                                    event_type=event_type,
                                    data=data,
                                )
                            except json.JSONDecodeError:
                                # Not JSON, yield raw data
                                yield StreamingResult(
                                    event_type=event_type,
                                    data={"raw": data_str},
                                )
                            event_data = []
                            event_type = "message"
                        continue

                    if line.startswith("event:"):
                        event_type = line[6:].strip()
                    elif line.startswith("data:"):
                        event_data.append(line[5:].strip())

        except httpx.HTTPError as e:
            raise StreamingError(f"HTTP error during SSE streaming: {e}")
