"""RipTide API Client

Official Python client for the RipTide web crawling and content extraction API.

Example usage:
    >>> from riptide_client import RipTide
    >>> client = RipTide('http://localhost:8080')
    >>> result = client.crawl(['https://example.com'])
    >>> print(result['results'][0]['document']['title'])
"""

from .client import RipTide
from .exceptions import (
    RipTideError,
    APIError,
    ValidationError,
    RateLimitError,
    TimeoutError,
)
from .types import CrawlOptions, SearchOptions, SessionConfig

__version__ = "1.0.0"
__all__ = [
    "RipTide",
    "RipTideError",
    "APIError",
    "ValidationError",
    "RateLimitError",
    "TimeoutError",
    "CrawlOptions",
    "SearchOptions",
    "SessionConfig",
]
