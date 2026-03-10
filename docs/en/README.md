# 📚 Runique Documentation — English

Complete documentation for the Runique web framework.

---

## 📖 Documentation Sections

### 1️⃣ [Installation](https://github.com/seb-alliot/runique/blob/main/docs/en/installation/01-installation.md)

Get started with Runique. Installation, dependencies, and first steps.

**Topics covered:**

* Prerequisites
* Installation steps
* Project setup
* First application

👉 **Go to**: [Installation Guide](https://github.com/seb-alliot/runique/blob/main/docs/en/installation/01-installation.md)

---

### 2️⃣ [Architecture](https://github.com/seb-alliot/runique/blob/main/docs/en/architecture/02-architecture.md)

Understand Runique’s internal architecture.

**Topics covered:**

* Project structure
* Component overview
* Design patterns
* How it works

👉 **Go to**: [Architecture Guide](https://github.com/seb-alliot/runique/blob/main/docs/en/architecture/02-architecture.md)

---

### 3️⃣ [Configuration](https://github.com/seb-alliot/runique/blob/main/docs/en/configuration/03-configuration.md)

Configure your Runique application.

**Topics covered:**

* Server configuration
* Database setup
* Environment variables
* Security settings

👉 **Go to**: [Configuration Guide](https://github.com/seb-alliot/runique/blob/main/docs/en/configuration/03-configuration.md)

---

### 4️⃣ [Routing](https://github.com/seb-alliot/runique/blob/main/docs/en/routing/04-routing.md)

URL routing and request handling.

**Topics covered:**

* URL patterns
* Route definition
* Request handlers
* URL parameters

👉 **Go to**: [Routing Guide](https://github.com/seb-alliot/runique/blob/main/docs/en/routing/04-routing.md)

---

### 5️⃣ [Forms](https://github.com/seb-alliot/runique/blob/main/docs/en/formulaire/05-forms.md)

Creating and managing forms.

**Topics covered:**

* Prisme extractor
* Manual declaration via `RuniqueForm`
* Model/schema-based declaration (AST) and automatic form generation
* Field types (FieldBuilder)
* Validation and persistence
* Template rendering

👉 **Go to**: [Forms Guide](https://github.com/seb-alliot/runique/blob/main/docs/en/formulaire/05-forms.md)

---

### 6️⃣ [Templates](https://github.com/seb-alliot/runique/blob/main/docs/en/template/06-templates.md)

Working with Tera templates.

**Topics covered:**

* Django-like tags (`{% static %}`, `{% form.xxx %}`, `{% link %}`, `{% csrf %}`, `{% messages %}`, `{% csp_nonce %}`)
* Tera filters (`static`, `media`, `form`, `csrf_field`)
* Tera functions (`csrf()`, `nonce()`, `link()`)
* `context_update!` macro
* Template inheritance
* Auto-injected variables

👉 **Go to**: [Templates Guide](https://github.com/seb-alliot/runique/blob/main/docs/en/template/06-templates.md)

---

### 7️⃣ [ORM](https://github.com/seb-alliot/runique/blob/main/docs/en/orm/07-orm.md)

Database operations with SeaORM.

**Topics covered:**

* Model definition
* Queries
* Relations
* Migrations

👉 **Go to**: [ORM Guide](https://github.com/seb-alliot/runique/blob/main/docs/en/orm/07-orm.md)

---

### 8️⃣ [Middlewares](https://github.com/seb-alliot/runique/blob/main/docs/en/middleware/08-middleware.md)

Security and request middlewares.

**Topics covered:**

* Middleware stack with slot system
* CSRF protection (Double Submit Cookie)
* Content Security Policy (CSP) with nonce
* Allowed Hosts validation
* Security headers
* Session configuration
* Intelligent Builder vs Classic Builder

👉 **Go to**: [Middlewares Guide](https://github.com/seb-alliot/runique/blob/main/docs/en/middleware/08-middleware.md)

---

### 9️⃣ [Flash Messages](https://github.com/seb-alliot/runique/blob/main/docs/en/flash/09-flash-messages.md)

User feedback and notifications.

**Topics covered:**

* Redirect macros: `success!`, `error!`, `info!`, `warning!`
* Immediate macro: `flash_now!`
* Rendering with `{% messages %}`
* Flash vs `flash_now` pattern
* Single-read consumption behavior

👉 **Go to**: [Flash Messages Guide](https://github.com/seb-alliot/runique/blob/main/docs/en/flash/09-flash-messages.md)

---

### 🔟 [Examples](https://github.com/seb-alliot/runique/blob/main/docs/en/exemple/10-examples.md)

Complete code examples and projects.

**Topics covered:**

* Blog application
* Authentication
* File uploads
* REST API

👉 **Go to**: [Examples Guide](https://github.com/seb-alliot/runique/blob/main/docs/en/exemple/10-examples.md)

---

### 11️⃣ [Admin](https://github.com/seb-alliot/runique/blob/main/docs/en/admin/11-Admin.md)

## 🧭 Admin View (Beta)

Runique includes a **beta admin view**, based on the declarative `admin!` macro and an automatic code generation system.

Administrable resources are declared in `src/admin.rs`.
From this declaration, Runique automatically generates a complete CRUD interface (routes, handlers, forms) as **standard Rust code**, readable and auditable.

This approach emphasizes:

* **Type safety** (compile-time validation of models and forms)
* **Transparency** (no hidden logic, no opaque procedural magic)
* **Developer control** over generated code

The daemon (`runique start`) enables automatic regeneration, while a `cargo run` workflow can be used when manual modifications are required.

> The admin view is currently in **beta** and intentionally built on simple, declarative, and safe foundations. Future improvements are planned (finer permissions, better feedback, additional safeguards).

👉 **Go to**: [Examples Guide](https://github.com/seb-alliot/runique/blob/main/docs/en/admin/11-Admin.md)

---

### 12. [Model](https://github.com/seb-alliot/runique/blob/main/docs/en/model/12-model.md)

`model!` DSL, SeaORM entity generation and form generation.

**Topics covered:**
- `model!` DSL
- Form generation from model
- Code generation (SeaORM entities)

👉 **Go to**: [Model Guide](https://github.com/seb-alliot/runique/blob/main/docs/en/model/12-model.md)

---

### 13. [Authentication](https://github.com/seb-alliot/runique/blob/main/docs/en/auth/13-authentification.md)

User authentication: sessions, middleware, `is_staff` / `is_superuser` model.

**Topics covered:**
- User model
- Session helpers (login, logout)
- Authentication middleware
- Full example

👉 **Go to**: [Authentication Guide](https://github.com/seb-alliot/runique/blob/main/docs/en/auth/13-authentification.md)

---

### 14. [Sessions](https://github.com/seb-alliot/runique/blob/main/docs/en/session/14-sessions.md)

In-memory session management with leak protection.

**Topics covered:**
- Store (`CleaningMemoryStore`)
- Reading / writing session data
- Protecting sensitive sessions

👉 **Go to**: [Sessions Guide](https://github.com/seb-alliot/runique/blob/main/docs/en/session/14-sessions.md)

---

### 15. [Environment Variables](https://github.com/seb-alliot/runique/blob/main/docs/en/env/15-env.md)

All `.env` variables recognized by Runique.

**Topics covered:**
- Application and server
- Security and sessions
- Assets and media

👉 **Go to**: [Environment Variables Guide](https://github.com/seb-alliot/runique/blob/main/docs/en/env/15-env.md)

---

## 🎯 Quick Navigation

| Section  | File                                   | Topics                                     |
| -------- | -------------------------------------- | ------------------------------------------ |
| Setup    | [Installation](https://github.com/seb-alliot/runique/blob/main/docs/en/installation/01-installation.md)     | Prerequisites, install, first steps        |
| Learning | [Architecture](https://github.com/seb-alliot/runique/blob/main/docs/en/architecture/02-architecture.md)     | Structure, design, internals               |
| Config   | [Configuration](https://github.com/seb-alliot/runique/blob/main/docs/en/configuration/03-configuration.md)   | Settings, environment, security            |
| Routes   | [Routing](https://github.com/seb-alliot/runique/blob/main/docs/en/routing/04-routing.md)               | URL patterns, handlers, params             |
| Forms    | [Forms](https://github.com/seb-alliot/runique/blob/main/docs/en/formulaire/05-forms.md)                   | Prisme, FieldBuilder, `#[form(...)]`       |
| Views    | [Templates](https://github.com/seb-alliot/runique/blob/main/docs/en/template/06-templates.md)           | Django-like tags, filters, Tera functions  |
| Data     | [ORM](https://github.com/seb-alliot/runique/blob/main/docs/en/orm/07-orm.md)                       | Models, queries, `impl_objects!`           |
| Security | [Middlewares](https://github.com/seb-alliot/runique/blob/main/docs/en/middleware/08-middleware.md)        | Slots, CSRF, CSP, sessions                 |
| Feedback | [Flash Messages](https://github.com/seb-alliot/runique/blob/main/docs/en/flash/09-flash-messages.md) | `success!`, `flash_now!`, `{% messages %}` |
| Code     | [Examples](https://github.com/seb-alliot/runique/blob/main/docs/en/exemple/10-examples.md)             | Complete projects                          |
| Admin    | [Admin](https://github.com/seb-alliot/runique/blob/main/docs/en/admin/11-Admin.md)                   | Admin (beta)                               |
| Model    | [Model](https://github.com/seb-alliot/runique/blob/main/docs/en/model/12-model.md)                   | `model!`, entities, generation             |
| Auth     | [Authentication](https://github.com/seb-alliot/runique/blob/main/docs/en/auth/13-authentification.md) | login, logout, middleware, is_staff        |
| Sessions | [Sessions](https://github.com/seb-alliot/runique/blob/main/docs/en/session/14-sessions.md)           | store, read/write, protection              |
| Env      | [Env Variables](https://github.com/seb-alliot/runique/blob/main/docs/en/env/15-env.md)               | configuration, security, assets            |

---

## 🚀 Where to Start?

1. **New to Runique?** → Start with [Installation](https://github.com/seb-alliot/runique/blob/main/docs/en/installation/01-installation.md)
2. **Want to understand the internals?** → Read [Architecture](https://github.com/seb-alliot/runique/blob/main/docs/en/architecture/02-architecture.md)
3. **Ready to code?** → Check out [Examples](https://github.com/seb-alliot/runique/blob/main/docs/en/exemple/10-examples.md)
4. **Need help?** → Browse the relevant section above

---

## 📋 Documentation Features

* ✅ Complete and detailed
* ✅ Code examples included
* ✅ Best practices highlighted
* ✅ Common pitfalls addressed
* ✅ Cross-references and links

---

## 🌍 Language

* 🇫🇷 **[Français](https://github.com/seb-alliot/runique/blob/main/README.fr.md)**
* 🇬🇧 **[English](https://github.com/seb-alliot/runique/blob/main/README.md)**

---

## 💡 Tips

* Each guide includes practical examples
* Follow sections in order for a structured learning path
* Refer to examples for real-world code
* Use your browser’s search feature for quick navigation

---

**Need help?** Check [Examples](https://github.com/seb-alliot/runique/blob/main/docs/en/exemple/10-examples.md) or review the relevant section.
