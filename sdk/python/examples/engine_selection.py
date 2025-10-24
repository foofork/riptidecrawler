"""
Engine Selection example (Phase 10)

Demonstrates intelligent engine selection and analysis capabilities.
"""

import asyncio
from riptide_sdk import RipTideClient


async def main():
    """Run engine selection examples"""

    async with RipTideClient(base_url="http://localhost:8080") as client:

        # Example 1: Analyze HTML and get engine recommendation
        print("Example 1: Analyze HTML for engine recommendation")
        print("-" * 50)

        # Static HTML example
        static_html = """
        <html>
            <head><title>Static Page</title></head>
            <body>
                <h1>Hello World</h1>
                <p>This is a static HTML page with no JavaScript.</p>
            </body>
        </html>
        """

        decision = await client.engine.analyze(
            html=static_html,
            url="https://example.com/static",
        )

        print(f"Recommended engine: {decision.engine}")
        print(f"Confidence: {decision.confidence:.2%}")
        print(f"Reasoning: {decision.reasoning}")
        print(f"Flags: {decision.flags}")
        print()

        # Example 2: SPA detection
        print("\nExample 2: SPA detection")
        print("-" * 50)

        spa_html = """
        <html>
            <head>
                <title>React App</title>
                <script src="/static/js/main.chunk.js"></script>
            </head>
            <body>
                <div id="root"></div>
                <script>
                    // React application entry point
                    ReactDOM.render(App, document.getElementById('root'));
                </script>
            </body>
        </html>
        """

        decision = await client.engine.analyze(
            html=spa_html,
            url="https://example.com/app",
        )

        print(f"Recommended engine: {decision.engine}")
        print(f"Confidence: {decision.confidence:.2%}")
        print(f"SPA detected: {decision.flags.get('has_spa', False)}")
        print(f"Requires JavaScript: {decision.flags.get('requires_js', False)}")
        print()

        # Example 3: Custom flags for engine decision
        print("\nExample 3: Engine decision with custom flags")
        print("-" * 50)

        decision = await client.engine.decide(
            html=static_html,
            url="https://example.com/custom",
            flags={
                "has_spa": False,
                "requires_js": False,
                "has_ajax": False,
                "has_websockets": False,
            }
        )

        print(f"Final engine decision: {decision.engine}")
        print(f"Applied flags: {decision.flags}")
        print()

        # Example 4: Get engine usage statistics
        print("\nExample 4: Get engine usage statistics")
        print("-" * 50)

        stats = await client.engine.get_stats()

        print(f"Total decisions: {stats.total_decisions}")
        print(f"Raw engine: {stats.raw_count}")
        print(f"Probes-first: {stats.probes_first_count}")
        print(f"Headless: {stats.headless_count}")
        print(f"Probe-first enabled: {stats.probe_first_enabled}")

        if stats.total_decisions > 0:
            print(f"\nEngine distribution:")
            print(f"  Raw: {stats.raw_count / stats.total_decisions:.1%}")
            print(f"  Probes-first: {stats.probes_first_count / stats.total_decisions:.1%}")
            print(f"  Headless: {stats.headless_count / stats.total_decisions:.1%}")
        print()

        # Example 5: Toggle probe-first mode
        print("\nExample 5: Toggle probe-first mode")
        print("-" * 50)

        # Enable probe-first
        result = await client.engine.toggle_probe_first(True)
        print(f"Probe-first enabled: {result.get('enabled')}")

        # Disable probe-first
        result = await client.engine.toggle_probe_first(False)
        print(f"Probe-first enabled: {result.get('enabled')}")
        print()

        # Example 6: Complex HTML analysis
        print("\nExample 6: Complex HTML with multiple indicators")
        print("-" * 50)

        complex_html = """
        <html>
            <head>
                <title>Complex Page</title>
                <script src="https://cdn.example.com/react.js"></script>
                <script src="https://cdn.example.com/vue.js"></script>
            </head>
            <body>
                <div id="app" data-framework="vue">
                    <nav class="dynamic-nav"></nav>
                </div>
                <script>
                    // AJAX calls
                    fetch('/api/data').then(r => r.json());

                    // WebSocket
                    const ws = new WebSocket('wss://example.com/live');

                    // Dynamic content loading
                    window.addEventListener('scroll', loadMore);
                </script>
            </body>
        </html>
        """

        decision = await client.engine.analyze(
            html=complex_html,
            url="https://example.com/complex",
        )

        print(f"Recommended engine: {decision.engine}")
        print(f"Confidence: {decision.confidence:.2%}")
        print(f"Reasoning: {decision.reasoning}")
        print(f"\nDetected features:")
        for flag, value in decision.flags.items():
            print(f"  {flag}: {value}")


if __name__ == "__main__":
    asyncio.run(main())
