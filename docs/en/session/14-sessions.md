# Sessions

## Overview

Runique uses `CleaningMemoryStore` as the default session store — a wrapper around an in-memory `HashMap` that adds:

- Automatic cleanup of expired sessions (configurable timer)
- Watermark-based memory protection against leaks and session-flooding attacks
- Protection for high-value sessions (authenticated users, shopping carts, multi-step forms)

Data is lost on server restart. For persistence, use an external store (Redis, database).

---

## CleaningMemoryStore

### Why

The tower-sessions `MemoryStore` does not implement expired-session cleanup. Without purging, every request from a bot without cookies creates a session that is never deleted — memory grows unboundedly.

`CleaningMemoryStore` solves this with three mechanisms:

| Mechanism | Trigger | Behavior |
|-----------|---------|----------|
| Periodic timer | Every 60s (configurable) | Deletes all expired sessions |
| Low watermark | 128 MB (configurable) | Async purge of expired anonymous sessions |
| High watermark | 256 MB (configurable) | Synchronous emergency purge + 503 refusal if still exceeded |

### Size estimation

Each record is estimated as: `24 bytes (UUID + expiry) + JSON length of session data`.

A warning is logged if a record exceeds 50 KB (image or file accidentally stored in session).

---

## Watermark system

### Low watermark (128 MB default)

When the total store size exceeds this threshold, a non-blocking background cleanup is launched via `tokio::spawn`. It removes **expired anonymous sessions** without blocking the current request.

### High watermark (256 MB default)

When the size exceeds this threshold at session creation time:

1. **Pass 1** — removes expired anonymous sessions
2. **Pass 2** — if still exceeded, removes all expired sessions (including authenticated)
3. **Refusal** — if still exceeded, returns `503 Service Unavailable`

Protected sessions (see below) are never sacrificed in pass 1.

---

## Session protection

### Automatic — `user_id`

Any session containing the `user_id` key is treated as authenticated and will never be removed under memory pressure (only expires normally).

This key is inserted automatically by Runique's authentication system on login.

### Manual — `session_active`

To protect a high-value anonymous session (cart, multi-step form, wizard), use `protect_session`:

```rust
use runique::middleware::auth::protect_session;

// Protect the session for 30 minutes
protect_session(&session, 60 * 30).await?;
```

The `session_active` key stores a future Unix timestamp. Protection expires automatically at that date — no manual cleanup needed.

To remove protection explicitly:

```rust
use runique::middleware::auth::unprotect_session;

unprotect_session(&session).await?;
```

### Protection logic

```
is_protected(record) = true if:
  - record contains "user_id"
  - OR record contains "session_active" with a future timestamp
```

---

## Accessing the session in handlers

```rust
pub async fn handler(request: Request) -> AppResult<Response> {
    // Read
    let user_id: Option<i32> = request.session.get("user_id").await.ok().flatten();

    // Write
    request.session.insert("cart_id", 42).await?;

    // Remove a key
    request.session.remove::<i32>("cart_id").await?;

    // Destroy the entire session
    request.session.flush().await?;
}
```

---

## `.env` configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `RUNIQUE_SESSION_CLEANUP_SECS` | `60` | Cleanup timer interval (seconds) |
| `RUNIQUE_SESSION_LOW_WATERMARK` | `134217728` (128 MB) | Proactive cleanup threshold (bytes) |
| `RUNIQUE_SESSION_HIGH_WATERMARK` | `268435456` (256 MB) | Emergency threshold + refusal (bytes) |

```env
# Example: cleanup every 30s, reduced watermarks for memory-constrained server
RUNIQUE_SESSION_CLEANUP_SECS=30
RUNIQUE_SESSION_LOW_WATERMARK=67108864
RUNIQUE_SESSION_HIGH_WATERMARK=134217728
```

---

## Builder configuration

```rust
let app = RuniqueApp::builder(config)
    // Session lifetime
    .with_session_duration(time::Duration::hours(2))
    // Custom watermarks
    .with_session_memory_limit(64 * 1024 * 1024, 128 * 1024 * 1024)
    // Cleanup interval
    .with_session_cleanup_interval(30)
    .build()
    .await?;
```

---

## Example — protecting a shopping cart

```rust
pub async fn add_to_cart(request: Request, item: Item) -> AppResult<Response> {
    // Store cart in session
    request.session.insert("cart", &cart).await?;

    // Protect the session for 2 hours against emergency cleanup
    protect_session(&request.session, 60 * 60 * 2).await?;

    Ok(redirect("/cart"))
}

pub async fn checkout_complete(request: Request) -> AppResult<Response> {
    // Clear cart and remove protection
    request.session.remove::<Cart>("cart").await?;
    unprotect_session(&request.session).await?;

    Ok(redirect("/confirmation"))
}
```

---

## Next steps

← [**Authentication**](https://github.com/seb-alliot/runique/blob/main/docs/en/13-authentification.md) | [**Environment variables**](https://github.com/seb-alliot/runique/blob/main/docs/en/15-env.md) →
