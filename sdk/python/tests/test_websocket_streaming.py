"""
Tests for WebSocket streaming functionality

Tests the WebSocket streaming implementation including:
- Basic WebSocket connection and streaming
- Message callbacks
- Error handling
- Connection health monitoring
- Protocol compliance
"""

import pytest
import asyncio
from unittest.mock import AsyncMock, MagicMock, patch
from riptide_sdk.endpoints.streaming import StreamingAPI, WEBSOCKETS_AVAILABLE
from riptide_sdk.models import CrawlOptions, CacheMode, StreamingResult
from riptide_sdk.exceptions import StreamingError, ValidationError


# Skip all tests if websockets is not available
pytestmark = pytest.mark.skipif(
    not WEBSOCKETS_AVAILABLE, reason="websockets library not installed"
)


@pytest.fixture
def streaming_api():
    """Create a StreamingAPI instance for testing."""
    mock_client = AsyncMock()
    return StreamingAPI(mock_client, "http://localhost:3000/api/v1")


class TestWebSocketStreaming:
    """Test WebSocket streaming functionality."""

    @pytest.mark.asyncio
    async def test_websocket_basic_streaming(self, streaming_api):
        """Test basic WebSocket streaming with mocked connection."""
        if not WEBSOCKETS_AVAILABLE:
            pytest.skip("websockets not available")

        # Mock WebSocket messages
        welcome_msg = {
            "message_type": "welcome",
            "data": {
                "session_id": "test-session-123",
                "server_time": "2024-01-01T00:00:00Z",
                "protocol_version": "1.0",
            },
            "timestamp": "2024-01-01T00:00:00Z",
        }

        metadata_msg = {
            "message_type": "metadata",
            "data": {
                "total_urls": 2,
                "session_id": "test-session-123",
                "stream_type": "crawl",
            },
            "timestamp": "2024-01-01T00:00:01Z",
        }

        result_msg = {
            "message_type": "result",
            "data": {
                "index": 0,
                "result": {
                    "url": "https://example.com",
                    "status": 200,
                    "from_cache": False,
                    "gate_decision": "raw",
                    "quality_score": 0.95,
                    "processing_time_ms": 100,
                    "document": {"text": "Example content"},
                    "error": None,
                    "cache_key": "key1",
                },
                "progress": {"completed": 1, "total": 2, "success_rate": 1.0},
            },
            "timestamp": "2024-01-01T00:00:02Z",
        }

        summary_msg = {
            "message_type": "summary",
            "data": {
                "total_urls": 2,
                "successful": 2,
                "failed": 0,
                "total_processing_time_ms": 200,
            },
            "timestamp": "2024-01-01T00:00:03Z",
        }

        import json

        messages = [
            json.dumps(welcome_msg),
            json.dumps(metadata_msg),
            json.dumps(result_msg),
            json.dumps(summary_msg),
        ]

        # Mock websockets.connect
        with patch("websockets.connect") as mock_connect:
            mock_ws = AsyncMock()
            mock_ws.recv = AsyncMock(side_effect=messages)
            mock_ws.send = AsyncMock()
            mock_ws.__aenter__ = AsyncMock(return_value=mock_ws)
            mock_ws.__aexit__ = AsyncMock()

            # Make the async iterator work properly
            async def mock_aiter():
                for msg in messages:
                    yield msg

            mock_ws.__aiter__ = lambda self: mock_aiter()

            mock_connect.return_value = mock_ws

            urls = ["https://example.com", "https://httpbin.org"]
            results = []

            async for result in streaming_api.crawl_websocket(urls):
                results.append(result)

            # Verify we got all expected messages
            assert len(results) == 4
            assert results[0].event_type == "welcome"
            assert results[1].event_type == "metadata"
            assert results[2].event_type == "result"
            assert results[3].event_type == "summary"

            # Verify crawl request was sent
            mock_ws.send.assert_called_once()
            sent_data = json.loads(mock_ws.send.call_args[0][0])
            assert sent_data["request_type"] == "crawl"
            assert sent_data["data"]["urls"] == urls

    @pytest.mark.asyncio
    async def test_websocket_with_callback(self, streaming_api):
        """Test WebSocket streaming with message callback."""
        if not WEBSOCKETS_AVAILABLE:
            pytest.skip("websockets not available")

        callback_results = []

        async def on_message(result: StreamingResult):
            callback_results.append(result)

        import json

        messages = [
            json.dumps(
                {
                    "message_type": "welcome",
                    "data": {"session_id": "test"},
                    "timestamp": "2024-01-01T00:00:00Z",
                }
            ),
            json.dumps(
                {
                    "message_type": "summary",
                    "data": {"total_urls": 0},
                    "timestamp": "2024-01-01T00:00:01Z",
                }
            ),
        ]

        with patch("websockets.connect") as mock_connect:
            mock_ws = AsyncMock()
            mock_ws.recv = AsyncMock(side_effect=messages)
            mock_ws.send = AsyncMock()
            mock_ws.__aenter__ = AsyncMock(return_value=mock_ws)
            mock_ws.__aexit__ = AsyncMock()

            async def mock_aiter():
                for msg in messages:
                    yield msg

            mock_ws.__aiter__ = lambda self: mock_aiter()
            mock_connect.return_value = mock_ws

            result_count = 0
            async for result in streaming_api.crawl_websocket(
                ["https://example.com"], on_message=on_message
            ):
                result_count += 1

            # Verify callback was called for each message
            assert len(callback_results) == 2
            assert result_count == 2

    @pytest.mark.asyncio
    async def test_websocket_empty_urls(self, streaming_api):
        """Test WebSocket with empty URLs list."""
        with pytest.raises(ValidationError, match="URLs list cannot be empty"):
            async for _ in streaming_api.crawl_websocket([]):
                pass

    @pytest.mark.asyncio
    async def test_websocket_connection_error(self, streaming_api):
        """Test WebSocket connection error handling."""
        if not WEBSOCKETS_AVAILABLE:
            pytest.skip("websockets not available")

        with patch("websockets.connect") as mock_connect:
            mock_connect.side_effect = Exception("Connection failed")

            with pytest.raises(StreamingError, match="Unexpected error"):
                async for _ in streaming_api.crawl_websocket(["https://example.com"]):
                    pass

    @pytest.mark.asyncio
    async def test_websocket_invalid_json(self, streaming_api):
        """Test WebSocket with invalid JSON message."""
        if not WEBSOCKETS_AVAILABLE:
            pytest.skip("websockets not available")

        import json

        messages = [
            json.dumps({"message_type": "welcome", "data": {}}),
            "invalid json{",  # Invalid JSON
            json.dumps({"message_type": "summary", "data": {}}),
        ]

        with patch("websockets.connect") as mock_connect:
            mock_ws = AsyncMock()
            mock_ws.recv = AsyncMock(side_effect=messages)
            mock_ws.send = AsyncMock()
            mock_ws.__aenter__ = AsyncMock(return_value=mock_ws)
            mock_ws.__aexit__ = AsyncMock()

            async def mock_aiter():
                for msg in messages:
                    yield msg

            mock_ws.__aiter__ = lambda self: mock_aiter()
            mock_connect.return_value = mock_ws

            results = []
            async for result in streaming_api.crawl_websocket(["https://example.com"]):
                results.append(result)

            # Should have error result for invalid JSON
            error_results = [r for r in results if r.event_type == "error"]
            assert len(error_results) >= 1


class TestWebSocketUtilities:
    """Test WebSocket utility methods."""

    @pytest.mark.asyncio
    async def test_ping_websocket(self, streaming_api):
        """Test WebSocket ping functionality."""
        if not WEBSOCKETS_AVAILABLE:
            pytest.skip("websockets not available")

        import json

        messages = [
            json.dumps(
                {"message_type": "welcome", "data": {"session_id": "test-session"}}
            ),
            json.dumps(
                {
                    "message_type": "pong",
                    "data": {
                        "timestamp": "2024-01-01T00:00:00Z",
                        "session_id": "test-session",
                    },
                }
            ),
        ]

        with patch("websockets.connect") as mock_connect:
            mock_ws = AsyncMock()
            mock_ws.recv = AsyncMock(side_effect=messages)
            mock_ws.send = AsyncMock()
            mock_ws.__aenter__ = AsyncMock(return_value=mock_ws)
            mock_ws.__aexit__ = AsyncMock()
            mock_connect.return_value = mock_ws

            result = await streaming_api.ping_websocket()

            assert result["success"] is True
            assert "latency_ms" in result
            assert result["session_id"] == "test-session"
            assert result["server_time"] == "2024-01-01T00:00:00Z"

    @pytest.mark.asyncio
    async def test_get_websocket_status(self, streaming_api):
        """Test getting WebSocket status."""
        if not WEBSOCKETS_AVAILABLE:
            pytest.skip("websockets not available")

        import json

        messages = [
            json.dumps({"message_type": "welcome", "data": {}}),
            json.dumps(
                {
                    "message_type": "status",
                    "data": {
                        "session_id": "test-session",
                        "is_healthy": True,
                        "message_count": 42,
                        "connected_duration_ms": 5000,
                    },
                }
            ),
        ]

        with patch("websockets.connect") as mock_connect:
            mock_ws = AsyncMock()
            mock_ws.recv = AsyncMock(side_effect=messages)
            mock_ws.send = AsyncMock()
            mock_ws.__aenter__ = AsyncMock(return_value=mock_ws)
            mock_ws.__aexit__ = AsyncMock()
            mock_connect.return_value = mock_ws

            status = await streaming_api.get_websocket_status()

            assert status["session_id"] == "test-session"
            assert status["is_healthy"] is True
            assert status["message_count"] == 42

    @pytest.mark.asyncio
    async def test_websocket_with_options(self, streaming_api):
        """Test WebSocket streaming with crawl options."""
        if not WEBSOCKETS_AVAILABLE:
            pytest.skip("websockets not available")

        import json

        options = CrawlOptions(cache_mode=CacheMode.READ, concurrency=3, timeout_secs=60)

        messages = [
            json.dumps({"message_type": "welcome", "data": {}}),
            json.dumps({"message_type": "summary", "data": {}}),
        ]

        with patch("websockets.connect") as mock_connect:
            mock_ws = AsyncMock()
            mock_ws.recv = AsyncMock(side_effect=messages)
            mock_ws.send = AsyncMock()
            mock_ws.__aenter__ = AsyncMock(return_value=mock_ws)
            mock_ws.__aexit__ = AsyncMock()

            async def mock_aiter():
                for msg in messages:
                    yield msg

            mock_ws.__aiter__ = lambda self: mock_aiter()
            mock_connect.return_value = mock_ws

            async for _ in streaming_api.crawl_websocket(
                ["https://example.com"], options=options
            ):
                pass

            # Verify options were sent
            sent_data = json.loads(mock_ws.send.call_args[0][0])
            assert "options" in sent_data["data"]
            assert sent_data["data"]["options"]["cache_mode"] == "read"
            assert sent_data["data"]["options"]["concurrency"] == 3


class TestWebSocketImportError:
    """Test behavior when websockets library is not available."""

    @pytest.mark.asyncio
    async def test_websocket_not_installed(self, streaming_api, monkeypatch):
        """Test graceful handling when websockets is not installed."""
        # Temporarily mark websockets as unavailable
        monkeypatch.setattr("riptide_sdk.endpoints.streaming.WEBSOCKETS_AVAILABLE", False)

        with pytest.raises(ImportError, match="websockets library is required"):
            async for _ in streaming_api.crawl_websocket(["https://example.com"]):
                pass

    @pytest.mark.asyncio
    async def test_ping_without_websockets(self, streaming_api, monkeypatch):
        """Test ping fails gracefully without websockets."""
        monkeypatch.setattr("riptide_sdk.endpoints.streaming.WEBSOCKETS_AVAILABLE", False)

        with pytest.raises(ImportError, match="websockets library is required"):
            await streaming_api.ping_websocket()


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
