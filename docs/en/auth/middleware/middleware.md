# Protection Middlewares & CurrentUser

## Route protection — recommended pattern

`login_required` and `redirect_if_authenticated` have been removed. Protection is written directly in the handler, which is more explicit and gives the dev full control over the redirect URL.

```rust
use runique::prelude::*;

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
use runique::prelude::*;

let app = Router::new()
    .route("/profile", get(profile))
    .layer(axum::middleware::from_fn(load_user_middleware));
```

Access in a handler:

```rust
use runique::prelude::*;

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
    pub groupes: Vec<Groupe>,
}
```

### Available Methods

```rust
// Effective permissions (all resources, logical OR across all groups)
user.permissions_effectives()                 // → Vec<Permission>

// Permission for a specific resource
user.permission_for("users")                  // → Option<Permission>

// Read access to a resource (is_superuser bypasses everything)
user.can_access_resource("users")             // → bool

// Admin panel access (is_staff || is_superuser)
user.can_access_admin()                       // → bool
```

---

## See also

| Section | Description |
| --- | --- |
| [User model](/docs/en/auth/model) | Built-in model, `RuniqueUser` trait |
| [Session helpers](/docs/en/auth/session) | `login`, `logout`, checks |

## Back to summary

- [Authentication](/docs/en/auth)
