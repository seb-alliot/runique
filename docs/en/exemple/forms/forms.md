# CRUD with forms

## Registration form

### Manual form (without model)

```rust
// src/forms.rs
use runique::prelude::*;

pub struct RegisterForm {
    pub form: Forms,
}

#[async_trait]
impl RuniqueForm for RegisterForm {
    impl_form_access!();

    fn register_fields(form: &mut Forms) {
        form.field(
            &TextField::text("username")
                .label("Username")
                .required()
                .min_length(3, "Minimum 3 characters")
                .max_length(50, "Maximum 50 characters")
        );
        form.field(
            &TextField::email("email")
                .label("Email")
                .required()
        );
        form.field(
            &TextField::password("password")
                .label("Password")
                .required()
                .min_length(8, "Minimum 8 characters")
        );
    }

    // Business validation — called automatically by is_valid()
    async fn clean(&mut self) -> Result<(), StrMap> {
        let mut errors = StrMap::new();
        if !self.get_string("email").contains('@') {
            errors.insert("email".to_string(), "Invalid email".to_string());
        }
        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}
```

### Model-based form

`#[form(...)]` generates the struct and `impl ModelForm`.
The developer writes `impl RuniqueForm` with `impl_form_access!(model)`:

```rust
use runique::prelude::*;

#[form(schema = users_schema, fields = [username, email, password])]
pub struct RegisterForm;

#[async_trait]
impl RuniqueForm for RegisterForm {
    impl_form_access!(model);

    async fn clean(&mut self) -> Result<(), StrMap> {
        let mut errors = StrMap::new();
        if self.get_string("username").len() < 3 {
            errors.insert("username".to_string(), "Minimum 3 characters".to_string());
        }
        if !self.get_string("email").contains('@') {
            errors.insert("email".to_string(), "Invalid email".to_string());
        }
        if self.get_string("password").len() < 10 {
            errors.insert("password".to_string(), "Minimum 10 characters".to_string());
        }
        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}
```

> `#[async_trait]` is required only when overriding `clean` or `clean_field`.
> Without async override, `impl RuniqueForm { impl_form_access!(model); }` is enough.

---

## Registration handler

```rust
pub async fn signup(
    mut request: Request,
    Prisme(mut form): Prisme<RegisterForm>,
) -> AppResult<Response> {
    let template = "signup_form.html";

    if request.is_get() {
        context_update!(request => {
            "title" => "Sign Up",
            "signup_form" => &form,
        });
        return request.render(template);
    }

    if request.is_post() {
        if form.is_valid().await {
            let user = form.save(&request.engine.db).await.map_err(|err| {
                form.get_form_mut().database_error(&err);
                AppError::from(err)
            })?;

            success!(request.notices => format!("Welcome {}!", user.username));
            return Ok(Redirect::to("/").into_response());
        }

        context_update!(request => {
            "title" => "Validation Error",
            "signup_form" => &form,
            "messages" => flash_now!(error => "Please fix the errors"),
        });
        return request.render(template);
    }

    request.render(template)
}
```

---

## Registration template

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

---

## Search and display an entity

### Search form

```rust
pub struct UsernameForm {
    pub form: Forms,
}

impl RuniqueForm for UsernameForm {
    fn register_fields(form: &mut Forms) {
        form.field(
            &TextField::text("username")
                .label("Username")
                .required()
                .placeholder("Search a user")
        );
    }
    impl_form_access!();
}
```

### Search handler

```rust
pub async fn info_user(
    mut request: Request,
    Prisme(mut form): Prisme<UsernameForm>,
) -> AppResult<Response> {
    let template = "profile/view_user.html";

    if request.is_get() && form.is_valid().await {
        let username = form.get_form().get_value("username").unwrap_or_default();
        let db = request.engine.db.clone();

        let user_opt = UserEntity::find()
            .filter(user::Column::Username.eq(&username))
            .one(&*db)
            .await
            .unwrap_or(None);

        match user_opt {
            Some(user) => {
                context_update!(request => {
                    "title" => "User view",
                    "found_user" => &user,  // ⚠️ DO NOT name it "user" → collision with the form
                    "user" => &form,
                    "messages" => flash_now!(success => "User found!"),
                });
            }
            None => {
                context_update!(request => {
                    "title" => "User view",
                    "user" => &form,
                    "messages" => flash_now!(warning => "User not found"),
                });
            }
        }

        return request.render(template);
    }

    context_update!(request => { "title" => "Search a user", "user" => &form });
    request.render(template)
}
```

---

## See also

| Section | Description |
| --- | --- |
| [Minimal application](/docs/en/exemple/minimal) | Simple starting point |
| [Upload](/docs/en/exemple/upload) | File upload |

## Back to summary

- [Examples](/docs/en/exemple)
