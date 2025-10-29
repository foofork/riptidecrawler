"""
PDF Processing Examples

Demonstrates how to use the RipTide SDK PDF API for text extraction
from PDF files with both synchronous and streaming approaches.
"""

import asyncio
from pathlib import Path
from riptide_sdk import RipTideClient, PdfExtractionOptions


async def example_basic_extraction():
    """Example: Basic synchronous PDF extraction"""
    print("\n=== Basic PDF Extraction ===\n")

    async with RipTideClient(base_url="http://localhost:8080") as client:
        # Load PDF file
        pdf_path = Path("sample.pdf")
        if not pdf_path.exists():
            print(f"Sample PDF not found at {pdf_path}")
            print("Please provide a PDF file to test with")
            return

        with open(pdf_path, "rb") as f:
            pdf_data = f.read()

        # Extract with default options
        result = await client.pdf.extract(
            pdf_data=pdf_data,
            filename="sample.pdf",
        )

        if result.success and result.document:
            print(f"✓ Extraction successful!")
            print(f"  Title: {result.document.title}")
            print(f"  Text length: {len(result.document.text or '')} characters")
            print(f"  Word count: {result.document.word_count}")
            print(f"  Processing time: {result.stats.processing_time_ms}ms")
            print(f"  Pages processed: {result.stats.pages_processed}")
            print(f"  Processing rate: {result.stats.pages_per_second:.2f} pages/sec")
            print(f"\nFirst 200 characters of text:")
            print(result.document.text[:200] if result.document.text else "No text")
        else:
            print(f"✗ Extraction failed: {result.error}")


async def example_custom_options():
    """Example: PDF extraction with custom options"""
    print("\n=== PDF Extraction with Custom Options ===\n")

    async with RipTideClient(base_url="http://localhost:8080") as client:
        pdf_path = Path("sample.pdf")
        if not pdf_path.exists():
            print("Sample PDF not found")
            return

        with open(pdf_path, "rb") as f:
            pdf_data = f.read()

        # Extract with custom options
        options = PdfExtractionOptions(
            extract_text=True,
            extract_metadata=True,
            extract_images=True,  # Enable image extraction
            include_page_numbers=True,
        )

        result = await client.pdf.extract(
            pdf_data=pdf_data,
            options=options,
            filename="sample.pdf",
            timeout=60,  # 60 second timeout
        )

        if result.success and result.document:
            print(f"✓ Extraction with images successful!")
            print(f"  Images found: {len(result.document.media or [])}")
            print(f"  Links found: {len(result.document.links or [])}")
            if result.document.markdown:
                print(f"  Markdown length: {len(result.document.markdown)} characters")
        else:
            print(f"✗ Extraction failed: {result.error}")


async def example_streaming_extraction():
    """Example: Streaming PDF extraction with progress tracking"""
    print("\n=== Streaming PDF Extraction ===\n")

    async with RipTideClient(base_url="http://localhost:8080") as client:
        pdf_path = Path("sample.pdf")
        if not pdf_path.exists():
            print("Sample PDF not found")
            return

        with open(pdf_path, "rb") as f:
            pdf_data = f.read()

        print(f"Starting extraction of {len(pdf_data)} bytes...")
        print("Progress updates:")
        print("-" * 60)

        # Stream progress updates
        async for progress in client.pdf.extract_with_progress(
            pdf_data=pdf_data,
            filename="sample.pdf"
        ):
            if progress.event_type == "progress":
                print(
                    f"  [{progress.percentage:.1f}%] "
                    f"Page {progress.current_page}/{progress.total_pages} | "
                    f"Stage: {progress.stage} | "
                    f"Speed: {progress.pages_per_second:.2f} pages/sec"
                )
                if progress.estimated_remaining_ms:
                    print(f"    ETA: {progress.estimated_remaining_ms}ms remaining")

            elif progress.event_type == "completed":
                print("-" * 60)
                print(f"✓ Extraction completed!")
                if progress.document:
                    print(f"  Text length: {len(progress.document.text or '')} characters")
                    print(f"  Word count: {progress.document.word_count}")
                print(f"  Final speed: {progress.pages_per_second:.2f} pages/sec")

            elif progress.event_type == "failed":
                print("-" * 60)
                print(f"✗ Extraction failed: {progress.error}")

            elif progress.event_type == "keepalive":
                pass  # Keep connection alive


async def example_health_check():
    """Example: Check PDF processing capabilities"""
    print("\n=== PDF Processing Health Check ===\n")

    async with RipTideClient(base_url="http://localhost:8080") as client:
        # Get metrics and capabilities
        metrics = await client.pdf.get_metrics()

        print(f"Status: {metrics.status}")
        print(f"PDF Processing Available: {metrics.pdf_processing_available}\n")

        print("Capabilities:")
        print(f"  ✓ Text extraction: {metrics.capabilities.text_extraction}")
        print(f"  ✓ Image extraction: {metrics.capabilities.image_extraction}")
        print(f"  ✓ Metadata extraction: {metrics.capabilities.metadata_extraction}")
        print(f"  ✓ Table extraction: {metrics.capabilities.table_extraction}")
        print(f"  ✓ Form extraction: {metrics.capabilities.form_extraction}")
        print(f"  ✓ Encrypted PDFs: {metrics.capabilities.encrypted_pdfs}")
        print(f"  Max file size: {metrics.capabilities.max_file_size_mb}MB")
        print(f"  Supported versions: {', '.join(metrics.capabilities.supported_versions)}\n")

        print("Features:")
        print(f"  ✓ Progress streaming: {metrics.features.progress_streaming}")
        print(f"  ✓ Concurrent processing: {metrics.features.concurrent_processing}")
        print(f"  ✓ Memory monitoring: {metrics.features.memory_monitoring}")
        print(f"  ✓ Performance metrics: {metrics.features.performance_metrics}")


async def example_batch_pdf_processing():
    """Example: Process multiple PDFs efficiently"""
    print("\n=== Batch PDF Processing ===\n")

    async with RipTideClient(base_url="http://localhost:8080") as client:
        # Find all PDFs in a directory
        pdf_dir = Path("pdfs")
        if not pdf_dir.exists():
            print(f"Directory {pdf_dir} not found")
            print("Create a 'pdfs' directory with sample PDF files")
            return

        pdf_files = list(pdf_dir.glob("*.pdf"))
        if not pdf_files:
            print(f"No PDF files found in {pdf_dir}")
            return

        print(f"Found {len(pdf_files)} PDF files to process\n")

        # Process each PDF concurrently
        tasks = []
        for pdf_file in pdf_files:
            with open(pdf_file, "rb") as f:
                pdf_data = f.read()

            task = client.pdf.extract(
                pdf_data=pdf_data,
                filename=pdf_file.name,
            )
            tasks.append(task)

        # Wait for all extractions to complete
        results = await asyncio.gather(*tasks, return_exceptions=True)

        # Print summary
        successful = sum(1 for r in results if not isinstance(r, Exception) and r.success)
        failed = len(results) - successful

        print(f"Results:")
        print(f"  ✓ Successful: {successful}")
        print(f"  ✗ Failed: {failed}")

        total_pages = sum(
            r.stats.pages_processed
            for r in results
            if not isinstance(r, Exception) and r.success and r.stats
        )
        print(f"  Total pages processed: {total_pages}")


async def example_error_handling():
    """Example: Proper error handling for PDF extraction"""
    print("\n=== Error Handling Examples ===\n")

    from riptide_sdk import ValidationError, APIError, TimeoutError

    async with RipTideClient(base_url="http://localhost:8080") as client:
        # Example 1: Empty PDF data
        print("1. Testing empty PDF data...")
        try:
            await client.pdf.extract(pdf_data=b"")
        except ValidationError as e:
            print(f"   ✓ Caught validation error: {e}")

        # Example 2: Invalid PDF data
        print("\n2. Testing invalid PDF data...")
        try:
            result = await client.pdf.extract(
                pdf_data=b"This is not a PDF",
                filename="invalid.pdf",
            )
            if not result.success:
                print(f"   ✓ API returned error: {result.error}")
        except APIError as e:
            print(f"   ✓ Caught API error: {e}")

        # Example 3: Timeout handling
        print("\n3. Testing timeout (1 second)...")
        try:
            # Create a small valid PDF for testing
            test_pdf = b"%PDF-1.4\n1 0 obj\n<</Type/Catalog>>\nendobj\nxref\n0 1\ntrailer\n<</Size 1>>\n%%EOF"
            result = await client.pdf.extract(
                pdf_data=test_pdf,
                filename="test.pdf",
                timeout=1,  # Very short timeout
            )
            print(f"   Result: {result.success}")
        except TimeoutError as e:
            print(f"   ✓ Caught timeout error: {e}")
        except Exception as e:
            print(f"   Other error: {type(e).__name__}: {e}")


async def main():
    """Run all examples"""
    print("=" * 60)
    print("RipTide SDK - PDF Processing Examples")
    print("=" * 60)

    # Run examples
    await example_health_check()
    await example_basic_extraction()
    await example_custom_options()
    await example_streaming_extraction()
    await example_batch_pdf_processing()
    await example_error_handling()

    print("\n" + "=" * 60)
    print("All examples completed!")
    print("=" * 60)


if __name__ == "__main__":
    # Run the async main function
    asyncio.run(main())
