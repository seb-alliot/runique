# ⚙️ Configuration

## RuniqueConfig

All configuration is handled via `.env` and loaded into a `RuniqueConfig` struct.

### Load configuration

```rust
use runique::prelude::*;

let config = RuniqueConfig::from_env()?;

// Access variables:
println!("Debug: {}", config.debug);
println!("Port: {}", config.port);
println!("DB: {}", config.database_url);
```

---

## Environment Variables

### Server

| Variable    | Default   | Description                        |
| ----------- | --------- | ---------------------------------- |
| `IP_SERVER` | 127.0.0.1 | Listening IP address               |
| `PORT`      | 3000      | Server port                        |
| `DEBUG`     | true      | Debug mode (templates, logs, etc.) |

**Example:**

```env
# Server Configuration
IP_SERVER=127.0.0.1
PORT=3000

DEBUG=true
# Database Configuration (SQLite by default)

# Secret key for csrf management
SECRETE_KEY=your_secret_key_here

# To be completed for any DB other than SQLite
DB_ENGINE=postgres
DB_USER=postgres
DB_PASSWORD=password
DB_HOST=localhost
DB_PORT=5432
DB_NAME=runique

# Optional, not mandatory unless for personal use
DATABASE_URL=postgresql://monuser:monmotdepasse@localhost:5432/mabase

# Allowed hosts for production
ALLOWED_HOSTS=exemple.com,www.exemple.com,.api.exemple.com,localhost,127.0.0.1
```

### Database

| Variable       | Default   | Description             |
| -------------- | --------- | ----------------------- |
| `DATABASE_URL` | -         | Full connection string  |
| `DB_ENGINE`    | postgres  | postgres, sqlite, mysql |
| `DB_USER`      | postgres  | DB user                 |
| `DB_PASSWORD`  | -         | DB password             |
| `DB_HOST`      | localhost | DB host                 |
| `DB_PORT`      | 5432      | DB port                 |
| `DB_NAME`      | runique   | Database name           |

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

| Variable           | Default   | Description               |
| ------------------ | --------- | ------------------------- |
| `TEMPLATES_DIR`    | templates | Templates directory       |
| `STATICFILES_DIRS` | static    | Static assets directory   |
| `MEDIA_ROOT`       | media     | Media directory (uploads) |

**Example:**

```env
TEMPLATES_DIR=templates
STATICFILES_DIRS=static:demo-app/static
MEDIA_ROOT=uploads
```

### Security

| Variable | Default | Description |
| --- | --- | --- |
| `SECRETE_KEY` | - | CSRF secret key (⚠️ CHANGE IN PROD!) |
| `ALLOWED_HOSTS` | * | Allowed hosts (comma-separated) |

**Example:**

```env
SECRETE_KEY=your_secret_key_change_this_in_production
ALLOWED_HOSTS=localhost,127.0.0.1,example.com,.api.example.com
```

**ALLOWED_HOSTS patterns:**

* `localhost` — Exact match
* `*` — Wildcard for all hosts (DANGEROUS in production!)
* `.example.com` — Matches example.com and *.example.com

---

## Full .env File

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

## Generate a Secret Key

```bash
# Python
python3 -c "import secrets; print(secrets.token_urlsafe(32))"

# Rust
cargo run --example generate_secret

# OpenSSL
openssl rand -base64 32
```

---

## Access Configuration in Code

```rust
use runique::prelude::*;

pub async fn my_handler(request: Request) -> AppResult<Response> {
    let config = &request.engine.config;

    println!("Debug mode: {}", config.debug);
    println!("Port: {}", config.server.port);
    println!("IP: {}", config.server.ip_server);
    println!("Allowed hosts: {:?}", config.security.allowed_hosts);
    println!("Secret key: {}", config.security.secrete_key);
}
```

### Conditional configuration

```rust
if request.engine.config.debug {
    // Debug mode: detailed logs, templates reloaded
} else {
    // Production mode: template cache, no sensitive logs
}

if request.engine.config.security.allowed_hosts.contains("*") {
    // ⚠️ Warning: all hosts are allowed (dangerous in production!)
}
```

---

## Configuration Validation

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

In addition to configuring via the `.env` file, the builder provides methods to customize your application directly.

### Classic builder

```rust
let app = RuniqueApp::builder(config)
    .routes(url::routes())
    .with_database(db)
    .with_error_handler(true)
    .with_csp(true)
    .with_allowed_hosts(true)
    .with_cache(true)
    .with_static_files()
    .build()
    .await?;

app.run().await?;
```

### Intelligent Builder (new)

The Intelligent Builder simplifies configuration and automatically manages middleware order:

```rust
use runique::app::RuniqueAppBuilder as IntelligentBuilder;

let app = IntelligentBuilder::new(config)
    .routes(url::routes())
    .with_database(db)
    .statics()
    .build()
    .await?;

app.run().await?;
```

### Builder methods

#### 📦 Database

```rust
// Option 1: direct connection
let db_config = DatabaseConfig::from_env()?.build();
let db = db_config.connect().await?;

let app = RuniqueApp::builder(config)
    .with_database(db)
    .routes(router)
    .build()
    .await?;

// Option 2: deferred configuration (Intelligent Builder)
let db_config = DatabaseConfig::from_env()?.build();

let app = IntelligentBuilder::new(config)
    .with_database_config(db_config)  // Connects during .build()
    .routes(router)
    .build()
    .await?;
```

#### 🔄 Routes

```rust
use runique::{urlpatterns, view};

pub fn routes() -> Router {
    urlpatterns! {
        "/" => view!{ views::index }, name = "index",
        "/about" => view!{ views::about }, name = "about",
    }
}

let app = RuniqueApp::builder(config)
    .routes(routes())
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
```

#### 🛡️ Middlewares (classic builder)

```rust
let app = RuniqueApp::builder(config)
    .with_error_handler(true)   // Error capture (default: true)
    .with_csp(true)             // CSP & security headers (default: false in debug)
    .with_allowed_hosts(true)   // Host validation (default: false in debug)
    .with_cache(true)           // No-cache in dev (default: true)
    .routes(router)
    .build()
    .await?;
```

#### 🛡️ Middlewares (Intelligent Builder)

The Intelligent Builder uses the **debug/production profile** for default values:

```rust
let app = IntelligentBuilder::new(config)
    .routes(router)
    .middleware(|m| {
        m.disable_csp();             // Disable CSP
        m.disable_host_validation(); // Disable host validation
    })
    .build()
    .await?;
```

> In debug mode, CSP and host validation are disabled by default. In production, everything is enabled.

#### 📁 Static files

```rust
// Classic builder
let app = RuniqueApp::builder(config)
    .with_static_files()
    .build()
    .await?;

// Intelligent Builder
let app = IntelligentBuilder::new(config)
    .statics()     // Enable static files
    // or
    .no_statics()  // Explicitly disable
    .build()
    .await?;
```

### Default values

| Configuration | Default | Notes |
| --- | --- | --- |
| **Session duration** | 24 hours | |
| **Session store** | `MemoryStore` | |
| **CSRF protection** | ✅ Always enabled | Cannot be disabled |
| **Error handler** | ✅ Enabled | |
| **CSP** | Debug: ❌ / Prod: ✅ | Depends on mode |
| **Host validation** | Debug: ❌ / Prod: ✅ | Depends on mode |
| **Cache control** | ✅ Enabled | No-cache in debug |
| **Static files** | ❌ Disabled | Call `.statics()` or `.with_static_files()` |

---

## Next Steps

← [**Architecture**](https://github.com/seb-alliot/runique/blob/main/docs/en/02-architecture.md) | [**Routing**](https://github.com/seb-alliot/runique/blob/main/docs/en/04-routing.md) →
