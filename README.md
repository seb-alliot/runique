# Runique — Django-inspired Rust Framework

![Rust](https://img.shields.io/badge/rust-1.75%2B-orange)
![Tests passing](https://img.shields.io/badge/tests-1523%2F1523%20passing-orange)
![License](https://img.shields.io/badge/license-MIT-green)
![Version](https://img.shields.io/badge/version-1.1.38-blue)
![Crates.io](https://img.shields.io/crates/v/runique)

Runique is a web framework built on Axum, focused on type-safe forms, security middleware, template rendering, ORM integration, and a code-generated admin workflow.

> Current state: active development. The framework source of truth is the `runique` crate.
> `demo-app` is used as a validation/testing application for framework behavior.

🌍 **Languages**: [English](README.md) | [Français](README.fr.md)

---

## What this repository contains

- `runique/` → framework crate (main product)
- `demo-app/` → test/validation app for framework development
- `docs/` → EN/FR documentation

Workspace version (source of truth): **1.1.38**.

---

## Core capabilities

- Type-safe form system (`forms`, extractors, validators, renderers)
- Routing macros and URL helpers
- Tera template integration and context helpers
- Security middleware (CSRF, CSP, allowed hosts, sanitization, auth/session)
- SeaORM integration + migration tooling
- Flash message system
- Admin beta (`admin!` macro + daemon-generated CRUD code)

Main public modules are exposed from `runique/src/lib.rs`.

---

## Installation

```bash
git clone https://github.com/seb-alliot/runique
cd runique
cargo build --workspace
cargo test --workspace
```

Detailed guide: [docs/en/01-installation.md](docs/en/01-installation.md)

---

## Quick usage

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

## CLI (actual commands)

`runique` provides:

- `runique new <name>`
- `runique start [--main src/main.rs] [--admin src/admin.rs]`
- `runique create-superuser`
- `runique makemigrations --entities src/entities --migrations migration/src [--force false]`
- `runique migration up|down|status --migrations migration/src`

```

```

> ⚠️ **Warning**
> The `makemigrations` command generates SeaORM tables while preserving the
> chronological order of the migration system.
> To ensure migration tracking remains consistent, only use the SeaORM CLI
> to apply or manage migrations.
> Using other commands may lead to migration desynchronization.

```

Admin daemon behavior in `start`:

- checks whether `.with_admin(...)` exists in `src/main.rs`
- starts the admin watcher when enabled
- otherwise exits with an explicit hint

---

## Admin beta status

Admin resources are declared in `src/admin.rs` using `admin!`.

The workflow:

1. parse `admin!` declarations
2. generate admin code under `src/admins/`
3. refresh on changes with watcher mode

Current beta limits:

- mostly resource-level permissions
- generated folder overwrite (`src/admins/`)
- iterative hardening still in progress

Admin docs: [docs/en/11-Admin.md](docs/en/11-Admin.md)

---

## Features and database backends

Default features:

- `orm`
- `all-databases`

Selectable backends:

- `sqlite`
- `postgres`
- `mysql`
- `mariadb`

---

## Test and coverage snapshot

  - Reported tests: **1523/1523 passing**
  - Coverage snapshot (`2026-03-01`, package `runique`):
  - Functions: **76.66%**
  - Lines: **71.04%**
  - Regions: **67.22%**

Coverage command used:

```bash
cargo llvm-cov --tests --package runique --ignore-filename-regex "admin" --summary-only
```

See: [couverture_test.md](couverture_test.md)

---

## Sessions

`CleaningMemoryStore` replaces the default `MemoryStore` with automatic expired-session cleanup, a two-tier watermark system (128 MB / 256 MB), and priority-based protection for authenticated and high-value anonymous sessions (shopping carts, multi-step forms).

- Low watermark: background purge of expired anonymous sessions
- High watermark: synchronous emergency purge + 503 refusal if still exceeded
- `protect_session(&session, duration_secs)` — marks an anonymous session as untouchable until a given timestamp
- `user_id` key — automatically protects authenticated sessions

Full reference: [docs/en/14-sessions.md](docs/en/14-sessions.md)

---

## Environment variables

All behavior is configurable via `.env`. Key variables:

```env
RUNIQUE_SESSION_CLEANUP_SECS=60
RUNIQUE_SESSION_LOW_WATERMARK=134217728
RUNIQUE_SESSION_HIGH_WATERMARK=268435456
SECRET_KEY=your-secret-key
ALLOWED_HOSTS=localhost,example.com
DATABASE_URL=sqlite://db.sqlite3
```

Full reference: [docs/en/15-env.md](docs/en/15-env.md)

---

## Documentation map

### English

- [Installation](docs/en/01-installation.md)
- [Architecture](docs/en/02-architecture.md)
- [Configuration](docs/en/03-configuration.md)
- [Routing](docs/en/04-routing.md)
- [Forms](docs/en/05-forms.md)
- [Model/Schema](docs/en/12-model.md)
- [Templates](docs/en/06-templates.md)
- [ORM](docs/en/07-orm.md)
- [Middleware](docs/en/08-middleware.md)
- [Flash Messages](docs/en/09-flash-messages.md)
- [Examples](docs/en/10-examples.md)
- [Admin beta](docs/en/11-Admin.md)
- [Sessions](docs/en/14-sessions.md)
- [Environment variables](docs/en/15-env.md)


### Français

- [Installation](docs/fr/01-installation.md)
- [Architecture](docs/fr/02-architecture.md)
- [Configuration](docs/fr/03-configuration.md)
- [Routage](docs/fr/04-routing.md)
- [Formulaires](docs/fr/05-forms.md)
- [Model/Schema](https://github.com/seb-alliot/runique/blob/main/docs/fr/12-model.md)
- [Templates](docs/fr/06-templates.md)
- [ORM](docs/fr/07-orm.md)
- [Middlewares](docs/fr/08-middleware.md)
- [Flash Messages](docs/fr/09-flash-messages.md)
- [Exemples](docs/fr/10-examples.md)
- [Admin bêta](docs/fr/11-Admin.md)
- [Sessions](docs/fr/14-sessions.md)
- [Variables d'environnement](docs/fr/15-env.md)

---

## Project status

For the detailed, continuously updated state report, see [PROJECT_STATUS.md](PROJECT_STATUS.md).

---

## Resources

- [Project structure](INDEX.md)
- [Changelog](CHANGELOG.md)
- [Documentation hub](docs/README.md)
- [Security policy](SECURITY.md)
- [Benchmark](benchmark.md)

---

## License

MIT — see [LICENCE](LICENCE)
