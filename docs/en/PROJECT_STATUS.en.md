
# Runique Framework — Project Status (English)

This document consolidates the actual state of the repository from the reference sources:

- `Cargo.toml` (workspace version)
- `README.md`
- `CHANGELOG.md`

---

## Snapshot (as of May 30, 2026)

- **Workspace version**: `2.1.12`
- **derive_form**: `2.1.7`
- **License**: MIT
- **Branch**: `main`
- **Stack**: Axum 0.8.7 + SeaORM 2.0.0-rc.38 + Tera 1.20.1 · Rust edition 2024 · Rust 1.88

---

## Workspace scope

- `runique` — main framework crate
- `derive_form` — proc-macro DSL (`derive_form!{}`, `extend!{}`)
- `demo-app` — framework validation application
- `demo-app/migration` — migrations linked to the demo app

---

## Features implemented

### Forms

- Typed form system: `#[form]`, `RuniqueForm`, validation, HTML rendering via Tera
- Integrated CSRF protection (masked token anti-BREACH, constant-time comparison)
- Structured `FormTracing` + `eprintln!` debug on the full pipeline (field, set_value, validate, finalize, render)
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
- Watcher via `runique start` (300ms debounce, initial generation on startup)
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
- `LoginGuard` — brute-force login protection
- `AntiBot` — configurable honeypot per scope
- HTML sanitization (ammonia), argon2/bcrypt/scrypt for passwords
- Secure redirects (open-redirect guard), `HttpOnly`/`SameSite=Strict`/`Secure` cookies

### ORM / Migrations

- `derive_form!{}` DSL → SeaORM entity + SQL migration + AdminForm
- `extend!{}` — framework table extension (e.g. `eihwaz_users`)
- `makemigrations` with destructive change detection + confirmation prompt
- Supported backends: PostgreSQL, MariaDB, SQLite

### I18n

- 8 languages (en, fr, de, es, it, pt, ja, zh), `AtomicU8` storage, `RUNIQUE_LANG`

### CLI

- `runique new`, `runique start`, `runique create-superuser`, `runique makemigrations`, `runique migration`

---

## Security — fix history

| Version | Issue | Severity |
|---------|-------|----------|
| 2.1.9 | SQL injection in admin list filters | High |
| 2.1.9 | Session fixation on login (missing cycle_id) | Medium |
| 2.1.9 | Admin write permission granularity (create/update/delete indistinct) | Medium |
| 2.1.9 | IDOR — can_update_own/can_delete_own not enforced | Low |

---

## Admin — permissions state

- `can_read`, `can_create`, `can_update`, `can_delete`: enforced per operation ✅
- `can_update_own`, `can_delete_own`: enforced when `own_field` is declared in `admin!{}` ✅
- Per-group permissions, memory cache with immediate revocation ✅

---

## Fixes to apply / roadmap

### High priority (v2.x)

- **SQLi filters via `configure {}`**: builtin resource filters go through a separate path, to be verified
- **Security non-regression tests**: add tests covering SQL whitelist, cycle_id, operation guards

### Medium priority (v3.0, breaking)

- **Sequential validation S1/S2/S3**: `req.form()` → S1 CSRF → S2 rules → S3 accessible data. Structural guarantee that CSRF + validation precede all data access (~115 call sites)
- **TypeState form**: variant `validate() -> Result<ValidForm<T>, T>`

### Low priority

- **In-memory reset tokens**: not persisted across restarts, inoperative in multi-instance
- **`makemigrations` DROP COLUMN**: removed columns not detected
- **Targeted coverage**: `migration/migrate.rs` (22%), `engine/core.rs` (50%), `forms/fields/file.rs` (61%)

---

## References

- Repository: [github.com/seb-alliot/runique](https://github.com/seb-alliot/runique)
- Changelog: [CHANGELOG.md](https://github.com/seb-alliot/runique/blob/main/CHANGELOG.md)
- Documentation: [English](https://github.com/seb-alliot/runique/tree/main/docs/en) | [Français](https://github.com/seb-alliot/runique/tree/main/docs/fr)

---

**Last update**: May 30, 2026
**Global status**: ✅ Stable framework · 🟡 Admin mature beta · 🔒 Security audit completed 2026-05-28 · 📖 Full public API documentation (docs.rs)
