# Rate Limiting

Runique provides two rate limiting approaches: **declarative** at the route level, or **fine-grained** inside the handler.

---

## Declarative approach — route level

Directly in `url.rs`, via the `RouterExt` trait:

```rust
use runique::prelude::*;

pub fn routes() -> Routes {
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

> This integration calls `spawn_cleanup` automatically: expired entries are purged periodically,
> but this is not a strict memory cap.
> Expired entries are also removed when accessed. Periodic cleanup removes entries that are no longer used.

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
An expired entry is removed as soon as it is accessed.

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

Spawns a background task that periodically purges expired entries. For a manual handler or middleware integration, call it once while a Tokio runtime is active. The effective period is clamped between 1 and 60 seconds.

```rust
let limiter = RateLimiter::new().max_requests(5).retry_after(60);
limiter.spawn_cleanup(Duration::from_secs(60));
let limiter = Arc::new(limiter);
```

> With the declarative approach (`.rate_limit()` / `.rate_limit_many()`), `spawn_cleanup` is called automatically.
> The admin builder also starts it when a rate limiter is configured.

---

## Several instances

A `RateLimiter` counts in the memory of its process, and so does `LoginGuard`. With several instances of the app, each one keeps its own counters: a limit of 5 attempts per minute becomes 5 per instance, so 15 behind three instances. If the limit has to hold across instances, enforce it in front of them too (reverse proxy, load balancer) or route each visitor to the same instance.

---

## Current limit: no memory cap

**The limit.** `RateLimiter` and `LoginGuard` keep one entry per key in the process memory: one per IP address for the limiter, one per username typed for `LoginGuard`. Expired entries are removed when they are read and by the periodic cleanup (every 1 to 60 seconds), but **no maximum number of entries is enforced**. A flood of distinct keys (many IP addresses, made-up usernames) grows the memory until they expire.

**Why.** A cap means deciding what to do once it is reached, and every simple answer helps the attacker: refusing new keys blocks everyone's login, evicting the oldest erases a lockout in progress, no longer counting lets brute force through. `LoginGuard` also counts names that don't exist on purpose: counting only real accounts would turn the lockout into a way to find out which usernames exist. A cap handling that avoids these traps is planned for a 4.x release.

**Meanwhile:**

- put the application behind a reverse proxy that **rate-limits requests itself**. A proxy that only forwards doesn't protect these tables. With nginx, `limit_req` keeps its counters in a fixed-size zone, which bounds the memory:

  ```nginx
  limit_req_zone $binary_remote_addr zone=login:10m rate=10r/m;

  location = /login {
      limit_req zone=login burst=5 nodelay;
      proxy_pass http://127.0.0.1:3000;
  }
  ```

  Do the same for the admin login, at its own path;
- combine `LoginGuard` with a per-IP limiter on the login route (see [LoginGuard](/docs/en/auth/login-guard)): the limiter caps how many names one address can try;
- keep durations reasonable (`retry_after`, `lockout_secs`): the longer they are, the longer entries stay in memory;
- monitor the process memory in production.

---

← [**Builder & Configuration**](/docs/en/middleware/builder) | [**Flash Messages**](/docs/en/flash) →
