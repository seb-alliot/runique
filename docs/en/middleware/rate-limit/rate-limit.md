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
    .rate_limit("/upload-image", "upload_image", view!(upload_image_submit), 5, 60, vec![Method::POST])
}
```

Multiple routes sharing the **same counter**:

```rust
.rate_limit_many(5, 60, vec![Method::POST], vec![
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

- The rate limit key is the request's **IP address** — the `ClientIp` extension set by the `trusted_proxies` middleware (always active) if present, otherwise the raw TCP peer address (`ConnectInfo`)
- `X-Forwarded-For` support goes through this `trusted_proxies` mechanism (validated against a trusted CIDR list), not a direct header read here — `X-Real-IP` is never read anywhere in the code
- **Fixed window**: the counter resets after `retry_after` seconds
- Returns `429 Too Many Requests` when the limit is exceeded, with a `Retry-After: <seconds>` header

> **⚠️ Security:** the reliability of the IP key depends entirely on `trusted_proxies` configuration (see [Trusted Proxies](/docs/en/middleware/trusted-proxies)) — it decides whether `X-Forwarded-For` is validated or ignored. Without a correctly declared trusted reverse proxy, an attacker can forge that header to switch rate-limit keys at will.

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

### `.only_methods(methods: Vec<Method>)`

Restricts rate limiting to the specified HTTP methods. Requests with other methods pass through freely without being counted.

```rust
use axum::http::Method;

RateLimiter::new()
    .max_requests(5)
    .retry_after(60)
    .only_methods(vec![Method::POST])
```

Use case: protect a login route against brute force on POST submissions only, while letting GET (display the form) pass freely.

> Without `.only_methods()`, all HTTP methods are counted.

---

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
