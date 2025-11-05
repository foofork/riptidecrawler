"""
Performance benchmarks for Riptide Python SDK.

Run with:
    maturin develop && pytest tests/test_performance.py -v
"""

import pytest
import time
import riptide


class TestPerformance:
    """Performance benchmark tests."""

    def test_instance_creation_overhead(self):
        """Benchmark RipTide instance creation."""
        iterations = 10
        start = time.time()

        for _ in range(iterations):
            rt = riptide.RipTide()
            assert rt.is_healthy()

        elapsed = time.time() - start
        avg_time = (elapsed / iterations) * 1000  # Convert to ms

        print(f"\nInstance creation: {avg_time:.2f}ms average")
        assert avg_time < 100, f"Instance creation too slow: {avg_time:.2f}ms"

    def test_extract_overhead(self):
        """Benchmark extract() method overhead."""
        rt = riptide.RipTide()

        # Warm-up
        rt.extract("https://example.com")

        # Benchmark
        iterations = 5
        start = time.time()

        for _ in range(iterations):
            doc = rt.extract("https://example.com")
            assert doc is not None

        elapsed = time.time() - start
        avg_time = (elapsed / iterations) * 1000

        print(f"\nExtract operation: {avg_time:.2f}ms average")
        # Note: This includes network time, so we're lenient
        assert avg_time < 5000, f"Extract too slow: {avg_time:.2f}ms"

    def test_spider_overhead(self):
        """Benchmark spider() method overhead."""
        rt = riptide.RipTide()

        # Benchmark
        iterations = 3
        start = time.time()

        for _ in range(iterations):
            urls = rt.spider("https://example.com", max_depth=1, max_urls=5)
            assert len(urls) > 0

        elapsed = time.time() - start
        avg_time = (elapsed / iterations) * 1000

        print(f"\nSpider operation: {avg_time:.2f}ms average")
        assert avg_time < 1000, f"Spider too slow: {avg_time:.2f}ms"

    def test_crawl_throughput(self):
        """Benchmark crawl() throughput."""
        rt = riptide.RipTide()

        urls = ["https://example.com"] * 5

        start = time.time()
        docs = rt.crawl(urls)
        elapsed = time.time() - start

        throughput = len(urls) / elapsed

        print(f"\nCrawl throughput: {throughput:.2f} URLs/second")
        print(f"Total time: {elapsed*1000:.2f}ms for {len(urls)} URLs")

        assert len(docs) == len(urls)

    def test_document_to_dict_overhead(self):
        """Benchmark Document.to_dict() overhead."""
        rt = riptide.RipTide()
        doc = rt.extract("https://example.com")

        iterations = 1000
        start = time.time()

        for _ in range(iterations):
            doc_dict = doc.to_dict()
            assert "url" in doc_dict

        elapsed = time.time() - start
        avg_time = (elapsed / iterations) * 1000

        print(f"\nDocument.to_dict(): {avg_time:.4f}ms average")
        assert avg_time < 1, f"to_dict() too slow: {avg_time:.4f}ms"

    def test_concurrent_requests(self):
        """Benchmark concurrent request handling."""
        rt = riptide.RipTide()

        # Multiple concurrent extracts
        urls = [
            "https://example.com",
            "https://example.org",
            "https://example.net",
        ]

        start = time.time()
        docs = rt.crawl(urls)
        elapsed = time.time() - start

        print(f"\nConcurrent crawl: {elapsed*1000:.2f}ms for {len(urls)} URLs")
        print(f"Average: {(elapsed/len(urls))*1000:.2f}ms per URL")

        assert len(docs) == len(urls)


class TestMemoryUsage:
    """Memory usage tests."""

    def test_document_memory_footprint(self):
        """Test document memory usage."""
        rt = riptide.RipTide()
        doc = rt.extract("https://example.com")

        # Basic check - document should have reasonable size
        text_size = len(doc.text)
        assert text_size > 0
        print(f"\nDocument text size: {text_size} bytes")

    def test_multiple_instances_memory(self):
        """Test memory usage with multiple instances."""
        instances = []
        for _ in range(10):
            rt = riptide.RipTide()
            instances.append(rt)

        # All should be healthy
        assert all(rt.is_healthy() for rt in instances)
        print(f"\nCreated {len(instances)} instances successfully")


class TestScalability:
    """Scalability tests."""

    @pytest.mark.slow
    def test_large_batch_crawl(self):
        """Test crawling a larger batch of URLs."""
        rt = riptide.RipTide()

        # Generate 20 URLs
        urls = [f"https://example.com/page{i}" for i in range(20)]

        start = time.time()
        docs = rt.crawl(urls)
        elapsed = time.time() - start

        throughput = len(urls) / elapsed

        print(f"\nLarge batch: {elapsed:.2f}s for {len(urls)} URLs")
        print(f"Throughput: {throughput:.2f} URLs/second")

        assert len(docs) == len(urls)

    def test_repeated_operations(self):
        """Test repeated operations don't degrade performance."""
        rt = riptide.RipTide()

        times = []
        for i in range(5):
            start = time.time()
            doc = rt.extract("https://example.com")
            elapsed = time.time() - start
            times.append(elapsed * 1000)
            assert doc is not None

        avg_time = sum(times) / len(times)
        print(f"\nRepeated extracts: {avg_time:.2f}ms average")
        print(f"Times: {[f'{t:.2f}ms' for t in times]}")

        # Performance should be consistent
        assert max(times) < avg_time * 2, "Performance degraded significantly"


if __name__ == "__main__":
    pytest.main([__file__, "-v", "-s"])
