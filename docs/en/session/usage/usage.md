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

```rust,ignore
.middleware(|m| {
    m.with_session_memory_limit(5 * 1024 * 1024, 10 * 1024 * 1024)
     .with_session_cleanup_interval(5)
})
```

---

## Cookie security defaults

Session cookies are configured with the following security attributes by default:

| Attribute | Value | Description |
| --- | --- | --- |
| `HttpOnly` | `true` | Always enabled — inaccessible to JavaScript |
| `SameSite` | `Strict` | Blocks cross-site requests |
| `Secure` | `true` in production | HTTPS only (disabled in debug mode) |

These defaults are set automatically by the builder and cannot be overridden without modifying the framework.

---

## Builder configuration

```rust
let app = RuniqueApp::builder(config)
    // Session lifetime (available directly on the builder)
    .with_session_duration(time::Duration::hours(2))
    // Watermarks and cleanup interval (via .middleware())
    .middleware(|m| {
        m.with_session_memory_limit(64 * 1024 * 1024, 128 * 1024 * 1024)
         .with_session_cleanup_interval(30)
    })
    .build()
    .await?;
```

---

## See also

| Section | Description |
| --- | --- |
| [Store & watermarks](/docs/en/session/store) | CleaningMemoryStore, purge mechanisms |
| [Protection](/docs/en/session/protection) | Session protection |

## Back to summary

- [Sessions](/docs/en/session)
