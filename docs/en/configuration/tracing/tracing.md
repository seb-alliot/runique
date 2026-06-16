# Structured Tracing

Runique exposes a structured tracing system via `RuniqueLog`. By default a **console** subscriber is installed and **domains are opt-in**: until a domain is enabled, its events are not emitted. A few critical sites always emit (see [Unconditional errors](#unconditional-errors-always-active)).

## Quick activation in development

```rust
RuniqueApp::builder(config)
    .with_log(|l| l.dev())   // everything at DEBUG if DEBUG=true
    // ...
```

`.dev()` is a no-op if `DEBUG` is not `true` — safe to use unconditionally.

---

## Granular configuration

```rust
.with_log(|l| l
    .forms(|f| f
        .validate(Level::DEBUG)
        .finalize(Level::DEBUG)
    )
    .admin(|a| a
        .crud(Level::INFO)
        .auth(Level::WARN)
    )
    .auth(|a| a
        .login(Level::INFO)
        .reset(Level::WARN)
    )
    .mailer(|m| m.send(Level::INFO))
    .builder(|b| b
        .templates(Level::INFO)
        .middleware(Level::DEBUG)
        .routes(Level::INFO)
        .statics(Level::INFO)
    )
    .rate_limit(Level::WARN)
)
```

---

## Available domains

### `forms` — Form pipeline

| Field | When | Logged data |
|-------|------|-------------|
| `field` | Field registration | name, type, required |
| `set_value` | Value assigned by `fill()` | name, value (password masked) |
| `validate` | Validation result | field, ok/error, global error count |
| `render` | HTML rendering | field, ok/error |
| `finalize` | Hash/file move | field, ok/error |

### `admin` — Admin panel

| Field | When | Logged data |
|-------|------|-------------|
| `auth` | Access check + CSRF fail | resource, action |
| `crud` | Dispatch + create/edit/delete result | resource, action, ok/error |
| `list` | Dispatch + list result | resource, rows, total, page |
| `bulk` | Bulk actions | resource, action |

### `auth` — Authentication

| Field | When | Logged data |
|-------|------|-------------|
| `login` | Session creation | user_id, username, is_superuser, exclusive, db_persist |
| `reset` | Password reset flow | email, step (token generated / email sent / invalid / ok / error) |

### `mailer`

| Field | When | Logged data |
|-------|------|-------------|
| `send` | Email dispatch | backend, to, subject, ok/error |

### `builder` — Startup (one-time)

| Field | When | Logged data |
|-------|------|-------------|
| `templates` | Tera loading | internal, user, total |
| `registry` | Admin resources | count |
| `middleware` | Middleware stack | count + slot + name for each entry |
| `routes` | URL registry | count |
| `statics` | Static files | static_url, static_dir, media_url, media_dir |

### `errors` — HTTP error pages

| Field | When | Logged data |
|-------|------|-------------|
| `http` | Handled HTTP error (404/validation/forbidden) | method, path, type / error |
| `render` | Error-template render failure (404/429/500) | template, error — **WARN floor** (always visible, see below) |

```rust
.with_log(|l| l.errors(|e| e.http(Level::INFO).render(Level::WARN)))
```

### Flat fields on `RuniqueLog`

| Field | When | Logged data |
|-------|------|-------------|
| `rate_limit` | Request blocked | ip, retry_after |
| `csrf` | CSRF token detected in a GET URL | path |
| `session` | Session store operations | event |
| `db` | Database queries | query, duration |
| `host_validation` | Host rejected | host |

---

## Log outputs

By default Runique installs a console subscriber (`Stdout`, colors). Configure one or more **cumulative** outputs via `.output()`:

```rust
use runique::prelude::{LogOutput, LogRotation};

.with_log(|l| l
    .output(LogOutput::stdout())                 // colored console
    .output(LogOutput::file("logs/app.json"))    // JSON (inferred from the .json extension)
    .output(LogOutput::file("logs/app.log")      // plain text
        .rotation(LogRotation::Daily)))
```

- The **format is inferred from the extension**: `.json` → structured JSON (one event per line), otherwise plain text.
- File writing is **non-blocking**; logs are flushed cleanly on shutdown.
- Rotation: `Daily` (default), `Hourly`, `Never`.
- `RUNIQUE_LOG_FILE=/path/app.json` adds a file output at runtime, without recompiling.

## Custom sink

To route logs to an arbitrary destination (database, HTTP collector, message queue), implement `LogSink` — no `tracing` type is exposed:

```rust
use runique::prelude::{LogOutput, LogRecord, LogSink};

struct MySink;

impl LogSink for MySink {
    fn log(&self, record: &LogRecord) {
        // record.level / target / message / file / line / fields
        // Must not block: for async, enqueue into your own channel.
    }
}

.with_log(|l| l.output(LogOutput::sink(MySink)))
```

The sink receives **all** events in the process (Runique **and** your application); tell them apart via `record.target` (Runique events have a target starting with `runique`). Runique deliberately ships **no** database sink (it would overload the DB) — `LogSink` is the hook to wire one yourself.

## External subscriber

If your application owns its own `tracing` subscriber (custom layer stack, OpenTelemetry…), declare `.external()`: Runique installs **nothing** and leaves you the single global slot, while still **emitting** its events to the `tracing` facade (your subscriber receives them).

Minimal:

```rust
.with_log(|l| l.external())
```

Full:

```rust
use runique::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // You install YOUR subscriber, before build()
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    RuniqueApp::builder(RuniqueConfig::from_env())
        .with_database().await
        .routes(url::urlpatterns())
        .with_log(|l| l.external())   // Runique does not install its subscriber
        .build().await?
        .run().await
}
```

To **ignore** Runique's internal logs, filter their target in your `EnvFilter`:

```rust
tracing_subscriber::fmt()
    .with_env_filter("info,runique=off")   // keep your logs, mute Runique's
    .init();
```

In `.external()` mode, `.output()` destinations are ignored (your subscriber decides where logs go).

---

## Unconditional errors (always active)

Regardless of tracing config, some events are always emitted:

- **Invalid template at startup** — `tracing::error!` (template name + line) before startup aborts.
- **Critical server errors** (500: database, IO, template, internal) — `tracing::error!`.
- **Security-critical sites with a `WARN` floor** — even with the domain disabled, these failures emit at least at `WARN`, because a silent failure there would break a guarantee: session ID rotation (anti-fixation), invalidation of other sessions (exclusive login), session persistence at login, reset-email dispatch, and error-template render failure (`errors.render`).

---

## See also

- [Environment variables](/docs/en/configuration/variables) — `RUST_LOG`, `DEBUG`
- [Programmatic configuration](/docs/en/configuration/builder) — `.with_log()`
