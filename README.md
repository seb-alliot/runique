# Runique — the Django developer experience, in type-safe Rust

![Rust](https://img.shields.io/badge/rust-1.94%2B-orange)
![Tests passing](https://img.shields.io/badge/tests-2011%2B%20passing-green)
![License](https://img.shields.io/badge/license-MIT-green)
![Version](https://img.shields.io/badge/version-2.1.17-blue)
[![Crates.io](https://img.shields.io/crates/v/runique)](https://crates.io/crates/runique)
[![Runique](https://img.shields.io/badge/Runique-brightgreen)](https://runique.io)

**Declare a model once — get the database table, the migration, a type-safe form *and* a full admin panel.** Runique is a batteries-included web framework that brings Django's productivity to Rust, without giving up Rust's safety and performance. Built on Axum, SeaORM and Tera.

> **Status — honest:** active development. The framework crate (`runique`) is the source of truth; `demo-app` is a real validation app exercised against it. The admin is in **beta**. Nothing below is overstated — see [Project status](https://github.com/seb-alliot/runique/blob/main/docs/en/PROJECT_STATUS.en.md).

🌍 **Languages**: English | [Français](https://runique.io/readme/fr)

---

## Declarative macros, not boilerplate

```rust
model! {
    Article,
    table: "articles",
    pk: id => Pk,
    enums: { Status: [Draft="Draft", Published="Published"], },
    {
        title:  text [required],
        slug:   text [unique],
        body:   richtext [required],
        status: choice [enum(Status), default: "Draft"],
        views:  int [default: 0],
    }
}
```

`model!` generates the **SeaORM entity** (`article::Model`) and its **SQL migration** (`runique makemigrations`). A matching **type-safe form** is declared with `#[form]` (server-side validated, derivable from the schema). Register the resource and you get a **full CRUD admin** — list display, search, filters, permissions:

```rust
admin! {
    article: article::Model => ArticleForm {
        title: "Articles",
        list_display: [["title", "Title"], ["status", "Status"], ["views", "Views"]],
        search_fields: ["title", "body"],
        list_filter:   [["status", "Status", 5]],
    }
}
```

<!-- Add a real screenshot of the generated admin here — it sells the framework better than any paragraph: -->
<!-- ![Runique admin panel](docs/assets/admin.png) -->

---

## Why Runique

Rust has fast, low-level web building blocks — but no *batteries-included* framework with Django's productivity. Wiring an ORM, a template engine, a form layer and an admin together by hand is a project in itself. Runique integrates them, convention-driven, so you ship features instead of plumbing — while keeping type safety and performance.

| Django (Python) | Runique (Rust) |
|---|---|
| `models.py` | `model!` → SeaORM entity + migration |
| `forms.py` | `#[form]` type-safe forms |
| `admin.py` | `admin!` generated admin panel |
| `urls.py` | `urlpatterns!` routing macro |
| Django templates | Tera (auto-escaped) |
| QuerySet | SeaORM + `search!` query DSL |
| middleware | ordered middleware slots |

Full side-by-side: [Runique vs Django](https://runique.io/docs/en/comparatif).

---

## Security by default

Security ships on by construction, not as an add-on:

- **CSRF** protection with constant-time token comparison (`ct_eq`)
- **CSP** with per-response nonces, configurable via the builder
- **Auth**: timing-safe login (no user enumeration), Argon2 password hashing
- **Sessions** persisted with priority protection for authenticated users
- **Password reset**: DB-persisted, SHA-256-hashed, single-use, IDOR-hardened tokens
- **Output sanitization** (ammonia) + Tera auto-escaping, allowed-host validation

[Security policy](https://runique.io/docs/en/middleware)

---

## Quick start

```bash
runique new myapp
cd myapp
cargo run            # your app is a normal Rust binary
```

> `runique start` is **not** how you launch the app — it's the admin code
> generator: it watches your `admin!` declarations and regenerates the CRUD
> code (see [Admin (beta)](#admin-beta)).

A trimmed `main.rs` (full version in `demo-app/src/main.rs`):

```rust,no_run
use runique::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = RuniqueConfig::from_env();
    let db = DatabaseConfig::from_env()?.build().connect().await?;

    RuniqueApp::builder(config)
        .routes(url::routes())
        .with_database(db)
        .statics()
        .build()
        .await
        .map_err(|e| -> Box<dyn std::error::Error> { Box::new(e) })?
        .run()
        .await?;
    Ok(())
}
```

Routes are declared with the `urlpatterns!` macro and return an Axum `Router`:

```rust
pub fn routes() -> Router {
    urlpatterns! {
        "/"          => view!{ index },        name = "index",
        "/blog/{id}" => view!{ blog_detail },  name = "blog_detail",
    }
    .rate_limit("/login", "login", view!(login_user), 10, 60, vec![Method::POST])
}
```

Detailed guide: [Installation](https://runique.io/docs/en/installation)

---

## What this repository contains

- `runique/` → framework crate (the product, source of truth)
- `demo-app/` → validation app exercised against the framework
- `docs/` → EN/FR documentation

Workspace version (source of truth): **2.1.17**.

---

## CLI

`runique` provides:

- `runique new <name>`
- `runique start [--main src/main.rs] [--admin src/admin.rs]` — admin code generator/watcher, **not** the app launcher (run the app with `cargo run`)
- `runique create-superuser`
- `runique makemigrations --entities src/entities --migrations migration/src [--force false]`
- `runique migration up|down|status --migrations migration/src`

> ⚠️ **Warning — rolling back migrations**
> `runique makemigrations` generates migrations while preserving the chronological
> order of the migration system. When you need to **roll a migration back**, prefer
> the SeaORM CLI: it keeps the migration tracking table synchronized with the actual
> schema state. Mixing rollback tooling can desynchronize migration tracking.

---

## Admin (beta)

The admin daemon, run via `runique start`:

1. parses your `admin!` declarations in `src/admin.rs`
2. generates CRUD code under `src/admins/`
3. refreshes on changes in watcher mode

It checks whether `.with_admin(...)` exists in `src/main.rs` and starts the watcher only when enabled, otherwise exits with an explicit hint.

Current beta limits: mostly resource-level permissions, the generated `src/admins/` folder is overwritten, and iterative hardening is ongoing.

Admin docs: [Admin](https://runique.io/docs/en/admin)

---

## Features and database backends

Default features: `orm`, `all-databases`.

Selectable backends: `sqlite`, `postgres`, `mysql`, `mariadb`.

---

## Sessions

`CleaningMemoryStore` replaces the default `MemoryStore` with automatic expired-session cleanup, a two-tier watermark system (128 MB / 256 MB), and priority protection for authenticated sessions (purged last, survive restarts via a DB fallback).

Full reference: [Sessions](https://runique.io/docs/en/session)

---

## Tests and coverage

- Reported tests: **2011+ passing**
- Coverage snapshot (`2026-05-24`, package `runique`): functions **78.32%**, lines **76.05%**, regions **73.93%**

```bash
cargo llvm-cov --tests --package runique --ignore-filename-regex "admin" --summary-only
```

Full per-file breakdown: [docs/couverture_test.md](docs/couverture_test.md)

---

## Documentation

- [Installation](https://runique.io/docs/en/installation)
- [Architecture](https://runique.io/docs/en/architecture)
- [Configuration](https://runique.io/docs/en/configuration)
- [Routing](https://runique.io/docs/en/routing)
- [Forms](https://runique.io/docs/en/formulaire)
- [Model/Schema](https://runique.io/docs/en/model)
- [Templates](https://runique.io/docs/en/template)
- [ORM](https://runique.io/docs/en/orm)
- [Middlewares](https://runique.io/docs/en/middleware)
- [Flash Messages](https://runique.io/docs/en/flash)
- [Examples](https://runique.io/docs/en/exemple)
- [Admin beta](https://runique.io/docs/en/admin)
- [Sessions](https://runique.io/docs/en/session)
- [Environment variables](https://runique.io/docs/en/env)

---

## Project status & resources

- [Project status](https://github.com/seb-alliot/runique/blob/main/docs/en/PROJECT_STATUS.en.md) — continuously updated state report
- [Changelog](https://runique.io/changelog)
- [Runique vs Django — feature comparison](https://runique.io/docs/en/comparatif)
- [Crates.io](https://crates.io/crates/runique)
- [Security policy](https://github.com/seb-alliot/runique/blob/main/SECURITY.md)

---

## License

MIT — see [LICENSE](https://github.com/seb-alliot/runique/blob/main/LICENSE)
