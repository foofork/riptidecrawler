# SDK & Client Libraries

Official and community SDKs for integrating with RipTide.

## 🚧 Coming Soon

This section is under development. SDKs and client libraries will be available soon for:

- **JavaScript/TypeScript** - Node.js and browser support
- **Python** - Async and sync clients
- **Rust** - Native Rust SDK
- **Go** - Idiomatic Go client
- **Java** - Enterprise Java integration

## 📖 Current Resources

While SDKs are in development, you can use the RipTide API directly:

### API Documentation
- **[REST API Reference](../02-api-reference/README.md)** - Complete endpoint documentation
- **[OpenAPI Specification](../02-api-reference/openapi.yaml)** - Machine-readable API schema
- **[Code Examples](../02-api-reference/examples.md)** - Multi-language examples

### Quick Integration

```javascript
// JavaScript/Node.js Example
const response = await fetch('http://localhost:8080/crawl', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
        urls: ['https://example.com'],
        options: {
            extract_mode: 'article',
            cache_mode: 'read_write'
        }
    })
});

const result = await response.json();
console.log(result.results[0].document);
```

```python
# Python Example
import requests

response = requests.post('http://localhost:8080/crawl', json={
    'urls': ['https://example.com'],
    'options': {
        'extract_mode': 'article',
        'cache_mode': 'read_write'
    }
})

result = response.json()
print(result['results'][0]['document'])
```

```rust
// Rust Example
use reqwest::Client;
use serde_json::json;

let client = Client::new();
let response = client
    .post("http://localhost:8080/crawl")
    .json(&json!({
        "urls": ["https://example.com"],
        "options": {
            "extract_mode": "article",
            "cache_mode": "read_write"
        }
    }))
    .send()
    .await?;

let result = response.json::<CrawlResponse>().await?;
println!("{:?}", result.results[0].document);
```

## 🔧 Generate Your Own Client

Use the OpenAPI specification to generate clients in your preferred language:

```bash
# OpenAPI Generator
openapi-generator generate \
  -i docs/02-api-reference/openapi.yaml \
  -g <language> \
  -o ./client

# Supported languages: typescript, python, rust, go, java, php, ruby, and more
```

## 📚 Related Documentation

- **[API Reference](../02-api-reference/README.md)** - API endpoints and usage
- **[Examples Collection](../02-api-reference/examples.md)** - Working code samples
- **[Getting Started](../00-getting-started/README.md)** - Quick start guide
- **[Streaming Guide](../02-api-reference/streaming.md)** - Real-time protocols

## 🆘 Need Help?

- **[GitHub Issues](https://github.com/your-org/eventmesh/issues)** - Report bugs or request features
- **[Discussions](https://github.com/your-org/eventmesh/discussions)** - Ask questions

---

**Want to contribute?** See [Contributing Guide](../05-development/contributing.md) for SDK development guidelines.
