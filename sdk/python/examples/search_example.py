"""
Search API Example

Demonstrates usage of the RipTide Search API endpoint with various options
and search providers.
"""

import asyncio
from riptide_sdk import RipTideClient, SearchOptions


async def basic_search():
    """Basic search example"""
    async with RipTideClient(base_url="http://localhost:8080") as client:
        # Simple search
        result = await client.search.search("rust web scraping")

        print(f"Query: {result.query}")
        print(f"Found {result.total_results} results")
        print(f"Provider: {result.provider_used}")
        print(f"Search time: {result.search_time_ms}ms\n")

        # Display results
        for item in result.results:
            print(f"{item.position}. {item.title}")
            print(f"   URL: {item.url}")
            print(f"   {item.snippet}\n")


async def search_with_options():
    """Search with custom options"""
    async with RipTideClient(base_url="http://localhost:8080") as client:
        # Search with custom country and language
        options = SearchOptions(
            country="uk",
            language="en",
        )

        result = await client.search.search(
            query="python machine learning",
            limit=20,
            options=options
        )

        print(f"Search results for UK region: {result.total_results} found")
        for item in result.results[:5]:  # Show first 5
            print(f"- {item.title}")


async def force_provider():
    """Force specific search provider"""
    async with RipTideClient(base_url="http://localhost:8080") as client:
        # Force Serper provider
        options = SearchOptions(provider="serper")

        result = await client.search.search(
            query="golang frameworks",
            limit=10,
            options=options
        )

        print(f"Using provider: {result.provider_used}")
        print(f"Results: {result.total_results}")


async def quick_search_example():
    """Using the quick_search convenience method"""
    async with RipTideClient(base_url="http://localhost:8080") as client:
        # Quick search with defaults
        result = await client.search.quick_search("artificial intelligence")

        print(f"Quick search found {result.total_results} results")
        urls = [item.url for item in result.results]
        print(f"URLs: {urls[:3]}")  # First 3 URLs


async def error_handling():
    """Demonstrate error handling"""
    from riptide_sdk import ValidationError, APIError

    async with RipTideClient(base_url="http://localhost:8080") as client:
        try:
            # Empty query - will raise ValidationError
            result = await client.search.search("")
        except ValidationError as e:
            print(f"Validation error: {e}")

        try:
            # Invalid limit - will raise ValidationError
            result = await client.search.search("test", limit=100)
        except ValidationError as e:
            print(f"Validation error: {e}")

        try:
            # This might fail if provider is not configured
            result = await client.search.search("test")
        except APIError as e:
            print(f"API error: {e.message} (status: {e.status_code})")


async def main():
    """Run all examples"""
    print("=== Basic Search ===")
    await basic_search()

    print("\n=== Search with Options ===")
    await search_with_options()

    print("\n=== Force Provider ===")
    await force_provider()

    print("\n=== Quick Search ===")
    await quick_search_example()

    print("\n=== Error Handling ===")
    await error_handling()


if __name__ == "__main__":
    asyncio.run(main())
