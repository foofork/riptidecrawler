#!/usr/bin/env python3
"""
Riptide Python SDK - Basic Usage Examples

This script demonstrates the core functionality of the Riptide Python SDK.

Usage:
    maturin develop && python examples/basic_usage.py
"""

import riptide

def example_extract():
    """Example 1: Extract content from a single URL"""
    print("=" * 70)
    print("Example 1: Extract Single URL")
    print("=" * 70)

    rt = riptide.RipTide()

    # Extract in standard mode (fast)
    doc = rt.extract("https://example.com")

    print(f"URL: {doc.url}")
    print(f"Title: {doc.title}")
    print(f"Text length: {len(doc.text)} characters")
    print(f"Word count: {doc.word_count}")
    print(f"Quality score: {doc.quality_score:.2f}")
    print(f"From cache: {doc.from_cache}")
    print(f"Processing time: {doc.processing_time_ms}ms")
    print()

def example_extract_enhanced():
    """Example 2: Extract with enhanced mode"""
    print("=" * 70)
    print("Example 2: Extract with Enhanced Mode")
    print("=" * 70)

    rt = riptide.RipTide()

    # Enhanced mode uses multiple extraction strategies
    doc = rt.extract("https://example.com", mode="enhanced")

    print(f"Enhanced extraction:")
    print(f"  URL: {doc.url}")
    print(f"  Title: {doc.title}")
    print(f"  Quality: {doc.quality_score:.2f}")
    print()

def example_spider():
    """Example 3: Spider to discover URLs"""
    print("=" * 70)
    print("Example 3: Spider URL Discovery")
    print("=" * 70)

    rt = riptide.RipTide()

    # Spider with depth 2, max 50 URLs
    urls = rt.spider("https://example.com", max_depth=2, max_urls=50)

    print(f"Discovered {len(urls)} URLs:")
    for i, url in enumerate(urls[:5], 1):
        print(f"  {i}. {url}")

    if len(urls) > 5:
        print(f"  ... and {len(urls) - 5} more")
    print()

def example_batch_crawl():
    """Example 4: Batch crawl multiple URLs"""
    print("=" * 70)
    print("Example 4: Batch Crawl")
    print("=" * 70)

    rt = riptide.RipTide()

    urls = [
        "https://example.com",
        "https://example.org",
        "https://example.net",
    ]

    docs = rt.crawl(urls)

    print(f"Crawled {len(docs)} URLs:")
    for doc in docs:
        print(f"  {doc.url}")
        print(f"    Title: {doc.title}")
        print(f"    Words: {doc.word_count}")
        print(f"    Quality: {doc.quality_score:.2f}")
    print()

def example_document_methods():
    """Example 5: Document methods and properties"""
    print("=" * 70)
    print("Example 5: Document Methods")
    print("=" * 70)

    rt = riptide.RipTide()
    doc = rt.extract("https://example.com")

    # String representations
    print(f"repr: {repr(doc)}")
    print(f"str: {str(doc)}")
    print(f"len: {len(doc)} characters")

    # Convert to dictionary
    doc_dict = doc.to_dict()
    print(f"\nAs dictionary:")
    for key, value in doc_dict.items():
        if isinstance(value, str) and len(value) > 50:
            value = value[:50] + "..."
        print(f"  {key}: {value}")
    print()

def example_error_handling():
    """Example 6: Error handling"""
    print("=" * 70)
    print("Example 6: Error Handling")
    print("=" * 70)

    rt = riptide.RipTide()

    try:
        # Empty URL should raise ValueError
        doc = rt.extract("")
    except ValueError as e:
        print(f"✅ Caught ValueError: {e}")

    try:
        # Invalid mode should raise ValueError
        doc = rt.extract("https://example.com", mode="invalid")
    except ValueError as e:
        print(f"✅ Caught ValueError: {e}")

    try:
        # Empty URLs list should raise ValueError
        docs = rt.crawl([])
    except ValueError as e:
        print(f"✅ Caught ValueError: {e}")

    print()

def example_version_and_health():
    """Example 7: Version and health check"""
    print("=" * 70)
    print("Example 7: Version and Health")
    print("=" * 70)

    rt = riptide.RipTide()

    print(f"Riptide version: {riptide.RipTide.version()}")
    print(f"Instance healthy: {rt.is_healthy()}")
    print(f"Instance repr: {repr(rt)}")
    print(f"Instance str: {str(rt)}")
    print()

def main():
    """Run all examples"""
    print("\n" + "=" * 70)
    print("Riptide Python SDK - Usage Examples")
    print("=" * 70 + "\n")

    examples = [
        example_extract,
        example_extract_enhanced,
        example_spider,
        example_batch_crawl,
        example_document_methods,
        example_error_handling,
        example_version_and_health,
    ]

    for example in examples:
        try:
            example()
        except Exception as e:
            print(f"❌ Example failed: {e}")
            import traceback
            traceback.print_exc()
            print()

    print("=" * 70)
    print("Examples complete!")
    print("=" * 70)

if __name__ == "__main__":
    main()
