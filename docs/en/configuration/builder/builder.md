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
        "/" => view!{ views::index }, name = "index",
        "/about" => view!{ views::about }, name = "about",
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
        m.with_csp(|c| c.with_header_security(true))                        // Enable Content Security Policy
         .with_allowed_hosts(|h| h.enabled(true).host("mydomain.com"))      // Enable host validation
         .with_cache(true)              // Enable no-cache in dev
         .with_debug_errors(true)       // Enable detailed errors
    })
    .build()
    .await?;
```

Host validation is enabled via `.with_allowed_hosts(|h| h.enabled(true).host("..."))` in the builder — without this call, validation is disabled. No `.env` variable controls this behaviour.

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

### Framework logs

`RuniqueLog` centralises all logging configuration: the global tracing subscriber level **and** the internal framework categories.

The subscriber is initialised automatically by `build()` — **no call to `init_logging()` is needed in `main.rs`**.

Everything goes through `.with_log(|l| ...)` — the closure receives an empty `RuniqueLog` and returns the final configuration.

```rust
use tracing::Level;

// Fine-grained per-category control
RuniqueApp::builder(config)
    .with_log(|l| l
        .csrf(Level::WARN)
        .session(Level::WARN)
        .db(Level::INFO)
    )
    .routes(router)
    .build()
    .await?;
```

#### `subscriber_level` — subscriber level

Default: `"debug"` if `DEBUG=true` in `.env`, otherwise `"warn"`. The `RUST_LOG` environment variable always takes priority.

```rust
RuniqueApp::builder(config)
    .with_log(|l| l.subscriber_level("info"))
    .routes(router)
    .build()
    .await?;
```

#### `.dev()` — enable everything in development

Preset that activates all categories at `DEBUG` level. No-op if `DEBUG` is not `true` in `.env` — safe to use unconditionally.

```rust
// Dev only (no-op if DEBUG != true)
RuniqueApp::builder(config)
    .with_log(|l| l.dev())
    .routes(router)
    .build()
    .await?;

// Dev with subscriber level override
RuniqueApp::builder(config)
    .with_log(|l| l.dev().subscriber_level("info").db(Level::INFO))
    .routes(router)
    .build()
    .await?;
```

#### Available categories

| Category         | What is logged                                              |
| ---------------- | ----------------------------------------------------------- |
| `csrf`           | CSRF token detected in a GET URL (silent cleanup)           |
| `exclusive_login`| Sessions invalidated on exclusive login                     |
| `filter_fn`      | Failed `filter_fn` in the admin list view                   |
| `roles`          | Errors accessing the admin roles registry                   |
| `password_init`  | `password_init()` called more than once                     |
| `session`        | Memory watermarks, large records, cleanup errors            |
| `db`             | DB connection in progress / connection established          |

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
| ------------ | ------- | ----- |
| **Session duration** | 24 hours | |
| **Session store** | `MemoryStore` | |
| **CSRF protection** | ✅ Always enabled | Cannot be disabled |
| **Error handler** | ✅ Enabled | |
| **CSP** | Debug: ❌ / Prod: ✅ | Depends on mode |
| **Host validation** | Debug: ❌ / Prod: ✅ | Depends on mode |
| **Cache control** | ✅ Enabled | No-cache in debug |
| **Static files** | ❌ Disabled | Call `.statics()` |
| **Admin hot reload** | Follows `DEBUG` | Automatic via `is_debug()` |
| **Framework logs** | ❌ Disabled | Enable via `.with_log(\|l\| ...)` |

---

## See also

| Section | Description |
| --- | --- |
| [Environment variables](/docs/en/configuration/variables) | All `.env` variables |
| [Accessing config in code](/docs/en/configuration/code) | `RuniqueConfig`, validation |

## Back to summary

- [Configuration](/docs/en/configuration)
