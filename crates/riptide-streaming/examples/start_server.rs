//! Simple server starter for testing

use riptide_streaming::{create_server, ServerState};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt().with_env_filter("info").init();

    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()?;

    println!("Starting RipTide Streaming Server on port {}", port);

    let state = ServerState::new();
    let app = create_server(state);

    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    println!("✓ Server listening on {}", addr);
    println!("  POST /crawl/stream");
    println!("  POST /deepsearch/stream");
    println!("  GET /health");

    axum::serve(listener, app).await?;

    Ok(())
}
