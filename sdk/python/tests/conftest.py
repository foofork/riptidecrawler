"""
Pytest configuration and fixtures for RipTide SDK tests

Provides shared fixtures for HTTP mocking, async utilities, and test data.
"""

import pytest
import pytest_asyncio
import httpx
import json
from typing import Dict, Any, List
from unittest.mock import AsyncMock, Mock, patch


# ============================================================================
# Test Data Fixtures
# ============================================================================

@pytest.fixture
def sample_urls() -> List[str]:
    """Sample URLs for testing"""
    return [
        "https://example.com",
        "https://test.com",
        "https://demo.org",
    ]


@pytest.fixture
def sample_crawl_response() -> Dict[str, Any]:
    """Sample crawl API response"""
    return {
        "total_urls": 3,
        "successful": 2,
        "failed": 1,
        "from_cache": 1,
        "results": [
            {
                "url": "https://example.com",
                "status": 200,
                "from_cache": True,
                "gate_decision": "raw",
                "quality_score": 0.95,
                "processing_time_ms": 45,
                "cache_key": "abc123",
                "document": {
                    "html": "<html><body>Test</body></html>",
                    "text": "Test",
                    "markdown": "# Test",
                    "metadata": {"title": "Test Page"},
                    "links": ["https://example.com/link"],
                },
            },
            {
                "url": "https://test.com",
                "status": 200,
                "from_cache": False,
                "gate_decision": "probes_first",
                "quality_score": 0.88,
                "processing_time_ms": 123,
                "cache_key": "def456",
                "document": {
                    "text": "Another test",
                },
            },
            {
                "url": "https://failed.com",
                "status": 500,
                "from_cache": False,
                "gate_decision": "raw",
                "quality_score": 0.0,
                "processing_time_ms": 50,
                "cache_key": "",
                "error": {
                    "error_type": "server_error",
                    "message": "Internal server error",
                    "retryable": True,
                },
            },
        ],
        "statistics": {
            "total_processing_time_ms": 218,
            "avg_processing_time_ms": 72.7,
            "gate_decisions": {
                "raw": 2,
                "probes_first": 1,
                "headless": 0,
                "cached": 1,
            },
            "cache_hit_rate": 0.333,
        },
    }


@pytest.fixture
def sample_domain_profile() -> Dict[str, Any]:
    """Sample domain profile response"""
    return {
        "domain": "example.com",
        "config": {
            "stealth_level": "medium",
            "rate_limit": 2.0,
            "respect_robots_txt": True,
            "ua_strategy": "rotate",
            "confidence_threshold": 0.8,
            "enable_javascript": False,
            "request_timeout_secs": 30,
        },
        "metadata": {
            "description": "Test domain",
            "tags": ["test", "example"],
            "author": "test-user",
        },
        "created_at": "2024-01-01T00:00:00Z",
        "updated_at": "2024-01-02T00:00:00Z",
    }


@pytest.fixture
def sample_engine_stats() -> Dict[str, Any]:
    """Sample engine statistics response"""
    return {
        "total_decisions": 100,
        "raw_count": 45,
        "probes_first_count": 35,
        "headless_count": 20,
        "probe_first_enabled": True,
    }


@pytest.fixture
def sample_streaming_result() -> Dict[str, Any]:
    """Sample streaming result"""
    return {
        "url": "https://example.com",
        "status": 200,
        "text": "Sample content",
        "processing_time_ms": 156,
    }


# ============================================================================
# HTTP Client Mocking
# ============================================================================

@pytest.fixture
def mock_httpx_client():
    """Mock httpx.AsyncClient for testing"""
    client = AsyncMock(spec=httpx.AsyncClient)

    # Default successful response
    mock_response = Mock()
    mock_response.status_code = 200
    mock_response.json.return_value = {"status": "healthy"}
    mock_response.raise_for_status = Mock()

    client.get = AsyncMock(return_value=mock_response)
    client.post = AsyncMock(return_value=mock_response)
    client.put = AsyncMock(return_value=mock_response)
    client.delete = AsyncMock(return_value=mock_response)
    client.aclose = AsyncMock()

    return client


@pytest_asyncio.fixture
async def mock_response_factory():
    """Factory for creating mock HTTP responses"""

    def create_response(
        status_code: int = 200,
        json_data: Dict[str, Any] = None,
        text: str = "",
        headers: Dict[str, str] = None,
    ):
        response = Mock(spec=httpx.Response)
        response.status_code = status_code
        response.headers = headers or {}

        if json_data:
            response.json.return_value = json_data

        response.text = text
        response.raise_for_status = Mock()

        if status_code >= 400:
            response.raise_for_status.side_effect = httpx.HTTPStatusError(
                f"HTTP {status_code}",
                request=Mock(),
                response=response,
            )

        return response

    return create_response


# ============================================================================
# Async Utilities
# ============================================================================

@pytest.fixture
def event_loop_policy():
    """Use default event loop policy"""
    import asyncio
    return asyncio.DefaultEventLoopPolicy()


@pytest_asyncio.fixture
async def async_client_context():
    """Context for async client testing"""

    class AsyncClientContext:
        def __init__(self):
            self.client = None
            self.base_url = "http://test.localhost:8080"

        async def create_client(self, **kwargs):
            """Create a test client"""
            from riptide_sdk import RipTideClient

            self.client = RipTideClient(
                base_url=self.base_url,
                **kwargs,
            )
            return self.client

        async def cleanup(self):
            """Cleanup client resources"""
            if self.client:
                await self.client.close()

    ctx = AsyncClientContext()
    yield ctx
    await ctx.cleanup()


# ============================================================================
# Streaming Mocks
# ============================================================================

@pytest.fixture
def mock_ndjson_stream():
    """Mock NDJSON streaming response"""

    async def create_stream(data_items: List[Dict[str, Any]]):
        """Create an async iterator that yields NDJSON lines"""

        class MockStream:
            def __init__(self, items):
                self.items = items
                self.status_code = 200

            async def __aenter__(self):
                return self

            async def __aexit__(self, *args):
                pass

            async def aiter_lines(self):
                for item in self.items:
                    yield json.dumps(item)

            async def aread(self):
                return b""

        return MockStream(data_items)

    return create_stream


@pytest.fixture
def mock_sse_stream():
    """Mock Server-Sent Events streaming response"""

    async def create_stream(events: List[Dict[str, Any]]):
        """Create an async iterator that yields SSE events"""

        class MockStream:
            def __init__(self, event_data):
                self.event_data = event_data
                self.status_code = 200

            async def __aenter__(self):
                return self

            async def __aexit__(self, *args):
                pass

            async def aiter_lines(self):
                for event in self.event_data:
                    event_type = event.get("event_type", "message")
                    data = event.get("data", {})

                    yield f"event: {event_type}"
                    yield f"data: {json.dumps(data)}"
                    yield ""  # Empty line marks end of event

            async def aread(self):
                return b""

        return MockStream(events)

    return create_stream


# ============================================================================
# Performance Testing Utilities
# ============================================================================

@pytest.fixture
def performance_tracker():
    """Track performance metrics during tests"""

    class PerformanceTracker:
        def __init__(self):
            self.measurements = []

        def record(self, operation: str, duration_ms: float, **metadata):
            """Record a performance measurement"""
            self.measurements.append({
                "operation": operation,
                "duration_ms": duration_ms,
                **metadata,
            })

        def get_stats(self, operation: str = None):
            """Get statistics for measurements"""
            measurements = self.measurements
            if operation:
                measurements = [m for m in measurements if m["operation"] == operation]

            if not measurements:
                return None

            durations = [m["duration_ms"] for m in measurements]
            return {
                "count": len(durations),
                "min": min(durations),
                "max": max(durations),
                "avg": sum(durations) / len(durations),
                "total": sum(durations),
            }

    return PerformanceTracker()


# ============================================================================
# Error Simulation
# ============================================================================

@pytest.fixture
def error_simulator():
    """Simulate various error conditions"""

    class ErrorSimulator:
        @staticmethod
        def timeout_error():
            """Simulate a timeout"""
            return httpx.TimeoutException("Request timed out")

        @staticmethod
        def network_error():
            """Simulate a network error"""
            return httpx.NetworkError("Network unreachable")

        @staticmethod
        def http_error(status_code: int):
            """Simulate an HTTP error"""
            response = Mock()
            response.status_code = status_code
            response.text = f"HTTP {status_code} Error"
            return httpx.HTTPStatusError(
                f"HTTP {status_code}",
                request=Mock(),
                response=response,
            )

    return ErrorSimulator()


# ============================================================================
# Pytest Configuration
# ============================================================================

def pytest_configure(config):
    """Configure pytest with custom markers"""
    config.addinivalue_line(
        "markers", "unit: mark test as a unit test"
    )
    config.addinivalue_line(
        "markers", "integration: mark test as an integration test"
    )
    config.addinivalue_line(
        "markers", "performance: mark test as a performance test"
    )
    config.addinivalue_line(
        "markers", "slow: mark test as slow running"
    )
    config.addinivalue_line(
        "markers", "asyncio: mark test as async"
    )
