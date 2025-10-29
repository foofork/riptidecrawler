"""
Example: Using output formatters for various display formats

Demonstrates the built-in formatting methods for converting API
responses to Markdown, JSON, and summary formats.
"""

import asyncio
from riptide_sdk import RipTideClient


async def format_examples():
    """Demonstrate various output formats"""

    async with RipTideClient(base_url="http://localhost:8080") as client:
        # Perform a batch crawl
        result = await client.crawl.batch([
            "https://example.com",
            "https://example.org",
            "https://example.net",
        ])

        print("=" * 60)
        print("SUMMARY FORMAT (Quick Overview)")
        print("=" * 60)
        print(result.to_summary())

        print("\n" + "=" * 60)
        print("MARKDOWN FORMAT (Detailed)")
        print("=" * 60)
        print(result.to_markdown())

        print("\n" + "=" * 60)
        print("JSON FORMAT (For APIs/Storage)")
        print("=" * 60)
        print(result.to_json(include_documents=False))

        # Domain profile formatting
        print("\n" + "=" * 60)
        print("DOMAIN PROFILE FORMATS")
        print("=" * 60)

        profile = await client.profiles.get("example.com")
        if profile:
            print("\nProfile Summary:")
            print(profile.to_summary())

            print("\nProfile Markdown:")
            print(profile.to_markdown())

        # Engine stats formatting
        print("\n" + "=" * 60)
        print("ENGINE STATS FORMATS")
        print("=" * 60)

        stats = await client.engine.get_stats()
        print("\nStats Summary:")
        print(stats.to_summary())

        print("\nStats Markdown:")
        print(stats.to_markdown())


async def custom_formatting():
    """Use the standalone formatter functions"""

    from riptide_sdk import (
        format_crawl_response,
        format_domain_profile,
        format_engine_stats
    )

    async with RipTideClient(base_url="http://localhost:8080") as client:
        result = await client.crawl.batch(["https://example.com"])

        # Use standalone functions for more control
        summary = format_crawl_response(result, format="summary")
        markdown = format_crawl_response(result, format="markdown", include_documents=True)
        json_output = format_crawl_response(result, format="json", include_documents=False)

        print("Custom formatted output:")
        print(summary)


if __name__ == "__main__":
    print("RipTide SDK - Output Formatters Example\n")
    asyncio.run(format_examples())

    print("\n" + "=" * 60)
    print("Custom Formatting Functions")
    print("=" * 60)
    asyncio.run(custom_formatting())
