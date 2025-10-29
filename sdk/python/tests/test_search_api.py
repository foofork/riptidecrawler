"""
Tests for Search API endpoint

Comprehensive test suite for the SearchAPI class covering:
- Basic search functionality
- Search with options
- Parameter validation
- Error handling
- Response parsing
"""

import pytest
from unittest.mock import AsyncMock, MagicMock
import httpx

from riptide_sdk.endpoints.search import SearchAPI
from riptide_sdk.models import SearchOptions, SearchResponse, SearchResultItem
from riptide_sdk.exceptions import ValidationError, APIError


@pytest.fixture
def mock_client():
    """Create a mock httpx.AsyncClient"""
    client = AsyncMock(spec=httpx.AsyncClient)
    return client


@pytest.fixture
def search_api(mock_client):
    """Create SearchAPI instance with mock client"""
    return SearchAPI(mock_client, "http://localhost:8080")


class TestSearchAPI:
    """Tests for SearchAPI class"""

    @pytest.mark.asyncio
    async def test_basic_search(self, search_api, mock_client):
        """Test basic search functionality"""
        # Mock response
        mock_response = MagicMock()
        mock_response.status_code = 200
        mock_response.json.return_value = {
            "query": "rust web scraping",
            "results": [
                {
                    "title": "Rust Web Scraping",
                    "url": "https://example.com/rust",
                    "snippet": "Learn web scraping with Rust",
                    "position": 1,
                },
                {
                    "title": "Rust Tutorial",
                    "url": "https://example.com/tutorial",
                    "snippet": "Complete Rust guide",
                    "position": 2,
                },
            ],
            "total_results": 2,
            "provider_used": "Serper",
            "search_time_ms": 150,
        }
        mock_client.get = AsyncMock(return_value=mock_response)

        # Perform search
        result = await search_api.search("rust web scraping")

        # Verify request
        mock_client.get.assert_called_once()
        call_args = mock_client.get.call_args
        assert call_args[0][0] == "http://localhost:8080/api/v1/search"
        assert call_args[1]["params"]["q"] == "rust web scraping"
        assert call_args[1]["params"]["limit"] == 10

        # Verify response
        assert isinstance(result, SearchResponse)
        assert result.query == "rust web scraping"
        assert result.total_results == 2
        assert result.provider_used == "Serper"
        assert result.search_time_ms == 150
        assert len(result.results) == 2
        assert result.results[0].title == "Rust Web Scraping"
        assert result.results[0].url == "https://example.com/rust"

    @pytest.mark.asyncio
    async def test_search_with_options(self, search_api, mock_client):
        """Test search with custom options"""
        mock_response = MagicMock()
        mock_response.status_code = 200
        mock_response.json.return_value = {
            "query": "python tutorial",
            "results": [],
            "total_results": 0,
            "provider_used": "Serper",
            "search_time_ms": 100,
        }
        mock_client.get = AsyncMock(return_value=mock_response)

        # Search with options
        options = SearchOptions(country="uk", language="en", provider="serper")
        result = await search_api.search(
            query="python tutorial",
            limit=20,
            options=options
        )

        # Verify request parameters
        call_args = mock_client.get.call_args
        params = call_args[1]["params"]
        assert params["q"] == "python tutorial"
        assert params["limit"] == 20
        assert params["country"] == "uk"
        assert params["language"] == "en"
        assert params["provider"] == "serper"

    @pytest.mark.asyncio
    async def test_empty_query_validation(self, search_api):
        """Test that empty query raises ValidationError"""
        with pytest.raises(ValidationError, match="Search query cannot be empty"):
            await search_api.search("")

        with pytest.raises(ValidationError, match="Search query cannot be empty"):
            await search_api.search("   ")

    @pytest.mark.asyncio
    async def test_limit_validation(self, search_api):
        """Test limit parameter validation"""
        with pytest.raises(ValidationError, match="Limit must be between 1 and 50"):
            await search_api.search("test", limit=0)

        with pytest.raises(ValidationError, match="Limit must be between 1 and 50"):
            await search_api.search("test", limit=51)

        with pytest.raises(ValidationError, match="Limit must be between 1 and 50"):
            await search_api.search("test", limit=-1)

    @pytest.mark.asyncio
    async def test_query_trimming(self, search_api, mock_client):
        """Test that queries are properly trimmed"""
        mock_response = MagicMock()
        mock_response.status_code = 200
        mock_response.json.return_value = {
            "query": "test",
            "results": [],
            "total_results": 0,
            "provider_used": "None",
            "search_time_ms": 0,
        }
        mock_client.get = AsyncMock(return_value=mock_response)

        await search_api.search("  test  ")

        # Verify query was trimmed
        call_args = mock_client.get.call_args
        assert call_args[1]["params"]["q"] == "test"

    @pytest.mark.asyncio
    async def test_api_error_503(self, search_api, mock_client):
        """Test handling of 503 Service Unavailable"""
        mock_response = MagicMock()
        mock_response.status_code = 503
        mock_response.text = '{"error": {"message": "Provider unavailable"}}'
        mock_response.json.return_value = {
            "error": {"message": "Provider unavailable"}
        }
        mock_client.get = AsyncMock(return_value=mock_response)

        with pytest.raises(APIError) as exc_info:
            await search_api.search("test")

        assert exc_info.value.status_code == 503
        assert "Provider unavailable" in str(exc_info.value.message)

    @pytest.mark.asyncio
    async def test_api_error_400(self, search_api, mock_client):
        """Test handling of 400 Bad Request"""
        mock_response = MagicMock()
        mock_response.status_code = 400
        mock_response.text = '{"error": {"message": "Invalid query"}}'
        mock_response.json.return_value = {
            "error": {"message": "Invalid query"}
        }
        mock_client.get = AsyncMock(return_value=mock_response)

        with pytest.raises(ValidationError, match="Invalid query"):
            await search_api.search("test")

    @pytest.mark.asyncio
    async def test_api_error_generic(self, search_api, mock_client):
        """Test handling of generic API errors"""
        mock_response = MagicMock()
        mock_response.status_code = 500
        mock_response.text = '{"error": {"message": "Internal server error"}}'
        mock_response.json.return_value = {
            "error": {"message": "Internal server error"}
        }
        mock_client.get = AsyncMock(return_value=mock_response)

        with pytest.raises(APIError) as exc_info:
            await search_api.search("test")

        assert exc_info.value.status_code == 500
        assert "Internal server error" in exc_info.value.message

    @pytest.mark.asyncio
    async def test_quick_search(self, search_api, mock_client):
        """Test quick_search convenience method"""
        mock_response = MagicMock()
        mock_response.status_code = 200
        mock_response.json.return_value = {
            "query": "golang frameworks",
            "results": [],
            "total_results": 0,
            "provider_used": "None",
            "search_time_ms": 50,
        }
        mock_client.get = AsyncMock(return_value=mock_response)

        result = await search_api.quick_search("golang frameworks", country="us", language="en")

        # Verify request
        call_args = mock_client.get.call_args
        params = call_args[1]["params"]
        assert params["q"] == "golang frameworks"
        assert params["limit"] == 10
        assert params["country"] == "us"
        assert params["language"] == "en"

        # Verify response
        assert result.query == "golang frameworks"


class TestSearchModels:
    """Tests for search-related models"""

    def test_search_options_to_dict(self):
        """Test SearchOptions.to_dict()"""
        options = SearchOptions(country="uk", language="en", provider="serper")
        data = options.to_dict()

        assert data["country"] == "uk"
        assert data["language"] == "en"
        assert data["provider"] == "serper"

    def test_search_options_defaults(self):
        """Test SearchOptions default values"""
        options = SearchOptions()

        assert options.country == "us"
        assert options.language == "en"
        assert options.provider is None

    def test_search_result_item_from_dict(self):
        """Test SearchResultItem.from_dict()"""
        data = {
            "title": "Test Title",
            "url": "https://example.com",
            "snippet": "Test snippet",
            "position": 1,
        }
        item = SearchResultItem.from_dict(data)

        assert item.title == "Test Title"
        assert item.url == "https://example.com"
        assert item.snippet == "Test snippet"
        assert item.position == 1

    def test_search_result_item_missing_fields(self):
        """Test SearchResultItem handles missing optional fields"""
        data = {
            "url": "https://example.com",
        }
        item = SearchResultItem.from_dict(data)

        assert item.url == "https://example.com"
        assert item.title == ""
        assert item.snippet == ""
        assert item.position == 0

    def test_search_response_from_dict(self):
        """Test SearchResponse.from_dict()"""
        data = {
            "query": "test query",
            "results": [
                {
                    "title": "Result 1",
                    "url": "https://example.com/1",
                    "snippet": "Snippet 1",
                    "position": 1,
                },
                {
                    "title": "Result 2",
                    "url": "https://example.com/2",
                    "snippet": "Snippet 2",
                    "position": 2,
                },
            ],
            "total_results": 2,
            "provider_used": "Serper",
            "search_time_ms": 200,
        }
        response = SearchResponse.from_dict(data)

        assert response.query == "test query"
        assert response.total_results == 2
        assert response.provider_used == "Serper"
        assert response.search_time_ms == 200
        assert len(response.results) == 2
        assert response.results[0].title == "Result 1"
        assert response.results[1].url == "https://example.com/2"


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
