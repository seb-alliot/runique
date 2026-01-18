# Configuration Guide - Runique Framework

Runique uses a centralized configuration system via the `Settings` struct and the `.env` file.

## Table of Contents

1. [Settings Structure](#settings-structure)
2. [Configuration via .env](#configuration-via-env)
3. [Programmatic Configuration](#programmatic-configuration)
4. [Environment Variables](#environment-variables)
5. [Security](#security)
6. [Middleware](#middleware)

---

## Settings Structure

The `Settings` struct centralizes all configuration for your Runique application.

### Definition

```rust
pub struct Settings {
    pub server: ServerSettings,
    pub base_dir: String,
    pub debug: bool,
    pub allowed_hosts: Vec<String>,
    pub installed_apps: Vec<String>,
    pub middleware: Vec<String>,
    pub root_urlconf: String,
    pub static_runique_path: String,
    pub static_runique_url: String,
    pub media_runique_path: String,
    pub media_runique_url: String,
    pub templates_runique: String,
    pub templates_dir: Vec<String>,
    pub staticfiles_dirs: String,
    pub media_root: String,
    pub static_url: String,
    pub media_url: String,
    pub staticfiles_storage: String,
    pub language_code: String,
    pub time_zone: String,
    pub use_i18n: bool,
    pub use_tz: bool,
    pub sanitize_inputs: bool,
    pub strict_csp: bool,
    pub rate_limiting: bool,
    pub enforce_https: bool,
    pub auth_password_validators: Vec<String>,
    pub password_hashers: Vec<String>,
    pub default_auto_field: String,
    pub logging_config: String,
}

pub struct ServerSettings {
    pub ip_server: String,
    pub domain_server: String,
    pub port: u16,
    pub secret_key: String,
}
```

### Loading from `.env`

```rust
use runique::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Automatically loads from .env
    let settings = Settings::default_values();

    RuniqueApp::new(settings).await?
        .routes(routes())
        .with_default_middleware()
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
IP_SERVER=127.0.0.1
PORT=3000

# Security
SECRET_KEY=your-very-long-and-random-secret-key
ALLOWED_HOSTS=localhost,127.0.0.1,example.com

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
use runique::prelude::*;

let mut settings = Settings::default_values();

// Modify after loading
settings.server.port = 9000;
settings.allowed_hosts.push("api.example.com".to_string());

RuniqueApp::new(settings).await?
    .routes(routes())
    .with_default_middleware()
    .run()
    .await?;
```

### Modifying Default Values

```rust
let mut settings = Settings::default_values();

// Modify after loading
settings.server.port = 9000;
settings.allowed_hosts.push("api.example.com".to_string());

RuniqueApp::new(settings).await?
    .routes(routes())
    .with_default_middleware()
    .run()
    .await?;
```

---

## Environment Variables

### Server

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `IP_SERVER` | String | `127.0.0.1` | Server listening address |
| `PORT` | u16 | `3000` | Listening port |

**Example:**

```env
IP_SERVER=0.0.0.0
PORT=3000
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

The `RATE_LIMITING` flag exists in the configuration but **no rate limiting middleware is currently implemented in Runique**.

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

// Add to RuniqueApp
RuniqueApp::new(settings).await?
    .middleware(governor_layer)  // ✅ Rate limiting active
    .routes(routes())
    .run()
    .await?;
```

**Future Roadmap:**

This feature is planned for a future version of Runique as an integrated middleware. Until then, use `tower-governor` directly.

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

### Configuration via RuniqueApp

```rust
use runique::prelude::*;
use runique::middleware::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = Settings::default_values();

    RuniqueApp::new(settings).await?
        .with_default_middleware()
        .routes(routes())
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

See [Security Guide](informations/documentation_english/CSP.md) for complete details.

---

## Configuration Examples

### Development Configuration

```env
# .env.development
IP_SERVER=127.0.0.1
PORT=3000
SECRET_KEY=dev-secret-key-change-in-production
ALLOWED_HOSTS=localhost,127.0.0.1

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
IP_SERVER=0.0.0.0
PORT=3000
SECRET_KEY=a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6q7r8s9t0
ALLOWED_HOSTS=example.com,www.example.com,api.example.com

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
IP_SERVER=0.0.0.0
PORT=3000
SECRET_KEY=${SECRET_KEY}
ALLOWED_HOSTS=localhost,app

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
IP_SERVER=127.0.0.1
PORT=3000
SECRET_KEY=change-me-in-production
ALLOWED_HOSTS=localhost,127.0.0.1

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
use runique::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = Settings::default_values();

    // Validations
    assert!(settings.server.secret_key.len() >= 32, "SECRET_KEY too short");
    assert!(!settings.allowed_hosts.is_empty(), "ALLOWED_HOSTS empty");

    RuniqueApp::new(settings).await?
        .routes(routes())
        .with_default_middleware()
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
    let mut settings = Settings::default_values();
    settings.server.secret_key = load_secret_key().await;

    RuniqueApp::new(settings).await?
        .routes(routes())
        .with_default_middleware()
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

Configure Runique securely and efficiently!

---

**Version:** 0.1.86 (January 17, 2026)
**License:** MIT