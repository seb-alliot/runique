# Demo App - Rusti Framework Example

This is a demonstration application showcasing the Rusti framework capabilities.

## Features Demonstrated

- ✅ Basic routing
- ✅ Template rendering with Tera
- ✅ Static file serving
- ✅ Environment configuration
- ✅ Multiple views
- ✅ Template inheritance
- ✅ JSON context passing

## Quick Start

```bash
# From the demo-app directory
cp .env.example .env
cargo run
```

Visit: http://127.0.0.1:3000

## Project Structure

```
demo-app/
├── Cargo.toml          # Dependencies
├── .env.example        # Configuration template
├── .env                # Your local config (create this)
└── src/
    ├── main.rs         # Application entry point
    ├── views.rs        # View handlers
    ├── templates/      # Tera templates
    │   ├── base.html   # Base layout
    │   ├── index.html  # Home page
    │   └── about.html  # About page
    └── static/         # Static assets
        └── css/
            └── main.css
```

## Code Walkthrough

### main.rs - Entry Point

```rust
use rusti::prelude::*;

mod views;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load .env file
    dotenvy::dotenv().ok();

    // Define routes
    let router = Router::new()
        .route("/", get(views::index))
        .route("/about", get(views::about));

    // Build and run
    let app = RustiApp::new()
        .with_default_config()
        .with_router(router)
        .build()
        .await?;

    app.run().await?;
    Ok(())
}
```

**Key Points:**
- Uses `rusti::prelude::*` for common imports
- Defines routes with Axum's router
- Uses builder pattern for configuration
- Minimal boilerplate

### views.rs - Handler Functions

```rust
use rusti::prelude::*;
use std::sync::Arc;

pub async fn index(
    Extension(tera): Extension<Arc<Tera>>,
) -> Response {
    let mut context = Context::new();
    context.insert("title", "Welcome to Rusti");
    context.insert("message", "A Django-inspired web framework for Rust");

    render(&tera, "index.html", &context)
}
```

**Key Points:**
- Clean handler signature
- Use `Extension` to access shared state
- Simple context creation
- Easy template rendering

### Templates - Tera Syntax

```html
<!-- base.html -->
<!DOCTYPE html>
<html>
<head>
    <title>{% block title %}Rusti App{% endblock %}</title>
</head>
<body>
    <nav><!-- navigation --></nav>
    {% block content %}{% endblock %}
</body>
</html>

<!-- index.html -->
{% extends "base.html" %}

{% block title %}{{ title }}{% endblock %}

{% block content %}
    <h1>{{ message }}</h1>
{% endblock %}
```

**Key Points:**
- Django/Jinja2-like syntax
- Template inheritance with `extends`
- Blocks for customization
- Variable interpolation with `{{ }}`

## Configuration

### .env File

```env
HOST=127.0.0.1
PORT=3000
DEBUG=true
SECRET_KEY=your-secret-key-here
```

### Available Settings

| Setting | Default | Description |
|---------|---------|-------------|
| HOST | 127.0.0.1 | Server bind address |
| PORT | 3000 | Server port |
| DEBUG | true | Debug mode (shows errors) |
| SECRET_KEY | (required) | Secret for sessions |

## Extending the Demo

### Add a New Route

1. **Add handler in `views.rs`:**
```rust
pub async fn contact(
    Extension(tera): Extension<Arc<Tera>>,
) -> Response {
    let context = Context::new();
    render(&tera, "contact.html", &context)
}
```

2. **Add route in `main.rs`:**
```rust
let router = Router::new()
    .route("/", get(views::index))
    .route("/about", get(views::about))
    .route("/contact", get(views::contact));  // New!
```

3. **Create template `templates/contact.html`:**
```html
{% extends "base.html" %}
{% block content %}
    <h1>Contact Us</h1>
{% endblock %}
```

### Add Dynamic Routes

```rust
// In views.rs
pub async fn user_profile(
    Extension(tera): Extension<Arc<Tera>>,
    Path(username): Path<String>,
) -> Response {
    let context = Context::from_serialize(json!({
        "username": username
    })).unwrap();
    render(&tera, "profile.html", &context)
}

// In main.rs
.route("/user/:username", get(views::user_profile))
```

### Add Database Support

1. **Update `Cargo.toml`:**
```toml
[dependencies]
rusti = { path = "../../rusti", features = ["orm"] }
```

2. **Update `.env`:**
```env
DB_ENGINE=sqlite
DATABASE_URL=sqlite://app.db?mode=rwc
```

3. **Update `main.rs`:**
```rust
let app = RustiApp::new()
    .with_default_config()
    .with_database().await?  // Add this
    .with_router(router)
    .build().await?;
```

4. **Use in views:**
```rust
pub async fn users(
    Extension(db): Extension<Arc<DatabaseConnection>>,
    Extension(tera): Extension<Arc<Tera>>,
) -> Response {
    // Query database here
    let context = Context::new();
    render(&tera, "users.html", &context)
}
```

## Common Tasks

### Add CSS

Place files in `src/static/css/` and reference:
```html
<link rel="stylesheet" href="/static/css/custom.css">
```

### Add JavaScript

Place files in `src/static/js/` and reference:
```html
<script src="/static/js/app.js"></script>
```

### Add Images

Place files in `src/media/` and reference:
```html
<img src="/media/logo.png" alt="Logo">
```

### Handle Forms

```rust
use axum::Form;

#[derive(Deserialize)]
pub struct ContactForm {
    name: String,
    email: String,
    message: String,
}

pub async fn contact_submit(
    Form(form): Form<ContactForm>,
) -> Response {
    // Process form
    // Redirect or render response
}
```

## Development Tips

### Auto-Reload on Changes

```bash
cargo install cargo-watch
cargo watch -x run
```

### Better Logging

Set `RUST_LOG` environment variable:
```bash
RUST_LOG=debug cargo run
```

### Format Code

```bash
cargo fmt
```

### Check for Issues

```bash
cargo clippy
```

## Production Deployment

### Build Release

```bash
cargo build --release
```

### Update .env for Production

```env
HOST=0.0.0.0
PORT=80
DEBUG=false
SECRET_KEY=generate-a-strong-random-key
```

### Run Release Binary

```bash
./target/release/demo-app
```

## Testing

### Unit Tests

Add tests to `views.rs`:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_something() {
        // Your tests
    }
}
```

Run: `cargo test`

## Performance

This demo app can handle:
- Thousands of requests per second
- Concurrent connections
- Static file serving at high speed

Rust + Axum + Tokio = Excellent performance! 🚀

## Learn More

- [Main Framework Docs](../../README.md)
- [Full Tutorial](../../TUTORIAL.md)
- [Architecture Guide](../../PROJECT_STRUCTURE.md)

## Need Help?

- Check the main README
- Look at the code comments
- Try the examples
- Open an issue

Happy coding! 🦀
