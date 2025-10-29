"""
Example: Using RipTideClientBuilder for fluent configuration

This example demonstrates the modern builder pattern for configuring
the RipTide Python SDK with chainable methods.
"""

import asyncio
from riptide_sdk import RipTideClientBuilder, CrawlOptions, CacheMode


async def basic_builder_example():
    """Basic usage with builder pattern"""

    # Create client using fluent builder
    client = (RipTideClientBuilder()
        .with_base_url("http://localhost:8080")
        .with_timeout(60.0)
        .with_max_connections(200)
        .build())

    async with client:
        # Crawl with output formatting
        result = await client.crawl.batch([
            "https://example.com",
            "https://example.org"
        ])

        # Use built-in formatters
        print(result.to_summary())
        print("\n" + "="*60 + "\n")
        print(result.to_markdown())


async def advanced_builder_example():
    """Advanced configuration with retry and custom headers"""

    client = (RipTideClientBuilder()
        .with_base_url("http://localhost:8080")
        .with_api_key("your-api-key-here")
        .with_timeout(90.0)
        .with_max_connections(300)
        .with_retry_config(max_retries=5, backoff_factor=2.0)
        .with_user_agent("MyApp/2.0 (Production)")
        .with_custom_header("X-Client-Version", "2.0.0")
        .build())

    async with client:
        # Use custom options
        options = CrawlOptions(
            cache_mode=CacheMode.READ_WRITE,
            concurrency=10,
            timeout_secs=30
        )

        result = await client.crawl.batch(
            ["https://example.com"],
            options=options
        )

        # Format as JSON
        print(result.to_json())


async def parallel_crawling_example():
    """High-throughput parallel crawling"""

    client = (RipTideClientBuilder()
        .with_base_url("http://localhost:8080")
        .with_timeout(120.0)
        .with_max_connections(500)
        .build())

    # Generate 100 URLs
    urls = [f"https://example.com/page/{i}" for i in range(100)]

    async with client:
        # Use parallel batch crawling
        results = await client.batch_crawl_parallel(
            urls,
            batch_size=10,
            max_concurrent=5
        )

        # Aggregate results
        total_successful = sum(r.successful for r in results if not isinstance(r, Exception))
        total_failed = sum(r.failed for r in results if not isinstance(r, Exception))

        print(f"\nParallel Crawl Complete:")
        print(f"  Successful: {total_successful}")
        print(f"  Failed: {total_failed}")


async def error_handling_example():
    """Demonstrate enhanced error handling with suggestions"""

    client = (RipTideClientBuilder()
        .with_base_url("http://localhost:8080")
        .build())

    try:
        async with client:
            # This will trigger validation error
            result = await client.crawl.batch([])
    except Exception as e:
        # Enhanced errors include suggestions and documentation links
        print(f"Error: {e}")
        if hasattr(e, 'suggestion'):
            print(f"Suggestion: {e.suggestion}")


if __name__ == "__main__":
    print("=" * 60)
    print("RipTide Python SDK - Builder Pattern Examples")
    print("=" * 60)

    # Run basic example
    print("\n1. Basic Builder Example:")
    asyncio.run(basic_builder_example())

    # Run advanced example
    print("\n2. Advanced Builder Example:")
    asyncio.run(advanced_builder_example())

    # Run parallel example
    print("\n3. Parallel Crawling Example:")
    asyncio.run(parallel_crawling_example())

    # Run error handling example
    print("\n4. Error Handling Example:")
    asyncio.run(error_handling_example())
