
# Runique Framework — Project Status (English)

This document consolidates the actual state of the repository from the reference sources:

- `Cargo.toml` (workspace version)
- `README.md`
- `CHANGELOG.md`

---

## Snapshot (as of November 4, 2026)

- **Workspace version**: `4.0.0` (**unreleased** on crates.io — code + docs in tree)
- **derive_form**: `4.0.0`
- **runique_dsl**: `0.2.0`
- **License**: MIT
- **Branch**: `fix-bug`
- **Stack**: Axum 0.8.9 + SeaORM 2.0.4 + Tera 2.4 · Rust edition 2024 · Rust 1.94
- **Main work shipped**: `ModelAdmin` admin + documentation aligned to 4.0

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
- `ValidationForm<F>` (`forms/validation_form.rs`): `try_form(form, request) -> Result<ValidationForm<F>, F>`, type-level proof a handler validated before acting — additive, doesn't replace `RuniqueForm::is_valid()`; defaults to dispatching on `Method::is_safe()`, with overridable `register_dynamic_fields`/`allow_get`/`allow_post` hooks
- Integrated CSRF protection (masked token anti-BREACH, constant-time comparison); a CSRF failure now sets an explicit message (`csrf.invalid_or_missing`) instead of failing silently, and `js/csrf.js` refreshes the token right before every form submission
- Structured per-domain tracing (`RuniqueLog` tree) over the full pipeline (field, set_value, validate, finalize, render)
- All field types: Text, Numeric, Boolean, Choice, Radio, Checkbox, Date, Time, DateTime, Duration, File, Color, Slug, UUID, JSON, IP, Hidden, Honeypot
- `save()` / `delete()` exist only on the validated form (`ValidationForm`) — no persistence without prior validation

### Routing

- `urlpatterns!{}` with typed segments, separate GET/POST
- Named URL registry, `{% link "name" %}` tag

### Templates

- Tera engine + context helpers (`{% csrf %}`, `{% static %}`, `{% link %}`, `{% media %}`)
- Autoescape active on `.html`/`.xml`

### Admin panel (stable beta)

- Typed `ModelAdmin<Entity>` builder — no `admin!{}` macro, no `src/admins/` generation
- Declared in `src/admin.rs`, wired via `.with_admin(admin::config)`; plain `cargo run` is enough
- Generic CRUD: list, detail, create, edit, delete, grouped actions (`.action(column, value, label)`), grouped delete
- Typed SeaORM columns: `display`, `filter`, `search`, `sortable`, `ordering`
- Ranks (superuser, dirigeant, admin, lambda), per-account rights for admins, groups for lambdas, rights grid
- Admin routes declared in `urlpatterns!` (`admin "…" => …`, standalone page or detail replacement)
- `OWNER` on `ModelMeta` (from `model!` meta): ownership check for `can_update_own`/`can_delete_own`
- Admin action history (log, batch_id, old/new diff)
- Templates overridable per block (HTML, CSS or both) or per page, for every model or for one

### Security

- Masked CSRF (BREACH protection), session-bound token, `subtle::ct_eq` comparison
- `session.cycle_id()` on login — session fixation protection
- Rights per operation (create, read, read_own, update, update_own, delete, delete_own) and per column, decided by a single gate (`auth/gate`)
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
- Form sent from another site or a subdomain refused before the handler (`Sec-Fetch-Site`)
- Session content encrypted in the database (XChaCha20-Poly1305, key derived from `SECRET_KEY`)

### ORM / Migrations

- `model!{}` DSL → SeaORM entity + SQL migration + model form (`ModelForm`)
- `extend!{}` — framework table extension (e.g. `eihwaz_users`)
- `makemigrations` — plan → validate → **atomic commit/rollback** + snapshots; `DROP COLUMN` on removed columns (destructive guard)
- Supported backends: PostgreSQL, MariaDB, SQLite
- `Pk` alias: `i32` by default, `i64` (`big-pk`), or `Uuid` via `Uuid::now_v7()` (`pk-uuid`) — mutually exclusive features (`compile_error!`). Usable on any field (not just the PK), typically a foreign key, to stay automatically in sync with the referenced table's type
- `model!{}` — unified field syntax (the legacy `fields: { name: SqlType }` grammar is removed): single anonymous block, 43 semantic types, including `readonly`/`label` options
- Migration generator hardened across engines (2026-09-01): runtime backend guards for `CREATE TYPE`/`updated_at` triggers (instead of a choice frozen at generation time), Postgres identifier case-folding fixed on `ALTER TYPE`, invalid `TYPE`/`USING` ordering fixed, `modify_column` skipped on SQLite (sea-query panic there); foreign keys always inline in `CREATE TABLE`
- `makemigrations` creates the `migration/` crate when missing and wires the framework's tables in even without any model of the project's own; a column type change isn't migrated (warning after `--force`) — see [Migrations](/docs/en/installation/migrations)
- Multi-engine-portable search/filters (2026-09-02): `CAST(col AS TEXT)` is invalid on MySQL/MariaDB (requires `CHAR`) — fixed via `text_cast_type`/`text_eq`/`ilike` helpers that detect the backend at runtime (`db.get_database_backend()`). `search_cond!` now takes the `db` connection as its first argument across all 4 forms — see [Queries](/docs/en/orm/queries)

### I18n

- 9 languages (en, fr, de, es, it, pt, ja, zh, ru), `AtomicU8` storage, `RUNIQUE_LANG`

### Tracing & observability

- Per-domain `RuniqueLog` tree (forms, middleware, session, auth, admin, db, mailer, migration, templates, errors, builder), each leaf an `Option<LogLevel>`
- `TraceResult::trace` / `trace_or` — swallowed `Result` sites log their `file:line`; security-critical sites floor at `WARN` even when their category is off
- Outputs: colored stdout, rolling files (JSON/plain, non-blocking), custom `LogSink` (no `tracing` type exposed); `.external()` delegates the global subscriber to the host app
- `RUNIQUE_LOG_FILE` runtime override

### CLI

- `runique new` (project + `migration/` crate), `runique create-superuser`, `runique makemigrations`, `runique migration up | down | status` (no `sea-orm-cli` needed), `runique test` — no `runique start` (`cargo run` + `ModelAdmin`)

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
| 3.0.3 | `bulk_create`: a create-only right could update existing rows | High |
| 3.0.3 | Staff member able to take over a superuser's account (email change + password reset) | High |
| 3.0.3 | Open redirect through a tab, a non-ASCII `Location`, or the `Refresh` header | Medium |
| 3.0.2 | Stored XSS through a staged upload (path shown back, folder served without CSP) | High |
| 3.0.2 | Files written to disk before the CSRF check (multipart) | Medium |
| 3.0.2 | Open redirect through credentials in the URL (`site.com:x@evil.com`) | Medium |
| 3.0.2 | urlencoded/JSON bodies with no size limit | Medium |
| 3.0.2 | Admin reset-password possible from a non-account resource | Low |
| 3.0.2 | Admin: raw CSRF token accepted (BREACH masking bypassed) | Hardening |
| 4.0.0 | Forgot-password timing (faster response for unknown address) | Medium |
| 4.0.0 | Admin superuser refusal message too revealing | Low |
| 4.0.0 | `csrf.js` sent the CSRF token off-origin | High |
| 4.0.0 | `password` / `is_active` exposed on `extend!` forms | High |
| 4.0.0 | Debug template page: read outside `templates/` | High |
| 4.0.0 | Form global errors unescaped (XSS) | Medium |
| 4.0.0 | Session content stored in clear in the database | Medium |
| 4.0.0 | Reset leaving the other sessions of a custom store valid (#15) | Medium |
| 4.0.0 | Form from another site reaching a handler that doesn't check the token | Medium |
| 4.0.0 | A database delete error could leave a reusable session; session revocation outside the reset's transaction (faille.md #12) | Medium |
| 4.0.0 | Non-atomic reset token replacement: two valid tokens under concurrency (#13) | Medium |
| 4.0.0 | Hash with an unrecognized prefix: faster, hence measurable, check (#3) | Low |
| 4.0.0 | Duplicate CSRF token in a form body (#10, fixed part) | Low |
| 4.0.0 | Ambiguous string enum aliases read as another variant (#26) | Low |

---

## Admin — permissions state

- Ranks: superuser (everything, bypass included), dirigeant (everything), admin (its own grid, set by a dirigeant), lambda (union of its groups, never the framework's tables) ✅
- `*_own` rights enforced when `ModelMeta::OWNER` is set (via `model!` meta) ✅
- Account and rights read from the database on every request (immediate revocation) ✅
- Details: [Permissions](/docs/en/admin/permission)

---

## Fixes to apply / roadmap

Detailed source of truth: [ROADMAP.md](https://github.com/seb-alliot/runique/blob/main/ROADMAP.md).

### Done (checked October 29, 2026)

- **Admin filters and SQL injection**: allow-lists + bound values; built-in accounts ignore URL filters
- **Security regression tests**: session/CSRF rotation, per-operation checks, 3.0.2 fixes (mutation)
- **Admin rework**: `ModelAdmin<Entity>` shipped — no `admin!{}` / `src/admins/` / `runique start`
- **4.0 documentation**: fr/en guides, README, migration, SECURITY, demo-app, INDEX aligned on `ModelAdmin`

### Open — 4.0.0 (before or right after release)

- Mandatory delete action on `belongs_to` (`cascade` / `set_null` / `restrict`)
- Typed uploads (`StagedFile`, out of text values)

### Next work (fixed order)

1. **Security / maintenance `4.0.x` patches** — numbered queue below, **before** any feature minor.
2. **`4.x` minors** — only after that: M2M DSL → `makemigrations` → tracing → project languages → additive admin parity.

Detail and “finished” criteria: [ROADMAP.md](https://github.com/seb-alliot/runique/blob/main/ROADMAP.md).

### Open — 4.0.x patch (security / maintenance, in this order)

1. Cap `RateLimiter` / `LoginGuard` in-memory counters
2. Open-redirect test battery (PortSwigger)

→ Expected outcome: `4.0.1` (and later if needed). No minor feature in these patches.

### Open — 4.x minors (no breaking change, after the 4.0.x queue)

1. Many-to-many generated by the DSL + migration, then in the admin
2. `makemigrations` finished (rename+modify, constraint names from DB, NOT NULL backfill, per-engine identifier limits, type ALTER)
3. Tracing (file rotation, request id, visible i18n fallbacks)
4. Project-provided languages (trait + builder)
5. Additive Django-admin parity (fieldsets, inlines, bulk actions, exports…)

### Low priority

- **Coverage**: per-file breakdown in [couverture_test.md](https://github.com/seb-alliot/runique/blob/main/docs/couverture_test.md) (October 8: `migrate.rs` 79%, `engine/core.rs` 92%, `forms/fields/file.rs` 96%)

---

## References

- Repository: [github.com/seb-alliot/runique](https://github.com/seb-alliot/runique)
- Changelog: [CHANGELOG.md](https://github.com/seb-alliot/runique/blob/main/CHANGELOG.md)
- Roadmap: [ROADMAP.md](https://github.com/seb-alliot/runique/blob/main/ROADMAP.md)
- Documentation: [English](https://github.com/seb-alliot/runique/tree/main/docs/en) | [Français](https://github.com/seb-alliot/runique/tree/main/docs/fr)

---

**Last update**: November 2, 2026
**Global status**: ✅ Stable framework · 🟡 Mature admin beta (`ModelAdmin`) · 🔒 Security: DB reset tokens, timing-safe auth, 4.0 fixes in tree · 📖 4.0 docs aligned · 📦 crates.io 4.0.0 still unreleased · ➡️ Next: `4.0.x` security patch queue (then 4.x minors)
