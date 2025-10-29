"""
Extract API endpoint implementation

Provides single-URL content extraction with multi-strategy support including
CSS, WASM, and hybrid extraction pipelines.
"""

from typing import Optional
import httpx

from ..models import ExtractOptions, ExtractionResult
from ..exceptions import APIError, ValidationError


class ExtractAPI:
    """API for content extraction operations"""

    def __init__(self, client: httpx.AsyncClient, base_url: str):
        """
        Initialize ExtractAPI

        Args:
            client: Async HTTP client
            base_url: Base URL for the API
        """
        self.client = client
        self.base_url = base_url

    async def extract(
        self,
        url: str,
        mode: str = "standard",
        options: Optional[ExtractOptions] = None,
    ) -> ExtractionResult:
        """
        Extract content from a single URL using multi-strategy extraction

        This endpoint provides a unified interface for content extraction,
        using the multi-strategy extraction pipeline (CSS, WASM, fallback).

        Args:
            url: URL to extract content from
            mode: Extraction mode - "standard", "article", "product", or "markdown"
            options: Optional extraction configuration

        Returns:
            ExtractionResult with extracted content and metadata

        Raises:
            ValidationError: If URL is invalid
            APIError: If the API returns an error

        Example:
            >>> # Basic extraction
            >>> result = await client.extract.extract("https://example.com")
            >>> print(f"Title: {result.title}")
            >>> print(f"Content: {result.content[:100]}...")
            >>> print(f"Quality: {result.quality_score:.2f}")
            >>>
            >>> # With custom options
            >>> from riptide_sdk.models import ExtractOptions
            >>> options = ExtractOptions(
            ...     strategy="wasm",
            ...     quality_threshold=0.8,
            ...     timeout_ms=15000
            ... )
            >>> result = await client.extract.extract(
            ...     "https://example.com/article",
            ...     mode="article",
            ...     options=options
            ... )
            >>> print(f"Strategy used: {result.strategy_used}")
            >>> print(f"Word count: {result.metadata.word_count}")
        """
        # Validate URL
        if not url:
            raise ValidationError("URL cannot be empty")

        if not url.startswith(("http://", "https://")):
            raise ValidationError(f"Invalid URL protocol: {url}")

        # Build request body
        body = {
            "url": url,
            "mode": mode,
        }

        if options:
            body["options"] = options.to_dict()

        # Make request
        response = await self.client.post(
            f"{self.base_url}/api/v1/extract",
            json=body,
        )

        if response.status_code != 200:
            error_data = response.json() if response.text else {}
            raise APIError(
                message=error_data.get("error", {}).get("message", "Extraction failed"),
                status_code=response.status_code,
                response_data=error_data,
            )

        return ExtractionResult.from_dict(response.json())

    async def extract_article(
        self,
        url: str,
        options: Optional[ExtractOptions] = None,
    ) -> ExtractionResult:
        """
        Extract content in article mode (optimized for news/blog posts)

        Args:
            url: URL to extract content from
            options: Optional extraction configuration

        Returns:
            ExtractionResult with article content

        Example:
            >>> result = await client.extract.extract_article(
            ...     "https://example.com/blog/post"
            ... )
            >>> print(f"Author: {result.metadata.author}")
            >>> print(f"Published: {result.metadata.publish_date}")
        """
        return await self.extract(url, mode="article", options=options)

    async def extract_markdown(
        self,
        url: str,
        options: Optional[ExtractOptions] = None,
    ) -> ExtractionResult:
        """
        Extract content as Markdown

        Args:
            url: URL to extract content from
            options: Optional extraction configuration

        Returns:
            ExtractionResult with content in Markdown format

        Example:
            >>> result = await client.extract.extract_markdown(
            ...     "https://example.com"
            ... )
            >>> print(result.content)  # Markdown formatted
        """
        return await self.extract(url, mode="markdown", options=options)

    async def extract_product(
        self,
        url: str,
        options: Optional[ExtractOptions] = None,
    ) -> ExtractionResult:
        """
        Extract content in product mode (optimized for e-commerce)

        Args:
            url: URL to extract content from
            options: Optional extraction configuration

        Returns:
            ExtractionResult with product information

        Example:
            >>> result = await client.extract.extract_product(
            ...     "https://shop.example.com/product/123"
            ... )
            >>> print(f"Product: {result.title}")
        """
        return await self.extract(url, mode="product", options=options)
