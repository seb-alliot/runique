# Rate Limiting

Runique's rate limiter is an IP-based application middleware, designed to be applied on sensitive routes (login, API, etc.).

Unlike a global nginx rule, it is **granular**: each route can have its own limits.

---

## Quick Start

```rust
use runique::prelude::*;
use std::sync::Arc;

let limiter = Arc::new(RateLimiter::new(5, 60)); // 5 requests per 60 seconds

let app = Router::new()
    .route("/login", post(login_view))
    .layer(axum::middleware::from_fn_with_state(
        limiter,
        rate_limit_middleware,
    ));
```

---

## Configuration

### In code

```rust
// 5 requests maximum per 60-second window
RateLimiter::new(5, 60)

// 3 requests per 10 seconds
RateLimiter::new(3, 10)

// 100 requests per minute
RateLimiter::new(100, 60)
```

### Via environment variables

```env
RUNIQUE_RATE_LIMIT_REQUESTS=60
RUNIQUE_RATE_LIMIT_WINDOW_SECS=60
```

```rust
let limiter = Arc::new(RateLimiter::from_env());
```

---

## Different Limits per Route

```rust
let login_limiter = Arc::new(RateLimiter::new(5, 60));   // strict
let api_limiter   = Arc::new(RateLimiter::new(100, 60)); // relaxed

let app = Router::new()
    .route("/login", post(login_view))
    .layer(axum::middleware::from_fn_with_state(
        login_limiter,
        rate_limit_middleware,
    ))
    .route("/api/data", get(api_view))
    .layer(axum::middleware::from_fn_with_state(
        api_limiter,
        rate_limit_middleware,
    ));
```

---

## Behavior

- The rate limit key is the request's **IP address**
- Supports `X-Forwarded-For` and `X-Real-IP` headers (reverse proxy)
- **Fixed window**: the counter resets after `window_secs` seconds
- Returns `429 Too Many Requests` when the limit is exceeded

---

## API

### `RateLimiter::new(max_requests, window_secs)`

| Parameter | Type | Description |
|-----------|------|-------------|
| `max_requests` | `u32` | Number of requests allowed in the window |
| `window_secs` | `u64` | Window duration in seconds |

### `RateLimiter::from_env()`

Builds from `RUNIQUE_RATE_LIMIT_REQUESTS` and `RUNIQUE_RATE_LIMIT_WINDOW_SECS`.

### `rate_limit_middleware`

Axum middleware function — pass to `from_fn_with_state`.

---

← [**Builder & Configuration**](https://github.com/seb-alliot/runique/blob/main/docs/en/middleware/builder/builder.md) | [**Flash Messages**](https://github.com/seb-alliot/runique/blob/main/docs/en/flash/09-flash-messages.md) →
