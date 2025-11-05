"""
Pytest tests for Riptide Python SDK.

Run with:
    maturin develop && pytest tests/
"""

import pytest
import riptide


class TestRipTideClass:
    """Tests for the RipTide class."""

    def test_create_instance(self):
        """Test creating a RipTide instance."""
        rt = riptide.RipTide()
        assert rt is not None

    def test_create_instance_with_api_key(self):
        """Test creating instance with API key."""
        rt = riptide.RipTide(api_key="test-key")
        assert rt is not None

    def test_version(self):
        """Test version method."""
        version = riptide.RipTide.version()
        assert isinstance(version, str)
        assert len(version) > 0

    def test_is_healthy(self):
        """Test health check."""
        rt = riptide.RipTide()
        assert rt.is_healthy() is True

    def test_repr(self):
        """Test __repr__."""
        rt = riptide.RipTide()
        repr_str = repr(rt)
        assert "RipTide" in repr_str
        assert "version" in repr_str

    def test_str(self):
        """Test __str__."""
        rt = riptide.RipTide()
        str_str = str(rt)
        assert "RipTide" in str_str


class TestExtract:
    """Tests for extract() method."""

    def test_extract_basic(self):
        """Test basic extraction."""
        rt = riptide.RipTide()
        doc = rt.extract("https://example.com")

        assert isinstance(doc, riptide.Document)
        assert doc.url == "https://example.com"
        assert isinstance(doc.title, str)
        assert isinstance(doc.text, str)

    def test_extract_with_mode_standard(self):
        """Test extraction with standard mode."""
        rt = riptide.RipTide()
        doc = rt.extract("https://example.com", mode="standard")
        assert isinstance(doc, riptide.Document)

    def test_extract_with_mode_enhanced(self):
        """Test extraction with enhanced mode."""
        rt = riptide.RipTide()
        doc = rt.extract("https://example.com", mode="enhanced")
        assert isinstance(doc, riptide.Document)

    def test_extract_empty_url_raises(self):
        """Test that empty URL raises ValueError."""
        rt = riptide.RipTide()
        with pytest.raises(ValueError, match="URL cannot be empty"):
            rt.extract("")

    def test_extract_invalid_mode_raises(self):
        """Test that invalid mode raises ValueError."""
        rt = riptide.RipTide()
        with pytest.raises(ValueError, match="Invalid mode"):
            rt.extract("https://example.com", mode="invalid")


class TestSpider:
    """Tests for spider() method."""

    def test_spider_basic(self):
        """Test basic spider operation."""
        rt = riptide.RipTide()
        urls = rt.spider("https://example.com")

        assert isinstance(urls, list)
        assert len(urls) > 0
        assert all(isinstance(url, str) for url in urls)

    def test_spider_with_depth(self):
        """Test spider with custom depth."""
        rt = riptide.RipTide()
        urls = rt.spider("https://example.com", max_depth=3)
        assert isinstance(urls, list)

    def test_spider_with_max_urls(self):
        """Test spider with max URL limit."""
        rt = riptide.RipTide()
        urls = rt.spider("https://example.com", max_depth=2, max_urls=50)
        assert len(urls) <= 50

    def test_spider_empty_url_raises(self):
        """Test that empty URL raises ValueError."""
        rt = riptide.RipTide()
        with pytest.raises(ValueError, match="URL cannot be empty"):
            rt.spider("")


class TestCrawl:
    """Tests for crawl() method."""

    def test_crawl_single_url(self):
        """Test crawling a single URL."""
        rt = riptide.RipTide()
        docs = rt.crawl(["https://example.com"])

        assert isinstance(docs, list)
        assert len(docs) == 1
        assert isinstance(docs[0], riptide.Document)

    def test_crawl_multiple_urls(self):
        """Test crawling multiple URLs."""
        rt = riptide.RipTide()
        urls = [
            "https://example.com",
            "https://example.org",
            "https://example.net",
        ]
        docs = rt.crawl(urls)

        assert len(docs) == len(urls)
        assert all(isinstance(doc, riptide.Document) for doc in docs)

    def test_crawl_with_mode(self):
        """Test crawl with different modes."""
        rt = riptide.RipTide()
        docs = rt.crawl(["https://example.com"], mode="standard")
        assert len(docs) == 1

        docs = rt.crawl(["https://example.com"], mode="enhanced")
        assert len(docs) == 1

    def test_crawl_empty_list_raises(self):
        """Test that empty URL list raises ValueError."""
        rt = riptide.RipTide()
        with pytest.raises(ValueError, match="URLs list cannot be empty"):
            rt.crawl([])

    def test_crawl_invalid_mode_raises(self):
        """Test that invalid mode raises ValueError."""
        rt = riptide.RipTide()
        with pytest.raises(ValueError, match="Invalid mode"):
            rt.crawl(["https://example.com"], mode="invalid")


class TestDocument:
    """Tests for Document class."""

    @pytest.fixture
    def sample_doc(self):
        """Create a sample document for testing."""
        rt = riptide.RipTide()
        return rt.extract("https://example.com")

    def test_document_properties(self, sample_doc):
        """Test document properties."""
        assert isinstance(sample_doc.url, str)
        assert isinstance(sample_doc.title, str)
        assert isinstance(sample_doc.text, str)
        assert isinstance(sample_doc.quality_score, float)
        assert isinstance(sample_doc.word_count, int)
        assert isinstance(sample_doc.from_cache, bool)
        assert isinstance(sample_doc.processing_time_ms, int)

    def test_document_html_optional(self, sample_doc):
        """Test that HTML is optional."""
        assert sample_doc.html is None or isinstance(sample_doc.html, str)

    def test_document_to_dict(self, sample_doc):
        """Test to_dict() method."""
        doc_dict = sample_doc.to_dict()

        assert isinstance(doc_dict, dict)
        assert "url" in doc_dict
        assert "title" in doc_dict
        assert "text" in doc_dict
        assert "quality_score" in doc_dict
        assert "word_count" in doc_dict

    def test_document_repr(self, sample_doc):
        """Test __repr__."""
        repr_str = repr(sample_doc)
        assert "Document" in repr_str
        assert "url=" in repr_str

    def test_document_str(self, sample_doc):
        """Test __str__."""
        str_str = str(sample_doc)
        assert sample_doc.url in str_str
        assert sample_doc.title in str_str

    def test_document_len(self, sample_doc):
        """Test __len__."""
        length = len(sample_doc)
        assert isinstance(length, int)
        assert length == len(sample_doc.text)


class TestSpikeCompatibility:
    """Tests for spike test compatibility."""

    def test_spike_functions_exist(self):
        """Test that spike test functions still exist."""
        assert hasattr(riptide, 'test_async_basic')
        assert hasattr(riptide, 'test_async_concurrent')
        assert hasattr(riptide, 'test_async_timeout')
        assert hasattr(riptide, 'test_async_error_handling')

    def test_spike_class_exists(self):
        """Test that spike test class still exists."""
        assert hasattr(riptide, 'RipTideSpike')

    def test_spike_basic_async(self):
        """Test spike basic async function."""
        result = riptide.test_async_basic()
        assert isinstance(result, str)

    def test_spike_concurrent(self):
        """Test spike concurrent function."""
        results = riptide.test_async_concurrent()
        assert isinstance(results, list)
        assert len(results) == 5


class TestErrorHandling:
    """Tests for error handling."""

    def test_value_error_empty_url_extract(self):
        """Test ValueError for empty URL in extract."""
        rt = riptide.RipTide()
        with pytest.raises(ValueError):
            rt.extract("")

    def test_value_error_empty_url_spider(self):
        """Test ValueError for empty URL in spider."""
        rt = riptide.RipTide()
        with pytest.raises(ValueError):
            rt.spider("")

    def test_value_error_empty_urls_crawl(self):
        """Test ValueError for empty URLs list in crawl."""
        rt = riptide.RipTide()
        with pytest.raises(ValueError):
            rt.crawl([])

    def test_value_error_invalid_mode(self):
        """Test ValueError for invalid mode."""
        rt = riptide.RipTide()
        with pytest.raises(ValueError):
            rt.extract("https://example.com", mode="invalid_mode")


class TestIntegration:
    """Integration tests."""

    def test_extract_then_spider(self):
        """Test extract followed by spider."""
        rt = riptide.RipTide()

        # Extract first
        doc = rt.extract("https://example.com")
        assert doc is not None

        # Then spider
        urls = rt.spider("https://example.com")
        assert len(urls) > 0

    def test_spider_then_crawl(self):
        """Test spider followed by crawl."""
        rt = riptide.RipTide()

        # Spider to get URLs
        urls = rt.spider("https://example.com", max_depth=1, max_urls=3)
        assert len(urls) > 0

        # Crawl discovered URLs
        docs = rt.crawl(urls[:2])  # Crawl first 2 URLs
        assert len(docs) == 2

    def test_multiple_instances(self):
        """Test creating multiple RipTide instances."""
        rt1 = riptide.RipTide()
        rt2 = riptide.RipTide()

        doc1 = rt1.extract("https://example.com")
        doc2 = rt2.extract("https://example.org")

        assert doc1.url != doc2.url
