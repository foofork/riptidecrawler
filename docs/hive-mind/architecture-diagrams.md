# RipTide CLI-API Architecture Diagrams

**Created By:** Hive Mind System Architect
**Date:** 2025-10-17
**Related:** [architecture-cli-api-hybrid.md](./architecture-cli-api-hybrid.md)

---

## Execution Mode Decision Tree

```mermaid
graph TD
    A[CLI Command] --> B{CLI Flags?}
    B -->|--direct| C[DirectOnly Mode]
    B -->|--api-only| D[ApiOnly Mode]
    B -->|None| E{RIPTIDE_CLI_MODE?}

    E -->|direct/offline| C
    E -->|api_only/api-only| D
    E -->|api_first/api-first| F[ApiFirst Mode]
    E -->|Not set/Invalid| F

    F --> G{API Health Check}
    G -->|Healthy| H[Execute via API]
    G -->|Unhealthy| I[Fallback to Direct]

    D --> J{API Health Check}
    J -->|Healthy| H
    J -->|Unhealthy| K[Error: API Required]

    C --> L[Execute Direct]

    H --> M{Success?}
    M -->|Yes| N[Return Result]
    M -->|No, Retryable| O{Retry Count < 3?}
    M -->|No, Fatal| P{Fallback Allowed?}

    O -->|Yes| Q[Exponential Backoff]
    O -->|No| P
    Q --> H

    P -->|Yes| I
    P -->|No| R[Return Error]

    I --> L
    L --> N

    K --> R

    style F fill:#90EE90
    style D fill:#FFB6C1
    style C fill:#87CEEB
    style H fill:#FFD700
    style L fill:#DDA0DD
```

---

## API-First Execution Flow

```mermaid
sequenceDiagram
    autonumber

    participant U as User
    participant C as CLI
    participant M as ModeResolver
    participant HC as HealthCheck
    participant AC as APIClient
    participant API as RipTide API
    participant DE as DirectExecutor
    participant O as OutputManager

    U->>C: riptide extract URL
    C->>M: Resolve execution mode
    M-->>C: ApiFirst

    rect rgb(200, 230, 255)
        Note over HC,API: API Availability Check
        C->>HC: Check API health
        HC->>API: GET /health
        alt API Healthy
            API-->>HC: 200 OK
            HC-->>C: Available
        else API Down
            API--xHC: Connection refused
            HC-->>C: Unavailable
            Note over C,DE: Skip to Fallback (step 16)
        end
    end

    rect rgb(255, 230, 200)
        Note over AC,API: API Execution Attempt
        C->>AC: Extract request
        AC->>API: POST /api/v1/extract
        alt Success
            API-->>AC: 200 OK + Result
            AC-->>C: Extraction complete
        else Retryable Error
            API-->>AC: 503 Service Unavailable
            Note over AC: Retry with backoff (3x)
            AC->>API: POST /api/v1/extract
            alt Retry Success
                API-->>AC: 200 OK + Result
                AC-->>C: Extraction complete
            else Max Retries
                API--xAC: Still failing
                AC-->>C: API failed
                Note over C,DE: Proceed to Fallback
            end
        end
    end

    rect rgb(230, 255, 230)
        Note over DE: Fallback to Direct Execution
        C->>DE: Extract (direct mode)
        DE->>DE: Fetch HTML
        DE->>DE: Gate decision (engine)
        DE->>DE: WASM/Headless extract
        DE-->>C: Extraction complete
    end

    C->>O: Save results
    O-->>U: Success + artifact paths
```

---

## Engine Selection Gate

```mermaid
graph TD
    A[Raw HTML Input] --> B{Content Analysis}

    B --> C{JavaScript Framework?}
    C -->|React/Vue/Angular| D[Headless Engine]
    C -->|None detected| E{Content Ratio?}

    E -->|< 10%| D
    E -->|>= 10%| F{WASM Content?}

    F -->|Yes| G[WASM Engine]
    F -->|No| H{Static HTML?}

    H -->|Yes| G
    H -->|Complex| D

    D --> I[Launch Browser]
    G --> J[Load WASM Module]

    I --> K[Execute JavaScript]
    K --> L[Extract Content]

    J --> M[Parse HTML]
    M --> L

    L --> N[Structured Output]

    style D fill:#FFB6C1
    style G fill:#90EE90
    style L fill:#FFD700
```

---

## Retry Logic with Exponential Backoff

```mermaid
graph LR
    A[Request] --> B{Send HTTP}
    B --> C{Success?}

    C -->|2xx| D[Return Response]
    C -->|4xx non-retryable| E[Return Error]
    C -->|Network Error| F{Retry Count < 3?}
    C -->|408/429/5xx| F

    F -->|Yes| G[Increment Counter]
    F -->|No| H{Fallback Allowed?}

    G --> I[Calculate Backoff]
    I --> J[Sleep]
    J --> B

    H -->|Yes| K[Fallback to Direct]
    H -->|No| E

    K --> L[Direct Execution]
    L --> D

    style D fill:#90EE90
    style E fill:#FFB6C1
    style K fill:#FFD700
```

---

## Configuration Priority

```mermaid
graph TD
    A[Start CLI] --> B[Parse CLI Args]
    B --> C[Read Environment Vars]
    C --> D[Load Config File]
    D --> E[Apply Defaults]

    E --> F{Merge Configuration}

    F --> G[CLI Flags]
    F --> H[Environment]
    F --> I[Config File]
    F --> J[Defaults]

    G --> K[Highest Priority]
    H --> L[High Priority]
    I --> M[Low Priority]
    J --> N[Lowest Priority]

    K --> O[Final Config]
    L --> O
    M --> O
    N --> O

    O --> P[Execute Command]

    style G fill:#FF6B6B
    style H fill:#FFD93D
    style I fill:#6BCF7F
    style J fill:#95E1D3
```

---

## Component Dependencies

```mermaid
graph TB
    subgraph CLI Application
        A[main.rs]
        B[execution_mode.rs]
        C[client.rs]
        D[commands/*]
        E[output.rs]
        F[config.rs]
        G[direct_executor.rs]
        H[fallback.rs]
    end

    subgraph External Services
        I[RipTide API]
        J[Redis Cache]
        K[Headless Service]
    end

    subgraph Core Libraries
        L[riptide-extraction]
        M[riptide-headless]
        N[riptide-core]
        O[riptide-workers]
    end

    A --> B
    A --> C
    A --> D
    A --> E
    A --> F

    D --> G
    D --> H

    C --> I
    G --> J
    G --> K
    G --> L
    G --> M

    H --> C
    H --> G

    L --> N
    M --> N
    M --> O

    style A fill:#FF6B6B
    style C fill:#4ECDC4
    style G fill:#95E1D3
    style I fill:#FFD93D
```

---

## Output Directory Structure

```mermaid
graph TD
    A[riptide-output/] --> B[screenshots/]
    A --> C[html/]
    A --> D[pdf/]
    A --> E[reports/]
    A --> F[crawl/]
    A --> G[sessions/]
    A --> H[artifacts/]
    A --> I[temp/]
    A --> J[logs/]
    A --> K[cache/]

    B --> B1[*.png/jpg]
    B --> B2[metadata/]

    C --> C1[*.html]
    C --> C2[metadata/]

    D --> D1[*.pdf]
    D --> D2[metadata/]

    E --> E1[*.json]
    E --> E2[*.md]

    F --> F1[crawl-123/]
    F1 --> F1A[index.json]
    F1 --> F1B[pages/]
    F1 --> F1C[metadata.json]

    G --> G1[session-*.json]

    H --> H1[custom-data.*]

    J --> J1[riptide-cli-*.log]
    J --> J2[errors/]

    K --> K1[http-cache.db]

    style A fill:#FFD93D
    style B fill:#95E1D3
    style E fill:#4ECDC4
```

---

## Authentication Flow

```mermaid
sequenceDiagram
    participant U as User/Config
    participant C as CLI
    participant AC as APIClient
    participant API as RipTide API
    participant DB as API Key Store

    rect rgb(255, 230, 200)
        Note over U,C: Configuration Phase
        U->>C: RIPTIDE_API_KEY
        alt Config File
            U->>C: ~/.riptide/config.toml
        else CLI Flag
            U->>C: --api-key KEY
        end
    end

    rect rgb(200, 230, 255)
        Note over C,API: Authentication
        C->>AC: Create client(url, key)
        AC->>API: Request + Bearer token
        API->>DB: Validate token
        alt Valid Token
            DB-->>API: Authorized
            API-->>AC: 200 OK + Response
        else Invalid Token
            DB-->>API: Unauthorized
            API-->>AC: 401 Unauthorized
            AC-->>C: Auth error
            C-->>U: Error: Check API key
        end
    end
```

---

## Error Handling Strategy

```mermaid
graph TD
    A[CLI Operation] --> B{Error Occurred?}
    B -->|No| C[Success Path]

    B -->|Yes| D{Error Category?}

    D -->|Network| E{Retryable?}
    D -->|API 4xx| F[User Error]
    D -->|API 5xx| E
    D -->|Extraction| G{Engine Available?}
    D -->|Configuration| H[Config Error]

    E -->|Yes| I{Retry Count < 3?}
    E -->|No| J{Fallback Allowed?}

    I -->|Yes| K[Exponential Backoff]
    I -->|No| J

    K --> L[Retry Operation]
    L --> A

    J -->|Yes| M[Fallback to Direct]
    J -->|No| N[Return Error]

    M --> O[Direct Execution]
    O --> P{Success?}
    P -->|Yes| C
    P -->|No| N

    G -->|Yes| Q[Try Alternative Engine]
    G -->|No| N

    Q --> R{Success?}
    R -->|Yes| C
    R -->|No| N

    F --> S[Log + Suggestion]
    H --> S

    S --> N

    C --> T[Output Results]
    N --> U[Exit with Error Code]

    style C fill:#90EE90
    style N fill:#FFB6C1
    style M fill:#FFD700
```

---

## Legend

**Colors:**
- 🟢 Green: Success/Happy path
- 🔴 Pink: Error/Failure path
- 🟡 Yellow: Fallback/Alternative path
- 🔵 Blue: API communication
- 🟣 Purple: Direct execution

**Diagram Types:**
- **Flowcharts**: Decision logic and branching
- **Sequence Diagrams**: Time-ordered interactions
- **Component Diagrams**: System structure
- **State Machines**: Status transitions

---

## Notes

These diagrams visualize the key architectural decisions in the RipTide hybrid CLI-API system:

1. **Execution Mode Decision Tree**: Shows how CLI determines which mode to use
2. **API-First Execution Flow**: Complete sequence with health check, API call, retry, and fallback
3. **Engine Selection Gate**: Logic for choosing WASM vs Headless extraction
4. **Retry Logic**: Exponential backoff implementation
5. **Configuration Priority**: How config values are resolved
6. **Component Dependencies**: System architecture and module relationships
7. **Output Directory**: Organized artifact storage
8. **Authentication Flow**: API key validation process
9. **Error Handling**: Comprehensive error recovery strategy

Refer to [architecture-cli-api-hybrid.md](./architecture-cli-api-hybrid.md) for detailed specifications.
