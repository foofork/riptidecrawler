"""
Unit tests for exception classes

Tests error handling, suggestions, and exception hierarchy.
"""

import pytest

from riptide_sdk.exceptions import (
    RipTideError,
    ValidationError,
    APIError,
    NetworkError,
    TimeoutError,
    ConfigError,
    StreamingError,
)


@pytest.mark.unit
class TestRipTideError:
    """Test base RipTideError exception"""

    def test_basic_creation(self):
        """Test creating basic error"""
        error = RipTideError("Test error")

        assert "Test error" in str(error)
        assert error.message == "Test error"

    def test_with_status_code(self):
        """Test error with status code"""
        error = RipTideError("Error message", status_code=404)

        assert error.status_code == 404
        assert "[404]" in str(error)

    def test_with_response_data(self):
        """Test error with response data"""
        error = RipTideError("Error", response_data={"detail": "Not found"})

        assert error.response_data == {"detail": "Not found"}

    def test_with_suggestion(self):
        """Test error with suggestion"""
        error = RipTideError("Error", suggestion="Try this instead")

        assert error.suggestion == "Try this instead"
        assert "💡 Suggestion:" in str(error)

    def test_with_docs_url(self):
        """Test error with documentation URL"""
        error = RipTideError("Error", docs_url="https://docs.example.com")

        assert "📚 Documentation:" in str(error)
        assert "https://docs.example.com" in str(error)

    def test_default_docs_url(self):
        """Test error has default docs URL"""
        error = RipTideError("Error")

        assert error.docs_url is not None
        assert "github.com" in error.docs_url

    def test_empty_response_data_defaults_to_dict(self):
        """Test response_data defaults to empty dict"""
        error = RipTideError("Error")

        assert error.response_data == {}


@pytest.mark.unit
class TestValidationError:
    """Test ValidationError exception"""

    def test_validation_error_basic(self):
        """Test basic validation error"""
        error = ValidationError("Invalid input")

        assert isinstance(error, RipTideError)
        assert "Invalid input" in str(error)

    def test_validation_error_has_suggestion(self):
        """Test validation error includes suggestion"""
        error = ValidationError("URL is invalid")

        assert error.suggestion is not None
        assert "http" in error.suggestion.lower() or "url" in error.suggestion.lower()

    def test_validation_error_with_field(self):
        """Test validation error with field parameter"""
        error = ValidationError("Field is required", field="email")

        assert error.field == "email"

    def test_url_validation_suggestion(self):
        """Test URL validation provides specific suggestion"""
        error = ValidationError("Invalid URL format")

        assert "http" in error.suggestion.lower()
        assert "example.com" in error.suggestion.lower()

    def test_empty_field_suggestion(self):
        """Test empty field validation suggestion"""
        error = ValidationError("Field cannot be empty", field="username")

        assert "required" in error.suggestion.lower()
        assert "username" in error.suggestion or "field" in error.suggestion

    def test_maximum_validation_suggestion(self):
        """Test maximum validation suggestion"""
        error = ValidationError("Maximum limit exceeded")

        assert "smaller batches" in error.suggestion.lower() or "split" in error.suggestion.lower()


@pytest.mark.unit
class TestAPIError:
    """Test APIError exception"""

    def test_api_error_basic(self):
        """Test basic API error"""
        error = APIError("API request failed", status_code=500)

        assert isinstance(error, RipTideError)
        assert error.status_code == 500

    def test_api_error_type(self):
        """Test API error with error type"""
        error = APIError("Error", status_code=400, error_type="validation")

        assert error.error_type == "validation"

    def test_api_error_default_type(self):
        """Test API error default type"""
        error = APIError("Error", status_code=500)

        assert error.error_type == "api_error"

    def test_retryable_408(self):
        """Test 408 Request Timeout is retryable"""
        error = APIError("Timeout", status_code=408)

        assert error.is_retryable is True

    def test_retryable_429(self):
        """Test 429 Rate Limit is retryable"""
        error = APIError("Rate limited", status_code=429)

        assert error.is_retryable is True

    def test_retryable_500(self):
        """Test 500 Internal Server Error is retryable"""
        error = APIError("Server error", status_code=500)

        assert error.is_retryable is True

    def test_retryable_502(self):
        """Test 502 Bad Gateway is retryable"""
        error = APIError("Bad gateway", status_code=502)

        assert error.is_retryable is True

    def test_retryable_503(self):
        """Test 503 Service Unavailable is retryable"""
        error = APIError("Service unavailable", status_code=503)

        assert error.is_retryable is True

    def test_retryable_504(self):
        """Test 504 Gateway Timeout is retryable"""
        error = APIError("Gateway timeout", status_code=504)

        assert error.is_retryable is True

    def test_not_retryable_400(self):
        """Test 400 Bad Request is not retryable"""
        error = APIError("Bad request", status_code=400)

        assert error.is_retryable is False

    def test_not_retryable_401(self):
        """Test 401 Unauthorized is not retryable"""
        error = APIError("Unauthorized", status_code=401)

        assert error.is_retryable is False

    def test_not_retryable_404(self):
        """Test 404 Not Found is not retryable"""
        error = APIError("Not found", status_code=404)

        assert error.is_retryable is False

    def test_400_suggestion(self):
        """Test 400 error provides helpful suggestion"""
        error = APIError("Bad request", status_code=400)

        assert "request parameters" in error.suggestion.lower()

    def test_401_suggestion(self):
        """Test 401 error mentions API key"""
        error = APIError("Unauthorized", status_code=401)

        assert "api key" in error.suggestion.lower()

    def test_403_suggestion(self):
        """Test 403 error mentions permissions"""
        error = APIError("Forbidden", status_code=403)

        assert "permission" in error.suggestion.lower()

    def test_404_suggestion(self):
        """Test 404 error mentions base URL"""
        error = APIError("Not found", status_code=404)

        assert "base_url" in error.suggestion

    def test_429_suggestion(self):
        """Test 429 error mentions rate limiting"""
        error = APIError("Rate limited", status_code=429)

        assert "rate limit" in error.suggestion.lower()
        assert "retryable" in error.suggestion.lower()

    def test_500_suggestion(self):
        """Test 500 error suggests retry"""
        error = APIError("Server error", status_code=500)

        assert "retry" in error.suggestion.lower()

    def test_502_suggestion(self):
        """Test 502 error mentions temporary issue"""
        error = APIError("Bad gateway", status_code=502)

        assert "unavailable" in error.suggestion.lower() or "retry" in error.suggestion.lower()

    def test_503_suggestion(self):
        """Test 503 error mentions service availability"""
        error = APIError("Service unavailable", status_code=503)

        assert "unavailable" in error.suggestion.lower() or "overload" in error.suggestion.lower()


@pytest.mark.unit
class TestNetworkError:
    """Test NetworkError exception"""

    def test_network_error_basic(self):
        """Test basic network error"""
        error = NetworkError("Connection failed")

        assert isinstance(error, RipTideError)
        assert "Connection failed" in str(error)

    def test_network_error_has_suggestion(self):
        """Test network error includes helpful suggestion"""
        error = NetworkError("Cannot connect")

        assert error.suggestion is not None
        assert "internet connection" in error.suggestion.lower() or "reachable" in error.suggestion.lower()

    def test_network_error_mentions_proxy(self):
        """Test network error suggestion mentions proxy"""
        error = NetworkError("Connection refused")

        assert "proxy" in error.suggestion.lower()


@pytest.mark.unit
class TestTimeoutError:
    """Test TimeoutError exception"""

    def test_timeout_error_basic(self):
        """Test basic timeout error"""
        error = TimeoutError("Request timed out")

        assert isinstance(error, RipTideError)
        assert "timed out" in str(error).lower()

    def test_timeout_error_has_suggestion(self):
        """Test timeout error includes suggestion"""
        error = TimeoutError("Timeout after 30s")

        assert error.suggestion is not None
        assert "timeout" in error.suggestion.lower()

    def test_timeout_error_suggests_increase(self):
        """Test timeout error suggests increasing timeout"""
        error = TimeoutError("Timeout")

        assert "increase" in error.suggestion.lower() or "with_timeout" in error.suggestion

    def test_timeout_error_suggests_streaming(self):
        """Test timeout error mentions streaming"""
        error = TimeoutError("Timeout")

        assert "streaming" in error.suggestion.lower()


@pytest.mark.unit
class TestConfigError:
    """Test ConfigError exception"""

    def test_config_error_basic(self):
        """Test basic config error"""
        error = ConfigError("Invalid configuration")

        assert isinstance(error, RipTideError)
        assert "Invalid configuration" in str(error)

    def test_config_error_inherits_base(self):
        """Test ConfigError is a RipTideError"""
        error = ConfigError("Config issue")

        assert isinstance(error, RipTideError)


@pytest.mark.unit
class TestStreamingError:
    """Test StreamingError exception"""

    def test_streaming_error_basic(self):
        """Test basic streaming error"""
        error = StreamingError("Stream interrupted")

        assert isinstance(error, RipTideError)
        assert "Stream interrupted" in str(error)

    def test_streaming_error_with_status_code(self):
        """Test streaming error with status code"""
        error = StreamingError("Streaming failed", status_code=500)

        assert error.status_code == 500


@pytest.mark.unit
class TestExceptionHierarchy:
    """Test exception inheritance hierarchy"""

    def test_all_inherit_from_base(self):
        """Test all exceptions inherit from RipTideError"""
        exceptions = [
            ValidationError("test"),
            APIError("test", 500),
            NetworkError("test"),
            TimeoutError("test"),
            ConfigError("test"),
            StreamingError("test"),
        ]

        for exc in exceptions:
            assert isinstance(exc, RipTideError)
            assert isinstance(exc, Exception)

    def test_all_inherit_from_exception(self):
        """Test all exceptions can be caught as Exception"""
        try:
            raise ValidationError("test")
        except Exception as e:
            assert isinstance(e, Exception)


@pytest.mark.unit
class TestExceptionStringRepresentation:
    """Test exception string formatting"""

    def test_error_with_all_fields(self):
        """Test error with all fields has complete output"""
        error = RipTideError(
            "Error message",
            status_code=404,
            suggestion="Try this",
            docs_url="https://docs.example.com",
        )

        error_str = str(error)
        assert "[404]" in error_str
        assert "Error message" in error_str
        assert "💡 Suggestion:" in error_str
        assert "📚 Documentation:" in error_str

    def test_error_minimal_fields(self):
        """Test error with minimal fields"""
        error = RipTideError("Simple error")

        error_str = str(error)
        assert "Simple error" in error_str

    def test_api_error_complete_output(self):
        """Test API error shows complete information"""
        error = APIError("Request failed", status_code=500, error_type="server_error")

        error_str = str(error)
        assert "[500]" in error_str
        assert "Request failed" in error_str
        assert "💡 Suggestion:" in error_str  # Auto-generated suggestion
