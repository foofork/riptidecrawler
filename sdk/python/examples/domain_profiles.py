"""
Domain Profiles example (Phase 10.4)

Demonstrates domain profile management and warm-start caching functionality.
"""

import asyncio
from riptide_sdk import RipTideClient
from riptide_sdk.models import ProfileConfig, ProfileMetadata, StealthLevel, UAStrategy


async def main():
    """Run domain profile examples"""

    async with RipTideClient(base_url="http://localhost:8080") as client:

        # Example 1: Create a domain profile
        print("Example 1: Create domain profile")
        print("-" * 50)

        config = ProfileConfig(
            stealth_level=StealthLevel.HIGH,
            rate_limit=2.0,
            respect_robots_txt=True,
            ua_strategy=UAStrategy.ROTATE,
            confidence_threshold=0.85,
            enable_javascript=True,
            request_timeout_secs=30,
        )

        metadata = ProfileMetadata(
            description="Production profile for example.com",
            tags=["production", "high-traffic"],
            author="DevOps Team",
        )

        profile = await client.profiles.create(
            domain="example.com",
            config=config,
            metadata=metadata,
        )

        print(f"Created profile: {profile.domain}")
        print(f"Stealth level: {profile.config.stealth_level.value if profile.config else 'N/A'}")
        print()

        # Example 2: Get profile
        print("\nExample 2: Get profile")
        print("-" * 50)

        profile = await client.profiles.get("example.com")
        print(f"Domain: {profile.domain}")
        if profile.config:
            print(f"Rate limit: {profile.config.rate_limit}")
            print(f"Respect robots.txt: {profile.config.respect_robots_txt}")
        print()

        # Example 3: Update profile
        print("\nExample 3: Update profile")
        print("-" * 50)

        updated_config = ProfileConfig(
            rate_limit=3.0,  # Increase rate limit
            confidence_threshold=0.90,  # Higher confidence
        )

        profile = await client.profiles.update(
            domain="example.com",
            config=updated_config,
        )
        print(f"Updated profile: {profile.domain}")
        print()

        # Example 4: Get profile statistics
        print("\nExample 4: Get profile statistics")
        print("-" * 50)

        stats = await client.profiles.get_stats("example.com")
        print(f"Domain: {stats.domain}")
        print(f"Total requests: {stats.total_requests}")
        print(f"Cache hits: {stats.cache_hits}")
        print(f"Cache misses: {stats.cache_misses}")

        if stats.total_requests > 0:
            hit_rate = stats.cache_hits / stats.total_requests
            print(f"Cache hit rate: {hit_rate:.2%}")

        print(f"Avg response time: {stats.avg_response_time_ms:.2f}ms")
        print()

        # Example 5: Warm cache for domain
        print("\nExample 5: Warm cache")
        print("-" * 50)

        result = await client.profiles.warm_cache(
            domain="example.com",
            url="https://example.com/api/data",
        )
        print(f"Cache warming result: {result}")
        print()

        # Example 6: List all profiles
        print("\nExample 6: List all profiles")
        print("-" * 50)

        profiles = await client.profiles.list()
        print(f"Found {len(profiles)} profiles:")
        for p in profiles:
            print(f"  - {p.domain}")
        print()

        # Example 7: Search profiles
        print("\nExample 7: Search profiles")
        print("-" * 50)

        results = await client.profiles.search("example")
        print(f"Search found {len(results)} profiles matching 'example':")
        for p in results:
            print(f"  - {p.domain}")
        print()

        # Example 8: Batch create profiles
        print("\nExample 8: Batch create profiles")
        print("-" * 50)

        from riptide_sdk.models import DomainProfile

        profiles = [
            DomainProfile(
                domain="example.org",
                config=ProfileConfig(stealth_level=StealthLevel.MEDIUM),
            ),
            DomainProfile(
                domain="example.net",
                config=ProfileConfig(stealth_level=StealthLevel.LOW),
            ),
        ]

        result = await client.profiles.batch_create(profiles)
        print(f"Created: {len(result.get('created', []))}")
        print(f"Failed: {len(result.get('failed', []))}")
        print()

        # Example 9: Get aggregated metrics
        print("\nExample 9: Get aggregated metrics")
        print("-" * 50)

        metrics = await client.profiles.get_metrics()
        print(f"Aggregated metrics: {metrics}")
        print()

        # Example 10: Delete profile
        print("\nExample 10: Delete profile")
        print("-" * 50)

        await client.profiles.delete("example.com")
        print("Deleted profile: example.com")


if __name__ == "__main__":
    asyncio.run(main())
