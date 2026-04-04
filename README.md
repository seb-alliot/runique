# Runique

**A Django-inspired web framework for Rust.**
Type-safe forms, built-in admin panel, security middleware, and a CLI — built on Axum + SeaORM + Tera.

[![Version](https://img.shields.io/badge/version-1.1.54-blue)](https://crates.io/crates/runique)
[![Tests](https://img.shields.io/badge/tests-1833%2F1833%20passing-green)](https://github.com/seb-alliot/runique)
[![Coverage](https://img.shields.io/badge/coverage-76%25-yellowgreen)](https://github.com/seb-alliot/runique)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-green)](https://github.com/seb-alliot/runique/blob/main/LICENSE)
[![Crates.io](https://img.shields.io/crates/v/runique)](https://crates.io/crates/runique)

🌍 [Français](https://runique.io/readme/fr)

---

## Why Runique?

Rust web frameworks leave most of the work to you. Runique takes the opposite approach — bring the ergonomics of Django to Rust, without hiding the type system.

| | Runique | Axum alone |
|---|---|---|
| Forms (parse, validate, render) | ✅ `Prisme` + `derive_form` | Manual |
| Admin panel | ✅ Code-generated from `admin!` | Manual |
| Auth + session | ✅ Built-in | Manual |
| CSRF | ✅ Always on | Manual |
| CSP + allowed hosts | ✅ Builder API | Manual |
| Migration CLI | ✅ `runique makemigrations` | SeaORM CLI only |

---

## Quick start

```rust
use runique::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = RuniqueConfig::from_env();

    RuniqueApp::builder(config)
        .routes(routes())
        .with_database_config(DatabaseConfig::from_env()?.build())
        .statics()
        .middleware(|m| {
            m.with_csp(|c| c.with_header_security(true))
             .with_allowed_hosts(|h| h.enabled(!is_debug()).host("mysite.com"))
        })
        .with_admin(|a| a.site_title("Admin"))
        .build().await?
        .run().await?;

    Ok(())
}
```

```toml
# Cargo.toml
[dependencies]
runique = { version = "1.1.54", features = ["orm", "postgres"] }
```

```env
# .env
DATABASE_URL=postgres://user:pass@localhost/mydb
SECRET_KEY=change-me-in-production
DEBUG=false
```

---

## Features

### Forms — type-safe, no boilerplate

```rust
#[derive(Forms)]
struct LoginForm {
    #[field(min_length = 3)]
    username: CharField,
    password: PasswordField,
}

async fn login_view(
    mut request: Request,
    Prisme(form): Prisme<LoginForm>,
) -> AppResult<Response> {
    if form.is_valid() {
        auth_login(&request.session, &request.engine.db, user.id).await?;
        return Ok(redirect("/dashboard"));
    }
    render!(request, "login.html", { "form" => form })
}
```

### Admin — generated from declarations

```rust
// src/admin.rs
admin! {
    Article {
        list: [title, author, published_at],
        search: [title, author],
        filters: [published_at],
    }
}
```

`runique start` watches `admin.rs` and regenerates the CRUD code under `src/admins/` automatically.

### Security — sane defaults, configurable

- **CSRF** — always active, cannot be disabled
- **CSP** — disabled in dev, strict in prod by default, fully configurable
- **Allowed hosts** — closure-based, wildcard support
- **Rate limiting** — `RateLimiter::new().max_requests(100).retry_after(60)`
- **Login guard** — brute-force protection per account, not per IP

### Sessions

`CleaningMemoryStore` replaces the default Axum `MemoryStore`:
- Two-tier watermark (128 MB low / 256 MB high) with automatic cleanup
- Priority protection for authenticated sessions
- Optional DB persistence (`eihwaz_sessions` table)
- Exclusive login (one active device per user) via `.with_exclusive_login(true)`

---

## CLI

```bash
runique new <name>                                          # scaffold a new project
runique start [--main src/main.rs] [--admin src/admin.rs]  # dev server + admin watcher
runique create-superuser                                    # create admin user
runique makemigrations --entities src/entities \
                       --migrations migration/src          # generate migrations
runique migration up|down|status \
                  --migrations migration/src               # apply / inspect migrations
```

---

## Database backends

```toml
runique = { version = "1.1.54", features = ["orm", "postgres"] }
# or: "sqlite" | "mysql" | "mariadb" | "all-databases"
```

Primary key size (default `i32`, opt-in `i64`):

```toml
runique = { version = "1.1.54", features = ["orm", "postgres", "big-pk"] }
```

---

## Tests & coverage

```bash
cargo test --workspace
cargo llvm-cov --tests --package runique --ignore-filename-regex "admin" --summary-only
```

- **1833/1833** tests passing
- Line coverage: **~76%** (snapshot 2026-03-30)

---

## Documentation

| Section | |
|---|---|
| [Installation](https://runique.io/docs/en/installation) | Setup, prerequisites, first project |
| [Configuration](https://runique.io/docs/en/configuration) | Builder API, environment variables |
| [Routing](https://runique.io/docs/en/routing) | `urlpatterns!`, extractors, responses |
| [Forms](https://runique.io/docs/en/formulaire) | `derive_form`, `Prisme`, validators |
| [Templates](https://runique.io/docs/en/template) | Tera, filters, context helpers |
| [ORM](https://runique.io/docs/en/orm) | SeaORM integration, migrations |
| [Middlewares](https://runique.io/docs/en/middleware) | CSRF, CSP, hosts, rate limiting |
| [Auth & Sessions](https://runique.io/docs/en/auth) | Login, logout, session store |
| [Admin](https://runique.io/docs/en/admin) | `admin!` macro, permissions, generated CRUD |
| [Flash Messages](https://runique.io/docs/en/flash) | Handler + template macros |
| [Examples](https://runique.io/docs/en/exemple) | Minimal, forms, upload |
| [Environment Variables](https://runique.io/docs/en/env) | Full reference |

---

## Project status

Active development — API stabilizing toward 1.2.

See [PROJECT_STATUS.md](https://github.com/seb-alliot/runique/blob/main/docs/en/PROJECT_STATUS.en.md) for the detailed state report.

## Resources

- [Changelog](https://runique.io/changelog)
- [Runique vs Django](https://runique.io/docs/en/comparatif)
- [Crates.io](https://crates.io/crates/runique)
- [Security policy](https://github.com/seb-alliot/runique/blob/main/SECURITY.md)

---

MIT — © seb-alliot
