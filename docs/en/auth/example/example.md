# Full Example & AdminPanel

## Full Example — Login / Logout

```rust
use runique::prelude::*;

// LoginForm — declared separately, .no_hash() is required on the password field
#[derive(Serialize, Debug, Clone)]
#[serde(transparent)]
pub struct LoginForm {
    pub form: Forms,
}

impl RuniqueForm for LoginForm {
    fn register_fields(form: &mut Forms) {
        form.field(&TextField::text("username").label("Username").required());
        form.field(&TextField::password("password").label("Password").no_hash().required());
    }
    impl_form_access!();
}

pub async fn login_post(mut request: Request) -> AppResult<Response> {
    let mut form: LoginForm = request.form();
    if request.is_post() && form.is_valid().await {
        let db = request.engine.db.clone();
        let username = form.cleaned_string("username").unwrap_or_default();
        let password = form.cleaned_string("password").unwrap_or_default();

        // 1. Find the user by username via search!
        let query = search!(users::Entity => Username eq username.trim());
        let user = query.first(&db).await.unwrap_or(None);

        if let Some(user) = user
            && user.is_active
            && verify(&password, &user.password)
        {
            // 2. Open the session — session-fixation-safe cycle_id() included
            auth_login(&request.session, &db, user.id).await.ok();
            return Ok(Redirect::to("/dashboard").into_response());
        }

        // Invalid credentials (generic message — don't distinguish unknown user / wrong password)
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
    logout(&request.session, None).await.ok();
    Ok(Redirect::to("/login").into_response())
}
```

---

## Authentication for the AdminPanel

### With the built-in User (zero config)

```rust
.with_admin(|a| a.auth(RuniqueAdminAuth::new()))
```

### With a custom model

```rust
use runique::prelude::*;

// 1. Implement UserEntity on your entity
impl UserEntity for users::Entity {
    type Model = users::Model;

    async fn find_by_id(db: &DatabaseConnection, id: Pk) -> Option<Self::Model> {
        users::Entity::find_by_id(id).one(db).await.ok().flatten()
    }

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

    async fn update_password(db: &DatabaseConnection, email: &str, new_hash: &str) -> Result<(), sea_orm::DbErr> {
        // update password hash implementation
        todo!()
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
| [Session helpers](/docs/en/auth/session) | `login`, `auth_login`, `logout` |
| [Middlewares & CurrentUser](/docs/en/auth/middleware) | Route protection |

## Back to summary

- [Authentication](/docs/en/auth)
