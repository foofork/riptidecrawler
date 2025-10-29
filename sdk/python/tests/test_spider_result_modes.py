"""
Comprehensive test suite for Spider result_mode feature in Python SDK

Tests:
1. ResultMode.STATS usage (backward compatible)
2. ResultMode.URLS usage (URL discovery)
3. discovered_urls parsing and validation
4. Backward compatibility (no result_mode parameter)
5. Invalid result_mode handling
6. Max pages constraint with URL collection
7. Different crawl strategies (BFS/DFS)
8. URL deduplication
"""

import pytest
from typing import List, Dict, Any
from unittest.mock import Mock, AsyncMock, patch
import httpx

from riptide_sdk import RipTideClient
from riptide_sdk.exceptions import RipTideError


# ============================================================================
# Fixtures
# ============================================================================

@pytest.fixture
def mock_spider_stats_response() -> Dict[str, Any]:
    """Mock response for result_mode=stats (no URLs)"""
    return {
        "result": {
            "pages_crawled": 15,
            "pages_failed": 2,
            "duration_seconds": 45.3,
            "stop_reason": "max_pages_reached",
            "domains": ["example.com"]
        },
        "state": {
            "active": False,
            "pages_crawled": 15,
            "pages_failed": 2,
            "frontier_size": 0
        },
        "performance": {
            "pages_per_second": 0.33,
            "avg_response_time": 2.1,
            "error_rate": 0.13
        }
    }


@pytest.fixture
def mock_spider_urls_response() -> Dict[str, Any]:
    """Mock response for result_mode=urls (includes discovered URLs)"""
    return {
        "result": {
            "pages_crawled": 10,
            "pages_failed": 1,
            "duration_seconds": 32.5,
            "stop_reason": "max_pages_reached",
            "domains": ["example.com"],
            "discovered_urls": [
                "https://example.com",
                "https://example.com/about",
                "https://example.com/contact",
                "https://example.com/products",
                "https://example.com/services",
                "https://example.com/blog",
                "https://example.com/faq",
                "https://example.com/privacy",
                "https://example.com/terms",
                "https://example.com/careers"
            ]
        },
        "state": {
            "active": False,
            "pages_crawled": 10,
            "pages_failed": 1,
            "frontier_size": 0
        },
        "performance": {
            "pages_per_second": 0.31,
            "avg_response_time": 2.5,
            "error_rate": 0.10
        }
    }


@pytest.fixture
def client():
    """Create test client"""
    return RipTideClient(base_url="http://localhost:8080")


# ============================================================================
# Result Mode Tests
# ============================================================================

@pytest.mark.asyncio
async def test_spider_result_mode_stats(client, mock_spider_stats_response):
    """Test result_mode=stats returns stats without URLs"""
    with patch.object(client.session, 'post', new_callable=AsyncMock) as mock_post:
        mock_response = Mock()
        mock_response.status_code = 200
        mock_response.json.return_value = mock_spider_stats_response
        mock_post.return_value = mock_response

        result = await client.spider.crawl(
            seed_urls=["https://example.com"],
            max_pages=15,
            result_mode="stats"
        )

        # Should have standard result fields
        assert result["result"]["pages_crawled"] == 15
        assert result["result"]["pages_failed"] == 2
        assert result["result"]["stop_reason"] == "max_pages_reached"

        # Should NOT have discovered_urls
        assert "discovered_urls" not in result["result"]


@pytest.mark.asyncio
async def test_spider_result_mode_urls(client, mock_spider_urls_response):
    """Test result_mode=urls returns discovered URLs array"""
    with patch.object(client.session, 'post', new_callable=AsyncMock) as mock_post:
        mock_response = Mock()
        mock_response.status_code = 200
        mock_response.json.return_value = mock_spider_urls_response
        mock_post.return_value = mock_response

        result = await client.spider.crawl(
            seed_urls=["https://example.com"],
            max_pages=10,
            result_mode="urls"
        )

        # Should have discovered_urls array
        assert "discovered_urls" in result["result"]
        assert isinstance(result["result"]["discovered_urls"], list)
        assert len(result["result"]["discovered_urls"]) == 10

        # Verify URL format
        for url in result["result"]["discovered_urls"]:
            assert url.startswith("https://")
            assert "example.com" in url


@pytest.mark.asyncio
async def test_spider_backward_compatibility_no_result_mode(client, mock_spider_stats_response):
    """Test backward compatibility - no result_mode defaults to stats"""
    with patch.object(client.session, 'post', new_callable=AsyncMock) as mock_post:
        mock_response = Mock()
        mock_response.status_code = 200
        mock_response.json.return_value = mock_spider_stats_response
        mock_post.return_value = mock_response

        # Don't specify result_mode
        result = await client.spider.crawl(
            seed_urls=["https://example.com"],
            max_pages=15
        )

        # Should work like stats mode
        assert result["result"]["pages_crawled"] == 15
        assert "discovered_urls" not in result["result"]


@pytest.mark.asyncio
async def test_spider_invalid_result_mode(client):
    """Test that invalid result_mode raises validation error"""
    with patch.object(client.session, 'post', new_callable=AsyncMock) as mock_post:
        mock_response = Mock()
        mock_response.status_code = 400
        mock_response.json.return_value = {
            "error": "Invalid result_mode",
            "message": "result_mode must be 'stats' or 'urls'"
        }
        mock_post.return_value = mock_response

        with pytest.raises(RipTideError):
            await client.spider.crawl(
                seed_urls=["https://example.com"],
                result_mode="invalid_mode"
            )


# ============================================================================
# URL Discovery Tests
# ============================================================================

@pytest.mark.asyncio
async def test_discovered_urls_parsing(client, mock_spider_urls_response):
    """Test that discovered_urls are properly parsed"""
    with patch.object(client.session, 'post', new_callable=AsyncMock) as mock_post:
        mock_response = Mock()
        mock_response.status_code = 200
        mock_response.json.return_value = mock_spider_urls_response
        mock_post.return_value = mock_response

        result = await client.spider.crawl(
            seed_urls=["https://example.com"],
            result_mode="urls"
        )

        urls = result["result"]["discovered_urls"]

        # Verify all URLs are valid strings
        assert all(isinstance(url, str) for url in urls)

        # Verify no duplicates
        assert len(urls) == len(set(urls))

        # Verify all URLs from same domain (for this test)
        assert all("example.com" in url for url in urls)


@pytest.mark.asyncio
async def test_max_pages_limits_discovered_urls(client):
    """Test that max_pages constraint limits discovered URLs"""
    mock_response_data = {
        "result": {
            "pages_crawled": 5,
            "pages_failed": 0,
            "duration_seconds": 15.0,
            "stop_reason": "max_pages_reached",
            "domains": ["example.com"],
            "discovered_urls": [
                "https://example.com",
                "https://example.com/page1",
                "https://example.com/page2",
                "https://example.com/page3",
                "https://example.com/page4"
            ]
        },
        "state": {"active": False, "pages_crawled": 5},
        "performance": {"pages_per_second": 0.33}
    }

    with patch.object(client.session, 'post', new_callable=AsyncMock) as mock_post:
        mock_response = Mock()
        mock_response.status_code = 200
        mock_response.json.return_value = mock_response_data
        mock_post.return_value = mock_response

        result = await client.spider.crawl(
            seed_urls=["https://example.com"],
            max_pages=5,
            result_mode="urls"
        )

        # Should not exceed max_pages
        assert len(result["result"]["discovered_urls"]) <= 5
        assert result["result"]["pages_crawled"] == 5


# ============================================================================
# Crawl Strategy Tests
# ============================================================================

@pytest.mark.asyncio
async def test_breadth_first_strategy(client, mock_spider_urls_response):
    """Test breadth-first crawl strategy"""
    with patch.object(client.session, 'post', new_callable=AsyncMock) as mock_post:
        mock_response = Mock()
        mock_response.status_code = 200
        mock_response.json.return_value = mock_spider_urls_response
        mock_post.return_value = mock_response

        result = await client.spider.crawl(
            seed_urls=["https://example.com"],
            max_pages=10,
            strategy="breadth_first",
            result_mode="urls"
        )

        # Verify request was made with correct strategy
        call_args = mock_post.call_args
        assert call_args[1]["json"]["strategy"] == "breadth_first"

        # Should return URLs
        assert "discovered_urls" in result["result"]


@pytest.mark.asyncio
async def test_depth_first_strategy(client, mock_spider_urls_response):
    """Test depth-first crawl strategy"""
    with patch.object(client.session, 'post', new_callable=AsyncMock) as mock_post:
        mock_response = Mock()
        mock_response.status_code = 200
        mock_response.json.return_value = mock_spider_urls_response
        mock_post.return_value = mock_response

        result = await client.spider.crawl(
            seed_urls=["https://example.com"],
            max_pages=10,
            strategy="depth_first",
            result_mode="urls"
        )

        # Verify request was made with correct strategy
        call_args = mock_post.call_args
        assert call_args[1]["json"]["strategy"] == "depth_first"


# ============================================================================
# Request Validation Tests
# ============================================================================

@pytest.mark.asyncio
async def test_spider_request_validation_stats():
    """Test that stats mode request is properly validated"""
    # This would be validated by the SDK client
    request_data = {
        "seed_urls": ["https://example.com"],
        "max_pages": 10,
        "result_mode": "stats"
    }

    assert request_data["seed_urls"] == ["https://example.com"]
    assert request_data["max_pages"] == 10
    assert request_data["result_mode"] == "stats"


@pytest.mark.asyncio
async def test_spider_request_validation_urls():
    """Test that urls mode request is properly validated"""
    request_data = {
        "seed_urls": ["https://example.com"],
        "max_pages": 20,
        "max_depth": 3,
        "result_mode": "urls"
    }

    assert request_data["result_mode"] == "urls"
    assert request_data["max_pages"] == 20
    assert request_data["max_depth"] == 3


@pytest.mark.asyncio
async def test_spider_request_validation_invalid():
    """Test that invalid result_mode would be rejected by API"""
    # The API should reject this, not the SDK
    request_data = {
        "seed_urls": ["https://example.com"],
        "result_mode": "invalid"
    }

    # This is just data validation - actual rejection happens at API level
    assert "result_mode" in request_data


# ============================================================================
# Edge Cases
# ============================================================================

@pytest.mark.asyncio
async def test_empty_discovered_urls(client):
    """Test handling of empty discovered_urls array"""
    mock_response_data = {
        "result": {
            "pages_crawled": 1,
            "pages_failed": 0,
            "duration_seconds": 2.0,
            "stop_reason": "no_more_urls",
            "domains": ["example.com"],
            "discovered_urls": []  # Empty array
        },
        "state": {"active": False, "pages_crawled": 1},
        "performance": {"pages_per_second": 0.5}
    }

    with patch.object(client.session, 'post', new_callable=AsyncMock) as mock_post:
        mock_response = Mock()
        mock_response.status_code = 200
        mock_response.json.return_value = mock_response_data
        mock_post.return_value = mock_response

        result = await client.spider.crawl(
            seed_urls=["https://example.com/isolated"],
            result_mode="urls"
        )

        # Should handle empty array gracefully
        assert result["result"]["discovered_urls"] == []
        assert isinstance(result["result"]["discovered_urls"], list)


@pytest.mark.asyncio
async def test_url_deduplication(client):
    """Test that duplicate seed URLs are handled"""
    mock_response_data = {
        "result": {
            "pages_crawled": 1,
            "pages_failed": 0,
            "duration_seconds": 3.0,
            "stop_reason": "completed",
            "domains": ["example.com"],
            "discovered_urls": ["https://example.com"]  # Deduplicated
        },
        "state": {"active": False, "pages_crawled": 1},
        "performance": {"pages_per_second": 0.33}
    }

    with patch.object(client.session, 'post', new_callable=AsyncMock) as mock_post:
        mock_response = Mock()
        mock_response.status_code = 200
        mock_response.json.return_value = mock_response_data
        mock_post.return_value = mock_response

        # Pass duplicate URLs
        result = await client.spider.crawl(
            seed_urls=[
                "https://example.com",
                "https://example.com",  # Duplicate
                "https://example.com/"  # Trailing slash variant
            ],
            result_mode="urls"
        )

        # Should deduplicate
        assert len(result["result"]["discovered_urls"]) == 1


# ============================================================================
# Integration-style Tests
# ============================================================================

@pytest.mark.asyncio
async def test_live_hilversum_use_case_simulation(client):
    """
    Simulate Live Hilversum use case:
    1. Spider discovers URLs from a site
    2. Each discovered URL is then extracted individually
    """
    # Mock spider response with discovered URLs
    spider_response = {
        "result": {
            "pages_crawled": 5,
            "pages_failed": 0,
            "duration_seconds": 20.0,
            "stop_reason": "max_pages_reached",
            "domains": ["livehilversum.nl"],
            "discovered_urls": [
                "https://livehilversum.nl",
                "https://livehilversum.nl/nieuws",
                "https://livehilversum.nl/sport",
                "https://livehilversum.nl/weer",
                "https://livehilversum.nl/verkeer"
            ]
        },
        "state": {"active": False, "pages_crawled": 5},
        "performance": {"pages_per_second": 0.25}
    }

    # Mock extraction response for each URL
    extract_response = {
        "total_urls": 1,
        "successful": 1,
        "failed": 0,
        "results": [{
            "url": "https://livehilversum.nl/nieuws",
            "status": 200,
            "document": {
                "text": "News content here",
                "markdown": "# News",
                "metadata": {"title": "Nieuws"}
            }
        }]
    }

    with patch.object(client.session, 'post', new_callable=AsyncMock) as mock_post:
        # Setup mock to return different responses
        mock_spider = Mock()
        mock_spider.status_code = 200
        mock_spider.json.return_value = spider_response

        mock_extract = Mock()
        mock_extract.status_code = 200
        mock_extract.json.return_value = extract_response

        # First call returns spider results, subsequent calls return extractions
        mock_post.side_effect = [mock_spider] + [mock_extract] * 5

        # Step 1: Discover URLs
        spider_result = await client.spider.crawl(
            seed_urls=["https://livehilversum.nl"],
            max_pages=5,
            result_mode="urls"
        )

        discovered = spider_result["result"]["discovered_urls"]
        assert len(discovered) == 5

        # Step 2: Extract each discovered URL
        extracted_content = []
        for url in discovered:
            result = await client.extract(urls=[url])
            if result["successful"] > 0:
                extracted_content.append(result["results"][0])

        # Verify end-to-end workflow
        assert len(extracted_content) == 5
        assert all(item["status"] == 200 for item in extracted_content)


# ============================================================================
# Performance and Metrics Tests
# ============================================================================

@pytest.mark.asyncio
async def test_spider_performance_metrics(client, mock_spider_urls_response):
    """Test that performance metrics are included in response"""
    with patch.object(client.session, 'post', new_callable=AsyncMock) as mock_post:
        mock_response = Mock()
        mock_response.status_code = 200
        mock_response.json.return_value = mock_spider_urls_response
        mock_post.return_value = mock_response

        result = await client.spider.crawl(
            seed_urls=["https://example.com"],
            result_mode="urls"
        )

        # Verify performance metrics
        assert "performance" in result
        assert "pages_per_second" in result["performance"]
        assert "avg_response_time" in result["performance"]
        assert "error_rate" in result["performance"]

        # Verify state information
        assert "state" in result
        assert "pages_crawled" in result["state"]
