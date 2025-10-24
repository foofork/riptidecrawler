"""
Streaming example

Demonstrates real-time streaming capabilities for crawl and search operations.
"""

import asyncio
from riptide_sdk import RipTideClient, CrawlOptions


async def main():
    """Run streaming examples"""

    async with RipTideClient(base_url="http://localhost:8080") as client:

        # Example 1: NDJSON crawl streaming
        print("Example 1: NDJSON crawl streaming")
        print("-" * 50)

        urls = [
            "https://example.com",
            "https://example.org",
            "https://example.net",
            "https://example.edu",
            "https://example.info",
        ]

        print("Starting streaming crawl...")
        count = 0

        async for result in client.streaming.crawl_ndjson(urls):
            count += 1
            data = result.data
            url = data.get("url", "unknown")
            status = data.get("status", 0)

            print(f"  [{count}/5] {url} - Status: {status}")

            if data.get("from_cache"):
                print("       📦 From cache")

            if data.get("error"):
                error = data["error"]
                print(f"       ❌ Error: {error.get('message')}")

        print(f"Completed streaming {count} results")
        print()

        # Example 2: NDJSON deep search streaming
        print("\nExample 2: Deep search streaming")
        print("-" * 50)

        print("Searching for 'python programming'...")
        count = 0

        async for result in client.streaming.deepsearch_ndjson(
            query="python programming",
            limit=5
        ):
            count += 1
            data = result.data

            print(f"  [{count}] {data.get('title', 'No title')}")
            print(f"      URL: {data.get('url')}")
            if data.get('snippet'):
                print(f"      {data['snippet'][:100]}...")
            print()

        print(f"Found {count} search results")
        print()

        # Example 3: SSE streaming
        print("\nExample 3: SSE (Server-Sent Events) streaming")
        print("-" * 50)

        urls_small = urls[:3]
        print(f"Streaming {len(urls_small)} URLs via SSE...")

        async for event in client.streaming.crawl_sse(urls_small):
            print(f"Event type: {event.event_type}")

            if event.event_type == "result":
                data = event.data
                print(f"  URL: {data.get('url')}")
                print(f"  Status: {data.get('status')}")
            elif event.event_type == "progress":
                data = event.data
                print(f"  Progress: {data.get('completed')}/{data.get('total')}")
            elif event.event_type == "complete":
                print("  Stream completed")

            print()

        # Example 4: Streaming with error handling
        print("\nExample 4: Streaming with error handling")
        print("-" * 50)

        try:
            count = 0
            async for result in client.streaming.crawl_ndjson(
                ["https://invalid-url-example-123456789.com"],
                options=CrawlOptions(concurrency=1)
            ):
                count += 1
                data = result.data

                if data.get("error"):
                    error = data["error"]
                    print(f"Handled error: {error.get('message')}")
                    print(f"Retryable: {error.get('retryable', False)}")
                else:
                    print(f"Success: {data.get('url')}")

            print(f"Processed {count} streaming results with error handling")

        except Exception as e:
            print(f"Streaming error: {e}")

        print()

        # Example 5: Concurrent streaming
        print("\nExample 5: Concurrent streaming (advanced)")
        print("-" * 50)

        async def process_stream(stream_id: int, urls_subset):
            """Process a single stream"""
            count = 0
            async for result in client.streaming.crawl_ndjson(urls_subset):
                count += 1
                url = result.data.get("url", "unknown")
                print(f"  Stream {stream_id}: {url}")
            return count

        # Split URLs into batches
        batch_size = 2
        batches = [urls[i:i + batch_size] for i in range(0, len(urls), batch_size)]

        # Run streams concurrently
        tasks = [
            process_stream(i + 1, batch)
            for i, batch in enumerate(batches)
        ]

        results = await asyncio.gather(*tasks)
        total = sum(results)

        print(f"\nProcessed {total} total results across {len(batches)} concurrent streams")


if __name__ == "__main__":
    asyncio.run(main())
