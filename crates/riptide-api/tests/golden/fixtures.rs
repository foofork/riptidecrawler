/// Test fixtures for golden tests
///
/// This module contains HTML content samples that represent different types
/// of web content for testing extraction capabilities.

/// Simple blog post with clear article structure
pub const BLOG_POST_HTML: &str = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>Understanding WebAssembly: A Comprehensive Guide</title>
    <meta name="author" content="Jane Developer">
    <meta name="description" content="Learn about WebAssembly, its benefits, and how to use it in modern web development.">
    <meta property="og:title" content="Understanding WebAssembly: A Comprehensive Guide">
    <meta property="og:description" content="Learn about WebAssembly, its benefits, and how to use it in modern web development.">
</head>
<body>
    <header>
        <nav>
            <a href="/">Home</a>
            <a href="/blog">Blog</a>
            <a href="/about">About</a>
        </nav>
    </header>

    <main>
        <article>
            <h1>Understanding WebAssembly: A Comprehensive Guide</h1>
            <p class="byline">By <span class="author">Jane Developer</span> | Published on <time datetime="2024-01-15">January 15, 2024</time></p>

            <p>WebAssembly (abbreviated Wasm) is a binary instruction format for a stack-based virtual machine.
            Wasm is designed as a portable compilation target for programming languages, enabling deployment
            on the web for client and server applications.</p>

            <h2>What Makes WebAssembly Special?</h2>
            <p>WebAssembly offers several key advantages over traditional JavaScript:</p>
            <ul>
                <li><strong>Performance</strong>: Near-native performance for compute-intensive operations</li>
                <li><strong>Security</strong>: Runs in a sandboxed execution environment</li>
                <li><strong>Portability</strong>: Can run on multiple platforms and architectures</li>
                <li><strong>Language Support</strong>: Compile from C/C++, Rust, Go, and other languages</li>
            </ul>

            <h2>Getting Started with WebAssembly</h2>
            <p>To begin working with WebAssembly, you'll need to understand a few key concepts:</p>

            <h3>1. Compilation Process</h3>
            <p>WebAssembly modules are typically created by compiling high-level languages like Rust or C++
            into the .wasm binary format. This process involves using specialized toolchains.</p>

            <h3>2. Integration with JavaScript</h3>
            <p>WebAssembly modules can be loaded and executed from JavaScript, allowing you to
            call Wasm functions from JS and vice versa.</p>

            <h3>3. Memory Management</h3>
            <p>WebAssembly has its own linear memory model that can be shared with JavaScript
            for efficient data exchange.</p>

            <p>In conclusion, WebAssembly represents a significant step forward in web development,
            providing developers with new possibilities for building high-performance web applications.</p>

            <p>To learn more about WebAssembly, check out the <a href="https://webassembly.org/">official WebAssembly website</a>
            or explore the <a href="https://developer.mozilla.org/en-US/docs/WebAssembly">MDN WebAssembly documentation</a>.</p>
        </article>
    </main>

    <footer>
        <p>&copy; 2024 Developer Blog. All rights reserved.</p>
    </footer>
</body>
</html>
"#;

/// News article with more complex structure
pub const NEWS_ARTICLE_HTML: &str = r#"
<!DOCTYPE html>
<html>
<head>
    <title>Breakthrough in Quantum Computing Announced by Research Team</title>
    <meta name="description" content="Scientists achieve new milestone in quantum error correction">
    <script type="application/ld+json">
    {
        "@context": "https://schema.org",
        "@type": "NewsArticle",
        "headline": "Breakthrough in Quantum Computing Announced by Research Team",
        "author": {
            "@type": "Person",
            "name": "Dr. Sarah Chen"
        },
        "datePublished": "2024-01-20T09:00:00Z",
        "publisher": {
            "@type": "Organization",
            "name": "Tech News Today"
        }
    }
    </script>
</head>
<body>
    <div class="header">
        <h1 class="site-title">Tech News Today</h1>
        <nav class="main-nav">
            <ul>
                <li><a href="/tech">Technology</a></li>
                <li><a href="/science">Science</a></li>
                <li><a href="/business">Business</a></li>
            </ul>
        </nav>
    </div>

    <div class="content">
        <article class="news-article">
            <header class="article-header">
                <h1>Breakthrough in Quantum Computing Announced by Research Team</h1>
                <div class="article-meta">
                    <span class="author">By Dr. Sarah Chen</span>
                    <span class="publish-date">January 20, 2024</span>
                    <span class="read-time">5 min read</span>
                </div>
            </header>

            <div class="article-body">
                <p class="lead">A team of researchers at the Institute for Advanced Computing has announced
                a significant breakthrough in quantum error correction, potentially bringing practical
                quantum computers closer to reality.</p>

                <p>The research, published today in the journal Nature Quantum Information, demonstrates
                a new approach to error correction that could reduce the number of physical qubits needed
                for fault-tolerant quantum computation by up to 90%.</p>

                <h2>The Challenge of Quantum Error Correction</h2>
                <p>Quantum computers are notoriously fragile. Unlike classical bits that are either 0 or 1,
                quantum bits (qubits) exist in a superposition of both states simultaneously. This quantum
                property is what gives quantum computers their power, but it also makes them extremely
                susceptible to errors from environmental interference.</p>

                <blockquote>
                    <p>"This breakthrough could accelerate the timeline for practical quantum computers
                    from decades to just a few years," said lead researcher Prof. Maria Rodriguez.</p>
                </blockquote>

                <h2>How the New Method Works</h2>
                <p>The team's approach uses a technique called "adaptive error correction" that
                dynamically adjusts the error correction strategy based on real-time analysis
                of the quantum system's behavior.</p>

                <ol>
                    <li>Continuous monitoring of qubit states</li>
                    <li>Real-time error pattern analysis</li>
                    <li>Dynamic adjustment of correction protocols</li>
                    <li>Optimized resource allocation</li>
                </ol>

                <h2>Implications for the Future</h2>
                <p>This development could have far-reaching implications for fields such as:</p>
                <ul>
                    <li>Cryptography and cybersecurity</li>
                    <li>Drug discovery and molecular modeling</li>
                    <li>Financial modeling and risk analysis</li>
                    <li>Climate modeling and weather prediction</li>
                    <li>Artificial intelligence and machine learning</li>
                </ul>

                <p>The research team is now working with several technology companies to
                implement their error correction method in prototype quantum systems.</p>

                <div class="related-links">
                    <h3>Related Articles</h3>
                    <ul>
                        <li><a href="/articles/quantum-computing-basics">Quantum Computing Basics: A Beginner's Guide</a></li>
                        <li><a href="/articles/quantum-vs-classical">Quantum vs Classical Computing: Key Differences</a></li>
                    </ul>
                </div>
            </div>
        </article>

        <aside class="sidebar">
            <div class="author-bio">
                <h3>About the Author</h3>
                <p>Dr. Sarah Chen is a science journalist specializing in quantum physics and emerging technologies.</p>
            </div>
        </aside>
    </div>

    <footer>
        <p>© 2024 Tech News Today. All rights reserved.</p>
    </footer>
</body>
</html>
"#;

/// SPA-style application with minimal content in initial HTML
pub const SPA_APPLICATION_HTML: &str = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>TaskMaster - Project Management Tool</title>
    <meta name="description" content="Powerful project management and task tracking application">
    <script>
        window.__INITIAL_STATE__ = {
            user: { id: 1, name: "John Doe" },
            projects: [
                { id: 1, name: "Website Redesign", tasks: 15, completed: 8 },
                { id: 2, name: "Mobile App", tasks: 23, completed: 12 }
            ]
        };
    </script>
</head>
<body>
    <div id="root">
        <div class="loading-screen">
            <h1>TaskMaster</h1>
            <p>Loading your projects...</p>
            <div class="spinner"></div>
        </div>
    </div>

    <!-- This content would normally be rendered by JavaScript -->
    <noscript>
        <div class="noscript-content">
            <h1>TaskMaster - Project Management Tool</h1>
            <p>This application requires JavaScript to function properly.</p>

            <h2>Key Features</h2>
            <ul>
                <li>Project tracking and management</li>
                <li>Task assignment and collaboration</li>
                <li>Real-time progress monitoring</li>
                <li>Team communication tools</li>
                <li>Advanced reporting and analytics</li>
            </ul>

            <h2>Getting Started</h2>
            <p>TaskMaster helps teams organize, track, and complete projects efficiently.
            Our intuitive interface makes it easy to:</p>
            <ol>
                <li>Create and organize projects</li>
                <li>Break down work into manageable tasks</li>
                <li>Assign tasks to team members</li>
                <li>Track progress in real-time</li>
                <li>Generate comprehensive reports</li>
            </ol>

            <p>Enable JavaScript to access the full TaskMaster experience.</p>
        </div>
    </noscript>

    <script src="/static/js/vendor.js"></script>
    <script src="/static/js/app.js"></script>
    <script>
        // Simulate heavy JavaScript application
        console.log('TaskMaster App Initializing...');

        function initializeApp() {
            const root = document.getElementById('root');
            if (root) {
                // Simulate React-style rendering
                root.innerHTML = `
                    <div class="app-container">
                        <header class="app-header">
                            <h1>TaskMaster</h1>
                            <nav>
                                <a href="/dashboard">Dashboard</a>
                                <a href="/projects">Projects</a>
                                <a href="/team">Team</a>
                            </nav>
                        </header>
                        <main class="app-main">
                            <div class="dashboard">
                                <h2>Welcome back, John!</h2>
                                <div class="project-summary">
                                    <div class="project-card">
                                        <h3>Website Redesign</h3>
                                        <p>8 of 15 tasks completed</p>
                                        <div class="progress-bar">
                                            <div class="progress" style="width: 53%"></div>
                                        </div>
                                    </div>
                                    <div class="project-card">
                                        <h3>Mobile App</h3>
                                        <p>12 of 23 tasks completed</p>
                                        <div class="progress-bar">
                                            <div class="progress" style="width: 52%"></div>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </main>
                    </div>
                `;
            }
        }

        // Simulate app initialization delay
        setTimeout(initializeApp, 100);
    </script>
</body>
</html>
"#;

/// E-commerce product page with structured data
pub const ECOMMERCE_PRODUCT_HTML: &str = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>Premium Wireless Headphones - AudioTech Store</title>
    <meta name="description" content="High-quality wireless headphones with noise cancellation and 30-hour battery life">
    <script type="application/ld+json">
    {
        "@context": "https://schema.org/",
        "@type": "Product",
        "name": "Premium Wireless Headphones",
        "image": "https://example.com/images/headphones.jpg",
        "description": "High-quality wireless headphones with active noise cancellation",
        "brand": {
            "@type": "Brand",
            "name": "AudioTech"
        },
        "offers": {
            "@type": "Offer",
            "price": "299.99",
            "priceCurrency": "USD",
            "availability": "https://schema.org/InStock"
        },
        "aggregateRating": {
            "@type": "AggregateRating",
            "ratingValue": "4.5",
            "reviewCount": "127"
        }
    }
    </script>
</head>
<body>
    <header class="site-header">
        <div class="container">
            <h1 class="logo">AudioTech Store</h1>
            <nav>
                <ul>
                    <li><a href="/headphones">Headphones</a></li>
                    <li><a href="/speakers">Speakers</a></li>
                    <li><a href="/accessories">Accessories</a></li>
                </ul>
            </nav>
        </div>
    </header>

    <main class="product-page">
        <div class="container">
            <div class="product-details">
                <div class="product-images">
                    <img src="/images/headphones-main.jpg" alt="Premium Wireless Headphones" class="main-image">
                    <div class="image-thumbnails">
                        <img src="/images/headphones-side.jpg" alt="Side view">
                        <img src="/images/headphones-folded.jpg" alt="Folded view">
                        <img src="/images/headphones-case.jpg" alt="With carrying case">
                    </div>
                </div>

                <div class="product-info">
                    <h1>Premium Wireless Headphones</h1>
                    <div class="rating">
                        <span class="stars">★★★★★</span>
                        <span class="rating-text">4.5 out of 5 (127 reviews)</span>
                    </div>

                    <div class="price">
                        <span class="current-price">$299.99</span>
                        <span class="original-price">$399.99</span>
                        <span class="discount">25% off</span>
                    </div>

                    <div class="product-description">
                        <h2>Product Description</h2>
                        <p>Experience superior sound quality with our Premium Wireless Headphones.
                        Featuring active noise cancellation technology, these headphones deliver
                        crystal-clear audio while blocking out unwanted background noise.</p>

                        <h3>Key Features</h3>
                        <ul>
                            <li>Active Noise Cancellation (ANC) technology</li>
                            <li>30-hour battery life with quick charge</li>
                            <li>Premium memory foam ear cushions</li>
                            <li>Bluetooth 5.0 connectivity</li>
                            <li>Built-in microphone for calls</li>
                            <li>Foldable design with carrying case</li>
                        </ul>

                        <h3>Technical Specifications</h3>
                        <table class="specs-table">
                            <tr>
                                <td>Driver Size</td>
                                <td>40mm</td>
                            </tr>
                            <tr>
                                <td>Frequency Response</td>
                                <td>20Hz - 20kHz</td>
                            </tr>
                            <tr>
                                <td>Impedance</td>
                                <td>32 ohms</td>
                            </tr>
                            <tr>
                                <td>Weight</td>
                                <td>250g</td>
                            </tr>
                            <tr>
                                <td>Bluetooth Version</td>
                                <td>5.0</td>
                            </tr>
                        </table>
                    </div>

                    <div class="purchase-options">
                        <div class="quantity-selector">
                            <label for="quantity">Quantity:</label>
                            <select id="quantity">
                                <option value="1">1</option>
                                <option value="2">2</option>
                                <option value="3">3</option>
                            </select>
                        </div>

                        <button class="add-to-cart-btn">Add to Cart</button>
                        <button class="buy-now-btn">Buy Now</button>
                    </div>

                    <div class="shipping-info">
                        <p><strong>Free shipping</strong> on orders over $50</p>
                        <p><strong>30-day return policy</strong></p>
                        <p><strong>2-year warranty</strong> included</p>
                    </div>
                </div>
            </div>

            <section class="reviews-section">
                <h2>Customer Reviews</h2>
                <div class="review">
                    <div class="review-header">
                        <span class="reviewer-name">Mike Johnson</span>
                        <span class="review-rating">★★★★★</span>
                        <span class="review-date">January 10, 2024</span>
                    </div>
                    <p class="review-text">Excellent sound quality and the noise cancellation works great!
                    Perfect for long flights and commuting.</p>
                </div>

                <div class="review">
                    <div class="review-header">
                        <span class="reviewer-name">Sarah Williams</span>
                        <span class="review-rating">★★★★☆</span>
                        <span class="review-date">January 8, 2024</span>
                    </div>
                    <p class="review-text">Very comfortable to wear for hours. The battery life is impressive.
                    Only minor complaint is the case could be smaller.</p>
                </div>
            </section>
        </div>
    </main>

    <footer class="site-footer">
        <div class="container">
            <p>&copy; 2024 AudioTech Store. All rights reserved.</p>
        </div>
    </footer>
</body>
</html>
"#;

/// Documentation page with code examples
pub const DOCUMENTATION_HTML: &str = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>API Reference - Authentication | DevDocs</title>
    <meta name="description" content="Complete API reference for user authentication endpoints">
</head>
<body>
    <div class="docs-layout">
        <nav class="sidebar">
            <h2>API Reference</h2>
            <ul>
                <li><a href="#getting-started">Getting Started</a></li>
                <li><a href="#authentication" class="active">Authentication</a></li>
                <li><a href="#users">Users</a></li>
                <li><a href="#projects">Projects</a></li>
                <li><a href="#errors">Error Handling</a></li>
            </ul>
        </nav>

        <main class="docs-content">
            <article>
                <h1>Authentication</h1>
                <p>The DevDocs API uses API keys for authentication. All API requests must include
                a valid API key in the Authorization header.</p>

                <h2>API Key Authentication</h2>
                <p>Include your API key in the Authorization header of your requests:</p>

                <div class="code-block">
                    <h3>HTTP Header</h3>
                    <pre><code>Authorization: Bearer YOUR_API_KEY</code></pre>
                </div>

                <h2>Obtaining an API Key</h2>
                <ol>
                    <li>Sign in to your DevDocs account</li>
                    <li>Navigate to the API Keys section in your dashboard</li>
                    <li>Click "Generate New Key"</li>
                    <li>Copy and store your key securely</li>
                </ol>

                <div class="warning-box">
                    <h3>⚠️ Important Security Notes</h3>
                    <ul>
                        <li>Never expose your API key in client-side code</li>
                        <li>Use environment variables to store keys</li>
                        <li>Rotate keys regularly for security</li>
                        <li>Use different keys for different environments</li>
                    </ul>
                </div>

                <h2>Authentication Endpoints</h2>

                <h3>POST /auth/login</h3>
                <p>Authenticate a user with email and password.</p>

                <h4>Request Body</h4>
                <div class="code-block">
                    <pre><code>{
  "email": "user@example.com",
  "password": "your-password"
}</code></pre>
                </div>

                <h4>Response</h4>
                <div class="code-block">
                    <pre><code>{
  "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "refresh_token": "dGhpcyBpcyBhIHJlZnJlc2ggdG9rZW4...",
  "expires_in": 3600,
  "user": {
    "id": 123,
    "email": "user@example.com",
    "name": "John Doe"
  }
}</code></pre>
                </div>

                <h3>POST /auth/refresh</h3>
                <p>Refresh an expired access token using a refresh token.</p>

                <h4>Request Body</h4>
                <div class="code-block">
                    <pre><code>{
  "refresh_token": "dGhpcyBpcyBhIHJlZnJlc2ggdG9rZW4..."
}</code></pre>
                </div>

                <h4>Response</h4>
                <div class="code-block">
                    <pre><code>{
  "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "expires_in": 3600
}</code></pre>
                </div>

                <h3>POST /auth/logout</h3>
                <p>Invalidate the current session and refresh token.</p>

                <h4>Headers</h4>
                <div class="code-block">
                    <pre><code>Authorization: Bearer YOUR_ACCESS_TOKEN</code></pre>
                </div>

                <h4>Response</h4>
                <div class="code-block">
                    <pre><code>{
  "message": "Successfully logged out"
}</code></pre>
                </div>

                <h2>Error Responses</h2>
                <p>Authentication errors return appropriate HTTP status codes with error details:</p>

                <div class="code-block">
                    <h4>401 Unauthorized</h4>
                    <pre><code>{
  "error": {
    "code": "INVALID_CREDENTIALS",
    "message": "Invalid email or password",
    "details": null
  }
}</code></pre>
                </div>

                <div class="code-block">
                    <h4>403 Forbidden</h4>
                    <pre><code>{
  "error": {
    "code": "EXPIRED_TOKEN",
    "message": "Access token has expired",
    "details": {
      "expired_at": "2024-01-15T10:30:00Z"
    }
  }
}</code></pre>
                </div>

                <h2>Code Examples</h2>

                <h3>JavaScript</h3>
                <div class="code-block">
                    <pre><code>const response = await fetch('https://api.devdocs.com/auth/login', {
  method: 'POST',
  headers: {
    'Content-Type': 'application/json',
  },
  body: JSON.stringify({
    email: 'user@example.com',
    password: 'your-password'
  })
});

const data = await response.json();
console.log('Access token:', data.access_token);</code></pre>
                </div>

                <h3>Python</h3>
                <div class="code-block">
                    <pre><code>import requests

response = requests.post('https://api.devdocs.com/auth/login',
    json={
        'email': 'user@example.com',
        'password': 'your-password'
    }
)

data = response.json()
access_token = data['access_token']
print(f'Access token: {access_token}')</code></pre>
                </div>

                <h3>cURL</h3>
                <div class="code-block">
                    <pre><code>curl -X POST https://api.devdocs.com/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "user@example.com",
    "password": "your-password"
  }'</code></pre>
                </div>
            </article>
        </main>
    </div>
</body>
</html>
"#;

/// Test fixture for different content types
pub fn get_fixture(name: &str) -> Option<&'static str> {
    match name {
        "blog_post" => Some(BLOG_POST_HTML),
        "news_article" => Some(NEWS_ARTICLE_HTML),
        "spa_application" => Some(SPA_APPLICATION_HTML),
        "ecommerce_product" => Some(ECOMMERCE_PRODUCT_HTML),
        "documentation" => Some(DOCUMENTATION_HTML),
        _ => None,
    }
}

/// Get expected extraction data for fixtures
pub fn get_expected_extraction(name: &str) -> Option<ExpectedExtraction> {
    match name {
        "blog_post" => Some(ExpectedExtraction {
            title: Some("Understanding WebAssembly: A Comprehensive Guide".to_string()),
            author: Some("Jane Developer".to_string()),
            published: Some("2024-01-15".to_string()),
            content_type: "article",
            min_text_length: 800,
            key_phrases: vec![
                "WebAssembly",
                "binary instruction format",
                "performance",
                "security",
                "portability",
                "compilation target",
            ],
            expected_links: 2,
            gate_decision: "raw", // Should be good quality content
        }),
        "news_article" => Some(ExpectedExtraction {
            title: Some("Breakthrough in Quantum Computing Announced by Research Team".to_string()),
            author: Some("Dr. Sarah Chen".to_string()),
            published: Some("2024-01-20".to_string()),
            content_type: "news",
            min_text_length: 600,
            key_phrases: vec![
                "quantum computing",
                "error correction",
                "breakthrough",
                "qubits",
                "research team",
            ],
            expected_links: 2,
            gate_decision: "raw",
        }),
        "spa_application" => Some(ExpectedExtraction {
            title: Some("TaskMaster - Project Management Tool".to_string()),
            author: None,
            published: None,
            content_type: "application",
            min_text_length: 200,
            key_phrases: vec![
                "TaskMaster",
                "project management",
                "JavaScript",
                "requires JavaScript",
            ],
            expected_links: 0,
            gate_decision: "headless", // SPA needs JavaScript rendering
        }),
        "ecommerce_product" => Some(ExpectedExtraction {
            title: Some("Premium Wireless Headphones - AudioTech Store".to_string()),
            author: None,
            published: None,
            content_type: "product",
            min_text_length: 400,
            key_phrases: vec![
                "Premium Wireless Headphones",
                "noise cancellation",
                "battery life",
                "AudioTech",
                "$299.99",
            ],
            expected_links: 0,
            gate_decision: "raw",
        }),
        "documentation" => Some(ExpectedExtraction {
            title: Some("API Reference - Authentication | DevDocs".to_string()),
            author: None,
            published: None,
            content_type: "documentation",
            min_text_length: 600,
            key_phrases: vec![
                "API Reference",
                "Authentication",
                "API key",
                "Authorization header",
                "access_token",
            ],
            expected_links: 0,
            gate_decision: "raw",
        }),
        _ => None,
    }
}

/// Expected extraction results for validation
pub struct ExpectedExtraction {
    pub title: Option<String>,
    pub author: Option<String>,
    pub published: Option<String>,
    pub content_type: &'static str,
    pub min_text_length: usize,
    pub key_phrases: Vec<&'static str>,
    pub expected_links: usize,
    pub gate_decision: &'static str,
}