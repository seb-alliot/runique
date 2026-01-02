# Configuration Guide - Rusti Framework

Rusti uses a centralized configuration system via the `Settings` struct and the `.env` file.

## Table of Contents

1. [Settings Structure](#settings-structure)
2. [Configuration via .env](#configuration-via-env)
3. [Programmatic Configuration](#programmatic-configuration)
4. [Environment Variables](#environment-variables)
5. [Security](#security)
6. [Middleware](#middleware)

---

## Settings Structure

The `Settings` struct centralizes all configuration for your Rusti application.

### Definition

```rust
pub struct Settings {
    // Server
    pub host: String,
    pub port: u16,
    pub workers: usize,

    // Security
    pub secret_key: String,
    pub allowed_hosts: Vec<String>,
    pub debug: bool,

    // Database
    pub database_url: Option<String>,

    // Static files
    pub static_url: String,
    pub static_root: PathBuf,
    pub media_url: String,
    pub media_root: PathBuf,

    // Templates
    pub templates_dir: PathBuf,

    // Sessions
    pub session_cookie_name: String,
    pub session_cookie_secure: bool,
    pub session_cookie_httponly: bool,
    pub session_cookie_samesite: String,

    // CSRF
    pub csrf_cookie_name: String,
    pub csrf_header_name: String,

    // Placeholder for future features
    pub rate_limiting: bool,  // ⚠️ Not implemented - See Rate Limiting section
}
```

### Loading from `.env`

```rust
use rusti::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Automatically loads from .env
    let settings = Settings::from_env();

    RustiApp::new(settings).await?
        .routes(routes())
        .run()
        .await?;

    Ok(())
}
```

---

## Configuration via .env

Create a `.env` file at the root of your project:

```env
# Server
HOST=127.0.0.1
PORT=8000
WORKERS=4

# Security
SECRET_KEY=your-very-long-and-random-secret-key
ALLOWED_HOSTS=localhost,127.0.0.1,example.com
DEBUG=true

# Database (PostgreSQL)
DB_ENGINE=postgres
DB_USER=myuser
DB_PASSWORD=mypassword
DB_HOST=localhost
DB_PORT=5432
DB_NAME=mydb

# Static files
STATIC_URL=/static/
STATIC_ROOT=static/
MEDIA_URL=/media/
MEDIA_ROOT=media/

# Templates
TEMPLATES_DIR=templates/

# Sessions
SESSION_COOKIE_NAME=sessionid
SESSION_COOKIE_SECURE=false
SESSION_COOKIE_HTTPONLY=true
SESSION_COOKIE_SAMESITE=Lax

# CSRF
CSRF_COOKIE_NAME=csrftoken
CSRF_HEADER_NAME=X-CSRFToken

# Placeholder (not implemented)
RATE_LIMITING=false
```

---

## Programmatic Configuration

### Manual Configuration

```rust
use rusti::prelude::*;
use std::path::PathBuf;

let settings = Settings {
    host: "0.0.0.0".to_string(),
    port: 3000,
    workers: 8,
    secret_key: "my-secret-key".to_string(),
    allowed_hosts: vec![
        "example.com".to_string(),
        "www.example.com".to_string(),
    ],
    debug: false,
    database_url: Some("postgres://user:pass@localhost/db".to_string()),
    static_url: "/static/".to_string(),
    static_root: PathBuf::from("static"),
    media_url: "/media/".to_string(),
    media_root: PathBuf::from("media"),
    templates_dir: PathBuf::from("templates"),
    session_cookie_name: "sessionid".to_string(),
    session_cookie_secure: true,
    session_cookie_httponly: true,
    session_cookie_samesite: "Strict".to_string(),
    csrf_cookie_name: "csrftoken".to_string(),
    csrf_header_name: "X-CSRFToken".to_string(),
    rate_limiting: false,
};

RustiApp::new(settings).await?
    .routes(routes())
    .run()
    .await?;
```

### Modifying Default Values

```rust
let mut settings = Settings::from_env();

// Modify after loading
settings.port = 9000;
settings.workers = 16;
settings.allowed_hosts.push("api.example.com".to_string());

RustiApp::new(settings).await?
    .routes(routes())
    .run()
    .await?;
```

---

## Environment Variables

### Server

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `HOST` | String | `127.0.0.1` | Server listening address |
| `PORT` | u16 | `8000` | Listening port |
| `WORKERS` | usize | `4` | Number of Tokio workers |

**Example:**

```env
HOST=0.0.0.0
PORT=3000
WORKERS=8
```

### Security

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `SECRET_KEY` | String | **Required** | Secret key for CSRF/sessions (min 32 chars) |
| `ALLOWED_HOSTS` | Vec | `[]` | List of authorized domains (comma-separated) |
| `DEBUG` | bool | `false` | Debug mode (shows detailed errors) |

**Example:**

```env
SECRET_KEY=a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6q7r8s9t0
ALLOWED_HOSTS=localhost,127.0.0.1,example.com,*.example.com
DEBUG=false
```

**⚠️ IMPORTANT:**
- `SECRET_KEY` must be **minimum 32 characters**
- Generate it with: `openssl rand -base64 32`
- **NEVER** commit your `.env` to Git
- In production: `DEBUG=false` is mandatory

### Database

See [Database Guide](DATABASE.md) for complete configuration.

```env
DB_ENGINE=postgres
DB_USER=myuser
DB_PASSWORD=mypassword
DB_HOST=localhost
DB_PORT=5432
DB_NAME=mydb
```

### Static and Media Files

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `STATIC_URL` | String | `/static/` | Base URL for static files |
| `STATIC_ROOT` | Path | `static/` | Physical path to static files |
| `MEDIA_URL` | String | `/media/` | Base URL for uploaded files |
| `MEDIA_ROOT` | Path | `media/` | Physical path to uploaded files |

**Example:**

```env
STATIC_URL=/static/
STATIC_ROOT=/var/www/myapp/static/
MEDIA_URL=/media/
MEDIA_ROOT=/var/www/myapp/media/
```

### Templates

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `TEMPLATES_DIR` | Path | `templates/` | Tera templates directory |

**Example:**

```env
TEMPLATES_DIR=templates/
```

### Sessions

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `SESSION_COOKIE_NAME` | String | `sessionid` | Session cookie name |
| `SESSION_COOKIE_SECURE` | bool | `false` | Cookie only on HTTPS |
| `SESSION_COOKIE_HTTPONLY` | bool | `true` | Cookie not accessible via JavaScript |
| `SESSION_COOKIE_SAMESITE` | String | `Lax` | SameSite policy (`Strict`, `Lax`, `None`) |

**Example (production):**

```env
SESSION_COOKIE_NAME=sessionid
SESSION_COOKIE_SECURE=true
SESSION_COOKIE_HTTPONLY=true
SESSION_COOKIE_SAMESITE=Strict
```

### CSRF

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `CSRF_COOKIE_NAME` | String | `csrftoken` | CSRF cookie name |
| `CSRF_HEADER_NAME` | String | `X-CSRFToken` | HTTP header for AJAX requests |

**Example:**

```env
CSRF_COOKIE_NAME=csrftoken
CSRF_HEADER_NAME=X-CSRFToken
```

### Rate Limiting

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `RATE_LIMITING` | bool | `false` | ⚠️ **Non-implemented placeholder** |

**⚠️ IMPORTANT: Feature Not Implemented**

The `RATE_LIMITING` flag exists in the configuration but **no rate limiting middleware is currently implemented in Rusti**.

**If you need rate limiting:**

You can manually integrate the [tower-governor](https://crates.io/crates/tower-governor) library:

```rust
use tower_governor::{
    governor::GovernorConfigBuilder, 
    GovernorLayer,
};
use std::time::Duration;

// Configuration: 10 requests per minute per IP
let governor_conf = Box::new(
    GovernorConfigBuilder::default()
        .per_second(10)
        .burst_size(20)
        .finish()
        .unwrap(),
);

let governor_limiter = governor_conf.limiter().clone();
let governor_layer = GovernorLayer {
    config: Box::leak(governor_conf),
};

// Add to RustiApp
RustiApp::new(settings).await?
    .middleware(governor_layer)  // ✅ Rate limiting active
    .routes(routes())
    .run()
    .await?;
```

**Future Roadmap:**

This feature is planned for a future version of Rusti as an integrated middleware. Until then, use `tower-governor` directly.

---

## Security

### Generating SECRET_KEY

```bash
# Method 1: OpenSSL
openssl rand -base64 32

# Method 2: Python
python3 -c "import secrets; print(secrets.token_urlsafe(32))"

# Method 3: Rust
cargo add rand
```

```rust
use rand::Rng;
use rand::distributions::Alphanumeric;

fn generate_secret_key() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(64)
        .map(char::from)
        .collect()
}
```

### ALLOWED_HOSTS Configuration

**Syntax:**

```env
# Exact domains
ALLOWED_HOSTS=example.com,www.example.com

# Wildcard for subdomains
ALLOWED_HOSTS=*.example.com

# Localhost + production
ALLOWED_HOSTS=localhost,127.0.0.1,example.com

# All subdomains AND main domain
ALLOWED_HOSTS=example.com,*.example.com
```

**⚠️ Security:**
- Never use `*` alone in production
- Always explicitly list authorized domains
- Wildcards match only one level: `*.example.com` matches `api.example.com` but not `v1.api.example.com`

### DEBUG Mode

```env
# Development
DEBUG=true

# Production
DEBUG=false
```

**In DEBUG=true mode:**
- Shows complete stack traces
- Verbose logging
- Detailed error messages

**In DEBUG=false mode (production):**
- Generic errors for users
- Logs only to files
- No exposed stack traces

---

## Middleware

### Configuration via RustiApp

```rust
use rusti::prelude::*;
use rusti::middleware::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = Settings::from_env();

    RustiApp::new(settings).await?
        // Security middleware
        .middleware(CsrfMiddleware::new())
        .middleware(SecurityHeadersMiddleware::new())
        .middleware(AllowedHostsMiddleware)

        // Session and messages middleware
        .middleware(FlashMiddleware)
        .middleware(MessageMiddleware)

        // Sanitization middleware
        .middleware(XssSanitizerMiddleware)

        // Routes
        .routes(routes())

        // Launch
        .run()
        .await?;

    Ok(())
}
```

### Available Middleware

| Middleware | Description | Required |
|------------|-------------|----------|
| `CsrfMiddleware` | CSRF protection via HMAC-SHA256 token | ✅ Include |
| `SecurityHeadersMiddleware` | HTTP security headers | ✅ Recommended |
| `AllowedHostsMiddleware` | Host header validation | ✅ Recommended |
| `FlashMiddleware` | Flash messages between requests | ✅ Include |
| `MessageMiddleware` | User messages | ✅ Include |
| `XssSanitizerMiddleware` | XSS sanitization (ammonia) | ✅ Recommended |
| `CspMiddleware` | Content Security Policy | ✅ Recommended |

See [Security Guide](SECURITY.md) for complete details.

---

## Configuration Examples

### Development Configuration

```env
# .env.development
HOST=127.0.0.1
PORT=8000
WORKERS=4
SECRET_KEY=dev-secret-key-change-in-production
ALLOWED_HOSTS=localhost,127.0.0.1
DEBUG=true

DB_ENGINE=sqlite
DB_NAME=dev.sqlite

STATIC_URL=/static/
STATIC_ROOT=static/
MEDIA_URL=/media/
MEDIA_ROOT=media/

TEMPLATES_DIR=templates/

SESSION_COOKIE_SECURE=false
SESSION_COOKIE_HTTPONLY=true
SESSION_COOKIE_SAMESITE=Lax
```

### Production Configuration

```env
# .env.production
HOST=0.0.0.0
PORT=8000
WORKERS=16
SECRET_KEY=a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6q7r8s9t0
ALLOWED_HOSTS=example.com,www.example.com,api.example.com
DEBUG=false

DB_ENGINE=postgres
DB_USER=produser
DB_PASSWORD=secure-password-here
DB_HOST=db.internal.example.com
DB_PORT=5432
DB_NAME=proddb

STATIC_URL=/static/
STATIC_ROOT=/var/www/example.com/static/
MEDIA_URL=/media/
MEDIA_ROOT=/var/www/example.com/media/

TEMPLATES_DIR=/var/www/example.com/templates/

SESSION_COOKIE_SECURE=true
SESSION_COOKIE_HTTPONLY=true
SESSION_COOKIE_SAMESITE=Strict

CSRF_COOKIE_NAME=csrftoken
CSRF_HEADER_NAME=X-CSRFToken
```

### Docker Configuration

```env
# .env.docker
HOST=0.0.0.0
PORT=8000
WORKERS=8
SECRET_KEY=${SECRET_KEY}
ALLOWED_HOSTS=localhost,app
DEBUG=false

DB_ENGINE=postgres
DB_USER=${POSTGRES_USER}
DB_PASSWORD=${POSTGRES_PASSWORD}
DB_HOST=postgres
DB_PORT=5432
DB_NAME=${POSTGRES_DB}

STATIC_URL=/static/
STATIC_ROOT=/app/static/
MEDIA_URL=/media/
MEDIA_ROOT=/app/media/

TEMPLATES_DIR=/app/templates/
```

---

## Best Practices

### 1. Never Commit .env File

```gitignore
# .gitignore
.env
.env.*
!.env.example
```

### 2. Create a .env.example

```env
# .env.example
HOST=127.0.0.1
PORT=8000
WORKERS=4
SECRET_KEY=change-me-in-production
ALLOWED_HOSTS=localhost,127.0.0.1
DEBUG=true

DB_ENGINE=postgres
DB_USER=your_user
DB_PASSWORD=your_password
DB_HOST=localhost
DB_PORT=5432
DB_NAME=your_database

STATIC_URL=/static/
STATIC_ROOT=static/
MEDIA_URL=/media/
MEDIA_ROOT=media/

TEMPLATES_DIR=templates/
```

### 3. Use Different .env Files per Environment

```bash
# Recommended structure
.
├── .env                    # Ignored by Git
├── .env.example           # Template committed
├── .env.development       # Dev config (ignored)
├── .env.production        # Prod config (ignored)
└── .env.docker           # Docker config (ignored)
```

### 4. Validate Configuration at Startup

```rust
use rusti::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = Settings::from_env();

    // Validations
    assert!(settings.secret_key.len() >= 32, "SECRET_KEY too short");
    assert!(!settings.allowed_hosts.is_empty(), "ALLOWED_HOSTS empty");
    
    if !settings.debug {
        assert!(settings.session_cookie_secure, "COOKIE_SECURE must be true in production");
    }

    RustiApp::new(settings).await?
        .routes(routes())
        .run()
        .await?;

    Ok(())
}
```

### 5. Use Managed Secrets in Production

```rust
// Example with AWS Secrets Manager, Vault, etc.
use aws_sdk_secretsmanager::Client;

async fn load_secret_key() -> String {
    let config = aws_config::load_from_env().await;
    let client = Client::new(&config);

    let response = client
        .get_secret_value()
        .secret_id("myapp/secret_key")
        .send()
        .await
        .unwrap();

    response.secret_string().unwrap().to_string()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut settings = Settings::from_env();
    settings.secret_key = load_secret_key().await;

    RustiApp::new(settings).await?
        .routes(routes())
        .run()
        .await?;

    Ok(())
}
```

---

## See Also

- [Getting Started](GETTING_STARTED.md)
- [Security](SECURITY.md)
- [Database](DATABASE.md)
- [Middleware](MIDDLEWARE.md)

Configure Rusti securely and efficiently!

---

**Version:** 1.0 (Corrected - January 2, 2026)
**License:** MIT