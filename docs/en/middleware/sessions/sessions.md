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

```rust
// Custom session duration
let app = RuniqueApp::builder(config)
    .with_session_duration(time::Duration::hours(2))
    .build()
    .await?;
```

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
