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

pub fn routes() -> Routes {
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
    .middleware(|m| m.with_debug_errors(true))  // Mounts the error handler (default: true)
    .routes(router)
    .build()
    .await?;
```

The error handler is mounted by default; `DEBUG` decides what it shows (a detailed page with `DEBUG=true`, a generic one otherwise). `with_debug_errors(false)` removes it altogether: errors then leave with no Runique page.

### Middlewares

All middleware configuration goes through `.middleware(|m| { ... })` where `m` is a `MiddlewareStaging`:

```rust
let app = RuniqueApp::builder(config)
    .routes(router)
    .middleware(|m| {
        m.with_csp(|c| c)              // CSP is always active — customize it here
         .with_allowed_hosts(|h| h.host("mydomain.com"))      // Enable host validation
         .with_cache(true)              // Keep the HTTP cache in dev (no no-cache headers)
         .with_debug_errors(true)       // Enable detailed errors
    })
    .build()
    .await?;
```

Host validation is declared via `.with_allowed_hosts(|h| h.host("..."))` in the builder: on with `RUNIQUE_ENV=prod`, off with `RUNIQUE_ENV=dev`, and `.enabled(...)` has the last word. Without this call, nothing is validated.

> **`is_dev()`** — global helper available via `use runique::prelude::*`. Returns `true` if `RUNIQUE_ENV=dev`. Read once (`LazyLock`), after loading `.env`, available everywhere without passing a parameter. **`is_debug()`** does the same for `DEBUG`, which only drives the detailed error pages.

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

### Public URL — `with_public_url`

The application's public address, the one a visitor types: the base of every absolute link it sends out — password reset links, from the public pages and from the admin — and of the admin's "view site" link unless the admin sets its own:

```rust
let app = RuniqueApp::builder(config)
    .with_public_url("https://mysite.com")
    .with_password_reset(|pr| pr)
    .build()
    .await?;
```

In production (`RUNIQUE_ENV=prod`), the app **refuses to boot** without it as soon as the password reset or the admin is enabled: a link built from the request's `Host` header would go wherever the client decides (`Host: evil.com` → the reset token goes to the attacker). With `RUNIQUE_ENV=dev`, the public URL is always `http://127.0.0.1:{PORT}`, even when `.with_public_url()` is called: links lead to your machine, not to the production site.

Your own links (account activation, an email pointing to the site…) use the same rule through `request.public_url()`: the public URL, else `None`, never the `Host`. Never build them from the `Host` header yourself.

```rust
let Some(base) = request.public_url() else {
    // production without .with_public_url(): send no link rather than a forged one
    return request.render("auth/registered.html");
};
let activate_url = format!("{base}/activate/{token}");
```

It's called **once**: a second `.with_public_url(...)` doesn't compile ("`with_public_url()` has already been called on this builder"), rather than silently replacing the first. Write the production address in it: development ignores it.

### Framework logs

`RuniqueLog` centralises all logging configuration: the global tracing subscriber level **and** the internal framework categories.

The subscriber is initialised automatically by `build()` — **no call to `init_logging()` is needed in `main.rs`**.

Everything goes through `.with_log(|l| ...)` — the closure receives an empty `RuniqueLog` and returns the final configuration.

```rust
use runique::prelude::*; // LogLevel = tracing::Level

// Fine-grained per-category control — each submodule takes its own closure
RuniqueApp::builder(config)
    .with_log(|l| l
        .middleware(|m| m.csrf(LogLevel::WARN))
        .session(|s| s.store(LogLevel::WARN))
        .db(|d| d.connect(LogLevel::INFO))
    )
    .routes(router)
    .build()
    .await?;
```

#### `subscriber_level` — subscriber level

Default: `"debug"` if `RUNIQUE_ENV=dev`, otherwise `"warn"`. The `RUST_LOG` environment variable always takes priority.

```rust
RuniqueApp::builder(config)
    .with_log(|l| l.subscriber_level("info"))
    .routes(router)
    .build()
    .await?;
```

#### `.dev()` — enable everything in development

Preset that activates all categories at `DEBUG` level. No-op if `RUNIQUE_ENV` is not `dev` — safe to use unconditionally.

```rust
// Dev only (no-op if RUNIQUE_ENV != dev)
RuniqueApp::builder(config)
    .with_log(|l| l.dev())
    .routes(router)
    .build()
    .await?;

// Dev with subscriber level override
RuniqueApp::builder(config)
    .with_log(|l| l.dev().subscriber_level("info").db(|d| d.connect(LogLevel::INFO)))
    .routes(router)
    .build()
    .await?;
```

#### Available categories

`RuniqueLog` only exposes root methods per submodule (`forms`, `middleware`, `session`, `auth`, `admin`, `db`, `mailer`, `migration`, `templates`, `errors`, `builder`), each taking a closure into its own nested builder — there is no flat `.csrf()`/`.session()` method directly on `RuniqueLog`:

| Category (nested leaf) | Access | What is logged |
| ----------------------- | ------ | --------------- |
| `csrf`            | `.middleware(\|m\| m.csrf(...))`         | CSRF token detected in a GET URL (silent cleanup) |
| `exclusive_login` | `.session(\|s\| s.exclusive_login(...))` | Sessions invalidated on exclusive login |
| `password_init`   | `.auth(\|a\| a.password_init(...))`      | `password_init()` called more than once |
| `store`           | `.session(\|s\| s.store(...))`           | Memory watermarks, large records, cleanup errors |
| `connect`         | `.db(\|d\| d.connect(...))`              | DB connection in progress / connection established |

### Secondary database — `with_custom_db`

Attaches any `Any + Send + Sync + 'static` value into the `RuniqueEngine`.
Useful for connecting MongoDB, Redis, or any other external source.
Can be called multiple times with different types.

```rust
let mongo = mongodb::Client::with_uri_str("mongodb://localhost:27017").await?;
let redis = redis::Client::open("redis://localhost")?;

let app = RuniqueApp::builder(config)
    .with_custom_db(mongo)
    .with_custom_db(redis)
    .routes(url::routes())
    .build()
    .await?;
```

Accessing in a handler via `engine.extension::<T>()` — returns `Option<Arc<T>>`:

```rust
async fn my_handler(req: Request) -> Response {
    if let Some(mongo) = req.engine.extension::<mongodb::Client>() {
        let collection = mongo.database("mydb").collection::<Document>("users");
    }
    if let Some(redis) = req.engine.extension::<redis::Client>() {
        // ...
    }
}
```

`engine.custom_db::<T>()` is kept as a backward-compatible alias — both methods are equivalent.

### Administration

`.with_admin(...)` only exists when `runique` is compiled with the `admin` feature. See [Admin](/docs/en/admin).

### Static files

```rust
let app = RuniqueApp::builder(config)
    .statics()     // Enable static files
    // or
    .static_files(|s| s.enabled(false))  // Explicitly disable
    .build()
    .await?;
```

## Once-only settings

Some settings replace their value on every call: declaring them twice would silently lose the first. So they're called **once per builder**, and a second call **doesn't compile**:

| Setting | Note |
| --- | --- |
| `.with_public_url(…)` | |
| `.routes(…)` | build a single `Router` (merge or nest the others into it) |
| `.with_log(…)` | set every category in the same call |
| `.with_password_reset(…)` | |
| `.with_database(…)` / `.with_database_config(…)` | one **or** the other, once |
| `.with_session_duration(…)` | |
| `.with_mailer(…)` / `.with_mailer_from_env()` | one **or** the other, once |

The error names the setting, for instance:

```
error[E0277]: `routes()` has already been called on this builder
```

Settings that **compose** — `.core(…)`, `.middleware(…)`, `.static_files(…)`, `.with_admin(…)`, `.with_custom_db(…)`, `.statics()` — continue from the existing state: they can be called several times.

The state lives in the builder's type (`RuniqueAppBuilder<S>`), invisible while calls are chained. A function returning a half-built builder has to spell it: `runique::app::builder::state::{No, Yes}`, one slot per setting, in the order of the table above.

---

## Default values

| Configuration | Default | Notes |
| ------------ | ------- | ----- |
| **Session duration** | 24 hours | |
| **Session store** | `CleaningMemoryStore` | |
| **CSRF protection** | ✅ Always enabled | Cannot be disabled |
| **Error handler** | ✅ Enabled | |
| **CSP + security headers** | ✅ Always on | Unconditional, regardless of mode; `.with_csp(...)` customizes, doesn't enable |
| **Host validation** | ❌ Disabled | No `.env` variable controls it — call `.with_allowed_hosts(...)` |
| **Cache control** | ✅ Enabled | No-cache in development; `.with_cache(true)` turns it off |
| **Static files** | ✅ Enabled | Served under `STATIC_URL` and `MEDIA_URL`; `.static_files(\|s\| s.enabled(false))` to disable (files served by Nginx, for instance) |
| **Public URL** | — | Required in production when the password reset or the admin is enabled — `.with_public_url(...)` |
| **Admin hot reload** | Follows `RUNIQUE_ENV` | Automatic via `is_dev()` |
| **Framework logs** | ❌ Disabled | Enable via `.with_log(\|l\| ...)` |

---

## See also

| Section | Description |
| --- | --- |
| [Environment variables](/docs/en/env) | All `.env` variables |
| [Accessing config in code](/docs/en/configuration/code) | `RuniqueConfig`, validation |

## Back to summary

- [Configuration](/docs/en/configuration)
