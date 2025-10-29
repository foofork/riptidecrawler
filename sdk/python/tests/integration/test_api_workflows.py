"""
Integration tests for complete API workflows

Tests end-to-end scenarios combining multiple SDK features.
"""

import pytest
import asyncio
from unittest.mock import AsyncMock, Mock, patch

from riptide_sdk import RipTideClient, RipTideClientBuilder, CrawlOptions, CacheMode
from riptide_sdk.models import DomainProfile, ProfileConfig, StealthLevel


@pytest.mark.integration
@pytest.mark.asyncio
class TestCompleteCrawlWorkflow:
    """Test complete crawl workflow from start to finish"""

    async def test_batch_crawl_workflow(self, sample_crawl_response, mocker):
        """Test complete batch crawl workflow"""
        async with RipTideClient() as client:
            # Mock the HTTP response
            mock_response = Mock()
            mock_response.status_code = 200
            mock_response.json.return_value = sample_crawl_response
            mock_response.raise_for_status = Mock()

            mocker.patch.object(
                client._client,
                "post",
                new_callable=AsyncMock,
                return_value=mock_response,
            )

            # Execute crawl
            result = await client.crawl.batch(
                ["https://example.com", "https://test.com"]
            )

            # Verify results
            assert result.total_urls == 3
            assert result.successful == 2
            assert len(result.results) == 3

            # Test formatter methods
            summary = result.to_summary()
            assert "Total: 3 URLs" in summary

            markdown = result.to_markdown()
            assert "# Crawl Results" in markdown

    async def test_streaming_workflow(self, mock_ndjson_stream, mocker):
        """Test streaming crawl workflow"""
        async with RipTideClient() as client:
            test_data = [
                {"url": "https://example.com", "status": 200},
                {"url": "https://test.com", "status": 200},
            ]

            mocker.patch.object(
                client._client,
                "stream",
                new_callable=AsyncMock,
                return_value=await mock_ndjson_stream(test_data),
            )

            results = []
            async for result in client.streaming.crawl_ndjson(
                ["https://example.com"]
            ):
                results.append(result)

            assert len(results) == 2

    async def test_crawl_with_options(self, sample_crawl_response, mocker):
        """Test crawl with custom options"""
        async with RipTideClient() as client:
            mock_response = Mock()
            mock_response.json.return_value = sample_crawl_response
            mock_response.raise_for_status = Mock()

            mocker.patch.object(
                client._client, "post", new_callable=AsyncMock, return_value=mock_response
            )

            options = CrawlOptions(
                cache_mode=CacheMode.READ_WRITE, concurrency=10
            )

            result = await client.crawl.batch(
                ["https://example.com"], options=options
            )

            assert result is not None


@pytest.mark.integration
@pytest.mark.asyncio
class TestDomainProfileWorkflow:
    """Test domain profile management workflow"""

    async def test_create_and_retrieve_profile(self, sample_domain_profile, mocker):
        """Test creating and retrieving a domain profile"""
        async with RipTideClient() as client:
            # Mock create response
            create_response = Mock()
            create_response.json.return_value = sample_domain_profile
            create_response.raise_for_status = Mock()

            # Mock get response
            get_response = Mock()
            get_response.json.return_value = sample_domain_profile
            get_response.raise_for_status = Mock()

            mock_post = mocker.patch.object(
                client._client,
                "post",
                new_callable=AsyncMock,
                return_value=create_response,
            )
            mock_get = mocker.patch.object(
                client._client,
                "get",
                new_callable=AsyncMock,
                return_value=get_response,
            )

            # Create profile
            config = ProfileConfig(stealth_level=StealthLevel.MEDIUM)
            profile = await client.profiles.create("example.com", config=config)

            assert profile.domain == "example.com"

            # Retrieve profile
            retrieved = await client.profiles.get("example.com")

            assert retrieved.domain == "example.com"
            assert retrieved.config.stealth_level == StealthLevel.MEDIUM


@pytest.mark.integration
@pytest.mark.asyncio
class TestBuilderWorkflow:
    """Test client builder workflow"""

    async def test_builder_creates_functional_client(
        self, sample_crawl_response, mocker
    ):
        """Test builder creates a working client"""
        # Build client with custom config
        client = (
            RipTideClientBuilder()
            .with_base_url("http://localhost:8080")
            .with_api_key("test-key")
            .with_timeout(60.0)
            .with_max_connections(150)
            .build()
        )

        # Mock HTTP response
        mock_response = Mock()
        mock_response.json.return_value = sample_crawl_response
        mock_response.raise_for_status = Mock()

        mocker.patch.object(
            client._client, "post", new_callable=AsyncMock, return_value=mock_response
        )

        # Use the client
        async with client:
            result = await client.crawl.batch(["https://example.com"])
            assert result.total_urls == 3

        # Verify configuration was applied
        assert client.base_url == "http://localhost:8080"
        assert client.api_key == "test-key"


@pytest.mark.integration
@pytest.mark.asyncio
class TestErrorHandlingWorkflow:
    """Test error handling across workflows"""

    async def test_retry_on_server_error(self, mocker):
        """Test retry behavior on server errors"""
        async with RipTideClient() as client:
            # First call fails, second succeeds
            error_response = Mock()
            error_response.status_code = 500
            error_response.raise_for_status.side_effect = Exception("Server Error")

            success_response = Mock()
            success_response.json.return_value = {"status": "healthy"}
            success_response.raise_for_status = Mock()

            mock_get = mocker.patch.object(
                client._client,
                "get",
                new_callable=AsyncMock,
                side_effect=[error_response, success_response],
            )

            # Implement manual retry logic
            max_retries = 2
            for attempt in range(max_retries):
                try:
                    result = await client.health_check()
                    break
                except Exception:
                    if attempt == max_retries - 1:
                        raise
                    await asyncio.sleep(0.1)

            assert result == {"status": "healthy"}


@pytest.mark.integration
@pytest.mark.asyncio
class TestConcurrentOperations:
    """Test concurrent API operations"""

    async def test_concurrent_crawls(self, sample_crawl_response, mocker):
        """Test multiple concurrent crawl requests"""
        async with RipTideClient() as client:
            mock_response = Mock()
            mock_response.json.return_value = sample_crawl_response
            mock_response.raise_for_status = Mock()

            mocker.patch.object(
                client._client, "post", new_callable=AsyncMock, return_value=mock_response
            )

            # Execute multiple crawls concurrently
            tasks = [
                client.crawl.batch([f"https://example{i}.com"])
                for i in range(5)
            ]

            results = await asyncio.gather(*tasks)

            assert len(results) == 5
            assert all(r.total_urls == 3 for r in results)

    async def test_mixed_concurrent_operations(
        self, sample_crawl_response, sample_domain_profile, mocker
    ):
        """Test different operations running concurrently"""
        async with RipTideClient() as client:
            # Mock responses
            crawl_response = Mock()
            crawl_response.json.return_value = sample_crawl_response
            crawl_response.raise_for_status = Mock()

            profile_response = Mock()
            profile_response.json.return_value = sample_domain_profile
            profile_response.raise_for_status = Mock()

            health_response = Mock()
            health_response.json.return_value = {"status": "healthy"}
            health_response.raise_for_status = Mock()

            mocker.patch.object(
                client._client,
                "post",
                new_callable=AsyncMock,
                return_value=crawl_response,
            )
            mocker.patch.object(
                client._client,
                "get",
                new_callable=AsyncMock,
                side_effect=[profile_response, health_response],
            )

            # Run different operations concurrently
            crawl_task = client.crawl.batch(["https://example.com"])
            profile_task = client.profiles.get("example.com")
            health_task = client.health_check()

            crawl_result, profile_result, health_result = await asyncio.gather(
                crawl_task, profile_task, health_task
            )

            assert crawl_result.total_urls == 3
            assert profile_result.domain == "example.com"
            assert health_result["status"] == "healthy"


@pytest.mark.integration
@pytest.mark.asyncio
class TestContextManagerBehavior:
    """Test context manager behavior in various scenarios"""

    async def test_multiple_sequential_contexts(self, sample_crawl_response, mocker):
        """Test using client in multiple sequential contexts"""
        mock_response = Mock()
        mock_response.json.return_value = sample_crawl_response
        mock_response.raise_for_status = Mock()

        for i in range(3):
            async with RipTideClient() as client:
                mocker.patch.object(
                    client._client,
                    "post",
                    new_callable=AsyncMock,
                    return_value=mock_response,
                )

                result = await client.crawl.batch([f"https://example{i}.com"])
                assert result is not None

    async def test_nested_contexts(self):
        """Test nested context managers (unusual but valid)"""
        async with RipTideClient() as client1:
            async with RipTideClient() as client2:
                assert client1 is not client2
                assert client1._client is not client2._client


@pytest.mark.integration
@pytest.mark.asyncio
class TestRealWorldScenarios:
    """Test realistic usage scenarios"""

    async def test_web_scraping_pipeline(
        self, sample_crawl_response, sample_domain_profile, mocker
    ):
        """Test a complete web scraping pipeline"""
        async with RipTideClient() as client:
            # Setup mocks
            profile_response = Mock()
            profile_response.json.return_value = sample_domain_profile
            profile_response.raise_for_status = Mock()

            crawl_response = Mock()
            crawl_response.json.return_value = sample_crawl_response
            crawl_response.raise_for_status = Mock()

            mocker.patch.object(
                client._client,
                "post",
                new_callable=AsyncMock,
                return_value=crawl_response,
            )
            mocker.patch.object(
                client._client,
                "get",
                new_callable=AsyncMock,
                return_value=profile_response,
            )

            # Step 1: Create domain profile
            config = ProfileConfig(stealth_level=StealthLevel.HIGH, rate_limit=1.0)
            profile = await client.profiles.create("example.com", config=config)

            assert profile is not None

            # Step 2: Batch crawl URLs
            urls = [f"https://example.com/page{i}" for i in range(10)]
            result = await client.crawl.batch(urls)

            assert result is not None

            # Step 3: Process results
            successful_urls = [
                r.url for r in result.results if r.status == 200
            ]

            assert len(successful_urls) > 0

    async def test_api_health_monitoring(self, mocker):
        """Test API health monitoring workflow"""
        async with RipTideClient() as client:
            mock_response = Mock()
            mock_response.json.return_value = {
                "status": "healthy",
                "uptime": 12345,
                "version": "1.0.0",
            }
            mock_response.raise_for_status = Mock()

            mocker.patch.object(
                client._client,
                "get",
                new_callable=AsyncMock,
                return_value=mock_response,
            )

            # Poll health endpoint
            for _ in range(3):
                health = await client.health_check()
                assert health["status"] == "healthy"
                await asyncio.sleep(0.01)
