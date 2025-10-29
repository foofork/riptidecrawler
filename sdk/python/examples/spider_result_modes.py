#!/usr/bin/env python3
"""
Spider Result Modes Example

Demonstrates the use of result_mode parameter in spider crawl operations:
- STATS mode: Returns only statistics (default, lightweight)
- URLS mode: Returns statistics + discovered URLs (for discovery workflows)

Use Cases:
1. Stats mode: Quick crawl statistics and performance metrics
2. URLs mode: Discover URLs for subsequent extraction
3. Discover → Extract workflow: Two-phase content collection
"""

import asyncio
from riptide_sdk import RipTideClient, SpiderConfig, ResultMode


async def stats_mode_example():
    """
    Example 1: STATS mode (default)

    Returns only crawl statistics and performance metrics.
    Lightweight and efficient for monitoring crawl operations.
    """
    print("=" * 60)
    print("Example 1: STATS Mode (Default)")
    print("=" * 60)

    async with RipTideClient(base_url="http://localhost:8080") as client:
        # Crawl with default STATS mode
        result = await client.spider.crawl(
            seed_urls=["https://example.com"],
            config=SpiderConfig(
                max_depth=2,
                max_pages=50,
                strategy="breadth_first",
            ),
            result_mode=ResultMode.STATS,  # Default - can be omitted
        )

        # Print statistics
        print(f"\n✅ Crawl completed!")
        print(f"📊 Statistics:")
        print(f"  - Pages crawled: {result.pages_crawled}")
        print(f"  - Pages failed: {result.pages_failed}")
        print(f"  - Duration: {result.duration_seconds:.2f}s")
        print(f"  - Stop reason: {result.stop_reason}")
        print(f"  - Domains: {', '.join(result.domains)}")
        print(f"\n⚡ Performance:")
        print(f"  - Pages/second: {result.performance.pages_per_second:.2f}")
        print(f"  - Avg response time: {result.performance.avg_response_time_ms:.2f}ms")
        print(f"  - Error rate: {result.performance.error_rate:.2%}")

        # Note: discovered_urls is None in STATS mode
        print(f"\n📍 Discovered URLs: {result.discovered_urls}")  # None


async def urls_mode_example():
    """
    Example 2: URLS mode

    Returns statistics + list of all discovered URLs.
    Useful for URL discovery and subsequent processing.
    """
    print("\n" + "=" * 60)
    print("Example 2: URLS Mode (Discovery)")
    print("=" * 60)

    async with RipTideClient(base_url="http://localhost:8080") as client:
        # Crawl with URLS mode to get discovered URLs
        result = await client.spider.crawl(
            seed_urls=["https://example.com"],
            config=SpiderConfig(
                max_depth=2,
                max_pages=50,
                strategy="breadth_first",
            ),
            result_mode=ResultMode.URLS,  # Include discovered URLs
        )

        # Print statistics
        print(f"\n✅ Crawl completed!")
        print(f"📊 Statistics:")
        print(f"  - Pages crawled: {result.pages_crawled}")
        print(f"  - Duration: {result.duration_seconds:.2f}s")

        # Print discovered URLs
        if result.discovered_urls:
            print(f"\n🔍 Discovered {len(result.discovered_urls)} URLs:")
            for i, url in enumerate(result.discovered_urls[:10], 1):
                print(f"  {i}. {url}")

            if len(result.discovered_urls) > 10:
                print(f"  ... and {len(result.discovered_urls) - 10} more")
        else:
            print("\n⚠️ No URLs discovered")


async def discover_then_extract_workflow():
    """
    Example 3: Discover → Extract Workflow

    A powerful two-phase approach:
    1. Phase 1: Use spider with URLS mode to discover relevant URLs
    2. Phase 2: Extract full content from discovered URLs

    This is ideal for:
    - Blog content collection
    - Documentation scraping
    - Product catalog extraction
    - News article gathering
    """
    print("\n" + "=" * 60)
    print("Example 3: Discover → Extract Workflow")
    print("=" * 60)

    async with RipTideClient(base_url="http://localhost:8080") as client:
        # Phase 1: Discovery - Find all relevant URLs
        print("\n🔍 Phase 1: Discovering URLs...")
        discovery = await client.spider.crawl(
            seed_urls=["https://example.com/blog"],
            config=SpiderConfig(
                max_depth=2,
                max_pages=100,
                strategy="breadth_first",
                concurrency=5,
            ),
            result_mode=ResultMode.URLS,  # Get URLs for extraction
        )

        print(f"✅ Discovered {len(discovery.discovered_urls or [])} URLs")

        # Phase 2: Extraction - Get full content from discovered URLs
        if discovery.discovered_urls:
            print(f"\n📄 Phase 2: Extracting content from discovered URLs...")

            # Filter URLs (e.g., only blog posts)
            blog_urls = [
                url for url in discovery.discovered_urls
                if "/blog/" in url or "/post/" in url
            ]

            print(f"Found {len(blog_urls)} blog URLs")

            # Extract content from first 5 blog URLs
            for i, url in enumerate(blog_urls[:5], 1):
                try:
                    content = await client.extract.extract(url)
                    print(f"\n  {i}. {content.title or 'Untitled'}")
                    print(f"     URL: {url}")
                    print(f"     Word count: {content.metadata.word_count}")
                    print(f"     Quality: {content.quality_score:.2f}")

                    # Print first 150 characters of content
                    preview = content.content[:150].replace('\n', ' ')
                    print(f"     Preview: {preview}...")

                except Exception as e:
                    print(f"  {i}. ❌ Failed to extract {url}: {e}")


async def comparison_example():
    """
    Example 4: Comparing STATS vs URLS modes

    Shows the difference in response size and use cases.
    """
    print("\n" + "=" * 60)
    print("Example 4: STATS vs URLS Comparison")
    print("=" * 60)

    async with RipTideClient(base_url="http://localhost:8080") as client:
        config = SpiderConfig(
            max_depth=2,
            max_pages=50,
            strategy="breadth_first",
        )
        seed_urls = ["https://example.com"]

        # STATS mode
        print("\n📊 STATS mode (lightweight):")
        stats_result = await client.spider.crawl(
            seed_urls=seed_urls,
            config=config,
            result_mode=ResultMode.STATS,
        )
        print(f"  - Pages crawled: {stats_result.pages_crawled}")
        print(f"  - Response includes URLs: {stats_result.discovered_urls is not None}")
        print(f"  - Use case: Quick metrics, monitoring, health checks")

        # URLS mode
        print("\n🔗 URLS mode (comprehensive):")
        urls_result = await client.spider.crawl(
            seed_urls=seed_urls,
            config=config,
            result_mode=ResultMode.URLS,
        )
        print(f"  - Pages crawled: {urls_result.pages_crawled}")
        print(f"  - Response includes URLs: {urls_result.discovered_urls is not None}")
        print(f"  - Discovered URLs: {len(urls_result.discovered_urls or [])}")
        print(f"  - Use case: URL discovery, content pipeline, sitemap generation")


async def main():
    """Run all examples"""
    print("\n🕷️  RipTide Spider Result Modes Examples\n")

    try:
        # Example 1: STATS mode (default)
        await stats_mode_example()

        # Example 2: URLS mode
        await urls_mode_example()

        # Example 3: Discover → Extract workflow
        await discover_then_extract_workflow()

        # Example 4: Comparison
        await comparison_example()

        print("\n" + "=" * 60)
        print("✅ All examples completed successfully!")
        print("=" * 60)

    except Exception as e:
        print(f"\n❌ Error: {e}")
        import traceback
        traceback.print_exc()


if __name__ == "__main__":
    asyncio.run(main())
