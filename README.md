# Runique — the Django developer experience, in type-safe Rust

![Rust](https://img.shields.io/badge/rust-1.94%2B-orange)
![Tests passing](https://img.shields.io/badge/tests-2361%20passing-green)
![License](https://img.shields.io/badge/license-MIT-green)
![Version](https://img.shields.io/badge/version-2.2.0-blue)
[![Crates.io](https://img.shields.io/crates/v/runique)](https://crates.io/crates/runique)
[![Runique](https://img.shields.io/badge/Runique-brightgreen)](https://runique.io)

Declare a model once, and you get the database table, the migration, a type-safe form, and a full admin panel — no extra wiring. Runique brings Django's productivity to Rust without asking you to give up Rust's safety or performance. It's built on Axum, SeaORM and Tera, and it stays out of your way once the boilerplate is gone.

> **Status, plainly:** active development. The framework crate (`runique`) is the source of truth; `demo-app` is a real application exercised against it, not a toy. The admin panel is in **beta**. Nothing below is dressed up — see the [project status](https://github.com/seb-alliot/runique/blob/main/docs/en/PROJECT_STATUS.en.md) for the unfiltered version.

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

`model!` generates the SeaORM entity (`article::Model`) and its SQL migration (`runique makemigrations`) from the same declaration. Pair it with `#[form]` and you get a matching type-safe form, validated server-side and derivable straight from the schema. Register the resource in `admin!` and the CRUD panel is already there — list view, search, filters, permissions, all of it:

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

Rust already has fast, low-level building blocks for the web — what it doesn't have is a framework that gives you Django's day-to-day productivity out of the box. Wiring an ORM, a template engine, a forms layer and an admin together yourself is a project of its own before you've written a single feature. Runique does that wiring for you, following one set of conventions, so the time goes into your app instead of your plumbing — and you keep Rust's type safety and performance the whole way through.

| Django (Python) | Runique (Rust) |
|---|---|
| `models.py` | `model!` → SeaORM entity + migration |
| `forms.py` | `#[form]` type-safe forms |
| `admin.py` | `admin!` generated admin panel |
| `urls.py` | `urlpatterns!` routing macro |
| Django templates | Tera (auto-escaped) |
| QuerySet | SeaORM + `search!` query DSL |
| middleware | ordered middleware slots |

For the full picture: [Runique vs Django](https://runique.io/docs/en/comparatif).

---

## Security by default

None of this is bolted on afterward — it's part of the base you start from:

- CSRF protection compares tokens in constant time (`ct_eq`), so timing can't leak a match
- CSP ships with a per-response nonce, configurable through the builder
- Login is timing-safe (no user enumeration through response time) and passwords are hashed with Argon2
- Sessions persist to the database, with authenticated sessions protected first when memory runs low
- Password reset tokens live in the database hashed with SHA-256, single-use, and hardened against IDOR
- Output is sanitized (ammonia) on top of Tera's own auto-escaping, and host validation is enforced

[Security policy](https://runique.io/docs/en/middleware)

---

## Quick start

```bash
runique new myapp
cd myapp
cargo run            # your app is a normal Rust binary
```

> `runique start` regenerates the admin CRUD code from your `admin!`
> declarations, then launches `cargo run` itself — it's a one-shot generation
> step chained into the launch, not a background watcher (see
> [Admin (beta)](#admin-beta)). Plain `cargo run` skips regeneration.

A trimmed-down `main.rs` (the full version lives in `demo-app/src/main.rs`):

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

Routes go through the `urlpatterns!` macro and come out as a regular Axum `Router`:

```rust
pub fn routes() -> Router {
    urlpatterns! {
        "/"          => view!{ index },        name = "index",
        "/blog/{id}" => view!{ blog_detail },  name = "blog_detail",
    }
    .rate_limit("/login", "login", view!(login_user), 10, 60, vec![Method::POST])
}
```

For the full walkthrough: [Installation](https://runique.io/docs/en/installation)

---

## What's in this repository

- `runique/` — the framework crate itself, the product and the source of truth
- `demo-app/` — a real application built against the framework, used to validate it
- `docs/` — documentation in English and French

Workspace version (source of truth): **2.2.0**.

---

## CLI

`runique` gives you:

- `runique new <name>`
- `runique start [--main src/main.rs] [--admin src/admin.rs]` — regenerates admin code, then launches the app (one-shot, not a watcher)
- `runique create-superuser`
- `runique makemigrations --entities src/entities --migrations migration/src [--force false]`
- `runique migration up|down|status --migrations migration/src`

> ⚠️ **A note on rolling back migrations**
> `runique makemigrations` writes migrations that keep the chronological order
> of the migration system intact. If you ever need to roll one back, reach for
> the SeaORM CLI instead — it keeps the migration tracking table in sync with
> the schema's actual state. Mixing the two rollback paths can desynchronize
> that tracking.

---

## Admin (beta)

`runique start` does three things, in order, on a single thread:

1. parses your `admin!` declarations in `src/admin.rs`
2. generates the CRUD code under `src/admins/`
3. runs `cargo run --release`, blocking

It checks for `.with_admin(...)` in `src/main.rs` first and only generates/launches if that's present — otherwise it exits with a message telling you why. There's no continuous watching: run `runique start` again to regenerate after editing `src/admin.rs`.

It's still beta: permissions work mainly at the resource level for now, the generated `src/admins/` folder gets overwritten on each regeneration, and hardening is ongoing rather than finished.

Admin docs: [Admin](https://runique.io/docs/en/admin)

---

## Features and database backends

Enabled by default: `orm`, `all-databases`.

Pick a specific backend instead: `sqlite`, `postgres`, `mysql`, `mariadb`.

---

## Sessions

`CleaningMemoryStore` stands in for the default `MemoryStore`, adding automatic cleanup of expired sessions, a two-tier watermark (128 MB / 256 MB) to keep memory bounded, and priority for authenticated sessions — they're the last to be purged under pressure, and they survive restarts through a database fallback.

Full reference: [Sessions](https://runique.io/docs/en/session)

---

## Tests and coverage

- Tests reported: **2375 passing** (2 ignored)
- Coverage snapshot (`2026-09-02`, package `runique`, admin module included): functions **75.54%**, lines **73.69%**, regions **72.30%**

```bash
cargo llvm-cov --package runique --summary-only
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

- [Project status](https://github.com/seb-alliot/runique/blob/main/docs/en/PROJECT_STATUS.en.md) — kept up to date as the project moves
- [Changelog](https://runique.io/changelog)
- [Runique vs Django — feature comparison](https://runique.io/docs/en/comparatif)
- [Crates.io](https://crates.io/crates/runique)
- [Security policy](https://github.com/seb-alliot/runique/blob/main/SECURITY.md)

---

## License

MIT — see [LICENSE](https://github.com/seb-alliot/runique/blob/main/LICENSE)
