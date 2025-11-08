# Outbox Publisher Design

## Overview

The `OutboxPublisher` is a production-ready background worker that implements the polling and publishing phase of the Transactional Outbox pattern. It reads events from the `event_outbox` table and publishes them to a real message broker (RabbitMQ, Kafka, NATS, etc.).

## Architecture

```
┌──────────────────────────────────────────────────────────────┐
│                     Transactional Outbox                     │
└──────────────────────────────────────────────────────────────┘
         │                                    │
         │                                    │
         ▼                                    ▼
┌─────────────────┐                  ┌─────────────────┐
│ OutboxEventBus  │                  │OutboxPublisher  │
│                 │                  │                 │
│ - Writes events │                  │ - Polls events  │
│   to outbox     │                  │ - Publishes     │
│   table         │                  │ - Handles retry │
│ - Transactional │                  │ - Backoff logic │
│   consistency   │                  │                 │
└─────────────────┘                  └─────────────────┘
         │                                    │
         │                                    │
         ▼                                    ▼
┌───────────────────────────────────────────────────────────┐
│              PostgreSQL event_outbox Table                │
│                                                           │
│  Columns:                                                 │
│  - id: UUID (primary key)                                 │
│  - event_id: TEXT (unique, idempotency key)               │
│  - event_type: TEXT                                       │
│  - aggregate_id: TEXT                                     │
│  - payload: JSONB                                         │
│  - metadata: JSONB                                        │
│  - created_at: TIMESTAMPTZ                                │
│  - published_at: TIMESTAMPTZ (NULL = unpublished)         │
│  - retry_count: INTEGER                                   │
│  - last_error: TEXT                                       │
│  - last_retry_at: TIMESTAMPTZ                             │
│                                                           │
└───────────────────────────────────────────────────────────┘
                             │
                             │
                             ▼
                  ┌──────────────────────┐
                  │  Real Event Bus      │
                  │  (RabbitMQ/Kafka/    │
                  │   NATS/etc.)         │
                  └──────────────────────┘
```

## Key Features

### 1. At-Least-Once Delivery

Events are published with retry logic to ensure delivery:

```rust
// Publisher continues retrying until:
// - Event is successfully published
// - max_retries is exceeded
loop {
    match publish_to_broker(event).await {
        Ok(_) => mark_published(event).await,
        Err(e) => {
            increment_retry_count(event).await;
            record_error(event, e).await;
        }
    }
}
```

### 2. Exponential Backoff

Failed events retry with increasing delays:

```rust
// Backoff formula: min(min_backoff * multiplier^retry_count, max_backoff)
retry_0: 1s
retry_1: 2s
retry_2: 4s
retry_3: 8s
retry_4: 16s
retry_5: 32s
retry_6: 60s (capped at max_backoff)
```

### 3. Concurrent Publisher Safety

Multiple publisher instances can run concurrently using row-level locking:

```sql
SELECT * FROM event_outbox
WHERE published_at IS NULL
  AND retry_count < max_retries
  AND (last_retry_at IS NULL OR last_retry_at < NOW() - backoff_delay)
ORDER BY created_at ASC
LIMIT batch_size
FOR UPDATE SKIP LOCKED;  -- Critical for concurrent safety
```

The `FOR UPDATE SKIP LOCKED` clause ensures:
- Each event is processed by only one publisher instance
- Locked rows are skipped (not blocked)
- Allows horizontal scaling

### 4. Graceful Shutdown

Uses `CancellationToken` for clean termination:

```rust
pub async fn start(&self, cancel_token: CancellationToken) {
    loop {
        tokio::select! {
            _ = cancel_token.cancelled() => {
                info!("Shutdown requested");
                break;
            }
            _ = interval.tick() => {
                self.poll_and_publish().await;
            }
        }
    }
}
```

## Database Schema

### Required Columns

```sql
CREATE TABLE event_outbox (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_id TEXT NOT NULL UNIQUE,           -- Idempotency key
    event_type TEXT NOT NULL,                 -- Event type (e.g., "user.created")
    aggregate_id TEXT NOT NULL,               -- Aggregate root ID
    payload JSONB NOT NULL,                   -- Event data
    metadata JSONB NOT NULL,                  -- Metadata (correlation IDs, etc.)
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    published_at TIMESTAMPTZ,                 -- NULL = unpublished
    retry_count INTEGER NOT NULL DEFAULT 0,   -- For exponential backoff
    last_error TEXT,                          -- Error message for debugging
    last_retry_at TIMESTAMPTZ                 -- For backoff calculation
);
```

### Required Indexes

```sql
-- For efficient polling of unpublished events
CREATE INDEX idx_outbox_unpublished ON event_outbox (created_at)
    WHERE published_at IS NULL;

-- For exponential backoff queries
CREATE INDEX idx_outbox_retry_backoff ON event_outbox (last_retry_at)
    WHERE published_at IS NULL AND retry_count < 10;
```

## Configuration

### Builder Pattern

```rust
use riptide_persistence::adapters::OutboxPublisher;
use std::time::Duration;

let publisher = OutboxPublisher::builder()
    .pool(pg_pool)                         // Required: PostgreSQL connection pool
    .event_bus(rabbitmq_bus)               // Required: Real EventBus implementation
    .table_name("event_outbox")            // Optional: Custom table name
    .poll_interval(Duration::from_secs(5)) // Optional: Polling frequency
    .batch_size(100)                       // Optional: Events per poll
    .max_retries(5)                        // Optional: Max retry attempts
    .min_backoff(Duration::from_secs(1))   // Optional: Initial backoff delay
    .max_backoff(Duration::from_secs(300)) // Optional: Max backoff delay (5 min)
    .backoff_multiplier(2.0)               // Optional: Exponential multiplier
    .build();
```

### Recommended Settings

| Environment | poll_interval | batch_size | max_retries | max_backoff |
|-------------|---------------|------------|-------------|-------------|
| Development | 5s            | 50         | 3           | 60s         |
| Staging     | 5s            | 100        | 5           | 300s        |
| Production  | 5s            | 100        | 10          | 600s        |
| High-volume | 1s            | 500        | 10          | 600s        |

## Usage Examples

### Basic Usage

```rust
use riptide_persistence::adapters::{OutboxEventBus, OutboxPublisher};
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

// Setup
let pool = Arc::new(PgPool::connect(&database_url).await?);
let event_bus = Arc::new(RabbitMQEventBus::new(rabbitmq_url).await?);

// Create outbox for writing events
let outbox = OutboxEventBus::new(pool.clone());

// Create publisher for background processing
let publisher = OutboxPublisher::builder()
    .pool(pool)
    .event_bus(event_bus)
    .build();

// Run publisher in background
let cancel_token = CancellationToken::new();
let cancel_clone = cancel_token.clone();

tokio::spawn(async move {
    publisher.start(cancel_clone).await;
});

// Publish events (in application code)
let event = DomainEvent::new("user.created", "user-123", json!({"email": "test@example.com"}));
outbox.publish(event).await?;

// Shutdown gracefully
cancel_token.cancel();
```

### Multiple Publishers (Horizontal Scaling)

```rust
// Run multiple publisher instances for high availability
for i in 0..3 {
    let publisher = OutboxPublisher::builder()
        .pool(pool.clone())
        .event_bus(event_bus.clone())
        .build();

    let cancel = cancel_token.clone();

    tokio::spawn(async move {
        tracing::info!("Starting publisher instance {}", i);
        publisher.start(cancel).await;
    });
}

// All instances safely process events concurrently
// using row-level locking (FOR UPDATE SKIP LOCKED)
```

### Custom Error Handling

```rust
// Implement custom EventBus with retry logic
struct RetryEventBus {
    inner: Arc<dyn EventBus>,
    max_attempts: usize,
}

#[async_trait]
impl EventBus for RetryEventBus {
    async fn publish(&self, event: DomainEvent) -> RiptideResult<()> {
        let mut attempts = 0;

        loop {
            match self.inner.publish(event.clone()).await {
                Ok(_) => return Ok(()),
                Err(e) if attempts < self.max_attempts => {
                    attempts += 1;
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
                Err(e) => return Err(e),
            }
        }
    }
}
```

## Monitoring

### Key Metrics to Track

1. **Publishing Rate**: Events published per second
2. **Retry Rate**: Percentage of events requiring retry
3. **Backlog Size**: Number of unpublished events
4. **Error Rate**: Failed publishes per second
5. **Latency**: Time from event creation to publish

### Example Monitoring Query

```sql
-- Outbox health metrics
SELECT
    COUNT(*) FILTER (WHERE published_at IS NULL) AS unpublished_count,
    COUNT(*) FILTER (WHERE retry_count > 0) AS retry_count,
    COUNT(*) FILTER (WHERE retry_count >= 5) AS max_retry_count,
    AVG(EXTRACT(EPOCH FROM (published_at - created_at))) AS avg_publish_latency_secs,
    MAX(retry_count) AS max_retry
FROM event_outbox
WHERE created_at > NOW() - INTERVAL '1 hour';
```

## Error Handling

### Common Errors

1. **Network Failures**: Temporary broker unavailability
   - Solution: Automatic retry with exponential backoff

2. **Serialization Errors**: Invalid event payload
   - Solution: Log error, increment retry count, alert ops

3. **Max Retries Exceeded**: Persistent failure
   - Solution: Log to dead letter queue, alert ops, manual intervention

4. **Database Deadlocks**: Concurrent publisher conflicts
   - Solution: Use `FOR UPDATE SKIP LOCKED`, reduce batch size

### Dead Letter Queue

Events exceeding `max_retries` should be moved to a dead letter queue:

```sql
-- Move failed events to dead letter queue
INSERT INTO event_outbox_dlq
SELECT * FROM event_outbox
WHERE retry_count >= max_retries
  AND published_at IS NULL;

-- Archive or delete from main table
DELETE FROM event_outbox
WHERE retry_count >= max_retries
  AND published_at IS NULL;
```

## Performance Considerations

### Scaling Strategies

1. **Vertical**: Increase batch size, decrease poll interval
2. **Horizontal**: Run multiple publisher instances
3. **Partitioning**: Shard outbox table by aggregate_id
4. **Cleanup**: Regularly delete published events

### Cleanup Strategy

```sql
-- Delete published events older than 7 days
DELETE FROM event_outbox
WHERE published_at < NOW() - INTERVAL '7 days';

-- Or archive to cold storage
INSERT INTO event_outbox_archive
SELECT * FROM event_outbox
WHERE published_at < NOW() - INTERVAL '30 days';

DELETE FROM event_outbox
WHERE published_at < NOW() - INTERVAL '30 days';
```

## Testing

See `tests/outbox_publisher_tests.rs` for comprehensive integration tests:

- ✅ Basic polling and publishing
- ✅ Retry with exponential backoff
- ✅ Concurrent publisher safety
- ✅ Max retries exceeded handling
- ✅ Batch processing
- ✅ Graceful shutdown

## Best Practices

1. **Idempotency**: Ensure downstream consumers handle duplicate events
2. **Monitoring**: Track backlog size and error rates
3. **Cleanup**: Regularly delete old published events
4. **Alerting**: Alert when backlog exceeds threshold
5. **Testing**: Test failure scenarios (broker down, network issues)
6. **Scaling**: Run multiple publishers for high availability

## References

- [Transactional Outbox Pattern](https://microservices.io/patterns/data/transactional-outbox.html)
- [PostgreSQL Row-Level Locking](https://www.postgresql.org/docs/current/explicit-locking.html)
- [Exponential Backoff](https://en.wikipedia.org/wiki/Exponential_backoff)
