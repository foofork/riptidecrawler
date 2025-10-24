"""
Domain Profiles API endpoint implementation (Phase 10.4)

Provides domain profile management for warm-start caching and
engine preference optimization.
"""

from typing import List, Optional, Dict, Any
import httpx

from ..models import DomainProfile, ProfileStats, ProfileConfig, ProfileMetadata
from ..exceptions import APIError, ValidationError


class ProfilesAPI:
    """API for domain profile management (Phase 10.4)"""

    def __init__(self, client: httpx.AsyncClient, base_url: str):
        """
        Initialize ProfilesAPI

        Args:
            client: Async HTTP client
            base_url: Base URL for the API
        """
        self.client = client
        self.base_url = base_url

    async def create(
        self,
        domain: str,
        config: Optional[ProfileConfig] = None,
        metadata: Optional[ProfileMetadata] = None,
    ) -> DomainProfile:
        """
        Create a new domain profile

        Args:
            domain: Domain name (e.g., "example.com")
            config: Optional profile configuration
            metadata: Optional profile metadata

        Returns:
            Created DomainProfile

        Raises:
            ValidationError: If domain is invalid
            APIError: If the API returns an error

        Example:
            >>> from riptide_sdk.models import ProfileConfig, StealthLevel
            >>> config = ProfileConfig(stealth_level=StealthLevel.HIGH)
            >>> profile = await client.profiles.create(
            ...     "example.com",
            ...     config=config
            ... )
        """
        if not domain:
            raise ValidationError("Domain cannot be empty")

        body: Dict[str, Any] = {"domain": domain}
        if config:
            body["config"] = config.to_dict()
        if metadata:
            body["metadata"] = metadata.to_dict()

        response = await self.client.post(
            f"{self.base_url}/api/v1/profiles",
            json=body,
        )

        if response.status_code not in (200, 201):
            error_data = response.json() if response.text else {}
            raise APIError(
                message=error_data.get("error", {}).get("message", "Profile creation failed"),
                status_code=response.status_code,
                response_data=error_data,
            )

        return DomainProfile.from_dict(response.json())

    async def get(self, domain: str) -> DomainProfile:
        """
        Get a domain profile

        Args:
            domain: Domain name

        Returns:
            DomainProfile

        Raises:
            APIError: If profile not found or request fails
        """
        response = await self.client.get(
            f"{self.base_url}/api/v1/profiles/{domain}"
        )

        if response.status_code != 200:
            error_data = response.json() if response.text else {}
            raise APIError(
                message=error_data.get("error", {}).get("message", "Profile not found"),
                status_code=response.status_code,
                response_data=error_data,
            )

        return DomainProfile.from_dict(response.json())

    async def update(
        self,
        domain: str,
        config: Optional[ProfileConfig] = None,
        metadata: Optional[ProfileMetadata] = None,
    ) -> DomainProfile:
        """
        Update a domain profile

        Args:
            domain: Domain name
            config: Optional updated configuration
            metadata: Optional updated metadata

        Returns:
            Updated DomainProfile
        """
        body: Dict[str, Any] = {}
        if config:
            body["config"] = config.to_dict()
        if metadata:
            body["metadata"] = metadata.to_dict()

        if not body:
            raise ValidationError("No updates provided")

        response = await self.client.put(
            f"{self.base_url}/api/v1/profiles/{domain}",
            json=body,
        )

        if response.status_code != 200:
            error_data = response.json() if response.text else {}
            raise APIError(
                message=error_data.get("error", {}).get("message", "Profile update failed"),
                status_code=response.status_code,
                response_data=error_data,
            )

        return DomainProfile.from_dict(response.json())

    async def delete(self, domain: str) -> None:
        """
        Delete a domain profile

        Args:
            domain: Domain name

        Raises:
            APIError: If deletion fails
        """
        response = await self.client.delete(
            f"{self.base_url}/api/v1/profiles/{domain}"
        )

        if response.status_code not in (200, 204):
            error_data = response.json() if response.text else {}
            raise APIError(
                message=error_data.get("error", {}).get("message", "Profile deletion failed"),
                status_code=response.status_code,
                response_data=error_data,
            )

    async def list(self, filter_query: Optional[str] = None) -> List[DomainProfile]:
        """
        List all domain profiles

        Args:
            filter_query: Optional filter string

        Returns:
            List of DomainProfile objects
        """
        params = {}
        if filter_query:
            params["filter"] = filter_query

        response = await self.client.get(
            f"{self.base_url}/api/v1/profiles",
            params=params,
        )

        if response.status_code != 200:
            error_data = response.json() if response.text else {}
            raise APIError(
                message=error_data.get("error", {}).get("message", "Failed to list profiles"),
                status_code=response.status_code,
                response_data=error_data,
            )

        data = response.json()
        return [DomainProfile.from_dict(p) for p in data.get("profiles", [])]

    async def get_stats(self, domain: str) -> ProfileStats:
        """
        Get usage statistics for a domain profile

        Args:
            domain: Domain name

        Returns:
            ProfileStats with usage metrics

        Example:
            >>> stats = await client.profiles.get_stats("example.com")
            >>> print(f"Cache hit rate: {stats.cache_hits / stats.total_requests:.2%}")
        """
        response = await self.client.get(
            f"{self.base_url}/api/v1/profiles/{domain}/stats"
        )

        if response.status_code != 200:
            error_data = response.json() if response.text else {}
            raise APIError(
                message=error_data.get("error", {}).get("message", "Failed to get stats"),
                status_code=response.status_code,
                response_data=error_data,
            )

        return ProfileStats.from_dict(response.json())

    async def get_metrics(self) -> Dict[str, Any]:
        """
        Get aggregated caching metrics across all profiles

        Returns:
            Dictionary with aggregated metrics
        """
        response = await self.client.get(
            f"{self.base_url}/api/v1/profiles/metrics"
        )

        if response.status_code != 200:
            error_data = response.json() if response.text else {}
            raise APIError(
                message=error_data.get("error", {}).get("message", "Failed to get metrics"),
                status_code=response.status_code,
                response_data=error_data,
            )

        return response.json()

    async def batch_create(
        self,
        profiles: List[DomainProfile],
    ) -> Dict[str, Any]:
        """
        Create multiple profiles in a batch operation

        Args:
            profiles: List of profiles to create

        Returns:
            Dictionary with created and failed profile lists

        Example:
            >>> profiles = [
            ...     DomainProfile(domain="example.com"),
            ...     DomainProfile(domain="example.org"),
            ... ]
            >>> result = await client.profiles.batch_create(profiles)
            >>> print(f"Created: {len(result['created'])}")
        """
        body = {
            "profiles": [p.to_dict() for p in profiles]
        }

        response = await self.client.post(
            f"{self.base_url}/api/v1/profiles/batch",
            json=body,
        )

        if response.status_code not in (200, 201):
            error_data = response.json() if response.text else {}
            raise APIError(
                message=error_data.get("error", {}).get("message", "Batch create failed"),
                status_code=response.status_code,
                response_data=error_data,
            )

        return response.json()

    async def search(self, query: str) -> List[DomainProfile]:
        """
        Search profiles by query string

        Args:
            query: Search query

        Returns:
            List of matching DomainProfile objects
        """
        response = await self.client.get(
            f"{self.base_url}/api/v1/profiles/search",
            params={"query": query},
        )

        if response.status_code != 200:
            error_data = response.json() if response.text else {}
            raise APIError(
                message=error_data.get("error", {}).get("message", "Search failed"),
                status_code=response.status_code,
                response_data=error_data,
            )

        data = response.json()
        return [DomainProfile.from_dict(p) for p in data.get("profiles", [])]

    async def warm_cache(self, domain: str, url: str) -> Dict[str, Any]:
        """
        Warm the engine cache for a specific domain

        Args:
            domain: Domain name
            url: URL to use for cache warming

        Returns:
            Dictionary with warm-cache result

        Example:
            >>> result = await client.profiles.warm_cache(
            ...     "example.com",
            ...     "https://example.com/page"
            ... )
        """
        response = await self.client.post(
            f"{self.base_url}/api/v1/profiles/{domain}/warm",
            json={"url": url},
        )

        if response.status_code != 200:
            error_data = response.json() if response.text else {}
            raise APIError(
                message=error_data.get("error", {}).get("message", "Cache warming failed"),
                status_code=response.status_code,
                response_data=error_data,
            )

        return response.json()

    async def clear_all_caches(self) -> Dict[str, Any]:
        """
        Clear all cached engines across all profiles

        Returns:
            Dictionary with operation result

        Warning:
            This will clear ALL cached engine preferences. Use with caution.
        """
        response = await self.client.delete(
            f"{self.base_url}/api/v1/profiles/clear"
        )

        if response.status_code not in (200, 204):
            error_data = response.json() if response.text else {}
            raise APIError(
                message=error_data.get("error", {}).get("message", "Cache clear failed"),
                status_code=response.status_code,
                response_data=error_data,
            )

        return response.json() if response.text else {"status": "cleared"}
