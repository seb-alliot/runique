# Forms: sign up, sign in, search

The examples below follow the code of the project `runique new` creates (compiled on every CI run). See [Forms](/docs/en/formulaire) for the full API.

---

## Manual form: sign in

A manual form declares its fields by hand. Use one when it maps to no table (sign in, search, contact).

```rust
// src/forms/login.rs
use runique::prelude::*;
use serde::Serialize;

#[derive(Serialize, Debug, Clone)]
#[serde(transparent)]
pub struct LoginForm {
    pub form: Forms,
}

impl FormAccess for LoginForm {
    fn register_fields(form: &mut Forms) {
        form.field(&TextField::text("username").label("Username").required());
        form.field(&TextField::password("password").label("Password").required());
    }
    impl_form_access!();
}

impl RuniqueForm for LoginForm {}
```

`Serialize` (transparent) is needed to hand the form to the template. Don't put such forms in `src/entities/`: `runique makemigrations` looks for models there.

### Sign-in handler

```rust
pub async fn sign_in(mut request: Request) -> AppResult<Response> {
    let form: LoginForm = request.form();
    if request.user.is_some() {
        return Ok(Redirect::to("/").into_response());
    }

    let template = "login.html";
    let validated = match ValidationForm::try_form(form, &request, "login").await {
        Ok(validated) => validated,
        Err(form) => {
            context_update!(request => {
                "title" => "Sign in",
                "login_form" => &form,
            });
            return request.render(template);
        }
    };

    let username = validated.cleaned::<String>("username").unwrap_or_default();
    let password = validated.cleaned::<String>("password").unwrap_or_default();
    // Checks the password even for an unknown username (same timing either
    // way) and refuses an inactive or never activated account.
    if let Some(user) = authenticate_user(&request.engine.db, &username, &password).await
        && login(&request.session, &user, None, false).await.is_ok()
    {
        success!(request.notices => format!("Welcome back {}!", user.username));
        return Ok(Redirect::to("/").into_response());
    }

    context_update!(request => {
        "title" => "Sign in",
        "login_form" => &*validated,
        "messages" => flash_now!(error => "Invalid username or password."),
    });
    request.render(template)
}
```

`ValidationForm::try_form(form, &request, "login")` names the form and validates it only when the request sends that name back: a GET, or another form of the same page being sent, comes back as `Err(form)` with no field error. The CSRF token is checked when the form is read.

---

## Model-based form: sign up

`#[form(schema = …, fields = […])]` generates the struct and `FormAccess` from the model; you only write `RuniqueForm`, with its hooks.

```rust
// src/forms/signup.rs
use runique::prelude::*;

#[form(schema = runique_users, fields = [username, email, password])]
pub struct SignupForm;

#[async_trait]
impl RuniqueForm for SignupForm {
    async fn clean(&mut self) -> Result<(), StrMap> {
        let username = self.cleaned::<String>("username").unwrap_or_default();
        let mut errors = StrMap::new();
        if username.len() < 5 {
            errors.insert("username".to_string(), "At least 5 characters.".to_string());
        }
        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}
```

`#[async_trait]` is only needed when you override an async hook (`clean`, `clean_field`); with no hook, `impl RuniqueForm for SignupForm {}` is enough.

### Saving an account

For an ordinary model, `validated.save(&db)` saves the row. For an **account** it isn't enough: `password` is a secret column, which the generic save never writes. The generated project therefore writes its own `save`:

```rust
impl SignupForm {
    pub async fn save(&self, db: &ADb) -> Result<runique_users::Model, DbErr> {
        use runique::prelude::runique_users::ActiveModel;
        let now = chrono::Utc::now().naive_utc();
        let user = ActiveModel {
            username: Set(self.cleaned::<String>("username").unwrap_or_default()),
            email: Set(self.cleaned::<String>("email").unwrap_or_default()),
            // After is_valid(), the password field already holds the hash.
            password: Set(self.cleaned::<String>("password").unwrap_or_default()),
            is_active: Set(false),
            is_superuser: Set(false),
            is_dirigeant: Set(false),
            is_admin: Set(false),
            created_at: Set(Some(now)),
            updated_at: Set(Some(now)),
            ..Default::default()
        }
        .insert(db)
        .await?;

        // An account is created inactive. Activating it is a business
        // decision: here at once; to confirm the email first, send a link and
        // call `activate_account` when it's opened.
        BuiltinUserEntity::activate_account(db, user.id)
            .await?
            .ok_or_else(|| DbErr::Custom("account already activated".into()))
    }
}
```

### Sign-up handler

```rust
pub async fn sign_up(mut request: Request) -> AppResult<Response> {
    let form: SignupForm = request.form();
    if request.user.is_some() {
        return Ok(Redirect::to("/").into_response());
    }

    let template = "signup.html";
    let mut form = match ValidationForm::try_form(form, &request, "signup").await {
        Ok(validated) => validated.into_form(),
        Err(form) => {
            context_update!(request => {
                "title" => "Sign up",
                "signup_form" => &form,
            });
            return request.render(template);
        }
    };

    match form.save(&request.engine.db).await {
        Ok(user) => {
            if login(&request.session, &user, None, false).await.is_ok() {
                success!(request.notices => format!("Welcome {}!", user.username));
            }
            Ok(Redirect::to("/").into_response())
        }
        Err(err) => {
            // A username or email already taken shows on its field.
            form.get_form_mut().database_error(&err);
            context_update!(request => {
                "title" => "Sign up",
                "signup_form" => &form,
                "messages" => flash_now!(error => "Signing up failed."),
            });
            request.render(template)
        }
    }
}
```

### Template

```html
{% extends "base.html" %}

{% block content %}
  <h1>{{ title }}</h1>
  {% messages %}
  <form method="post" action='{% link "signup" %}'>
    {% form.signup_form %}
    <button type="submit">Sign up</button>
  </form>
{% endblock %}
```

`{% form.signup_form %}` writes the fields, their errors, the CSRF token and the form's name.

---

## Sign out

A POST, never a GET. The view reads no form, so it checks the CSRF token itself:

```rust
pub async fn sign_out(request: Request) -> AppResult<Response> {
    if !request.prisme.csrf_valid {
        return Ok(StatusCode::FORBIDDEN.into_response());
    }
    logout(&request.session, None).await?;
    Ok(Redirect::to("/").into_response())
}
```

```rust
// url.rs
"/logout" => post(sign_out), name = "logout",
```

```html
<form method="post" action='{% link "logout" %}'>
  {% csrf %}
  <button type="submit">Sign out</button>
</form>
```

---

## Search form (GET)

By default a form never validates on a GET: GETs carry no CSRF token, so a forged link could otherwise trigger its effects. A search form, which only reads, allows it explicitly:

```rust
#[derive(Serialize, Debug, Clone)]
#[serde(transparent)]
pub struct SearchForm {
    pub form: Forms,
}

impl FormAccess for SearchForm {
    fn register_fields(form: &mut Forms) {
        form.field(&TextField::text("username").label("User").required());
    }
    impl_form_access!();
}

impl RuniqueForm for SearchForm {
    fn allow_get(&self, _request: &Request) -> bool {
        self.is_submitted()
    }
}
```

```rust
pub async fn search_user(mut request: Request) -> AppResult<Response> {
    let form: SearchForm = request.form();
    let template = "search.html";

    let form = match ValidationForm::try_form(form, &request, "search").await {
        Ok(validated) => {
            let username = validated.cleaned::<String>("username").unwrap_or_default();
            let found = runique_users::Entity::find()
                .filter(runique_users::Column::Username.eq(&username))
                .one(&*request.engine.db)
                .await?;
            context_update!(request => { "found_user" => &found.map(|u| u.username) });
            validated.into_form()
        }
        Err(form) => form,
    };

    context_update!(request => { "title" => "Search", "search_form" => &form });
    request.render(template)
}
```

```html
<form method="get">
  {% form.search_form %}
  <button type="submit">Search</button>
</form>
{% if found_user %}<p>Found: {{ found_user }}</p>{% endif %}
```

Never hand a whole account model to the template: it holds the password hash. Keep only the columns to show.

---

## See also

| Section | Description |
| --- | --- |
| [Minimal application](/docs/en/exemple/minimal) | Simple starting point |
| [Upload](/docs/en/exemple/upload) | File upload |

## Back to summary

- [Examples](/docs/en/exemple)
