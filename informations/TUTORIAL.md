# Rusti Framework Tutorial

Welcome to the Rusti framework tutorial! This guide will walk you through building a complete web application from scratch.

## What We'll Build

A simple blog application with:
- Home page listing posts
- Individual post pages
- About page
- Static file serving
- Database integration (optional)

## Prerequisites

- Rust 1.70+ installed
- Basic Rust knowledge
- Familiarity with web concepts

## Part 1: Setup

### Create a New Project

```bash
cargo new myblog
cd myblog
```

### Add Dependencies

Edit `Cargo.toml`:

```toml
[dependencies]
rusti = "0.1"  # Use path to local rusti if not published
tokio = { version = "1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

### Create Directory Structure

```bash
mkdir -p src/templates src/static/css src/views
```

## Part 2: Basic Application

### Main Entry Point

Create `src/main.rs`:

```rust
use rusti::prelude::*;

mod views;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables
    dotenvy::dotenv().ok();

    // Define routes
    let router = Router::new()
        .route("/", get(views::home))
        .route("/about", get(views::about));

    // Build and run app
    let app = RustiApp::new()
        .with_default_config()
        .with_router(router)
        .build()
        .await?;

    println!("🦀 Starting blog application...");
    app.run().await?;
    Ok(())
}
```

### Create Views Module

Create `src/views.rs`:

```rust
use rusti::prelude::*;
use std::sync::Arc;

pub async fn home(
    Extension(tera): Extension<Arc<Tera>>,
) -> Response {
    let posts = vec![
        json!({
            "id": 1,
            "title": "Welcome to Rusti",
            "excerpt": "Building web apps with Rust is now easier than ever",
            "date": "2024-01-15"
        }),
        json!({
            "id": 2,
            "title": "Getting Started Guide",
            "excerpt": "Learn how to build your first Rusti application",
            "date": "2024-01-20"
        }),
    ];

    let context = Context::from_serialize(json!({
        "title": "My Rust Blog",
        "posts": posts
    })).unwrap();

    render(&tera, "index.html", &context)
}

pub async fn about(
    Extension(tera): Extension<Arc<Tera>>,
) -> Response {
    let context = Context::from_serialize(json!({
        "title": "About",
        "content": "This blog is built with the Rusti framework"
    })).unwrap();

    render(&tera, "about.html", &context)
}
```

## Part 3: Templates

### Base Template

Create `src/templates/base.html`:

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{% block title %}My Blog{% endblock %}</title>
    <link rel="stylesheet" href="/static/css/style.css">
</head>
<body>
    <nav class="navbar">
        <div class="container">
            <a href="/" class="logo">🦀 My Blog</a>
            <ul class="nav-links">
                <li><a href="/">Home</a></li>
                <li><a href="/about">About</a></li>
            </ul>
        </div>
    </nav>

    <main class="container">
        {% block content %}{% endblock %}
    </main>

    <footer class="footer">
        <div class="container">
            <p>Built with Rusti Framework</p>
        </div>
    </footer>
</body>
</html>
```

### Home Page Template

Create `src/templates/index.html`:

```html
{% extends "base.html" %}

{% block title %}{{ title }}{% endblock %}

{% block content %}
<div class="hero">
    <h1>Welcome to {{ title }}</h1>
    <p>Exploring web development with Rust</p>
</div>

<div class="posts">
    {% for post in posts %}
    <article class="post-card">
        <h2><a href="/post/{{ post.id }}">{{ post.title }}</a></h2>
        <p class="post-meta">{{ post.date }}</p>
        <p>{{ post.excerpt }}</p>
        <a href="/post/{{ post.id }}" class="read-more">Read more →</a>
    </article>
    {% endfor %}
</div>
{% endblock %}
```

### About Page Template

Create `src/templates/about.html`:

```html
{% extends "base.html" %}

{% block title %}{{ title }}{% endblock %}

{% block content %}
<div class="about">
    <h1>{{ title }}</h1>
    <p>{{ content }}</p>
    
    <h2>About This Framework</h2>
    <p>
        Rusti is a Django-inspired web framework for Rust that combines
        the elegance of Python web frameworks with the performance and
        safety of Rust.
    </p>
    
    <h3>Technology Stack</h3>
    <ul>
        <li>Axum - Web framework</li>
        <li>Tera - Template engine</li>
        <li>SeaORM - Database ORM (optional)</li>
        <li>Tokio - Async runtime</li>
    </ul>
</div>
{% endblock %}
```

## Part 4: Styling

Create `src/static/css/style.css`:

```css
* {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
}

body {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    line-height: 1.6;
    color: #333;
    background: #f5f5f5;
}

.container {
    max-width: 900px;
    margin: 0 auto;
    padding: 0 20px;
}

.navbar {
    background: white;
    box-shadow: 0 2px 4px rgba(0,0,0,0.1);
    padding: 1rem 0;
    margin-bottom: 2rem;
}

.navbar .container {
    display: flex;
    justify-content: space-between;
    align-items: center;
}

.logo {
    font-size: 1.5rem;
    font-weight: bold;
    text-decoration: none;
    color: #667eea;
}

.nav-links {
    display: flex;
    list-style: none;
    gap: 2rem;
}

.nav-links a {
    text-decoration: none;
    color: #666;
    font-weight: 500;
    transition: color 0.2s;
}

.nav-links a:hover {
    color: #667eea;
}

.hero {
    text-align: center;
    padding: 3rem 0;
    background: white;
    border-radius: 8px;
    margin-bottom: 2rem;
}

.hero h1 {
    font-size: 2.5rem;
    margin-bottom: 0.5rem;
    color: #333;
}

.hero p {
    font-size: 1.2rem;
    color: #666;
}

.posts {
    display: grid;
    gap: 1.5rem;
}

.post-card {
    background: white;
    padding: 2rem;
    border-radius: 8px;
    box-shadow: 0 2px 4px rgba(0,0,0,0.1);
    transition: transform 0.2s, box-shadow 0.2s;
}

.post-card:hover {
    transform: translateY(-2px);
    box-shadow: 0 4px 8px rgba(0,0,0,0.15);
}

.post-card h2 {
    margin-bottom: 0.5rem;
}

.post-card h2 a {
    color: #333;
    text-decoration: none;
}

.post-card h2 a:hover {
    color: #667eea;
}

.post-meta {
    color: #999;
    font-size: 0.9rem;
    margin-bottom: 1rem;
}

.read-more {
    color: #667eea;
    text-decoration: none;
    font-weight: 500;
}

.about {
    background: white;
    padding: 2rem;
    border-radius: 8px;
}

.about h1 {
    margin-bottom: 1rem;
}

.about h2, .about h3 {
    margin-top: 1.5rem;
    margin-bottom: 0.75rem;
    color: #667eea;
}

.about ul {
    margin-left: 2rem;
}

.about li {
    margin-bottom: 0.5rem;
}

.footer {
    margin-top: 3rem;
    padding: 2rem 0;
    text-align: center;
    color: #666;
    border-top: 1px solid #ddd;
}
```

## Part 5: Configuration

Create `.env`:

```env
HOST=127.0.0.1
PORT=3000
DEBUG=true
SECRET_KEY=your-secret-key-change-in-production
```

Create `.env.example` for version control:

```env
HOST=127.0.0.1
PORT=3000
DEBUG=true
SECRET_KEY=change-me-in-production
```

## Part 6: Run Your Application

```bash
cargo run
```

Visit `http://127.0.0.1:3000` in your browser!

## Part 7: Add Individual Post Pages

### Update Views

Add to `src/views.rs`:

```rust
pub async fn post_detail(
    Extension(tera): Extension<Arc<Tera>>,
    Path(id): Path<u32>,
) -> Response {
    // In a real app, fetch from database
    let posts = vec![
        json!({
            "id": 1,
            "title": "Welcome to Rusti",
            "content": "Full content of the first post...",
            "date": "2024-01-15"
        }),
        json!({
            "id": 2,
            "title": "Getting Started Guide",
            "content": "Full content of the second post...",
            "date": "2024-01-20"
        }),
    ];

    let post = posts.iter().find(|p| p["id"] == id);

    match post {
        Some(post) => {
            let context = Context::from_serialize(json!({
                "title": post["title"],
                "post": post
            })).unwrap();
            render(&tera, "post_detail.html", &context)
        }
        None => {
            // Return 404
            let context = Context::new();
            render_with_status(&tera, "errors/404.html", &context, StatusCode::NOT_FOUND)
        }
    }
}
```

### Update Routes

In `src/main.rs`:

```rust
let router = Router::new()
    .route("/", get(views::home))
    .route("/about", get(views::about))
    .route("/post/:id", get(views::post_detail));  // New route
```

### Create Post Detail Template

Create `src/templates/post_detail.html`:

```html
{% extends "base.html" %}

{% block title %}{{ post.title }}{% endblock %}

{% block content %}
<article class="post-detail">
    <h1>{{ post.title }}</h1>
    <p class="post-meta">{{ post.date }}</p>
    <div class="post-content">
        {{ post.content }}
    </div>
    <a href="/" class="back-link">← Back to all posts</a>
</article>
{% endblock %}
```

Add styling to `src/static/css/style.css`:

```css
.post-detail {
    background: white;
    padding: 2rem;
    border-radius: 8px;
}

.post-detail h1 {
    margin-bottom: 0.5rem;
}

.post-content {
    margin: 2rem 0;
    line-height: 1.8;
}

.back-link {
    color: #667eea;
    text-decoration: none;
    font-weight: 500;
}
```

## Next Steps

### Add Database Support

1. Enable ORM feature in `Cargo.toml`:
```toml
rusti = { version = "0.1", features = ["orm"] }
sea-orm = "2.0"
```

2. Configure database in `.env`:
```env
DB_ENGINE=sqlite
DATABASE_URL=sqlite://blog.db?mode=rwc
```

3. Connect database:
```rust
let app = RustiApp::new()
    .with_default_config()
    .with_database()
    .await?
    .with_router(router)
    .build()
    .await?;
```

### Add Form Handling

Create a contact form, comment system, or admin interface.

### Add Authentication

Implement user registration, login, and protected routes.

### Deploy

Deploy your Rusti application to:
- VPS (DigitalOcean, Linode, etc.)
- Railway
- Fly.io
- Your own server

## Congratulations!

You've built a complete web application with the Rusti framework! 🎉

Check out more examples in the `examples/` directory and read the full documentation for advanced features.

Happy coding! 🦀
