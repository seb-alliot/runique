
# 📊 Runique Framework — Project Status (English)

This document consolidates the actual state of the repository from the reference sources:

- `Cargo.toml` (workspace version)
- `README.md`
- `CHANGELOG.md`
- `ROADMAP.md`
- `couverture_test.md`

---

## 🧾 Snapshot (as of March 15, 2026)

- **Workspace version**: `1.1.54`
- **License**: MIT
- **Working branch**: `i18n` → merge into `main` for publication
- **Tests reported**: **~1,600 / ~1,600 passed** ✅
- **Coverage (report from 2026-03-15)**:
  - Functions: **82.83%**
  - Lines: **78.35%**
  - Regions: **75.38%**
- **Coverage command**: `cargo llvm-cov --tests --package runique --ignore-filename-regex "admin|bin/runique|runique_app" --summary-only`

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
- **I18n**: 8 languages (`en`, `fr`, `de`, `es`, `it`, `pt`, `ja`, `zh`), 14 sections, automatic fallback to `Lang::En`, stored via `AtomicU8`, configurable via `RUNIQUE_LANG`.
- **Enhanced security**: rate limiter (`RateLimiter`), login guard (`LoginGuard`), masked CSRF (BREACH protection), HSTS, CSP nonce, constant-time comparisons (`subtle`).

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

- **Pass rate**: 100% (~1,600/~1,600 passing)
- **Functional coverage**: 82.83%
- **Roadmap target before publication**: ~85%+
- **Note**: reported coverage ignores `admin`, `bin/runique`, `runique_app`

### Identified weak areas

Critical files still low according to `couverture_test.md`:

- `migration/migrate.rs` (22%) — depends on sea-orm CLI commands
- `engine/core.rs` (50%)
- `middleware/dev/cache.rs` (60%)
- `forms/fields/file.rs` (61%) — multipart upload
- `middleware/errors/error.rs` (60%)

### Areas with strong progress (session 2026-03-13 → 2026-03-15)

- `context/template.rs`: 0% → **80.95%**
- `middleware/security/csp.rs`: 66% → **95%**
- `errors/error.rs`: 38% → **77%**
- `context/request_extensions.rs`: 40% → **100%**

---

## 📌 Consolidated Roadmap

### Done

- Complete and stabilized migration pipeline
- Refactored/stabilized form system
- i18n complete: 8 languages, 14 sections, `AtomicU8`, `RUNIQUE_LANG`
- Security hardening: masked CSRF, CSP builder, HSTS, rate limiter, login guard
- Coverage significantly improved (76% → 82% functions)

### In progress

- Admin view beta (runtime permissions, pagination, `js:` in `admin!`)
- Raise coverage to 85%+

### To do

- More advanced error tracing
- Executable doctests/examples for crates.io
- Gradual deprecation of legacy aliases

---

## 🆕 Recent changes

See `CHANGELOG.md` for full details. Key points from `[1.1.54]`:

- CSP fully migrated to builder (env vars removed)
- Masked CSRF (BREACH protection), constant-time comparisons
- Rate limiter + Login guard in prelude
- i18n 8 languages delivered in `[1.1.46]`
- Test coverage: 82.83% functions

---

## 🚀 Maturity level

- **Core framework**: stable and production-ready on the main base
- **Admin**: usable beta, still in iteration phase
- **External publication**: still in preparation (mainly coverage + detailed docs)

---

## ⚠️ Gaps / inconsistencies to watch

- **Coverage**: 82.83% functions, 85% target not yet reached.
- **Admin permissions**: declared in `admin!{}` but not yet enforced at runtime in `admin_main`.
- **`migration/migrate.rs`**: 22% coverage — depends on sea-orm CLI, difficult to unit test.

---

## 🛠️ Fixes to apply

### High priority

- **Admin permissions**: actually enforce permissions declared per resource in generated CRUD handlers.
- **Runtime safety**: replace remaining `panic!/unwrap/expect` on runtime paths with propagated errors.

### Medium priority

- **Admin daemon**: clarify and stabilize the lifecycle (start, stop, error reporting).
- **Admin generation**: unify generation paths/contracts (`src/admins/`).

### Low priority

- **Targeted coverage**: `engine/core.rs`, `migration/migrate.rs`, `forms/fields/file.rs`, `middleware/dev/cache.rs`.
- **Doctests/docs publication**: convert `ignore/no_run` examples to executable ones.
- **Compatibility debt**: plan gradual deprecation of legacy aliases.

---

## 🔗 References

- Repository : [github.com/seb-alliot/runique](https://github.com/seb-alliot/runique)
- Status coverage : (https://github.com/seb-alliot/runique/blob/main/docs/couverture_test.md)
- Changelog : [Changelog](https://github.com/seb-alliot/runique/blob/main/CHANGELOG.md)
- Roadmap : [Roadmap](https://github.com/seb-alliot/runique/blob/main/ROADMAP.md)
- Documentation : [English](https://github.com/seb-alliot/runique/tree/main/docs/en) et [French](https://github.com/seb-alliot/runique/tree/main/docs/fr)

---

**Last update**: March 15, 2026
**Global status**: ✅ Stable core, 🟡 Admin beta evolving
Silent errors may occur, please report them if you find any.
Thank you!
