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
    let form: LoginForm = request.form();
    let validated = match ValidationForm::try_new(form, &request).await {
        Ok(validated) => validated,
        Err(form) => {
            context_update!(request => { "login_form" => &form });
            return request.render("login.html");
        }
    };

    let db = request.engine.db.clone();
    let username = validated.cleaned_string("username").unwrap_or_default();
    let password = validated.cleaned_string("password").unwrap_or_default();

    // 1. Check the credentials: the password is verified even for an unknown name
    //    (no leak through response time), account active and activated.
    // 2. Open the session — session id and CSRF token renewed.
    if let Some(user) = authenticate_user(&db, username.trim(), &password).await
        && login(&request.session, &user, None, false).await.is_ok()
    {
        return Ok(Redirect::to("/dashboard").into_response());
    }

    // Invalid credentials (generic message — don't distinguish unknown user / wrong password)
    let form = validated.into_form();
    context_update!(request => {
        "login_form" => &form,
        "messages" => flash_now!(error => "Invalid credentials"),
    });
    request.render("login.html")
}

pub async fn logout_view(mut request: Request) -> AppResult<Response> {
    logout(&request.session, None).await.ok();
    Ok(Redirect::to("/login").into_response())
}
```

---

## Authentication for the AdminPanel

Nothing to configure: the admin signs in accounts of `eihwaz_users` that are active and staff
or superuser. To add fields to the user model, use `extend!{ table: "eihwaz_users", ... }`.

To connect authentication to the admin panel, see also [11-Admin.md](/docs/en/admin).

---

## See also

| Section | Description |
| --- | --- |
| [User model](/docs/en/auth/model) | Built-in model, `RuniqueUser` trait |
| [Session helpers](/docs/en/auth/session) | `login`, `logout` |
| [Middlewares & CurrentUser](/docs/en/auth/middleware) | Route protection |

## Back to summary

- [Authentication](/docs/en/auth)
