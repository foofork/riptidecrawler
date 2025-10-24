"""
Engine Selection API endpoint implementation (Phase 10)

Provides engine selection and analysis capabilities for optimizing
crawl performance based on HTML characteristics.
"""

from typing import Dict, Any
import httpx

from ..models import EngineDecision, EngineStats
from ..exceptions import APIError, ValidationError


class EngineSelectionAPI:
    """API for engine selection operations (Phase 10)"""

    def __init__(self, client: httpx.AsyncClient, base_url: str):
        """
        Initialize EngineSelectionAPI

        Args:
            client: Async HTTP client
            base_url: Base URL for the API
        """
        self.client = client
        self.base_url = base_url

    async def analyze(self, html: str, url: str) -> EngineDecision:
        """
        Analyze HTML and get engine recommendation

        Args:
            html: HTML content to analyze
            url: URL of the page

        Returns:
            EngineDecision with recommendation and confidence

        Raises:
            ValidationError: If inputs are invalid
            APIError: If the API returns an error

        Example:
            >>> decision = await client.engine.analyze(
            ...     html="<html>...</html>",
            ...     url="https://example.com"
            ... )
            >>> print(f"Recommended: {decision.engine} ({decision.confidence:.2%})")
        """
        if not html:
            raise ValidationError("HTML content cannot be empty")
        if not url:
            raise ValidationError("URL cannot be empty")

        response = await self.client.post(
            f"{self.base_url}/api/v1/engine/analyze",
            json={"html": html, "url": url},
        )

        if response.status_code != 200:
            error_data = response.json() if response.text else {}
            raise APIError(
                message=error_data.get("error", {}).get("message", "Engine analysis failed"),
                status_code=response.status_code,
                response_data=error_data,
            )

        return EngineDecision.from_dict(response.json())

    async def decide(
        self,
        html: str,
        url: str,
        flags: Dict[str, bool],
    ) -> EngineDecision:
        """
        Make engine decision with custom flags

        Args:
            html: HTML content
            url: URL of the page
            flags: Decision flags (e.g., {"has_spa": true})

        Returns:
            EngineDecision with final decision

        Example:
            >>> decision = await client.engine.decide(
            ...     html="<html>...</html>",
            ...     url="https://example.com",
            ...     flags={"has_spa": True, "requires_js": False}
            ... )
        """
        if not html:
            raise ValidationError("HTML content cannot be empty")
        if not url:
            raise ValidationError("URL cannot be empty")

        response = await self.client.post(
            f"{self.base_url}/api/v1/engine/decide",
            json={
                "html": html,
                "url": url,
                "flags": flags,
            },
        )

        if response.status_code != 200:
            error_data = response.json() if response.text else {}
            raise APIError(
                message=error_data.get("error", {}).get("message", "Engine decision failed"),
                status_code=response.status_code,
                response_data=error_data,
            )

        return EngineDecision.from_dict(response.json())

    async def get_stats(self) -> EngineStats:
        """
        Get engine usage statistics

        Returns:
            EngineStats with usage metrics

        Example:
            >>> stats = await client.engine.get_stats()
            >>> print(f"Total decisions: {stats.total_decisions}")
            >>> print(f"Raw: {stats.raw_count}, Headless: {stats.headless_count}")
        """
        response = await self.client.get(
            f"{self.base_url}/api/v1/engine/stats"
        )

        if response.status_code != 200:
            error_data = response.json() if response.text else {}
            raise APIError(
                message=error_data.get("error", {}).get("message", "Failed to get stats"),
                status_code=response.status_code,
                response_data=error_data,
            )

        return EngineStats.from_dict(response.json())

    async def toggle_probe_first(self, enabled: bool) -> Dict[str, Any]:
        """
        Toggle probe-first mode for engine selection

        Args:
            enabled: Whether to enable probe-first mode

        Returns:
            Dictionary with updated configuration

        Example:
            >>> result = await client.engine.toggle_probe_first(True)
            >>> print(f"Probe-first enabled: {result['enabled']}")
        """
        response = await self.client.put(
            f"{self.base_url}/api/v1/engine/probe-first",
            json={"enabled": enabled},
        )

        if response.status_code != 200:
            error_data = response.json() if response.text else {}
            raise APIError(
                message=error_data.get("error", {}).get("message", "Failed to toggle probe-first"),
                status_code=response.status_code,
                response_data=error_data,
            )

        return response.json()
