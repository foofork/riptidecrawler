"""
Browser Automation API endpoint implementation

Provides headless browser automation for session management, page actions,
and browser pool monitoring.
"""

from typing import Optional, Dict, Any
import httpx

from ..models import (
    BrowserSession,
    BrowserSessionConfig,
    BrowserAction,
    BrowserPoolStatus,
    BrowserActionResult,
)
from ..exceptions import APIError, ValidationError


class BrowserAPI:
    """
    API for browser automation and session management

    Provides comprehensive browser automation including session creation,
    action execution (navigate, click, type, screenshot, etc.), and browser
    pool status monitoring.
    """

    def __init__(self, client: httpx.AsyncClient, base_url: str):
        """
        Initialize BrowserAPI

        Args:
            client: Async HTTP client
            base_url: Base URL for the API
        """
        self.client = client
        self.base_url = base_url

    async def create_session(
        self,
        config: Optional[BrowserSessionConfig] = None,
    ) -> BrowserSession:
        """
        Create a new browser session

        Creates a new headless browser session from the pool with optional
        stealth configuration and initial URL navigation.

        Args:
            config: Optional browser session configuration

        Returns:
            BrowserSession object with session ID and pool statistics

        Raises:
            APIError: If the API returns an error
            ValidationError: If configuration is invalid

        Example:
            >>> from riptide_sdk.models import BrowserSessionConfig
            >>> config = BrowserSessionConfig(
            ...     stealth_preset="medium",
            ...     initial_url="https://example.com",
            ...     timeout_secs=600
            ... )
            >>> session = await client.browser.create_session(config=config)
            >>> print(f"Session ID: {session.session_id}")
            >>> print(f"Pool utilization: {session.pool_stats.utilization_percent:.1f}%")
        """
        body: Dict[str, Any] = {}
        if config:
            body = config.to_dict()

        response = await self.client.post(
            f"{self.base_url}/api/v1/browser/session",
            json=body if body else None,
        )

        if response.status_code not in (200, 201):
            error_data = response.json() if response.text else {}
            raise APIError(
                message=error_data.get("error", {}).get(
                    "message", "Browser session creation failed"
                ),
                status_code=response.status_code,
                response_data=error_data,
            )

        return BrowserSession.from_dict(response.json())

    async def execute_action(
        self,
        session_id: str,
        action: BrowserAction,
    ) -> BrowserActionResult:
        """
        Execute a browser action

        Executes various actions on an existing browser session including
        navigation, element interaction, JavaScript execution, screenshots,
        and more.

        Args:
            session_id: Browser session ID
            action: Browser action to execute

        Returns:
            BrowserActionResult with success status and result data

        Raises:
            ValidationError: If session_id is empty or action is invalid
            APIError: If action execution fails

        Example:
            >>> from riptide_sdk.models import BrowserAction
            >>>
            >>> # Navigate to URL
            >>> action = BrowserAction.navigate(
            ...     session_id="session_123",
            ...     url="https://example.com",
            ...     wait_for_load=True
            ... )
            >>> result = await client.browser.execute_action("session_123", action)
            >>> print(f"Success: {result.success}")
            >>> print(f"Duration: {result.duration_ms}ms")
            >>>
            >>> # Click element
            >>> action = BrowserAction.click(
            ...     session_id="session_123",
            ...     selector="#submit-button"
            ... )
            >>> result = await client.browser.execute_action("session_123", action)
            >>>
            >>> # Take screenshot
            >>> action = BrowserAction.screenshot(
            ...     session_id="session_123",
            ...     full_page=True
            ... )
            >>> result = await client.browser.execute_action("session_123", action)
            >>> screenshot_data = result.result["screenshot_base64"]
        """
        if not session_id:
            raise ValidationError("Session ID cannot be empty")

        # Build request body from action
        body = action.to_dict()
        body["session_id"] = session_id

        response = await self.client.post(
            f"{self.base_url}/api/v1/browser/action",
            json=body,
        )

        if response.status_code != 200:
            error_data = response.json() if response.text else {}
            raise APIError(
                message=error_data.get("error", {}).get(
                    "message", "Browser action execution failed"
                ),
                status_code=response.status_code,
                response_data=error_data,
            )

        return BrowserActionResult.from_dict(response.json())

    async def get_pool_status(self) -> BrowserPoolStatus:
        """
        Get browser pool status

        Retrieves detailed statistics about the browser pool including
        utilization, performance metrics, and health status.

        Returns:
            BrowserPoolStatus with pool statistics and health information

        Raises:
            APIError: If the API returns an error

        Example:
            >>> status = await client.browser.get_pool_status()
            >>> print(f"Available browsers: {status.stats.available}")
            >>> print(f"In use: {status.stats.in_use}")
            >>> print(f"Utilization: {status.stats.utilization_percent:.1f}%")
            >>> print(f"Health: {status.health}")
            >>> print(f"Total requests: {status.launcher_stats.total_requests}")
            >>> print(f"Success rate: {status.launcher_stats.success_rate:.1%}")
        """
        response = await self.client.get(
            f"{self.base_url}/api/v1/browser/pool/status"
        )

        if response.status_code != 200:
            error_data = response.json() if response.text else {}
            raise APIError(
                message=error_data.get("error", {}).get(
                    "message", "Failed to get pool status"
                ),
                status_code=response.status_code,
                response_data=error_data,
            )

        return BrowserPoolStatus.from_dict(response.json())

    async def close_session(self, session_id: str) -> None:
        """
        Close a browser session

        Closes an existing browser session and returns the browser instance
        back to the pool for reuse.

        Args:
            session_id: Session ID to close

        Raises:
            ValidationError: If session_id is empty
            APIError: If session closure fails

        Example:
            >>> await client.browser.close_session("session_123")
            >>> print("Session closed successfully")
        """
        if not session_id:
            raise ValidationError("Session ID cannot be empty")

        response = await self.client.delete(
            f"{self.base_url}/api/v1/browser/session/{session_id}"
        )

        if response.status_code != 204:
            error_data = response.json() if response.text else {}
            raise APIError(
                message=error_data.get("error", {}).get(
                    "message", "Session closure failed"
                ),
                status_code=response.status_code,
                response_data=error_data,
            )

    # Convenience methods for common actions

    async def navigate(
        self,
        session_id: str,
        url: str,
        wait_for_load: bool = True,
    ) -> BrowserActionResult:
        """
        Navigate to a URL (convenience method)

        Args:
            session_id: Browser session ID
            url: URL to navigate to
            wait_for_load: Wait for page load completion (default: True)

        Returns:
            BrowserActionResult with navigation result

        Example:
            >>> result = await client.browser.navigate(
            ...     "session_123",
            ...     "https://example.com"
            ... )
            >>> print(f"Loaded: {result.result['loaded']}")
        """
        from ..models import BrowserAction

        action = BrowserAction.navigate(
            session_id=session_id,
            url=url,
            wait_for_load=wait_for_load,
        )
        return await self.execute_action(session_id, action)

    async def click(
        self,
        session_id: str,
        selector: str,
    ) -> BrowserActionResult:
        """
        Click an element (convenience method)

        Args:
            session_id: Browser session ID
            selector: CSS selector for element to click

        Returns:
            BrowserActionResult with click result

        Example:
            >>> result = await client.browser.click(
            ...     "session_123",
            ...     "#submit-button"
            ... )
        """
        from ..models import BrowserAction

        action = BrowserAction.click(session_id=session_id, selector=selector)
        return await self.execute_action(session_id, action)

    async def type_text(
        self,
        session_id: str,
        selector: str,
        text: str,
    ) -> BrowserActionResult:
        """
        Type text into an element (convenience method)

        Args:
            session_id: Browser session ID
            selector: CSS selector for input element
            text: Text to type

        Returns:
            BrowserActionResult with typing result

        Example:
            >>> result = await client.browser.type_text(
            ...     "session_123",
            ...     "#search-input",
            ...     "Hello World"
            ... )
        """
        from ..models import BrowserAction

        action = BrowserAction.type_text(
            session_id=session_id,
            selector=selector,
            text=text,
        )
        return await self.execute_action(session_id, action)

    async def screenshot(
        self,
        session_id: str,
        full_page: bool = False,
    ) -> BrowserActionResult:
        """
        Take a screenshot (convenience method)

        Args:
            session_id: Browser session ID
            full_page: Capture full page screenshot (default: False)

        Returns:
            BrowserActionResult with screenshot data (base64 encoded)

        Example:
            >>> result = await client.browser.screenshot(
            ...     "session_123",
            ...     full_page=True
            ... )
            >>> screenshot_data = result.result["screenshot_base64"]
            >>> size = result.result["size_bytes"]
            >>> print(f"Screenshot captured: {size:,} bytes")
        """
        from ..models import BrowserAction

        action = BrowserAction.screenshot(
            session_id=session_id,
            full_page=full_page,
        )
        return await self.execute_action(session_id, action)

    async def execute_script(
        self,
        session_id: str,
        script: str,
        timeout_ms: Optional[int] = None,
    ) -> BrowserActionResult:
        """
        Execute JavaScript code (convenience method)

        Args:
            session_id: Browser session ID
            script: JavaScript code to execute
            timeout_ms: Script execution timeout in milliseconds

        Returns:
            BrowserActionResult with script execution result

        Example:
            >>> result = await client.browser.execute_script(
            ...     "session_123",
            ...     "return document.title;"
            ... )
            >>> title = result.result["return_value"]
        """
        from ..models import BrowserAction

        action = BrowserAction.execute_script(
            session_id=session_id,
            script=script,
            timeout_ms=timeout_ms,
        )
        return await self.execute_action(session_id, action)

    async def get_content(self, session_id: str) -> BrowserActionResult:
        """
        Get page HTML content (convenience method)

        Args:
            session_id: Browser session ID

        Returns:
            BrowserActionResult with HTML content

        Example:
            >>> result = await client.browser.get_content("session_123")
            >>> html = result.result["html"]
            >>> length = result.result["length"]
        """
        from ..models import BrowserAction

        action = BrowserAction.get_content(session_id=session_id)
        return await self.execute_action(session_id, action)

    async def wait_for_element(
        self,
        session_id: str,
        selector: str,
        timeout_ms: int = 5000,
    ) -> BrowserActionResult:
        """
        Wait for an element to appear (convenience method)

        Args:
            session_id: Browser session ID
            selector: CSS selector to wait for
            timeout_ms: Wait timeout in milliseconds (default: 5000)

        Returns:
            BrowserActionResult indicating if element was found

        Example:
            >>> result = await client.browser.wait_for_element(
            ...     "session_123",
            ...     "#dynamic-content",
            ...     timeout_ms=10000
            ... )
            >>> found = result.result["found"]
        """
        from ..models import BrowserAction

        action = BrowserAction.wait_for_element(
            session_id=session_id,
            selector=selector,
            timeout_ms=timeout_ms,
        )
        return await self.execute_action(session_id, action)

    async def render_pdf(
        self,
        session_id: str,
        landscape: bool = False,
        print_background: bool = False,
    ) -> BrowserActionResult:
        """
        Render page to PDF (convenience method)

        Args:
            session_id: Browser session ID
            landscape: Use landscape orientation (default: False)
            print_background: Include background graphics (default: False)

        Returns:
            BrowserActionResult with PDF data (base64 encoded)

        Example:
            >>> result = await client.browser.render_pdf(
            ...     "session_123",
            ...     landscape=True,
            ...     print_background=True
            ... )
            >>> pdf_data = result.result["pdf_base64"]
        """
        from ..models import BrowserAction

        action = BrowserAction.render_pdf(
            session_id=session_id,
            landscape=landscape,
            print_background=print_background,
        )
        return await self.execute_action(session_id, action)
