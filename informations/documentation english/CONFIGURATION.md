# Rusti Configuration Guide

## Application Configuration in the Rusti Framework

### Overview

Rusti uses a centralized configuration system that allows you to manage all aspects of your application through a single `Settings` object. Configuration can be defined programmatically or loaded from environment variables for flexibility across different environments (development, testing, production).

### Table of Contents

1. [Basic Configuration](#basic-configuration)
2. [Configuration Builder](#configuration-builder)
3. [Server Configuration](#server-configuration)
4. [Database Configuration](#database-configuration)
5. [Template Configuration](#template-configuration)
6. [Static and Media Files](#static-and-media-files)
7. [Security Settings](#security-settings)
8. [Environment Variables](#environment-variables)
9. [Complete Examples](#complete-examples)

---

## Basic Configuration

### Minimal Setup

The simplest way to configure a Rusti application:
```rust
use rusti::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = Settings::builder()
        .server("127.0.0.1", 8080, "my-secret-key-change-in-production")
        .build();

    RustiApp::new(settings).await?
        .routes(routes)
        .run()
        .await?;

    Ok(())
}
```

### Settings Structure
```rust
pub struct Settings {
    pub debug: bool,
    pub secret_key: String,
    pub allowed_hosts: Vec<String>,
    pub server_host: String,
    pub server_port: u16,
    pub templates_dir: Vec<String>,
    pub static_url: String,
    pub static_root: String,
    pub media_url: String,
    pub media_root: String,
    pub database_url: Option<String>,
}
```

---

## Configuration Builder

### Using the Builder Pattern

Rusti uses the builder pattern for flexible configuration:
```rust
let settings = Settings::builder()
    .debug(true)                                    // Enable debug mode
    .secret_key("your-secret-key-here")            // Secret key
    .allowed_hosts(vec![                           // Allowed hosts
        "localhost".to_string(),
        "127.0.0.1".to_string(),
        "example.com".to_string(),
    ])
    .server_host("0.0.0.0")                        // Server host
    .server_port(8080)                              // Server port
    .templates_dir(vec!["templates".to_string()])  // Template directories
    .static_url("/static/")                         // Static URL
    .static_root("static")                          // Static files directory
    .media_url("/media/")                           // Media URL
    .media_root("media")                            // Media files directory
    .database_url("postgresql://user:pass@localhost/db") // Database
    .build();
```

### Available Methods
```rust
impl SettingsBuilder {
    pub fn new() -> Self;
    pub fn debug(mut self, debug: bool) -> Self;
    pub fn secret_key(mut self, key: String) -> Self;
    pub fn allowed_hosts(mut self, hosts: Vec<String>) -> Self;
    pub fn server_host(mut self, host: String) -> Self;
    pub fn server_port(mut self, port: u16) -> Self;
    pub fn server(mut self, host: &str, port: u16, secret_key: &str) -> Self;
    pub fn templates_dir(mut self, dirs: Vec<String>) -> Self;
    pub fn static_url(mut self, url: String) -> Self;
    pub fn static_root(mut self, root: String) -> Self;
    pub fn media_url(mut self, url: String) -> Self;
    pub fn media_root(mut self, root: String) -> Self;
    pub fn database_url(mut self, url: String) -> Self;
    pub fn build(self) -> Settings;
}
```

---

## Server Configuration

### Basic Server Setup
```rust
let settings = Settings::builder()
    .server("127.0.0.1", 8080, "secret-key")
    .build();
```

The `server()` method is a shortcut that configures:
- Server host
- Server port
- Secret key

### Custom Server Configuration
```rust
let settings = Settings::builder()
    .server_host("0.0.0.0")        // Listen on all interfaces
    .server_port(3000)              // Custom port
    .secret_key("my-secret-key")    // Secret key for sessions/CSRF
    .build();
```

### Production Server
```rust
let settings = Settings::builder()
    .debug(false)                   // Disable debug mode
    .server_host("0.0.0.0")        // Public access
    .server_port(8080)
    .secret_key(env::var("SECRET_KEY").expect("SECRET_KEY must be set"))
    .allowed_hosts(vec![           // Security: limit allowed hosts
        "example.com".to_string(),
        "www.example.com".to_string(),
    ])
    .build();
```

---

## Database Configuration

### PostgreSQL Configuration
```rust
let settings = Settings::builder()
    .database_url("postgresql://user:password@localhost:5432/mydb")
    .build();

let db = Database::new(&settings.database_url.unwrap()).await?;

RustiApp::new(settings).await?
    .with_database(db)
    .run()
    .await?;
```

### MySQL Configuration
```rust
let settings = Settings::builder()
    .database_url("mysql://user:password@localhost:3306/mydb")
    .build();
```

### SQLite Configuration
```rust
let settings = Settings::builder()
    .database_url("sqlite://database.db?mode=rwc")
    .build();
```

### Connection Pool Configuration

Connection pool settings are managed in `database/config.rs`:
```rust
pub struct DatabaseConfig {
    max_connections: u32,      // Default: 20
    min_connections: u32,      // Default: 5
    connect_timeout: Duration, // Default: 30s
    idle_timeout: Duration,    // Default: 600s
}
```

**Note:** These values are currently hardcoded in the framework. Future versions will expose them in `Settings`.

---

## Template Configuration

### Single Template Directory
```rust
let settings = Settings::builder()
    .templates_dir(vec!["templates".to_string()])
    .build();
```

Directory structure:
```
project/
├── templates/
│   ├── base.html
│   ├── home.html
│   └── about.html
└── src/
    └── main.rs
```

### Multiple Template Directories
```rust
let settings = Settings::builder()
    .templates_dir(vec![
        "templates".to_string(),
        "app1/templates".to_string(),
        "app2/templates".to_string(),
    ])
    .build();
```

Directory structure:
```
project/
├── templates/          # Global templates
│   └── base.html
├── app1/
│   └── templates/      # App1 templates
│       └── app1.html
├── app2/
│   └── templates/      # App2 templates
│       └── app2.html
└── src/
    └── main.rs
```

**Template resolution order:**
1. First directory in the list
2. Second directory
3. Etc.

### Template Engine (Tera)

Rusti uses Tera as its template engine with custom extensions:

**Built-in filters:**
- `{{ path | static }}` - Static file URL
- `{{ path | media }}` - Media file URL

**Built-in tags:**
- `{% csrf %}` - CSRF token
- `{% messages %}` - Flash messages
- `{% link "route_name" %}` - URL resolution

---

## Static and Media Files

### Static Files Configuration

Static files are CSS, JavaScript, images, fonts, etc., that don't change:
```rust
let settings = Settings::builder()
    .static_url("/static/")     // URL prefix
    .static_root("static")      // Filesystem directory
    .build();

RustiApp::new(settings).await?
    .with_static_files()?       // Enable static file serving
    .run()
    .await?;
```

Directory structure:
```
project/
├── static/
│   ├── css/
│   │   └── style.css
│   ├── js/
│   │   └── app.js
│   └── img/
│       └── logo.png
└── src/
    └── main.rs
```

Usage in templates:
```html
<link rel="stylesheet" href='{% static "css/style.css" %}'>
<script src='{% static "js/app.js" %}'></script>
<img src='{% static "img/logo.png" %}'>
```

### Media Files Configuration

Media files are user-uploaded content:
```rust
let settings = Settings::builder()
    .media_url("/media/")       // URL prefix
    .media_root("media")        // Filesystem directory
    .build();
```

Directory structure:
```
project/
├── media/
│   └── uploads/
│       ├── avatars/
│       └── documents/
└── src/
    └── main.rs
```

Usage in templates:
```html
<img src='{{ user.avatar | media }}'>
<a href='{{ document.file | media }}'>Download</a>
```

### Production Setup

In production, serve static/media files through a reverse proxy (nginx, Caddy):

**nginx configuration:**
```nginx
server {
    listen 80;
    server_name example.com;

    location /static/ {
        alias /var/www/myapp/static/;
        expires 30d;
    }

    location /media/ {
        alias /var/www/myapp/media/;
        expires 7d;
    }

    location / {
        proxy_pass http://127.0.0.1:8080;
    }
}
```

---

## Security Settings

### Debug Mode
```rust
let settings = Settings::builder()
    .debug(true)  // Development
    .build();

let settings = Settings::builder()
    .debug(false) // Production
    .build();
```

**Debug mode enabled:**
- Detailed error pages with stack traces
- Template auto-reload
- Verbose logging

**Debug mode disabled:**
- Generic error pages
- No stack traces exposed
- Production logging

### Secret Key

The secret key is used for:
- Session signing
- CSRF token generation
- Cookie signing
```rust
// Development (never use in production)
let settings = Settings::builder()
    .secret_key("dev-secret-key-change-me")
    .build();

// Production (from environment)
let settings = Settings::builder()
    .secret_key(env::var("SECRET_KEY")?)
    .build();
```

**Generate a secure secret key:**
```bash
openssl rand -base64 32
```

### Allowed Hosts

Protection against Host Header Injection attacks:
```rust
let settings = Settings::builder()
    .allowed_hosts(vec![
        "localhost".to_string(),
        "127.0.0.1".to_string(),
        "example.com".to_string(),
        "www.example.com".to_string(),
    ])
    .build();
```

**How it works:**
1. Middleware checks the `Host` header
2. If not in `allowed_hosts`, returns 400 Bad Request
3. Prevents attacks using malicious Host headers

**Development:**
```rust
.allowed_hosts(vec!["localhost".to_string(), "127.0.0.1".to_string()])
```

**Production:**
```rust
.allowed_hosts(vec!["example.com".to_string(), "www.example.com".to_string()])
```

---

## Environment Variables

### Loading from .env File

Create a `.env` file at project root:
```env
DEBUG=false
SECRET_KEY=your-super-secret-key-here-change-in-production
ALLOWED_HOSTS=localhost,127.0.0.1,example.com
SERVER_HOST=0.0.0.0
SERVER_PORT=8080
DATABASE_URL=postgresql://user:password@localhost:5432/mydb
STATIC_URL=/static/
STATIC_ROOT=static
MEDIA_URL=/media/
MEDIA_ROOT=media
```

### Using dotenv

Add to `Cargo.toml`:
```toml
[dependencies]
dotenv = "0.15"
```

Load in `main.rs`:
```rust
use dotenv::dotenv;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok(); // Load .env file

    let settings = Settings::builder()
        .debug(env::var("DEBUG")?.parse()?)
        .secret_key(env::var("SECRET_KEY")?)
        .allowed_hosts(
            env::var("ALLOWED_HOSTS")?
                .split(',')
                .map(|s| s.to_string())
                .collect()
        )
        .server_host(env::var("SERVER_HOST")?)
        .server_port(env::var("SERVER_PORT")?.parse()?)
        .database_url(env::var("DATABASE_URL")?)
        .static_url(env::var("STATIC_URL")?)
        .static_root(env::var("STATIC_ROOT")?)
        .media_url(env::var("MEDIA_URL")?)
        .media_root(env::var("MEDIA_ROOT")?)
        .build();

    RustiApp::new(settings).await?
        .routes(routes)
        .run()
        .await?;

    Ok(())
}
```

### Configuration Helper Function

Create a reusable configuration loader:
```rust
use dotenv::dotenv;
use std::env;

pub fn load_settings() -> Result<Settings, Box<dyn std::error::Error>> {
    dotenv().ok();

    Ok(Settings::builder()
        .debug(env::var("DEBUG")?.parse()?)
        .secret_key(env::var("SECRET_KEY")?)
        .allowed_hosts(
            env::var("ALLOWED_HOSTS")?
                .split(',')
                .map(|s| s.trim().to_string())
                .collect()
        )
        .server_host(env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()))
        .server_port(env::var("SERVER_PORT")?.parse()?)
        .database_url(env::var("DATABASE_URL")?)
        .templates_dir(vec!["templates".to_string()])
        .static_url(env::var("STATIC_URL").unwrap_or_else(|_| "/static/".to_string()))
        .static_root(env::var("STATIC_ROOT").unwrap_or_else(|_| "static".to_string()))
        .media_url(env::var("MEDIA_URL").unwrap_or_else(|_| "/media/".to_string()))
        .media_root(env::var("MEDIA_ROOT").unwrap_or_else(|_| "media".to_string()))
        .build())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = load_settings()?;

    RustiApp::new(settings).await?
        .routes(routes)
        .run()
        .await?;

    Ok(())
}
```

---

## Complete Examples

### Development Configuration
```rust
use rusti::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = Settings::builder()
        .debug(true)
        .server("127.0.0.1", 8080, "dev-secret-key")
        .templates_dir(vec!["templates".to_string()])
        .allowed_hosts(vec!["localhost".to_string(), "127.0.0.1".to_string()])
        .database_url("postgresql://user:pass@localhost/dev_db")
        .build();

    let db = Database::new(&settings.database_url.unwrap()).await?;

    RustiApp::new(settings).await?
        .routes(routes)
        .with_database(db)
        .with_static_files()?
        .with_default_middleware()
        .run()
        .await?;

    Ok(())
}
```

### Production Configuration
```rust
use rusti::prelude::*;
use dotenv::dotenv;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let settings = Settings::builder()
        .debug(false)
        .secret_key(env::var("SECRET_KEY")?)
        .server_host("0.0.0.0")
        .server_port(8080)
        .allowed_hosts(vec![
            env::var("DOMAIN")?,
            format!("www.{}", env::var("DOMAIN")?),
        ])
        .templates_dir(vec!["templates".to_string()])
        .database_url(env::var("DATABASE_URL")?)
        .static_url("/static/")
        .static_root("/var/www/static")
        .media_url("/media/")
        .media_root("/var/www/media")
        .build();

    let db = Database::new(&settings.database_url.unwrap()).await?;

    RustiApp::new(settings).await?
        .routes(routes)
        .with_database(db)
        .with_static_files()?
        .with_security_headers(CspConfig::strict())
        .with_default_middleware()
        .run()
        .await?;

    Ok(())
}
```

### Multi-Environment Configuration
```rust
use rusti::prelude::*;
use std::env;

pub enum Environment {
    Development,
    Testing,
    Production,
}

impl Environment {
    pub fn from_env() -> Self {
        match env::var("ENVIRONMENT")
            .unwrap_or_else(|_| "development".to_string())
            .to_lowercase()
            .as_str()
        {
            "production" | "prod" => Environment::Production,
            "testing" | "test" => Environment::Testing,
            _ => Environment::Development,
        }
    }
}

pub fn load_settings() -> Result<Settings, Box<dyn std::error::Error>> {
    let env = Environment::from_env();

    let builder = Settings::builder();

    let settings = match env {
        Environment::Development => {
            builder
                .debug(true)
                .server("127.0.0.1", 8080, "dev-key")
                .allowed_hosts(vec!["localhost".to_string()])
                .database_url("postgresql://localhost/dev_db")
        },
        Environment::Testing => {
            builder
                .debug(true)
                .server("127.0.0.1", 8081, "test-key")
                .allowed_hosts(vec!["localhost".to_string()])
                .database_url("postgresql://localhost/test_db")
        },
        Environment::Production => {
            builder
                .debug(false)
                .secret_key(env::var("SECRET_KEY")?)
                .server_host("0.0.0.0")
                .server_port(8080)
                .allowed_hosts(vec![env::var("DOMAIN")?])
                .database_url(env::var("DATABASE_URL")?)
        },
    };

    Ok(settings
        .templates_dir(vec!["templates".to_string()])
        .build())
}
```

---

## Best Practices

### Security

1. Never commit `.env` files with secrets
2. Use strong, random secret keys in production
3. Always set `allowed_hosts` in production
4. Disable debug mode in production
5. Use environment variables for sensitive data

### Performance

1. Use connection pooling for databases
2. Serve static files through reverse proxy in production
3. Enable compression at reverse proxy level
4. Use CDN for static assets when possible

### Organization

1. Separate configuration per environment
2. Use `.env.example` as template (without secrets)
3. Document all configuration options
4. Validate configuration at startup
5. Use type-safe configuration

---

## Troubleshooting

### Common Issues

**Issue: "Secret key not set"**
```
Solution: Set SECRET_KEY environment variable or provide via builder
```

**Issue: "Database connection failed"**
```
Solution: Verify DATABASE_URL format and database is running
```

**Issue: "Template not found"**
```
Solution: Check templates_dir path and file exists
```

**Issue: "Static files not served"**
```
Solution: Verify .with_static_files()? is called and path is correct
```

**Issue: "Host header validation failed"**
```
Solution: Add your domain to allowed_hosts list
```

---

## Further Reading

### Related Documentation

- Security Guide (SECURITY.md)
- Database Guide (DATABASE.md)
- Template Guide (TEMPLATES.md)
- Middleware Guide (MIDDLEWARE.md)

### External Resources

- Tera Template Documentation - https://keats.github.io/tera/
- Sea-ORM Documentation - https://www.sea-ql.org/SeaORM/
- Axum Documentation - https://docs.rs/axum/

---

This documentation is part of the Rusti web framework. For more information, see the complete documentation (README.md).

**Version:** 1.0
**Last updated:** January 2025
**License:** MIT