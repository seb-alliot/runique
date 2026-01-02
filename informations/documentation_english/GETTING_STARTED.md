# Quick Start Guide - Rusti Framework

Welcome to Rusti! This guide will walk you through step-by-step in creating your first web application with Rusti.

## Prerequisites

- **Rust 1.70+** - [Install Rust](https://www.rust-lang.org/tools/install)
- **Cargo** (installed automatically with Rust)
- Basic Rust knowledge (ownership, borrowing, async/await)

---

## Installation

### 1. Create a New Project

```bash
cargo new my_app
cd my_app
```

### 2. Add Rusti to Dependencies

Edit `Cargo.toml`:

```toml
[package]
name = "my_app"
version = "0.1.0"
edition = "2021"

[dependencies]
rusti = "1.0"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
```

**With PostgreSQL:**

```toml
[dependencies]
rusti = { version = "1.0", features = ["postgres"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
```

---

## Minimal Application

### 1. Source Code (src/main.rs)

```rust
use rusti::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration from .env
    let settings = Settings::from_env();

    // Create and launch application
    RustiApp::new(settings).await?
        .routes(routes())
        .run()
        .await?;

    Ok(())
}

fn routes() -> Router {
    urlpatterns![
        path!("", index),
    ]
}

async fn index() -> &'static str {
    "Welcome to Rusti!"
}
```

### 2. Configuration (.env)

Create a `.env` file at the root:

```env
HOST=127.0.0.1
PORT=8000
SECRET_KEY=change-me-in-production-with-32-chars-minimum
ALLOWED_HOSTS=localhost,127.0.0.1
DEBUG=true
```

### 3. Launch

```bash
cargo run
```

Open [http://localhost:8000](http://localhost:8000) in your browser.

✅ You should see: **"Welcome to Rusti!"**

---

## Basic Routing

### URLs with Parameters

```rust
use rusti::prelude::*;

fn routes() -> Router {
    urlpatterns![
        path!("", index),
        path!("hello/<n>", hello),
        path!("user/<id>", user_detail),
    ]
}

async fn index() -> &'static str {
    "Homepage"
}

async fn hello(Path(name): Path<String>) -> String {
    format!("Hello, {}!", name)
}

async fn user_detail(Path(id): Path<i32>) -> String {
    format!("User details #{}", id)
}
```

**Test:**
- `GET /` → "Homepage"
- `GET /hello/Alice` → "Hello, Alice!"
- `GET /user/42` → "User details #42"

### Route Names (reverse routing)

```rust
use rusti::prelude::*;

fn routes() -> Router {
    urlpatterns![
        path!("", index, "index"),
        path!("posts/", list_posts, "post_list"),
        path!("posts/<id>/", detail_post, "post_detail"),
    ]
}
```

---

## Templates with Tera

### 1. Folder Structure

```
my_app/
├── Cargo.toml
├── .env
├── src/
│   └── main.rs
└── templates/
    ├── base.html
    └── index.html
```

### 2. Base Template (templates/base.html)

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{% block title %}My App{% endblock %}</title>
</head>
<body>
    <header>
        <h1>My Rusti Application</h1>
        <nav>
            <a href="{% link 'index' %}">Home</a>
        </nav>
    </header>

    <main>
        {% block content %}{% endblock %}
    </main>

    <footer>
        <p>© 2026 My App</p>
    </footer>
</body>
</html>
```

### 3. Page Template (templates/index.html)

```html
{% extends "base.html" %}

{% block title %}Home{% endblock %}

{% block content %}
<h2>Welcome {{ username }}!</h2>
<p>You've been connected since {{ date }}.</p>
{% endblock %}
```

### 4. Usage in a Handler

```rust
use rusti::prelude::*;

async fn index(template: Template) -> Response {
    template.render("index.html", context! {
        username: "Alice",
        date: chrono::Utc::now().format("%m/%d/%Y").to_string(),
    })
}
```

---

## Database

### 1. Configuration

Add to `.env`:

```env
# PostgreSQL
DB_ENGINE=postgres
DB_USER=myuser
DB_PASSWORD=mypassword
DB_HOST=localhost
DB_PORT=5432
DB_NAME=mydb

# SQLite (alternative)
# DB_ENGINE=sqlite
# DB_NAME=database.sqlite
```

### 2. Define a Model

Create `src/models.rs`:

```rust
use rusti::prelude::*;
use sea_orm::entity::prelude::*;
use rusti::impl_objects;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub username: String,
    pub email: String,
    pub created_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

// Enable Django-like API
impl_objects!(Entity);
```

### 3. Database Connection

```rust
use rusti::prelude::*;
mod models;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = Settings::from_env();

    // Database connection
    let db_config = DatabaseConfig::from_env()?.build();
    let db = db_config.connect().await?;

    RustiApp::new(settings).await?
        .with_database(db)
        .routes(routes())
        .run()
        .await?;

    Ok(())
}
```

### 4. Usage in a Handler

```rust
use rusti::prelude::*;
use crate::models::{users, Entity as User};

async fn list_users(
    Extension(db): Extension<Arc<DatabaseConnection>>,
    template: Template,
) -> Response {
    // Django-like API
    let users = User::objects
        .filter(users::Column::IsActive.eq(true))
        .order_by_asc(users::Column::Username)
        .all(&*db)
        .await
        .unwrap_or_default();

    template.render("users.html", context! {
        users: users,
    })
}
```

---

## Forms

### 1. Define a Form

Create `src/forms.rs`:

```rust
use rusti::prelude::*;
use rusti::forms::prelude::*;

#[rusti_form]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactForm {
    #[field(max_length = 100, required = true)]
    pub name: CharField,

    #[field(required = true)]
    pub email: EmailField,

    #[field(max_length = 50, required = true)]
    pub subject: CharField,

    #[field(widget = "textarea", required = true)]
    pub message: CharField,
}
```

### 2. Display the Form

```rust
use rusti::prelude::*;
use crate::forms::ContactForm;

async fn contact_view(template: Template) -> Response {
    let form = ContactForm::new();
    template.render("contact.html", context! {
        form: form,
    })
}
```

Template `templates/contact.html`:

```html
{% extends "base.html" %}

{% block content %}
<h2>Contact Us</h2>

<form method="post">
    {% csrf %}
    {{ form }}
    <button type="submit">Send</button>
</form>
{% endblock %}
```

### 3. Process the Form

```rust
use rusti::prelude::*;
use crate::forms::ContactForm;

async fn contact_submit(
    Form(form): Form<ContactForm>,
    template: Template,
    mut message: Message,
) -> Response {
    // Validation
    if !form.is_valid() {
        return template.render("contact.html", context! {
            form: form,
            errors: form.errors(),
        });
    }

    // Processing (send email, etc.)
    success!(message, "Message sent successfully!");

    redirect("/")
}
```

---

## Middleware and Security

### Recommended Configuration

```rust
use rusti::prelude::*;
use rusti::middleware::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = Settings::from_env();

    RustiApp::new(settings).await?
        // Security
        .middleware(CsrfMiddleware::new())
        .middleware(SecurityHeadersMiddleware::new())
        .middleware(AllowedHostsMiddleware)
        .middleware(XssSanitizerMiddleware)

        // Features
        .middleware(FlashMiddleware)
        .middleware(MessageMiddleware)

        // Routes
        .routes(routes())

        // Launch
        .run()
        .await?;

    Ok(())
}
```

### CSRF Protection

Automatic with `CsrfMiddleware`:

```html
<form method="post">
    {% csrf %}
    <!-- Token is verified automatically -->
</form>
```

### Content Security Policy

```rust
use rusti::prelude::*;
use rusti::middleware::CspConfig;

let csp_config = CspConfig {
    default_src: vec!["'self'".to_string()],
    script_src: vec!["'self'".to_string()],
    style_src: vec!["'self'".to_string(), "'unsafe-inline'".to_string()],
    use_nonce: true,
    ..Default::default()
};

RustiApp::new(settings).await?
    .middleware(CspMiddleware::new(csp_config))
    .routes(routes())
    .run()
    .await?;
```

---

## Static Files

### 1. Configuration (.env)

```env
STATIC_URL=/static/
STATIC_ROOT=static/
MEDIA_URL=/media/
MEDIA_ROOT=media/
```

### 2. Folder Structure

```
my_app/
├── static/
│   ├── css/
│   │   └── style.css
│   └── js/
│       └── app.js
└── media/
    └── uploads/
```

### 3. Usage in Templates

```html
<!-- Static files -->
<link rel="stylesheet" href="{% static 'css/style.css' %}">
<script src="{% static 'js/app.js' %}"></script>

<!-- Media files (uploaded) -->
<img src="{% media user.avatar %}" alt="Avatar">
```

---

## Flash Messages

### 1. Activation

```rust
use rusti::prelude::*;

RustiApp::new(settings).await?
    .middleware(FlashMiddleware)
    .middleware(MessageMiddleware)
    .routes(routes())
    .run()
    .await?;
```

### 2. Usage in Handlers

```rust
use rusti::prelude::*;

async fn create_user(
    Form(form): Form<UserForm>,
    mut message: Message,
) -> Response {
    if !form.is_valid() {
        let _ = message.error("Invalid data").await;
        return redirect("/register");
    }

    // Create user...

    let _ = message.success("Account created successfully!").await;
    redirect("/dashboard")
}
```

### 3. Display in Templates

```html
{% messages %}
```

Or manually:

```html
{% for msg in get_messages() %}
<div class="alert alert-{{ msg.level }}">
    {{ msg.message }}
</div>
{% endfor %}
```

### 4. Utility Macros

To simplify sending messages, Rusti provides macros:

```rust
use rusti::prelude::*;

async fn create_user(
    Form(form): Form<UserForm>,
    mut message: Message,
) -> Response {
    if !form.is_valid() {
        // error! macro - more concise
        error!(message, "Invalid data");
        return redirect("/register");
    }

    // Create user...

    // success! macro - more concise
    success!(message, "Account created successfully!");
    redirect("/dashboard")
}
```

**Available macros:**

| Macro | Equivalent | Usage |
|-------|-----------|-------|
| `success!(msg, "text")` | `msg.success("text").await.unwrap()` | Success messages |
| `error!(msg, "text")` | `msg.error("text").await.unwrap()` | Error messages |
| `info!(msg, "text")` | `msg.info("text").await.unwrap()` | Information messages |
| `warning!(msg, "text")` | `msg.warning("text").await.unwrap()` | Warning messages |

**Multiple messages at once:**

```rust
// Send multiple successive messages
success!(
    message,
    "User created",
    "Email sent",
    "Welcome!"
);

// Or more readable
success!(message, "User created");
info!(message, "Check your email");
warning!(message, "Remember to validate your account");
```

**Macro advantages:**
- ✅ More concise syntax
- ✅ Automatic handling of `.await.unwrap()`
- ✅ Support for multiple messages
- ✅ More readable code

---

## Complete Example: Simple Blog

### Structure

```
blog/
├── Cargo.toml
├── .env
├── src/
│   ├── main.rs
│   ├── models.rs
│   ├── forms.rs
│   └── views.rs
├── templates/
│   ├── base.html
│   ├── posts/
│   │   ├── list.html
│   │   ├── detail.html
│   │   └── create.html
└── static/
    └── css/
        └── style.css
```

### Model (src/models.rs)

```rust
use rusti::prelude::*;
use sea_orm::entity::prelude::*;
use rusti::impl_objects;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "posts")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub title: String,
    pub slug: String,
    #[sea_orm(column_type = "Text")]
    pub content: String,
    pub published: bool,
    pub created_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl_objects!(Entity);
```

### Form (src/forms.rs)

```rust
use rusti::prelude::*;
use rusti::forms::prelude::*;

#[derive(DeriveModelForm, Debug, Clone, Serialize, Deserialize)]
#[sea_orm(model = "crate::models::Model", entity = "crate::models::Entity")]
pub struct PostForm {
    #[field(max_length = 200, required = true)]
    pub title: CharField,

    #[field(max_length = 200, required = true)]
    pub slug: CharField,

    #[field(widget = "textarea", required = true)]
    pub content: CharField,

    #[field(default = "false")]
    pub published: BooleanField,
}
```

### Views (src/views.rs)

```rust
use rusti::prelude::*;
use crate::models::{posts, Entity as Post};
use crate::forms::PostForm;

// List articles
pub async fn list_posts(
    Extension(db): Extension<Arc<DatabaseConnection>>,
    template: Template,
) -> Response {
    let posts = Post::objects
        .filter(posts::Column::Published.eq(true))
        .order_by_desc(posts::Column::CreatedAt)
        .all(&*db)
        .await
        .unwrap_or_default();

    template.render("posts/list.html", context! {
        posts: posts,
    })
}

// Article detail
pub async fn detail_post(
    Path(id): Path<i32>,
    Extension(db): Extension<Arc<DatabaseConnection>>,
    template: Template,
) -> Response {
    let post = match Post::objects.get(&*db, id).await {
        Ok(p) => p,
        Err(_) => return (StatusCode::NOT_FOUND, "Article not found").into_response(),
    };

    template.render("posts/detail.html", context! {
        post: post,
    })
}

// Creation form
pub async fn create_post_view(template: Template) -> Response {
    let form = PostForm::new();
    template.render("posts/create.html", context! {
        form: form,
    })
}

// Creation processing
pub async fn create_post_submit(
    Form(form): Form<PostForm>,
    Extension(db): Extension<Arc<DatabaseConnection>>,
    template: Template,
    mut message: Message,
) -> Response {
    if !form.is_valid() {
        return template.render("posts/create.html", context! {
            form: form,
        });
    }

    match form.save(&*db).await {
        Ok(post) => {
            success!(message, "Article created successfully!");
            redirect(&format!("/posts/{}/", post.id))
        }
        Err(_) => {
            error!(message, "Error creating article");
            template.render("posts/create.html", context! {
                form: form,
            })
        }
    }
}
```

### Routes (src/main.rs)

```rust
use rusti::prelude::*;

mod models;
mod forms;
mod views;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = Settings::from_env();

    let db_config = DatabaseConfig::from_env()?.build();
    let db = db_config.connect().await?;

    RustiApp::new(settings).await?
        .with_database(db)
        .middleware(CsrfMiddleware::new())
        .middleware(SecurityHeadersMiddleware::new())
        .middleware(FlashMiddleware)
        .middleware(MessageMiddleware)
        .routes(routes())
        .run()
        .await?;

    Ok(())
}

fn routes() -> Router {
    urlpatterns![
        path!("", views::list_posts, "post_list"),
        path!("posts/<id>/", views::detail_post, "post_detail"),
        path!("posts/create/", views::create_post_view, "post_create"),
        path!("posts/create/submit/", views::create_post_submit),
    ]
}
```

### List Template (templates/posts/list.html)

```html
{% extends "base.html" %}

{% block title %}Articles{% endblock %}

{% block content %}
<h2>All Articles</h2>

{% messages %}

{% for post in posts %}
<article>
    <h3>{{ post.title }}</h3>
    <p>{{ post.content|truncate(200) }}</p>
    <a href="{% link 'post_detail' id=post.id %}">Read more</a>
</article>
{% endfor %}

<a href="{% link 'post_create' %}">Create article</a>
{% endblock %}
```

---

## Next Steps

Now that you master the basics, explore:

1. **[Advanced Configuration](CONFIGURATION.md)** - Environment variables, settings
2. **[Database](DATABASE.md)** - Relations, transactions, migrations
3. **[Security](SECURITY.md)** - CSP, CSRF, XSS, HTTP headers
4. **[Templates](TEMPLATES.md)** - Custom tags, filters, preprocessing
5. **[Deployment](DEPLOYMENT.md)** - Production, Docker, reverse proxy

---

## Need Help?

- 📖 [Complete Documentation](README.md)
- 🐛 [Report a bug](https://github.com/your-username/rusti/issues)
- 💬 [Discord](#)

---

**Happy coding with Rusti! 🚀**

---

**Version:** 1.0 (Corrected - January 2, 2026)
**License:** MIT