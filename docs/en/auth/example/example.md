# Full Example & AdminPanel

## Full Example — Login / Logout

```rust
use runique::middleware::auth::{login_staff, logout};
use runique::utils::password::verify;
use runique::prelude::*;

pub async fn login_post(
    mut request: Request,
    Prisme(mut form): Prisme<LoginForm>,
) -> AppResult<Response> {
    if request.is_post() && form.is_valid().await {
        // 1. Find the user by username
        let username = form.get_form().get_string("username");
        let user = users::Entity::objects
            .filter(users::Column::Username.eq(&username))
            .first(&*request.engine.db)
            .await?;

        if let Some(user) = user {
            // 2. Verify the password (plain text vs hash)
            let password = form.get_form().get_string("password");
            if verify(&password, &user.password) {
                // 3. Open the session
                login_staff(
                    &request.session,
                    user.id,
                    &user.username,
                    user.is_staff,
                    user.is_superuser,
                    user.roles(),
                ).await?;
                return Ok(Redirect::to("/dashboard").into_response());
            }
        }

        // Invalid credentials
        context_update!(request => {
            "login_form" => &form,
            "messages" => flash_now!(error => "Invalid credentials"),
        });
    } else {
        context_update!(request => { "login_form" => &form });
    }

    request.render("login.html")
}

pub async fn logout_view(mut request: Request) -> AppResult<Response> {
    logout(&request.session).await.ok();
    Ok(Redirect::to("/login").into_response())
}
```

---

## Authentication for the AdminPanel

### With the built-in User (zero config)

```rust
use runique::middleware::auth::RuniqueAdminAuth;

.with_admin(|a| a.auth(RuniqueAdminAuth::new()))
```

### With a custom model

```rust
use runique::middleware::auth::{DefaultAdminAuth, UserEntity};

// 1. Implement UserEntity on your entity
impl UserEntity for users::Entity {
    type Model = users::Model;

    async fn find_by_username(db: &DatabaseConnection, username: &str) -> Option<Self::Model> {
        users::Entity::find()
            .filter(users::Column::Username.eq(username))
            .one(db)
            .await
            .ok()
            .flatten()
    }

    async fn find_by_email(db: &DatabaseConnection, email: &str) -> Option<Self::Model> {
        users::Entity::find()
            .filter(users::Column::Email.eq(email))
            .one(db)
            .await
            .ok()
            .flatten()
    }
}

// 2. Pass DefaultAdminAuth to the admin config
.with_admin(|a| a.auth(DefaultAdminAuth::<users::Entity>::new()))
```

To connect authentication to the admin panel, see also [11-Admin.md](/docs/en/admin).

---

## See also

| Section | Description |
| --- | --- |
| [User model](/docs/en/auth/model) | Built-in model, `RuniqueUser` trait |
| [Session helpers](/docs/en/auth/session) | `login`, `logout`, checks |
| [Middlewares & CurrentUser](/docs/en/auth/middleware) | Route protection |

## Back to summary

- [Authentication](/docs/en/auth)
