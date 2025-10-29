# Browser Automation API

The Browser Automation API provides comprehensive headless browser control for web automation, testing, and scraping scenarios.

## Overview

The Browser API (`client.browser`) enables:

- **Session Management**: Create and manage isolated browser sessions with stealth configurations
- **Page Navigation**: Navigate to URLs with automatic page load detection
- **Element Interaction**: Click elements, type text, and interact with forms
- **JavaScript Execution**: Run custom JavaScript code in the browser context
- **Screenshot Capture**: Take full-page or viewport screenshots
- **PDF Rendering**: Convert web pages to PDF documents
- **Content Extraction**: Get page HTML and metadata
- **Pool Monitoring**: Track browser pool utilization and performance

## API Endpoints

### 1. Create Browser Session

**POST /api/v1/browser/session**

Creates a new headless browser session from the pool with optional stealth configuration.

```python
from riptide_sdk import RipTideClient
from riptide_sdk.models import BrowserSessionConfig

async with RipTideClient() as client:
    config = BrowserSessionConfig(
        stealth_preset="medium",      # none, low, medium, high
        initial_url="https://example.com",
        timeout_secs=600,             # Session TTL
    )

    session = await client.browser.create_session(config=config)
    print(f"Session ID: {session.session_id}")
    print(f"Pool utilization: {session.pool_stats.utilization_percent:.1f}%")
```

**Response:**
```python
BrowserSession(
    session_id="uuid",
    pool_stats=PoolStatusInfo(...),
    created_at="2025-10-29T10:00:00Z",
    expires_at="2025-10-29T10:10:00Z"
)
```

### 2. Execute Browser Action

**POST /api/v1/browser/action**

Executes various actions on an existing browser session.

#### Supported Actions

##### Navigate to URL
```python
result = await client.browser.navigate(
    session_id="session_123",
    url="https://example.com",
    wait_for_load=True
)
```

##### Click Element
```python
result = await client.browser.click(
    session_id="session_123",
    selector="#submit-button"
)
```

##### Type Text
```python
result = await client.browser.type_text(
    session_id="session_123",
    selector="#search-input",
    text="Hello World"
)
```

##### Take Screenshot
```python
result = await client.browser.screenshot(
    session_id="session_123",
    full_page=True
)
screenshot_data = result.result["screenshot_base64"]
```

##### Execute JavaScript
```python
result = await client.browser.execute_script(
    session_id="session_123",
    script="return document.title;",
    timeout_ms=5000
)
```

##### Get Page Content
```python
result = await client.browser.get_content(session_id="session_123")
html = result.result["html"]
```

##### Wait for Element
```python
result = await client.browser.wait_for_element(
    session_id="session_123",
    selector=".dynamic-content",
    timeout_ms=10000
)
```

##### Render to PDF
```python
result = await client.browser.render_pdf(
    session_id="session_123",
    landscape=True,
    print_background=True
)
pdf_data = result.result["pdf_base64"]
```

**Action Result:**
```python
BrowserActionResult(
    success=True,
    result={"key": "value"},  # Action-specific result data
    duration_ms=150,
    messages=["Action completed"]
)
```

### 3. Get Browser Pool Status

**GET /api/v1/browser/pool/status**

Retrieves detailed statistics about the browser pool.

```python
status = await client.browser.get_pool_status()
print(status.to_summary())
```

**Response:**
```python
BrowserPoolStatus(
    stats=PoolStatusInfo(
        available=5,
        in_use=3,
        total_capacity=8,
        utilization_percent=37.5
    ),
    launcher_stats=LauncherStatsInfo(
        total_requests=1000,
        successful_requests=980,
        failed_requests=20,
        avg_response_time_ms=450.5,
        stealth_requests=600,
        non_stealth_requests=400
    ),
    health="healthy"
)
```

### 4. Close Browser Session

**DELETE /api/v1/browser/session/{session_id}**

Closes a browser session and returns the browser to the pool.

```python
await client.browser.close_session(session_id="session_123")
```

## Data Models

### BrowserSessionConfig

Configuration for creating a browser session.

```python
@dataclass
class BrowserSessionConfig:
    stealth_preset: Optional[Literal["none", "low", "medium", "high"]] = None
    initial_url: Optional[str] = None
    timeout_secs: Optional[int] = None
```

**Stealth Presets:**
- `none`: No stealth features (fastest)
- `low`: Basic fingerprinting resistance
- `medium`: Standard anti-detection (recommended)
- `high`: Maximum stealth (slower, most realistic)

### BrowserSession

Browser session information returned on creation.

```python
@dataclass
class BrowserSession:
    session_id: str
    pool_stats: PoolStatusInfo
    created_at: str
    expires_at: str
```

### BrowserAction

Browser action to execute (use static factory methods).

```python
@dataclass
class BrowserAction:
    action_type: str
    # ... action-specific fields

    @staticmethod
    def navigate(session_id: str, url: str, wait_for_load: bool = True)

    @staticmethod
    def click(session_id: str, selector: str)

    @staticmethod
    def type_text(session_id: str, selector: str, text: str)

    # ... other factory methods
```

### BrowserActionResult

Result from executing a browser action.

```python
@dataclass
class BrowserActionResult:
    success: bool
    result: Dict[str, Any]  # Action-specific result data
    duration_ms: int
    messages: List[str]
```

### BrowserPoolStatus

Browser pool status and performance metrics.

```python
@dataclass
class BrowserPoolStatus:
    stats: PoolStatusInfo
    launcher_stats: LauncherStatsInfo
    health: str

    def to_summary(self) -> str:
        """Generate human-readable summary"""
```

## Usage Examples

### Example 1: Simple Web Scraping

```python
from riptide_sdk import RipTideClient
from riptide_sdk.models import BrowserSessionConfig

async with RipTideClient() as client:
    # Create session
    config = BrowserSessionConfig(
        stealth_preset="medium",
        initial_url="https://example.com"
    )
    session = await client.browser.create_session(config)

    # Extract content
    content = await client.browser.get_content(session.session_id)
    html = content.result["html"]

    # Take screenshot
    screenshot = await client.browser.screenshot(
        session.session_id,
        full_page=True
    )

    # Cleanup
    await client.browser.close_session(session.session_id)
```

### Example 2: Form Automation

```python
async with RipTideClient() as client:
    config = BrowserSessionConfig(initial_url="https://example.com/login")
    session = await client.browser.create_session(config)

    # Fill login form
    await client.browser.type_text(
        session.session_id,
        "#username",
        "user@example.com"
    )

    await client.browser.type_text(
        session.session_id,
        "#password",
        "secure_password"
    )

    # Submit form
    await client.browser.click(session.session_id, "#login-button")

    # Wait for redirect
    await client.browser.wait_for_element(
        session.session_id,
        ".dashboard",
        timeout_ms=10000
    )

    await client.browser.close_session(session.session_id)
```

### Example 3: JavaScript Data Extraction

```python
async with RipTideClient() as client:
    config = BrowserSessionConfig(initial_url="https://news.example.com")
    session = await client.browser.create_session(config)

    # Extract structured data with JavaScript
    script = """
    return Array.from(document.querySelectorAll('.article'))
        .map(article => ({
            title: article.querySelector('h2').textContent,
            url: article.querySelector('a').href,
            author: article.querySelector('.author').textContent
        }));
    """

    result = await client.browser.execute_script(
        session.session_id,
        script
    )

    articles = result.result.get("return_value", [])
    print(f"Found {len(articles)} articles")

    await client.browser.close_session(session.session_id)
```

### Example 4: PDF Generation

```python
async with RipTideClient() as client:
    config = BrowserSessionConfig(initial_url="https://example.com/report")
    session = await client.browser.create_session(config)

    # Wait for dynamic content to load
    await client.browser.wait_for_element(
        session.session_id,
        ".report-content",
        timeout_ms=15000
    )

    # Generate PDF
    pdf_result = await client.browser.render_pdf(
        session.session_id,
        landscape=True,
        print_background=True
    )

    # Save PDF
    import base64
    pdf_data = base64.b64decode(pdf_result.result["pdf_base64"])
    with open("report.pdf", "wb") as f:
        f.write(pdf_data)

    await client.browser.close_session(session.session_id)
```

### Example 5: Pool Monitoring

```python
async with RipTideClient() as client:
    status = await client.browser.get_pool_status()

    print(status.to_summary())

    # Check pool health
    if status.health == "healthy":
        print("✓ Browser pool is healthy")

    # Monitor utilization
    utilization = status.stats.utilization_percent
    if utilization > 80:
        print(f"⚠ High utilization: {utilization:.1f}%")

    # Performance metrics
    success_rate = status.launcher_stats.success_rate
    avg_time = status.launcher_stats.avg_response_time_ms
    print(f"Success rate: {success_rate:.1%}")
    print(f"Avg response time: {avg_time:.2f}ms")
```

## Error Handling

```python
from riptide_sdk.exceptions import APIError, ValidationError, TimeoutError

try:
    async with RipTideClient() as client:
        session = await client.browser.create_session()
        result = await client.browser.navigate(
            session.session_id,
            "https://example.com"
        )

except ValidationError as e:
    print(f"Validation error: {e.message}")
    print(f"Suggestion: {e.suggestion}")

except TimeoutError as e:
    print(f"Request timed out: {e.message}")
    print(f"Suggestion: {e.suggestion}")

except APIError as e:
    print(f"API error [{e.status_code}]: {e.message}")
    if e.is_retryable:
        print("This error is retryable - try again")
```

## Best Practices

### 1. Session Lifecycle Management

```python
# Always close sessions when done
async with RipTideClient() as client:
    session = await client.browser.create_session()
    try:
        # Perform actions
        pass
    finally:
        await client.browser.close_session(session.session_id)
```

### 2. Stealth Configuration

```python
# Use appropriate stealth level for your use case
config = BrowserSessionConfig(
    stealth_preset="medium",  # Balanced performance and detection resistance
    timeout_secs=600,         # Adequate session lifetime
)
```

### 3. Wait for Dynamic Content

```python
# Always wait for content before interacting
await client.browser.wait_for_element(
    session_id,
    "#dynamic-content",
    timeout_ms=10000
)
```

### 4. Pool Monitoring

```python
# Monitor pool health before creating many sessions
status = await client.browser.get_pool_status()
if status.stats.utilization_percent > 90:
    # Wait or scale pool
    await asyncio.sleep(5)
```

### 5. Error Recovery

```python
# Implement retry logic for transient failures
from tenacity import retry, stop_after_attempt, wait_exponential

@retry(stop=stop_after_attempt(3), wait=wait_exponential(multiplier=1, min=4, max=10))
async def navigate_with_retry(client, session_id, url):
    return await client.browser.navigate(session_id, url)
```

## Performance Considerations

### Session Reuse

Reuse sessions for multiple operations instead of creating new sessions:

```python
# Good - Reuse session
session = await client.browser.create_session()
for url in urls:
    await client.browser.navigate(session.session_id, url)
    # Process page
await client.browser.close_session(session.session_id)

# Bad - Creating new sessions repeatedly
for url in urls:
    session = await client.browser.create_session()
    await client.browser.navigate(session.session_id, url)
    await client.browser.close_session(session.session_id)
```

### Concurrent Sessions

Process multiple pages concurrently:

```python
import asyncio

async def process_url(client, url):
    session = await client.browser.create_session()
    try:
        await client.browser.navigate(session.session_id, url)
        content = await client.browser.get_content(session.session_id)
        return content.result["html"]
    finally:
        await client.browser.close_session(session.session_id)

# Process 10 URLs concurrently
results = await asyncio.gather(
    *[process_url(client, url) for url in urls[:10]]
)
```

### Screenshot Optimization

Use viewport screenshots instead of full-page when possible:

```python
# Faster - viewport only
await client.browser.screenshot(session_id, full_page=False)

# Slower - full page scroll
await client.browser.screenshot(session_id, full_page=True)
```

## Advanced Use Cases

### Dynamic Content Scraping

```python
# Wait for AJAX content to load
await client.browser.wait_for_element(
    session_id,
    ".ajax-loaded-content",
    timeout_ms=15000
)

# Execute JavaScript to trigger loading
await client.browser.execute_script(
    session_id,
    "window.scrollTo(0, document.body.scrollHeight);"
)

# Wait for new content
await asyncio.sleep(2)
```

### Multi-Step Workflows

```python
# Login, navigate, extract, logout
session = await client.browser.create_session()

# Step 1: Login
await client.browser.type_text(session.session_id, "#user", "username")
await client.browser.type_text(session.session_id, "#pass", "password")
await client.browser.click(session.session_id, "#login")

# Step 2: Navigate to target page
await client.browser.navigate(session.session_id, "/dashboard")
await client.browser.wait_for_element(session.session_id, ".data-table")

# Step 3: Extract data
content = await client.browser.get_content(session.session_id)

# Step 4: Logout
await client.browser.click(session.session_id, "#logout")

await client.browser.close_session(session.session_id)
```

## See Also

- [Browser Automation Example](../examples/browser_example.py) - Comprehensive examples
- [API Reference](./API_REFERENCE.md) - Complete SDK documentation
- [Sessions API](./SESSIONS_API.md) - Cookie and session management
- [Rust Browser Handler](../../crates/riptide-api/src/handlers/browser.rs) - Backend implementation
