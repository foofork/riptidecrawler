export function generateCode(endpoint, requestBody, language) {
  if (!endpoint) {
    return '// Select an endpoint to generate code'
  }

  const apiUrl = 'http://localhost:8080'
  const fullUrl = `${apiUrl}${endpoint.path}`
  const body = requestBody || '{}'

  switch (language) {
    case 'javascript':
      return generateJavaScript(endpoint, fullUrl, body)
    case 'python':
      return generatePython(endpoint, fullUrl, body)
    case 'curl':
      return generateCurl(endpoint, fullUrl, body)
    case 'rust':
      return generateRust(endpoint, fullUrl, body)
    default:
      return ''
  }
}

function generateJavaScript(endpoint, url, body) {
  if (endpoint.method === 'GET') {
    return `// Fetch API
const response = await fetch('${url}')
const data = await response.json()
console.log(data)

// Axios
import axios from 'axios'

const { data } = await axios.get('${url}')
console.log(data)`
  }

  return `// Fetch API
const response = await fetch('${url}', {
  method: '${endpoint.method}',
  headers: {
    'Content-Type': 'application/json',
  },
  body: JSON.stringify(${body})
})

const data = await response.json()
console.log(data)

// Axios
import axios from 'axios'

const { data } = await axios.post('${url}', ${body})
console.log(data)`
}

function generatePython(endpoint, url, body) {
  if (endpoint.method === 'GET') {
    return `import requests

response = requests.get('${url}')
data = response.json()
print(data)

# Or with RipTide Python SDK
from riptide import RipTide

client = RipTide('${url.replace(endpoint.path, '')}')
result = client.${endpoint.id.replace(/-/g, '_')}()
print(result)`
  }

  return `import requests
import json

response = requests.post(
    '${url}',
    headers={'Content-Type': 'application/json'},
    json=${body}
)

data = response.json()
print(data)

# Or with RipTide Python SDK
from riptide import RipTide

client = RipTide('${url.replace(endpoint.path, '')}')
result = client.${endpoint.id.replace(/-/g, '_')}(${body})
print(result)`
}

function generateCurl(endpoint, url, body) {
  if (endpoint.method === 'GET') {
    return `curl -X GET '${url}' \\
  -H 'Accept: application/json'

# Pretty print with jq
curl -s '${url}' | jq '.'`
  }

  return `curl -X ${endpoint.method} '${url}' \\
  -H 'Content-Type: application/json' \\
  -d '${body.replace(/\n/g, '').replace(/\s+/g, ' ')}'

# Pretty print with jq
curl -X ${endpoint.method} '${url}' \\
  -H 'Content-Type: application/json' \\
  -d '${body.replace(/\n/g, '').replace(/\s+/g, ' ')}' \\
  | jq '.'`
}

function generateRust(endpoint, url, body) {
  if (endpoint.method === 'GET') {
    return `use reqwest;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let response = reqwest::get("${url}")
        .await?
        .json::<serde_json::Value>()
        .await?;

    println!("{:#?}", response);
    Ok(())
}`
  }

  return `use reqwest;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();

    let response = client
        .post("${url}")
        .json(&json!(${body}))
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    println!("{:#?}", response);
    Ok(())
}`
}
