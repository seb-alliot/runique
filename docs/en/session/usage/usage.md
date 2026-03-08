# Session access & configuration

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

## See also

| Section | Description |
| --- | --- |
| [Store & watermarks](https://github.com/seb-alliot/runique/blob/main/docs/en/session/store/store.md) | CleaningMemoryStore, purge mechanisms |
| [Protection](https://github.com/seb-alliot/runique/blob/main/docs/en/session/protection/protection.md) | Session protection |

## Back to summary

- [Sessions](https://github.com/seb-alliot/runique/blob/main/docs/en/session/14-sessions.md)
