  Voici la version optimisée pour GitHub (Markdown pur sans HTML, compatible rendu GitHub) :

```markdown
# Runique — Django-inspired Rust Framework

![Rust](https://img.shields.io/badge/rust-1.85%2B-orange)
![Tests passing](https://img.shields.io/badge/tests-1731%2F1731%20passing-green)
![License](https://img.shields.io/badge/license-MIT-green)
![Version](https://img.shields.io/badge/version-1.1.54-blue)
[![Crates.io](https://img.shields.io/crates/v/runique)](https://crates.io/crates/runique)
[![Runique](https://img.shields.io/badge/Runique-brightgreen)](https://runique.io)

**Type-safe forms • Security middleware • Template rendering • ORM integration • Admin workflow**

🌍 **English** | [Français](https://runique.io/readme/fr)

---

## 📋 Overview

Runique is a web framework built on Axum, focused on developer productivity with Django-like ergonomics while maintaining Rust's type safety.

> **Current state:** Active development. The framework source of truth is the `runique` crate.  
> `demo-app` serves as the validation/testing application for framework behavior.

### Repository Structure

| Path | Description |
|------|-------------|
| `runique/` | Framework crate (main product) |
| `demo-app/` | Test/validation app for framework development |
| `docs/` | EN/FR documentation |

**Workspace version:** `1.1.54`

---

## ✨ Core Capabilities

| Feature | Description |
|---------|-------------|
| **Forms** | Type-safe form system with extractors, validators, and renderers |
| **Routing** | Macros and URL helpers for clean route definitions |
| **Templates** | Tera integration with context helpers |
| **Security** | CSRF, CSP, allowed hosts, sanitization, auth/session middleware |
| **Database** | SeaORM integration with migration tooling |
| **Flash** | Session-based flash message system |
| **Admin** | Beta admin interface (`admin!` macro + daemon-generated CRUD) |

---

## 🚀 Quick Start

### Installation

```bash
git clone https://github.com/seb-alliot/runique
cd runique
cargo build --workspace
cargo test --workspace
```

📖 [Detailed installation guide](https://runique.io/docs/en/installation)

### Minimal Example

```rust,no_run
use runique::prelude::*;

#[tokio::main]
async fn main() {
    let config = RuniqueConfig::from_env();
    let app = RuniqueApp::builder(config).build().await.unwrap();
    app.run().await.unwrap();
}
```

---

## 🛠️ CLI Commands

| Command | Description |
|---------|-------------|
| `runique new <name>` | Create a new project |
| `runique start [--main src/main.rs] [--admin src/admin.rs]` | Start the application |
| `runique create-superuser` | Create an admin user |
| `runique makemigrations --entities src/entities --migrations migration/src [--force false]` | Generate migrations |
| `runique migration up\|down\|status --migrations migration/src` | Manage migrations |

> ⚠️ **Migration Warning**
> 
> `makemigrations` generates SeaORM tables while preserving chronological order.
> **Always use SeaORM CLI** to apply migrations. Other commands may cause desynchronization.

---

## 🔧 Admin Beta

The admin daemon (`start` command):

1. Detects `.with_admin(...)` in `src/main.rs`
2. Starts watcher mode when enabled
3. Exits with hint if admin is not configured

**Workflow:**
- Parse `admin!` declarations in `src/admin.rs`
- Generate code under `src/admins/`
- Auto-refresh on changes

**Current Limitations:**
- Mostly resource-level permissions
- Generated folder overwrite (`src/admins/`)
- Iterative hardening in progress

📖 [Admin documentation](https://runique.io/docs/en/admin)

---

## 🗄️ Database Backends

**Default features:** `orm`, `all-databases`

**Available backends:**
- `sqlite`
- `postgres` 
- `mysql`
- `mariadb`

---

## 📊 Test & Coverage Status

| Metric | Value |
|--------|-------|
| Tests | **1731/1731 passing** ✅ |
| Functions | 76.66% |
| Lines | 71.04% |
| Regions | 67.22% |

*Coverage snapshot: `2026-03-01`, package `runique`*

```bash
cargo llvm-cov --tests --package runique \
  --ignore-filename-regex "admin" \
  --summary-only
```

---

## 🔐 Sessions

`CleaningMemoryStore` provides automatic expired-session cleanup with memory protection:

| Watermark | Behavior |
|-----------|----------|
| **Low (128 MB)** | Background purge of expired anonymous sessions |
| **High (256 MB)** | Emergency purge + 503 refusal if exceeded |

**Key features:**
- `protect_session(&session, duration_secs)` — Mark anonymous session as untouchable
- `user_id` key — Auto-protects authenticated sessions

📖 [Sessions documentation](https://runique.io/docs/en/session)

---

## ⚙️ Configuration

Key environment variables (`.env`):

```env
# Session
RUNIQUE_SESSION_CLEANUP_SECS=60
RUNIQUE_SESSION_LOW_WATERMARK=134217728
RUNIQUE_SESSION_HIGH_WATERMARK=268435456

# Security
SECRET_KEY=your-secret-key

# Database
DATABASE_URL=sqlite://db.sqlite3
```

📖 [Full environment reference](https://runique.io/docs/en/env)

---

## 📚 Documentation

### Getting Started
- [Installation](https://runique.io/docs/en/installation)
- [Architecture](https://runique.io/docs/en/architecture)
- [Configuration](https://runique.io/docs/en/configuration)

### Core Concepts
- [Routing](https://runique.io/docs/en/routing)
- [Forms](https://runique.io/docs/en/formulaire)
- [Model/Schema](https://runique.io/docs/en/model)
- [Templates](https://runique.io/docs/en/template)
- [ORM](https://runique.io/docs/en/orm)

### Advanced
- [Middlewares](https://runique.io/docs/en/middleware)
- [Flash Messages](https://runique.io/docs/en/flash)
- [Admin Beta](https://runique.io/docs/en/admin)
- [Sessions](https://runique.io/docs/en/session)

### Reference
- [Examples](https://runique.io/docs/en/exemple)
- [Environment Variables](https://runique.io/docs/en/env)
- [Changelog](https://runique.io/changelog)
- [Runique vs Django](https://runique.io/docs/en/comparatif)

---

## 📌 Project Status

For the detailed, continuously updated state report:  
📄 [PROJECT_STATUS.md](https://github.com/seb-alliot/runique/blob/main/docs/en/PROJECT_STATUS.en.md)

---

## 🔗 Resources

| Resource | Link |
|----------|------|
| Crates.io | [crates.io/crates/runique](https://crates.io/crates/runique) |
| Website | [runique.io](https://runique.io) |
| Security Policy | [SECURITY.md](https://github.com/seb-alliot/runique/blob/main/SECURITY.md) |

---

## 📄 License

MIT — see [LICENSE](https://github.com/seb-alliot/runique/blob/main/LICENSE)
```

