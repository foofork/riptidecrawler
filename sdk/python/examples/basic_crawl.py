"""
Basic crawl example

Demonstrates basic batch crawling functionality with the RipTide SDK.
"""

import asyncio
from riptide_sdk import RipTideClient, CrawlOptions, CacheMode


async def main():
    """Run basic crawl examples"""

    # Create client
    async with RipTideClient(base_url="http://localhost:8080") as client:

        # Example 1: Simple batch crawl
        print("Example 1: Simple batch crawl")
        print("-" * 50)

        urls = [
            "https://example.com",
            "https://example.org",
            "https://example.net",
        ]

        result = await client.crawl.batch(urls)

        print(f"Total URLs: {result.total_urls}")
        print(f"Successful: {result.successful}")
        print(f"Failed: {result.failed}")
        print(f"From cache: {result.from_cache}")
        print(f"Cache hit rate: {result.statistics.cache_hit_rate:.2%}")
        print()

        # Print individual results
        for item in result.results:
            status_icon = "✓" if item.document else "✗"
            cache_icon = "📦" if item.from_cache else "🌐"
            print(f"  {status_icon} {cache_icon} {item.url}")
            print(f"     Status: {item.status} | Quality: {item.quality_score:.2f}")
            print(f"     Engine: {item.gate_decision} | Time: {item.processing_time_ms}ms")
            if item.error:
                print(f"     Error: {item.error.message}")
            print()

        # Example 2: Crawl with custom options
        print("\nExample 2: Crawl with custom options")
        print("-" * 50)

        options = CrawlOptions(
            cache_mode=CacheMode.READ_WRITE,
            concurrency=5,
            use_spider=False,
        )

        result = await client.crawl.batch(urls[:2], options=options)
        print(f"Crawled {result.successful} URLs with custom options")
        print()

        # Example 3: Single URL crawl
        print("\nExample 3: Single URL crawl")
        print("-" * 50)

        result = await client.crawl.single("https://example.com")
        item = result.results[0]

        print(f"URL: {item.url}")
        print(f"Status: {item.status}")
        print(f"Quality score: {item.quality_score}")
        print(f"Processing time: {item.processing_time_ms}ms")

        if item.document:
            print(f"Document extracted: {len(item.document.html or '')} bytes HTML")
            if item.document.text:
                print(f"Text preview: {item.document.text[:100]}...")


if __name__ == "__main__":
    asyncio.run(main())
