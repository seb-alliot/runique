# 📚 Runique Documentation — English

Complete documentation for the Runique web framework.

---

## 📖 Documentation Sections

### 1️⃣ [Installation](01-installation.md)

Get started with Runique. Installation, dependencies, and first steps.

**Topics covered:**

* Prerequisites
* Installation steps
* Project setup
* First application

👉 **Go to**: [Installation Guide](01-installation.md)

---

### 2️⃣ [Architecture](02-architecture.md)

Understand Runique’s internal architecture.

**Topics covered:**

* Project structure
* Component overview
* Design patterns
* How it works

👉 **Go to**: [Architecture Guide](02-architecture.md)

---

### 3️⃣ [Configuration](03-configuration.md)

Configure your Runique application.

**Topics covered:**

* Server configuration
* Database setup
* Environment variables
* Security settings

👉 **Go to**: [Configuration Guide](03-configuration.md)

---

### 4️⃣ [Routing](04-routing.md)

URL routing and request handling.

**Topics covered:**

* URL patterns
* Route definition
* Request handlers
* URL parameters

👉 **Go to**: [Routing Guide](04-routing.md)

---

### 5️⃣ [Forms](05-forms.md)

Creating and managing forms.

**Topics covered:**

* Prisme extractor
* Manual declaration via `RuniqueForm`
* Model/schema-based declaration (AST) and automatic form generation
* Field types (FieldBuilder)
* Validation and persistence
* Template rendering

👉 **Go to**: [Forms Guide](05-forms.md)

---

### 6️⃣ [Templates](06-templates.md)

Working with Tera templates.

**Topics covered:**

* Django-like tags (`{% static %}`, `{% form.xxx %}`, `{% link %}`, `{% csrf %}`, `{% messages %}`, `{% csp_nonce %}`)
* Tera filters (`static`, `media`, `form`, `csrf_field`)
* Tera functions (`csrf()`, `nonce()`, `link()`)
* `context_update!` macro
* Template inheritance
* Auto-injected variables

👉 **Go to**: [Templates Guide](06-templates.md)

---

### 7️⃣ [ORM](07-orm.md)

Database operations with SeaORM.

**Topics covered:**

* Model definition
* Queries
* Relations
* Migrations

👉 **Go to**: [ORM Guide](07-orm.md)

---

### 8️⃣ [Middlewares](08-middleware.md)

Security and request middlewares.

**Topics covered:**

* Middleware stack with slot system
* CSRF protection (Double Submit Cookie)
* Content Security Policy (CSP) with nonce
* Allowed Hosts validation
* Security headers
* Session configuration
* Intelligent Builder vs Classic Builder

👉 **Go to**: [Middlewares Guide](08-middleware.md)

---

### 9️⃣ [Flash Messages](09-flash-messages.md)

User feedback and notifications.

**Topics covered:**

* Redirect macros: `success!`, `error!`, `info!`, `warning!`
* Immediate macro: `flash_now!`
* Rendering with `{% messages %}`
* Flash vs `flash_now` pattern
* Single-read consumption behavior

👉 **Go to**: [Flash Messages Guide](09-flash-messages.md)

---

### 🔟 [Examples](10-examples.md)

Complete code examples and projects.

**Topics covered:**

* Blog application
* Authentication
* File uploads
* REST API

👉 **Go to**: [Examples Guide](10-examples.md)

---

### 11️⃣ Admin

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

---

## 🎯 Quick Navigation

| Section  | File                                   | Topics                                     |
| -------- | -------------------------------------- | ------------------------------------------ |
| Setup    | [Installation](01-installation.md)     | Prerequisites, install, first steps        |
| Learning | [Architecture](02-architecture.md)     | Structure, design, internals               |
| Config   | [Configuration](03-configuration.md)   | Settings, environment, security            |
| Routes   | [Routing](04-routing.md)               | URL patterns, handlers, params             |
| Forms    | [Forms](05-forms.md)                   | Prisme, FieldBuilder, `#[form(...)]`       |
| Views    | [Templates](06-templates.md)           | Django-like tags, filters, Tera functions  |
| Data     | [ORM](07-orm.md)                       | Models, queries, `impl_objects!`           |
| Security | [Middlewares](08-middleware.md)        | Slots, CSRF, CSP, sessions                 |
| Feedback | [Flash Messages](09-flash-messages.md) | `success!`, `flash_now!`, `{% messages %}` |
| Code     | [Examples](10-examples.md)             | Complete projects                          |
| Admin    | [Admin](11-Admin.md)                   | Admin (beta)                               |

---

## 🚀 Where to Start?

1. **New to Runique?** → Start with [Installation](01-installation.md)
2. **Want to understand the internals?** → Read [Architecture](02-architecture.md)
3. **Ready to code?** → Check out [Examples](10-examples.md)
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

* 🇫🇷 **Français**
* 🇬🇧 **English** (you are here)

---

## 💡 Tips

* Each guide includes practical examples
* Follow sections in order for a structured learning path
* Refer to examples for real-world code
* Use your browser’s search feature for quick navigation

---

**Need help?** Check [Examples](https://github.com/seb-alliot/runique/blob/main/docs/en/10-examples.md) or review the relevant section.
