# RipTide Pool

Resource pooling and lifecycle management for the RipTide framework.

## Overview

`riptide-pool` provides efficient resource pooling with lifecycle management, health checks, and automatic cleanup for expensive-to-create instances like browser sessions, database connections, and HTTP clients.

## Features

- **Generic Pooling**: Flexible pool implementation for any resource type
- **Lifecycle Management**: Automatic creation, validation, and cleanup
- **Health Checks**: Periodic health validation for pooled resources
- **Connection Recycling**: Intelligent reuse of healthy connections
- **Min/Max Pool Sizes**: Configurable pool size boundaries
- **Idle Timeout**: Automatic cleanup of stale resources
- **Pool Statistics**: Real-time monitoring of pool usage
- **Thread-Safe**: Concurrent access with tokio synchronization
- **Custom Validators**: Extensible validation logic

## Usage

### Basic Pool Operations

```rust
use riptide_pool::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Create pool
    let pool = Pool::builder()
        .max_size(10)
        .min_size(2)
        .idle_timeout(Duration::from_secs(300))
        .build(|| async {
            // Resource creation logic
            create_browser().await
        })
        .await?;

    // Get resource from pool
    let resource = pool.get().await?;

    // Use resource
    resource.do_work().await?;

    // Return to pool (automatic via Drop)
    drop(resource);

    Ok(())
}
```

### Custom Resource Pool

```rust
use riptide_pool::*;

struct BrowserConnection {
    id: String,
    page: Page,
}

impl Poolable for BrowserConnection {
    async fn is_healthy(&self) -> bool {
        self.page.is_connected().await
    }

    async fn reset(&mut self) -> Result<()> {
        self.page.clear_cookies().await?;
        self.page.goto("about:blank").await?;
        Ok(())
    }
}

let pool = Pool::<BrowserConnection>::builder()
    .max_size(5)
    .validator(Box::new(|conn| async move {
        conn.is_healthy().await
    }))
    .build(|| async {
        BrowserConnection {
            id: uuid::Uuid::new_v4().to_string(),
            page: browser.new_page().await?,
        }
    })
    .await?;
```

### Pool Statistics

```rust
use riptide_pool::*;

let stats = pool.stats().await;

println!("Total: {}", stats.total);
println!("Active: {}", stats.active);
println!("Idle: {}", stats.idle);
println!("Created: {}", stats.created);
println!("Destroyed: {}", stats.destroyed);
println!("Hit rate: {:.2}%", stats.hit_rate * 100.0);
```

## License

Apache-2.0
