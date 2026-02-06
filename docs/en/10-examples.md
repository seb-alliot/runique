Voici la traduction complète en anglais de ton chapitre **Practical Examples** :

# 📚 Practical Examples

## 1️⃣ Minimal Application

### Structure

```
my_app/
├── Cargo.toml
├── .env
├── src/
│   ├── main.rs
│   ├── url.rs
│   └── views.rs
├── templates/
│   └── index.html
└── static/
    └── css/
        └── main.css
```

### main.rs

```rust
#[macro_use]
extern crate runique;

mod url;
mod views;

use runique::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = RuniqueConfig::from_env();

    let db_config = DatabaseConfig::from_env()?.build();
    let db = db_config.connect().await?;

    RuniqueApp::builder(config)
        .routes(url::routes())
        .with_database(db)
        .with_static_files()
        .build()
        .await?
        .run()
        .await?;

    Ok(())
}
```

### url.rs

```rust
use crate::views;
use runique::prelude::*;
use runique::{urlpatterns, view};

pub fn routes() -> Router {
    urlpatterns! {
        "/" => view!{ GET => views::index }, name = "index",
        "/about" => view!{ GET => views::about }, name = "about",
    }
}
```

### views.rs

```rust
use runique::prelude::*;

pub async fn index(mut request: Request) -> AppResult<Response> {
    context_update!(request => {
        "title" => "Home",
        "message" => "Welcome to my Runique app!",
    });
    request.render("index.html")
}

pub async fn about(mut request: Request) -> AppResult<Response> {
    context_update!(request => {
        "title" => "About",
    });
    request.render("about.html")
}
```

### templates/index.html

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>{{ title }}</title>
    <link rel="stylesheet" href='{% static "css/main.css" %}'>
</head>
<body>
    {% messages %}
    <h1>{{ title }}</h1>
    <p>{{ message }}</p>
    <a href='{% link "about" %}'>About</a>
</body>
</html>
```

---

## 2️⃣ CRUD with Forms

### Registration Form

```rust
// src/forms.rs
use runique::prelude::*;

#[runique_form]
pub struct RegisterForm {
    pub form: Forms,
}

impl FormTrait for RegisterForm {
    fn new() -> Self {
        let mut form = Forms::new();
        form.add_field("username", FieldBuilder::text()
            .label("Username")
            .required()
            .min_length(3)
            .max_length(50)
            .build());
        form.add_field("email", FieldBuilder::email()
            .label("Email")
            .required()
            .build());
        form.add_field("password", FieldBuilder::password()
            .label("Password")
            .required()
            .min_length(8)
            .build());
        Self { form }
    }

    fn get_form(&self) -> &Forms { &self.form }
    fn get_form_mut(&mut self) -> &mut Forms { &mut self.form }
    fn get_name(&self) -> &str { "register_form" }
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

### Registration Handler

```rust
// src/views.rs
pub async fn register(
    mut request: Request,
    Prisme(mut form): Prisme<RegisterForm>,
) -> AppResult<Response> {
    let template = "registration_form.html";

    if request.is_get() {
        context_update!(request => {
            "title" => "Register",
            "registration_form" => &form,
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
            "registration_form" => &form,
            "messages" => flash_now!(error => "Please correct the errors"),
        });
        return request.render(template);
    }

    request.render(template)
}
```

### Registration Template

```html
{% extends "base.html" %}

{% block title %}{{ title }}{% endblock %}

{% block content %}
    <h1>{{ title }}</h1>
    {% messages %}

    <form method="post" action='{% link "register" %}'>
        {% form.registration_form %}
        <button type="submit" class="btn btn-primary">Register</button>
    </form>
{% endblock %}
```

---

## 3️⃣ Search and Display an Entity

### Username Form

```rust
#[runique_form]
pub struct UsernameForm {
    pub form: Forms,
}

impl FormTrait for UsernameForm {
    fn new() -> Self {
        let mut form = Forms::new();
        form.add_field("username", FieldBuilder::text()
            .label("Username")
            .required()
            .placeholder("Search a user")
            .build());
        Self { form }
    }

    fn get_form(&self) -> &Forms { &self.form }
    fn get_form_mut(&mut self) -> &mut Forms { &mut self.form }
    fn get_name(&self) -> &str { "username_form" }
}
```

### Search Handler

```rust
pub async fn view_user(
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

        let user_opt = UserEntity::objects
            .filter(users::Column::Username.eq(&username))
            .first(&db)
            .await?;

        match user_opt {
            Some(user) => {
                context_update!(request => {
                    "title" => "User view",
                    "username" => &user.username,
                    "email" => &user.email,
                    "found_user" => &user,  // ⚠️ Do NOT name "user" → collision with the form
                    "user" => &form,         // Form must keep the name "user" for {% form.user %}
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

## 4️⃣ File Upload

### Image Form

```rust
#[runique_form]
pub struct ImageForm {
    pub form: Forms,
}

impl FormTrait for ImageForm {
    fn new() -> Self {
        let mut form = Forms::new();
        form.add_field("image", FieldBuilder::image()
            .label("Image")
            .required()
            .max_size_mb(5)
            .max_files(1)
            .max_dimensions(1920, 1080)
            .allowed_extensions(vec!["jpg", "png", "webp", "avif"])
            .build());
        Self { form }
    }

    fn get_form(&self) -> &Forms { &self.form }
    fn get_form_mut(&mut self) -> &mut Forms { &mut self.form }
    fn get_name(&self) -> &str { "image_form" }
}
```

### Upload Handler

```rust
pub async fn upload_image(
    mut request: Request,
    Prisme(mut form): Prisme<ImageForm>,
) -> AppResult<Response> {
    let template = "forms/upload_image.html";

    if request.is_get() {
        context_update!(request => {
            "title" => "Upload a file",
            "image_form" => &form,
        });
        return request.render(template);
    }

    if request.is_post() {
        if form.is_valid().await {
            success!(request.notices => "File uploaded successfully!");
            return Ok(Redirect::to("/").into_response());
        }

        context_update!(request => {
            "title" => "Error",
            "image_form" => &form,
            "messages" => flash_now!(error => "Please correct the errors"),
        });
        return request.render(template);
    }

    request.render(template)
}
```

### Upload Template

```html
{% extends "base.html" %}

{% block content %}
    <h1>{{ title }}</h1>
    {% messages %}

    <form method="post" enctype="multipart/form-data">
        {% form.image_form %}
        <button type="submit">Upload</button>
    </form>
{% endblock %}
```

---

## 5️⃣ Page Demonstrating All Message Types

```rust
pub async fn demo_messages(mut request: Request) -> AppResult<Response> {
    // Flash messages (displayed after redirect)
    success!(request.notices => "This is a success message.");
    info!(request.notices => "This is an info message.");
    warning!(request.notices => "This is a warning message.");
    error!(request.notices => "This is an error message.");

    context_update!(request => {
        "title" => "Message Demo",
    });
    request.render("demo.html")
}
```

```html
{% extends "base.html" %}

{% block content %}
    <h1>{{ title }}</h1>
    {% messages %}
    <p>The messages above come from the flash session.</p>
{% endblock %}
```

---

## 6️⃣ REST API

### API Routes

```rust
pub fn routes() -> Router {
    urlpatterns! {
        "/api/users" => view!{api_create_user },
         name = "api_users",
    }
}
```

### JSON API Handler

```rust
use axum::Json;
use serde_json::json;

pub async fn api_list_users(request: Request) -> AppResult<Response> {

    let users = users::Entity::find()
        .all(&*&request.engine.db)
        .await?;

    Ok(Json(json!({
        "status": "success",
        "count": users.len(),
        "data": users
    })).into_response())
}
```

---

## 7️⃣ Full Base Template

```html
<!-- templates/base.html -->
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{% block title %}My App{% endblock %}</title>

    <!-- App CSS -->
    <link rel="stylesheet" href='{% static "css/main.css" %}'>
    {% block extra_css %}{% endblock %}
</head>
<body>
    <header>
        <nav>
            <a href='{% link "index" %}'>🏠 Home</a>
            <a href='{% link "about" %}'>ℹ️ About</a>
            <a href='{% link "register" %}'>📝 Register</a>
            <a href='{% link "blog" %}'>📰 Blog</a>
        </nav>
    </header>

    <!-- Automatic flash messages -->
    {% messages %}

    <main>
        {% block content %}{% endblock %}
    </main>

    <footer>
        <p>&copy; 2026 — Powered by Runique 🦀</p>
    </footer>

    <!-- Scripts with CSP nonce -->
    {% block extra_js %}{% endblock %}
</body>
</html>
```

---

## Pattern Summary

| Pattern                                    | When to Use                        |
| ------------------------------------------ | ---------------------------------- |
| `request.render("template.html")`          | Standard HTML render               |
| `Redirect::to("/").into_response()`        | After a successful POST            |
| `context_update!(request => {...})`        | Inject variables into the template |
| `success!(request.notices => "...")`       | Flash message before redirect      |
| `flash_now!(error => "...")`               | Immediate message (no redirect)    |
| `form.is_valid().await`                    | Validate a Prisme form             |
| `form.save(&db).await`                     | Save to database                   |
| `form.get_form_mut().database_error(&err)` | Display DB error in the form       |

---

## Further Reading

* [Installation](https://github.com/seb-alliot/runique/blob/main/docs/en/01-installation.md)
* [Architecture](https://github.com/seb-alliot/runique/blob/main/docs/en/02-architecture.md)
* [Configuration](https://github.com/seb-alliot/runique/blob/main/docs/en/03-configuration.md)
* [Routing](https://github.com/seb-alliot/runique/blob/main/docs/en/04-routing.md)
* [Forms](https://github.com/seb-alliot/runique/blob/main/docs/en/05-forms.md)
* [Templates](https://github.com/seb-alliot/runique/blob/main/docs/en/06-templates.md)
* [ORM](https://github.com/seb-alliot/runique/blob/main/docs/en/07-orm.md)
* [Middleware](https://github.com/seb-alliot/runique/blob/main/docs/en/08-middleware.md)
* [Flash Messages](https://github.com/seb-alliot/runique/blob/main/docs/en/09-flash-messages.md)

← [**Flash Messages**](https://github.com/seb-alliot/runique/blob/main/docs/en/08-middleware.md) | [**Back to README**](https://github.com/seb-alliot/runique/blob/main/README.md) →
