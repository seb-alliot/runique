
# 📊 Runique Framework — Project Status (English)

This document consolidates the actual state of the repository from the reference sources:

- `Cargo.toml` (workspace version)
- `README.md`
- `CHANGELOG.md`
- `ROADMAP.md`
- `couverture_test.md`

---

## 🧾 Snapshot (as of March 3, 2026)

- **Workspace version**: `1.1.42`
- **License**: MIT
- **Working branch**: `vue_admin`
- **Tests reported**: **1523/1523 passed**✅
- **Coverage (report from 2026-03-04)**:
  - Functions: **76.66%**
  - Lines: **71.04%**
  - Regions: **67.22%**
- **Coverage command**: `cargo llvm-cov --tests --package runique --ignore-filename-regex "admin" --summary-only`

---

## 🧱 Workspace scope

Member crates declared in the workspace:

- `runique` (main framework crate)
- `demo-app` (framework test application)
- `demo-app/migration` (migration related to the test app)

The status below concerns the **`runique`** crate (product source).
`demo-app` is only used to validate/test the framework during development.

---

## ✅ Features implemented

- **Forms**: typed form system, validation, rendering, integrated CSRF protection.
- **Routing**: macros and route registration.
- **Templates**: Tera engine + context helpers.
- **ORM / Migration**: SeaORM integration, `makemigrations`, => possible but avoided to prevent desynchronization with sea-orm -> `migration up/down/status`.
- **Security**: CSRF middleware, CSP, allowed hosts, sanitization, auth session.
- **Flash messages**: temporary session message system.
- **CLI `runique`**: `new`, `start`, `create-superuser` => password hashing via Argon2, reflection in progress for flexibility, `makemigrations`, `migration` => uses sea-orm CLI.
- **I18n (base)**: translation module `utils::trad::switch_lang` with `Lang` (FR/EN), embedded JSON dictionaries and message formatting.

### Exported modules (crate `runique`)

- `app`, `config`, `context`, `engine`, `flash`, `forms`, `macros`, `middleware`, `migration`, `admin`, `errors`, `utils`
- `db` is conditional on the **`orm`** feature.

### Legacy API compatibility

Compatibility aliases remain exposed (`config_runique`, `formulaire`, `middleware_runique`, etc.), making it easier to transition old projects.

---

## ⚙️ Cargo Features & Technical Base

- Default features: `orm` + `all-databases`
- DB backends available: `sqlite`, `postgres`, `mysql`, `mariadb`
- Main stack: Axum + Tower + Tokio + Tera + SeaORM (optional via feature)
- Password security: `argon2`, `bcrypt`, `scrypt`, `password-hash`

---

## 🧭 Admin View Status (beta)

The admin view is **operational in beta** on a declarative model + code generation:

- Declaration via `admin!` macro in `src/admin.rs`
- Macro parsing (`syn`) + generation of `src/admins/`
- Watcher via `runique start` for automatic regeneration
- CRUD routes/handlers generated (basic functionality)

### Known limitations (assumed at this stage)

- Permissions mainly global per resource
- Little fine-grained control per operation
- `src/admins/` is regenerated (manual changes are overwritten)
- **CSRF**: reliable protection in the form flow (`Prisme` / `csrf_gate`), but middleware still permissive for some mutating endpoints outside the form flow.

### Practical workflow state

- `runique start` detects `.with_admin(...)` in `src/main.rs`
- If admin enabled: launches watcher + generation
- If admin not enabled: explicit message, no daemon launched

---

## 🧪 Quality & Tests

### Current state

- **Pass rate**: 100% (1523/1523 passing)
- **Functional coverage**: 76.66%
- **Roadmap target before publication**: 85%+
- **Note**: reported coverage ignores files matching `admin` (see command above)

### Identified weak areas

Critical files still low or at 0% according to `couverture_test.md`:

- `engine/core.rs`
- `errors/error.rs`
- `migration/utils/parser_seaorm.rs`
- `forms/fields/datetime.rs`
- `forms/fields/file.rs`
- several modules depending on a complete HTTP stack (`context/template.rs`, extractors, etc.)

### Areas with strong progress (per `couverture_test.md`)

- `db/config.rs`: ~22% → **93%**
- `migration/makemigrations.rs`: ~22% → **76%**
- `migration/migrate.rs`: 0% → **60%**

---

## 📌 Consolidated Roadmap

### Done

- Complete and stabilized migration pipeline
- Refactored/stabilized form system
- Improved coverage (baseline rising)

### In progress

- Admin view beta (ergonomics, permissions, workflow security)
- Simplification and hardening of some middleware points (notably CSP)
- Application i18n integration: FR/EN base already present, global connection (config/runtime) still in progress

### To do

- End-to-end configurable i18n (centralized runtime selection)
- More advanced error tracing
- Raise coverage to 85%+
- Prepare crates.io publication (docs + target coverage)

---

## 🆕 Recent changes (Unreleased)

Visible in `CHANGELOG.md`:

- complete migration pipeline announced and stabilized
- broad support for column types + FK/index/nullable/unique
- E2E DB tests on Postgres/MariaDB/SQLite
- fixes on `runique start` and password rendering in admin

---

## 🚀 Maturity level

- **Core framework**: stable and production-ready on the main base
- **Admin**: usable beta, still in iteration phase
- **External publication**: still in preparation (mainly coverage + detailed docs)

---

## ⚠️ Gaps / inconsistencies to watch

- **Version**: `1.1.42`
- **Admin status**: the admin technical doc describes a functional base, but the roadmap still lists it as in progress.
- **Coverage**: the global percentage is improving but still below the publication target.
- **CSRF**: no systematic flaw if the framework is used as intended; the sensitive point is not respecting the usage contract on mutating routes outside the Prisme flow.

---

## 🛠️ Fixes to apply

### High priority (security / robustness)

- **Admin permissions**: actually apply the permissions declared per resource in the generated CRUD handlers (not just `is_staff` / `is_superuser`).
- **CSRF (potential track)**: stabilize security by **enforcing the usage contract** (`http method -> prisme -> handler`) on mutating methods, with unique body reading in Prisme and no re-reading in middleware.
- **CSP**: reduce permissive default directives (`unsafe-inline` / `unsafe-eval`) and harmonize the nonce strategy.
- **Runtime safety**: replace `panic!/unwrap/expect` on runtime paths with propagated errors (`Result` + typed errors).

### Medium priority (technical consistency)

- **I18n end-to-end**: connect the existing i18n base (FR/EN + JSON) to a centralized runtime selection (config/session/request).
- **Admin daemon**: clarify and stabilize the lifecycle (start, stop, error reporting).
- **Environment variables**: standardize naming and error messages (e.g., allowed hosts).
- **Admin generation**: unify generation paths/contracts (`src/admins/` vs other documented paths).

### Low priority (continuous quality)

- **Targeted coverage**: strengthen still-weak areas (`engine/core.rs`, `errors/error.rs`, `migration/utils/parser_seaorm.rs`, `forms/fields/datetime.rs`, `forms/fields/file.rs`).
- **Doctests/docs publication**: convert `ignore/no_run` examples to executable ones and align crates.io docs.
- **Compatibility debt**: plan the gradual deprecation of legacy aliases while maintaining backward compatibility.

---

## 🔗 References

- Repository : [github.com/seb-alliot/runique](https://github.com/seb-alliot/runique)
- Status coverage : (https://github.com/seb-alliot/runique/blob/main/docs/couverture_test.md)
- Changelog : [Changelog](https://github.com/seb-alliot/runique/blob/main/CHANGELOG.md)
- Roadmap : [Roadmap](https://github.com/seb-alliot/runique/blob/main/ROADMAP.md)
- Documentation : [English](https://github.com/seb-alliot/runique/tree/main/docs/en) et [French](https://github.com/seb-alliot/runique/tree/main/docs/fr)

---

**Last update**: March 4, 2026
**Global status**: ✅ Stable core, 🟡 Admin beta evolving
Silent errors may occur, please report them if you find any.
Thank you!
