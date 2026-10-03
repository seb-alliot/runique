# Protection Middlewares & CurrentUser

## Route protection — recommended pattern

Two approaches coexist. The manual pattern below is written directly in the handler — more explicit, full control over the redirect URL. To protect an entire route without touching it, `login_required` (via the `RouterExt` trait, at the `urlpatterns!{}` level) is still available — see [Login Required](/docs/en/middleware/login-required).

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

## `request.user` — the signed-in user

No middleware to add: Runique injects a `CurrentUser` into every request of a signed-in user,
available as `request.user`.

The session only keeps **who** signed in. The account (active, staff, superuser) is read from
the database on every request — one lookup by primary key. A deleted or deactivated account
therefore has its session closed on its next request: `request.user` is `None` and
`is_authenticated` returns `false`, whatever made the change (admin, CLI, SQL, another instance).

```rust
use runique::prelude::*;

async fn profile(request: Request) -> impl IntoResponse {
    if let Some(user) = &request.user {
        println!("Logged in as: {}", user.username);
    }
}
```

## CurrentUser

```rust
pub struct CurrentUser {
    pub id: Pk,      // i32 by default, i64 with "big-pk", Uuid with "pk-uuid"
    pub username: String,
    pub is_staff: bool,
    pub is_superuser: bool,
    pub groupes: Vec<Groupe>,
}
```

Groups and their rights are **not** loaded by default: most pages never look at them. A view
that checks a right loads them first — they are read from the database, and `current_user` is
updated in the template context:

```rust
async fn articles(mut request: Request) -> AppResult<Response> {
    request.load_user_rights().await;
    let can_edit = request
        .user
        .as_ref()
        .is_some_and(|u| u.permission_for("articles").is_some_and(|p| p.can_update));
    // ...
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
