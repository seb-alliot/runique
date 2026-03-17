# Rate Limiting

Runique's rate limiter is granular: each handler can have its own limits, declared directly in the code.

---

## Usage

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

---

← [**Builder & Configuration**](https://github.com/seb-alliot/runique/blob/main/docs/en/middleware/builder/builder.md) | [**Flash Messages**](https://github.com/seb-alliot/runique/blob/main/docs/en/flash/09-flash-messages.md) →
