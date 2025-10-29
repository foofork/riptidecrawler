"""
Unit tests for output formatters

Tests markdown, JSON, and summary format generation.
"""

import pytest
import json

from riptide_sdk.models import (
    CrawlResponse,
    CrawlResult,
    CrawlStatistics,
    GateDecisionBreakdown,
    Document,
    DomainProfile,
    ProfileConfig,
    ProfileMetadata,
    EngineStats,
    StealthLevel,
    UAStrategy,
)
from riptide_sdk.formatters import (
    format_crawl_response,
    format_domain_profile,
    format_engine_stats,
)


@pytest.fixture
def sample_crawl_response():
    """Create sample CrawlResponse for testing"""
    results = [
        CrawlResult(
            url="https://example.com",
            status=200,
            from_cache=True,
            gate_decision="raw",
            quality_score=0.95,
            processing_time_ms=45,
            document=Document(
                text="Sample content",
                html="<html>Sample</html>",
                markdown="# Sample",
            ),
        ),
        CrawlResult(
            url="https://test.com",
            status=200,
            from_cache=False,
            gate_decision="probes_first",
            quality_score=0.88,
            processing_time_ms=123,
        ),
    ]

    statistics = CrawlStatistics(
        total_processing_time_ms=168,
        avg_processing_time_ms=84.0,
        gate_decisions=GateDecisionBreakdown(
            raw=1,
            probes_first=1,
            headless=0,
            cached=1,
        ),
        cache_hit_rate=0.5,
    )

    return CrawlResponse(
        total_urls=2,
        successful=2,
        failed=0,
        from_cache=1,
        results=results,
        statistics=statistics,
    )


@pytest.mark.unit
class TestCrawlResponseFormatting:
    """Test CrawlResponse formatting"""

    def test_format_as_summary(self, sample_crawl_response):
        """Test summary format"""
        output = format_crawl_response(sample_crawl_response, format="summary")

        assert "Total: 2 URLs" in output
        assert "Successful: 2" in output
        assert "Failed: 0" in output
        assert "From Cache: 1" in output

    def test_format_as_json(self, sample_crawl_response):
        """Test JSON format"""
        output = format_crawl_response(sample_crawl_response, format="json")

        data = json.loads(output)
        assert data["summary"]["total_urls"] == 2
        assert data["summary"]["successful"] == 2
        assert len(data["results"]) == 2

    def test_format_as_markdown(self, sample_crawl_response):
        """Test markdown format"""
        output = format_crawl_response(sample_crawl_response, format="markdown")

        assert "# Crawl Results" in output
        assert "## Summary" in output
        assert "## Gate Decisions" in output
        assert "## Results" in output

    def test_format_as_dict(self, sample_crawl_response):
        """Test dict format"""
        output = format_crawl_response(sample_crawl_response, format="dict")

        assert isinstance(output, str)
        assert "total_urls" in output

    def test_invalid_format_raises_error(self, sample_crawl_response):
        """Test invalid format raises ValueError"""
        with pytest.raises(ValueError, match="Unknown format"):
            format_crawl_response(sample_crawl_response, format="xml")

    def test_markdown_with_documents(self, sample_crawl_response):
        """Test markdown includes document preview"""
        output = format_crawl_response(
            sample_crawl_response,
            format="markdown",
            include_documents=True,
        )

        assert "Document Preview" in output
        assert "Sample content" in output

    def test_json_with_documents(self, sample_crawl_response):
        """Test JSON includes document info"""
        output = format_crawl_response(
            sample_crawl_response,
            format="json",
            include_documents=True,
        )

        data = json.loads(output)
        assert "document" in data["results"][0]
        assert "has_text" in data["results"][0]["document"]

    def test_summary_shows_percentages(self, sample_crawl_response):
        """Test summary includes percentage calculations"""
        output = format_crawl_response(sample_crawl_response, format="summary")

        assert "100.0%" in output  # Success rate
        assert "50.0%" in output   # Cache rate

    def test_markdown_with_error_result(self):
        """Test markdown format handles errors"""
        from riptide_sdk.models import ErrorInfo

        result_with_error = CrawlResult(
            url="https://failed.com",
            status=500,
            from_cache=False,
            gate_decision="raw",
            quality_score=0.0,
            processing_time_ms=50,
            error=ErrorInfo(
                error_type="server_error",
                message="Internal server error",
                retryable=True,
            ),
        )

        response = CrawlResponse(
            total_urls=1,
            successful=0,
            failed=1,
            from_cache=0,
            results=[result_with_error],
            statistics=CrawlStatistics(
                total_processing_time_ms=50,
                avg_processing_time_ms=50.0,
                gate_decisions=GateDecisionBreakdown(1, 0, 0, 0),
                cache_hit_rate=0.0,
            ),
        )

        output = format_crawl_response(response, format="markdown")

        assert "Error" in output
        assert "server_error" in output
        assert "Retryable" in output


@pytest.mark.unit
class TestDomainProfileFormatting:
    """Test DomainProfile formatting"""

    def test_format_as_summary(self):
        """Test profile summary format"""
        profile = DomainProfile(
            domain="example.com",
            config=ProfileConfig(
                stealth_level=StealthLevel.MEDIUM,
                rate_limit=2.0,
                ua_strategy=UAStrategy.ROTATE,
            ),
            created_at="2024-01-01T00:00:00Z",
        )

        output = format_domain_profile(profile, format="summary")

        assert "example.com" in output
        assert "medium" in output
        assert "2.0" in output

    def test_format_as_json(self):
        """Test profile JSON format"""
        profile = DomainProfile(
            domain="example.com",
            config=ProfileConfig(stealth_level=StealthLevel.HIGH),
        )

        output = format_domain_profile(profile, format="json")

        data = json.loads(output)
        assert data["domain"] == "example.com"
        assert "config" in data

    def test_format_as_markdown(self):
        """Test profile markdown format"""
        profile = DomainProfile(
            domain="example.com",
            config=ProfileConfig(
                stealth_level=StealthLevel.HIGH,
                rate_limit=1.5,
                respect_robots_txt=True,
            ),
            metadata=ProfileMetadata(
                description="Test domain",
                tags=["test", "example"],
                author="test-user",
            ),
        )

        output = format_domain_profile(profile, format="markdown")

        assert "# Domain Profile" in output
        assert "## Metadata" in output
        assert "## Configuration" in output
        assert "Test domain" in output

    def test_invalid_format_raises_error(self):
        """Test invalid format raises ValueError"""
        profile = DomainProfile(domain="example.com")

        with pytest.raises(ValueError, match="Unknown format"):
            format_domain_profile(profile, format="yaml")


@pytest.mark.unit
class TestEngineStatsFormatting:
    """Test EngineStats formatting"""

    def test_format_as_summary(self):
        """Test stats summary format"""
        stats = EngineStats(
            total_decisions=100,
            raw_count=45,
            probes_first_count=35,
            headless_count=20,
            probe_first_enabled=True,
        )

        output = format_engine_stats(stats, format="summary")

        assert "100 total decisions" in output
        assert "45" in output  # Raw count
        assert "Enabled" in output

    def test_format_as_json(self):
        """Test stats JSON format"""
        stats = EngineStats(
            total_decisions=100,
            raw_count=45,
            probes_first_count=35,
            headless_count=20,
            probe_first_enabled=False,
        )

        output = format_engine_stats(stats, format="json")

        data = json.loads(output)
        assert data["total_decisions"] == 100
        assert data["probe_first_enabled"] is False

    def test_format_as_markdown(self):
        """Test stats markdown format"""
        stats = EngineStats(
            total_decisions=100,
            raw_count=45,
            probes_first_count=35,
            headless_count=20,
            probe_first_enabled=True,
        )

        output = format_engine_stats(stats, format="markdown")

        assert "# Engine Statistics" in output
        assert "## Decision Breakdown" in output
        assert "45.0%" in output  # Percentage calculation

    def test_invalid_format_raises_error(self):
        """Test invalid format raises ValueError"""
        stats = EngineStats(100, 45, 35, 20, True)

        with pytest.raises(ValueError, match="Unknown format"):
            format_engine_stats(stats, format="csv")


@pytest.mark.unit
class TestModelFormatMethods:
    """Test monkey-patched format methods on models"""

    def test_crawl_response_to_summary(self, sample_crawl_response):
        """Test CrawlResponse.to_summary() method"""
        output = sample_crawl_response.to_summary()

        assert isinstance(output, str)
        assert "Total: 2 URLs" in output

    def test_crawl_response_to_markdown(self, sample_crawl_response):
        """Test CrawlResponse.to_markdown() method"""
        output = sample_crawl_response.to_markdown()

        assert isinstance(output, str)
        assert "# Crawl Results" in output

    def test_crawl_response_to_json(self, sample_crawl_response):
        """Test CrawlResponse.to_json() method"""
        output = sample_crawl_response.to_json()

        data = json.loads(output)
        assert data["summary"]["total_urls"] == 2

    def test_domain_profile_to_summary(self):
        """Test DomainProfile.to_summary() method"""
        profile = DomainProfile(domain="test.com")
        output = profile.to_summary()

        assert isinstance(output, str)
        assert "test.com" in output

    def test_domain_profile_to_markdown(self):
        """Test DomainProfile.to_markdown() method"""
        profile = DomainProfile(domain="test.com")
        output = profile.to_markdown()

        assert isinstance(output, str)
        assert "# Domain Profile" in output

    def test_engine_stats_to_summary(self):
        """Test EngineStats.to_summary() method"""
        stats = EngineStats(100, 45, 35, 20, True)
        output = stats.to_summary()

        assert isinstance(output, str)
        assert "100 total decisions" in output


@pytest.mark.unit
class TestFormatterEdgeCases:
    """Test formatter edge cases"""

    def test_zero_urls_handled(self):
        """Test formatting with zero URLs"""
        response = CrawlResponse(
            total_urls=0,
            successful=0,
            failed=0,
            from_cache=0,
            results=[],
            statistics=CrawlStatistics(
                total_processing_time_ms=0,
                avg_processing_time_ms=0.0,
                gate_decisions=GateDecisionBreakdown(0, 0, 0, 0),
                cache_hit_rate=0.0,
            ),
        )

        output = format_crawl_response(response, format="summary")
        assert "0 URLs" in output

    def test_profile_without_config(self):
        """Test formatting profile without config"""
        profile = DomainProfile(domain="example.com")

        output = format_domain_profile(profile, format="markdown")
        assert "example.com" in output

    def test_profile_without_metadata(self):
        """Test formatting profile without metadata"""
        profile = DomainProfile(
            domain="example.com",
            config=ProfileConfig(rate_limit=1.0),
        )

        output = format_domain_profile(profile, format="markdown")
        assert "Configuration" in output
