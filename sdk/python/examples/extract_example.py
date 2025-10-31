#!/usr/bin/env python3
"""
Example usage of the Extract API endpoint

The Extract API provides single-URL content extraction with support for:
- Multi-strategy extraction (native, WASM, multi-fallback)
- Multiple modes (standard, article, markdown, product)
- Quality scoring and parser observability
- Comprehensive metadata extraction

Strategy Changes in v0.9.0+:
- Default strategy changed from "multi" to "native" for better performance
- Native: 2-5ms per page (always available)
- WASM: 10-20ms per page (requires server feature flag)
- Multi: Server auto-selects (falls back to native if WASM unavailable)
"""

import asyncio
from riptide_sdk import RipTideClient
from riptide_sdk.models import ExtractOptions
from riptide_sdk.exceptions import ExtractionError


async def basic_extraction():
    """Basic content extraction"""
    print("1. Basic Extraction")
    print("=" * 60)

    async with RipTideClient(base_url="http://localhost:8080") as client:
        # Simple extraction with defaults
        result = await client.extract.extract("https://example.com")

        print(f"URL: {result.url}")
        print(f"Title: {result.title}")
        print(f"Quality Score: {result.quality_score:.2f}")
        print(f"Strategy Used: {result.strategy_used}")
        print(f"Word Count: {result.metadata.word_count}")
        print(f"Extraction Time: {result.extraction_time_ms}ms")
        print()


async def extraction_with_options():
    """Extraction with custom options"""
    print("2. Extraction with Custom Options")
    print("=" * 60)

    async with RipTideClient(base_url="http://localhost:8080") as client:
        # Custom extraction options
        options = ExtractOptions(
            strategy="native",  # Explicit native strategy (default in v0.9.0+)
            quality_threshold=0.8,  # Higher quality threshold
            timeout_ms=15000  # 15 second timeout
        )

        result = await client.extract.extract(
            "https://example.com/article",
            mode="article",
            options=options
        )

        # Use the to_summary() method for a formatted overview
        print(result.to_summary())
        print()


async def article_extraction():
    """Extract in article mode (optimized for news/blog posts)"""
    print("3. Article Mode Extraction")
    print("=" * 60)

    async with RipTideClient(base_url="http://localhost:8080") as client:
        result = await client.extract.extract_article(
            "https://example.com/blog/post"
        )

        print(f"Title: {result.title}")
        print(f"Author: {result.metadata.author or 'Unknown'}")
        print(f"Published: {result.metadata.publish_date or 'Unknown'}")
        print(f"Language: {result.metadata.language or 'Unknown'}")
        print(f"Content Preview: {result.content[:200]}...")
        print()


async def markdown_extraction():
    """Extract content as Markdown"""
    print("4. Markdown Extraction")
    print("=" * 60)

    async with RipTideClient(base_url="http://localhost:8080") as client:
        result = await client.extract.extract_markdown(
            "https://example.com/documentation"
        )

        print(f"Markdown Content:\n{result.content[:300]}...")
        print()


async def product_extraction():
    """Extract in product mode (optimized for e-commerce)"""
    print("5. Product Mode Extraction")
    print("=" * 60)

    async with RipTideClient(base_url="http://localhost:8080") as client:
        result = await client.extract.extract_product(
            "https://shop.example.com/product/123"
        )

        print(f"Product: {result.title}")
        print(f"Quality: {result.quality_score:.2f}")
        print()


async def batch_extraction():
    """Extract multiple URLs in parallel"""
    print("6. Batch Extraction (Parallel)")
    print("=" * 60)

    urls = [
        "https://example.com/page1",
        "https://example.com/page2",
        "https://example.com/page3",
    ]

    async with RipTideClient(base_url="http://localhost:8080") as client:
        # Create tasks for parallel execution
        tasks = [client.extract.extract(url) for url in urls]

        # Execute in parallel
        results = await asyncio.gather(*tasks, return_exceptions=True)

        for i, result in enumerate(results):
            if isinstance(result, Exception):
                print(f"URL {i+1}: Failed - {result}")
            else:
                print(f"URL {i+1}: {result.title} ({result.quality_score:.2f})")

        print()


async def extraction_with_observability():
    """Extraction with parser metadata for observability"""
    print("7. Extraction with Parser Observability")
    print("=" * 60)

    async with RipTideClient(base_url="http://localhost:8080") as client:
        result = await client.extract.extract("https://example.com")

        print(f"Strategy Used: {result.strategy_used}")
        print(f"Quality Score: {result.quality_score:.2f}")

        if result.parser_metadata:
            print(f"\nParser Details:")
            print(f"  Parser: {result.parser_metadata.parser_used}")
            print(f"  Confidence: {result.parser_metadata.confidence_score:.2f}")
            print(f"  Fallback Occurred: {result.parser_metadata.fallback_occurred}")
            print(f"  Parse Time: {result.parser_metadata.parse_time_ms}ms")

            if result.parser_metadata.extraction_path:
                print(f"  Path: {result.parser_metadata.extraction_path}")
            if result.parser_metadata.primary_error:
                print(f"  Primary Error: {result.parser_metadata.primary_error}")

        print()


async def strategy_comparison():
    """Compare different extraction strategies (v0.9.0+)"""
    print("8. Strategy Comparison (Native vs WASM)")
    print("=" * 60)

    async with RipTideClient(base_url="http://localhost:8080") as client:
        url = "https://example.com"

        # 1. Native extraction (default, fastest)
        print("Native Strategy (default):")
        options_native = ExtractOptions(strategy="native")
        result_native = await client.extract.extract(url, options=options_native)
        print(f"  Strategy: {result_native.strategy_used}")
        print(f"  Time: {result_native.extraction_time_ms}ms")
        print(f"  Quality: {result_native.quality_score:.2f}")

        # 2. WASM extraction (only if server supports it)
        print("\nWASM Strategy (if available):")
        try:
            options_wasm = ExtractOptions(strategy="wasm")
            result_wasm = await client.extract.extract(url, options=options_wasm)
            print(f"  Strategy: {result_wasm.strategy_used}")
            print(f"  Time: {result_wasm.extraction_time_ms}ms")
            print(f"  Quality: {result_wasm.quality_score:.2f}")
            print(f"  Slowdown: {result_wasm.extraction_time_ms / result_native.extraction_time_ms:.1f}x")
        except ExtractionError as e:
            print(f"  ⚠️  WASM not available: {e}")
            print(f"  Server needs: cargo build --features wasm-extractor")

        # 3. Multi-strategy (server auto-selects)
        print("\nMulti Strategy (auto-select):")
        options_multi = ExtractOptions(strategy="multi")
        result_multi = await client.extract.extract(url, options=options_multi)
        print(f"  Server selected: {result_multi.strategy_used}")
        print(f"  Time: {result_multi.extraction_time_ms}ms")

        print()


async def graceful_fallback_pattern():
    """Demonstrate graceful fallback from WASM to native"""
    print("9. Graceful Fallback Pattern")
    print("=" * 60)

    async with RipTideClient(base_url="http://localhost:8080") as client:
        url = "https://example.com"

        # Try strategies in order of preference
        strategies = ["wasm", "native"]

        for strategy in strategies:
            try:
                print(f"Trying {strategy} strategy...")
                options = ExtractOptions(strategy=strategy)
                result = await client.extract.extract(url, options=options)

                print(f"  ✅ Success with {strategy}")
                print(f"  Strategy used: {result.strategy_used}")
                print(f"  Extraction time: {result.extraction_time_ms}ms")
                break

            except ExtractionError as e:
                if "not available" in str(e).lower():
                    print(f"  ⚠️  {strategy} unavailable: {e}")
                    print(f"  Falling back to next strategy...")
                    continue
                else:
                    # Other errors should be raised
                    print(f"  ❌ {strategy} failed: {e}")
                    raise
        else:
            print("  ❌ All strategies failed!")

        print()


async def server_compatibility_check():
    """Check which strategies are available on the server"""
    print("10. Server Compatibility Check")
    print("=" * 60)

    async with RipTideClient(base_url="http://localhost:8080") as client:
        strategies_to_test = ["native", "wasm", "multi"]
        available_strategies = []

        for strategy in strategies_to_test:
            try:
                options = ExtractOptions(strategy=strategy)
                result = await client.extract.extract("https://example.com", options=options)
                available_strategies.append(strategy)
                print(f"  ✅ {strategy}: Available (used: {result.strategy_used})")
            except ExtractionError as e:
                if "not available" in str(e).lower():
                    print(f"  ❌ {strategy}: Not available")
                else:
                    print(f"  ⚠️  {strategy}: Error - {e}")

        print(f"\nAvailable strategies: {', '.join(available_strategies)}")

        if "wasm" not in available_strategies:
            print("\n💡 Tip: To enable WASM extraction:")
            print("   cargo build --release --features wasm-extractor")
            print("   export WASM_EXTRACTOR_PATH=/path/to/extractor.wasm")

        print()


async def main():
    """Run all examples"""
    print("\n" + "=" * 60)
    print("Extract API Examples (v0.9.0+)")
    print("=" * 60 + "\n")

    try:
        await basic_extraction()
        await extraction_with_options()
        await article_extraction()
        await markdown_extraction()
        await product_extraction()
        await batch_extraction()
        await extraction_with_observability()

        # New examples for v0.9.0+
        await strategy_comparison()
        await graceful_fallback_pattern()
        await server_compatibility_check()

        print("=" * 60)
        print("All examples completed successfully!")
        print("=" * 60)

    except Exception as e:
        print(f"\nError: {e}")
        print("Make sure the RipTide API is running at http://localhost:8080")


if __name__ == "__main__":
    asyncio.run(main())
