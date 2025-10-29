"""
Performance tests for load handling

Tests concurrent request handling, throughput, and resource usage.
"""

import pytest
import asyncio
import time
from unittest.mock import AsyncMock, Mock

from riptide_sdk import RipTideClient


@pytest.mark.performance
@pytest.mark.asyncio
class TestConcurrentLoad:
    """Test handling of concurrent requests"""

    async def test_concurrent_requests_10(
        self, sample_crawl_response, performance_tracker, mocker
    ):
        """Test handling 10 concurrent requests"""
        async with RipTideClient(max_connections=20) as client:
            mock_response = Mock()
            mock_response.json.return_value = sample_crawl_response
            mock_response.raise_for_status = Mock()

            mocker.patch.object(
                client._client,
                "post",
                new_callable=AsyncMock,
                return_value=mock_response,
            )

            start = time.time()

            tasks = [
                client.crawl.batch([f"https://example{i}.com"])
                for i in range(10)
            ]

            results = await asyncio.gather(*tasks)

            duration_ms = (time.time() - start) * 1000

            performance_tracker.record(
                "concurrent_10", duration_ms, requests=10
            )

            assert len(results) == 10
            assert duration_ms < 5000  # Should complete in <5s

    async def test_concurrent_requests_50(
        self, sample_crawl_response, performance_tracker, mocker
    ):
        """Test handling 50 concurrent requests"""
        async with RipTideClient(max_connections=100) as client:
            mock_response = Mock()
            mock_response.json.return_value = sample_crawl_response
            mock_response.raise_for_status = Mock()

            mocker.patch.object(
                client._client,
                "post",
                new_callable=AsyncMock,
                return_value=mock_response,
            )

            start = time.time()

            tasks = [
                client.crawl.batch([f"https://example{i}.com"])
                for i in range(50)
            ]

            results = await asyncio.gather(*tasks)

            duration_ms = (time.time() - start) * 1000

            performance_tracker.record(
                "concurrent_50", duration_ms, requests=50
            )

            assert len(results) == 50
            # Allow more time for 50 requests
            assert duration_ms < 10000

    async def test_concurrent_requests_100(
        self, sample_crawl_response, performance_tracker, mocker
    ):
        """Test handling 100 concurrent requests"""
        async with RipTideClient(max_connections=150) as client:
            mock_response = Mock()
            mock_response.json.return_value = sample_crawl_response
            mock_response.raise_for_status = Mock()

            mocker.patch.object(
                client._client,
                "post",
                new_callable=AsyncMock,
                return_value=mock_response,
            )

            start = time.time()

            tasks = [
                client.crawl.batch([f"https://example{i}.com"])
                for i in range(100)
            ]

            results = await asyncio.gather(*tasks)

            duration_ms = (time.time() - start) * 1000

            performance_tracker.record(
                "concurrent_100", duration_ms, requests=100
            )

            assert len(results) == 100


@pytest.mark.performance
@pytest.mark.asyncio
class TestSequentialPerformance:
    """Test sequential request performance"""

    async def test_sequential_requests_time(
        self, sample_crawl_response, performance_tracker, mocker
    ):
        """Test sequential request timing"""
        async with RipTideClient() as client:
            mock_response = Mock()
            mock_response.json.return_value = sample_crawl_response
            mock_response.raise_for_status = Mock()

            mocker.patch.object(
                client._client,
                "post",
                new_callable=AsyncMock,
                return_value=mock_response,
            )

            durations = []

            for i in range(10):
                start = time.time()
                await client.crawl.batch([f"https://example{i}.com"])
                duration_ms = (time.time() - start) * 1000
                durations.append(duration_ms)

            avg_duration = sum(durations) / len(durations)

            performance_tracker.record(
                "sequential_avg", avg_duration, count=10
            )

            # Each request should be reasonably fast
            assert avg_duration < 1000


@pytest.mark.performance
@pytest.mark.asyncio
class TestStreamingPerformance:
    """Test streaming performance"""

    async def test_ndjson_streaming_throughput(
        self, mock_ndjson_stream, performance_tracker, mocker
    ):
        """Test NDJSON streaming throughput"""
        async with RipTideClient() as client:
            # Create 1000 items
            test_data = [
                {"url": f"https://example{i}.com", "status": 200}
                for i in range(1000)
            ]

            mocker.patch.object(
                client._client,
                "stream",
                new_callable=AsyncMock,
                return_value=await mock_ndjson_stream(test_data),
            )

            start = time.time()

            count = 0
            async for result in client.streaming.crawl_ndjson(
                ["https://example.com"]
            ):
                count += 1

            duration_ms = (time.time() - start) * 1000

            performance_tracker.record(
                "streaming_1000", duration_ms, items=count
            )

            assert count == 1000
            # Should process 1000 items quickly
            assert duration_ms < 3000


@pytest.mark.performance
@pytest.mark.asyncio
class TestConnectionPooling:
    """Test connection pooling performance"""

    async def test_connection_reuse(
        self, sample_crawl_response, mocker
    ):
        """Test that connections are reused efficiently"""
        async with RipTideClient(max_connections=10) as client:
            mock_response = Mock()
            mock_response.json.return_value = sample_crawl_response
            mock_response.raise_for_status = Mock()

            mocker.patch.object(
                client._client,
                "post",
                new_callable=AsyncMock,
                return_value=mock_response,
            )

            # Make 20 requests with only 10 max connections
            # This forces connection reuse
            tasks = [
                client.crawl.batch([f"https://example{i}.com"])
                for i in range(20)
            ]

            results = await asyncio.gather(*tasks)

            assert len(results) == 20


@pytest.mark.performance
@pytest.mark.asyncio
class TestMemoryUsage:
    """Test memory efficiency"""

    async def test_large_response_handling(self, mocker):
        """Test handling of large responses"""
        async with RipTideClient() as client:
            # Create a large response
            large_response_data = {
                "total_urls": 100,
                "successful": 100,
                "failed": 0,
                "from_cache": 0,
                "results": [
                    {
                        "url": f"https://example{i}.com",
                        "status": 200,
                        "from_cache": False,
                        "gate_decision": "raw",
                        "quality_score": 0.95,
                        "processing_time_ms": 50,
                        "document": {
                            "text": "x" * 10000,  # 10KB per result
                        },
                    }
                    for i in range(100)
                ],
                "statistics": {
                    "total_processing_time_ms": 5000,
                    "avg_processing_time_ms": 50.0,
                    "gate_decisions": {"raw": 100, "probes_first": 0, "headless": 0, "cached": 0},
                    "cache_hit_rate": 0.0,
                },
            }

            mock_response = Mock()
            mock_response.json.return_value = large_response_data
            mock_response.raise_for_status = Mock()

            mocker.patch.object(
                client._client,
                "post",
                new_callable=AsyncMock,
                return_value=mock_response,
            )

            result = await client.crawl.batch(
                [f"https://example{i}.com" for i in range(100)]
            )

            # Should handle large response without issues
            assert len(result.results) == 100


@pytest.mark.performance
@pytest.mark.slow
@pytest.mark.asyncio
class TestSustainedLoad:
    """Test sustained load over time"""

    async def test_sustained_requests(
        self, sample_crawl_response, performance_tracker, mocker
    ):
        """Test sustained request load (100 requests over time)"""
        async with RipTideClient(max_connections=50) as client:
            mock_response = Mock()
            mock_response.json.return_value = sample_crawl_response
            mock_response.raise_for_status = Mock()

            mocker.patch.object(
                client._client,
                "post",
                new_callable=AsyncMock,
                return_value=mock_response,
            )

            start = time.time()

            # Send requests in batches
            batch_size = 10
            total_requests = 100

            for batch_num in range(total_requests // batch_size):
                tasks = [
                    client.crawl.batch([f"https://example{i}.com"])
                    for i in range(
                        batch_num * batch_size, (batch_num + 1) * batch_size
                    )
                ]

                await asyncio.gather(*tasks)

                # Small delay between batches
                await asyncio.sleep(0.1)

            duration_ms = (time.time() - start) * 1000

            performance_tracker.record(
                "sustained_100", duration_ms, requests=total_requests
            )

            # Should complete all requests
            assert duration_ms < 30000  # 30 seconds


@pytest.mark.performance
class TestPerformanceReporting:
    """Test performance tracking and reporting"""

    def test_performance_tracker_stats(self, performance_tracker):
        """Test performance tracker statistics"""
        # Record some measurements
        performance_tracker.record("test_op", 100)
        performance_tracker.record("test_op", 150)
        performance_tracker.record("test_op", 125)

        stats = performance_tracker.get_stats("test_op")

        assert stats["count"] == 3
        assert stats["min"] == 100
        assert stats["max"] == 150
        assert stats["avg"] == 125.0
        assert stats["total"] == 375

    def test_performance_tracker_multiple_operations(
        self, performance_tracker
    ):
        """Test tracking multiple operation types"""
        performance_tracker.record("op1", 100)
        performance_tracker.record("op2", 200)

        stats1 = performance_tracker.get_stats("op1")
        stats2 = performance_tracker.get_stats("op2")

        assert stats1["count"] == 1
        assert stats2["count"] == 1
        assert stats1["avg"] == 100
        assert stats2["avg"] == 200
