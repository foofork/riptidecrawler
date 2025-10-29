# WebSocket Streaming Guide

## Overview

WebSocket streaming provides bidirectional real-time communication for the RipTide SDK, enabling efficient crawling with automatic connection management, health monitoring, and backpressure handling.

## Installation

WebSocket support requires the `websockets` library:

```bash
pip install websockets
```

Or install with the SDK:

```bash
pip install riptide-sdk[websockets]
```

## Features

### Core Features
- **Bidirectional Communication**: Send and receive messages in real-time
- **Automatic Keepalive**: Built-in ping/pong mechanism (30-second interval)
- **Connection Health Monitoring**: Track connection status and performance
- **Backpressure Handling**: Intelligent flow control for slow clients
- **Real-time Progress**: Get streaming progress updates during crawls
- **Error Recovery**: Robust error handling with graceful shutdown

### Message Types
- `welcome` - Initial connection acknowledgment
- `metadata` - Crawl session information
- `result` - Individual URL results with progress
- `summary` - Final crawl statistics
- `error` - Error messages
- `pong` - Ping response
- `status` - Connection health information

## Basic Usage

### Simple WebSocket Streaming

```python
from riptide_sdk import AsyncRipTideClient

async with AsyncRipTideClient(base_url="http://localhost:3000") as client:
    urls = ["https://example.com", "https://httpbin.org/html"]

    async for result in client.streaming.crawl_websocket(urls):
        if result.event_type == "result":
            crawl_result = result.data["result"]
            print(f"URL: {crawl_result['url']}")
            print(f"Status: {crawl_result['status']}")
            print(f"Quality: {crawl_result['quality_score']}")

        elif result.event_type == "summary":
            print(f"Completed: {result.data['successful']} successful")
```

### With Crawl Options

```python
from riptide_sdk.models import CrawlOptions, CacheMode

options = CrawlOptions(
    cache_mode=CacheMode.READ_WRITE,
    concurrency=5,
    timeout_secs=30
)

async for result in client.streaming.crawl_websocket(urls, options=options):
    # Process results
    pass
```

## Advanced Usage

### Real-time Callbacks

Process messages as they arrive with async callbacks:

```python
async def handle_message(result):
    """Process each message in real-time."""
    if result.event_type == "result":
        crawl_result = result.data["result"]

        # High-quality results
        if crawl_result["quality_score"] > 0.8:
            await save_to_database(crawl_result)

        # Failed results
        elif crawl_result.get("error"):
            await log_error(crawl_result["url"], crawl_result["error"])

async for result in client.streaming.crawl_websocket(
    urls,
    on_message=handle_message
):
    # Results are also yielded for iteration
    pass
```

### Error Handling and Retry

```python
import asyncio
from riptide_sdk.exceptions import StreamingError

max_retries = 3
retry_count = 0

while retry_count < max_retries:
    try:
        async for result in client.streaming.crawl_websocket(urls):
            # Process results
            pass
        break  # Success

    except StreamingError as e:
        retry_count += 1
        if retry_count < max_retries:
            wait_time = 2 ** retry_count  # Exponential backoff
            print(f"Error: {e}. Retrying in {wait_time}s...")
            await asyncio.sleep(wait_time)
        else:
            print("Max retries exceeded")
            raise
```

### Connection Health Monitoring

```python
# Test connection latency
ping_result = await client.streaming.ping_websocket()
print(f"Latency: {ping_result['latency_ms']:.2f}ms")
print(f"Session: {ping_result['session_id']}")

# Get detailed connection status
status = await client.streaming.get_websocket_status()
print(f"Healthy: {status['is_healthy']}")
print(f"Messages: {status['message_count']}")
print(f"Duration: {status['connected_duration_ms']}ms")
```

### Graceful Shutdown

```python
import asyncio

shutdown_event = asyncio.Event()

async def shutdown_handler():
    """Handle shutdown signal."""
    await asyncio.sleep(10)  # Wait for signal
    shutdown_event.set()

# Start shutdown handler
shutdown_task = asyncio.create_task(shutdown_handler())

try:
    async for result in client.streaming.crawl_websocket(urls):
        if shutdown_event.is_set():
            print("Shutdown requested, stopping stream...")
            break

        # Process results

finally:
    shutdown_task.cancel()
```

## Message Structure

### Welcome Message
```json
{
  "message_type": "welcome",
  "data": {
    "session_id": "550e8400-e29b-41d4-a716-446655440000",
    "server_time": "2024-01-01T00:00:00Z",
    "protocol_version": "1.0",
    "supported_operations": ["crawl", "ping", "status"]
  },
  "timestamp": "2024-01-01T00:00:00Z"
}
```

### Metadata Message
```json
{
  "message_type": "metadata",
  "data": {
    "total_urls": 10,
    "session_id": "550e8400-e29b-41d4-a716-446655440000",
    "timestamp": "2024-01-01T00:00:00Z",
    "stream_type": "crawl"
  },
  "timestamp": "2024-01-01T00:00:00Z"
}
```

### Result Message
```json
{
  "message_type": "result",
  "data": {
    "index": 0,
    "result": {
      "url": "https://example.com",
      "status": 200,
      "from_cache": false,
      "gate_decision": "raw",
      "quality_score": 0.95,
      "processing_time_ms": 150,
      "document": {
        "html": "<html>...</html>",
        "text": "Example content",
        "markdown": "# Example\n\nContent..."
      },
      "error": null,
      "cache_key": "https://example.com:sha256:..."
    },
    "progress": {
      "completed": 1,
      "total": 10,
      "success_rate": 1.0
    }
  },
  "timestamp": "2024-01-01T00:00:01Z"
}
```

### Summary Message
```json
{
  "message_type": "summary",
  "data": {
    "total_urls": 10,
    "successful": 9,
    "failed": 1,
    "total_processing_time_ms": 5000
  },
  "timestamp": "2024-01-01T00:00:05Z"
}
```

### Error Message
```json
{
  "message_type": "error",
  "data": {
    "error_type": "validation_error",
    "message": "Invalid URL format"
  },
  "timestamp": "2024-01-01T00:00:00Z"
}
```

## Performance Comparison

### WebSocket vs NDJSON

```python
import time

# WebSocket streaming
ws_start = time.time()
ws_count = 0
async for result in client.streaming.crawl_websocket(urls):
    if result.event_type == "result":
        ws_count += 1
ws_elapsed = time.time() - ws_start

# NDJSON streaming
ndjson_start = time.time()
ndjson_count = 0
async for result in client.streaming.crawl_ndjson(urls):
    ndjson_count += 1
ndjson_elapsed = time.time() - ndjson_start

print(f"WebSocket: {ws_count} results in {ws_elapsed:.2f}s")
print(f"NDJSON: {ndjson_count} results in {ndjson_elapsed:.2f}s")
```

### When to Use WebSocket

**Use WebSocket when you need:**
- Bidirectional communication
- Real-time progress updates
- Connection health monitoring
- Long-running streaming sessions
- Interactive applications

**Use NDJSON/SSE when you need:**
- Simpler unidirectional streaming
- Better compatibility with HTTP tooling
- Easier debugging with curl/httpie
- Lower overhead for short streams

## Protocol Details

### Connection Lifecycle

1. **Connect**: Client initiates WebSocket connection to `/crawl/ws`
2. **Welcome**: Server sends welcome message with session info
3. **Request**: Client sends crawl request as JSON
4. **Streaming**: Server streams results as they complete
5. **Summary**: Server sends final summary
6. **Close**: Connection closes gracefully

### Keepalive Mechanism

- Server sends ping every 30 seconds
- Client must respond with pong
- Connection closed if pong timeout (10 seconds)
- Automatic health tracking

### Backpressure Handling

The server monitors client responsiveness:
- Tracks message send times
- Detects slow clients
- Drops messages if necessary
- Terminates unhealthy connections

### Error Recovery

```python
from riptide_sdk.exceptions import StreamingError

try:
    async for result in client.streaming.crawl_websocket(urls):
        # Process results
        pass

except StreamingError as e:
    # WebSocket-specific error
    print(f"Streaming error: {e}")

except ConnectionError as e:
    # Network error
    print(f"Connection error: {e}")

except asyncio.TimeoutError:
    # Timeout error
    print("Connection timeout")
```

## Best Practices

### 1. Always Use Context Managers

```python
# Good
async with AsyncRipTideClient() as client:
    async for result in client.streaming.crawl_websocket(urls):
        pass

# Bad - may leak connections
client = AsyncRipTideClient()
async for result in client.streaming.crawl_websocket(urls):
    pass
```

### 2. Handle All Message Types

```python
async for result in client.streaming.crawl_websocket(urls):
    match result.event_type:
        case "welcome":
            # Log connection
            pass
        case "metadata":
            # Initialize processing
            pass
        case "result":
            # Process URL result
            pass
        case "summary":
            # Finalize and cleanup
            pass
        case "error":
            # Handle errors
            pass
```

### 3. Use Callbacks for Real-time Processing

```python
async def process_result(result):
    """Process results immediately as they arrive."""
    if result.event_type == "result":
        # Save to database, trigger actions, etc.
        await save_result(result.data["result"])

async for result in client.streaming.crawl_websocket(
    urls,
    on_message=process_result
):
    # Results also available here
    pass
```

### 4. Implement Proper Error Handling

```python
from riptide_sdk.exceptions import ValidationError, StreamingError

try:
    if not urls:
        raise ValidationError("URLs required")

    async for result in client.streaming.crawl_websocket(urls):
        if result.event_type == "error":
            # Handle server-side errors
            print(f"Server error: {result.data}")

except ValidationError as e:
    # Input validation failed
    print(f"Invalid input: {e}")

except StreamingError as e:
    # WebSocket streaming failed
    print(f"Streaming failed: {e}")
```

### 5. Monitor Connection Health

```python
# Periodic health checks
while True:
    try:
        ping_result = await client.streaming.ping_websocket()
        if ping_result['latency_ms'] > 1000:
            print("Warning: High latency detected")
    except Exception as e:
        print(f"Health check failed: {e}")
        break

    await asyncio.sleep(30)
```

## Troubleshooting

### Common Issues

#### 1. ImportError: websockets not installed

```bash
pip install websockets
```

#### 2. Connection Timeout

```python
# Increase timeout in crawl options
options = CrawlOptions(timeout_secs=60)
async for result in client.streaming.crawl_websocket(urls, options):
    pass
```

#### 3. Connection Closed Unexpectedly

Check server logs and ensure:
- Server is running and accessible
- No firewall blocking WebSocket connections
- Client is responding to pings

#### 4. Slow Performance

- Reduce concurrency in options
- Check network latency with `ping_websocket()`
- Monitor backpressure events

## API Reference

### `crawl_websocket(urls, options=None, on_message=None)`

Stream crawl results via WebSocket.

**Parameters:**
- `urls` (List[str]): URLs to crawl
- `options` (CrawlOptions, optional): Crawl configuration
- `on_message` (Callable, optional): Async callback for each message

**Yields:**
- `StreamingResult`: Results as they arrive

**Raises:**
- `ValidationError`: Invalid input
- `StreamingError`: WebSocket error
- `ImportError`: websockets not installed

### `ping_websocket(websocket_url=None)`

Test WebSocket connection and measure latency.

**Parameters:**
- `websocket_url` (str, optional): Custom WebSocket URL

**Returns:**
- `Dict`: Ping result with latency and session info

### `get_websocket_status(websocket_url=None)`

Get connection health information.

**Parameters:**
- `websocket_url` (str, optional): Custom WebSocket URL

**Returns:**
- `Dict`: Connection status and metrics

## Examples

See `examples/websocket_streaming_example.py` for comprehensive examples including:

1. Basic WebSocket streaming
2. Callbacks for real-time processing
3. Error handling and retry logic
4. Connection health monitoring
5. Performance comparisons
6. Graceful shutdown
7. Advanced features

Run the examples:

```bash
python examples/websocket_streaming_example.py
```

## Support

For issues or questions:
- GitHub Issues: https://github.com/yourusername/riptide-sdk/issues
- Documentation: https://docs.riptide.dev
- API Reference: https://api.riptide.dev/docs
