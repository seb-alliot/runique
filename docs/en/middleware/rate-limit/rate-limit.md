# Rate Limiting

Runique's rate limiter is granular: each handler can have its own limits, declared directly in the code.

---

## Usage

```rust
use runique::prelude::*;

static LIMITER: LazyLock<RateLimiter> = LazyLock::new(|| RateLimiter::new(10, 60));

pub async fn login(/* ... */) -> impl IntoResponse {
    if !LIMITER.is_allowed(&ip) {
        return StatusCode::TOO_MANY_REQUESTS.into_response();
    }
    // ...
}
```

`RateLimiter::new(max_requests, window_secs)` — the dev declares their limits wherever and however they want.

---

## Configuration

### In code

```rust
RateLimiter::new(5, 60)    // 5 requests per minute
RateLimiter::new(3, 300)   // 3 requests per 5 minutes
RateLimiter::new(100, 60)  // 100 requests per minute
```

### Via environment variables

```env
RUNIQUE_RATE_LIMIT_REQUESTS=60
RUNIQUE_RATE_LIMIT_WINDOW_SECS=60
```

```rust
static LIMITER: LazyLock<RateLimiter> = LazyLock::new(RateLimiter::from_env);
```

---

## Behavior

- The rate limit key is the request's **IP address**
- Supports `X-Forwarded-For` and `X-Real-IP` headers (reverse proxy)
- **Fixed window**: the counter resets after `window_secs` seconds
- Returns `429 Too Many Requests` when the limit is exceeded

> **⚠️ Security:** This middleware trusts `X-Forwarded-For` and `X-Real-IP` headers. Ensure your reverse proxy (nginx, etc.) controls these headers and does not allow them to be forged by clients. Without a trusted proxy, an attacker can bypass rate limiting by modifying these headers.

---

## API

### `RateLimiter::new(max_requests, window_secs)`

| Parameter | Type | Description |
|-----------|------|-------------|
| `max_requests` | `u32` | Number of requests allowed in the window |
| `window_secs` | `u64` | Window duration in seconds |

### `RateLimiter::from_env()`

Builds from `RUNIQUE_RATE_LIMIT_REQUESTS` and `RUNIQUE_RATE_LIMIT_WINDOW_SECS`.

### `is_allowed(key: &str) -> bool`

Returns `true` if the key is under the limit, `false` otherwise.

---

← [**Builder & Configuration**](https://github.com/seb-alliot/runique/blob/main/docs/en/middleware/builder/builder.md) | [**Flash Messages**](https://github.com/seb-alliot/runique/blob/main/docs/en/flash/09-flash-messages.md) →
