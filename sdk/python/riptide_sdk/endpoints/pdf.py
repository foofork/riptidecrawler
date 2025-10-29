"""
PDF processing API endpoint implementation

Provides PDF text extraction operations with support for both synchronous
and asynchronous (progress-tracked) extraction modes.
"""

from typing import Optional, AsyncIterator
import httpx
import base64

from ..models import (
    PdfExtractionOptions,
    PdfExtractionResult,
    PdfJobStatus,
    PdfMetrics,
    PdfStreamProgress,
)
from ..exceptions import APIError, ValidationError, TimeoutError


class PdfAPI:
    """API for PDF processing operations"""

    def __init__(self, client: httpx.AsyncClient, base_url: str):
        """
        Initialize PdfAPI

        Args:
            client: Async HTTP client
            base_url: Base URL for the API
        """
        self.client = client
        self.base_url = base_url

    async def extract(
        self,
        pdf_data: bytes,
        options: Optional[PdfExtractionOptions] = None,
        filename: Optional[str] = None,
        timeout: Optional[int] = None,
    ) -> PdfExtractionResult:
        """
        Extract text from PDF synchronously

        This method processes the PDF and returns the complete result.
        Use extract_with_progress() for large PDFs that need progress tracking.

        Args:
            pdf_data: Raw PDF bytes
            options: Optional extraction options
            filename: Optional original filename
            timeout: Optional timeout override in seconds

        Returns:
            PdfExtractionResult with extracted content and statistics

        Raises:
            ValidationError: If PDF data is invalid
            APIError: If the API returns an error
            TimeoutError: If processing exceeds timeout

        Example:
            >>> with open("document.pdf", "rb") as f:
            ...     pdf_data = f.read()
            >>> result = await client.pdf.extract(
            ...     pdf_data,
            ...     options=PdfExtractionOptions(extract_images=True)
            ... )
            >>> print(result.document.text)
            >>> print(f"Processing time: {result.stats.processing_time_ms}ms")
        """
        if not pdf_data:
            raise ValidationError("PDF data cannot be empty")

        # Validate file size (50MB limit from Rust API)
        if len(pdf_data) > 50 * 1024 * 1024:
            raise ValidationError("PDF file too large (max 50MB)")

        # Encode PDF data as base64
        encoded_data = base64.b64encode(pdf_data).decode('utf-8')

        # Build request body
        body = {
            "pdf_data": encoded_data,
        }
        if filename:
            body["filename"] = filename
        if timeout:
            body["timeout"] = timeout

        # Make request
        try:
            response = await self.client.post(
                f"{self.base_url}/api/v1/pdf/process",
                json=body,
                timeout=timeout if timeout else None,
            )
        except httpx.TimeoutException as e:
            raise TimeoutError(
                operation="PDF extraction",
                details=f"Request timed out after {timeout or 'default'}s",
            ) from e

        if response.status_code != 200:
            error_data = response.json() if response.text else {}
            raise APIError(
                message=error_data.get("error", "PDF extraction failed"),
                status_code=response.status_code,
                response_data=error_data,
            )

        return PdfExtractionResult.from_dict(response.json())

    async def extract_with_progress(
        self,
        pdf_data: bytes,
        options: Optional[PdfExtractionOptions] = None,
        filename: Optional[str] = None,
    ) -> AsyncIterator[PdfStreamProgress]:
        """
        Extract text from PDF with streaming progress updates

        This method streams real-time progress updates using NDJSON format.
        Each progress update includes current page, estimated time, and metrics.

        Args:
            pdf_data: Raw PDF bytes
            options: Optional extraction options
            filename: Optional original filename

        Yields:
            PdfStreamProgress objects with progress updates

        Raises:
            ValidationError: If PDF data is invalid
            APIError: If the API returns an error

        Example:
            >>> with open("large_document.pdf", "rb") as f:
            ...     pdf_data = f.read()
            >>> async for progress in client.pdf.extract_with_progress(pdf_data):
            ...     if progress.event_type == "progress":
            ...         print(f"Page {progress.current_page}/{progress.total_pages}")
            ...         print(f"Progress: {progress.percentage:.1f}%")
            ...     elif progress.event_type == "completed":
            ...         print(f"Done! Text: {progress.document.text[:100]}")
        """
        if not pdf_data:
            raise ValidationError("PDF data cannot be empty")

        if len(pdf_data) > 50 * 1024 * 1024:
            raise ValidationError("PDF file too large (max 50MB)")

        # Encode PDF data as base64
        encoded_data = base64.b64encode(pdf_data).decode('utf-8')

        # Build request body
        body = {
            "pdf_data": encoded_data,
            "stream_progress": True,
        }
        if filename:
            body["filename"] = filename

        # Make streaming request
        async with self.client.stream(
            "POST",
            f"{self.base_url}/api/v1/pdf/process-stream",
            json=body,
        ) as response:
            if response.status_code != 200:
                error_text = await response.aread()
                try:
                    error_data = response.json() if error_text else {}
                except Exception:
                    error_data = {"error": error_text.decode('utf-8', errors='ignore')}
                raise APIError(
                    message=error_data.get("error", "PDF extraction failed"),
                    status_code=response.status_code,
                    response_data=error_data,
                )

            # Stream NDJSON responses
            async for line in response.aiter_lines():
                if line.strip():
                    import json
                    try:
                        data = json.loads(line)
                        yield PdfStreamProgress.from_dict(data)
                    except json.JSONDecodeError:
                        # Skip invalid JSON lines
                        continue

    async def get_job_status(self, job_id: str) -> PdfJobStatus:
        """
        Get status of an asynchronous PDF extraction job

        Note: This endpoint is planned but not yet implemented in the Rust API.
        Currently, the API only supports synchronous and streaming extraction.

        Args:
            job_id: Job identifier

        Returns:
            PdfJobStatus with current job state

        Raises:
            APIError: If the API returns an error

        Example:
            >>> status = await client.pdf.get_job_status("job_12345")
            >>> print(f"Status: {status.status}")
            >>> if status.status == "completed":
            ...     print(status.result.document.text)
        """
        response = await self.client.get(
            f"{self.base_url}/api/v1/pdf/extract/{job_id}"
        )

        if response.status_code != 200:
            error_data = response.json() if response.text else {}
            raise APIError(
                message=error_data.get("error", "Failed to get job status"),
                status_code=response.status_code,
                response_data=error_data,
            )

        return PdfJobStatus.from_dict(response.json())

    async def get_metrics(self) -> PdfMetrics:
        """
        Get PDF processing metrics and health information

        Returns comprehensive metrics about PDF processing capabilities,
        feature support, and system health.

        Returns:
            PdfMetrics with processing statistics and capabilities

        Raises:
            APIError: If the API returns an error

        Example:
            >>> metrics = await client.pdf.get_metrics()
            >>> print(f"PDF processing available: {metrics.pdf_processing_available}")
            >>> print(f"Text extraction: {metrics.capabilities.text_extraction}")
            >>> print(f"Max file size: {metrics.capabilities.max_file_size_mb}MB")
        """
        response = await self.client.get(
            f"{self.base_url}/api/v1/pdf/healthz"
        )

        if response.status_code != 200:
            error_data = response.json() if response.text else {}
            raise APIError(
                message=error_data.get("error", "Failed to get metrics"),
                status_code=response.status_code,
                response_data=error_data,
            )

        return PdfMetrics.from_dict(response.json())

    async def health_check(self) -> dict:
        """
        Perform health check on PDF processing capabilities

        This is a convenience wrapper around get_metrics() that returns
        the raw health check data.

        Returns:
            Dictionary with health status and capabilities

        Example:
            >>> health = await client.pdf.health_check()
            >>> print(health["status"])
            >>> print(health["capabilities"])
        """
        metrics = await self.get_metrics()
        return {
            "status": metrics.status,
            "pdf_processing_available": metrics.pdf_processing_available,
            "capabilities": {
                "text_extraction": metrics.capabilities.text_extraction,
                "image_extraction": metrics.capabilities.image_extraction,
                "metadata_extraction": metrics.capabilities.metadata_extraction,
                "table_extraction": metrics.capabilities.table_extraction,
                "form_extraction": metrics.capabilities.form_extraction,
                "encrypted_pdfs": metrics.capabilities.encrypted_pdfs,
                "max_file_size_mb": metrics.capabilities.max_file_size_mb,
                "supported_versions": metrics.capabilities.supported_versions,
            },
            "features": {
                "progress_streaming": metrics.features.progress_streaming,
                "concurrent_processing": metrics.features.concurrent_processing,
                "memory_monitoring": metrics.features.memory_monitoring,
                "performance_metrics": metrics.features.performance_metrics,
            },
        }
