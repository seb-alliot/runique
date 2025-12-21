# ⚙️ Configuration - Rusti Framework

Complete configuration guide for your Rusti application.

## Table of Contents

1. [Configuration Methods](#configuration-methods)
2. [Settings](#settings)
3. [Environment Variables](#environment-variables)
4. [Server Configuration](#server-configuration)
5. [Static and Media Files](#static-and-media-files)
6. [Middleware](#middleware)
7. [Production](#production)

---

## Configuration Methods

Rusti offers 3 ways to configure your application:

### 1. Default Values

```rust
use rusti::Settings;

let settings = Settings::default_values();
```

### 2. From Environment Variables

```rust
let settings = Settings::from_env();
```

### 3. Builder Pattern (recommended)

```rust
let settings = Settings::builder()
    .debug(true)
    .templates_dir(vec!["templates".to_string()])
    .server("127.0.0.1", 3000, "secret-key")
    .build();
```

---

## Settings

### Complete Structure

```rust
pub struct Settings {
    // Server
    pub server: ServerSettings,
    
    // Project
    pub base_dir: String,
    pub debug: bool,
    pub allowed_hosts: Vec<String>,
    
    // Templates
    pub templates_dir: Vec<String>,
    
    // Static files (project)
    pub staticfiles_dirs: String,
    pub static_url: String,
    
    // Media files (uploads)
    pub media_root: String,
    pub media_url: String,
    
    // Internationalization
    pub language_code: String,
    pub time_zone: String,
    pub use_i18n: bool,
    pub use_tz: bool,
}
```

### Server Parameters

```rust
pub struct ServerSettings {
    pub ip_server: String,
    pub domain_server: String,
    pub port: u16,
    pub secret_key: String,
}
```

---

## Environment Variables

### `.env` File

Create a `.env` file at your project root:

```env
# Server
IP_SERVER=127.0.0.1
PORT=3000
SECRET_KEY=change-this-key-in-production

# PostgreSQL Database
DB_ENGINE=postgres
DB_USER=myuser
DB_PASSWORD=mypassword
DB_HOST=localhost
DB_PORT=5432
DB_NAME=mydb

# MySQL Database
# DB_ENGINE=mysql
# DB_USER=root
# DB_PASSWORD=secret
# DB_HOST=localhost
# DB_PORT=3306
# DB_NAME=mydb

# SQLite Database
# DB_ENGINE=sqlite
# DB_NAME=database.sqlite
```

### `.env.example` File

Create a template for other developers:

```env
IP_SERVER=127.0.0.1
PORT=3000
SECRET_KEY=your-secret-key-here

DB_ENGINE=postgres
DB_USER=your-db-user
DB_PASSWORD=your-db-password
DB_HOST=localhost
DB_PORT=5432
DB_NAME=your-db-name
```

### Load Variables

```rust
use rusti::Settings;

// Automatically loads from .env
let settings = Settings::from_env();
```

---

## Server Configuration

### IP and Port

```rust
let settings = Settings::builder()
    .server("127.0.0.1", 3000, "secret")
    .build();

// Or from .env
// IP_SERVER=0.0.0.0
// PORT=8080
```

### Secret Key

The secret key is used for:
- CSRF token generation
- Session signing
- Cookie encryption

**⚠️ IMPORTANT:** Change the secret key in production!

```rust
// ❌ Bad - default key
.server("127.0.0.1", 3000, "default_secret_key")

// ✅ Good - unique and long key
.server("127.0.0.1", 3000, "8k2jF9mN4pQr7sW1xY5zA3bC6eD8gH0j")

// ✅ Better - from environment variable
let secret = std::env::var("SECRET_KEY")?;
.server("127.0.0.1", 3000, &secret)
```

### Generate a Secret Key

```bash
# Linux/Mac
openssl rand -hex 32

# Or in Rust
use rand::Rng;
let key: String = rand::thread_rng()
    .sample_iter(&rand::distributions::Alphanumeric)
    .take(64)
    .map(char::from)
    .collect();
```

---

## Static and Media Files

### Configuration

```rust
let settings = Settings::builder()
    // Static files (CSS, JS, project images)
    .staticfiles_dirs("static")
    .static_url("/static")
    
    // Media files (user uploads)
    .media_root("media")
    .media_url("/media")
    
    .build();
```

### Recommended Structure

```
my-project/
├── static/              # Project static files
│   ├── css/
│   │   └── main.css
│   ├── js/
│   │   └── app.js
│   └── images/
│       └── logo.png
│
└── media/               # Uploaded files
    ├── avatars/
    ├── documents/
    └── uploads/
```

### Use in Templates

```html
<!-- Project static files -->
<link rel="stylesheet" href='{% static "css/main.css" %}'>
<script src='{% static "js/app.js" %}'></script>
<img src='{% static "images/logo.png" %}' alt="Logo">

<!-- Uploaded files -->
<img src='{% media "avatars/user-123.jpg" %}' alt="Avatar">
<a href='{% media "documents/report.pdf" %}'>Download</a>
```

### Serve Files

```rust
RustiApp::new(settings).await?
    .routes(routes())
    .with_static_files()? // ✅ Enable file serving
    .run()
    .await?;
```

---

## Middleware

### Available Middleware

```rust
RustiApp::new(settings).await?
    .routes(routes())
    
    // Static files
    .with_static_files()?
    
    // Flash messages
    .with_flash_messages()
    
    // CSRF protection
    .with_csrf_tokens()
    
    // Default middleware (errors + timeout)
    .with_default_middleware()
    
    .run()
    .await?;
```

### Custom Middleware

```rust
use axum::middleware::{Next, from_fn};
use axum::extract::Request;
use axum::response::Response;

async fn my_middleware(
    request: Request,
    next: Next,
) -> Response {
    // Logic before request
    println!("Request: {} {}", request.method(), request.uri());
    
    let response = next.run(request).await;
    
    // Logic after request
    println!("Status: {}", response.status());
    
    response
}

// Add middleware
let app = RustiApp::new(settings).await?
    .routes(routes())
    .build()
    .layer(from_fn(my_middleware));
```

---

## Production

### Production Configuration

```rust
let settings = Settings::builder()
    .debug(false) // ✅ Disable debug mode
    .server("0.0.0.0", 8080, &env::var("SECRET_KEY")?)
    .build();
```

### Production Environment Variables

```env
# .env.production
IP_SERVER=0.0.0.0
PORT=8080
SECRET_KEY=your-very-long-and-unique-secret-key

DB_ENGINE=postgres
DB_URL=postgresql://user:pass@prod-host:5432/prod_db
```

### Security

#### 1. Always disable debug mode

```rust
// ❌ Danger in production
.debug(true)

// ✅ Good
.debug(false)
```

#### 2. Use HTTPS

```nginx
# nginx.conf
server {
    listen 443 ssl http2;
    server_name myapp.com;
    
    ssl_certificate /path/to/cert.pem;
    ssl_certificate_key /path/to/key.pem;
    
    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

#### 3. CORS Restrictions

```rust
use tower_http::cors::{CorsLayer, Any};

let cors = CorsLayer::new()
    .allow_origin("https://myapp.com".parse::<HeaderValue>()?)
    .allow_methods([Method::GET, Method::POST])
    .allow_headers(Any);

let app = RustiApp::new(settings).await?
    .routes(routes())
    .build()
    .layer(cors);
```

### Optimized Build

```bash
# Production build
cargo build --release

# Strip debug symbols
strip target/release/my-app

# With LTO optimizations
RUSTFLAGS="-C lto=fat" cargo build --release
```

### Optimized `Cargo.toml`

```toml
[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
panic = "abort"
strip = true
```

---

## Logging and Tracing

### Basic Configuration

```rust
use tracing_subscriber;

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    
    // ...
}
```

### Advanced Configuration

```rust
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

tracing_subscriber::registry()
    .with(
        tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "info".into()),
    )
    .with(tracing_subscriber::fmt::layer())
    .init();
```

### Log Levels

```env
# .env
RUST_LOG=debug           # Everything in debug
RUST_LOG=info            # Info and above
RUST_LOG=warn            # Warnings and errors only
RUST_LOG=error           # Errors only

# Specific by module
RUST_LOG=my_app=debug,rusti=info,sea_orm=warn
```

---

## Complete Example

### `src/main.rs`

```rust
use rusti::prelude::*;
use std::env;

mod urls;
mod views;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    
    // Load .env
    dotenvy::dotenv().ok();
    
    // Configuration
    let is_production = env::var("PRODUCTION")
        .unwrap_or_else(|_| "false".to_string())
        == "true";
    
    let settings = Settings::builder()
        .debug(!is_production)
        .templates_dir(vec!["templates".to_string()])
        .staticfiles_dirs("static")
        .media_root("media")
        .server(
            &env::var("IP_SERVER").unwrap_or_else(|_| "127.0.0.1".to_string()),
            env::var("PORT")?.parse()?,
            &env::var("SECRET_KEY")?,
        )
        .build();
    
    // Database
    let db_config = DatabaseConfig::from_env()?.build();
    let db = db_config.connect().await?;
    
    tracing::info!("🦀 Starting Rusti application");
    
    // Launch application
    RustiApp::new(settings).await?
        .with_database(db)
        .routes(urls::routes())
        .with_static_files()?
        .with_flash_messages()
        .with_csrf_tokens()
        .with_default_middleware()
        .run()
        .await?;
    
    Ok(())
}
```

---

## Production Checklist

- [ ] Debug mode disabled (`debug = false`)
- [ ] Unique and secure secret key
- [ ] HTTPS configured (via nginx/Caddy)
- [ ] Production database configured
- [ ] Logs configured (INFO or WARN level)
- [ ] CORS configured according to your needs
- [ ] Rate limiting enabled
- [ ] Build in `--release` mode
- [ ] Secured environment variables
- [ ] Automatic database backups
- [ ] Monitoring (Prometheus, Grafana, etc.)

---

## See Also

- 🚀 [Getting Started](GETTING_STARTED.md)
- 🗄️ [Database](DATABASE.md)
- 📖 [Templates](TEMPLATES.md)

**Configure your Rusti application efficiently! 🦀**
