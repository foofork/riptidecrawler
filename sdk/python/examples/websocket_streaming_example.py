#!/usr/bin/env python3
"""
WebSocket Streaming Example for RipTide SDK

Demonstrates real-time bidirectional WebSocket streaming for web crawling with:
- Basic WebSocket crawling
- Message callbacks for real-time processing
- Error handling and recovery
- Connection health monitoring
- Performance comparison with NDJSON/SSE
- Graceful shutdown

Prerequisites:
    pip install websockets

Usage:
    python websocket_streaming_example.py
"""

import asyncio
import time
from typing import List
from riptide_sdk import AsyncRipTideClient
from riptide_sdk.models import CrawlOptions, CacheMode, StreamingResult


# =============================================================================
# Example 1: Basic WebSocket Streaming
# =============================================================================

async def basic_websocket_streaming():
    """Basic WebSocket streaming with automatic connection management."""
    print("\n=== Example 1: Basic WebSocket Streaming ===\n")

    async with AsyncRipTideClient(base_url="http://localhost:3000") as client:
        urls = [
            "https://example.com",
            "https://httpbin.org/html",
            "https://www.wikipedia.org",
        ]

        print(f"Streaming {len(urls)} URLs via WebSocket...")
        start_time = time.time()

        result_count = 0
        async for result in client.streaming.crawl_websocket(urls):
            if result.event_type == "welcome":
                print(f"✓ Connected to WebSocket")
                print(f"  Session ID: {result.data.get('session_id')}")
                print(f"  Protocol: {result.data.get('protocol_version')}")

            elif result.event_type == "metadata":
                print(f"\n✓ Crawl started")
                print(f"  Total URLs: {result.data.get('total_urls')}")
                print(f"  Stream Type: {result.data.get('stream_type')}")

            elif result.event_type == "result":
                result_count += 1
                crawl_result = result.data["result"]
                progress = result.data.get("progress", {})

                print(f"\n[{result_count}] {crawl_result['url']}")
                print(f"  Status: {crawl_result['status']}")
                print(f"  Gate: {crawl_result['gate_decision']}")
                print(f"  Quality: {crawl_result['quality_score']:.2f}")
                print(
                    f"  Progress: {progress.get('completed', 0)}/{progress.get('total', 0)} "
                    f"({progress.get('success_rate', 0):.1%} success)"
                )

            elif result.event_type == "summary":
                print(f"\n✓ Crawl completed")
                print(f"  Total URLs: {result.data.get('total_urls')}")
                print(f"  Successful: {result.data.get('successful')}")
                print(f"  Failed: {result.data.get('failed')}")
                print(
                    f"  Processing time: {result.data.get('total_processing_time_ms')}ms"
                )

            elif result.event_type == "error":
                print(f"✗ Error: {result.data.get('message')}")

        elapsed = time.time() - start_time
        print(f"\nTotal time: {elapsed:.2f}s")


# =============================================================================
# Example 2: WebSocket with Callbacks
# =============================================================================

async def websocket_with_callbacks():
    """WebSocket streaming with real-time message callbacks."""
    print("\n=== Example 2: WebSocket with Callbacks ===\n")

    results_received = []

    async def handle_message(result: StreamingResult):
        """Process each message as it arrives."""
        if result.event_type == "result":
            crawl_result = result.data["result"]
            results_received.append(crawl_result)

            # Real-time processing
            if crawl_result["quality_score"] > 0.8:
                print(f"✓ High quality result: {crawl_result['url']}")
            elif crawl_result.get("error"):
                print(f"✗ Failed: {crawl_result['url']}")

    async with AsyncRipTideClient(base_url="http://localhost:3000") as client:
        urls = [
            "https://example.com",
            "https://httpbin.org/status/200",
            "https://httpbin.org/delay/2",
        ]

        print(f"Streaming {len(urls)} URLs with real-time callbacks...\n")

        # Stream with callback for immediate processing
        async for result in client.streaming.crawl_websocket(
            urls, on_message=handle_message
        ):
            if result.event_type == "summary":
                print(f"\n✓ Processing completed")
                print(f"  Results processed: {len(results_received)}")


# =============================================================================
# Example 3: Error Handling and Reconnection
# =============================================================================

async def websocket_error_handling():
    """Demonstrate WebSocket error handling and recovery."""
    print("\n=== Example 3: Error Handling and Reconnection ===\n")

    async with AsyncRipTideClient(base_url="http://localhost:3000") as client:
        urls = ["https://example.com", "https://invalid-url-that-fails"]

        max_retries = 3
        retry_count = 0

        while retry_count < max_retries:
            try:
                print(f"Attempt {retry_count + 1}/{max_retries}...")

                async for result in client.streaming.crawl_websocket(urls):
                    if result.event_type == "result":
                        crawl_result = result.data["result"]
                        if crawl_result.get("error"):
                            print(
                                f"✗ URL failed: {crawl_result['url']} - "
                                f"{crawl_result['error'].get('message')}"
                            )
                        else:
                            print(f"✓ URL succeeded: {crawl_result['url']}")

                    elif result.event_type == "summary":
                        print(f"\n✓ Batch completed successfully")
                        break

                # Success - exit retry loop
                break

            except Exception as e:
                retry_count += 1
                print(f"✗ WebSocket error: {e}")

                if retry_count < max_retries:
                    wait_time = 2**retry_count  # Exponential backoff
                    print(f"Retrying in {wait_time}s...")
                    await asyncio.sleep(wait_time)
                else:
                    print("✗ Max retries exceeded")


# =============================================================================
# Example 4: WebSocket Health Monitoring
# =============================================================================

async def websocket_health_monitoring():
    """Monitor WebSocket connection health and latency."""
    print("\n=== Example 4: WebSocket Health Monitoring ===\n")

    async with AsyncRipTideClient(base_url="http://localhost:3000") as client:
        # Test WebSocket ping
        print("Testing WebSocket connection...\n")

        try:
            ping_result = await client.streaming.ping_websocket()

            print("✓ WebSocket connection healthy")
            print(f"  Latency: {ping_result['latency_ms']:.2f}ms")
            print(f"  Session ID: {ping_result['session_id']}")
            print(f"  Server time: {ping_result['server_time']}")

        except Exception as e:
            print(f"✗ WebSocket ping failed: {e}")

        # Get connection status
        print("\nGetting connection status...\n")

        try:
            status = await client.streaming.get_websocket_status()

            print("✓ WebSocket status:")
            print(f"  Session ID: {status.get('session_id')}")
            print(f"  Healthy: {status.get('is_healthy')}")
            print(f"  Messages: {status.get('message_count')}")
            print(
                f"  Connected duration: {status.get('connected_duration_ms')}ms"
            )

        except Exception as e:
            print(f"✗ Failed to get status: {e}")


# =============================================================================
# Example 5: WebSocket vs NDJSON Performance Comparison
# =============================================================================

async def compare_streaming_protocols():
    """Compare performance of WebSocket vs NDJSON streaming."""
    print("\n=== Example 5: WebSocket vs NDJSON Performance ===\n")

    async with AsyncRipTideClient(base_url="http://localhost:3000") as client:
        urls = [
            "https://example.com",
            "https://httpbin.org/html",
            "https://www.wikipedia.org",
            "https://github.com",
        ]

        options = CrawlOptions(cache_mode=CacheMode.NONE, concurrency=4)

        # Test WebSocket streaming
        print("Testing WebSocket streaming...\n")
        ws_start = time.time()
        ws_count = 0

        try:
            async for result in client.streaming.crawl_websocket(urls, options):
                if result.event_type == "result":
                    ws_count += 1
        except ImportError as e:
            print(f"⚠ WebSocket not available: {e}")
            ws_elapsed = 0
        else:
            ws_elapsed = time.time() - ws_start

        print(f"✓ WebSocket: {ws_count} results in {ws_elapsed:.2f}s")

        # Test NDJSON streaming
        print("\nTesting NDJSON streaming...\n")
        ndjson_start = time.time()
        ndjson_count = 0

        async for result in client.streaming.crawl_ndjson(urls, options):
            ndjson_count += 1

        ndjson_elapsed = time.time() - ndjson_start
        print(f"✓ NDJSON: {ndjson_count} results in {ndjson_elapsed:.2f}s")

        # Compare
        if ws_elapsed > 0:
            print("\n=== Performance Comparison ===")
            print(f"WebSocket: {ws_elapsed:.2f}s")
            print(f"NDJSON:    {ndjson_elapsed:.2f}s")
            if ws_elapsed < ndjson_elapsed:
                speedup = (ndjson_elapsed / ws_elapsed - 1) * 100
                print(f"WebSocket is {speedup:.1f}% faster")
            else:
                speedup = (ws_elapsed / ndjson_elapsed - 1) * 100
                print(f"NDJSON is {speedup:.1f}% faster")


# =============================================================================
# Example 6: Graceful Shutdown
# =============================================================================

async def websocket_graceful_shutdown():
    """Demonstrate graceful WebSocket shutdown with cleanup."""
    print("\n=== Example 6: Graceful Shutdown ===\n")

    shutdown_event = asyncio.Event()

    async def shutdown_handler():
        """Simulate shutdown signal after 3 seconds."""
        await asyncio.sleep(3)
        print("\n⚠ Shutdown signal received, cleaning up...")
        shutdown_event.set()

    async with AsyncRipTideClient(base_url="http://localhost:3000") as client:
        # Start shutdown handler
        shutdown_task = asyncio.create_task(shutdown_handler())

        urls = [
            f"https://httpbin.org/delay/{i}" for i in range(1, 6)
        ]  # 5 URLs with delays

        print(f"Starting WebSocket stream (will shutdown after 3s)...\n")

        try:
            async for result in client.streaming.crawl_websocket(urls):
                if shutdown_event.is_set():
                    print("✓ Graceful shutdown initiated")
                    break

                if result.event_type == "result":
                    crawl_result = result.data["result"]
                    print(f"Received: {crawl_result['url']}")

                elif result.event_type == "summary":
                    print("\n✓ Stream completed before shutdown")
                    break

        except asyncio.CancelledError:
            print("✓ Stream cancelled gracefully")

        finally:
            shutdown_task.cancel()
            print("✓ Cleanup completed")


# =============================================================================
# Example 7: Advanced WebSocket Features
# =============================================================================

async def advanced_websocket_features():
    """Demonstrate advanced WebSocket features."""
    print("\n=== Example 7: Advanced WebSocket Features ===\n")

    async with AsyncRipTideClient(base_url="http://localhost:3000") as client:
        urls = ["https://example.com", "https://httpbin.org/html"]

        # Use advanced crawl options
        options = CrawlOptions(
            cache_mode=CacheMode.READ_WRITE,
            concurrency=2,
            timeout_secs=30,
        )

        print("Streaming with advanced options...\n")

        metadata_received = None
        results = []

        async for result in client.streaming.crawl_websocket(urls, options):
            if result.event_type == "welcome":
                print("✓ WebSocket Features:")
                supported_ops = result.data.get("supported_operations", [])
                print(f"  Supported operations: {', '.join(supported_ops)}")

            elif result.event_type == "metadata":
                metadata_received = result.data
                print(f"\n✓ Stream metadata:")
                print(f"  Session ID: {metadata_received.get('session_id')}")
                print(f"  Timestamp: {metadata_received.get('timestamp')}")

            elif result.event_type == "result":
                crawl_result = result.data["result"]
                results.append(crawl_result)

                # Access progress information
                progress = result.data.get("progress", {})
                print(
                    f"\nProgress: {progress.get('completed')}/{progress.get('total')} "
                    f"(Success rate: {progress.get('success_rate', 0):.1%})"
                )

                # Show result details
                print(f"  URL: {crawl_result['url']}")
                print(f"  Cached: {crawl_result['from_cache']}")
                print(f"  Gate: {crawl_result['gate_decision']}")

            elif result.event_type == "summary":
                print(f"\n✓ Final Summary:")
                print(f"  Total: {result.data.get('total_urls')}")
                print(f"  Success: {result.data.get('successful')}")
                print(f"  Failed: {result.data.get('failed')}")
                print(
                    f"  Duration: {result.data.get('total_processing_time_ms')}ms"
                )


# =============================================================================
# Main Entry Point
# =============================================================================

async def main():
    """Run all examples."""
    examples = [
        ("Basic WebSocket Streaming", basic_websocket_streaming),
        ("WebSocket with Callbacks", websocket_with_callbacks),
        ("Error Handling", websocket_error_handling),
        ("Health Monitoring", websocket_health_monitoring),
        ("Performance Comparison", compare_streaming_protocols),
        ("Graceful Shutdown", websocket_graceful_shutdown),
        ("Advanced Features", advanced_websocket_features),
    ]

    print("=" * 70)
    print("RipTide SDK - WebSocket Streaming Examples")
    print("=" * 70)

    for i, (name, example_func) in enumerate(examples, 1):
        print(f"\n[{i}/{len(examples)}] {name}")
        print("-" * 70)

        try:
            await example_func()
        except Exception as e:
            print(f"\n✗ Example failed: {e}")
            import traceback

            traceback.print_exc()

        print("\n" + "=" * 70)

        # Small delay between examples
        if i < len(examples):
            await asyncio.sleep(1)

    print("\n✓ All examples completed!")


if __name__ == "__main__":
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        print("\n\n✓ Examples interrupted by user")
    except Exception as e:
        print(f"\n✗ Fatal error: {e}")
        import traceback

        traceback.print_exc()
