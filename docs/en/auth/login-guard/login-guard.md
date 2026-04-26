# LoginGuard — Brute-force Protection

`LoginGuard` tracks failed login attempts **per username**, independently of the IP address.

Unlike IP rate limiting, there are no false positives from NAT or shared proxies: each account is tracked individually.

`LoginGuard` is used **in the handler**, not as middleware — it is the only way to access the submitted username without consuming the request body before `request.form()`.

---

## Full usage with `effective_key`

`effective_key` automatically determines the right key based on whether a username was submitted:

- Username submitted → key by account (targeted protection)
- Empty username → `"anonym:{ip}"` (per-IP protection, independent counters)

```rust
use runique::prelude::*;

static GUARD: LazyLock<LoginGuard> = LazyLock::new(|| {
    LoginGuard::new()
        .max_attempts(5)
        .lockout_secs(300)
});

pub async fn login(
    session: Session,
    State(db): State<DatabaseConnection>,
    mut request: Request,
) -> impl IntoResponse {
    let form: LoginForm = request.form();
    let username = form.username();
    let ip = /* extract IP from headers */;

    // username submitted → "alice"  |  empty username → "anonym:1.2.3.4"
    let key = LoginGuard::effective_key(&username, &ip);

    if GUARD.is_locked(&key) {
        let remaining = GUARD.remaining_lockout_secs(&key).unwrap_or(0);
        return (StatusCode::TOO_MANY_REQUESTS, format!("Try again in {remaining}s")).into_response();
    }

    match authenticate(&username, &form.password(), &db).await {
        Some(user) => {
            GUARD.record_success(&key);
            auth_login(&session, &db, user.id).await.unwrap();
            Redirect::to("/dashboard").into_response()
        }
        None => {
            GUARD.record_failure(&key);
            (StatusCode::UNAUTHORIZED, "Invalid credentials").into_response()
        }
    }
}
```

---

## Why in the handler and not middleware?

A middleware runs before the handler and cannot read the body without consuming it.
`request.form()` extracts the form exactly once — the username is only available after that extraction.

The session knows the authentication state, but at login time the user is not yet authenticated: `session.username` would always be `"anonym"` on this route, which does not protect the targeted account.

| Source | Available in middleware | Reliable for LoginGuard |
| --- | --- | --- |
| IP address | ✅ | ✅ (used by `effective_key` for anon) |
| Username (session) | ✅ | ❌ (always anon on `/login`) |
| Username (form body) | ❌ (consumes the body) | ✅ (via `request.form()` in the handler) |

---

## Configuration

```rust
LoginGuard::new().max_attempts(5).lockout_secs(300)    // 5 failures → 5 minute lockout
LoginGuard::new().max_attempts(3).lockout_secs(900)    // 3 failures → 15 minute lockout
LoginGuard::new().max_attempts(10).lockout_secs(3600)  // 10 failures → 1 hour lockout
```

---

## API

### `LoginGuard::new()`

Creates a LoginGuard with default values (5 attempts / 300 s).

### `.max_attempts(max: u32)`

Number of failures before account lockout.

### `.lockout_secs(secs: u64)`

Lockout duration in seconds.

### `LoginGuard::effective_key(username, ip) -> Cow<str>`

Returns the key to use based on context:

- Non-empty username → key by username (targeted account protection)
- Empty or missing username → `"anonym:{ip}"` (anonymous protection per IP)

Guarantees that two different anonymous IPs have independent counters — locking `"anonym:1.2.3.4"` does not affect `"anonym:5.6.7.8"`.

> **Why not a global `"anonym"` key?** A single abusive attempt would lock out every anonymous user worldwide. The per-IP key isolates each attacker.

### `record_failure(key)`

Increments the failure counter for this key. Call after each failed authentication attempt.

### `record_success(key)`

Resets the counter. Call after a successful login.

### `is_locked(key) -> bool`

Returns `true` if the failure count exceeds `max_attempts` and the lockout duration has not elapsed.

### `attempts(key) -> u32`

Current failure count for this key.

### `remaining_lockout_secs(key) -> Option<u64>`

Seconds remaining until unlock. `None` if not locked.

---

## Combining with IP Rate Limiting

Both mechanisms cover different attack vectors and are complementary:

| | `RateLimiter` | `LoginGuard` |
| --- | --- | --- |
| Key | IP address | Username or `anonym:{ip}` |
| Where | Middleware (automatic) | Handler (manual) |
| Target | All routes | Login only |
| Protects against | Volume attack per IP | Brute-force per account |
| False positives | Possible (NAT) | None (per account) |

```rust
static IP_LIMITER: LazyLock<RateLimiter> = LazyLock::new(|| RateLimiter::new().max_requests(10).retry_after(60));
static GUARD:      LazyLock<LoginGuard>  = LazyLock::new(|| LoginGuard::new().max_attempts(5).lockout_secs(300));

pub async fn login(/* ... */) -> impl IntoResponse {
    // 1. Volume per IP → RateLimiter (middleware or manual)
    if !IP_LIMITER.is_allowed(&ip) { /* 429 */ }

    // 2. Brute-force per account or anonymous IP → LoginGuard
    let key = LoginGuard::effective_key(&username, &ip);
    if GUARD.is_locked(&key) { /* 429 */ }

    // 3. Authentication
    match authenticate(&username, &password, &db).await {
        Some(user) => { GUARD.record_success(&key); /* ... */ }
        None       => { GUARD.record_failure(&key); /* ... */ }
    }
}
```

---

← [**Protection Middlewares**](/docs/en/auth/middleware) | [**Complete Example**](/docs/en/auth/exemple) →
