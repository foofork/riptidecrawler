"""
Spider Crawling API Example

Demonstrates how to use the Spider API for deep web crawling with the RipTide SDK.
"""

import asyncio
from riptide_sdk import RipTideClient, SpiderConfig


async def basic_spider_crawl():
    """Basic spider crawl example"""
    async with RipTideClient(base_url="http://localhost:8080") as client:
        # Simple crawl with defaults
        result = await client.spider.crawl(
            seed_urls=["https://example.com"],
        )

        print("=" * 60)
        print("BASIC SPIDER CRAWL")
        print("=" * 60)
        print(result.to_summary())
        print()


async def advanced_spider_crawl():
    """Advanced spider crawl with custom configuration"""
    async with RipTideClient(base_url="http://localhost:8080") as client:
        # Configure crawl parameters
        config = SpiderConfig(
            max_depth=3,              # Crawl up to 3 levels deep
            max_pages=100,            # Limit to 100 pages
            strategy="breadth_first", # Use BFS strategy
            concurrency=10,           # 10 concurrent requests
            delay_ms=500,             # 500ms delay between requests
            respect_robots=True,      # Respect robots.txt
            follow_redirects=True,    # Follow HTTP redirects
        )

        result = await client.spider.crawl(
            seed_urls=[
                "https://example.com",
                "https://example.org",
            ],
            config=config,
        )

        print("=" * 60)
        print("ADVANCED SPIDER CRAWL")
        print("=" * 60)
        print(result.to_summary())
        print()


async def spider_status_monitoring():
    """Monitor spider status during crawl"""
    async with RipTideClient(base_url="http://localhost:8080") as client:
        # Get current spider status
        status = await client.spider.status(include_metrics=True)

        print("=" * 60)
        print("SPIDER STATUS")
        print("=" * 60)
        print(status.to_summary())
        print()


async def spider_control_operations():
    """Control spider operations"""
    async with RipTideClient(base_url="http://localhost:8080") as client:
        # Stop current crawl
        response = await client.spider.control(action="stop")
        print(f"Stop response: {response}")

        # Reset spider state
        response = await client.spider.control(action="reset")
        print(f"Reset response: {response}")


async def crawl_with_progress_callback():
    """Crawl with progress monitoring"""
    async with RipTideClient(base_url="http://localhost:8080") as client:
        def on_progress(status):
            """Callback for progress updates"""
            print(f"Progress: {status.state.pages_crawled} pages crawled, "
                  f"{status.state.frontier_size} in frontier")

        config = SpiderConfig(
            max_pages=50,
            strategy="breadth_first",
        )

        result = await client.spider.crawl_with_status_polling(
            seed_urls=["https://example.com"],
            config=config,
            poll_interval=2.0,  # Check status every 2 seconds
            callback=on_progress,
        )

        print("=" * 60)
        print("CRAWL WITH PROGRESS MONITORING")
        print("=" * 60)
        print(result.to_summary())


async def multi_domain_crawl():
    """Crawl multiple domains simultaneously"""
    async with RipTideClient(base_url="http://localhost:8080") as client:
        config = SpiderConfig(
            max_depth=2,
            max_pages=200,
            strategy="best_first",  # Use best-first search
            concurrency=15,
        )

        result = await client.spider.crawl(
            seed_urls=[
                "https://news.ycombinator.com",
                "https://reddit.com/r/programming",
                "https://github.com/trending",
            ],
            config=config,
        )

        print("=" * 60)
        print("MULTI-DOMAIN CRAWL")
        print("=" * 60)
        print(f"Domains discovered: {', '.join(result.domains)}")
        print(f"Total pages: {result.pages_crawled}")
        print(f"Success rate: {(result.pages_crawled / (result.pages_crawled + result.pages_failed) * 100):.1f}%")
        print()


async def error_handling_example():
    """Demonstrate error handling"""
    async with RipTideClient(base_url="http://localhost:8080") as client:
        try:
            # This will fail if spider is not enabled on server
            result = await client.spider.crawl(
                seed_urls=["https://example.com"],
            )
            print("Crawl succeeded!")
        except Exception as e:
            print(f"Error type: {type(e).__name__}")
            print(f"Error message: {str(e)}")

            # Check specific error types
            from riptide_sdk import ConfigError, ValidationError, APIError

            if isinstance(e, ConfigError):
                print("Spider is not enabled on the server")
            elif isinstance(e, ValidationError):
                print("Invalid request parameters")
            elif isinstance(e, APIError):
                print(f"API error with status code: {e.status_code}")


async def main():
    """Run all examples"""
    print("\n" + "=" * 60)
    print("RIPTIDE SPIDER API EXAMPLES")
    print("=" * 60 + "\n")

    try:
        # Run examples
        await basic_spider_crawl()
        await advanced_spider_crawl()
        await spider_status_monitoring()
        await multi_domain_crawl()

        # Uncomment to test control operations
        # await spider_control_operations()

        # Uncomment to test progress monitoring
        # await crawl_with_progress_callback()

        # Demonstrate error handling
        print("\n" + "=" * 60)
        print("ERROR HANDLING EXAMPLE")
        print("=" * 60)
        await error_handling_example()

    except Exception as e:
        print(f"\nUnexpected error: {e}")
        import traceback
        traceback.print_exc()


if __name__ == "__main__":
    asyncio.run(main())
