"""
Streaming API endpoint implementation

Provides streaming operations for crawl and deep search with support for
NDJSON, Server-Sent Events (SSE), and WebSocket protocols.
"""

from typing import AsyncIterator, List, Optional, Dict, Any, Callable, Awaitable
import httpx
import json
import asyncio

from ..models import StreamingResult, CrawlOptions
from ..exceptions import APIError, StreamingError, ValidationError

try:
    import websockets
    from websockets.client import WebSocketClientProtocol
    WEBSOCKETS_AVAILABLE = True
except ImportError:
    WEBSOCKETS_AVAILABLE = False
    WebSocketClientProtocol = None  # type: ignore


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

    async def crawl_websocket(
        self,
        urls: List[str],
        options: Optional[CrawlOptions] = None,
        on_message: Optional[Callable[[StreamingResult], Awaitable[None]]] = None,
    ) -> AsyncIterator[StreamingResult]:
        """
        Stream crawl results via WebSocket for bidirectional real-time communication.

        WebSocket streaming provides:
        - Bidirectional communication (can send/receive)
        - Automatic ping/pong keepalive
        - Connection health monitoring
        - Backpressure handling
        - Real-time progress updates

        Args:
            urls: List of URLs to crawl
            options: Optional crawl options
            on_message: Optional async callback for each message received

        Yields:
            StreamingResult: Streaming results as they arrive including:
                - welcome: Initial connection message
                - metadata: Crawl session metadata
                - result: Individual URL results with progress
                - summary: Final statistics and summary
                - error: Error messages

        Raises:
            ValidationError: If URLs are invalid
            StreamingError: If WebSocket connection fails
            ImportError: If websockets library is not installed

        Example:
            >>> async for result in client.streaming.crawl_websocket(urls):
            ...     if result.event_type == "result":
            ...         data = result.data["result"]
            ...         print(f"Got result for: {data['url']}")
            ...     elif result.event_type == "summary":
            ...         print(f"Completed: {result.data}")

            >>> # With callback for real-time processing
            >>> async def handle_result(result: StreamingResult):
            ...     if result.event_type == "result":
            ...         print(f"Processing: {result.data['result']['url']}")
            >>>
            >>> async for result in client.streaming.crawl_websocket(
            ...     urls, on_message=handle_result
            ... ):
            ...     pass
        """
        if not WEBSOCKETS_AVAILABLE:
            raise ImportError(
                "websockets library is required for WebSocket streaming. "
                "Install it with: pip install websockets"
            )

        if not urls:
            raise ValidationError("URLs list cannot be empty")

        # Convert base_url from HTTP to WebSocket scheme
        ws_base_url = self.base_url.replace("http://", "ws://").replace("https://", "wss://")
        ws_url = f"{ws_base_url}/crawl/ws"

        try:
            async with websockets.connect(
                ws_url,
                ping_interval=30,
                ping_timeout=10,
                close_timeout=10,
                max_size=10 * 1024 * 1024,  # 10MB max message size
            ) as websocket:
                # Receive welcome message
                welcome_msg = await websocket.recv()
                welcome_data = json.loads(welcome_msg)
                welcome_result = StreamingResult(
                    event_type=welcome_data.get("message_type", "welcome"),
                    data=welcome_data.get("data", welcome_data),
                    timestamp=welcome_data.get("timestamp"),
                )
                if on_message:
                    await on_message(welcome_result)
                yield welcome_result

                # Prepare crawl request
                request_body = {"urls": urls}
                if options:
                    request_body["options"] = options.to_dict()

                crawl_request = {
                    "request_type": "crawl",
                    "data": request_body,
                }

                # Send crawl request
                await websocket.send(json.dumps(crawl_request))

                # Receive and yield results
                async for message in websocket:
                    try:
                        if isinstance(message, bytes):
                            message = message.decode("utf-8")

                        data = json.loads(message)
                        result = StreamingResult(
                            event_type=data.get("message_type", "message"),
                            data=data.get("data", data),
                            timestamp=data.get("timestamp"),
                        )

                        if on_message:
                            await on_message(result)

                        yield result

                        # Stop after receiving summary
                        if result.event_type == "summary":
                            break

                    except json.JSONDecodeError as e:
                        error_result = StreamingResult(
                            event_type="error",
                            data={"error": f"Invalid JSON received: {e}", "raw": message},
                        )
                        if on_message:
                            await on_message(error_result)
                        yield error_result

        except websockets.exceptions.WebSocketException as e:
            raise StreamingError(f"WebSocket error: {e}")
        except asyncio.TimeoutError:
            raise StreamingError("WebSocket connection timeout")
        except Exception as e:
            raise StreamingError(f"Unexpected error during WebSocket streaming: {e}")

    async def ping_websocket(self, websocket_url: Optional[str] = None) -> Dict[str, Any]:
        """
        Send a ping request to the WebSocket server and measure response time.

        Args:
            websocket_url: Optional custom WebSocket URL. If not provided,
                          uses the default crawl WebSocket endpoint.

        Returns:
            Dict containing:
                - success: Whether ping was successful
                - latency_ms: Round-trip time in milliseconds
                - server_time: Server timestamp from response
                - session_id: WebSocket session ID

        Raises:
            ImportError: If websockets library is not installed
            StreamingError: If WebSocket connection fails

        Example:
            >>> result = await client.streaming.ping_websocket()
            >>> print(f"WebSocket latency: {result['latency_ms']}ms")
        """
        if not WEBSOCKETS_AVAILABLE:
            raise ImportError(
                "websockets library is required for WebSocket streaming. "
                "Install it with: pip install websockets"
            )

        ws_url = websocket_url
        if not ws_url:
            ws_base_url = self.base_url.replace("http://", "ws://").replace(
                "https://", "wss://"
            )
            ws_url = f"{ws_base_url}/crawl/ws"

        try:
            start_time = asyncio.get_event_loop().time()

            async with websockets.connect(ws_url, ping_interval=30) as websocket:
                # Receive welcome message
                await websocket.recv()

                # Send ping request
                ping_request = {"request_type": "ping", "data": {}}
                await websocket.send(json.dumps(ping_request))

                # Receive pong response
                pong_msg = await websocket.recv()
                end_time = asyncio.get_event_loop().time()

                pong_data = json.loads(pong_msg)
                latency_ms = (end_time - start_time) * 1000

                return {
                    "success": True,
                    "latency_ms": latency_ms,
                    "server_time": pong_data.get("data", {}).get("timestamp"),
                    "session_id": pong_data.get("data", {}).get("session_id"),
                }

        except Exception as e:
            raise StreamingError(f"WebSocket ping failed: {e}")

    async def get_websocket_status(
        self, websocket_url: Optional[str] = None
    ) -> Dict[str, Any]:
        """
        Get status information from an active WebSocket connection.

        Args:
            websocket_url: Optional custom WebSocket URL

        Returns:
            Dict containing connection status information

        Raises:
            ImportError: If websockets library is not installed
            StreamingError: If WebSocket connection fails

        Example:
            >>> status = await client.streaming.get_websocket_status()
            >>> print(f"Connection healthy: {status['is_healthy']}")
        """
        if not WEBSOCKETS_AVAILABLE:
            raise ImportError(
                "websockets library is required for WebSocket streaming. "
                "Install it with: pip install websockets"
            )

        ws_url = websocket_url
        if not ws_url:
            ws_base_url = self.base_url.replace("http://", "ws://").replace(
                "https://", "wss://"
            )
            ws_url = f"{ws_base_url}/crawl/ws"

        try:
            async with websockets.connect(ws_url) as websocket:
                # Receive welcome
                await websocket.recv()

                # Send status request
                status_request = {"request_type": "status", "data": {}}
                await websocket.send(json.dumps(status_request))

                # Receive status response
                status_msg = await websocket.recv()
                status_data = json.loads(status_msg)

                return status_data.get("data", {})

        except Exception as e:
            raise StreamingError(f"Failed to get WebSocket status: {e}")
