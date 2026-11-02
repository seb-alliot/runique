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

The session only keeps **who** signed in. The account (active, rank, groups) is read from
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
    pub id: Pk,                    // i32 by default, i64 with "big-pk", Uuid with "pk-uuid"
    pub username: String,
    pub rang: Rang,                // Lambda, Admin, Dirigeant, Superuser
    pub groupes: Vec<Groupe>,      // loaded by load_user_rights()
    pub droits: Vec<Permission>,   // an admin's own rights, loaded by load_user_rights()
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
        .is_some_and(|u| u.rights_on("articles").crud.can_update);
    // ...
}
```

### Available Methods

```rust
// Rights on a resource, per rank: everything for a dirigeant or a superuser,
// an admin's own grid, the union of the groups for everyone else
user.rights_on("articles")              // → Rights { crud, columns }
user.rights_on("articles").crud.can_update

// May the user read a resource?
user.can_access_resource("articles")    // → bool

// Admin panel access: rank admin or above, or member of a staff group
user.can_access_admin()                 // → bool
user.is_staff()                         // member of a group that opens the admin
```

These methods read the groups and rights: call `request.load_user_rights().await` first.

---

## See also

| Section | Description |
| --- | --- |
| [User model](/docs/en/auth/model) | Built-in model, `RuniqueUser` trait |
| [Session helpers](/docs/en/auth/session) | `login`, `logout`, checks |

## Back to summary

- [Authentication](/docs/en/auth)
