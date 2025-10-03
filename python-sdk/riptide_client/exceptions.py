"""RipTide API exceptions."""


class RipTideError(Exception):
    """Base exception for RipTide client."""

    pass


class APIError(RipTideError):
    """API request failed."""

    pass


class ValidationError(RipTideError):
    """Request validation failed."""

    pass


class RateLimitError(RipTideError):
    """Rate limit exceeded."""

    pass


class TimeoutError(RipTideError):
    """Request timeout."""

    pass
