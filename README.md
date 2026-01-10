# Runique

**A Django-inspired Rust web framework**

Runique is a modern web framework that combines Rust's safety and performance with Django's ergonomics. It offers a familiar development experience for Django developers while leveraging the power of Rust's type system.

[![Version](https://img.shields.io/badge/version-1.0.86-blue.svg)](https://crates.io/crates/runique)
[![docs.rs](https://img.shields.io/docsrs/runique)](https://docs.rs/runique)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE-MIT)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)

---

## 🤔 Why Runique?

- **For Django developers**: Familiar API and patterns with Rust's performance and safety
- **For Rust developers**: Django-inspired ergonomics without sacrificing type safety
- **For everyone**: Security built-in from day one, not bolted-on as an afterthought

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
- **Complete documentation**

---

## 📦 Installation

### Prerequisites

- Rust 1.75+ ([install Rust](https://www.rust-lang.org/tools/install))
- Cargo

### Add Runique to Your Project

```toml
# Cargo.toml

# Minimal configuration (SQLite by default)
[dependencies]
runique = "1.0.86"

# With PostgreSQL
[dependencies]
runique = { version = "1.0.86", features = ["postgres"] }

# With MySQL
[dependencies]
runique = { version = "1.0.86", features = ["mysql"] }

# With MariaDB
[dependencies]
runique = { version = "1.0.86", features = ["mariadb"] }

# With all databases
[dependencies]
runique = { version = "1.0.86", features = ["all-databases"] }
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
runique = "1.0.86"

# PostgreSQL + MySQL
[dependencies]
runique = { version = "1.0.86", features = ["postgres", "mysql"] }

# All databases
[dependencies]
runique = { version = "1.0.86", features = ["all-databases"] }

# Without ORM (minimal framework)
[dependencies]
runique = { version = "1.0.86", default-features = false }
```

### Create a New Project

```bash
cargo install runique
runique new my_app
cd my_app
```

Add Runique to `Cargo.toml`:

```toml
[dependencies]
runique = { version = "1.0.86", features = ["sqlite"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
```

---

## 🏁 Quick Start

### Minimal Application

```rust
// src/main.rs
use runique::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = Settings::from_env();

    RuniqueApp::new(settings).await?
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
        name = "index",

        "/hello/:name" => view!{
            GET => views::hello
        },
        name = "hello",
    ]
}

async fn index() -> &'static str {
    "Welcome to Runique! 🚀"
}

async fn hello(Path(name): Path<String>) -> String {
    format!("Hello, {}!", name)
}
```

### Configuration (.env)

```env
HOST=127.0.0.1
PORT=8000
SECRET_KEY=your-secret-key-here
ALLOWED_HOSTS=localhost,127.0.0.1
DEBUG=true

# PostgreSQL (optional)
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

**For more advanced examples, see the [Complete Example](#-complete-example) section below.**

---

## 📚 Documentation

- [🚀 Getting Started](informations/documentation_english/GETTING_STARTED.md)
- [⚙️ Configuration](informations/documentation_english/CONFIGURATION.md)
- [🗄️ Database](informations/documentation_english/DATABASE.md)
- [📝 Forms](informations/documentation_english/FORMULAIRE.md)
- [🎨 Templates](informations/documentation_english/TEMPLATES.md)
- [🔒 Security](informations/documentation_english/CSP.md)
- [🛣️ Macro](informations/documentation_english/MACRO_CONTEXT.md)
- [🔧 Changelog](informations/documentation_english/CHANGELOG.md)
- [🚀 Contributing](informations/documentation_english/CONTRIBUTING.md)
- [🆕 New project](informations/documentation_english/NEW_PROJECT.md)
- [📖 API Documentation](https://docs.rs/runique)

---

## 🎯 Complete Example

### Project Structure
### You can use: `cargo install runique` → `runique new project_name`

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

### Advanced Handler with Form Validation

```rust
use runique::prelude::*;

// Form handler with validation
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

// Form submission with error handling
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

    // Validation error scenarios
    error!(message, "Form validation error");

    let ctx = context! {
        "form", ModelForm::build();
        "forms_errors", user.get_errors();
        "title", "Validation Error"
    };
    template.render("name.html", &ctx)
}
```

---

## 🔒 Security

### CSRF Protection

CSRF protection is automatically enabled when using `.with_default_middleware()`.

```rust
use runique::prelude::*;

RuniqueApp::new(settings).await?
    .with_default_middleware()  // Includes CSRF protection
    .routes(routes())
    .run()
    .await?;
```

In your templates:

```html
<form method="post">
    {% csrf %}
    <!-- form fields -->
</form>
```

### Content Security Policy

```rust
use runique::prelude::*;

RuniqueApp::new(settings).await?
    .with_security_headers(CspConfig::strict())
    .with_default_middleware()
    .routes(routes())
    .run()
    .await?;
```

### Security Headers

```rust
RuniqueApp::new(settings).await?
    .with_static_files()?
    .with_allowed_hosts(
        env::var("ALLOWED_HOSTS")
        .ok()
        .map(|s| s.split(',').map(|h| h.to_string()).collect()),
    )
    .with_default_middleware()
    .routes(routes())
    .run()
    .await?;
```

- ✅ Content-Security-Policy
- ✅ X-Content-Type-Options
- ✅ X-Frame-Options
- ✅ X-XSS-Protection
- ✅ Referrer-Policy
- ✅ Permissions-Policy
- 🆕 Cross-Origin-Embedder-Policy
- 🆕 Cross-Origin-Opener-Policy
- 🆕 Cross-Origin-Resource-Policy

---

## 🗄️ Database

### Configuration

```rust
RuniqueApp::new(settings).await?
    .with_database(db)
    .with_static_files()?
    .with_allowed_hosts(
        env::var("ALLOWED_HOSTS")
        .ok()
        .map(|s| s.split(',').map(|h| h.to_string()).collect()),
    )
    .with_sanitize_text_inputs(false)
    .with_default_middleware()
    .routes(routes())
    .run()
    .await?;
```

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

Runique provides macros to simplify common operations.

### Flash Messages

```rust
use runique::prelude::*;

async fn my_handler(mut message: Message) -> Response {
    // Note: Must use `mut message: Message' otherwise it won't work
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

Runique leverages Rust and Tokio performance:

- **Zero-cost abstractions**: No runtime overhead
- **Native async/await**: Efficient concurrency with Tokio
- **Connection pooling**: Optimized DB connection management
- **Optimized compilation**: Highly optimized binary

### Benchmark (indicative)

```
Setup: Local development machine
Requests/sec: ~50,000
Latency p50: ~1ms
Latency p99: ~5ms
Memory: ~20MB
```

*Note: Actual performance depends on your hardware and application complexity. Run your own benchmarks for production estimates.*

---

## 🛠️ Development

### Tests

```bash
# Run all tests
cargo test

# Run integration tests
cargo test --test integration

# Run doc tests
cargo test --doc
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
# Generate and open documentation
cargo doc --open

# Test documentation examples
cargo test --doc
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

See [CONTRIBUTING.md](informations/documentation_english/CONTRIBUTING.md) for more details.

---

## 📝 Roadmap

### Version 1.1

- [ ] Integrated authentication system
- [ ] Auto-generated admin panel
- [ ] Rate limiting middleware
- [ ] WebSocket support
- [ ] Cache layer (Redis)

### Version 1.2

- [x] CLI for scaffolding
- [ ] Improved hot reload
- [ ] GraphQL support
- [ ] Background jobs (Tokio tasks)

### Version 2.0

- [ ] Plugin system
- [ ] Multi-tenancy
- [ ] Internationalization (i18n)
- [ ] Advanced ORM features

---

## 📄 License

This project is licensed under the MIT License. See the [LICENSE-MIT](LICENSE-MIT) file for details.

---

## 🙏 Acknowledgments

Runique builds upon excellent libraries from the Rust ecosystem:

- [Axum](https://github.com/tokio-rs/axum) - Web framework
- [Tokio](https://tokio.rs/) - Async runtime
- [SeaORM](https://www.sea-ql.org/SeaORM/) - ORM
- [Tera](https://keats.github.io/tera/) - Template engine
- [Tower](https://github.com/tower-rs/tower) - Middleware
- [Argon2](https://github.com/RustCrypto/password-hashes) - Password hashing
- [ammonia](https://github.com/rust-ammonia/ammonia) - HTML sanitization

Special thanks to all contributors and the Rust community!

---

## 📧 Contact

- **GitHub Issues**: [Report bugs or request features](https://github.com/seb-alliot/runique/tree/issues)
- **Discord**: [Join our community](https://discord.gg/Y5zW7rbt)
- **Email**: alliotsebastien04@gmail.com
- **Crates.io**: [View on crates.io](https://crates.io/crates/runique)
- **Docs.rs**: [Read the API documentation](https://docs.rs/runique)

---

## ⭐ Support the Project

If Runique is useful to you, consider:

- ⭐ [Starring on GitHub](https://github.com/seb-alliot/runique)
- 🐛 Reporting bugs
- 💡 Suggesting features
- 📖 Improving documentation
- 🤝 Contributing code
- 💬 Joining our Discord community

---

**Build secure and performant web applications with Runique!** 🚀

---

**Version:** 1.0.8
**License:** MIT
**Status:** Stable

*Made with ❤️ and 🦀 by the Runique community*