# Sessions (middleware)

## Default store

Runique uses `MemoryStore` by default (in-memory data, lost on restart).

---

## Session durations

| Duration | Use case |
|-------|-------|
| `Duration::minutes(30)` | Short sessions (security) |
| `Duration::hours(2)` | Standard usage |
| `Duration::hours(24)` | Runique default |
| `Duration::days(7)` | "Remember me" |

---

## Configuration

The duration of an **authenticated** session is set **only via the builder**
`with_session_duration(...)` — there is **no** `.env` variable for it (deliberate: a
**single source** of truth). It is the equivalent of Django's `SESSION_COOKIE_AGE`.

```rust
use tower_sessions::cookie::time::Duration;

// Custom session duration
let app = RuniqueApp::builder(config)
    .with_session_duration(Duration::hours(2))   // default if never called: 24h
    .build()
    .await?;
```

This value applies **everywhere, consistently**:

- the session **cookie** (browser expiry);
- the **`eihwaz_sessions`** row in the database (`expires_at` column);
- the per-request **refresh** (`session_ttl_upgrade` middleware).

`login()` reads this same value (set at build) → cookie, database and refresh **cannot
diverge**. If `with_session_duration` is never called, the default is **24h** (86,400 s).

> **Anonymous** sessions (logged-out visitors) have their own, shorter duration via
> `with_anonymous_session_duration(Duration)` (default 5 min).

### "Remember me" (per-session duration)

The global default above fits most cases. For a **different duration on a specific session**
(e.g. a "remember me" checkbox → 30 days), keep the global default and call `set_expiry` on
that session at login:

```rust
use tower_sessions::{Expiry, cookie::time::Duration};

if remember_me {
    request.session.set_expiry(Some(Expiry::OnInactivity(Duration::days(30))));
}
```

Since `expires_at` is stored **per row** in `eihwaz_sessions`, no schema change is needed:
only that one session is extended.

### Custom store (production)

```rust
// Example with a Redis store
let app = RuniqueApp::builder(config)
    .middleware(|m| m.with_session_store(RedisStore::new(client)))
    .build()
    .await?;
```

---

## Accessing session data in handlers

```rust
pub async fn dashboard(request: Request) -> AppResult<Response> {
    // Read a session value
    let user_id: Option<i32> = request.session
        .get("user_id")
        .await
        .ok()
        .flatten();

    // Write a value
    let _ = request.session.insert("last_visit", "2026-02-06").await;
}
```

> For the full session system with watermarks, see [Sessions](/docs/en/session).

---

## See also

| Section | Description |
| --- | --- |
| [CSRF](/docs/en/middleware/csrf) | CSRF protection |
| [Builder](/docs/en/middleware/builder) | Builder configuration |

## Back to summary

- [Middleware & Security](/docs/en/middleware)
