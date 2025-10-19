# Google Vertex AI OAuth Authentication Guide

**Provider:** Google Vertex AI (Gemini)
**Status:** ✅ Production Ready
**Last Updated:** 2025-10-06

---

## Overview

Google Vertex AI requires OAuth 2.0 authentication using Google Cloud credentials. This guide covers all authentication methods for production and development environments.

## Prerequisites

1. **Google Cloud Project**: Active GCP project with billing enabled
2. **Vertex AI API**: Enabled in your project
3. **Permissions**: `Vertex AI User` role (minimum)

## Authentication Methods

### Method 1: Service Account (Recommended for Production)

#### Step 1: Create Service Account

```bash
# Set your project ID
export PROJECT_ID="your-gcp-project-id"

# Create service account
gcloud iam service-accounts create riptide-vertex-sa \
    --display-name="RipTide Vertex AI Service Account" \
    --project=$PROJECT_ID

# Get the service account email
export SA_EMAIL="riptide-vertex-sa@${PROJECT_ID}.iam.gserviceaccount.com"
```

#### Step 2: Grant Permissions

```bash
# Grant Vertex AI User role
gcloud projects add-iam-policy-binding $PROJECT_ID \
    --member="serviceAccount:${SA_EMAIL}" \
    --role="roles/aiplatform.user"
```

#### Step 3: Create and Download Key

```bash
# Create key file
gcloud iam service-accounts keys create vertex-ai-key.json \
    --iam-account=$SA_EMAIL

# Store key securely
export GOOGLE_APPLICATION_CREDENTIALS="$(pwd)/vertex-ai-key.json"
```

#### Step 4: Use in RipTide

```rust
use riptide_intelligence::providers::VertexAIProvider;

// Get access token programmatically
use google_auth::TokenGenerator;

async fn get_vertex_token() -> Result<String> {
    let token_generator = google_auth::TokenGenerator::new(
        google_auth::Credentials::find_default().await?
    )?;

    let scopes = &["https://www.googleapis.com/auth/cloud-platform"];
    let token = token_generator.token(scopes).await?;

    Ok(token.access_token)
}

// Create provider with service account token
let access_token = get_vertex_token().await?;
let provider = VertexAIProvider::new(
    "your-gcp-project-id".to_string(),
    "us-central1".to_string(),
)
.with_access_token(access_token);
```

### Method 2: Application Default Credentials (Development)

#### Setup

```bash
# Authenticate with your Google account
gcloud auth application-default login

# Verify authentication
gcloud auth application-default print-access-token
```

#### Use in RipTide

```rust
// Same as Method 1 - credentials are automatically discovered
let access_token = get_vertex_token().await?;
let provider = VertexAIProvider::new(
    "your-gcp-project-id".to_string(),
    "us-central1".to_string(),
)
.with_access_token(access_token);
```

### Method 3: Manual Token (Testing)

```bash
# Get temporary access token
gcloud auth print-access-token

# Export for testing
export VERTEX_AI_TOKEN="ya29.c.b0Aaek..."
```

```rust
use std::env;

let access_token = env::var("VERTEX_AI_TOKEN")?;
let provider = VertexAIProvider::new(
    "your-gcp-project-id".to_string(),
    "us-central1".to_string(),
)
.with_access_token(access_token);
```

## Token Refresh Strategy

OAuth tokens expire after 1 hour. Implement automatic refresh:

```rust
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

pub struct TokenManager {
    token: RwLock<String>,
    expires_at: RwLock<Instant>,
    token_generator: google_auth::TokenGenerator,
}

impl TokenManager {
    pub async fn new() -> Result<Self> {
        let credentials = google_auth::Credentials::find_default().await?;
        let token_generator = google_auth::TokenGenerator::new(credentials)?;

        let scopes = &["https://www.googleapis.com/auth/cloud-platform"];
        let token = token_generator.token(scopes).await?;

        Ok(Self {
            token: RwLock::new(token.access_token.clone()),
            expires_at: RwLock::new(Instant::now() + Duration::from_secs(3600)),
            token_generator,
        })
    }

    pub async fn get_token(&self) -> Result<String> {
        // Check if token is expired
        if Instant::now() >= *self.expires_at.read().await {
            // Refresh token
            let scopes = &["https://www.googleapis.com/auth/cloud-platform"];
            let new_token = self.token_generator.token(scopes).await?;

            *self.token.write().await = new_token.access_token.clone();
            *self.expires_at.write().await = Instant::now() + Duration::from_secs(3600);
        }

        Ok(self.token.read().await.clone())
    }
}

// Use in application
let token_manager = Arc::new(TokenManager::new().await?);
let access_token = token_manager.get_token().await?;
```

## Configuration Examples

### Environment Variables

```bash
# .env file
GOOGLE_APPLICATION_CREDENTIALS=/path/to/vertex-ai-key.json
GCP_PROJECT_ID=your-gcp-project-id
GCP_LOCATION=us-central1
```

### Configuration File

```toml
# config.toml
[intelligence]
provider = "google_vertex"
model = "gemini-1.5-pro"
project_id = "${GCP_PROJECT_ID}"
location = "${GCP_LOCATION}"
```

### Rust Code

```rust
use riptide_intelligence::{
    providers::VertexAIProvider,
    CompletionRequest, Message, Role
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load configuration
    let project_id = std::env::var("GCP_PROJECT_ID")?;
    let location = std::env::var("GCP_LOCATION")?;

    // Get access token
    let token_manager = TokenManager::new().await?;
    let access_token = token_manager.get_token().await?;

    // Create provider
    let provider = VertexAIProvider::new(project_id, location)
        .with_access_token(access_token);

    // Health check
    provider.health_check().await?;
    println!("✅ Connected to Vertex AI");

    // Make completion request
    let request = CompletionRequest {
        id: uuid::Uuid::new_v4(),
        model: "gemini-1.5-pro".to_string(),
        messages: vec![
            Message {
                role: Role::User,
                content: "Explain quantum computing in simple terms".to_string(),
            }
        ],
        max_tokens: Some(1024),
        temperature: Some(0.7),
        ..Default::default()
    };

    let response = provider.complete(request).await?;
    println!("Response: {}", response.content);

    Ok(())
}
```

## Security Best Practices

### 1. Never Commit Keys

```bash
# Add to .gitignore
vertex-ai-key.json
.env
*.pem
*.key
```

### 2. Use Secret Managers

#### Google Secret Manager

```bash
# Store service account key
gcloud secrets create vertex-ai-key \
    --data-file=vertex-ai-key.json \
    --replication-policy=automatic

# Grant access
gcloud secrets add-iam-policy-binding vertex-ai-key \
    --member="serviceAccount:${SERVICE_ACCOUNT}" \
    --role="roles/secretmanager.secretAccessor"
```

```rust
// Retrieve from Secret Manager
use google_secretmanager::v1::SecretManagerServiceClient;

async fn get_service_account_key() -> Result<String> {
    let mut client = SecretManagerServiceClient::connect("https://secretmanager.googleapis.com").await?;

    let name = "projects/your-project/secrets/vertex-ai-key/versions/latest";
    let response = client.access_secret_version(name).await?;

    Ok(String::from_utf8(response.payload.data)?)
}
```

#### Kubernetes Secrets

```yaml
apiVersion: v1
kind: Secret
metadata:
  name: vertex-ai-credentials
type: Opaque
stringData:
  key.json: |
    {
      "type": "service_account",
      "project_id": "your-project",
      ...
    }
```

### 3. Principle of Least Privilege

Only grant `roles/aiplatform.user` - avoid broader roles like `owner` or `editor`.

### 4. Rotate Keys Regularly

```bash
# Disable old key
gcloud iam service-accounts keys disable KEY_ID \
    --iam-account=$SA_EMAIL

# Create new key
gcloud iam service-accounts keys create new-key.json \
    --iam-account=$SA_EMAIL
```

## Troubleshooting

### Error: "Invalid authentication credentials"

```bash
# Verify credentials are set
echo $GOOGLE_APPLICATION_CREDENTIALS

# Test authentication
gcloud auth application-default print-access-token

# Refresh if needed
gcloud auth application-default login
```

### Error: "Permission denied"

```bash
# Check IAM permissions
gcloud projects get-iam-policy $PROJECT_ID \
    --flatten="bindings[].members" \
    --filter="bindings.members:serviceAccount:${SA_EMAIL}"

# Grant missing permissions
gcloud projects add-iam-policy-binding $PROJECT_ID \
    --member="serviceAccount:${SA_EMAIL}" \
    --role="roles/aiplatform.user"
```

### Error: "Quota exceeded"

```bash
# Check quota usage
gcloud ai-platform quotas list --filter="metric=aiplatform.googleapis.com/online_prediction_requests"

# Request quota increase in GCP Console:
# Navigation Menu > IAM & Admin > Quotas > Vertex AI
```

### Error: "Token expired"

Implement token refresh (see Token Refresh Strategy section above).

## Monitoring and Costs

### Track API Usage

```bash
# View Vertex AI metrics in Cloud Console
open "https://console.cloud.google.com/monitoring/dashboards/resourceList/aiplatform.googleapis.com"

# Query billing data
gcloud billing accounts list
```

### Cost Optimization

1. **Use Flash models** for lower cost: `gemini-1.5-flash` ($0.000075/$0.0003 per 1K tokens)
2. **Cache responses** to avoid redundant API calls
3. **Monitor quota** to prevent unexpected charges
4. **Set budget alerts** in Billing console

## Additional Resources

- [Google Cloud Authentication](https://cloud.google.com/docs/authentication)
- [Vertex AI Documentation](https://cloud.google.com/vertex-ai/docs)
- [Service Account Best Practices](https://cloud.google.com/iam/docs/best-practices-service-accounts)
- [OAuth 2.0 for Server Applications](https://developers.google.com/identity/protocols/oauth2/service-account)

---

**Version:** 1.0
**Status:** ✅ Production Ready
**Last Updated:** 2025-10-06
