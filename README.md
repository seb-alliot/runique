# 🚀 Runique - Django-inspired Rust Web Framework

> **⚠️ Note**: This documentation was generated with the assistance of AI.
> While efforts have been made to ensure accuracy, some links or details may contain errors.
> Please report any issues on [GitHub](https://github.com/seb-alliot/runique/issues).


[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange)]()
[![Tests](https://img.shields.io/badge/tests-36%2F36%20passing-brightgreen)]()
[![License](https://img.shields.io/badge/license-MIT-green)]()
[![Version](https://img.shields.io/badge/version-1.1.20-blue)]()
[![Crates.io](https://img.shields.io/crates/v/runique)]()

A modern and comprehensive Rust web framework inspired by Django, for building robust and performant web applications.

🌍 **Languages** : [English](#-installation) | [🇫🇷 Français](README.fr.md)

## 📚 Table of Contents

- 🚀 [Installation](#-installation)
- 🏗️ [Architecture](#️-architecture)
- ⚙️ [Configuration](#️-configuration)
- 🛣️ [Routing](#️-routing)
- 📝 [Forms](#-forms)
- 🎨 [Templates](#-templates)
- 🗄️ [ORM](#️-orm)
- 🔒 [Middleware](#-middleware)
- 💬 [Flash Messages](#-flash-messages)
- 🎓 [Examples](#-examples)

---

## 🚀 Installation

**Full Documentation** : [Installation Guide](https://github.com/seb-alliot/runique/blob/main/docs/en/01-installation.md)

Quick start:

```bash
git clone https://github.com/seb-alliot/runique
cd runique
cargo build
cargo test --all
```

👉 **Read** : [docs/en/01-installation.md](https://github.com/seb-alliot/runique/blob/main/docs/en/01-installation.md) for complete details

---

## 🏗️ Architecture

**Full Documentation** : [Architecture Guide](https://github.com/seb-alliot/runique/blob/main/docs/en/02-architecture.md)

Overview of Runique's architecture:

```
Runique Framework
├── Forms System      # Type-safe forms
├── Routing Engine    # URL pattern routing
├── Template Engine   # Tera templates
├── Middleware Stack  # Security & headers
├── ORM Layer         # SeaORM integration
└── Utils             # Helpers and utilities
```

👉 **Read** : [docs/en/02-architecture.md](https://github.com/seb-alliot/runique/blob/main/docs/en/02-architecture.md) for internal structure

---

## ⚙️ Configuration

**Full Documentation** : [Configuration Guide](https://github.com/seb-alliot/runique/blob/main/docs/en/03-configuration.md)

Configure your server and application:

```rust
let settings = Settings {
    server: ServerConfig { ... },
    database: DatabaseConfig { ... },
    security: SecurityConfig { ... },
};
```

👉 **Read** : [docs/en/03-configuration.md](https://github.com/seb-alliot/runique/blob/main/docs/en/03-configuration.md) for all options

---

## 🛣️ Routing

**Full Documentation** : [Routing Guide](https://github.com/seb-alliot/runique/blob/main/docs/en/04-routing.md)

Define your routes with Axum's `Router`:

```rust
use axum::routing::{get, post};

fn routes() -> Router {
    Router::new()
        .route("/", (views::home))
        .route("/api/users", (views::create_user))
}
```

👉 **Read** : [docs/en/04-routing.md](https://github.com/seb-alliot/runique/blob/main/docs/en/04-routing.md) for patterns and options

---

## 📝 Forms

**Full Documentation** : [Forms Guide](https://github.com/seb-alliot/runique/blob/main/docs/en/05-forms.md)

Create forms easily with `#[derive(RuniqueForm)]`:

```rust
use crate::views;
use runique::prelude::*;
use runique::{urlpatterns, view}; // <= Macros must be here

pub fn routes() -> Router {
    let router = urlpatterns! {
        "/" => view!{ views::index }, name = "index",

        "/about" => view! { views::about }, name = "about",
        "/inscription" => view! { views::inscription }, name = "inscription",
    };
    router
}

pub async fn registration(
    mut request: Request,
    Prisme(mut form): Prisme<RegisterForm>,
) -> AppResult<Response> {
    if request.is_get() {
        context_update!(request => {
            "title" => "User Registration",
            "registration_form" => &form,
        });
        return request.render("registration_form.html");
    }

    if request.is_post() {
        if form.is_valid().await {
            let user = form.save(&request.engine.db).await.map_err(|err| {
                form.get_form_mut().database_error(&err);
                AppError::from(err)
            })?;
            println!("New user created in views.rs: {:?}", user);

            success!(
                request.notices =>
                format!("Welcome {}, your account has been created!", user.username)
            );
            return Ok(Redirect::to("/").into_response());
        }

        // Validation failed
        context_update!(request => {
            "title" => "Validation Error",
            "registration_form" => &form,
            "messages" => flash_now!(error => "Please correct the errors"),
        });
        return request.render("registration_form.html");
    }

    // Fallback case
    request.render("registration_form.html")
}

```

👉 **Read** : [docs/en/05-forms.md](https://github.com/seb-alliot/runique/blob/main/docs/en/05-forms.md) for all field types## 🎨 Templates

**Full Documentation** : [Templates Guide](https://github.com/seb-alliot/runique/blob/main/docs/en/06-templates.md)

Use Tera templates:

```html
<h1>{{ title }}</h1>
{% for item in items %}
  <p>{{ item }}</p>
{% endfor %}
```

👉 **Read** : [docs/en/06-templates.md](https://github.com/seb-alliot/runique/blob/main/docs/en/06-templates.md) for complete syntax

---

## 🗄️ ORM

**Full Documentation** : [ORM Guide](https://github.com/seb-alliot/runique/blob/main/docs/en/07-orm.md)

Use SeaORM with Django-like pattern:

```rust
impl_objects!(User);

let users = User::objects
    .filter(active.eq(true))
    .all(&db)
    .await?;
```

👉 **Read** : [docs/en/07-orm.md](https://github.com/seb-alliot/runique/blob/main/docs/en/07-orm.md) for advanced queries

---

## 🔒 Middleware

**Full Documentation** : [Middleware Guide](https://github.com/seb-alliot/runique/blob/main/docs/en/08-middleware.md)

Integrated security middleware:

- CSRF Protection
- Content-Security-Policy (CSP)
- Allowed Hosts
- Security Headers
- XSS Sanitizer

👉 **Read** : [docs/en/08-middleware.md](https://github.com/seb-alliot/runique/blob/main/docs/en/08-middleware.md) for configuration

---

## 💬 Flash Messages

**Full Documentation** : [Flash Messages Guide](https://github.com/seb-alliot/runique/blob/main/docs/en/09-flash-messages.md)

Temporary messages for users:

```rust
success!("Operation successful!");
error!("An error occurred");
warning!("Warning!");
```

👉 **Read** : [docs/en/09-flash-messages.md](https://github.com/seb-alliot/runique/blob/main/docs/en/09-flash-messages.md) for details

---

## 🎓 Examples

**Full Documentation** : [Examples Guide](https://github.com/seb-alliot/runique/blob/main/docs/en/10-examples.md)

Complete usage examples:

- Complete blog application
- User authentication
- File upload
- REST API

👉 **Read** : [docs/en/10-examples.md](https://github.com/seb-alliot/runique/blob/main/docs/en/10-examples.md) for complete examples

---

## 🧪 Tests

```bash
# Unit tests
cargo test --lib

# Integration tests
cargo test --test integration_tests

# All tests
cargo test --all
```

Results: **36/36 tests passing** ✅

---

## 📖 Full Documentation

### English (EN)
- [Installation](https://github.com/seb-alliot/runique/blob/main/docs/en/01-installation.md)
- [Architecture](https://github.com/seb-alliot/runique/blob/main/docs/en/02-architecture.md)
- [Configuration](https://github.com/seb-alliot/runique/blob/main/docs/en/03-configuration.md)
- [Routing](https://github.com/seb-alliot/runique/blob/main/docs/en/04-routing.md)
- [Forms](https://github.com/seb-alliot/runique/blob/main/docs/en/05-forms.md)
- [Templates](https://github.com/seb-alliot/runique/blob/main/docs/en/06-templates.md)
- [ORM](https://github.com/seb-alliot/runique/blob/main/docs/en/07-orm.md)
- [Middleware](https://github.com/seb-alliot/runique/blob/main/docs/en/08-middleware.md)
- [Flash Messages](https://github.com/seb-alliot/runique/blob/main/docs/en/09-flash-messages.md)
- [Examples](https://github.com/seb-alliot/runique/blob/main/docs/en/10-examples.md)

### Français (FR)
- [Installation](https://github.com/seb-alliot/runique/blob/main/docs/fr/01-installation.md)
- [Architecture](https://github.com/seb-alliot/runique/blob/main/docs/fr/02-architecture.md)
- [Configuration](https://github.com/seb-alliot/runique/blob/main/docs/fr/03-configuration.md)
- [Routage](https://github.com/seb-alliot/runique/blob/main/docs/fr/04-routing.md)
- [Formulaires](https://github.com/seb-alliot/runique/blob/main/docs/fr/05-forms.md)
- [Templates](https://github.com/seb-alliot/runique/blob/main/docs/fr/06-templates.md)
- [ORM](https://github.com/seb-alliot/runique/blob/main/docs/fr/07-orm.md)
- [Middlewares](https://github.com/seb-alliot/runique/blob/main/docs/fr/08-middleware.md)
- [Flash Messages](https://github.com/seb-alliot/runique/blob/main/docs/fr/09-flash-messages.md)
- [Exemples](https://github.com/seb-alliot/runique/blob/main/docs/fr/10-examples.md)

---

## 🎯 Quick Start

1. **Read** [Installation](https://github.com/seb-alliot/runique/blob/main/docs/en/01-installation.md)
2. **Understand** [Architecture](https://github.com/seb-alliot/runique/blob/main/docs/en/02-architecture.md)
3. **Check** [Examples](https://github.com/seb-alliot/runique/blob/main/docs/en/10-examples.md)
4. **Start coding** your application

---

## 📊 Project Status

- ✅ **Compilation** : No errors
- ✅ **Tests** : 36/36 passing (100%)
- ✅ **Documentation** : Complete (EN & FR)
- ✅ **Production** : Ready

See [PROJECT_STATUS.md](https://github.com/seb-alliot/runique/blob/main/PROJECT_STATUS.md) for more details.

---

## 🔗 Resources

- 📁 [Project Structure](https://github.com/seb-alliot/runique/blob/main/INDEX.md)
- 📊 [Full Status](https://github.com/seb-alliot/runique/blob/main/PROJECT_STATUS.md)
- 🧪 [Test Reports](https://github.com/seb-alliot/runique/blob/main/TEST_REPORT.md)
- 📋 [Changelog](https://github.com/seb-alliot/runique/blob/main/CHANGELOG.md)
- 📖 [Documentation Guide](https://github.com/seb-alliot/runique/blob/main/docs/README.md)

---

## 📝 License

MIT License [LICENCE](https://github.com/seb-alliot/runique/blob/main/LICENCE) - see [SECURITY.md](https://github.com/seb-alliot/runique/blob/main/SECURITY.md)

---

## 🚀 Production Ready

The Runique framework is **stable, tested and documented**, ready for production use.

**Start now** → [Installation](https://github.com/seb-alliot/runique/blob/main/docs/en/01-installation.md)

---

🌍 **Available in**: [English](https://github.com/seb-alliot/runique/blob/main/README.md) | [🇫🇷 Français](https://github.com/seb-alliot/runique/blob/main/README.fr.md)