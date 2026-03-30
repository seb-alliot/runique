# Rate Limiting

Runique provides two rate limiting approaches: **declarative** at the route level, or **fine-grained** inside the handler.

---

## Declarative approach — route level

Directly in `url.rs`, via the `RouterExt` trait:

```rust
use runique::prelude::*;

pub fn routes() -> Router {
    urlpatterns! {
        "/" => view!{ index }, name = "index",
        // ...
    }
    // Single route
    .rate_limit("/upload-image", "upload_image", view!(upload_image_submit), 5, 60)
}
```

Multiple routes sharing the **same counter**:

```rust
.rate_limit_many(5, 60, vec![
    ("/upload-image".into(), "upload_image".into(), view!(upload_image_submit)),
    ("/register".into(),     "register".into(),     view!(register)),
])
```

> `spawn_cleanup` is called automatically — no memory leak.

---

## Handler approach — fine-grained logic

For per-user limits, per-action logic, or a custom key:

```rust
use runique::prelude::*;

static LIMITER: LazyLock<RateLimiter> = LazyLock::new(|| {
    RateLimiter::new()
        .max_requests(10)
        .retry_after(60)
});

pub async fn login(/* ... */) -> impl IntoResponse {
    if !LIMITER.is_allowed(&ip) {
        return StatusCode::TOO_MANY_REQUESTS.into_response();
    }
    // ...
}
```

---

## When to use which?

| Case                                          | Approach                      |
| --------------------------------------------- | ----------------------------- |
| Public route, global IP-based limit           | Declarative (`.rate_limit()`) |
| Limit per authenticated user                  | Handler                       |
| Different logic depending on context          | Handler                       |
| Multiple routes sharing the same quota        | `.rate_limit_many()`          |

---

## Configuration

```rust
RateLimiter::new().max_requests(5).retry_after(60)    // 5 requests per minute
RateLimiter::new().max_requests(3).retry_after(300)   // 3 requests per 5 minutes
RateLimiter::new().max_requests(100).retry_after(60)  // 100 requests per minute
```

---

## Behavior

- The rate limit key is the request's **IP address**
- Supports `X-Forwarded-For` and `X-Real-IP` headers (reverse proxy)
- **Fixed window**: the counter resets after `retry_after` seconds
- Returns `429 Too Many Requests` when the limit is exceeded, with a `Retry-After: <seconds>` header

> **⚠️ Security:** This middleware trusts `X-Forwarded-For` and `X-Real-IP` headers. Ensure your reverse proxy (nginx, etc.) controls these headers and does not allow them to be forged by clients. Without a trusted proxy, an attacker can bypass rate limiting by modifying these headers.

---

## API

### `RateLimiter::new()`

Creates a rate limiter with default values (60 req / 60 s).

### `.max_requests(max: u32)`

Number of requests allowed in the window.

### `.retry_after(secs: u64)`

Window duration in seconds.

### `is_allowed(key: &str) -> bool`

Returns `true` if the key is under the limit, `false` otherwise.

### `retry_after_secs(key: &str) -> u64`

Seconds remaining until the window resets for this key. Returns `0` if the window has already expired or the key is unknown. Used to populate the `Retry-After` header in 429 responses.

### `.spawn_cleanup(period: Duration)`

Spawns a background task that periodically purges expired entries. Without this, the internal map grows indefinitely for each distinct IP. Call it once after building the limiter.

```rust
let limiter = RateLimiter::new().max_requests(5).retry_after(60);
limiter.spawn_cleanup(Duration::from_secs(60));
let limiter = Arc::new(limiter);
```

> With the declarative approach (`.rate_limit()` / `.rate_limit_many()`), `spawn_cleanup` is called automatically.

---

← [**Builder & Configuration**](/docs/en/middleware/builder) | [**Flash Messages**](/docs/en/flash) →
