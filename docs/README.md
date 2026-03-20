# Runique — Django-inspired Rust Framework

![Rust](https://img.shields.io/badge/rust-1.85%2B-orange)
![Tests passing](https://img.shields.io/badge/tests-1731%2F1731%20passing-green)
![License](https://img.shields.io/badge/license-MIT-green)
![Version](https://img.shields.io/badge/version-1.1.50-blue)
![Crates.io](https://img.shields.io/crates/v/runique)
![Runique](https://runique-production.up.railway.app/)

Runique is a web framework built on Axum, focused on type-safe forms, security middleware, template rendering, ORM integration, and a code-generated admin workflow.

> Current state: active development. The framework source of truth is the `runique` crate.
> `demo-app` is used as a validation/testing application for framework behavior.

🌍 **Languages**: [English](https://github.com/seb-alliot/runique/blob/main/README.md) | [Français](https://github.com/seb-alliot/runique/blob/main/README.fr.md)

---

## What this repository contains

- `runique/` → framework crate (main product)
- `demo-app/` → test/validation app for framework development
- `docs/` → EN/FR documentation

Workspace version (source of truth): **1.1.50**.

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

Detailed guide: [docs/en/01-installation.md](https://github.com/seb-alliot/runique/blob/main/docs/en/installation/01-installation.md)

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

```

## Admin beta status (bêta)

Admin daemon behavior in `start`:

- checks whether `.with_admin(...)` exists in `src/main.rs`
- starts the admin watcher when enabled
- otherwise exits with an explicit hint

---

Admin resources are declared in `src/admin.rs` using `admin!`.

The workflow:

1. parse `admin!` declarations
2. generate admin code under `src/admins/`
3. refresh on changes with watcher mode

Current beta limits:

- mostly resource-level permissions
- generated folder overwrite (`src/admins/`)
- iterative hardening still in progress

Admin docs: [docs/en/11-Admin.md](https://github.com/seb-alliot/runique/blob/main/docs/en/admin/11-Admin.md)

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

  - Reported tests: **1731/1731 passing**
  - Coverage snapshot (`2026-03-01`, package `runique`):
  - Functions: **76.66%**
  - Lines: **71.04%**
  - Regions: **67.22%**

Coverage command used:

```bash
cargo llvm-cov --tests --package runique --ignore-filename-regex "admin" --summary-only
```

See: [couverture_test.md](https://github.com/seb-alliot/runique/blob/main/docs/couverture_test.md)

---

## Sessions

`CleaningMemoryStore` replaces the default `MemoryStore` with automatic expired-session cleanup, a two-tier watermark system (128 MB / 256 MB), and priority-based protection for authenticated and high-value anonymous sessions (shopping carts, multi-step forms).

- Low watermark: background purge of expired anonymous sessions
- High watermark: synchronous emergency purge + 503 refusal if still exceeded
- `protect_session(&session, duration_secs)` — marks an anonymous session as untouchable until a given timestamp
- `user_id` key — automatically protects authenticated sessions

Full reference: [docs/en/14-sessions.md](https://github.com/seb-alliot/runique/blob/main/docs/en/session/14-sessions.md)

---

## Environment variables

All behavior is configurable via `.env`. Key variables:

```env
SECRET_KEY=your-secret-key
ALLOWED_HOSTS=localhost,example.com
DATABASE_URL=sqlite://db.sqlite3
```

Full reference: [docs/en/15-env.md](https://github.com/seb-alliot/runique/blob/main/docs/en/env/15-env.md)

---

## Documentation map

### English

- [Installation](https://github.com/seb-alliot/runique/blob/main/docs/en/installation/01-installation.md)
- [Architecture](https://github.com/seb-alliot/runique/blob/main/docs/en/architecture/02-architecture.md)
- [Configuration](https://github.com/seb-alliot/runique/blob/main/docs/en/configuration/03-configuration.md)
- [Routage](https://github.com/seb-alliot/runique/blob/main/docs/en/routing/04-routing.md)
- [Formulaires](https://github.com/seb-alliot/runique/blob/main/docs/en/formulaire/05-forms.md)
- [Model/Schema](https://github.com/seb-alliot/runique/blob/main/docs/en/model/12-model.md)
- [Templates](https://github.com/seb-alliot/runique/blob/main/docs/en/template/06-templates.md)
- [ORM](https://github.com/seb-alliot/runique/blob/main/docs/en/orm/07-orm.md)
- [Middlewares](https://github.com/seb-alliot/runique/blob/main/docs/en/middleware/08-middleware.md)
- [Flash Messages](https://github.com/seb-alliot/runique/blob/main/docs/en/flash/09-flash-messages.md)
- [Exemples](https://github.com/seb-alliot/runique/blob/main/docs/en/exemple/10-examples.md)
- [Admin bêta](https://github.com/seb-alliot/runique/blob/main/docs/en/admin/11-Admin.md)
- [Sessions](https://github.com/seb-alliot/runique/blob/main/docs/en/session/14-sessions.md)
- [Environment variables](https://github.com/seb-alliot/runique/blob/main/docs/en/env/15-env.md)

### Français

- [Installation](https://github.com/seb-alliot/runique/blob/main/docs/fr/installation/01-installation.md)
- [Architecture](https://github.com/seb-alliot/runique/blob/main/docs/fr/architecture/02-architecture.md)
- [Configuration](https://github.com/seb-alliot/runique/blob/main/docs/fr/configuration/03-configuration.md)
- [Routage](https://github.com/seb-alliot/runique/blob/main/docs/fr/routing/04-routing.md)
- [Formulaires](https://github.com/seb-alliot/runique/blob/main/docs/fr/formulaire/05-forms.md)
- [Model/Schema](https://github.com/seb-alliot/runique/blob/main/docs/fr/model/12-model.md)
- [Templates](https://github.com/seb-alliot/runique/blob/main/docs/fr/template/06-templates.md)
- [ORM](https://github.com/seb-alliot/runique/blob/main/docs/fr/orm/07-orm.md)
- [Middlewares](https://github.com/seb-alliot/runique/blob/main/docs/fr/middleware/08-middleware.md)
- [Flash Messages](https://github.com/seb-alliot/runique/blob/main/docs/fr/flash/09-flash-messages.md)
- [Exemples](https://github.com/seb-alliot/runique/blob/main/docs/fr/exemple/10-examples.md)
- [Admin bêta](https://github.com/seb-alliot/runique/blob/main/docs/fr/admin/11-Admin.md)
- [Sessions](https://github.com/seb-alliot/runique/blob/main/docs/fr/session/14-sessions.md)
- [Variables d'environnement](https://github.com/seb-alliot/runique/blob/main/docs/fr/env/15-env.md)

---

## Project status

For the detailed, continuously updated state report, see [PROJECT_STATUS.md](https://github.com/seb-alliot/runique/blob/main/docs/en/PROJECT_STATUS.md).

---

## Resources

- [Project structure](https://github.com/seb-alliot/runique/blob/main/docs/en/PROJECT_STATUS.en.md)
- [Changelog](https://github.com/seb-alliot/runique/blob/main/CHANGELOG.md)
- [Documentation hub](https://github.com/seb-alliot/runique/blob/main/docs/en/README.md)
- [Security policy](https://github.com/seb-alliot/runique/blob/main/SECURITY.md)
- [Benchmark](https://github.com/seb-alliot/runique/blob/main/benchmark.md)
- [Runique vs Django — Feature Comparison](https://github.com/seb-alliot/runique/blob/main/docs/en/comparatif-runique-django.md)

---

## License

MIT — see [LICENSE](https://github.com/seb-alliot/runique/blob/main/LICENSE)
