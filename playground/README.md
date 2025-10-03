# RipTide API Playground

Interactive web-based playground for testing the RipTide API.

## Features

- 🎮 **Interactive Request Builder** - Build and test API requests visually
- 📝 **Request Body Editor** - JSON editor with syntax highlighting
- 📊 **Response Viewer** - View responses with syntax highlighting
- 💻 **Code Generator** - Generate code in JavaScript, Python, cURL, and Rust
- 📚 **Example Gallery** - Ready-to-use code examples for common use cases
- 📖 **Documentation** - Quick access to all RipTide docs

## Quick Start

```bash
# Install dependencies
npm install

# Start development server
npm run dev

# Build for production
npm run build

# Preview production build
npm run preview
```

The playground will be available at `http://localhost:3000`

## Development

### Tech Stack

- **React 18** - UI framework
- **Vite** - Build tool
- **Tailwind CSS** - Styling
- **CodeMirror** - Code editor
- **Zustand** - State management
- **Axios** - HTTP client

### Project Structure

```
playground/
├── src/
│   ├── components/      # Reusable components
│   │   ├── Layout.jsx
│   │   ├── EndpointSelector.jsx
│   │   ├── RequestBuilder.jsx
│   │   └── ResponseViewer.jsx
│   ├── pages/          # Page components
│   │   ├── Playground.jsx
│   │   ├── Examples.jsx
│   │   └── Documentation.jsx
│   ├── hooks/          # Custom hooks
│   │   └── usePlaygroundStore.js
│   ├── utils/          # Utilities
│   │   ├── endpoints.js
│   │   └── codeGenerator.js
│   ├── styles/         # CSS files
│   │   └── index.css
│   ├── App.jsx         # Main app component
│   └── main.jsx        # Entry point
├── public/             # Static assets
├── index.html          # HTML template
├── package.json
├── vite.config.js
└── tailwind.config.js
```

## API Proxy

The playground uses Vite's proxy to forward API requests to the RipTide API:

```javascript
// vite.config.js
proxy: {
  '/api': {
    target: 'http://localhost:8080',
    changeOrigin: true,
    rewrite: (path) => path.replace(/^\/api/, '')
  }
}
```

This means requests to `/api/crawl` in the playground are forwarded to `http://localhost:8080/crawl`

## Deployment

### Docker

```bash
# Build
docker build -t riptide-playground .

# Run
docker run -p 3000:80 riptide-playground
```

### Docker Compose

Add to your `docker-compose.yml`:

```yaml
services:
  playground:
    build: ./playground
    ports:
      - "3000:80"
    depends_on:
      - riptide-api
```

## Configuration

Edit `src/utils/endpoints.js` to add or modify endpoints:

```javascript
export const endpoints = [
  {
    id: 'my-endpoint',
    category: 'Custom',
    name: 'My Endpoint',
    method: 'POST',
    path: '/my-endpoint',
    description: 'Description here',
    defaultBody: {
      // Default request body
    }
  }
]
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Test thoroughly
5. Submit a pull request

## License

MIT License - see LICENSE file for details
