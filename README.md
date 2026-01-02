# Rusti

**A Django-inspired Rust web framework**

Rusti is a modern web framework that combines Rust's safety and performance with Django's ergonomics. It offers a familiar development experience for Django developers while leveraging the power of Rust's type system.

[![Version](https://img.shields.io/badge/version-1.0.0-blue.svg)](https://github.com/seb-alliot/rusti)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE-MIT)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)

---

## 🚀 Main Features

### Django-like Architecture
- **Declarative routing** with `urlpatterns!` macro
- **Intuitive ORM** based on SeaORM with Django-style API
- **Template system** Tera with custom preprocessing
- **Automatic form generation** via procedural macros
- **Flash messages** between requests
- **Static and media file management**

### Built-in Security
- ✅ **CSRF Protection** (HMAC-SHA256)
- ✅ **Content Security Policy** (CSP) with nonces
- ✅ **XSS Sanitization** (ammonia)
- ✅ **Automatic Security Headers** (HSTS, X-Frame-Options, etc.)
- ✅ **ALLOWED_HOSTS Validation**
- ✅ **Integrated Argon2id Hashing**

### Multi-database Support
- PostgreSQL
- MySQL / MariaDB
- SQLite

### Modern Development
- **Native Async/await** with Tokio
- **Type-safe** thanks to Rust's type system
- **Zero-cost abstractions**
- **Hot reload** in development
- **Complete documentation** in French and English

---

## 📦 Installation

### Prerequisites

- Rust 1.75+ ([install Rust](https://www.rust-lang.org/tools/install))
- Cargo

### Add Rusti to Your Project

```toml
# Cargo.toml

# Minimal configuration (SQLite by default)
[dependencies]
rusti = "1.0.0"

# With PostgreSQL
[dependencies]
rusti = { version = "1.0.0", features = ["postgres"] }

# With MySQL
[dependencies]
rusti = { version = "1.0.0", features = ["mysql"] }

# With MariaDB
[dependencies]
rusti = { version = "1.0.0", features = ["mariadb"] }

# With all databases
[dependencies]
rusti = { version = "1.0.0", features = ["all-databases"] }
```

### Available Cargo Features

| Feature | Description | Default |
|---------|-------------|---------|
| `default` | Enables ORM support with SQLite | ✅ |
| `orm` | Enables SeaORM | ✅ (included in `default`) |
| `sqlite` | SQLite driver | ✅ (included in `orm`) |
| `postgres` | PostgreSQL driver | ❌ |
| `mysql` | MySQL driver | ❌ |
| `mariadb` | MariaDB driver (uses MySQL driver) | ❌ |
| `all-databases` | Enables all drivers simultaneously | ❌ |

**Configuration examples:**

```toml
# SQLite only (default configuration)
[dependencies]
rusti = "1.0.0"

# PostgreSQL + MySQL
[dependencies]
rusti = { version = "1.0.0", features = ["postgres", "mysql"] }

# All databases
[dependencies]
rusti = { version = "1.0.0", features = ["all-databases"] }

# Without ORM (minimal framework)
[dependencies]
rusti = { version = "1.0.0", default-features = false }
```

### Create a New Project

```bash
cargo new my_app
cd my_app
```

Add Rusti to `Cargo.toml`:

```toml
[dependencies]
rusti = { version = "1.0.0", features = ["postgres"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
```

---

## 🏁 Quick Start

### Minimal Application

```rust
// src/main.rs
use rusti::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = Settings::from_env();

    RustiApp::new(settings).await?
        .routes(routes())
        .run()
        .await?;

    Ok(())
}

fn routes() -> Router {
    urlpatterns![
        "/" => view!{
            GET => views::index
        },
        name ="index",

        "/hello" => view!{
            GET => views::hello
        },
        name ="hello",

        "/user" => view! {
            GET => views::user_profile,
            POST => views::user_profile_submit
        },
         name = "user_profile",
    ]
}

async fn index() -> &'static str {
    "Welcome to Rusti!"
}

async fn hello(Path(name): Path<String>) -> String {
    format!("Hello, {}!", name)
}

pub async fn user_profile(
    template: Template,
    ExtractForm(form): ExtractForm<ModelForm>,
) -> Response {
    let ctx = context! {
        "title", "User Profile";
        "form", form
    };
    template.render("profile/register_profile.html", &ctx)
}

pub async fn user_profile_submit(
    Extension(db): Extension<Arc<DatabaseConnection>>,
    mut message: Message,
    template: Template,
    ExtractForm(user): ExtractForm<ModelForm>,
) -> Response {
    if user.is_valid() {
        match user.save(&db).await {
            Ok(created_user) => {
                success!(message, "User profile created successfully!");
                let target = reverse_with_parameters(
                    "user_profile",
                    &[
                        ("id", &created_user.id.to_string()),
                        ("name", &created_user.username),
                    ],
                )
                .unwrap();
                return Redirect::to(&target).into_response();
            }
            Err(err) => {
                // Database unique constraint error handling
                let error_msg = if err.to_string().contains("unique") {
                    if err.to_string().contains("username") {
                        "This username is already taken!"
                    } else if err.to_string().contains("email") {
                        "This email is already in use!"
                    } else {
                        "This value already exists in the database"
                    }
                } else {
                    "Error occurred during save"
                };
                error!(message, error_msg);
                let ctx = context! {
                    "form", ModelForm::build();
                    "forms_errors", user.get_errors();
                    "title", "Profile";
                    "db_error", error_msg
                };
                return template.render("name.html", &ctx);
            }
        }
    }
    
    // 2. Validation error scenarios (invalid field inputs)
    error!(message, "Form validation error");

    let ctx = context! {
        "form", ModelForm::build();
        "forms_errors", user.get_errors();
        "title", "Validation Error"
    };
    template.render("name.html", &ctx)
}
```

### Configuration (.env)

```env
HOST=127.0.0.1
PORT=8000
SECRET_KEY=your-secret-key-here
ALLOWED_HOSTS=localhost,127.0.0.1
DEBUG=true

# PostgreSQL
DB_ENGINE=postgres
DB_USER=user
DB_PASSWORD=password
DB_HOST=localhost
DB_PORT=5432
DB_NAME=mydb
```

### Launch

```bash
cargo run
```

Open [http://localhost:8000](http://localhost:8000)

---
## 📚 Complete Documentation

### 📚 Documentation (English)

- [🚀 Getting Started](informations/documentation_english/GETTING_STARTED.md)
- [⚙️ Configuration](informations/documentation_english/CONFIGURATION.md)
- [🗄️ Database](informations/documentation_english/DATABASE.md)
- [📝 Forms](informations/documentation_english/FORMULAIRE.md)
- [🎨 Templates](informations/documentation_english/TEMPLATES.md)
- [🔒 Security](informations/documentation_english/CSP.md)
- [🛣️ Macro](informations/documentation_english/MACRO%2520CONTEXT.md)
- [🔧 changelog](informations/documentation_english/CHANGELOG.md)
- [🚀 Contribuer](informations/documentation_english/CONTRIBUTING.md)
- [🆕 New project](informations/documentation_english/NEW_PROJECT.md)
- [📖 Documentation Overview](README.md)

### 📚 Documentation (French)

- [🚀 Getting Started](informations/documentation_french/GETTING_STARTED.md)
- [⚙️ Configuration](informations/documentation_french/CONFIGURATION.md)
- [🗄️ Database](informations/documentation_french/DATABASE.md)
- [📝 Forms](informations/documentation_french/FORMULAIRE.md)
- [🎨 Templates](informations/documentation_french/TEMPLATES.md)
- [🔒 Security](informations/documentation_french/CSP.md)
- [🛣️ Macro](informations/documentation_french/MACRO%2520CONTEXT.md)
- [🔧 changelog](informations/documentation_french/CHANGELOG.md)
- [🚀 Contribuer](informations/documentation_french/CONTRIBUTING.md)
- [🆕 New project](informations/documentation_english/NOUVEAU_PROJET.md)
- [📖 Documentation Overview](README.fr.md)

---

## 🎯 Complete Example

### Project Structure

```
my_app/
├── Cargo.toml
├── .env
├── src/
│   ├── main.rs
│   ├── models/
│   │   └── mod.rs
│   ├── views/
│   │   └── mod.rs
│   ├──  forms/
│   |   └── mod.rs
│   └── urls/
│       └── mod.rs
├── templates/
│   ├── base.html
│   └── index.html
└── static/
    ├── css/
    └── js/
```

### Model (models/mod.rs)

```rust
use sea_orm::entity::prelude::*;
use rusti::impl_objects;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "posts")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub title: String,
    pub content: String,
    pub published: bool,
    pub created_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

// Django-like API
impl_objects!(Entity);
```

### Form (forms/mod.rs)

```rust
use rusti::forms::prelude::*;

#[derive(DeriveModelForm, Debug, Clone, Serialize, Deserialize)]
#[sea_orm(model = "crate::models::Model", entity = "crate::models::Entity")]
pub struct PostForm {
    #[form_field(widget = "textarea", required = true)]
    pub title: CharField,

    #[form_field(widget = "textarea", required = true)]
    pub content: CharField,

    #[form_field(default = "false")]
    pub published: BooleanField,
}
```

### View (views/mod.rs)

```rust
use rusti::prelude::*;
use crate::models::{posts, Entity as Post};
use crate::forms::PostForm;

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

pub async fn create_post(
    Form(form): Form<PostForm>,
    Extension(db): Extension<Arc<DatabaseConnection>>,
    template: Template,
    mut message: Message,
) -> Response {
    if !form.is_valid() {
        return template.render("posts/create.html", context! { form });
    }

    match form.save(&*db).await {
        Ok(post) => {
            success!(message, "Article created successfully!");
            redirect(&format!("/posts/{}", post.id))
        }
        Err(_) => {
            error!(message, "Error creating article");
            template.render("posts/create.html", context! { form })
        }
    }
}

```

### Template (templates/posts/list.html)

```html
{% extends "base.html" %}

{% block content %}
<h1>Articles</h1>

{% for post in posts %}
<article>
    <h2>{{ post.title }}</h2>
    <p>{{ post.content|truncate(200) }}</p>
    <a href="{% link 'post_detail' id=post.id %}">Read more</a>
</article>
{% endfor %}

<a href="{% link 'post_create' %}">Create article</a>
{% endblock %}
```

### Routes (main.rs)

```rust
use rusti::prelude::*;

fn routes() -> Router {
    urlpatterns![
        path!("", views::index, "index"),
        path!("posts/", views::list_posts, "post_list"),
        path!("posts/create/", views::create_post, "post_create"),
        path!("posts/<id>/", views::detail_post, "post_detail"),
    ]
}
```

---

## 🔒 Security

Rusti integrates multiple security layers by default:

### CSRF Protection

```rust
RustiApp::new(settings).await?
    .middleware(CsrfMiddleware::new())
    .routes(routes())
    .run()
    .await?;
```

In templates:
```html
<form method="post">
    {% csrf %}
    <!-- form fields -->
</form>
```

### Content Security Policy

```rust
use rusti::middleware::CspConfig;

let csp_config = CspConfig {
    default_src: vec!["'self'".to_string()],
    script_src: vec!["'self'".to_string()],
    style_src: vec!["'self'".to_string(), "'unsafe-inline'".to_string()],
    img_src: vec!["'self'".to_string(), "data:".to_string()],
    font_src: vec!["'self'".to_string()],
    connect_src: vec!["'self'".to_string()],
    frame_ancestors: vec!["'none'".to_string()],
    base_uri: vec!["'self'".to_string()],
    form_action: vec!["'self'".to_string()],
    use_nonce: false,
    ..Default::default()
};

RustiApp::new(settings).await?
    .middleware(SecurityHeadersMiddleware::new())
    .with_default_middleware()
    .routes(routes())
    .run()
    .await?;
```

### Security Headers

```rust
RustiApp::new(settings).await?
    .middleware(SecurityHeadersMiddleware::new())
    .with_default_middleware()
    .routes(routes())
    .run()
    .await?;
```

Headers automatically configured:
- `Strict-Transport-Security`
- `X-Content-Type-Options`
- `X-Frame-Options`
- `X-XSS-Protection`
- `Referrer-Policy`
- `Permissions-Policy`

---

## 🗄️ Database

### Django-like API

```rust
use crate::models::{users, Entity as User};

// Retrieval
let all_users = User::objects.all().all(&db).await?;
let user = User::objects.get(&db, 1).await?;

// Filtering
let active_users = User::objects
    .filter(users::Column::IsActive.eq(true))
    .filter(users::Column::Age.gte(18))
    .all(&db)
    .await?;

// Ordering and pagination
let recent_users = User::objects
    .order_by_desc(users::Column::CreatedAt)
    .limit(10)
    .all(&db)
    .await?;

// Count
let count = User::objects.count(&db).await?;
```

### Migrations

Use `sea-orm-cli` for migrations:

```bash
cargo install sea-orm-cli

# Create a migration
sea-orm-cli migrate generate create_users_table

# Apply
sea-orm-cli migrate up

# Rollback
sea-orm-cli migrate down
```

---

## 🎨 Templates

### Custom Tags

```html
<!-- Static files -->
<link rel="stylesheet" href="{% static 'css/style.css' %}">
<script src='{% static "js/main.js" %}'></script>

<!-- Media files -->
<img src='{% media "media.jpg" %}' alt="Avatar">

<!-- CSRF token -->
<form method="post">
    {% csrf %}
    <!-- ... -->
</form>

<!-- Flash messages -->
{% messages %}

<!-- Links with reverse routing -->
<a href="{% link 'post_detail' id=post.id %}">Details</a>

<!-- CSP nonce (if enabled) -->
<script {{ csp }}>
    // JavaScript code
</script>
```

---

## 📦 Utility Macros

Rusti provides macros to simplify common operations.

### Flash Messages

```rust
use rusti::prelude::*;

async fn my_handler(mut message: Message) -> Response {
    // Simple messages
    success!(message, "Operation successful!");
    error!(message, "An error occurred");
    info!(message, "Important information");
    warning!(message, "Warning");

    // Multiple messages
    success!(
        message,
        "User created",
        "Email sent",
        "Welcome!"
    );

    redirect("/")
}
```

**Advantages:**
- Concise and expressive syntax
- Automatic handling of `.await.unwrap()`
- Support for multiple messages
- More readable and maintainable code

**Available macros:**
- `success!(message, "text")` - Success messages
- `error!(message, "text")` - Error messages
- `info!(message, "text")` - Information messages
- `warning!(message, "text")` - Warning messages

---

## 🚀 Performance

Rusti leverages Rust and Tokio performance:

- **Zero-cost abstractions**: No runtime overhead
- **Native async/await**: Efficient concurrency with Tokio
- **Connection pooling**: Optimized DB connection management
- **Optimized compilation**: Highly optimized binary

### Benchmark (example)

```
Requests/sec: ~50,000
Latency p50: ~1ms
Latency p99: ~5ms
Memory: ~20MB
```

---

## 🛠️ Development

### Tests

```bash
cargo test
```

### Linting

```bash
cargo clippy
```

### Formatting

```bash
cargo fmt
```

### Documentation

```bash
cargo doc --open
```

---

## 🤝 Contributing

Contributions are welcome! Here's how to contribute:

1. Fork the project
2. Create a branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Guidelines

- Write tests for new features
- Follow Rust code conventions (rustfmt)
- Document public APIs
- Add examples if relevant

---

## 📝 Roadmap

### Version 1.1 (Q1 2026)

- [ ] Integrated authentication system
- [ ] Auto-generated admin panel
- [ ] Rate limiting middleware
- [ ] WebSocket support
- [ ] Cache layer (Redis)

### Version 1.2 (Q2 2026)

- [ ] CLI for scaffolding
- [ ] Improved hot reload
- [ ] GraphQL support
- [ ] Background jobs (Tokio tasks)

### Version 2.0 (Q3 2026)

- [ ] Plugin system
- [ ] Multi-tenancy
- [ ] Internationalization (i18n)
- [ ] Advanced ORM features

---

## 📄 License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

---

## 🙏 Acknowledgments

Rusti builds upon excellent libraries from the Rust ecosystem:

- [Axum](https://github.com/tokio-rs/axum) - Web framework
- [Tokio](https://tokio.rs/) - Async runtime
- [SeaORM](https://www.sea-ql.org/SeaORM/) - ORM
- [Tera](https://keats.github.io/tera/) - Template engine
- [Tower](https://github.com/tower-rs/tower) - Middleware
- [Argon2](https://github.com/RustCrypto/password-hashes) - Password hashing
- [ammonia](https://github.com/rust-ammonia/ammonia) - HTML sanitization

---

## 📧 Contact

- **GitHub Issues**: [github.com/seb-alliot/rusti/tree/issues](https://github.com/seb-alliot/rusti/tree/issues)
- **Discord**: [Join the server](https://discord.gg/Y5zW7rbt)
- **Email**: alliotsebastien04@gmail.com

---

## ⭐ Support the Project

If Rusti is useful to you, consider:

- ⭐ Starring on GitHub
- 🐛 Reporting bugs
- 💡 Suggesting features
- 📖 Improving documentation
- 🤝 Contributing code

---

**Build secure and performant web applications with Rusti!**

---

**Version:** 1.0.0 (Corrected - January 2, 2026)
**License:** MIT