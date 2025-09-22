#[cfg(test)]
mod tests {
    use reqwest::Client;
    use serde_json::json;

    #[tokio::test]
    async fn test_health_endpoint() {
        let client = Client::new();
        let res = client
            .get("http://localhost:8080/healthz")
            .send()
            .await;

        if let Ok(response) = res {
            assert_eq!(response.status(), 200);
            let body: serde_json::Value = response.json().await.unwrap();
            assert_eq!(body["status"], "ok");
        }
    }

    #[tokio::test]
    async fn test_crawl_endpoint() {
        let client = Client::new();
        let res = client
            .post("http://localhost:8080/crawl")
            .json(&json!({
                "urls": ["https://example.com"]
            }))
            .send()
            .await;

        if let Ok(response) = res {
            assert_eq!(response.status(), 200);
            let body: serde_json::Value = response.json().await.unwrap();
            assert!(body["received"].is_number());
        }
    }

    #[tokio::test]
    async fn test_render_endpoint() {
        let client = Client::new();
        let res = client
            .post("http://localhost:9123/render")
            .json(&json!({
                "url": "https://example.com",
                "scroll_steps": 2
            }))
            .send()
            .await;

        if let Ok(response) = res {
            assert_eq!(response.status(), 200);
            let body: serde_json::Value = response.json().await.unwrap();
            assert!(body["html"].is_string());
            assert!(body["final_url"].is_string());
        }
    }
}