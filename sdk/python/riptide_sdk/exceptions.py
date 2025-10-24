"""
Exception classes for RipTide SDK

Provides a hierarchy of exceptions for different error scenarios.
"""

from typing import Optional, Dict, Any


class RipTideError(Exception):
    """Base exception for all RipTide SDK errors"""

    def __init__(self, message: str, status_code: Optional[int] = None,
                 response_data: Optional[Dict[str, Any]] = None):
        super().__init__(message)
        self.message = message
        self.status_code = status_code
        self.response_data = response_data or {}

    def __str__(self) -> str:
        if self.status_code:
            return f"[{self.status_code}] {self.message}"
        return self.message


class ValidationError(RipTideError):
    """Raised when request validation fails"""
    pass


class APIError(RipTideError):
    """Raised when the API returns an error response"""

    def __init__(self, message: str, status_code: int,
                 error_type: Optional[str] = None,
                 response_data: Optional[Dict[str, Any]] = None):
        super().__init__(message, status_code, response_data)
        self.error_type = error_type or "api_error"

    @property
    def is_retryable(self) -> bool:
        """Check if the error is retryable based on status code"""
        return self.status_code in (408, 429, 500, 502, 503, 504)


class NetworkError(RipTideError):
    """Raised when network communication fails"""
    pass


class TimeoutError(RipTideError):
    """Raised when a request times out"""
    pass


class ConfigError(RipTideError):
    """Raised when there's a configuration error"""
    pass


class StreamingError(RipTideError):
    """Raised when streaming operations fail"""
    pass
