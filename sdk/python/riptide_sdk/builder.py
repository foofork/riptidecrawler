"""
Fluent Builder Pattern for RipTide Client Configuration

Provides a chainable interface for configuring the RipTide client with
comprehensive validation and sensible defaults.

Example:
    >>> client = (RipTideClientBuilder()
    ...     .with_base_url("http://localhost:8080")
    ...     .with_api_key("your-api-key")
    ...     .with_timeout(60.0)
    ...     .with_max_connections(200)
    ...     .with_retry_config(max_retries=3, backoff_factor=2.0)
    ...     .build())
"""

from typing import Optional, Dict, Any
from dataclasses import dataclass, field


@dataclass
class RetryConfig:
    """Configuration for automatic retry behavior"""
    max_retries: int = 3
    backoff_factor: float = 2.0
    retry_on_status: tuple = (408, 429, 500, 502, 503, 504)
    max_backoff: float = 60.0

    def to_dict(self) -> Dict[str, Any]:
        return {
            "max_retries": self.max_retries,
            "backoff_factor": self.backoff_factor,
            "retry_on_status": self.retry_on_status,
            "max_backoff": self.max_backoff,
        }


class RipTideClientBuilder:
    """
    Fluent builder for creating RipTideClient instances

    Provides a chainable API for configuring all client options with
    validation and helpful error messages.

    Example:
        >>> from riptide_sdk import RipTideClientBuilder
        >>>
        >>> client = (RipTideClientBuilder()
        ...     .with_base_url("http://localhost:8080")
        ...     .with_api_key("sk_test_123")
        ...     .with_timeout(30.0)
        ...     .with_max_connections(100)
        ...     .with_retry_config(max_retries=3)
        ...     .with_user_agent("MyApp/1.0")
        ...     .build())
    """

    def __init__(self):
        """Initialize builder with default values"""
        self._base_url: str = "http://localhost:8080"
        self._api_key: Optional[str] = None
        self._timeout: float = 30.0
        self._max_connections: int = 100
        self._max_keepalive: int = 20
        self._retry_config: Optional[RetryConfig] = None
        self._custom_headers: Dict[str, str] = {}
        self._user_agent: Optional[str] = None
        self._verify_ssl: bool = True
        self._follow_redirects: bool = True
        self._extra_kwargs: Dict[str, Any] = {}

    def with_base_url(self, url: str) -> 'RipTideClientBuilder':
        """
        Set the base URL for the API

        Args:
            url: Base URL (e.g., "http://localhost:8080" or "https://api.riptide.com")

        Returns:
            Self for chaining

        Raises:
            ValueError: If URL is empty or invalid format
        """
        if not url:
            raise ValueError("Base URL cannot be empty")

        if not url.startswith(("http://", "https://")):
            raise ValueError(
                f"Invalid base URL: {url}\n"
                "URL must start with 'http://' or 'https://'\n"
                "Example: http://localhost:8080"
            )

        self._base_url = url.rstrip("/")
        return self

    def with_api_key(self, api_key: str) -> 'RipTideClientBuilder':
        """
        Set the API key for authentication

        Args:
            api_key: Your API key (Bearer token)

        Returns:
            Self for chaining

        Raises:
            ValueError: If API key is empty
        """
        if not api_key:
            raise ValueError("API key cannot be empty")

        self._api_key = api_key
        return self

    def with_timeout(self, timeout: float) -> 'RipTideClientBuilder':
        """
        Set request timeout in seconds

        Args:
            timeout: Timeout in seconds (must be > 0)

        Returns:
            Self for chaining

        Raises:
            ValueError: If timeout is invalid
        """
        if timeout <= 0:
            raise ValueError(
                f"Timeout must be positive, got: {timeout}\n"
                "Recommended values: 30.0 (default), 60.0 (slow networks), 120.0 (large files)"
            )

        if timeout > 300:
            import warnings
            warnings.warn(
                f"Timeout of {timeout}s is very high. "
                "Consider using streaming endpoints for long operations."
            )

        self._timeout = timeout
        return self

    def with_max_connections(self, max_connections: int) -> 'RipTideClientBuilder':
        """
        Set maximum concurrent connections

        Args:
            max_connections: Max concurrent connections (1-1000)

        Returns:
            Self for chaining

        Raises:
            ValueError: If value is out of range
        """
        if max_connections < 1:
            raise ValueError("max_connections must be at least 1")

        if max_connections > 1000:
            raise ValueError(
                f"max_connections too high: {max_connections}\n"
                "Maximum allowed: 1000\n"
                "Note: High values may cause resource exhaustion"
            )

        self._max_connections = max_connections
        return self

    def with_max_keepalive(self, max_keepalive: int) -> 'RipTideClientBuilder':
        """
        Set maximum keepalive connections

        Args:
            max_keepalive: Max keepalive connections

        Returns:
            Self for chaining
        """
        if max_keepalive < 1:
            raise ValueError("max_keepalive must be at least 1")

        self._max_keepalive = max_keepalive
        return self

    def with_retry_config(
        self,
        max_retries: int = 3,
        backoff_factor: float = 2.0,
        max_backoff: float = 60.0,
    ) -> 'RipTideClientBuilder':
        """
        Configure automatic retry behavior

        Args:
            max_retries: Maximum number of retry attempts (default: 3)
            backoff_factor: Exponential backoff factor (default: 2.0)
            max_backoff: Maximum backoff time in seconds (default: 60.0)

        Returns:
            Self for chaining

        Example:
            >>> builder.with_retry_config(max_retries=5, backoff_factor=1.5)
        """
        if max_retries < 0:
            raise ValueError("max_retries cannot be negative")

        if backoff_factor < 1.0:
            raise ValueError("backoff_factor must be >= 1.0")

        self._retry_config = RetryConfig(
            max_retries=max_retries,
            backoff_factor=backoff_factor,
            max_backoff=max_backoff,
        )
        return self

    def with_user_agent(self, user_agent: str) -> 'RipTideClientBuilder':
        """
        Set custom User-Agent header

        Args:
            user_agent: User-Agent string

        Returns:
            Self for chaining
        """
        self._user_agent = user_agent
        return self

    def with_custom_header(self, name: str, value: str) -> 'RipTideClientBuilder':
        """
        Add a custom HTTP header

        Args:
            name: Header name
            value: Header value

        Returns:
            Self for chaining
        """
        self._custom_headers[name] = value
        return self

    def with_ssl_verification(self, verify: bool) -> 'RipTideClientBuilder':
        """
        Enable or disable SSL certificate verification

        Args:
            verify: Whether to verify SSL certificates (default: True)

        Returns:
            Self for chaining

        Warning:
            Disabling SSL verification is insecure and should only be used
            for testing against self-signed certificates.
        """
        if not verify:
            import warnings
            warnings.warn(
                "SSL verification disabled! This is insecure and should only "
                "be used for testing with self-signed certificates.",
                UserWarning,
                stacklevel=2,
            )

        self._verify_ssl = verify
        return self

    def with_follow_redirects(self, follow: bool) -> 'RipTideClientBuilder':
        """
        Enable or disable following HTTP redirects

        Args:
            follow: Whether to follow redirects (default: True)

        Returns:
            Self for chaining
        """
        self._follow_redirects = follow
        return self

    def with_extra_kwargs(self, **kwargs) -> 'RipTideClientBuilder':
        """
        Add extra keyword arguments for httpx.AsyncClient

        Args:
            **kwargs: Additional arguments for AsyncClient

        Returns:
            Self for chaining
        """
        self._extra_kwargs.update(kwargs)
        return self

    def build(self):
        """
        Build the RipTideClient with configured options

        Returns:
            Configured RipTideClient instance

        Example:
            >>> client = builder.build()
            >>> async with client:
            ...     result = await client.crawl.batch(urls)
        """
        from .client import RipTideClient

        # Build kwargs for client
        kwargs = {
            "base_url": self._base_url,
            "api_key": self._api_key,
            "timeout": self._timeout,
            "max_connections": self._max_connections,
            **self._extra_kwargs,
        }

        # Add custom headers if any
        if self._custom_headers or self._user_agent:
            headers = kwargs.get("headers", {})
            if self._user_agent:
                headers["User-Agent"] = self._user_agent
            headers.update(self._custom_headers)
            kwargs["headers"] = headers

        # Add SSL and redirect settings
        if not self._verify_ssl:
            kwargs["verify"] = False

        if not self._follow_redirects:
            kwargs["follow_redirects"] = False

        # Note: Retry config would need httpx-retry or similar library
        # For now, we store it in the client for manual retry logic
        client = RipTideClient(**kwargs)

        # Attach retry config if provided
        if self._retry_config:
            client._retry_config = self._retry_config

        return client

    def __repr__(self) -> str:
        """String representation of builder state"""
        return (
            f"RipTideClientBuilder("
            f"base_url={self._base_url!r}, "
            f"api_key={'***' if self._api_key else None}, "
            f"timeout={self._timeout}, "
            f"max_connections={self._max_connections})"
        )
