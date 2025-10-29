#!/usr/bin/env python3
"""
Example usage of the Extract API endpoint

The Extract API provides single-URL content extraction with support for:
- Multi-strategy extraction (CSS, WASM, hybrid fallback)
- Multiple modes (standard, article, markdown, product)
- Quality scoring and parser observability
- Comprehensive metadata extraction
"""

import asyncio
from riptide_sdk import RipTideClient
from riptide_sdk.models import ExtractOptions


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
            strategy="wasm",  # Force WASM parser
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


async def main():
    """Run all examples"""
    print("\n" + "=" * 60)
    print("Extract API Examples")
    print("=" * 60 + "\n")

    try:
        await basic_extraction()
        await extraction_with_options()
        await article_extraction()
        await markdown_extraction()
        await product_extraction()
        await batch_extraction()
        await extraction_with_observability()

        print("=" * 60)
        print("All examples completed successfully!")
        print("=" * 60)

    except Exception as e:
        print(f"\nError: {e}")
        print("Make sure the RipTide API is running at http://localhost:8080")


if __name__ == "__main__":
    asyncio.run(main())
