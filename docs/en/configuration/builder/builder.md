# Programmatic Configuration — Builder

`RuniqueApp::builder(config)` returns a `RuniqueAppBuilder`. There is only one builder — there are no two separate versions.

## Minimal example

```rust
let app = RuniqueApp::builder(config)
    .routes(url::routes())
    .with_database(db)
    .statics()
    .build()
    .await?;

app.run().await?;
```

---

## Available methods

### Database

```rust
// Option 1: direct connection (DatabaseConnection)
let db_config = DatabaseConfig::from_env()?.build();
let db = db_config.connect().await?;

let app = RuniqueApp::builder(config)
    .with_database(db)       // takes a DatabaseConnection
    .routes(router)
    .build()
    .await?;

// Option 2: deferred connection (DatabaseConfig)
let db_config = DatabaseConfig::from_env()?.build();

let app = RuniqueApp::builder(config)
    .with_database_config(db_config)  // connection happens at .build()
    .routes(router)
    .build()
    .await?;
```

### Routes

```rust
use runique::{urlpatterns, view};

pub fn routes() -> Router {
    urlpatterns! {
        "/" => view!{ GET => views::index }, name = "index",
        "/about" => view!{ GET => views::about }, name = "about",
    }
}

let app = RuniqueApp::builder(config)
    .routes(routes())
    .build()
    .await?;
```

### Error handling

```rust
let app = RuniqueApp::builder(config)
    .with_error_handler(true)   // Enable error handler (default: true)
    .routes(router)
    .build()
    .await?;
```

### Middlewares

All middleware configuration goes through `.middleware(|m| { ... })` where `m` is a `MiddlewareStaging`:

```rust
let app = RuniqueApp::builder(config)
    .routes(router)
    .middleware(|m| {
        m.with_csp(true)                // Enable Content Security Policy
         .with_host_validation(true)    // Enable host validation
         .with_cache(true)              // Enable no-cache in dev
         .with_debug_errors(true)       // Enable detailed errors
    })
    .build()
    .await?;
```

In `DEBUG=true` mode, `with_csp` and `with_host_validation` are **disabled by default**. In production, everything is enabled. The `RUNIQUE_ENABLE_*` variables in `.env` take priority over the defaults.

> **`is_debug()`** — global helper available via `use runique::prelude::*`. Returns `true` if `DEBUG=true` in `.env`. Read once at startup (`LazyLock`), available everywhere without passing a parameter.

### Session duration

```rust
use tower_sessions::cookie::time::Duration;

let app = RuniqueApp::builder(config)
    .with_session_duration(Duration::hours(2))  // Default: 24h
    .routes(router)
    .build()
    .await?;
```

Or via `.middleware()` for advanced options:

```rust
.middleware(|m| {
    m.with_session_duration(Duration::hours(2))
     .with_anonymous_session_duration(Duration::minutes(5))
     .with_session_memory_limit(128 * 1024 * 1024, 256 * 1024 * 1024)
})
```

### Static files

```rust
let app = RuniqueApp::builder(config)
    .statics()     // Enable static files
    // or
    .no_statics()  // Explicitly disable
    .build()
    .await?;
```

---

## Default values

| Configuration | Default | Notes |
|--------------|---------|-------|
| **Session duration** | 24 hours | |
| **Session store** | `MemoryStore` | |
| **CSRF protection** | ✅ Always enabled | Cannot be disabled |
| **Error handler** | ✅ Enabled | |
| **CSP** | Debug: ❌ / Prod: ✅ | Depends on mode |
| **Host validation** | Debug: ❌ / Prod: ✅ | Depends on mode |
| **Cache control** | ✅ Enabled | No-cache in debug |
| **Static files** | ❌ Disabled | Call `.statics()` |
| **Admin hot reload** | Follows `DEBUG` | Automatic via `is_debug()` |
| **Log level** | Follows `DEBUG` | `debug` if `DEBUG=true`, `warn` otherwise |

---

## See also

| Section | Description |
| --- | --- |
| [Environment variables](https://github.com/seb-alliot/runique/blob/main/docs/en/configuration/variables/variables.md) | All `.env` variables |
| [Accessing config in code](https://github.com/seb-alliot/runique/blob/main/docs/en/configuration/code/code.md) | `RuniqueConfig`, validation |

## Back to summary

- [Configuration](https://github.com/seb-alliot/runique/blob/main/docs/en/configuration/03-configuration.md)
