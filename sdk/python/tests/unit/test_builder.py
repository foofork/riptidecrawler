"""
Unit tests for RipTideClientBuilder

Tests fluent API, validation, chaining, and configuration building.
"""

import pytest
import warnings

from riptide_sdk import RipTideClientBuilder, RipTideClient, RetryConfig
from riptide_sdk.exceptions import ConfigError


@pytest.mark.unit
class TestBuilderInitialization:
    """Test builder initialization and defaults"""

    def test_builder_creates_with_defaults(self):
        """Test builder initializes with default values"""
        builder = RipTideClientBuilder()

        assert builder._base_url == "http://localhost:8080"
        assert builder._api_key is None
        assert builder._timeout == 30.0
        assert builder._max_connections == 100

    def test_builder_repr(self):
        """Test builder string representation"""
        builder = RipTideClientBuilder().with_base_url("http://test.com")

        repr_str = repr(builder)
        assert "RipTideClientBuilder" in repr_str
        assert "test.com" in repr_str


@pytest.mark.unit
class TestBuilderBaseURL:
    """Test base URL configuration"""

    def test_with_base_url(self):
        """Test setting base URL"""
        builder = RipTideClientBuilder().with_base_url("http://api.example.com")

        assert builder._base_url == "http://api.example.com"

    def test_with_base_url_removes_trailing_slash(self):
        """Test trailing slash is removed"""
        builder = RipTideClientBuilder().with_base_url("http://test.com/")

        assert builder._base_url == "http://test.com"

    def test_with_base_url_https(self):
        """Test HTTPS URLs are accepted"""
        builder = RipTideClientBuilder().with_base_url("https://api.example.com")

        assert builder._base_url == "https://api.example.com"

    def test_empty_base_url_raises_error(self):
        """Test empty base URL raises ValueError"""
        builder = RipTideClientBuilder()

        with pytest.raises(ValueError, match="Base URL cannot be empty"):
            builder.with_base_url("")

    def test_invalid_protocol_raises_error(self):
        """Test invalid protocol raises ValueError"""
        builder = RipTideClientBuilder()

        with pytest.raises(ValueError, match="must start with"):
            builder.with_base_url("ftp://example.com")

    def test_no_protocol_raises_error(self):
        """Test missing protocol raises ValueError"""
        builder = RipTideClientBuilder()

        with pytest.raises(ValueError, match="must start with"):
            builder.with_base_url("example.com")

    def test_with_base_url_returns_self(self):
        """Test method returns self for chaining"""
        builder = RipTideClientBuilder()

        result = builder.with_base_url("http://test.com")

        assert result is builder


@pytest.mark.unit
class TestBuilderAPIKey:
    """Test API key configuration"""

    def test_with_api_key(self):
        """Test setting API key"""
        builder = RipTideClientBuilder().with_api_key("test-key-123")

        assert builder._api_key == "test-key-123"

    def test_empty_api_key_raises_error(self):
        """Test empty API key raises ValueError"""
        builder = RipTideClientBuilder()

        with pytest.raises(ValueError, match="API key cannot be empty"):
            builder.with_api_key("")

    def test_with_api_key_returns_self(self):
        """Test method returns self for chaining"""
        builder = RipTideClientBuilder()

        result = builder.with_api_key("test-key")

        assert result is builder

    def test_api_key_masked_in_repr(self):
        """Test API key is masked in string representation"""
        builder = RipTideClientBuilder().with_api_key("secret-key")

        repr_str = repr(builder)
        assert "secret-key" not in repr_str
        assert "***" in repr_str


@pytest.mark.unit
class TestBuilderTimeout:
    """Test timeout configuration"""

    def test_with_timeout(self):
        """Test setting custom timeout"""
        builder = RipTideClientBuilder().with_timeout(60.0)

        assert builder._timeout == 60.0

    def test_zero_timeout_raises_error(self):
        """Test zero timeout raises ValueError"""
        builder = RipTideClientBuilder()

        with pytest.raises(ValueError, match="Timeout must be positive"):
            builder.with_timeout(0)

    def test_negative_timeout_raises_error(self):
        """Test negative timeout raises ValueError"""
        builder = RipTideClientBuilder()

        with pytest.raises(ValueError, match="Timeout must be positive"):
            builder.with_timeout(-10.0)

    def test_very_high_timeout_warns(self):
        """Test very high timeout issues warning"""
        builder = RipTideClientBuilder()

        with warnings.catch_warnings(record=True) as w:
            warnings.simplefilter("always")
            builder.with_timeout(500.0)

            assert len(w) == 1
            assert "very high" in str(w[0].message).lower()

    def test_with_timeout_returns_self(self):
        """Test method returns self for chaining"""
        builder = RipTideClientBuilder()

        result = builder.with_timeout(45.0)

        assert result is builder


@pytest.mark.unit
class TestBuilderConnections:
    """Test connection configuration"""

    def test_with_max_connections(self):
        """Test setting max connections"""
        builder = RipTideClientBuilder().with_max_connections(200)

        assert builder._max_connections == 200

    def test_zero_connections_raises_error(self):
        """Test zero connections raises ValueError"""
        builder = RipTideClientBuilder()

        with pytest.raises(ValueError, match="must be at least 1"):
            builder.with_max_connections(0)

    def test_negative_connections_raises_error(self):
        """Test negative connections raises ValueError"""
        builder = RipTideClientBuilder()

        with pytest.raises(ValueError, match="must be at least 1"):
            builder.with_max_connections(-5)

    def test_too_many_connections_raises_error(self):
        """Test excessive connections raises ValueError"""
        builder = RipTideClientBuilder()

        with pytest.raises(ValueError, match="too high"):
            builder.with_max_connections(2000)

    def test_with_max_keepalive(self):
        """Test setting max keepalive connections"""
        builder = RipTideClientBuilder().with_max_keepalive(50)

        assert builder._max_keepalive == 50

    def test_zero_keepalive_raises_error(self):
        """Test zero keepalive raises ValueError"""
        builder = RipTideClientBuilder()

        with pytest.raises(ValueError, match="must be at least 1"):
            builder.with_max_keepalive(0)


@pytest.mark.unit
class TestBuilderRetryConfig:
    """Test retry configuration"""

    def test_with_retry_config_defaults(self):
        """Test retry config with default values"""
        builder = RipTideClientBuilder().with_retry_config()

        assert isinstance(builder._retry_config, RetryConfig)
        assert builder._retry_config.max_retries == 3
        assert builder._retry_config.backoff_factor == 2.0

    def test_with_retry_config_custom(self):
        """Test retry config with custom values"""
        builder = RipTideClientBuilder().with_retry_config(
            max_retries=5,
            backoff_factor=1.5,
            max_backoff=120.0,
        )

        assert builder._retry_config.max_retries == 5
        assert builder._retry_config.backoff_factor == 1.5
        assert builder._retry_config.max_backoff == 120.0

    def test_negative_retries_raises_error(self):
        """Test negative retries raises ValueError"""
        builder = RipTideClientBuilder()

        with pytest.raises(ValueError, match="cannot be negative"):
            builder.with_retry_config(max_retries=-1)

    def test_low_backoff_factor_raises_error(self):
        """Test backoff factor < 1 raises ValueError"""
        builder = RipTideClientBuilder()

        with pytest.raises(ValueError, match="must be >= 1.0"):
            builder.with_retry_config(backoff_factor=0.5)


@pytest.mark.unit
class TestBuilderHeaders:
    """Test header configuration"""

    def test_with_user_agent(self):
        """Test setting custom User-Agent"""
        builder = RipTideClientBuilder().with_user_agent("MyApp/1.0")

        assert builder._user_agent == "MyApp/1.0"

    def test_with_custom_header(self):
        """Test adding custom header"""
        builder = RipTideClientBuilder().with_custom_header("X-Custom", "value")

        assert builder._custom_headers["X-Custom"] == "value"

    def test_multiple_custom_headers(self):
        """Test adding multiple custom headers"""
        builder = (RipTideClientBuilder()
                   .with_custom_header("X-Header-1", "value1")
                   .with_custom_header("X-Header-2", "value2"))

        assert len(builder._custom_headers) == 2


@pytest.mark.unit
class TestBuilderSSL:
    """Test SSL configuration"""

    def test_with_ssl_verification_disabled(self):
        """Test disabling SSL verification"""
        builder = RipTideClientBuilder()

        with warnings.catch_warnings(record=True) as w:
            warnings.simplefilter("always")
            builder.with_ssl_verification(False)

            assert builder._verify_ssl is False
            assert len(w) == 1
            assert "insecure" in str(w[0].message).lower()

    def test_with_ssl_verification_enabled(self):
        """Test enabling SSL verification (default)"""
        builder = RipTideClientBuilder().with_ssl_verification(True)

        assert builder._verify_ssl is True


@pytest.mark.unit
class TestBuilderRedirects:
    """Test redirect configuration"""

    def test_with_follow_redirects_enabled(self):
        """Test enabling redirects (default)"""
        builder = RipTideClientBuilder().with_follow_redirects(True)

        assert builder._follow_redirects is True

    def test_with_follow_redirects_disabled(self):
        """Test disabling redirects"""
        builder = RipTideClientBuilder().with_follow_redirects(False)

        assert builder._follow_redirects is False


@pytest.mark.unit
class TestBuilderChaining:
    """Test fluent API chaining"""

    def test_full_chain(self):
        """Test complete chain of builder methods"""
        builder = (RipTideClientBuilder()
                   .with_base_url("https://api.example.com")
                   .with_api_key("test-key")
                   .with_timeout(60.0)
                   .with_max_connections(150)
                   .with_retry_config(max_retries=5)
                   .with_user_agent("TestApp/1.0")
                   .with_custom_header("X-Test", "value"))

        assert builder._base_url == "https://api.example.com"
        assert builder._api_key == "test-key"
        assert builder._timeout == 60.0
        assert builder._max_connections == 150
        assert builder._retry_config.max_retries == 5

    def test_partial_chain(self):
        """Test partial configuration chain"""
        builder = (RipTideClientBuilder()
                   .with_base_url("http://test.com")
                   .with_timeout(45.0))

        assert builder._base_url == "http://test.com"
        assert builder._timeout == 45.0
        assert builder._api_key is None  # Not set


@pytest.mark.unit
class TestBuilderBuild:
    """Test building client from builder"""

    def test_build_creates_client(self):
        """Test build() creates RipTideClient"""
        builder = RipTideClientBuilder()
        client = builder.build()

        assert isinstance(client, RipTideClient)

    def test_build_applies_base_url(self):
        """Test build() applies base URL"""
        client = (RipTideClientBuilder()
                  .with_base_url("https://api.test.com")
                  .build())

        assert client.base_url == "https://api.test.com"

    def test_build_applies_api_key(self):
        """Test build() applies API key"""
        client = (RipTideClientBuilder()
                  .with_api_key("test-key")
                  .build())

        assert client.api_key == "test-key"

    def test_build_applies_timeout(self):
        """Test build() applies timeout"""
        client = (RipTideClientBuilder()
                  .with_timeout(90.0)
                  .build())

        assert client._client.timeout.connect == 90.0

    def test_build_applies_custom_headers(self):
        """Test build() applies custom headers"""
        client = (RipTideClientBuilder()
                  .with_user_agent("CustomApp/2.0")
                  .with_custom_header("X-Custom", "test")
                  .build())

        assert client._client.headers["User-Agent"] == "CustomApp/2.0"
        assert client._client.headers["X-Custom"] == "test"

    def test_build_applies_retry_config(self):
        """Test build() attaches retry config"""
        client = (RipTideClientBuilder()
                  .with_retry_config(max_retries=7)
                  .build())

        assert hasattr(client, "_retry_config")
        assert client._retry_config.max_retries == 7

    def test_build_with_ssl_disabled(self):
        """Test build() with SSL verification disabled"""
        with warnings.catch_warnings():
            warnings.simplefilter("ignore")
            client = (RipTideClientBuilder()
                      .with_ssl_verification(False)
                      .build())

            assert client._client.verify is False


@pytest.mark.unit
class TestRetryConfig:
    """Test RetryConfig dataclass"""

    def test_retry_config_defaults(self):
        """Test RetryConfig default values"""
        config = RetryConfig()

        assert config.max_retries == 3
        assert config.backoff_factor == 2.0
        assert config.max_backoff == 60.0

    def test_retry_config_custom(self):
        """Test RetryConfig with custom values"""
        config = RetryConfig(
            max_retries=5,
            backoff_factor=1.5,
            retry_on_status=(500, 502, 503),
            max_backoff=120.0,
        )

        assert config.max_retries == 5
        assert config.backoff_factor == 1.5
        assert config.retry_on_status == (500, 502, 503)
        assert config.max_backoff == 120.0

    def test_retry_config_to_dict(self):
        """Test RetryConfig serialization"""
        config = RetryConfig(max_retries=3)
        data = config.to_dict()

        assert isinstance(data, dict)
        assert data["max_retries"] == 3
        assert "backoff_factor" in data
