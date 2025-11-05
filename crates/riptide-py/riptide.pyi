"""
Type hints for Riptide Python bindings.

This stub file provides type information for IDEs and type checkers
like mypy, pyright, and pylance.
"""

from typing import Optional, List, Dict, Any

class RipTide:
    """
    Main RipTide class for web scraping operations.

    This class provides a high-level interface for extracting content,
    discovering URLs, and batch processing web pages.

    Example:
        >>> import riptide
        >>> rt = riptide.RipTide()
        >>> doc = rt.extract("https://example.com")
        >>> print(doc.title)
    """

    def __init__(self, api_key: Optional[str] = None) -> None:
        """
        Create a new RipTide instance.

        Args:
            api_key: Optional API key for future cloud features

        Raises:
            RuntimeError: If async runtime creation fails
        """
        ...

    def extract(self, url: str, mode: str = "standard") -> Document:
        """
        Extract content from a single URL.

        This method fetches a URL and extracts structured content including
        title, text, metadata, and more.

        Args:
            url: URL to extract content from
            mode: Extraction mode - "standard" (default) or "enhanced"

        Returns:
            Document object containing extracted content

        Raises:
            ValueError: If URL is empty or mode is invalid
            RuntimeError: If network or extraction error occurs
            TimeoutError: If request times out

        Example:
            >>> rt = riptide.RipTide()
            >>> doc = rt.extract("https://example.com")
            >>> print(doc.title, doc.text)
        """
        ...

    def spider(
        self,
        url: str,
        max_depth: int = 2,
        max_urls: int = 100
    ) -> List[str]:
        """
        Spider a URL to discover linked URLs.

        This method crawls a URL and discovers all linked URLs up to a
        specified depth. It does NOT extract content, only discovers URLs.

        Args:
            url: Starting URL to spider
            max_depth: Maximum depth to crawl (default: 2)
            max_urls: Maximum URLs to discover (default: 100)

        Returns:
            List of discovered URLs

        Raises:
            ValueError: If URL is empty
            RuntimeError: If spider operation fails

        Example:
            >>> rt = riptide.RipTide()
            >>> urls = rt.spider("https://example.com", max_depth=3)
            >>> print(f"Found {len(urls)} URLs")
        """
        ...

    def crawl(self, urls: List[str], mode: str = "standard") -> List[Document]:
        """
        Batch crawl multiple URLs.

        This method extracts content from multiple URLs in parallel.

        Args:
            urls: List of URLs to crawl
            mode: Extraction mode - "standard" (default) or "enhanced"

        Returns:
            List of Document objects (one per URL)

        Raises:
            ValueError: If URLs list is empty or mode is invalid
            RuntimeError: If batch crawl fails

        Example:
            >>> rt = riptide.RipTide()
            >>> urls = ["https://example.com", "https://example.org"]
            >>> docs = rt.crawl(urls)
            >>> for doc in docs:
            ...     print(doc.title)
        """
        ...

    @staticmethod
    def version() -> str:
        """
        Get the version of the Riptide library.

        Returns:
            Version string (e.g., "0.1.0")
        """
        ...

    def is_healthy(self) -> bool:
        """
        Check if the RipTide instance is healthy.

        Returns:
            True if healthy, False otherwise
        """
        ...

    def __repr__(self) -> str: ...
    def __str__(self) -> str: ...


class Document:
    """
    Document class representing extracted web content.

    This class holds all extracted content including title, text,
    metadata, and processing information.

    Attributes:
        url: Source URL
        title: Page title
        text: Extracted text content
        html: Raw HTML (if available)
        quality_score: Content quality score (0.0-1.0)
        word_count: Number of words
        from_cache: Whether content was cached
        processing_time_ms: Processing time in milliseconds
    """

    url: str
    title: str
    text: str
    html: Optional[str]
    quality_score: float
    word_count: int
    from_cache: bool
    processing_time_ms: int

    def __init__(
        self,
        url: str,
        title: str,
        text: str,
        html: Optional[str],
        quality_score: float,
        word_count: int,
        from_cache: bool,
        processing_time_ms: int
    ) -> None:
        """
        Create a new Document.

        Args:
            url: Source URL
            title: Page title
            text: Extracted text content
            html: Raw HTML (optional)
            quality_score: Content quality score (0.0-1.0)
            word_count: Number of words
            from_cache: Whether content was cached
            processing_time_ms: Processing time in milliseconds
        """
        ...

    def to_dict(self) -> Dict[str, Any]:
        """
        Convert document to dictionary.

        Returns:
            Dictionary with all document fields

        Example:
            >>> doc = rt.extract("https://example.com")
            >>> doc_dict = doc.to_dict()
            >>> print(doc_dict['title'])
        """
        ...

    def __repr__(self) -> str: ...
    def __str__(self) -> str: ...
    def __len__(self) -> int:
        """Returns the length of the text content."""
        ...


# Spike test functions (for testing only)
def test_async_basic() -> str:
    """Test basic async operation. For testing only."""
    ...

def test_async_concurrent() -> List[str]:
    """Test concurrent async operations. For testing only."""
    ...

def test_async_timeout(timeout_ms: int) -> str:
    """Test async timeout handling. For testing only."""
    ...

def test_async_error_handling(should_fail: bool) -> str:
    """Test async error handling. For testing only."""
    ...


class RipTideSpike:
    """Spike test class. For testing only."""

    def __init__(self) -> None: ...
    def test_async_method(self) -> str: ...
    def test_crawl_simulation(self, url: str) -> Dict[str, Any]: ...
    def test_spider_simulation(self, url: str, count: int) -> List[str]: ...
    def is_healthy(self) -> bool: ...


__version__: str
__doc__: str
