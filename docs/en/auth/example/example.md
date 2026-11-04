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

impl FormAccess for LoginForm {
    fn register_fields(form: &mut Forms) {
        form.field(&TextField::text("username").label("Username").required());
        form.field(&TextField::password("password").label("Password").no_hash().required());
    }
    impl_form_access!();
}

impl RuniqueForm for LoginForm {}

pub async fn login_post(mut request: Request) -> AppResult<Response> {
    let form: LoginForm = request.form("my_form");
    let validated = match ValidationForm::try_form(form, &request).await {
        Ok(validated) => validated,
        Err(form) => {
            context_update!(request => { "login_form" => &form });
            return request.render("login.html");
        }
    };

    let db = request.engine.db.clone();
    let username = validated.cleaned::<String>("username").unwrap_or_default();
    let password = validated.cleaned::<String>("password").unwrap_or_default();

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

// POST-only route: "/logout" => post(views::logout_view), name = "logout"
pub async fn logout_view(request: Request) -> AppResult<Response> {
    // This view reads no form: it checks the CSRF token itself.
    if !request.prisme.csrf_valid {
        return Ok(StatusCode::FORBIDDEN.into_response());
    }
    // An error here means the session couldn't be closed: it is still
    // alive, so the user gets an error page.
    logout(&request.session, None).await?;
    Ok(Redirect::to("/login").into_response())
}
```

The sign-out form, in the template:

```html
<form method="post" action='{% link "logout" %}'>
    {% csrf %}
    <button type="submit">Sign out</button>
</form>
```

A sign-out link (GET) could be triggered by another site (an image, a link): it must go through a POST with its CSRF token.

---

## Authentication for the AdminPanel

Nothing to configure: the admin signs in the active, activated accounts of `eihwaz_users` of
rank admin, dirigeant or superuser, or members of a group that opens the admin. To add fields to the user model, use `extend!{ table: "eihwaz_users", ... }`.

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
