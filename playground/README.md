# RipTide API Playground

<div align="center">

![RipTide Playground](https://via.placeholder.com/800x400/4F46E5/FFFFFF?text=RipTide+API+Playground)

**Interactive web-based playground for testing the RipTide API**

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![React](https://img.shields.io/badge/React-18.2-61DAFB?logo=react)](https://reactjs.org/)
[![Vite](https://img.shields.io/badge/Vite-5.0-646CFF?logo=vite)](https://vitejs.dev/)
[![Tailwind CSS](https://img.shields.io/badge/Tailwind-3.3-38B2AC?logo=tailwind-css)](https://tailwindcss.com/)

[Live Demo](#) · [Documentation](#documentation) · [Report Bug](https://github.com/yourusername/riptide/issues) · [Request Feature](https://github.com/yourusername/riptide/issues)

</div>

---

## 📋 Table of Contents

- [Overview](#overview)
- [Features](#features)
- [Screenshots](#screenshots)
- [Getting Started](#getting-started)
  - [Prerequisites](#prerequisites)
  - [Installation](#installation)
  - [First-Time User Guide](#first-time-user-guide)
- [Component Documentation](#component-documentation)
- [Usage](#usage)
- [Development](#development)
- [Configuration](#configuration)
- [Deployment](#deployment)
- [Testing](#testing)
- [Troubleshooting](#troubleshooting)
- [Browser Compatibility](#browser-compatibility)
- [Roadmap](#roadmap)
- [Contributing](#contributing)
- [License](#license)

---

## 🌟 Overview

RipTide API Playground is a powerful, interactive web interface designed to help developers explore, test, and integrate with the RipTide web crawler API. Built with modern web technologies, it provides a seamless experience for crafting API requests, viewing responses, and generating client code in multiple programming languages.

### Architecture Flow

```
┌─────────────┐      ┌─────────────┐      ┌─────────────┐
│             │      │             │      │             │
│   Browser   │─────▶│ Vite Proxy  │─────▶│ RipTide API │
│   (React)   │      │ :3000/api   │      │   :8080     │
│             │◀─────│             │◀─────│             │
└─────────────┘      └─────────────┘      └─────────────┘
      │                     │                     │
      │                     │                     │
   Zustand              CORS/Proxy          Actual API
    State               Handling            Processing
```

---

## ✨ Features

### 🎮 Interactive Request Builder
Build and customize API requests through an intuitive visual interface. Select endpoints from organized categories, configure parameters, and watch your request take shape in real-time.

### 📝 Advanced JSON Editor
Powered by CodeMirror, featuring:
- Syntax highlighting for JSON, JavaScript, Python, and more
- Auto-completion and validation
- Line numbers and bracket matching
- Error detection and linting

### 📊 Smart Response Viewer
View API responses with:
- Syntax-highlighted JSON output
- Collapsible sections for nested data
- Response time and status indicators
- HTTP headers inspection
- Error message highlighting

### 💻 Multi-Language Code Generator
Generate production-ready code snippets in:
- **JavaScript** (fetch, axios)
- **Python** (requests)
- **cURL** (command line)
- **Rust** (reqwest)

Simply configure your request and get instant code you can copy into your project.

### 📚 Example Gallery
Explore pre-built examples covering:
- Basic web crawling
- Advanced extraction patterns
- Streaming responses
- Error handling scenarios
- Rate limiting and retries

### 📖 Integrated Documentation
Quick access to comprehensive API documentation without leaving the playground.

### 🎨 Modern UI/UX
- Clean, professional interface
- Dark mode support (coming soon)
- Responsive design for all screen sizes
- Keyboard shortcuts for power users

---

## 📸 Screenshots

> **Note:** Screenshots will be added in a future update. See the [Live Demo](#) for current visuals.

### Main Playground Interface
![Playground Main](https://via.placeholder.com/800x500/4F46E5/FFFFFF?text=Coming+Soon)

### Code Generator
![Code Generator](https://via.placeholder.com/800x500/4F46E5/FFFFFF?text=Coming+Soon)

### Example Gallery
![Examples](https://via.placeholder.com/800x500/4F46E5/FFFFFF?text=Coming+Soon)

---

## 🚀 Getting Started

### Prerequisites

Ensure you have the following installed:
- **Node.js** >= 18.0.0
- **npm** >= 9.0.0 or **yarn** >= 1.22.0
- **RipTide API** running on `http://localhost:8080` (or configure proxy)

### Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/riptide.git
cd riptide/playground

# Install dependencies
npm install

# Start development server
npm run dev
```

The playground will be available at `http://localhost:3000`

### Quick Start Commands

```bash
# Install dependencies
npm install

# Start development server (with hot reload)
npm run dev

# Build for production
npm run build

# Preview production build
npm run preview

# Lint code
npm run lint
```

---

## 📚 First-Time User Guide

### Step 1: Launch the Playground
```bash
npm run dev
```
Open your browser to `http://localhost:3000`

### Step 2: Select an Endpoint
1. Navigate to the **Playground** tab
2. Click the **Endpoint Selector** dropdown
3. Choose an endpoint (e.g., "Crawl URL")

### Step 3: Configure Your Request
1. The default request body appears in the JSON editor
2. Modify fields as needed (URL, options, etc.)
3. Watch syntax highlighting ensure valid JSON

### Step 4: Send the Request
1. Click the **Send Request** button
2. View the response in the right panel
3. Check response time and status code

### Step 5: Generate Code
1. Click the **Code** tab in the response panel
2. Select your preferred language
3. Copy the generated code to your clipboard

### Step 6: Explore Examples
1. Navigate to the **Examples** tab
2. Browse categorized code samples
3. Click any example to load it into the playground

---

## 🧩 Component Documentation

### EndpointSelector Component

**Location:** `src/components/EndpointSelector.jsx`

Provides a dropdown interface for selecting API endpoints organized by category.

**Features:**
- Categorized endpoint grouping
- Search/filter functionality
- HTTP method badges (GET, POST, etc.)
- Endpoint descriptions on hover

**Props:**
```javascript
{
  selectedEndpoint: object,      // Currently selected endpoint
  onSelectEndpoint: function,    // Callback when endpoint changes
  endpoints: array              // Available endpoints
}
```

---

### RequestBuilder Component

**Location:** `src/components/RequestBuilder.jsx`

The main request configuration interface with JSON editing capabilities.

**Features:**
- CodeMirror-powered JSON editor
- Real-time syntax validation
- Auto-formatting (Ctrl+Alt+F)
- Line numbers and bracket matching
- Error highlighting

**State Management:**
```javascript
// Zustand store hook
const { requestBody, setRequestBody } = usePlaygroundStore();
```

**Keyboard Shortcuts:**
- `Ctrl+Space` - Auto-complete
- `Ctrl+/` - Toggle comment
- `Ctrl+F` - Find
- `Ctrl+H` - Find and replace

---

### ResponseViewer Component

**Location:** `src/components/ResponseViewer.jsx`

Displays API responses with syntax highlighting and collapsible sections.

**Features:**
- Tabbed interface (Response, Headers, Code)
- Syntax-highlighted JSON
- Response time and status display
- Collapsible nested objects/arrays
- Copy response button

**Performance:**
- Lazy rendering for large responses (>1MB)
- Virtual scrolling for deep object trees
- Debounced search/filter

---

### CodeGenerator Utility

**Location:** `src/utils/codeGenerator.js`

Generates client code in multiple languages based on the current request configuration.

**Supported Languages:**

| Language   | Libraries         | Output Format          |
|------------|-------------------|------------------------|
| JavaScript | fetch, axios      | ES6+ with async/await  |
| Python     | requests          | Python 3.x compatible  |
| cURL       | Native            | Shell command          |
| Rust       | reqwest, tokio    | Async Rust code        |

**Usage Example:**
```javascript
import { generateCode } from '@/utils/codeGenerator';

const code = generateCode({
  endpoint: '/crawl',
  method: 'POST',
  body: { url: 'https://example.com' },
  language: 'javascript'
});
```

---

## 🎯 Usage

### Basic API Request

```javascript
// Select "Crawl URL" endpoint
// Modify request body:
{
  "url": "https://example.com",
  "options": {
    "depth": 2,
    "follow_redirects": true
  }
}

// Click "Send Request"
// View response with extracted data
```

### Using Generated Code

```python
# Generate Python code from playground
import requests

url = "http://localhost:8080/crawl"
payload = {
    "url": "https://example.com",
    "options": {
        "depth": 2,
        "follow_redirects": True
    }
}

response = requests.post(url, json=payload)
print(response.json())
```

---

## 💻 Development

### Tech Stack

| Category          | Technology            | Version | Purpose                    |
|-------------------|-----------------------|---------|----------------------------|
| **Framework**     | React                 | 18.2    | UI component library       |
| **Build Tool**    | Vite                  | 5.0     | Fast dev server & bundler  |
| **Styling**       | Tailwind CSS          | 3.3     | Utility-first CSS          |
| **Code Editor**   | CodeMirror            | 6.x     | Advanced code editing      |
| **State**         | Zustand               | 4.4     | Lightweight state mgmt     |
| **HTTP Client**   | Axios                 | 1.6     | Promise-based HTTP client  |
| **Routing**       | React Router          | 6.20    | Client-side routing        |
| **Icons**         | React Icons           | 4.12    | Icon library               |

### Project Structure

```
playground/
├── src/
│   ├── components/          # Reusable UI components
│   │   ├── Layout.jsx       # Main layout wrapper
│   │   ├── EndpointSelector.jsx   # Endpoint dropdown
│   │   ├── RequestBuilder.jsx     # Request editor
│   │   ├── ResponseViewer.jsx     # Response display
│   │   └── CodeGenerator.jsx      # Code generation UI
│   ├── pages/              # Page-level components
│   │   ├── Playground.jsx  # Main playground page
│   │   ├── Examples.jsx    # Examples gallery
│   │   └── Documentation.jsx      # API docs
│   ├── hooks/              # Custom React hooks
│   │   └── usePlaygroundStore.js  # Zustand store
│   ├── utils/              # Helper functions
│   │   ├── endpoints.js    # Endpoint definitions
│   │   ├── codeGenerator.js       # Code generation logic
│   │   └── api.js          # API client wrapper
│   ├── styles/             # Global styles
│   │   └── index.css       # Tailwind imports & custom CSS
│   ├── App.jsx             # Root component
│   └── main.jsx            # Application entry point
├── public/                 # Static assets
│   └── favicon.ico
├── index.html              # HTML template
├── package.json            # Dependencies & scripts
├── vite.config.js          # Vite configuration
├── tailwind.config.js      # Tailwind configuration
├── postcss.config.js       # PostCSS configuration
└── README.md               # This file
```

### State Management with Zustand

The playground uses Zustand for lightweight, efficient state management:

```javascript
// src/hooks/usePlaygroundStore.js
import { create } from 'zustand';

const usePlaygroundStore = create((set) => ({
  // State
  selectedEndpoint: null,
  requestBody: {},
  response: null,
  loading: false,

  // Actions
  setSelectedEndpoint: (endpoint) => set({ selectedEndpoint: endpoint }),
  setRequestBody: (body) => set({ requestBody: body }),
  setResponse: (response) => set({ response }),
  setLoading: (loading) => set({ loading })
}));

export default usePlaygroundStore;
```

**Benefits:**
- No provider boilerplate
- Minimal re-renders
- Simple API
- DevTools support

---

## ⚙️ Configuration

### API Proxy Configuration

The playground uses Vite's built-in proxy to forward API requests:

```javascript
// vite.config.js
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

export default defineConfig({
  plugins: [react()],
  server: {
    port: 3000,
    proxy: {
      '/api': {
        target: 'http://localhost:8080',
        changeOrigin: true,
        rewrite: (path) => path.replace(/^\/api/, '')
      }
    }
  }
});
```

**How it works:**
- Browser requests: `/api/crawl`
- Vite rewrites to: `http://localhost:8080/crawl`
- Response proxied back to browser

This avoids CORS issues during development.

### Environment Variables

Create a `.env` file in the playground directory:

```bash
# API Configuration
VITE_API_BASE_URL=http://localhost:8080
VITE_API_TIMEOUT=30000

# Feature Flags
VITE_ENABLE_CODE_GENERATOR=true
VITE_ENABLE_EXAMPLES=true

# Analytics (optional)
VITE_ANALYTICS_ID=your-analytics-id
```

Access in code:
```javascript
const API_URL = import.meta.env.VITE_API_BASE_URL;
```

### Adding Custom Endpoints

Edit `src/utils/endpoints.js`:

```javascript
export const endpoints = [
  {
    id: 'my-custom-endpoint',
    category: 'Custom',
    name: 'My Custom Endpoint',
    method: 'POST',
    path: '/my-endpoint',
    description: 'Description of what this endpoint does',
    defaultBody: {
      param1: 'value1',
      param2: {
        nested: 'value2'
      }
    },
    // Optional: response example
    exampleResponse: {
      status: 'success',
      data: {}
    }
  }
];
```

---

## 🚢 Deployment

### Docker

```bash
# Build image
docker build -t riptide-playground .

# Run container
docker run -p 3000:80 riptide-playground

# Access at http://localhost:3000
```

**Dockerfile:**
```dockerfile
FROM node:18-alpine AS build
WORKDIR /app
COPY package*.json ./
RUN npm ci
COPY . .
RUN npm run build

FROM nginx:alpine
COPY --from=build /app/dist /usr/share/nginx/html
COPY nginx.conf /etc/nginx/nginx.conf
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
```

### Docker Compose

Add to your `docker-compose.yml`:

```yaml
version: '3.8'

services:
  riptide-api:
    image: riptide-api:latest
    ports:
      - "8080:8080"
    environment:
      - RUST_LOG=info

  playground:
    build: ./playground
    ports:
      - "3000:80"
    depends_on:
      - riptide-api
    environment:
      - VITE_API_BASE_URL=http://riptide-api:8080
```

### Production Build

```bash
# Build optimized production bundle
npm run build

# Output in dist/ directory
# Deploy dist/ to your hosting provider
```

**Build Optimizations:**
- Code splitting
- Tree shaking
- Minification
- Gzip compression
- Asset hashing

---

## 🧪 Testing

### Unit Tests

```bash
# Run unit tests
npm run test

# Run with coverage
npm run test:coverage

# Watch mode
npm run test:watch
```

**Example Test:**
```javascript
// src/utils/codeGenerator.test.js
import { generateCode } from './codeGenerator';

describe('Code Generator', () => {
  test('generates valid JavaScript fetch code', () => {
    const code = generateCode({
      endpoint: '/crawl',
      method: 'POST',
      body: { url: 'https://example.com' },
      language: 'javascript'
    });

    expect(code).toContain('fetch');
    expect(code).toContain('POST');
    expect(code).toContain('example.com');
  });
});
```

### End-to-End Tests

```bash
# Run E2E tests (Playwright/Cypress)
npm run test:e2e
```

### Linting

```bash
# Check for code issues
npm run lint

# Auto-fix issues
npm run lint:fix
```

---

## 🐛 Troubleshooting

### Common Issues

#### Issue: CORS Errors

**Symptom:** Browser console shows "CORS policy" errors

**Solution:**
1. Ensure RipTide API is running on `http://localhost:8080`
2. Check Vite proxy configuration in `vite.config.js`
3. Verify API endpoint paths in `src/utils/endpoints.js`

```javascript
// Correct proxy configuration
proxy: {
  '/api': {
    target: 'http://localhost:8080',
    changeOrigin: true,  // Important!
    rewrite: (path) => path.replace(/^\/api/, '')
  }
}
```

#### Issue: Proxy Connection Refused

**Symptom:** `ERR_CONNECTION_REFUSED` or `ECONNREFUSED`

**Solution:**
1. Verify RipTide API is running: `curl http://localhost:8080/health`
2. Check if port 8080 is in use: `lsof -i :8080`
3. Update proxy target URL if API runs on different port

#### Issue: JSON Syntax Errors

**Symptom:** Red highlights in request body editor

**Solution:**
1. Check for missing commas, quotes, or brackets
2. Use auto-format: Select all (Ctrl+A) → Format (Ctrl+Alt+F)
3. Validate JSON externally: [jsonlint.com](https://jsonlint.com)

#### Issue: Large Response Freezing UI

**Symptom:** Browser becomes unresponsive with large API responses

**Solution:**
- Responses >1MB are automatically paginated
- Enable virtual scrolling in `ResponseViewer` settings
- Use streaming endpoints for large data transfers

#### Issue: Code Generator Not Working

**Symptom:** Generated code is empty or incorrect

**Solution:**
1. Ensure request body is valid JSON
2. Check console for JavaScript errors
3. Verify endpoint configuration in `src/utils/endpoints.js`

### Debug Mode

Enable verbose logging:

```bash
# Set environment variable
VITE_DEBUG=true npm run dev
```

Check browser console for detailed logs.

---

## 🌐 Browser Compatibility

| Browser           | Version     | Status             | Notes                    |
|-------------------|-------------|--------------------|--------------------------|
| Chrome            | >= 90       | ✅ Fully Supported | Recommended              |
| Firefox           | >= 88       | ✅ Fully Supported | Recommended              |
| Safari            | >= 14       | ✅ Fully Supported | iOS 14.5+                |
| Edge (Chromium)   | >= 90       | ✅ Fully Supported | Windows/macOS            |
| Opera             | >= 76       | ✅ Fully Supported | -                        |
| Brave             | >= 1.24     | ✅ Fully Supported | -                        |
| IE 11             | -           | ❌ Not Supported   | Use modern browser       |

**Required Browser Features:**
- ES6+ JavaScript support
- CSS Grid and Flexbox
- Fetch API
- LocalStorage
- WebSockets (for streaming)

---

## 🎹 Keyboard Shortcuts

### Editor Shortcuts

| Shortcut              | Action                    |
|-----------------------|---------------------------|
| `Ctrl+Space`          | Trigger auto-complete     |
| `Ctrl+/`              | Toggle line comment       |
| `Ctrl+F`              | Find in document          |
| `Ctrl+H`              | Find and replace          |
| `Ctrl+Alt+F`          | Format JSON               |
| `Ctrl+S`              | Send request              |
| `Ctrl+K`              | Clear editor              |

### Navigation Shortcuts

| Shortcut              | Action                    |
|-----------------------|---------------------------|
| `Ctrl+1`              | Go to Playground          |
| `Ctrl+2`              | Go to Examples            |
| `Ctrl+3`              | Go to Documentation       |
| `Ctrl+Enter`          | Send request (from editor)|

---

## 🗺️ Roadmap

### Version 1.1 (Q1 2025)
- [ ] Dark mode theme
- [ ] Request history with local storage
- [ ] Saved request collections
- [ ] WebSocket endpoint support
- [ ] Real-time streaming response viewer

### Version 1.2 (Q2 2025)
- [ ] Authentication flow testing
- [ ] Environment variable management
- [ ] Request chaining and workflows
- [ ] Export/import request collections
- [ ] GraphQL endpoint support

### Version 1.3 (Q3 2025)
- [ ] Collaborative workspace (multi-user)
- [ ] Mock server integration
- [ ] Performance benchmarking tools
- [ ] API documentation generator
- [ ] Custom themes and plugins

### Future Considerations
- AI-powered request suggestions
- Automated test generation
- Integration with CI/CD pipelines
- Mobile app version

---

## 🤝 Contributing

We welcome contributions from the community! Here's how you can help:

### Reporting Bugs

1. Check [existing issues](https://github.com/yourusername/riptide/issues)
2. Create a new issue with:
   - Clear title and description
   - Steps to reproduce
   - Expected vs actual behavior
   - Screenshots (if applicable)
   - Browser and OS information

### Suggesting Features

1. Open a feature request issue
2. Describe the feature and use case
3. Include mockups or examples (optional)

### Pull Requests

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/amazing-feature`
3. Make your changes following code style guidelines
4. Add tests for new functionality
5. Update documentation as needed
6. Commit with clear messages: `git commit -m 'feat: add amazing feature'`
7. Push to your fork: `git push origin feature/amazing-feature`
8. Open a pull request with description of changes

### Development Setup

```bash
# Fork and clone your fork
git clone https://github.com/YOUR_USERNAME/riptide.git
cd riptide/playground

# Add upstream remote
git remote add upstream https://github.com/ORIGINAL_OWNER/riptide.git

# Create feature branch
git checkout -b feature/my-feature

# Install dependencies
npm install

# Start development server
npm run dev

# Make changes and test
npm run lint
npm run test

# Commit and push
git add .
git commit -m "feat: description of feature"
git push origin feature/my-feature
```

### Code Style Guidelines

- Use ES6+ features
- Follow React hooks best practices
- Maintain consistent indentation (2 spaces)
- Add JSDoc comments for functions
- Use Tailwind CSS for styling
- Keep components under 300 lines

---

## 📄 License

MIT License

Copyright (c) 2024 RipTide Contributors

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.

---

## 📞 Support & Contact

- **Documentation:** [Link to full docs]
- **Issues:** [GitHub Issues](https://github.com/yourusername/riptide/issues)
- **Discussions:** [GitHub Discussions](https://github.com/yourusername/riptide/discussions)
- **Email:** support@riptide.dev

---

<div align="center">

**Built with ❤️ by the RipTide team**

[⭐ Star us on GitHub](https://github.com/yourusername/riptide) | [🐦 Follow on Twitter](https://twitter.com/riptide) | [💬 Join Discord](https://discord.gg/riptide)

</div>
