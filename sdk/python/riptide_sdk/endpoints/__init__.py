"""
API endpoint modules

Organizes API endpoints by functionality:
- crawl: Batch crawl operations
- profiles: Domain profile management (Phase 10.4)
- engine: Engine selection API (Phase 10)
- streaming: Streaming operations (NDJSON, SSE, WebSocket)
"""

from .crawl import CrawlAPI
from .profiles import ProfilesAPI
from .engine import EngineSelectionAPI
from .streaming import StreamingAPI

__all__ = [
    "CrawlAPI",
    "ProfilesAPI",
    "EngineSelectionAPI",
    "StreamingAPI",
]
