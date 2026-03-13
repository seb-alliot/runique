# LoginGuard — Brute-force Protection

`LoginGuard` tracks failed login attempts **per username**, independently of the IP address.

Unlike IP rate limiting, there are no false positives from NAT or shared proxies: each account is tracked individually.

---

## Usage

```rust
use runique::prelude::*;

static GUARD: LazyLock<LoginGuard> = LazyLock::new(|| LoginGuard::new(5, 300));

pub async fn login(
    session: Session,
    State(db): State<DatabaseConnection>,
    Prisme(form): Prisme<LoginForm>,
) -> impl IntoResponse {
    let username = form.username();

    if GUARD.is_locked(&username) {
        let remaining = GUARD.remaining_lockout_secs(&username).unwrap_or(0);
        return (StatusCode::TOO_MANY_REQUESTS, format!("Try again in {remaining}s")).into_response();
    }

    match authenticate(&username, &form.password(), &db).await {
        Some(user) => {
            GUARD.record_success(&username);
            login(&session, user.id, &user.username).await.unwrap();
            Redirect::to("/dashboard").into_response()
        }
        None => {
            GUARD.record_failure(&username);
            (StatusCode::UNAUTHORIZED, "Invalid credentials").into_response()
        }
    }
}
```

`LoginGuard::new(max_attempts, lockout_secs)` — the dev declares their limits wherever and however they want.

---

## Configuration

### In code

```rust
LoginGuard::new(5, 300)    // 5 failures → 5 minute lockout
LoginGuard::new(3, 900)    // 3 failures → 15 minute lockout
LoginGuard::new(10, 3600)  // 10 failures → 1 hour lockout
```

### Via environment variables

```env
RUNIQUE_LOGIN_MAX_ATTEMPTS=5
RUNIQUE_LOGIN_LOCKOUT_SECS=300
```

```rust
static GUARD: LazyLock<LoginGuard> = LazyLock::new(LoginGuard::from_env);
```

---

## API

### `record_failure(username)`

Increments the failure counter for this username. Call after each failed authentication attempt.

### `record_success(username)`

Resets the counter. Call after a successful login.

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
static IP_LIMITER: LazyLock<RateLimiter> = LazyLock::new(|| RateLimiter::new(10, 60));
static GUARD:      LazyLock<LoginGuard>  = LazyLock::new(|| LoginGuard::new(5, 300));

pub async fn login(/* ... */) -> impl IntoResponse {
    if !IP_LIMITER.is_allowed(&ip) { /* 429 */ }
    if GUARD.is_locked(&username)  { /* 429 */ }
    // ...
}
```

---

← [**Protection Middlewares**](https://github.com/seb-alliot/runique/blob/main/docs/en/auth/middleware/middleware.md) | [**Complete Example**](https://github.com/seb-alliot/runique/blob/main/docs/en/auth/exemple/exemple.md) →
