# Protection Middlewares & CurrentUser

## Route protection — recommended pattern

`login_required` and `redirect_if_authenticated` have been removed. Protection is written directly in the handler, which is more explicit and gives the dev full control over the redirect URL.

```rust
use runique::middleware::auth::is_authenticated;

// Protect a route
async fn dashboard(mut request: Request) -> AppResult<Response> {
    if !is_authenticated(&request.session).await {
        return Ok(Redirect::to("/login").into_response());
    }
    // ...
}

// Redirect if already authenticated (login/register pages)
async fn login_page(mut request: Request) -> AppResult<Response> {
    if is_authenticated(&request.session).await {
        return Ok(Redirect::to("/").into_response());
    }
    // ...
}
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
    pub id: Pk,      // i32 by default, i64 with the "big-pk" feature
    pub username: String,
    pub is_staff: bool,
    pub is_superuser: bool,
    pub droits: Vec<Droit>,
    pub groupes: Vec<Groupe>,
}
```

### Available Methods

```rust
// Effective rights (direct + inherited from groups, deduplicated)
user.droits_effectifs()           // → Vec<Droit>

// Check a specific right
user.has_droit("editor")          // → bool

// Check for at least one right from a list
user.has_any_droit(&["editor", "moderator"])  // → bool

// Admin panel access (is_staff || is_superuser)
user.can_access_admin()           // → bool

// Check admin permission (is_superuser bypasses everything)
user.can_admin(&["editor"])       // → bool
```

---

## See also

| Section | Description |
| --- | --- |
| [User model](/docs/en/auth/model) | Built-in model, `RuniqueUser` trait |
| [Session helpers](/docs/en/auth/session) | `login`, `logout`, checks |

## Back to summary

- [Authentication](/docs/en/auth)
