#!/usr/bin/env python3
"""
RipTide Python SDK - Phase 2 Usage Examples

This file demonstrates all Phase 2 features:
- Multiple result modes (stats, urls, pages, stream, store)
- Field selection (include/exclude)
- Job storage and pagination
- Batch extraction
- Discover → Extract workflows
"""

from riptide_client import RipTide


def example_1_lightweight_stats():
    """Example 1: Lightweight statistics only (minimal bandwidth)."""
    print("\n=== Example 1: Lightweight Stats ===")

    client = RipTide('http://localhost:8080')

    result = client.spider(
        seeds=['https://example.com'],
        result_mode='stats',
        max_pages=100,
        max_depth=3
    )

    print(f"Pages crawled: {result['pages_crawled']}")
    print(f"Pages failed: {result['pages_failed']}")
    print(f"Duration: {result['duration_seconds']}s")
    print(f"Stop reason: {result['stop_reason']}")


def example_2_discover_urls():
    """Example 2: Discover URLs for later processing."""
    print("\n=== Example 2: Discover URLs ===")

    client = RipTide('http://localhost:8080')

    result = client.spider(
        seeds=['https://example.com/docs'],
        result_mode='urls',
        max_pages=500,
        max_depth=3
    )

    print(f"Discovered {len(result['discovered_urls'])} URLs")
    print(f"Domains: {', '.join(result['domains'])}")

    # Sample of discovered URLs
    for url in result['discovered_urls'][:10]:
        print(f"  - {url}")


def example_3_full_pages_with_field_selection():
    """Example 3: Get full page data with selective fields."""
    print("\n=== Example 3: Full Pages with Field Selection ===")

    client = RipTide('http://localhost:8080')

    result = client.spider(
        seeds=['https://example.com/blog'],
        result_mode='pages',
        max_pages=50,
        include='title,markdown,links',  # Only these fields
        exclude='content'                 # Exclude heavy HTML
    )

    print(f"Retrieved {len(result['pages'])} pages")

    for page in result['pages'][:5]:
        print(f"\nPage: {page['url']}")
        print(f"  Title: {page.get('title', 'N/A')}")
        print(f"  Markdown: {len(page.get('markdown', ''))} chars")
        print(f"  Links: {len(page.get('links', []))} links")
        print(f"  Depth: {page.get('depth', 0)}")


def example_4_streaming_realtime():
    """Example 4: Stream results in real-time."""
    print("\n=== Example 4: Real-time Streaming ===")

    client = RipTide('http://localhost:8080')

    page_count = 0
    for event in client.spider_stream(
        seeds=['https://example.com'],
        max_pages=100,
        include='title,links'
    ):
        if event['type'] == 'page':
            page = event['data']
            page_count += 1
            print(f"[{page_count}] Crawled: {page['url']}")
            print(f"    Title: {page.get('title', 'N/A')}")
            print(f"    Links: {len(page.get('links', []))}")

        elif event['type'] == 'stats':
            stats = event['data']
            print(f"\n✅ Crawl complete!")
            print(f"   Total: {stats['pages_crawled']} pages")
            print(f"   Duration: {stats['duration_seconds']}s")


def example_5_async_job_storage():
    """Example 5: Large crawl with async storage and pagination."""
    print("\n=== Example 5: Async Job Storage ===")

    client = RipTide('http://localhost:8080')

    # Start large crawl job
    print("Starting large crawl job...")
    job_id = client.spider_store(
        seeds=['https://example.com'],
        max_pages=10000,
        max_depth=5,
        include='title,markdown'
    )

    print(f"Job started: {job_id}")

    # Poll for completion (in production, use webhooks)
    import time
    while True:
        stats = client.get_stats(job_id)
        print(f"Progress: {stats['pages_crawled']} pages crawled...")

        if stats['stop_reason'] != 'running':
            print(f"Job completed: {stats['stop_reason']}")
            break

        time.sleep(5)

    # Fetch results in pages
    print("\nFetching results...")
    cursor = None
    total_fetched = 0

    while True:
        batch = client.get_results(
            job_id,
            cursor=cursor,
            limit=200,
            include='title,markdown'
        )

        total_fetched += len(batch['pages'])
        print(f"Fetched {total_fetched} results so far...")

        # Process batch
        for page in batch['pages']:
            # Save to database, process, etc.
            pass

        if batch['done']:
            print("All results fetched!")
            break

        cursor = batch['next_cursor']


def example_6_discover_extract_workflow():
    """Example 6: Discover URLs, then extract content."""
    print("\n=== Example 6: Discover → Extract Workflow ===")

    client = RipTide('http://localhost:8080')

    # Phase 1: Discover all URLs
    print("Phase 1: Discovering URLs...")
    discover_result = client.spider(
        seeds=['https://example.com/docs'],
        result_mode='urls',
        max_pages=500
    )

    discovered = discover_result['discovered_urls']
    print(f"Discovered {len(discovered)} URLs")

    # Filter for documentation pages only
    doc_urls = [url for url in discovered if '/docs/' in url]
    print(f"Filtered to {len(doc_urls)} documentation pages")

    # Phase 2: Extract content from documentation pages
    print("\nPhase 2: Extracting content...")
    results = client.extract_batch(
        urls=doc_urls[:100],  # First 100 pages
        format='markdown'
    )

    success_count = 0
    error_count = 0

    for result in results:
        if result.get('markdown'):
            success_count += 1
            print(f"✅ {result['url']}: {len(result['markdown'])} chars")
        elif result.get('error'):
            error_count += 1
            print(f"❌ {result['url']}: {result['error']}")

    print(f"\nExtraction complete: {success_count} success, {error_count} errors")


def example_7_single_extraction():
    """Example 7: Extract single URL."""
    print("\n=== Example 7: Single URL Extraction ===")

    client = RipTide('http://localhost:8080')

    result = client.extract(
        url='https://example.com/article',
        format='markdown'
    )

    print(f"URL: {result['url']}")
    print(f"Markdown: {len(result.get('markdown', ''))} chars")
    print(f"Metadata: {result.get('metadata', {})}")


def example_8_batch_extraction():
    """Example 8: Extract multiple URLs in batch."""
    print("\n=== Example 8: Batch Extraction ===")

    client = RipTide('http://localhost:8080')

    urls = [
        'https://example.com/page1',
        'https://example.com/page2',
        'https://example.com/page3'
    ]

    results = client.extract_batch(
        urls=urls,
        format='markdown'
    )

    for result in results:
        print(f"\nURL: {result['url']}")
        if result.get('markdown'):
            print(f"  ✅ Success: {len(result['markdown'])} chars")
        elif result.get('error'):
            print(f"  ❌ Error: {result['error']}")


def example_9_custom_scope():
    """Example 9: Spider with custom scope restrictions."""
    print("\n=== Example 9: Custom Scope ===")

    client = RipTide('http://localhost:8080')

    result = client.spider(
        seeds=['https://example.com'],
        result_mode='pages',
        max_pages=100,
        scope={
            'allowed_domains': ['example.com', 'docs.example.com'],
            'exclude_patterns': ['/admin/', '/private/'],
            'include_patterns': ['/docs/', '/blog/']
        }
    )

    print(f"Crawled {len(result['pages'])} pages within scope")


def example_10_content_size_limits():
    """Example 10: Control content size with limits."""
    print("\n=== Example 10: Content Size Limits ===")

    client = RipTide('http://localhost:8080')

    result = client.spider(
        seeds=['https://example.com'],
        result_mode='pages',
        max_pages=50,
        max_content_bytes=1048576,  # 1MB per page
        include='title,content'
    )

    for page in result['pages']:
        truncated = page.get('truncated', False)
        size = len(page.get('content', ''))
        print(f"{page['url']}: {size} bytes {'(truncated)' if truncated else ''}")


def example_11_context_manager():
    """Example 11: Use context manager for automatic cleanup."""
    print("\n=== Example 11: Context Manager ===")

    with RipTide('http://localhost:8080') as client:
        result = client.spider(
            seeds=['https://example.com'],
            result_mode='stats'
        )
        print(f"Crawled {result['pages_crawled']} pages")

    # Session automatically closed


def example_12_error_handling():
    """Example 12: Proper error handling."""
    print("\n=== Example 12: Error Handling ===")

    from riptide_client import APIError, RateLimitError, TimeoutError

    client = RipTide('http://localhost:8080', timeout=60)

    try:
        result = client.spider(
            seeds=['https://example.com'],
            result_mode='pages',
            max_pages=1000
        )
        print(f"Success: {len(result['pages'])} pages")

    except RateLimitError:
        print("❌ Rate limit exceeded - wait before retrying")
    except TimeoutError:
        print("❌ Request timed out - increase timeout or reduce scope")
    except APIError as e:
        print(f"❌ API error: {e}")


if __name__ == '__main__':
    """
    Run all examples (requires running RipTide API server).

    To use in your code:
        from riptide_client import RipTide
        client = RipTide('http://localhost:8080')
        result = client.spider(seeds=['https://example.com'], result_mode='urls')
    """

    print("=" * 60)
    print("RipTide Python SDK - Phase 2 Usage Examples")
    print("=" * 60)

    # Run examples (comment out as needed)
    example_1_lightweight_stats()
    example_2_discover_urls()
    example_3_full_pages_with_field_selection()
    example_4_streaming_realtime()
    # example_5_async_job_storage()  # Uncomment for async job demo
    example_6_discover_extract_workflow()
    example_7_single_extraction()
    example_8_batch_extraction()
    example_9_custom_scope()
    example_10_content_size_limits()
    example_11_context_manager()
    example_12_error_handling()

    print("\n" + "=" * 60)
    print("Examples complete!")
    print("=" * 60)
