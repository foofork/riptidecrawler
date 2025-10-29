#!/usr/bin/env python3
"""
Browser Automation Example

Demonstrates comprehensive browser automation capabilities using the RipTide SDK,
including session management, page navigation, element interaction, JavaScript
execution, screenshots, and PDF rendering.

Usage:
    python browser_example.py
"""

import asyncio
import base64
import sys
from pathlib import Path

# Add parent directory to path for local imports
sys.path.insert(0, str(Path(__file__).parent.parent))

from riptide_sdk import RipTideClient
from riptide_sdk.models import BrowserSessionConfig, BrowserAction


async def example_1_basic_session_and_navigation():
    """Example 1: Create a browser session and navigate to a URL"""
    print("\n" + "="*70)
    print("Example 1: Basic Session Creation and Navigation")
    print("="*70)

    async with RipTideClient(base_url="http://localhost:8080") as client:
        # Create a browser session with stealth configuration
        config = BrowserSessionConfig(
            stealth_preset="medium",
            initial_url="https://example.com",
            timeout_secs=600,
        )

        session = await client.browser.create_session(config=config)
        print(f"\nBrowser session created:")
        print(f"  Session ID: {session.session_id}")
        print(f"  Created at: {session.created_at}")
        print(f"  Expires at: {session.expires_at}")
        print(f"  Pool utilization: {session.pool_stats.utilization_percent:.1f}%")

        # Navigate to a different page
        result = await client.browser.navigate(
            session.session_id,
            "https://www.wikipedia.org",
            wait_for_load=True,
        )

        print(f"\nNavigation result:")
        print(f"  Success: {result.success}")
        print(f"  Duration: {result.duration_ms}ms")
        print(f"  Messages: {', '.join(result.messages)}")

        # Clean up
        await client.browser.close_session(session.session_id)
        print(f"\nSession closed successfully")


async def example_2_form_interaction():
    """Example 2: Fill out a form with typing and clicking"""
    print("\n" + "="*70)
    print("Example 2: Form Interaction (Type and Click)")
    print("="*70)

    async with RipTideClient(base_url="http://localhost:8080") as client:
        # Create session and navigate to a page with a search form
        config = BrowserSessionConfig(
            stealth_preset="high",
            initial_url="https://www.wikipedia.org",
        )

        session = await client.browser.create_session(config=config)
        print(f"Session created: {session.session_id}")

        # Wait for search input to appear
        wait_result = await client.browser.wait_for_element(
            session.session_id,
            "#searchInput",
            timeout_ms=10000,
        )
        print(f"\nSearch input found: {wait_result.result.get('found', False)}")

        # Type into the search box
        type_result = await client.browser.type_text(
            session.session_id,
            "#searchInput",
            "Python programming language",
        )
        print(f"\nTyped text into search box:")
        print(f"  Success: {type_result.success}")
        print(f"  Characters typed: {type_result.result.get('text_length', 0)}")

        # Click the search button
        click_result = await client.browser.click(
            session.session_id,
            "button[type='submit']",
        )
        print(f"\nClicked search button:")
        print(f"  Success: {click_result.success}")
        print(f"  Duration: {click_result.duration_ms}ms")

        # Clean up
        await client.browser.close_session(session.session_id)


async def example_3_screenshot_capture():
    """Example 3: Take screenshots of pages"""
    print("\n" + "="*70)
    print("Example 3: Screenshot Capture")
    print("="*70)

    async with RipTideClient(base_url="http://localhost:8080") as client:
        # Create session
        config = BrowserSessionConfig(
            stealth_preset="low",
            initial_url="https://www.github.com",
        )

        session = await client.browser.create_session(config=config)
        print(f"Session created: {session.session_id}")

        # Take a full-page screenshot
        screenshot_result = await client.browser.screenshot(
            session.session_id,
            full_page=True,
        )

        if screenshot_result.success:
            screenshot_b64 = screenshot_result.result["screenshot_base64"]
            size_bytes = screenshot_result.result["size_bytes"]

            print(f"\nScreenshot captured:")
            print(f"  Format: {screenshot_result.result['format']}")
            print(f"  Size: {size_bytes:,} bytes")
            print(f"  Duration: {screenshot_result.duration_ms}ms")

            # Save screenshot to file
            screenshot_data = base64.b64decode(screenshot_b64)
            output_file = Path("screenshot.png")
            output_file.write_bytes(screenshot_data)
            print(f"  Saved to: {output_file.absolute()}")

        # Clean up
        await client.browser.close_session(session.session_id)


async def example_4_javascript_execution():
    """Example 4: Execute JavaScript code in the browser"""
    print("\n" + "="*70)
    print("Example 4: JavaScript Execution")
    print("="*70)

    async with RipTideClient(base_url="http://localhost:8080") as client:
        # Create session
        config = BrowserSessionConfig(
            initial_url="https://example.com",
        )

        session = await client.browser.create_session(config=config)
        print(f"Session created: {session.session_id}")

        # Execute JavaScript to get page title
        script_result = await client.browser.execute_script(
            session.session_id,
            "return document.title;",
            timeout_ms=5000,
        )

        print(f"\nJavaScript execution result:")
        print(f"  Success: {script_result.success}")
        print(f"  Duration: {script_result.duration_ms}ms")

        # Execute JavaScript to count links
        links_script = """
        return {
            totalLinks: document.querySelectorAll('a').length,
            visibleLinks: Array.from(document.querySelectorAll('a'))
                .filter(a => a.offsetParent !== null).length
        };
        """

        links_result = await client.browser.execute_script(
            session.session_id,
            links_script,
        )

        print(f"\nLink counting result:")
        print(f"  Success: {links_result.success}")
        print(f"  Messages: {', '.join(links_result.messages)}")

        # Clean up
        await client.browser.close_session(session.session_id)


async def example_5_get_page_content():
    """Example 5: Extract HTML content from the page"""
    print("\n" + "="*70)
    print("Example 5: Get Page Content")
    print("="*70)

    async with RipTideClient(base_url="http://localhost:8080") as client:
        # Create session and navigate
        config = BrowserSessionConfig(
            initial_url="https://example.com",
        )

        session = await client.browser.create_session(config=config)
        print(f"Session created: {session.session_id}")

        # Get page HTML content
        content_result = await client.browser.get_content(session.session_id)

        if content_result.success:
            html = content_result.result["html"]
            length = content_result.result["length"]

            print(f"\nPage content retrieved:")
            print(f"  HTML length: {length:,} characters")
            print(f"  Duration: {content_result.duration_ms}ms")
            print(f"\nFirst 200 characters:")
            print(f"  {html[:200]}...")

        # Clean up
        await client.browser.close_session(session.session_id)


async def example_6_pdf_rendering():
    """Example 6: Render page to PDF"""
    print("\n" + "="*70)
    print("Example 6: PDF Rendering")
    print("="*70)

    async with RipTideClient(base_url="http://localhost:8080") as client:
        # Create session
        config = BrowserSessionConfig(
            initial_url="https://www.wikipedia.org",
        )

        session = await client.browser.create_session(config=config)
        print(f"Session created: {session.session_id}")

        # Render page to PDF with landscape orientation
        pdf_result = await client.browser.render_pdf(
            session.session_id,
            landscape=True,
            print_background=True,
        )

        if pdf_result.success:
            pdf_b64 = pdf_result.result["pdf_base64"]
            size_bytes = pdf_result.result["size_bytes"]

            print(f"\nPDF rendered:")
            print(f"  Size: {size_bytes:,} bytes")
            print(f"  Duration: {pdf_result.duration_ms}ms")

            if pdf_b64:
                # Save PDF to file
                pdf_data = base64.b64decode(pdf_b64)
                output_file = Path("page.pdf")
                output_file.write_bytes(pdf_data)
                print(f"  Saved to: {output_file.absolute()}")

        # Clean up
        await client.browser.close_session(session.session_id)


async def example_7_browser_pool_monitoring():
    """Example 7: Monitor browser pool status and metrics"""
    print("\n" + "="*70)
    print("Example 7: Browser Pool Monitoring")
    print("="*70)

    async with RipTideClient(base_url="http://localhost:8080") as client:
        # Get pool status
        pool_status = await client.browser.get_pool_status()

        print(f"\n{pool_status.to_summary()}")


async def example_8_advanced_workflow():
    """Example 8: Advanced workflow - Web scraping with multiple actions"""
    print("\n" + "="*70)
    print("Example 8: Advanced Web Scraping Workflow")
    print("="*70)

    async with RipTideClient(base_url="http://localhost:8080") as client:
        # Create session with high stealth
        config = BrowserSessionConfig(
            stealth_preset="high",
            initial_url="https://news.ycombinator.com",
            timeout_secs=900,
        )

        session = await client.browser.create_session(config=config)
        print(f"Session created: {session.session_id}")

        # Step 1: Wait for content to load
        await client.browser.wait_for_element(
            session.session_id,
            ".itemlist",
            timeout_ms=10000,
        )
        print("✓ Page content loaded")

        # Step 2: Extract page content
        content_result = await client.browser.get_content(session.session_id)
        html_length = content_result.result["length"]
        print(f"✓ Extracted {html_length:,} characters of HTML")

        # Step 3: Execute JavaScript to extract article titles
        extract_script = """
        return Array.from(document.querySelectorAll('.titleline > a'))
            .slice(0, 10)
            .map(a => ({
                title: a.textContent,
                url: a.href
            }));
        """

        articles_result = await client.browser.execute_script(
            session.session_id,
            extract_script,
        )
        print(f"✓ Extracted article data")

        # Step 4: Take a screenshot for reference
        screenshot_result = await client.browser.screenshot(
            session.session_id,
            full_page=False,
        )
        if screenshot_result.success:
            print(f"✓ Captured screenshot ({screenshot_result.result['size_bytes']:,} bytes)")

        # Step 5: Navigate to a specific article (simulate click)
        # In a real scenario, you would click on an element
        print("✓ Workflow completed successfully")

        # Clean up
        await client.browser.close_session(session.session_id)
        print(f"\nSession closed")


async def main():
    """Run all examples"""
    print("\n" + "="*70)
    print("RipTide SDK - Browser Automation Examples")
    print("="*70)
    print("\nThis script demonstrates comprehensive browser automation features:")
    print("  1. Basic session creation and navigation")
    print("  2. Form interaction (typing and clicking)")
    print("  3. Screenshot capture")
    print("  4. JavaScript execution")
    print("  5. Page content extraction")
    print("  6. PDF rendering")
    print("  7. Browser pool monitoring")
    print("  8. Advanced web scraping workflow")
    print("\nNote: Ensure the RipTide API server is running on localhost:8080")
    print("="*70)

    try:
        # Run all examples
        await example_1_basic_session_and_navigation()
        await example_2_form_interaction()
        await example_3_screenshot_capture()
        await example_4_javascript_execution()
        await example_5_get_page_content()
        await example_6_pdf_rendering()
        await example_7_browser_pool_monitoring()
        await example_8_advanced_workflow()

        print("\n" + "="*70)
        print("All examples completed successfully!")
        print("="*70)

    except Exception as e:
        print(f"\n❌ Error: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)


if __name__ == "__main__":
    asyncio.run(main())
