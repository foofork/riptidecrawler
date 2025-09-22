use anyhow::Result;
use reqwest::{Client, Response};
use std::time::Duration;

pub fn http_client() -> Client {
    Client::builder()
        .user_agent("RipTide/1.0")
        .http2_prior_knowledge()
        .gzip(true)
        .brotli(true)
        .connect_timeout(Duration::from_secs(3))
        .timeout(Duration::from_secs(15))
        .build()
        .expect("client")
}

pub async fn get(client: &Client, url: &str) -> Result<Response> {
    let res = client.get(url).send().await?;
    Ok(res.error_for_status()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let _client = http_client();
    }
}
