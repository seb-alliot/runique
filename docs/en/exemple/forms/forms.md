# CRUD with forms

## Registration form

```rust
// src/forms.rs
use runique::prelude::*;

pub struct RegisterForm {
    pub form: Forms,
}

impl RuniqueForm for RegisterForm {
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

    impl_form_access!();
}

impl RegisterForm {
    pub async fn save(&self, db: &DatabaseConnection) -> Result<users::Model, DbErr> {
        use sea_orm::Set;
        let model = users::ActiveModel {
            username: Set(self.form.get_value("username").unwrap_or_default()),
            email: Set(self.form.get_value("email").unwrap_or_default()),
            password: Set(self.form.get_value("password").unwrap_or_default()),
            ..Default::default()
        };
        model.insert(db).await
    }
}
```

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

{% block title %}{{ title }}{% endblock %}

{% block content %}
    <h1>{{ title }}</h1>
    {% messages %}

    <form method="post" action='{% link "signup" %}'>
        {% form.signup_form %}
        <button type="submit" class="btn btn-primary">Sign up</button>
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

    if request.is_get() {
        context_update!(request => {
            "title" => "Search a user",
            "user" => &form,
        });
        return request.render(template);
    }

    if request.is_post() {
        if !form.is_valid().await {
            context_update!(request => {
                "title" => "Search a user",
                "user" => &form,
                "messages" => flash_now!(error => "Validation error"),
            });
            return request.render(template);
        }

        let username = form.get_form().get_value("username").unwrap_or_default();
        let db = request.engine.db.clone();

        let user_opt = users::Entity::objects
            .filter(users::Column::Username.eq(&username))
            .first(&db)
            .await?;

        match user_opt {
            Some(user) => {
                context_update!(request => {
                    "title" => "User view",
                    "username" => &user.username,
                    "email" => &user.email,
                    "found_user" => &user,  // ⚠️ DO NOT name it "user" → collision with the form
                    "user" => &form,         // The form must keep the name "user" for {% form.user %}
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

    request.render(template)
}
```

---

## See also

| Section | Description |
| --- | --- |
| [Minimal application](https://github.com/seb-alliot/runique/blob/main/docs/en/exemple/minimal/minimal.md) | Simple starting point |
| [Upload](https://github.com/seb-alliot/runique/blob/main/docs/en/exemple/upload/upload.md) | File upload |

## Back to summary

- [Examples](https://github.com/seb-alliot/runique/blob/main/docs/en/exemple/10-examples.md)
