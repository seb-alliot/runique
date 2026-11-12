# Structured Tracing

Runique exposes a structured tracing system via `RuniqueLog`. By default a **console** subscriber is installed and **domains are opt-in**: until a domain is enabled, its events are not emitted. A few critical sites always emit (see [Unconditional errors](#unconditional-errors-always-active)).

## What `.env` turns on by itself

With nothing written in `.with_log(...)`, the channels follow `DEBUG` and `RUNIQUE_ENV`:

| `.env` | Channels on | Level |
| --- | --- | --- |
| `DEBUG=true` (with or without `RUNIQUE_ENV=dev`) | **all**: the useful channels, plus the chatty ones (`forms.*`, `admin.auth`, `admin.list`) | useful at `INFO`, chatty at `DEBUG` |
| `RUNIQUE_ENV=dev`, without `DEBUG` | the useful channels: sign-ins, reset, CSRF refusals, rate limit, hosts, redirects, anti-bot, admin writes and bulk actions, database, mailer, HTTP errors, boot | `INFO` |
| production, without `DEBUG` | none: only the warnings with a `WARN` floor come out (see below) | — |

A channel set by hand keeps its level. `.manual()` switches this off: only the channels written in `.with_log(...)` are on.

```rust
.with_log(|l| l.manual().auth(|a| a.login(LogLevel::INFO)))
```

> **⚠️ `DEBUG=true` in production.** It is allowed, to try a deployment on a test address or to look into a problem live. But while it is on:
>
> - every visitor sees the detailed error pages;
> - the `forms.set_value` channel writes **the value of every field typed in** to the logs: email addresses, names, messages. Only passwords are hidden, and the CSRF token is replaced by a message;
> - those logs become personal data: who reads them, how long they are kept and how they are backed up fall under the GDPR;
> - the global filter moves to `debug`, which also lets the dependencies' `debug` logs through.
>
> Keep it for a test address, or a short window on a live site. Turn it off as soon as the problem is found, and purge the logs written meanwhile.

> **Language of the logs.** The CSRF, admin, session and startup messages added in 4.1 follow the application's language (`set_lang`), like the messages shown to visitors. Field names (`at`, `reason`, `resource`, `error`…) stay in English; the value of `reason` and `outcome` is translated. To filter or aggregate logs by their text, keep the default language (`en`).

---

## Quick activation in development

```rust
RuniqueApp::builder(config)
    .with_log(|l| l.dev())   // everything at DEBUG if RUNIQUE_ENV=dev
    // ...
```

`.dev()` is a no-op if `RUNIQUE_ENV` is not `dev` — safe to use unconditionally.

---

## Granular configuration

```rust
.with_log(|l| l
    .forms(|f| f
        .validate(LogLevel::DEBUG)
        .finalize(LogLevel::DEBUG)
    )
    .admin(|a| a
        .crud(LogLevel::INFO)
        .auth(LogLevel::WARN)
    )
    .auth(|a| a
        .login(LogLevel::INFO)
        .reset(LogLevel::WARN)
    )
    .mailer(|m| m.send(LogLevel::INFO))
    .builder(|b| b
        .templates(LogLevel::INFO)
        .middleware(LogLevel::DEBUG)
        .routes(LogLevel::INFO)
        .statics(LogLevel::INFO)
    )
    .middleware(|m| m.rate_limit(LogLevel::WARN))
)
```

---

## Available domains

### `forms` — Form pipeline

| Field | When | Logged data |
|-------|------|-------------|
| `field` | Field registration | name, type, required |
| `set_value` | Value assigned by `fill()` | name, value (password masked; CSRF token replaced by a message) |
| `validate` | Validation result | field, ok/error, global error count |
| `render` | HTML rendering | field, ok/error |
| `finalize` | Hash/file move | field, ok/error |

### `admin` — Admin panel

| Field | When | Logged data |
|-------|------|-------------|
| `auth` | Access check + CSRF fail | resource, action |
| `crud` | A write through an admin form (create, edit, delete) | resource, outcome (`saved`, `save refused`, `deleted`, `delete refused`) |
| `list` | List shown; failed dashboard counts | resource, page, rows, total, filters, search |
| `bulk` | Bulk action done or refused | resource, action, rows |

### `auth` — Authentication

| Field | When | Logged data |
|-------|------|-------------|
| `login` | Session creation | user_id, username, is_superuser, exclusive, db_persist |
| `reset` | Password reset flow | email, step (token generated / email sent / invalid / ok / error) |
| `password_init` | `password_init()` called more than once | — |

### `mailer`

| Field | When | Logged data |
|-------|------|-------------|
| `send` | Email dispatch | backend, to, subject, ok/error |

### `builder` — Startup (one-time)

| Field | When | Logged data |
|-------|------|-------------|
| `templates` | Tera loading | internal, user, total |
| `registry` | The admin at boot: its configuration (prefix, page sizes), each registered model, each route declared `admin`, the pruned rights | prefix, models, key, title, display, filters, actions, form_fields, pruned |
| `middleware` | Middleware stack | count + slot + name for each entry |
| `routes` | URL registry | count |
| `statics` | Static files | static_url, static_dir, media_url, media_dir |

### `templates` — Template engine

| Field | When | Data logged |
|-------|------|-------------|
| `load` | Reserved: no event emitted for now (loading is reported under `builder.templates`) | — |
| `render` | Reserved: no event emitted for now (a render failure goes through `errors`) | — |

### `migration` — Migrations

| Field | When | Data logged |
|-------|------|-------------|
| `plan` | Reserved: no event emitted for now (the CLI writes to standard output) | — |
| `apply` | Reserved: no event emitted for now | — |
| `rollback` | Reserved: no event emitted for now (a plan's rollback is written to standard error) | — |

### `errors` — HTTP error pages

| Field | When | Logged data |
|-------|------|-------------|
| `http` | Handled HTTP error (404/validation/forbidden) | method, path, type / error |
| `render` | Error-template render failure (404/429/500) | template, error — **WARN floor** (always visible, see below) |

```rust
.with_log(|l| l.errors(|e| e.http(LogLevel::INFO).render(LogLevel::WARN)))
```

### `middleware` — Security (`.middleware(|m| m....)`)

| Field | When | Logged data |
|-------|------|-------------|
| `csrf` | The CSRF journal. Each refusal with its reason, wherever it is decided: middleware (invalid token, missing token, form sent from another site), form (no token, or one that is no longer the session's), admin sign-in and sign-out, file sent before the token. Each new token and each rotation (sign-in, sign-out). Failure reading or writing it in the session | at, reason / error |
| `csp` | Reserved: no event emitted for now | — |
| `cors` | Reserved: no event emitted for now | — |
| `rate_limit` | Request blocked | ip, retry_after |
| `host_validation` | Host rejected | host |
| `open_redirect` | Redirect refused | location |
| `anti_bot` | Honeypot filled (form refused); failure reading the trap's name in the session | form / error |
| `https` | Request received over HTTP redirected to HTTPS; the port 80 server stopping in ACME mode — **WARN floor** for that stop | host / error |

### `session` — Store (`.session(|s| s....)`)

| Field | When | Logged data |
|-------|------|-------------|
| `store` | Session store operations (watermarks, purge) | event |
| `cleanup` | Error of the periodic purge of expired sessions (memory and database) | error |
| `exclusive_login` | Invalidation of other sessions | — |

### `db` — Database (`.db(\|d\| d....)`)

| Field | When | Logged data |
|-------|------|-------------|
| `connect` | DB connection in progress / established | — |
| `query` | Database queries | query, duration |

These three domains have no direct method on `RuniqueLog` — each goes through its own closure (`.middleware(|m| ...)`, `.session(|s| ...)`, `.db(|d| ...)`), same as `forms`/`admin`/`auth` above.

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

    let db = DatabaseConfig::from_env()?.build().connect().await?;

    RuniqueApp::builder(RuniqueConfig::from_env())
        .with_database(db)
        .routes(url::routes())
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
- **Security-critical sites with a `WARN` floor** — even with the domain disabled, these failures emit at least at `WARN`, because a silent failure there would break a guarantee: session ID rotation (anti-fixation), invalidation of other sessions (exclusive login), session persistence at login, reset-email dispatch, error-template render failure (`errors.render`), closing the session of a deleted or deactivated account, saving a session to the database, pruning rights at boot (`builder.registry`) the port 80 server stopping in ACME mode (`middleware.https`), an admin sign-in with neither a `LoginGuard` nor a limiter (outside development), list options with no effect (`links`, `search_help`, `empty_value`) and Runique's stylesheets not found (`STATIC_RUNIQUE_PATH`).

---

## See also

- [Environment variables](/docs/en/env) — `RUST_LOG`, `RUNIQUE_ENV`
- [Programmatic configuration](/docs/en/configuration/builder) — `.with_log()`
