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

## Persistence — `with_db_fallback()`

By default, the store is purely in-memory. `with_db_fallback()` enables a database fallback for authenticated sessions:

- **Read**: memory first, DB on miss (session survives server restart)
- **Write**: synchronous to memory + asynchronous to DB for sessions with `user_id`
- **Anonymous sessions**: never persisted to DB

```rust
.middleware(|m| {
    m.with_session_db_fallback()
})
```

> The `runique_sessions` table must exist — it is created by the framework migrations.

---

## See also

| Section | Description |
| --- | --- |
| [Protection](/docs/en/session/protection) | Session protection |
| [Usage & configuration](/docs/en/session/usage) | Access and configuration |

## Back to summary

- [Sessions](/docs/en/session)
