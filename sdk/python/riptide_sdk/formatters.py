"""
Output Format Helpers for RipTide SDK

Provides convenience methods for converting API responses to various formats
like Markdown, JSON, and structured dictionaries.

Example:
    >>> from riptide_sdk.formatters import format_crawl_response
    >>>
    >>> result = await client.crawl.batch(urls)
    >>> markdown = format_crawl_response(result, format="markdown")
    >>> print(markdown)
"""

import json
from typing import Any, Dict, List, Literal, Optional
from datetime import datetime

from .models import (
    CrawlResponse,
    CrawlResult,
    Document,
    DomainProfile,
    EngineStats,
    ProfileStats,
)


def format_crawl_response(
    response: CrawlResponse,
    format: Literal["markdown", "json", "dict", "summary"] = "summary",
    include_documents: bool = False,
) -> str:
    """
    Format a CrawlResponse in various output formats

    Args:
        response: The CrawlResponse to format
        format: Output format (markdown, json, dict, summary)
        include_documents: Whether to include full document content

    Returns:
        Formatted string representation

    Example:
        >>> result = await client.crawl.batch(urls)
        >>> print(format_crawl_response(result, format="markdown"))

        # Crawl Results

        **Summary**
        - Total URLs: 10
        - Successful: 9
        - Failed: 1
        ...
    """
    if format == "dict":
        return _format_as_dict(response, include_documents)
    elif format == "json":
        data = _crawl_response_to_dict(response, include_documents)
        return json.dumps(data, indent=2)
    elif format == "markdown":
        return _format_crawl_response_markdown(response, include_documents)
    elif format == "summary":
        return _format_crawl_response_summary(response)
    else:
        raise ValueError(f"Unknown format: {format}")


def _format_crawl_response_summary(response: CrawlResponse) -> str:
    """Format a brief summary of crawl results"""
    cache_pct = (response.from_cache / response.total_urls * 100) if response.total_urls > 0 else 0
    success_pct = (response.successful / response.total_urls * 100) if response.total_urls > 0 else 0

    return f"""Crawl Summary:
  Total: {response.total_urls} URLs
  ✓ Successful: {response.successful} ({success_pct:.1f}%)
  ✗ Failed: {response.failed}
  ⚡ From Cache: {response.from_cache} ({cache_pct:.1f}%)
  ⏱  Avg Time: {response.statistics.avg_processing_time_ms:.0f}ms
  📊 Cache Hit Rate: {response.statistics.cache_hit_rate:.1%}"""


def _format_crawl_response_markdown(response: CrawlResponse, include_documents: bool) -> str:
    """Format crawl response as markdown"""
    md = ["# Crawl Results\n"]

    # Summary section
    md.append("## Summary\n")
    md.append(f"- **Total URLs**: {response.total_urls}")
    md.append(f"- **Successful**: {response.successful}")
    md.append(f"- **Failed**: {response.failed}")
    md.append(f"- **From Cache**: {response.from_cache}")
    md.append(f"- **Cache Hit Rate**: {response.statistics.cache_hit_rate:.1%}")
    md.append(f"- **Avg Processing Time**: {response.statistics.avg_processing_time_ms:.0f}ms\n")

    # Gate decisions
    md.append("## Gate Decisions\n")
    gates = response.statistics.gate_decisions
    md.append(f"- **Raw**: {gates.raw}")
    md.append(f"- **Probes First**: {gates.probes_first}")
    md.append(f"- **Headless**: {gates.headless}")
    md.append(f"- **Cached**: {gates.cached}\n")

    # Individual results
    md.append("## Results\n")
    for i, result in enumerate(response.results, 1):
        status_icon = "✓" if 200 <= result.status < 300 else "✗"
        cache_badge = "🔄 CACHED" if result.from_cache else ""

        md.append(f"### {i}. {status_icon} {result.url} {cache_badge}\n")
        md.append(f"- **Status**: {result.status}")
        md.append(f"- **Engine**: {result.gate_decision}")
        md.append(f"- **Quality Score**: {result.quality_score:.2f}")
        md.append(f"- **Processing Time**: {result.processing_time_ms}ms")

        if result.error:
            md.append(f"- **Error**: {result.error.message}")
            md.append(f"- **Error Type**: {result.error.error_type}")
            md.append(f"- **Retryable**: {result.error.retryable}")

        if include_documents and result.document:
            md.append("\n**Document Preview:**")
            if result.document.text:
                preview = result.document.text[:200].replace("\n", " ")
                md.append(f"```\n{preview}...\n```")

        md.append("")  # Empty line between results

    return "\n".join(md)


def _crawl_response_to_dict(response: CrawlResponse, include_documents: bool) -> Dict[str, Any]:
    """Convert CrawlResponse to dictionary"""
    data = {
        "summary": {
            "total_urls": response.total_urls,
            "successful": response.successful,
            "failed": response.failed,
            "from_cache": response.from_cache,
        },
        "statistics": {
            "total_processing_time_ms": response.statistics.total_processing_time_ms,
            "avg_processing_time_ms": response.statistics.avg_processing_time_ms,
            "cache_hit_rate": response.statistics.cache_hit_rate,
            "gate_decisions": {
                "raw": response.statistics.gate_decisions.raw,
                "probes_first": response.statistics.gate_decisions.probes_first,
                "headless": response.statistics.gate_decisions.headless,
                "cached": response.statistics.gate_decisions.cached,
            },
        },
        "results": [],
    }

    for result in response.results:
        result_data = {
            "url": result.url,
            "status": result.status,
            "from_cache": result.from_cache,
            "gate_decision": result.gate_decision,
            "quality_score": result.quality_score,
            "processing_time_ms": result.processing_time_ms,
        }

        if result.error:
            result_data["error"] = {
                "error_type": result.error.error_type,
                "message": result.error.message,
                "retryable": result.error.retryable,
            }

        if include_documents and result.document:
            result_data["document"] = {
                "has_html": result.document.html is not None,
                "has_text": result.document.text is not None,
                "has_markdown": result.document.markdown is not None,
                "text_length": len(result.document.text) if result.document.text else 0,
                "links_count": len(result.document.links) if result.document.links else 0,
            }

        data["results"].append(result_data)

    return data


def _format_as_dict(obj: Any, include_documents: bool) -> str:
    """Format any object as a dictionary representation"""
    if isinstance(obj, CrawlResponse):
        return str(_crawl_response_to_dict(obj, include_documents))
    return str(obj)


def format_domain_profile(
    profile: DomainProfile,
    format: Literal["markdown", "json", "summary"] = "summary",
) -> str:
    """
    Format a DomainProfile in various formats

    Args:
        profile: The DomainProfile to format
        format: Output format

    Returns:
        Formatted string
    """
    if format == "json":
        return json.dumps(profile.to_dict(), indent=2)
    elif format == "markdown":
        return _format_domain_profile_markdown(profile)
    elif format == "summary":
        return _format_domain_profile_summary(profile)
    else:
        raise ValueError(f"Unknown format: {format}")


def _format_domain_profile_summary(profile: DomainProfile) -> str:
    """Format domain profile summary"""
    lines = [f"Domain Profile: {profile.domain}"]

    if profile.config:
        cfg = profile.config
        if cfg.stealth_level:
            lines.append(f"  Stealth: {cfg.stealth_level.value}")
        if cfg.rate_limit:
            lines.append(f"  Rate Limit: {cfg.rate_limit} req/s")
        if cfg.ua_strategy:
            lines.append(f"  UA Strategy: {cfg.ua_strategy.value}")

    if profile.created_at:
        lines.append(f"  Created: {profile.created_at}")

    return "\n".join(lines)


def _format_domain_profile_markdown(profile: DomainProfile) -> str:
    """Format domain profile as markdown"""
    md = [f"# Domain Profile: {profile.domain}\n"]

    if profile.metadata:
        md.append("## Metadata\n")
        if profile.metadata.description:
            md.append(f"**Description**: {profile.metadata.description}\n")
        if profile.metadata.tags:
            md.append(f"**Tags**: {', '.join(profile.metadata.tags)}\n")
        if profile.metadata.author:
            md.append(f"**Author**: {profile.metadata.author}\n")

    if profile.config:
        md.append("## Configuration\n")
        cfg = profile.config

        if cfg.stealth_level:
            md.append(f"- **Stealth Level**: {cfg.stealth_level.value}")
        if cfg.rate_limit is not None:
            md.append(f"- **Rate Limit**: {cfg.rate_limit} requests/second")
        if cfg.respect_robots_txt is not None:
            md.append(f"- **Respect robots.txt**: {cfg.respect_robots_txt}")
        if cfg.ua_strategy:
            md.append(f"- **UA Strategy**: {cfg.ua_strategy.value}")
        if cfg.enable_javascript is not None:
            md.append(f"- **JavaScript Enabled**: {cfg.enable_javascript}")
        if cfg.request_timeout_secs is not None:
            md.append(f"- **Request Timeout**: {cfg.request_timeout_secs}s")

    if profile.created_at or profile.updated_at:
        md.append("\n## Timestamps\n")
        if profile.created_at:
            md.append(f"- **Created**: {profile.created_at}")
        if profile.updated_at:
            md.append(f"- **Updated**: {profile.updated_at}")

    return "\n".join(md)


def format_engine_stats(
    stats: EngineStats,
    format: Literal["markdown", "json", "summary"] = "summary",
) -> str:
    """
    Format EngineStats in various formats

    Args:
        stats: The EngineStats to format
        format: Output format

    Returns:
        Formatted string
    """
    if format == "json":
        return json.dumps({
            "total_decisions": stats.total_decisions,
            "raw_count": stats.raw_count,
            "probes_first_count": stats.probes_first_count,
            "headless_count": stats.headless_count,
            "probe_first_enabled": stats.probe_first_enabled,
        }, indent=2)
    elif format == "markdown":
        return f"""# Engine Statistics

## Decision Breakdown

- **Total Decisions**: {stats.total_decisions}
- **Raw Engine**: {stats.raw_count} ({stats.raw_count/stats.total_decisions*100:.1f}%)
- **Probes First**: {stats.probes_first_count} ({stats.probes_first_count/stats.total_decisions*100:.1f}%)
- **Headless**: {stats.headless_count} ({stats.headless_count/stats.total_decisions*100:.1f}%)

## Configuration

- **Probe First Enabled**: {stats.probe_first_enabled}
"""
    elif format == "summary":
        return f"""Engine Stats: {stats.total_decisions} total decisions
  Raw: {stats.raw_count}, Probes: {stats.probes_first_count}, Headless: {stats.headless_count}
  Probe First: {'Enabled' if stats.probe_first_enabled else 'Disabled'}"""
    else:
        raise ValueError(f"Unknown format: {format}")


# Extension methods for models (monkey patching)
def _add_format_methods():
    """Add format methods to model classes"""

    def to_markdown(self, include_documents: bool = False) -> str:
        """Convert to Markdown format"""
        return format_crawl_response(self, format="markdown", include_documents=include_documents)

    def to_json(self, include_documents: bool = False) -> str:
        """Convert to JSON format"""
        return format_crawl_response(self, format="json", include_documents=include_documents)

    def to_summary(self) -> str:
        """Convert to summary format"""
        return format_crawl_response(self, format="summary")

    # Add methods to CrawlResponse
    CrawlResponse.to_markdown = to_markdown
    CrawlResponse.to_json = to_json
    CrawlResponse.to_summary = to_summary

    # Add methods to DomainProfile
    def profile_to_markdown(self) -> str:
        return format_domain_profile(self, format="markdown")

    def profile_to_json(self) -> str:
        return format_domain_profile(self, format="json")

    def profile_to_summary(self) -> str:
        return format_domain_profile(self, format="summary")

    DomainProfile.to_markdown = profile_to_markdown
    DomainProfile.to_json = profile_to_json
    DomainProfile.to_summary = profile_to_summary

    # Add methods to EngineStats
    def stats_to_markdown(self) -> str:
        return format_engine_stats(self, format="markdown")

    def stats_to_json(self) -> str:
        return format_engine_stats(self, format="json")

    def stats_to_summary(self) -> str:
        return format_engine_stats(self, format="summary")

    EngineStats.to_markdown = stats_to_markdown
    EngineStats.to_json = stats_to_json
    EngineStats.to_summary = stats_to_summary


# Auto-apply format methods when module is imported
_add_format_methods()
