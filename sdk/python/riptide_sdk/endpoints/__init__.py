"""
API endpoint modules

Organizes API endpoints by functionality:
- crawl: Batch crawl operations
- profiles: Domain profile management (Phase 10.4)
- engine: Engine selection API (Phase 10)
- streaming: Streaming operations (NDJSON, SSE, WebSocket)
- search: Search operations with multiple provider support
- extract: Content extraction from single URLs
- pdf: PDF processing and extraction
- sessions: Session management for browser contexts and cookies
"""

from .crawl import CrawlAPI
from .profiles import ProfilesAPI
from .engine import EngineSelectionAPI
from .streaming import StreamingAPI
from .search import SearchAPI
from .extract import ExtractAPI
from .pdf import PdfAPI
from .sessions import SessionsAPI
from .spider import SpiderAPI
from .workers import WorkersAPI
from .browser import BrowserAPI

__all__ = [
    "CrawlAPI",
    "ProfilesAPI",
    "EngineSelectionAPI",
    "StreamingAPI",
    "SearchAPI",
    "ExtractAPI",
    "SessionsAPI",
    "SpiderAPI",
    "PdfAPI",
    "WorkersAPI",
    "BrowserAPI",
]
