
# Runique Framework — Project Status (English)

This document consolidates the actual state of the repository from the reference sources:

- `Cargo.toml` (workspace version)
- `README.md`
- `CHANGELOG.md`

---

## Snapshot (as of October 9, 2026)

- **Workspace version**: `3.0.3`
- **derive_form**: `3.0.0`
- **runique_dsl**: `0.1.0`
- **License**: MIT
- **Branch**: `main`
- **Stack**: Axum 0.8.9 + SeaORM 2.0.4 + Tera 2.4 · Rust edition 2024 · Rust 1.94

---

## Workspace scope

- `runique` — main framework crate
- `runique_dsl` — model DSL parser, shared by the macros and the CLI
- `derive_form` — procedural macros (`model!{}`, `extend!{}`, `#[form]`)
- `demo-app` — framework validation application
- `demo-app/migration` — migrations linked to the demo app

---

## Features implemented

### Forms

- Typed form system: `#[form]`, `RuniqueForm`, validation, HTML rendering via Tera
- `FormField::validate`/`finalize` are async — genuinely non-blocking I/O (file upload via `tokio::fs`, password hashing) under `is_valid()`'s already-async surface
- `ValidationForm<F>` (`forms/validation_form.rs`): `try_new(form, request) -> Result<ValidationForm<F>, F>`, type-level proof a handler validated before acting — additive, doesn't replace `RuniqueForm::is_valid()`; defaults to dispatching on `Method::is_safe()`, with overridable `register_dynamic_fields`/`allow_get`/`allow_post` hooks
- Integrated CSRF protection (masked token anti-BREACH, constant-time comparison); a CSRF failure now sets an explicit message (`csrf.invalid_or_missing`) instead of failing silently, and `js/csrf.js` refreshes the token right before every form submission
- Structured per-domain tracing (`RuniqueLog` tree) over the full pipeline (field, set_value, validate, finalize, render)
- All field types: Text, Numeric, Boolean, Choice, Radio, Checkbox, Date, Time, DateTime, Duration, File, Color, Slug, UUID, JSON, IP, Hidden, Honeypot
- `save()` / `save_as()` guard: returns `Err` if `is_valid()` was not called or returned `false` — prevents any persistence without prior validation

### Routing

- `urlpatterns!{}` with typed segments, separate GET/POST
- Named URL registry, `{% url %}` Tera helper

### Templates

- Tera engine + context helpers (`{% csrf %}`, `{% static %}`, `{% url %}`, `{% media %}`)
- Autoescape active on `.html`/`.xml`

### Admin panel (stable beta)

- Declarative `admin!{}` DSL → generation of `src/admins/` by the daemon
- `runique start`: generates `src/admins/` once, then runs the app (no watcher)
- Full generated CRUD: list, detail, create, edit, delete, bulk edit, bulk delete, group actions
- `list_display`, `list_filter` (paginated distinct values), `search!` on all columns
- `group_action`: booleans and exact enum values, multi-entry merge same field
- `bulk_create`: upsert by value (split by comma), auto-generation `edit_form_builder`
- `m2m`: many-to-many relations via junction table
- `own_field`: ownership check for `can_update_own`/`can_delete_own`
- Admin action history (log, batch_id, old/new diff)
- Resource-overridable templates

### Security

- Masked CSRF (BREACH protection), session-bound token, `subtle::ct_eq` comparison
- `session.cycle_id()` on login — session fixation protection
- Granular admin permissions per operation (`can_create`, `can_update`, `can_delete`, `can_update_own`, `can_delete_own`)
- Statically generated SQL column whitelist — SQL injection protection in admin filters/sort
- CSP builder with nonce, HSTS, host validation
- Global + per-HTTP-method `RateLimiter` (`rate_limit_get()`, `rate_limit_post()`, etc.)
- `LoginGuard` — brute-force login protection (opt-in for the admin: `with_login_guard`)
- Account read from the database on every request (`request.user`): the session holds only the id, a deactivation or a removed right applies to the next request; activation enforced by a `CHECK` constraint (`is_active` ⇒ `activated_at`)
- Uploads (3.0.2): no file written before a valid CSRF token, staging folders never served, script-free CSP on `/media`
- Request body size capped (2 MB by default, `RUNIQUE_MAX_UPLOAD_MB` to raise it)
- `AntiBot` — configurable honeypot per scope
- HTML sanitization (ammonia), argon2/bcrypt/scrypt for passwords
- Secure redirects (open-redirect guard), `HttpOnly`/`SameSite=Strict`/`Secure` cookies
- Timing-safe login (dummy-hash verify — no user enumeration)
- DB-persisted password-reset tokens: SHA-256-hashed, single-use, IDOR-hardened (mutation keyed on the token-bound user id)
- CSRF checked via both the `csrf_token` body field and the `X-CSRF-Token` header (2026-09-02) — needed for JSON/`fetch` clients that can't add a hidden form field; safe by construction, a cross-origin `fetch` setting this header triggers a CORS preflight Runique never allows by default (`CorsConfig` disabled, `any_origin() + allow_credentials(true)` is a build error)

### ORM / Migrations

- `model!{}` DSL → SeaORM entity + SQL migration + AdminForm
- `extend!{}` — framework table extension (e.g. `eihwaz_users`)
- `makemigrations` — plan → validate → **atomic commit/rollback** + snapshots; `DROP COLUMN` on removed columns (destructive guard)
- Supported backends: PostgreSQL, MariaDB, SQLite
- `Pk` alias: `i32` by default, `i64` (`big-pk`), or `Uuid` via `Uuid::now_v7()` (`pk-uuid`) — mutually exclusive features (`compile_error!`). Usable on any field (not just the PK), typically a foreign key, to stay automatically in sync with the referenced table's type
- `model!{}` — unified field syntax (the legacy `fields: { name: SqlType }` grammar is removed): single anonymous block, 43 semantic types, including `readonly`/`label` options
- Migration generator hardened across engines (2026-09-01): runtime backend guards for `CREATE TYPE`/`updated_at` triggers (instead of a choice frozen at generation time), Postgres identifier case-folding fixed on `ALTER TYPE`, invalid `TYPE`/`USING` ordering fixed, `modify_column` skipped on SQLite (sea-query panic there); foreign keys always inline in `CREATE TABLE`
- `makemigrations` now recognizes a `migration/` bootstrapped via `sea-orm-cli migrate init` (`lib.rs` canonically reformatted, `todo!()` placeholder removed), and wires the framework's tables in even without any model of the project's own (3.0.2) — see [Migrations](/docs/en/installation/migrations)
- Multi-engine-portable search/filters (2026-09-02): `CAST(col AS TEXT)` is invalid on MySQL/MariaDB (requires `CHAR`) — fixed via `text_cast_type`/`text_eq`/`ilike` helpers that detect the backend at runtime (`db.get_database_backend()`). `search_cond!` now takes the `db` connection as its first argument across all 4 forms — see [Queries](/docs/en/orm/queries)

### I18n

- 9 languages (en, fr, de, es, it, pt, ja, zh, ru), `AtomicU8` storage, `RUNIQUE_LANG`

### Tracing & observability

- Per-domain `RuniqueLog` tree (forms, middleware, session, auth, admin, db, mailer, migration, templates, errors, builder), each leaf an `Option<LogLevel>`
- `TraceResult::trace` / `trace_or` — swallowed `Result` sites log their `file:line`; security-critical sites floor at `WARN` even when their category is off
- Outputs: colored stdout, rolling files (JSON/plain, non-blocking), custom `LogSink` (no `tracing` type exposed); `.external()` delegates the global subscriber to the host app
- `RUNIQUE_LOG_FILE` runtime override

### CLI

- `runique new`, `runique start`, `runique create-superuser`, `runique makemigrations`, `runique migration up` (rollback and status: `sea-orm-cli`), `runique test`

---

## Security — fix history

| Version | Issue | Severity |
|---------|-------|----------|
| 2.1.9 | SQL injection in admin list filters | High |
| 2.1.9 | Session fixation on login (missing cycle_id) | Medium |
| 2.1.9 | Admin write permission granularity (create/update/delete indistinct) | Medium |
| 2.1.9 | IDOR — can_update_own/can_delete_own not enforced | Low |
| 2.1.15 | User enumeration via login timing attack | Medium |
| 2.1.15 | Missing authorization on admin reset-password action | Medium |
| 2.1.17 | Reset tokens: in-memory → DB (hashed, single-use, IDOR-hardened) | Hardening |
| 3.0.3 | Staff member able to take over a superuser's account (email change + password reset) | High |
| 3.0.3 | Open redirect through a tab, a non-ASCII `Location`, or the `Refresh` header | Medium |
| 3.0.2 | Stored XSS through a staged upload (path shown back, folder served without CSP) | High |
| 3.0.2 | Files written to disk before the CSRF check (multipart) | Medium |
| 3.0.2 | Open redirect through credentials in the URL (`site.com:x@evil.com`) | Medium |
| 3.0.2 | urlencoded/JSON bodies with no size limit | Medium |
| 3.0.2 | Admin reset-password possible from a non-account resource | Low |
| 3.0.2 | Admin: raw CSRF token accepted (BREACH masking bypassed) | Hardening |

---

## Admin — permissions state

- `can_read`, `can_create`, `can_update`, `can_delete`: enforced per operation ✅
- `can_update_own`, `can_delete_own`: enforced when `own_field` is declared in `admin!{}` ✅
- Per-group permissions, read from the database on every request (immediate revocation) ✅

---

## Fixes to apply / roadmap

### Done (checked October 9, 2026)

- **Admin filters and SQL injection**: generated resources only accept the columns of their allowlist, with bound values; the built-in `users` resource ignores filters from the URL
- **Security regression tests**: session and CSRF token rotation at login, per-operation checks, column allowlist, and the 3.0.2 fixes (each one verified by mutation)

### High priority (3.x)

- **Admin rework** around a typed `ModelAdmin<Entity>` builder, without generated code — see [the draft](https://github.com/seb-alliot/runique/blob/main/ebauche-model-admin.md)

### Low priority

- **Coverage**: per-file breakdown in [couverture_test.md](https://github.com/seb-alliot/runique/blob/main/docs/couverture_test.md) (October 8: `migrate.rs` 79%, `engine/core.rs` 92%, `forms/fields/file.rs` 96%)

---

## References

- Repository: [github.com/seb-alliot/runique](https://github.com/seb-alliot/runique)
- Changelog: [CHANGELOG.md](https://github.com/seb-alliot/runique/blob/main/CHANGELOG.md)
- Documentation: [English](https://github.com/seb-alliot/runique/tree/main/docs/en) | [Français](https://github.com/seb-alliot/runique/tree/main/docs/fr)

---

**Last update**: October 9, 2026
**Global status**: ✅ Stable framework · 🟡 Admin mature beta · 🔒 Security: reset tokens DB-hardened, timing-safe auth · 📖 Full public API documentation (docs.rs)
