# ⚙️ Configuration

## RuniqueConfig

All configuration is handled via `.env` and loaded into the `RuniqueConfig` struct.

### Load configuration

```rust
use runique::config_runique::RuniqueConfig;

let config = RuniqueConfig::from_env()?;

// Access variables
println!("Debug: {}", config.debug);
println!("Port: {}", config.port);
println!("DB: {}", config.database_url);
```

---

## Environment Variables

### Server

| Variable | Default | Description |
|----------|---------|-------------|
| `IP_SERVER` | 127.0.0.1 | Listening IP |
| `PORT` | 3000 | Server port |
| `DEBUG` | true | Debug mode (templates, logs, etc.) |

**Example:**
```env
# Server Configuration
IP_SERVER=127.0.0.1
PORT=3000

DEBUG=true
# Database Configuration (SQLite by default)

# Secret key for csrf management
SECRETE_KEY=your_secret_key_here

# Required for any DB other than SQLite
DB_ENGINE=postgres
DB_USER=postgres
DB_PASSWORD=password
DB_HOST=localhost
DB_PORT=5432
DB_NAME=runique

# Optional convenience shortcut
DATABASE_URL=postgresql://myuser:mypassword@localhost:5432/mydb

# Allowed hosts for production
ALLOWED_HOSTS=example.com,www.example.com,.api.example.com,localhost,127.0.0.1
```

### Database

| Variable | Default | Description |
|----------|---------|-------------|
| `DATABASE_URL` | - | Full connection string |
| `DB_ENGINE` | postgres | postgres, sqlite, mysql |
| `DB_USER` | postgres | DB user |
| `DB_PASSWORD` | - | DB password |
| `DB_HOST` | localhost | DB host |
| `DB_PORT` | 5432 | DB port |
| `DB_NAME` | runique | Database name |

**PostgreSQL:**
```env
DATABASE_URL=postgres://user:password@localhost:5432/dbname
DB_ENGINE=postgres
DB_USER=postgres
DB_PASSWORD=secret
DB_HOST=localhost
DB_PORT=5432
DB_NAME=runique
```

**SQLite (dev):**
```env
DATABASE_URL=sqlite:runique.db?mode=rwc
```

### Templates & Assets

| Variable | Default | Description |
|----------|---------|-------------|
| `TEMPLATES_DIR` | templates | Templates directory |
| `STATICFILES_DIRS` | static | Static assets directory |
| `MEDIA_ROOT` | media | Media (uploads) directory |

**Example:**
```env
TEMPLATES_DIR=templates
STATICFILES_DIRS=static:demo-app/static
MEDIA_ROOT=uploads
```

### Security

| Variable | Default | Description |
|----------|---------|-------------|
| `SECRETE_KEY` | - | CSRF secret key (⚠️ CHANGE IN PROD!) |
| `ALLOWED_HOSTS` | * | Allowed hosts (comma-separated) |

**Example:**
```env
SECRETE_KEY=your_secret_key_change_this_in_production
ALLOWED_HOSTS=localhost,127.0.0.1,example.com,.api.example.com
```

**ALLOWED_HOSTS patterns:**
- `localhost` - Exact match
- `*` - Wildcard all hosts (DANGER in production!)
- `.example.com` - Matches example.com and *.example.com

---

## Complete .env File

```env
# ============================================================================
# SERVER CONFIGURATION
# ============================================================================
IP_SERVER=127.0.0.1
PORT=3000
DEBUG=true

# ============================================================================
# DATABASE CONFIGURATION
# ============================================================================
# PostgreSQL (Recommended for production)
DATABASE_URL=postgres://postgres:password@localhost:5432/runique
DB_ENGINE=postgres
DB_USER=postgres
DB_PASSWORD=password
DB_HOST=localhost
DB_PORT=5432
DB_NAME=runique

# SQLite (Development only)
# DATABASE_URL=sqlite:runique.db?mode=rwc

# ============================================================================
# TEMPLATES & STATIC FILES
# ============================================================================
TEMPLATES_DIR=templates
STATICFILES_DIRS=static
MEDIA_ROOT=media

# ============================================================================
# SECURITY
# ============================================================================
# IMPORTANT: Generate a new key for production!
# python3 -c "import secrets; print(secrets.token_urlsafe(32))"
SECRETE_KEY=your_secret_key_here_change_in_production

# Format: comma-separated (no spaces)
# .example.com matches example.com and *.example.com
ALLOWED_HOSTS=localhost,127.0.0.1
```

---

## Advanced Configuration

### Production Mode

```env
DEBUG=false
PORT=443
IP_SERVER=0.0.0.0

# HTTPS
SECRETE_KEY=<generated dynamically>

# Strict hosts
ALLOWED_HOSTS=example.com,www.example.com,.api.example.com

# Externalized DB
DATABASE_URL=postgres://user:pwd@prod-db.example.com:5432/runique
```

### Development Mode

```env
DEBUG=true
PORT=3000
IP_SERVER=127.0.0.1

SECRETE_KEY=any_dev_key
ALLOWED_HOSTS=*

DATABASE_URL=sqlite:runique.db?mode=rwc
```

### Testing Mode

```env
DEBUG=true
SECRETE_KEY=test_key
ALLOWED_HOSTS=localhost,127.0.0.1

# In-memory database (SQLite)
DATABASE_URL=sqlite::memory:
```

---

## Generate a secret key

```bash
# Python
python3 -c "import secrets; print(secrets.token_urlsafe(32))"

# Rust
cargo run --example generate_secret

# OpenSSL
openssl rand -base64 32
```

---

## Access configuration in code

```rust
use runique::config_runique::RuniqueConfig;

async fn my_handler(ctx: RuniqueContext) -> Response {
    let config = &ctx.engine.config;

    println!("Debug mode: {}", config.debug);
    println!("Database: {}", config.database_url);
    println!("Secret key: {}", config.secret_key);
    println!("Allowed hosts: {:?}", config.allowed_hosts);
}
```

### Conditional configuration

```rust
if template.config.debug {
    // Debug mode: detailed logs, template reload
} else {
    // Production: template cache, no sensitive logs
}

if template.config.debug.allowed_hosts.contains("*") {
    // ⚠️ Warning: all hosts are allowed
}
```

---

## Configuration validation

Configuration is validated at startup:

```rust
let config = RuniqueConfig::from_env()
    .expect("Invalid configuration");

// Returns Err() if:
// - DATABASE_URL missing
// - SECRETE_KEY missing
// - Invalid variables
```

---

## Programmatic Configuration (Outside .env)

Beyond the `.env` file, the `RuniqueApp` builder offers methods to customize your app directly in code.

### Builder methods

#### 📦 Database

```rust
use sea_orm::Database;

let db = Database::connect("postgresql://localhost/mydb").await?;

let app = RuniqueApp::builder(config)
    .with_database(db)
    .routes(router)
    .build()
    .await?;
```

#### 🔄 Routes

```rust
let router = Router::new()
    .route("/", get(home))
    .route("/about", get(about));

let app = RuniqueApp::builder(config)
    .routes(router)  // Set routes
    .build()
    .await?;
```

#### ⏱️ Session duration

```rust
use tower_sessions::cookie::time::Duration;

let app = RuniqueApp::builder(config)
    .with_session_duration(Duration::hours(2))  // Default: 24h
    .routes(router)
    .build()
    .await?;
```

**Duration examples:**
```rust
Duration::hours(2)      // 2 hours
Duration::days(7)       // 7 days
Duration::minutes(30)   // 30 minutes
Duration::seconds(3600) // 1 hour
```

#### 💾 Custom session store

By default, Runique uses `MemoryStore`. For production, use Redis, PostgreSQL, or another store:

```rust
use tower_sessions::RedisStore;

let redis_pool = /* your Redis pool */;
let session_store = RedisStore::new(redis_pool);

let app = RuniqueApp::builder(config)
    .with_session_store(session_store)  // ⚠️ Returns RuniqueAppBuilderWithStore
    .with_session_duration(Duration::hours(12))
    .routes(router)
    .build()
    .await?;
```

**Note:** `with_session_store()` returns a different type (`RuniqueAppBuilderWithStore<Store>`), but you can keep chaining methods normally.

#### 🛡️ Middlewares

CSRF protection is always enabled (not toggleable) to keep forms working. You can still tweak other middlewares:

```rust
let app = RuniqueApp::builder(config)
    .with_sanitize(false)      // Disable sanitization (default: true)
    .with_error_handler(false) // Disable error handler (default: true)
    .routes(router)
    .build()
    .await?;
```

**Use cases:**
- `with_sanitize(false)` - Custom input validation
- `with_error_handler(false)` - Custom error handling

#### 📁 Static files

```rust
let app = RuniqueApp::builder(config)
    .with_static_files()  // Enable static files service
    .routes(router)
    .build()
    .await?;
```

### Full examples

#### Minimal setup (development)

```rust
use runique::{RuniqueApp, config_runique::RuniqueConfig};
use axum::{Router, routing::get};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = RuniqueConfig::from_env()?;
    let db = Database::connect(&config.database_url).await?;

    let router = Router::new()
        .route("/", get(home));

    let app = RuniqueApp::builder(config)
        .with_database(db)
        .routes(router)
        .build()
        .await?;

    app.run().await
}
```

#### Production setup with Redis

```rust
use tower_sessions::cookie::time::Duration;
use tower_sessions::RedisStore;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = RuniqueConfig::from_env()?;
    let db = Database::connect(&config.database_url).await?;

    // Redis session store for production
    let redis_url = std::env::var("REDIS_URL")?;
    let redis_pool = redis::Client::open(redis_url)?;
    let session_store = RedisStore::new(redis_pool);

    let router = Router::new()
        .route("/", get(home))
        .route("/login", post(login));

    let app = RuniqueApp::builder(config)
        .with_database(db)
        .with_session_store(session_store)
        .with_session_duration(Duration::hours(6))  // 6h sessions
        .routes(router)
        .with_static_files()
        .build()
        .await?;

    app.run().await
}
```

#### Test configuration

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_app_config() {
        let config = RuniqueConfig::from_env().unwrap();
        let db = Database::connect("sqlite::memory:").await.unwrap();

        let app = RuniqueApp::builder(config)
            .with_database(db)
            .with_session_duration(Duration::minutes(5))  // Short sessions
            .with_error_handler(false)  // Explicit errors in tests
            .routes(test_router())
            .build()
            .await
            .unwrap();

        // Tests...
    }
}
```

### Recommended call order

```rust
RuniqueApp::builder(config)
    // 1. Database
    .with_database(db)

    // 2. Session (optional)
    .with_session_store(store)  // ⚠️ If used, call before other builder methods
    .with_session_duration(Duration::hours(2))

    // 3. Middlewares (optional)
    // CSRF is always on by default (not toggleable)
    .with_sanitize(true)
    .with_error_handler(true)

    // 4. Routes (required)
    .routes(router)

    // 5. Static files (optional)
    .with_static_files()

    // 6. Build (required)
    .build()
    .await?
```

### Default values

If you configure nothing, defaults are:

| Configuration | Default |
|--------------|---------|
| **Session duration** | 24 hours |
| **Session store** | `MemoryStore` |
| **CSRF protection** | ✅ Enabled (not toggleable) |
| **Sanitize** | ✅ Enabled |
| **Error handler** | ✅ Enabled |
| **Static files** | ❌ Disabled (call `.with_static_files()`) |

---

## Next steps

→ [**Routing**](https://github.com/seb-alliot/runique/blob/main/docs/en/04-routing.md)
