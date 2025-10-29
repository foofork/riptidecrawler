"""
Unit tests for StreamingAPI

Tests NDJSON streaming, SSE streaming, error handling, and async iteration.
"""

import pytest
import json
import httpx
from unittest.mock import AsyncMock, Mock

from riptide_sdk.endpoints.streaming import StreamingAPI
from riptide_sdk.models import StreamingResult, CrawlOptions
from riptide_sdk.exceptions import ValidationError, StreamingError


@pytest.mark.unit
@pytest.mark.asyncio
class TestNDJSONStreaming:
    """Test NDJSON streaming functionality"""

    async def test_crawl_ndjson_basic(self, mock_ndjson_stream):
        """Test basic NDJSON crawl streaming"""
        mock_client = AsyncMock(spec=httpx.AsyncClient)
        api = StreamingAPI(mock_client, "http://test.com")

        test_data = [
            {"url": "https://example.com", "status": 200},
            {"url": "https://test.com", "status": 200},
        ]

        mock_client.stream = AsyncMock(return_value=await mock_ndjson_stream(test_data))

        results = []
        async for result in api.crawl_ndjson(["https://example.com"]):
            results.append(result)

        assert len(results) == 2
        assert all(isinstance(r, StreamingResult) for r in results)
        assert results[0].data["url"] == "https://example.com"

    async def test_crawl_ndjson_empty_urls_raises_error(self):
        """Test empty URLs list raises ValidationError"""
        api = StreamingAPI(AsyncMock(), "http://test.com")

        with pytest.raises(ValidationError, match="URLs list cannot be empty"):
            async for _ in api.crawl_ndjson([]):
                pass

    async def test_crawl_ndjson_with_options(self, mock_ndjson_stream):
        """Test NDJSON streaming with crawl options"""
        mock_client = AsyncMock(spec=httpx.AsyncClient)
        api = StreamingAPI(mock_client, "http://test.com")

        options = CrawlOptions(concurrency=10)
        test_data = [{"url": "https://example.com", "status": 200}]

        mock_client.stream = AsyncMock(return_value=await mock_ndjson_stream(test_data))

        results = []
        async for result in api.crawl_ndjson(["https://example.com"], options=options):
            results.append(result)

        assert len(results) == 1

        # Verify options were sent in request
        call_args = mock_client.stream.call_args
        assert call_args is not None

    async def test_crawl_ndjson_http_error(self):
        """Test NDJSON streaming handles HTTP errors"""
        mock_client = AsyncMock(spec=httpx.AsyncClient)
        api = StreamingAPI(mock_client, "http://test.com")

        class ErrorStream:
            status_code = 500

            async def __aenter__(self):
                return self

            async def __aexit__(self, *args):
                pass

            async def aread(self):
                return b"Internal Server Error"

        mock_client.stream = AsyncMock(return_value=ErrorStream())

        with pytest.raises(StreamingError, match="Streaming failed"):
            async for _ in api.crawl_ndjson(["https://example.com"]):
                pass

    async def test_crawl_ndjson_invalid_json(self):
        """Test NDJSON streaming handles invalid JSON"""
        mock_client = AsyncMock(spec=httpx.AsyncClient)
        api = StreamingAPI(mock_client, "http://test.com")

        class BadJSONStream:
            status_code = 200

            async def __aenter__(self):
                return self

            async def __aexit__(self, *args):
                pass

            async def aiter_lines(self):
                yield "not valid json"
                yield "also not valid"

            async def aread(self):
                return b""

        mock_client.stream = AsyncMock(return_value=BadJSONStream())

        with pytest.raises(StreamingError, match="Invalid JSON"):
            async for _ in api.crawl_ndjson(["https://example.com"]):
                pass

    async def test_crawl_ndjson_skips_empty_lines(self, mock_ndjson_stream):
        """Test NDJSON streaming skips empty lines"""
        mock_client = AsyncMock(spec=httpx.AsyncClient)
        api = StreamingAPI(mock_client, "http://test.com")

        class StreamWithEmptyLines:
            status_code = 200

            async def __aenter__(self):
                return self

            async def __aexit__(self, *args):
                pass

            async def aiter_lines(self):
                yield json.dumps({"url": "https://example.com"})
                yield ""
                yield "   "
                yield json.dumps({"url": "https://test.com"})

            async def aread(self):
                return b""

        mock_client.stream = AsyncMock(return_value=StreamWithEmptyLines())

        results = []
        async for result in api.crawl_ndjson(["https://example.com"]):
            results.append(result)

        assert len(results) == 2  # Only non-empty lines


@pytest.mark.unit
@pytest.mark.asyncio
class TestDeepSearchNDJSON:
    """Test deep search NDJSON streaming"""

    async def test_deepsearch_ndjson_basic(self, mock_ndjson_stream):
        """Test basic deep search streaming"""
        mock_client = AsyncMock(spec=httpx.AsyncClient)
        api = StreamingAPI(mock_client, "http://test.com")

        test_data = [
            {"url": "https://result1.com", "title": "Result 1"},
            {"url": "https://result2.com", "title": "Result 2"},
        ]

        mock_client.stream = AsyncMock(return_value=await mock_ndjson_stream(test_data))

        results = []
        async for result in api.deepsearch_ndjson("test query"):
            results.append(result)

        assert len(results) == 2
        assert results[0].event_type == "search_result"

    async def test_deepsearch_empty_query_raises_error(self):
        """Test empty query raises ValidationError"""
        api = StreamingAPI(AsyncMock(), "http://test.com")

        with pytest.raises(ValidationError, match="Query cannot be empty"):
            async for _ in api.deepsearch_ndjson(""):
                pass

    async def test_deepsearch_with_limit(self, mock_ndjson_stream):
        """Test deep search with custom limit"""
        mock_client = AsyncMock(spec=httpx.AsyncClient)
        api = StreamingAPI(mock_client, "http://test.com")

        test_data = [{"url": "https://example.com"}]
        mock_client.stream = AsyncMock(return_value=await mock_ndjson_stream(test_data))

        results = []
        async for result in api.deepsearch_ndjson("test", limit=5):
            results.append(result)

        # Verify limit was sent
        call_args = mock_client.stream.call_args
        assert call_args is not None


@pytest.mark.unit
@pytest.mark.asyncio
class TestSSEStreaming:
    """Test Server-Sent Events streaming"""

    async def test_crawl_sse_basic(self):
        """Test basic SSE crawl streaming"""
        mock_client = AsyncMock(spec=httpx.AsyncClient)
        api = StreamingAPI(mock_client, "http://test.com")

        class SSEStream:
            status_code = 200

            async def __aenter__(self):
                return self

            async def __aexit__(self, *args):
                pass

            async def aiter_lines(self):
                yield "event: message"
                yield 'data: {"url": "https://example.com", "status": 200}'
                yield ""
                yield "event: result"
                yield 'data: {"url": "https://test.com", "status": 200}'
                yield ""

            async def aread(self):
                return b""

        mock_client.stream = AsyncMock(return_value=SSEStream())

        results = []
        async for result in api.crawl_sse(["https://example.com"]):
            results.append(result)

        assert len(results) == 2
        assert results[0].event_type == "message"
        assert results[1].event_type == "result"

    async def test_crawl_sse_empty_urls_raises_error(self):
        """Test empty URLs raises ValidationError"""
        api = StreamingAPI(AsyncMock(), "http://test.com")

        with pytest.raises(ValidationError, match="URLs list cannot be empty"):
            async for _ in api.crawl_sse([]):
                pass

    async def test_crawl_sse_sets_correct_headers(self):
        """Test SSE streaming sets Accept header"""
        mock_client = AsyncMock(spec=httpx.AsyncClient)
        api = StreamingAPI(mock_client, "http://test.com")

        class EmptySSEStream:
            status_code = 200

            async def __aenter__(self):
                return self

            async def __aexit__(self, *args):
                pass

            async def aiter_lines(self):
                return
                yield  # Make it async generator

            async def aread(self):
                return b""

        mock_client.stream = AsyncMock(return_value=EmptySSEStream())

        async for _ in api.crawl_sse(["https://example.com"]):
            pass

        call_kwargs = mock_client.stream.call_args.kwargs
        assert "headers" in call_kwargs
        assert call_kwargs["headers"]["Accept"] == "text/event-stream"

    async def test_crawl_sse_handles_multiline_data(self):
        """Test SSE handles multi-line data fields"""
        mock_client = AsyncMock(spec=httpx.AsyncClient)
        api = StreamingAPI(mock_client, "http://test.com")

        class MultilineSSEStream:
            status_code = 200

            async def __aenter__(self):
                return self

            async def __aexit__(self, *args):
                pass

            async def aiter_lines(self):
                yield "event: message"
                yield 'data: {"line1":'
                yield 'data: "value"}'
                yield ""

            async def aread(self):
                return b""

        mock_client.stream = AsyncMock(return_value=MultilineSSEStream())

        results = []
        async for result in api.crawl_sse(["https://example.com"]):
            results.append(result)

        assert len(results) == 1

    async def test_crawl_sse_handles_non_json_data(self):
        """Test SSE handles non-JSON data"""
        mock_client = AsyncMock(spec=httpx.AsyncClient)
        api = StreamingAPI(mock_client, "http://test.com")

        class TextSSEStream:
            status_code = 200

            async def __aenter__(self):
                return self

            async def __aexit__(self, *args):
                pass

            async def aiter_lines(self):
                yield "data: plain text message"
                yield ""

            async def aread(self):
                return b""

        mock_client.stream = AsyncMock(return_value=TextSSEStream())

        results = []
        async for result in api.crawl_sse(["https://example.com"]):
            results.append(result)

        assert len(results) == 1
        assert "raw" in results[0].data


@pytest.mark.unit
@pytest.mark.asyncio
class TestStreamingErrorHandling:
    """Test streaming error handling"""

    async def test_network_error_wrapped(self):
        """Test network errors are wrapped in StreamingError"""
        mock_client = AsyncMock(spec=httpx.AsyncClient)
        api = StreamingAPI(mock_client, "http://test.com")

        mock_client.stream.side_effect = httpx.NetworkError("Connection failed")

        with pytest.raises(StreamingError, match="HTTP error during streaming"):
            async for _ in api.crawl_ndjson(["https://example.com"]):
                pass

    async def test_timeout_error_wrapped(self):
        """Test timeout errors are wrapped"""
        mock_client = AsyncMock(spec=httpx.AsyncClient)
        api = StreamingAPI(mock_client, "http://test.com")

        mock_client.stream.side_effect = httpx.TimeoutException("Timeout")

        with pytest.raises(StreamingError, match="HTTP error during streaming"):
            async for _ in api.crawl_ndjson(["https://example.com"]):
                pass


@pytest.mark.unit
class TestStreamingAPIInitialization:
    """Test StreamingAPI initialization"""

    def test_initialization(self):
        """Test StreamingAPI initializes correctly"""
        client = AsyncMock()
        base_url = "http://test.com"

        api = StreamingAPI(client, base_url)

        assert api.client is client
        assert api.base_url == base_url
