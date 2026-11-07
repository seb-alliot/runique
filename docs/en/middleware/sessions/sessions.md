# Sessions (middleware)

## Default store

Runique uses `CleaningMemoryStore` by default — memory-first, with automatic cleanup of expired sessions and a database fallback for authenticated sessions (which survive restarts). Full detail: [Sessions](/docs/en/session).

---

## Session data in the database

A signed-in session is saved in `eihwaz_sessions` to survive a restart. Its content is **encrypted** there (XChaCha20-Poly1305, key derived from `SECRET_KEY`), with nothing to configure:

- reading the database (a backup, an SQL injection) doesn't give the content of the sessions, nor the CSRF token it holds;
- content altered in the database, or copied onto another row, is refused;
- `user_id` and `expires_at` stay in clear: the store needs them for its queries.

Decryption only happens when restoring after a restart. A session still in memory is only checked against its row.

**Changing `SECRET_KEY`** makes the saved sessions unreadable: at the next restart, their users sign in again. An attacker who also reads your `.env` has the key. A store given to `with_session_store(...)` isn't encrypted by Runique; a reset still closes its sessions, through `session_version`.

---

## Session durations

| Duration | Use case |
|-------|-------|
| `Duration::minutes(30)` | Short sessions (security) |
| `Duration::hours(2)` | Standard usage |
| `Duration::hours(24)` | Runique default |
| `Duration::days(7)` | Long sessions, for every account |

---

## Configuration

The duration of an **authenticated** session is set **only via the builder**
`with_session_duration(...)` — there is **no** `.env` variable for it (deliberate: a
**single source** of truth). It is the equivalent of Django's `SESSION_COOKIE_AGE`.

```rust
use runique::tower_sessions::cookie::time::Duration;

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
> `with_anonymous_session_duration(Duration)` (default 5 min). The CSRF token lives in the
> session, so a public form left open longer than that will have its token expire — see
> [CSRF Protection](/docs/en/middleware/csrf), "Token expiry & automatic refresh" section.

### "Remember me" — not possible yet

A different duration on one session ("remember me" box → 30 days) **does not hold today**:

- `login()` itself sets the session's expiry to the global duration: a `set_expiry` made before it is overwritten;
- after sign-in, the `session_ttl_upgrade` middleware sets the global duration again on **every** request of a signed-in session: a `set_expiry` made after `login` only lasts until the next request.

The only duration that applies is `with_session_duration`'s, the same for every signed-in session.

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
    let cart_id: Option<i32> = request.session
        .get("cart_id")
        .await
        .ok()
        .flatten();

    // The signed-in user: request.user, read from the database on every request
    let username = request.user.as_ref().map(|user| user.username.clone());

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
