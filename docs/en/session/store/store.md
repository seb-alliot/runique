# Store & watermarks

## CleaningMemoryStore

### Why

The tower-sessions `MemoryStore` does not implement cleanup of expired sessions. Without purging, every request from a bot without cookies creates a session that is never deleted — memory grows unboundedly.

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

Protected sessions are never sacrificed in pass 1.

---

---

## Persistence — database fallback

By default, the store is purely in-memory. When the `orm` feature is active, a database fallback is **automatically enabled** for authenticated sessions — no additional configuration required.

- **Read**: memory first, DB on miss (session survives server restart)
- **Write**: synchronous to memory + asynchronous to DB for sessions with `user_id`
- **Anonymous sessions**: never persisted to DB

> The `eihwaz_sessions` table must exist — it is created by the framework migrations.

---

## Configuration

All the thresholds described above are tunable from the builder, inside the `.middleware()` block. No manual store construction is needed — the framework instantiates `CleaningMemoryStore` and applies these values.

```rust
use runique::prelude::*;
use time::Duration;

let app = RuniqueApp::builder(config)
    // Inactivity TTL before expiration (authenticated sessions)
    .with_session_duration(Duration::hours(2))
    .middleware(|m| {
        m
            // Low / high watermark — in bytes
            .with_session_memory_limit(64 * 1024 * 1024, 128 * 1024 * 1024)
            // Periodic purge timer interval — in seconds
            .with_session_cleanup_interval(30)
            // One connected device at a time per user
            .with_exclusive_login(true)
    })
    .build()
    .await?;
```

| Mechanism (section above) | Builder method | Default |
|---------------------------|----------------|---------|
| Periodic timer | `with_session_cleanup_interval(secs)` | 60 s |
| Low / High watermark | `with_session_memory_limit(low, high)` | 128 MB / 256 MB |
| Authenticated session TTL | `with_session_duration(Duration)` | — |
| Anonymous session TTL | `with_anonymous_session_duration(Duration)` | — |
| Exclusive login (one device) | `with_exclusive_login(bool)` | `false` |

> Watermarks are expressed in **bytes**: `64 * 1024 * 1024` = 64 MB. The timer interval is in **seconds**.

---

## See also

| Section | Description |
| --- | --- |
| [Protection](/docs/en/session/protection) | Session protection |
| [Usage & configuration](/docs/en/session/usage) | Access and configuration |

## Back to summary

- [Sessions](/docs/en/session)
