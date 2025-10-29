"""
Exception classes for RipTide SDK

Provides a hierarchy of exceptions for different error scenarios.
"""

from typing import Optional, Dict, Any


class RipTideError(Exception):
    """Base exception for all RipTide SDK errors"""

    def __init__(self, message: str, status_code: Optional[int] = None,
                 response_data: Optional[Dict[str, Any]] = None,
                 suggestion: Optional[str] = None,
                 docs_url: Optional[str] = None):
        super().__init__(message)
        self.message = message
        self.status_code = status_code
        self.response_data = response_data or {}
        self.suggestion = suggestion
        self.docs_url = docs_url or "https://github.com/yourusername/eventmesh/tree/main/sdk/python"

    def __str__(self) -> str:
        parts = []
        if self.status_code:
            parts.append(f"[{self.status_code}]")
        parts.append(self.message)

        if self.suggestion:
            parts.append(f"\n\n💡 Suggestion: {self.suggestion}")

        if self.docs_url:
            parts.append(f"\n📚 Documentation: {self.docs_url}")

        return " ".join(parts) if not self.suggestion else "".join(parts)


class ValidationError(RipTideError):
    """Raised when request validation fails"""

    def __init__(self, message: str, field: Optional[str] = None, **kwargs):
        suggestion = self._get_validation_suggestion(message, field)
        super().__init__(message, suggestion=suggestion, **kwargs)
        self.field = field

    @staticmethod
    def _get_validation_suggestion(message: str, field: Optional[str]) -> str:
        """Provide actionable suggestions for validation errors"""
        if "URL" in message or "url" in message:
            return "Ensure URLs start with 'http://' or 'https://'. Example: https://example.com"
        elif "empty" in message.lower():
            return f"The '{field}' field is required and cannot be empty."
        elif "Maximum" in message:
            return "Consider splitting your request into smaller batches."
        return "Check the API documentation for valid parameter formats."


class APIError(RipTideError):
    """Raised when the API returns an error response"""

    def __init__(self, message: str, status_code: int,
                 error_type: Optional[str] = None,
                 response_data: Optional[Dict[str, Any]] = None):
        suggestion = self._get_api_error_suggestion(status_code, error_type)
        super().__init__(message, status_code, response_data, suggestion=suggestion)
        self.error_type = error_type or "api_error"

    @property
    def is_retryable(self) -> bool:
        """Check if the error is retryable based on status code"""
        return self.status_code in (408, 429, 500, 502, 503, 504)

    @staticmethod
    def _get_api_error_suggestion(status_code: int, error_type: Optional[str]) -> str:
        """Provide actionable suggestions based on status code"""
        suggestions = {
            400: "Check your request parameters. The server couldn't understand the request.",
            401: "Authentication failed. Verify your API key is correct and not expired.",
            403: "Access forbidden. Check if your API key has the required permissions.",
            404: "Endpoint not found. Verify the base_url is correct.",
            408: "Request timeout. Try increasing the timeout or using smaller batches.",
            429: "Rate limit exceeded. Implement exponential backoff or reduce request frequency.",
            500: "Server error. This is retryable - try again in a few seconds.",
            502: "Bad gateway. The server is temporarily unavailable. Retry with backoff.",
            503: "Service unavailable. The server is overloaded or down for maintenance.",
            504: "Gateway timeout. The server took too long to respond. Try again.",
        }

        suggestion = suggestions.get(status_code, "An unexpected error occurred. Check the API status.")

        if status_code in (408, 429, 500, 502, 503, 504):
            suggestion += " This error is retryable."

        return suggestion


class NetworkError(RipTideError):
    """Raised when network communication fails"""

    def __init__(self, message: str, **kwargs):
        suggestion = (
            "Check your internet connection and verify the server is reachable. "
            "If using a proxy, ensure it's configured correctly."
        )
        super().__init__(message, suggestion=suggestion, **kwargs)


class TimeoutError(RipTideError):
    """Raised when a request times out"""

    def __init__(self, message: str, **kwargs):
        suggestion = (
            "Increase the timeout value using with_timeout() or break the request into smaller chunks. "
            "Consider using streaming endpoints for large operations."
        )
        super().__init__(message, suggestion=suggestion, **kwargs)


class ConfigError(RipTideError):
    """Raised when there's a configuration error"""
    pass


class StreamingError(RipTideError):
    """Raised when streaming operations fail"""
    pass
