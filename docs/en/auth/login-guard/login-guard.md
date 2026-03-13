# LoginGuard — Brute-force Protection

`LoginGuard` tracks failed login attempts **per username**, independently of the IP address.

Unlike IP rate limiting, there are no false positives from NAT or shared proxies: each account is tracked individually.

---

## Quick Start

```rust
use runique::prelude::*;
use std::sync::Arc;

// 5 failures → 5 minute lockout
let guard = Arc::new(LoginGuard::new(5, 300));
```

---

## Usage in a Login Handler

```rust
async fn login_post(
    session: Session,
    State(guard): State<Arc<LoginGuard>>,
    State(db): State<DatabaseConnection>,
    Prisme(form): Prisme<LoginForm>,
) -> impl IntoResponse {
    let username = form.username();

    // 1. Check if account is locked
    if guard.is_locked(&username) {
        let remaining = guard.remaining_lockout_secs(&username).unwrap_or(0);
        // show error with `remaining` seconds
        return (StatusCode::TOO_MANY_REQUESTS, "Account temporarily locked").into_response();
    }

    // 2. Authenticate
    match authenticate(&username, &form.password(), &db).await {
        Some(user) => {
            guard.record_success(&username);
            login(&session, user.id, &user.username).await.unwrap();
            Redirect::to("/dashboard").into_response()
        }
        None => {
            guard.record_failure(&username);
            // show login error
            (StatusCode::UNAUTHORIZED, "Invalid credentials").into_response()
        }
    }
}
```

---

## Configuration

### In code

```rust
// 5 failures → 5 minute lockout
LoginGuard::new(5, 300)

// 3 failures → 15 minute lockout
LoginGuard::new(3, 900)

// 10 failures → 1 hour lockout
LoginGuard::new(10, 3600)
```

### Via environment variables

```env
RUNIQUE_LOGIN_MAX_ATTEMPTS=5
RUNIQUE_LOGIN_LOCKOUT_SECS=300
```

```rust
let guard = Arc::new(LoginGuard::from_env());
```

---

## API

### `record_failure(username)`

Increments the failure counter for this username.
Call after each failed authentication attempt.

### `record_success(username)`

Resets the counter.
Call after a successful login.

### `is_locked(username) -> bool`

Returns `true` if the failure count exceeds `max_attempts` and the lockout duration has not elapsed.

### `attempts(username) -> u32`

Current failure count for this username.

### `remaining_lockout_secs(username) -> Option<u64>`

Seconds remaining until unlock. `None` if the account is not locked.

---

## Combining with IP Rate Limiting

Both mechanisms are complementary:

| | `RateLimiter` | `LoginGuard` |
|---|---|---|
| Key | IP address | Username |
| Target | All routes | Login only |
| False positives | Possible (NAT) | None |
| Goal | Reduce volume | Protect accounts |

```rust
// Maximum protection: both at the same time
let ip_limiter = Arc::new(RateLimiter::new(10, 60));
let login_guard = Arc::new(LoginGuard::new(5, 300));

Router::new()
    .route("/login", post(login_post))
    .layer(axum::middleware::from_fn_with_state(
        ip_limiter,
        rate_limit_middleware,
    ))
```

---

← [**Protection Middlewares**](https://github.com/seb-alliot/runique/blob/main/docs/en/auth/middleware/middleware.md) | [**Complete Example**](https://github.com/seb-alliot/runique/blob/main/docs/en/auth/exemple/exemple.md) →
