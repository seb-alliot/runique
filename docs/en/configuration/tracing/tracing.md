# Structured Tracing

Runique exposes an opt-in, per-domain tracing system via `RuniqueLog`. Each domain activates independently — nothing is logged by default.

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

### Flat fields on `RuniqueLog`

| Field | When | Logged data |
|-------|------|-------------|
| `rate_limit` | Request blocked | ip, retry_after |
| `csrf` | CSRF token detected in a GET URL | path |
| `session` | Session store operations | event |
| `db` | Database queries | query, duration |
| `host_validation` | Host rejected | host |

---

## Unconditional errors (always active)

Regardless of tracing config, some errors are always logged via `tracing::error!`:

- **Invalid template** — if a Tera template fails to load, the template name and error (including line number) are logged before startup aborts.

---

## See also

- [Environment variables](../variables/variables.md) — `RUST_LOG`, `DEBUG`
- [Programmatic configuration](../builder/builder.md) — `.with_log()`
