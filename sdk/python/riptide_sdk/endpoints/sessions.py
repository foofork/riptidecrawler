"""
Session Management API endpoint implementation

Provides session management for browser contexts, cookies, and user data
persistence in headless browser operations.
"""

from typing import List, Optional, Dict, Any
import httpx

from ..models import (
    Session,
    SessionConfig,
    SessionStats,
    Cookie,
    SetCookieRequest,
)
from ..exceptions import APIError, ValidationError


class SessionsAPI:
    """
    API for session management

    Sessions provide persistent browser contexts with cookie management,
    user data directories, and automatic cleanup.
    """

    def __init__(self, client: httpx.AsyncClient, base_url: str):
        """
        Initialize SessionsAPI

        Args:
            client: Async HTTP client
            base_url: Base URL for the API
        """
        self.client = client
        self.base_url = base_url

    async def create(self, config: Optional[SessionConfig] = None) -> Session:
        """
        Create a new session

        Creates a new browser session with isolated user data directory
        and cookie storage.

        Args:
            config: Optional session configuration

        Returns:
            Created Session object

        Raises:
            APIError: If the API returns an error

        Example:
            >>> from riptide_sdk.models import SessionConfig
            >>> config = SessionConfig(ttl_seconds=3600)
            >>> session = await client.sessions.create(config=config)
            >>> print(f"Session ID: {session.session_id}")
            >>> print(f"Expires at: {session.expires_at}")
        """
        body: Dict[str, Any] = {}
        if config:
            body = config.to_dict()

        response = await self.client.post(
            f"{self.base_url}/api/v1/sessions",
            json=body if body else None,
        )

        if response.status_code not in (200, 201):
            error_data = response.json() if response.text else {}
            raise APIError(
                message=error_data.get("error", {}).get("message", "Session creation failed"),
                status_code=response.status_code,
                response_data=error_data,
            )

        return Session.from_dict(response.json())

    async def list(
        self,
        include_expired: bool = False,
        limit: Optional[int] = None,
    ) -> List[str]:
        """
        List all sessions

        Args:
            include_expired: Include expired sessions in results (default: False)
            limit: Maximum number of sessions to return (default: 100)

        Returns:
            List of session IDs

        Raises:
            APIError: If the API returns an error

        Example:
            >>> sessions = await client.sessions.list(limit=50)
            >>> print(f"Active sessions: {len(sessions)}")
            >>> for session_id in sessions:
            ...     print(f"  - {session_id}")
        """
        params: Dict[str, Any] = {}
        if include_expired:
            params["include_expired"] = "true"
        if limit is not None:
            params["limit"] = str(limit)

        response = await self.client.get(
            f"{self.base_url}/api/v1/sessions",
            params=params,
        )

        if response.status_code != 200:
            error_data = response.json() if response.text else {}
            raise APIError(
                message=error_data.get("error", {}).get("message", "Failed to list sessions"),
                status_code=response.status_code,
                response_data=error_data,
            )

        return response.json()

    async def get(self, session_id: str) -> Session:
        """
        Get session information

        Retrieves detailed information about a session including
        user data directory, cookie count, and expiry.

        Args:
            session_id: Session ID

        Returns:
            Session object with detailed information

        Raises:
            ValidationError: If session_id is empty
            APIError: If session not found or request fails

        Example:
            >>> session = await client.sessions.get("session_123")
            >>> print(f"Created: {session.created_at}")
            >>> print(f"Cookies: {session.cookie_count}")
            >>> print(f"Domains: {session.total_domains}")
        """
        if not session_id:
            raise ValidationError("Session ID cannot be empty")

        response = await self.client.get(
            f"{self.base_url}/api/v1/sessions/{session_id}"
        )

        if response.status_code != 200:
            error_data = response.json() if response.text else {}
            raise APIError(
                message=error_data.get("error", {}).get("message", "Session not found"),
                status_code=response.status_code,
                response_data=error_data,
            )

        return Session.from_dict(response.json())

    async def delete(self, session_id: str) -> None:
        """
        Delete a session

        Removes a session and cleans up associated resources including
        user data directory and cookies.

        Args:
            session_id: Session ID to delete

        Raises:
            ValidationError: If session_id is empty
            APIError: If deletion fails

        Example:
            >>> await client.sessions.delete("session_123")
            >>> print("Session deleted successfully")
        """
        if not session_id:
            raise ValidationError("Session ID cannot be empty")

        response = await self.client.delete(
            f"{self.base_url}/api/v1/sessions/{session_id}"
        )

        if response.status_code != 204:
            error_data = response.json() if response.text else {}
            raise APIError(
                message=error_data.get("error", {}).get("message", "Session deletion failed"),
                status_code=response.status_code,
                response_data=error_data,
            )

    async def extend(self, session_id: str, additional_seconds: int) -> None:
        """
        Extend session expiry time

        Adds additional time to a session's TTL to prevent automatic cleanup.

        Args:
            session_id: Session ID to extend
            additional_seconds: Additional time in seconds

        Raises:
            ValidationError: If session_id is empty or additional_seconds is invalid
            APIError: If extension fails

        Example:
            >>> # Extend session by 1 hour
            >>> await client.sessions.extend("session_123", 3600)
            >>> print("Session extended")
        """
        if not session_id:
            raise ValidationError("Session ID cannot be empty")
        if additional_seconds <= 0:
            raise ValidationError("additional_seconds must be positive")

        response = await self.client.post(
            f"{self.base_url}/api/v1/sessions/{session_id}/extend",
            json={"additional_seconds": additional_seconds},
        )

        if response.status_code != 200:
            error_data = response.json() if response.text else {}
            raise APIError(
                message=error_data.get("error", {}).get("message", "Session extension failed"),
                status_code=response.status_code,
                response_data=error_data,
            )

    async def set_cookie(
        self,
        session_id: str,
        cookie: SetCookieRequest,
    ) -> None:
        """
        Set a cookie for a session

        Adds or updates a cookie in the session's cookie store.

        Args:
            session_id: Session ID
            cookie: Cookie to set

        Raises:
            ValidationError: If session_id is empty
            APIError: If setting cookie fails

        Example:
            >>> from riptide_sdk.models import SetCookieRequest
            >>> cookie = SetCookieRequest(
            ...     domain="example.com",
            ...     name="session_token",
            ...     value="abc123xyz",
            ...     path="/",
            ...     expires_in_seconds=3600,
            ...     secure=True,
            ...     http_only=True,
            ... )
            >>> await client.sessions.set_cookie("session_123", cookie)
        """
        if not session_id:
            raise ValidationError("Session ID cannot be empty")

        response = await self.client.post(
            f"{self.base_url}/api/v1/sessions/{session_id}/cookies",
            json=cookie.to_dict(),
        )

        if response.status_code not in (200, 201):
            error_data = response.json() if response.text else {}
            raise APIError(
                message=error_data.get("error", {}).get("message", "Failed to set cookie"),
                status_code=response.status_code,
                response_data=error_data,
            )

    async def get_cookies(self, session_id: str, domain: str) -> List[Cookie]:
        """
        Get all cookies for a domain

        Retrieves all cookies associated with a specific domain in the session.

        Args:
            session_id: Session ID
            domain: Domain name (e.g., "example.com")

        Returns:
            List of Cookie objects

        Raises:
            ValidationError: If session_id or domain is empty
            APIError: If retrieval fails

        Example:
            >>> cookies = await client.sessions.get_cookies(
            ...     "session_123",
            ...     "example.com"
            ... )
            >>> for cookie in cookies:
            ...     print(f"{cookie.name}: {cookie.value}")
        """
        if not session_id:
            raise ValidationError("Session ID cannot be empty")
        if not domain:
            raise ValidationError("Domain cannot be empty")

        response = await self.client.get(
            f"{self.base_url}/api/v1/sessions/{session_id}/cookies/{domain}"
        )

        if response.status_code != 200:
            error_data = response.json() if response.text else {}
            raise APIError(
                message=error_data.get("error", {}).get("message", "Failed to get cookies"),
                status_code=response.status_code,
                response_data=error_data,
            )

        return [Cookie.from_dict(c) for c in response.json()]

    async def get_stats(self) -> SessionStats:
        """
        Get session statistics

        Retrieves global statistics about all sessions including
        total count and cleanup metrics.

        Returns:
            SessionStats object

        Raises:
            APIError: If retrieval fails

        Example:
            >>> stats = await client.sessions.get_stats()
            >>> print(f"Total sessions: {stats.total_sessions}")
            >>> print(f"Expired cleaned: {stats.expired_sessions_cleaned}")
        """
        response = await self.client.get(
            f"{self.base_url}/api/v1/sessions/stats"
        )

        if response.status_code != 200:
            error_data = response.json() if response.text else {}
            raise APIError(
                message=error_data.get("error", {}).get("message", "Failed to get stats"),
                status_code=response.status_code,
                response_data=error_data,
            )

        return SessionStats.from_dict(response.json())
