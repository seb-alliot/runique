# Login Required

`login_required` protects a route against unauthenticated access. If the user is not logged in, they are redirected to the specified login URL.

---

## Usage — route level

In `url.rs`, via the `RouterExt` trait:

```rust
use runique::prelude::*;

pub fn routes() -> Router {
    urlpatterns! {
        "/" => view!{ index }, name = "index",
        // ...
    }
    .login_required("/profile", "profile", view!(profile), "/login")
}
```

Multiple protected routes:

```rust
urlpatterns! { ... }
    .login_required("/profile",   "profile",   view!(profile),   "/login")
    .login_required("/dashboard", "dashboard", view!(dashboard), "/login")
```

---

## Signature

```rust
fn login_required(
    self,
    path: impl Into<String>,
    name: impl Into<String>,
    handler: MethodRouter,
    redirect_url: impl Into<String>,
) -> Self;
```

| Parameter      | Description                                          |
| -------------- | ---------------------------------------------------- |
| `path`         | Route path (e.g. `"/profile"`)                       |
| `name`         | Route name for `reverse()`                           |
| `handler`      | View to protect (via `view!{}`)                      |
| `redirect_url` | Redirect URL if not authenticated (e.g. `"/login"`)  |

---

## Behavior

- Checks for a user identifier in the session
- Authenticated → request passes through to the handler normally
- Not authenticated → `302 Found` to `redirect_url`
- Compatible with all HTTP methods (GET, POST, etc.)

---

## Notes

- This protection applies to **any logged-in user**, regardless of role
- For the admin interface, Runique uses its own `admin_required` middleware (checks `is_staff` or `is_superuser`) — `login_required` is not needed on the admin side

---

← [**Rate Limiting**](/docs/en/middleware/rate-limit) | [**Flash Messages**](/docs/en/flash) →
