"""
Search API endpoint implementation

Provides search functionality using the RipTide search infrastructure with
support for multiple providers (Serper, SearXNG, None).
"""

from typing import Optional
import httpx

from ..models import SearchResponse, SearchOptions
from ..exceptions import APIError, ValidationError


class SearchAPI:
    """API for search operations"""

    def __init__(self, client: httpx.AsyncClient, base_url: str):
        """
        Initialize SearchAPI

        Args:
            client: Async HTTP client
            base_url: Base URL for the API
        """
        self.client = client
        self.base_url = base_url

    async def search(
        self,
        query: str,
        limit: int = 10,
        options: Optional[SearchOptions] = None,
    ) -> SearchResponse:
        """
        Perform a search query using configured search providers

        This endpoint provides search functionality with automatic fallback
        capabilities across multiple providers (Serper, SearXNG, None).

        Args:
            query: Search query string
            limit: Number of results to return (1-50, default: 10)
            options: Optional search configuration

        Returns:
            SearchResponse with results and metadata

        Raises:
            ValidationError: If query is empty or limit is invalid
            APIError: If the API returns an error

        Example:
            Basic search:
            >>> result = await client.search.search("rust web scraping")
            >>> print(f"Found {result.total_results} results")
            >>> for item in result.results:
            ...     print(f"{item.title}: {item.url}")

            Search with options:
            >>> result = await client.search.search(
            ...     "python tutorial",
            ...     limit=20,
            ...     options=SearchOptions(country="uk", language="en")
            ... )
            >>> print(f"Provider: {result.provider_used}")
            >>> print(f"Search time: {result.search_time_ms}ms")

            Force specific provider:
            >>> result = await client.search.search(
            ...     "machine learning",
            ...     options=SearchOptions(provider="serper")
            ... )
        """
        # Validate query
        if not query or not query.strip():
            raise ValidationError("Search query cannot be empty")

        # Validate and clamp limit
        if limit < 1 or limit > 50:
            raise ValidationError("Limit must be between 1 and 50")

        # Build query parameters
        params = {
            "q": query.strip(),
            "limit": limit,
        }

        # Add optional parameters
        if options:
            if options.country:
                params["country"] = options.country
            if options.language:
                params["language"] = options.language
            if options.provider:
                params["provider"] = options.provider

        # Make request
        response = await self.client.get(
            f"{self.base_url}/api/v1/search",
            params=params,
        )

        # Handle errors
        if response.status_code != 200:
            error_data = response.json() if response.text else {}
            error_msg = error_data.get("error", {}).get("message", "Search failed")

            # Provide helpful error context
            if response.status_code == 503:
                raise APIError(
                    message=f"Search provider unavailable: {error_msg}",
                    status_code=response.status_code,
                    response_data=error_data,
                )
            elif response.status_code == 400:
                raise ValidationError(error_msg)
            else:
                raise APIError(
                    message=error_msg,
                    status_code=response.status_code,
                    response_data=error_data,
                )

        return SearchResponse.from_dict(response.json())

    async def quick_search(
        self,
        query: str,
        country: str = "us",
        language: str = "en",
    ) -> SearchResponse:
        """
        Convenience method for quick searches with common parameters

        Args:
            query: Search query string
            country: Country code (default: "us")
            language: Language code (default: "en")

        Returns:
            SearchResponse with results

        Example:
            >>> result = await client.search.quick_search("golang frameworks")
            >>> urls = [r.url for r in result.results]
        """
        options = SearchOptions(country=country, language=language)
        return await self.search(query, limit=10, options=options)
