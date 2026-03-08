# Protection Middlewares & CurrentUser

## `login_required` — protect a route

Redirects to `REDIRECT_ANONYMOUS` if the user is not logged in.

```rust
use runique::middleware::auth::login_required;

let protected = Router::new()
    .route("/dashboard", get(dashboard))
    .layer(axum::middleware::from_fn(login_required));
```

---

## `redirect_if_authenticated` — login/register pages

Redirects to `USER_CONNECTED_URL` if the user is already logged in. Useful to prevent authenticated users from reaching `/login`.

```rust
use runique::middleware::auth::redirect_if_authenticated;

let public = Router::new()
    .route("/login", get(login_page).post(login_post))
    .layer(axum::middleware::from_fn(redirect_if_authenticated));
```

---

## `load_user_middleware` — load user context

Injects a `CurrentUser` into request extensions, making user information available in all handlers down the chain.

```rust
use runique::middleware::auth::load_user_middleware;

let app = Router::new()
    .route("/profile", get(profile))
    .layer(axum::middleware::from_fn(load_user_middleware));
```

Access in a handler:

```rust
use runique::middleware::auth::CurrentUser;
use runique::context::RequestExtensions;

async fn profile(req: RuniqueRequest) -> impl IntoResponse {
    if let Some(user) = req.extensions().current_user() {
        println!("Logged in as: {}", user.username);
    }
}
```

---

## CurrentUser

Struct injected by `load_user_middleware` into request extensions.

```rust
pub struct CurrentUser {
    pub id: i32,
    pub username: String,
    pub is_staff: bool,
    pub is_superuser: bool,
    pub roles: Vec<String>,
}
```

### Available Methods

```rust
// Check a specific role
user.has_role("editor")           // → bool

// Check for at least one role from a list
user.has_any_role(&["editor", "moderator"])  // → bool

// Admin panel access (is_staff || is_superuser)
user.can_access_admin()           // → bool

// Check admin permission (is_superuser bypasses everything)
user.can_admin(&["editor"])       // → bool
```

---

## See also

| Section | Description |
| --- | --- |
| [User model](https://github.com/seb-alliot/runique/blob/main/docs/en/auth/model/model.md) | Built-in model, `RuniqueUser` trait |
| [Session helpers](https://github.com/seb-alliot/runique/blob/main/docs/en/auth/session/session.md) | `login`, `logout`, checks |

## Back to summary

- [Authentication](https://github.com/seb-alliot/runique/blob/main/docs/en/auth/13-authentification.md)
