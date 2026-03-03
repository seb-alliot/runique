# 📚 Runique Documentation - English

Complete documentation of the Runique web framework.

## 📖 Documentation Sections

### 1️⃣ [Installation](01-installation.md)
Getting started with Runique. Setup, dependencies, and first steps.

**Topics covered:**
- Prerequisites
- Installation steps
- Project setup
- First application

👉 **Go to** : [Installation Guide](01-installation.md)

---

### 2️⃣ [Architecture](02-architecture.md)
Understanding the internal architecture of Runique.

**Topics covered:**
- Project structure
- Component overview
- `Request` extractor, `Prisme` pipeline
- Core macros (`urlpatterns!`, `view!`, `context_update!`)

👉 **Go to** : [Architecture Guide](02-architecture.md)

---

### 3️⃣ [Configuration](03-configuration.md)
Configuring your Runique application.

**Topics covered:**
- Server configuration
- Database setup
- Environment variables
- Security settings
- `RuniqueApp` builder

👉 **Go to** : [Configuration Guide](03-configuration.md)

---

### 4️⃣ [Routing](04-routing.md)
URL routing and request handling.

**Topics covered:**
- `urlpatterns!` and `view!` macros
- URL parameters
- Named routes and `link()`
- Request handlers

👉 **Go to** : [Routing Guide](04-routing.md)

---

### 5️⃣ [Forms](05-forms.md)
Building and handling forms.

**Topics covered:**
- `Prisme<T>` extractor (Sentinel → Aegis → CSRF Gate → Construction)
- Manual declaration via `RuniqueForm` + `impl_form_access!()`
- Automatic declaration via `#[derive(DeriveModelForm)]`
- Field types (`TextField`, `NumericField`, `FileField`…)
- `PasswordConfig` — Argon2/Bcrypt/Scrypt hashing, `pre_hash_hook`
- Validation, typed helpers (`get_string()`, `get_i32()`…)
- Saving and rendering in templates

👉 **Go to** : [Forms Guide](05-forms.md)

---

### 6️⃣ [Templates](06-templates.md)
Working with Tera templates.

**Topics covered:**
- Django-like tags (`{% static %}`, `{% form.xxx %}`, `{% link %}`, `{% csrf %}`, `{% messages %}`, `{% csp_nonce %}`)
- Tera filters and functions
- `context_update!` macro
- Template inheritance
- Auto-injected variables

👉 **Go to** : [Templates Guide](06-templates.md)

---

### 7️⃣ [ORM](07-orm.md)
Database operations with SeaORM.

**Topics covered:**
- Model definition
- `impl_objects!` macro
- Queries, filters, relations
- Migrations

👉 **Go to** : [ORM Guide](07-orm.md)

---

### 8️⃣ [Middleware](08-middleware.md)
Security and request middleware.

**Topics covered:**
- Middleware stack with slot system
- CSRF protection (Double Submit Cookie)
- Content Security Policy (CSP) with nonce
- Allowed Hosts validation
- Security headers
- Session configuration
- Smart Builder vs classic Builder

👉 **Go to** : [Middleware Guide](08-middleware.md)

---

### 9️⃣ [Flash Messages](09-flash-messages.md)
User feedback and notifications.

**Topics covered:**
- Redirect macros: `success!`, `error!`, `info!`, `warning!`
- Immediate macro: `flash_now!`
- Display with `{% messages %}`
- Flash vs flash_now pattern
- Consume-once behavior

👉 **Go to** : [Flash Messages Guide](09-flash-messages.md)

---

### 🔟 [Examples](10-examples.md)
Complete code examples.

**Topics covered:**
- Full application structure
- Authentication (register, login)
- File upload
- Profile update

👉 **Go to** : [Examples Guide](10-examples.md)

---

### 1️⃣1️⃣ [Admin](11-Admin.md)
Automatically generated admin interface (beta).

**Topics covered:**
- Declarative `admin!` macro
- Automatic CRUD generation
- Generated routes, handlers and forms
- Type safety and code transparency

👉 **Go to** : [Admin Guide](11-Admin.md)

---

### 1️⃣2️⃣ [Models](12-model.md)
Data model definition.

**Topics covered:**
- SeaORM entity structure
- Schema definition
- Form integration

👉 **Go to** : [Models Guide](12-model.md)

---

## 🎯 Quick Navigation

| Section | File | Topics |
|---------|------|--------|
| Setup | [Installation](01-installation.md) | Prerequisites, install, first steps |
| Learn | [Architecture](02-architecture.md) | Structure, Request, Prisme, macros |
| Config | [Configuration](03-configuration.md) | Settings, environment, security |
| Routes | [Routing](04-routing.md) | urlpatterns!, view!, URL parameters |
| Forms | [Forms](05-forms.md) | Prisme, TextField, PasswordConfig, DeriveModelForm |
| Views | [Templates](06-templates.md) | Django-like tags, filters, context_update! |
| Data | [ORM](07-orm.md) | Models, queries, impl_objects! |
| Security | [Middleware](08-middleware.md) | Slots, CSRF, CSP, sessions |
| Feedback | [Flash Messages](09-flash-messages.md) | success!, flash_now!, {% messages %} |
| Code | [Examples](10-examples.md) | Complete projects, auth, upload |
| Admin | [Admin](11-Admin.md) | Admin beta, generated CRUD |
| Models | [Models](12-model.md) | Entities, schemas, forms |

---

## 🚀 Where to Start?

1. **New to Runique?** → Start with [Installation](01-installation.md)
2. **Want to understand?** → Read [Architecture](02-architecture.md)
3. **Ready to code?** → Check [Examples](10-examples.md)
4. **Need help?** → Search the relevant section above

---

## 🌍 Language

- 📖 **English** (you are here)
- 🇫🇷 **[Français](../fr/README.md)**

---

**Need help?** Check [Examples](https://github.com/seb-alliot/runique/blob/main/docs/en/10-examples.md) or review the relevant section.
