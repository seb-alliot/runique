# Project Structure Guide

Navigate the Runique Framework codebase.

**Version**: 3.0.0 — **Updated**: 2026-10-06

---

## Root Level

```
runique/
├── README.md                 # Main documentation (English)
├── INDEX.md                  # This file
├── CHANGELOG.md              # Version history & release notes
├── SECURITY.md               # Security policy & guidelines
├── LICENSE                   # MIT License
├── Cargo.toml                # Workspace configuration
├── Cargo.lock                # Dependency lock file
├── audit.toml                # Cargo audit configuration
│
├── docs/                     # EN/FR documentation (sections)
├── runique/                  # Main framework crate (+ runique/derive_form, runique/runique_dsl)
├── demo-app/                 # Test/validation application
└── target/                   # Build output (git-ignored)
```

---

## Framework Crate (`runique/src/`)

### Entry Point
- **`lib.rs`** — public API, prelude, module exports

### `admin/` — Admin Panel (beta)
- **`admin_main/`** — CRUD request handlers, split by concern: `mod.rs` (entry, `check_csrf`, dispatch), `gate.rs` (single gate for every write: who, which rows, which fields), `handle_inline.rs` (nested child resources), `handle_list.rs` (pagination, sort, filters), `handle_crud.rs` (detail, create, edit, delete), `handle_bulk.rs` (bulk_action, group_set, bulk_delete), `handle_password.rs` (reset, user-created email)
- **`builtin/`** — builtin resources: `user.rs`, `groupe.rs`, `droit.rs` (+ `mod.rs` → `builtin_resources()`)
- **`daemon/`** — code generator + file watcher (`generator.rs`, `parser.rs`, `watcher.rs`)
- **`helper/`** — `dyn_form.rs` (dynamic form dispatch), `resource_entry.rs` (`ResourceEntry`, `ListParams`, closures), `fk_resolve.rs`, `m2m.rs`, `sql_dialect.rs`, `template.rs`
- **`registry.rs`** — `AdminRegistry` (`HashMap<name, ResourceEntry>`)
- **`resource.rs`** — `admin!` DSL output / `AdminResource`, `DisplayConfig`, `ColumnFilter`
- **`history.rs`** — admin action history (audit log)
- **`table_admin/`** — `migrations_table.rs` (migration tracking table)
- **`config/`** — `config_admin.rs` (admin configuration struct)
- **`middleware/`** — admin-specific middleware
- **`router/`** — admin router builder
- **`trad/`** — admin-specific i18n

### `app/` — Application Lifecycle
- **`builder/`** — `RuniqueApp::builder(config)` / `RuniqueAppBuilder<S>` (`mod.rs`: once-only settings tracked in the type; `build.rs`)
- **`runique_app.rs`** — `RuniqueApp` struct, `.run()` entry point
- **`error_build.rs`** — build-time error types
- **`templates.rs`** — Tera engine initialization & template loader
- **`staging/`** — builder stages: `core`, `middleware_staging/`, `admin`, `static`, `csp_config`, `host_config`, `cors_config`, `permissions_policy_config`, `trusted_proxies_config`

### `auth/` — Authentication
- **`session.rs`** — `login`/`auth_login`/`logout`, `CurrentUser`, session keys
- **`permissions/`** — `groupe`, `groupes_droits`, `users_groupes` (SeaORM entities for the rights tables)
- **`guard.rs`** — `LoginGuard` brute-force protection
- **`password.rs`** — Argon2 hashing / verification helpers
- **`user.rs`** — user extractor
- **`user_trait.rs`** — `RuniqueUser` trait
- **`form.rs`** — login form

### `bin/`
- **`runique.rs`** — CLI: `new`, `start`, `makemigrations`, `migration up`, `create-superuser`, `test`

### `composant-bin/` — Scaffolding Templates
- **`code/`** — generated code templates (main.rs, forms.rs, url.rs, views.rs, users.rs…)
- `css/`, `image/`, `template/`, `readme/` — default assets for new projects

### `config/` — Configuration
- **`app.rs`** — `RuniqueConfig` struct
- **`server.rs`** — server settings (host, port, domain)
- **`security.rs`** — `SecurityConfig` from `.env` (`ENFORCE_HTTPS`, HSTS, ACME)
- **`static_files.rs`** — static file paths

### `context/` — Request Context
- **`request_extensions.rs`** — `RequestExtensions` (injects CSRF, user data into requests)
- **`template.rs`** — template context builder
- **`request/extractor.rs`** — `Request` extractor for handlers
- **`request/mod.rs`** — request module exports
- **`tera/form.rs`** — form rendering Tera functions
- **`tera/static_tera.rs`** — `static_url` filter
- **`tera/url.rs`** — `url` reverse routing filter

### `db/` — Database
- **`adb.rs`** — `ADb`, the framework's database handle (`ConnectionTrait` + `TransactionTrait`, wraps a test transaction with `test-utils`)
- **`config.rs`** — connection pool config, `DATABASE_URL` parsing
- **`engine.rs`**, **`runique_db.rs`**, **`builder.rs`** — engine detection, connection wrapper, `DatabaseConfig` builder

### `engine/` — Core Engine
- **`core.rs`** — `RuniqueEngine` (config, `db: ADb`, Tera, `close_user_sessions`, `extension::<T>()`)

### `errors/` — Error Types
- **`error.rs`** — `AppError`, `AppResult`, `IntoResponse` impl, `html_escape` helpers

### `flash/` — Flash Messages
- **`flash_struct.rs`** — `FlashMessage` data structure
- **`flash_manager.rs`** — session-backed message manager

### `forms/` — Form System
- **`base.rs`** — base form state
- **`form.rs`** — `Forms` struct, `RuniqueForm` trait, `impl_form_access!`
- **`validation_form.rs`** — `ValidationForm<F>` (validation proven by the type)
- **`form_data.rs`** — form data → model conversions (`FormDataError`)
- **`field.rs`** — `FormField` trait
- **`generic.rs`** — generic field implementation
- **`extractor.rs`** — Prisme pipeline (`prisme_pipeline`): CSRF check + body parsing, consumed via `request.form::<T>()`
- **`renderer.rs`** — HTML renderer for form fields
- **`validator.rs`** — validation logic
- **`model_form/`** — `ModelForm` (form bound to a SeaORM model)
- **`fields/`** — field types: `text`, `number`, `boolean`, `choice`, `datetime`, `file`, `special`, `hidden`
- **`options/`** — `length`, `bool_choice` option structs
- **`prisme/`** — validation pipeline: `aegis` (security), `sentinel`, `rules` (CSRF check inline dans `extractor::check_csrf`)

### `macros/` — Macros
- **`admin/macros_admin.rs`** — `admin!` declaration macro
- **`bdd/`** — `impl_objects!`, `search!` / `search_cond!` (+ internal `search_apply_op!`, `search_munch!`), `Objects` query builder (`objects.rs`, `query.rs`), list fields `HasLists` / `ListField` (`list.rs`)
- **`context/`** — `context!` / `context_update!` (`context_simplifier.rs`), flash macros `flash_now!` / `info!` / `success!` / `warning!` / `error!` (`flash.rs`), `impl_from_error!` (`impl_error.rs`)
- **`forms/`** — `impl_form_access!` (`impl_form.rs`), `define_enum_kind!` (`enum_kind.rs`), `delegate_to_kind!` (`kind.rs`)
- **`routeur/`** — `view!` macro (`get_post.rs`), `urlpatterns!` (`router.rs`), URL registry/reverse helpers (`register_url.rs`, `router_ext.rs`)
- **`template/`** — template context macros

### `middleware/` — Middleware
(authentication lives in the top-level `auth/` module since the refactor)
- **`config.rs`** — middleware configuration builder (slot ordering)
- **`dev/cache.rs`** — dev-mode cache control headers
- **`errors/error.rs`** — error handling middleware (500, 404, custom pages)
- **`security/allowed_hosts.rs`** — `HostPolicy` middleware (slot 15)
- **`security/anti_bot.rs`** — honeypot / anti-bot heuristics
- **`security/csp.rs`** — CSP header injection (slot 30), HTTPS redirect behind a proxy (slot 17)
- **`security/csrf.rs`** — CSRF middleware (slot 60, always active)
- **`security/open_redirect.rs`** — open-redirect guard for `next`/redirect params
- **`security/permissions_policy.rs`** — `Permissions-Policy` header (allow/deny per feature)
- **`security/rate_limit.rs`** — `RateLimiter` with 429 + `Retry-After`
- **`security/trusted_proxies.rs`** — trusted proxy / client-IP resolution (CIDR, private nets)
- **`session/cleaning_store.rs`** — `CleaningMemoryStore` (128MB/256MB watermarks)
- **`session/session_db.rs`** — DB-backed session store

### `migration/` — Migration Tooling
- **`mod.rs`** — public migration API
- **`schema/`**, **`column/`**, **`primary_key/`**, **`foreign_key/`**, **`index/`**, **`hooks/`** — schema DSL
- **`utils/`** — `diff`, `generators`, `parser_seaorm`, `parser_builder` (reads `model!{}` through `runique_dsl`), `parser_extend`, `paths`, `types`

### `cli/` — CLI implementations
- `makemigration.rs`, `migrate.rs` (`migration up`), `new_project.rs`, `start.rs`, `cli_admin.rs` (`create-superuser`), `test_runner.rs` (`runique test`)

### `utils/` — Utilities
- **`acme/`** — automatic TLS via Let's Encrypt (ACME HTTP-01), `acme` feature
- **`aliases/`** — `ADb`, `AEngine`, `JsonMap`, `StrMap`… type aliases
- **`config/env.rs`** — `.env` flags (`env_flag`, `debug_from`), `is_debug()`, `css_token()`
- **`config/pk.rs`** — `Pk` type alias (`i32` by default, `i64` with `big-pk`, `Uuid` with `pk-uuid`)
- **`config/runique_log/`** — `RuniqueLog` categories
- **`config/url_params.rs`** — URL query parameter helpers
- **`constante/`** — session/CSRF/admin key constants, error strings, template names
- **`crypto/`** — `csrf.rs` (`CsrfToken`, mask/unmask), `csp_nonce.rs`
- **`forms/`** — `parse_html`, `sanitizer`, `parse_boolean`
- **`init_error/init.rs`** — `init_logging()` — reads `DEBUG`/`RUST_LOG`
- **`mailer/`** — email sending utilities
- **`password/`** — Argon2 / Bcrypt / Scrypt hashing, `PasswordConfig`
- **`reset_token/`** — secure token generation for password resets
- **`resolve_ogimage/`** — Open Graph image resolution
- **`trad/`** — i18n (9 languages), `switch_lang`

---

## Tests (`runique/tests/`)

**2162 integration tests passing** (2 ignored: `migration up` against a live database — SQLx fails on a non-UTF-8 Windows code page; run them with `-- --ignored`). Whole workspace: 2727 passing, 110 ignored.

```
tests/
├── mod.rs                    # Root: declares all test modules
├── helpers/                  # Shared test infrastructure
│   ├── server.rs             # build_engine(), test server
│   ├── request.rs            # get(), post(), request helpers
│   ├── assert.rs             # assert_body_str(), assert_status()
│   ├── db.rs                 # fresh_db() (SQLite)
│   ├── db_isolation.rs       # per-test transaction isolation
│   ├── db_postgres.rs        # fresh_db_postgres()
│   └── db_mariadb.rs         # fresh_db_mariadb()
├── admin/                    # registry, form filter, renderer, URL registry
├── app/                      # robots.txt, runique_app
├── auth/                     # admin auth, current user, login form, middlewares, session
├── config/                   # app config, builder, router, security, server, static
├── context/                  # app error, request extensions, Tera context, URL function
├── db/                       # SQLite, Postgres, MariaDB config tests
├── errors/                   # error rendering
├── flash/                    # flash manager
├── formulaire/               # all form tests (aegis, fields, prisme, validator, renderer…)
├── macros/                   # context helper, register URL
├── middleware/               # CSRF, CSP, hosts, rate limit, login guard, session, auth
├── migration/                # column, diff, foreign key, generators, parser, schema, list fields, belongs_to…
├── test_builder/             # runique_test builder
└── utils/                    # constante, flash message, logging, password, sanitizer
```

Run coverage (excluding admin):
```bash
cargo llvm-cov --tests --package runique --ignore-filename-regex "admin" --summary-only
```

---

## Example Application (`demo-app/src/`)

```
demo-app/src/
├── main.rs                   # App entry point (RuniqueApp::builder)
├── prelude.rs                # Local re-exports
├── url.rs                    # Route declarations
├── views.rs                  # Top-level view handlers
├── admin.rs                  # admin! macro declarations
├── demo_toggle.rs            # Feature toggle helpers
├── admins/                   # Generated admin code (admin! output)
├── entities/                 # SeaORM models (30+ entities)
├── formulaire/               # Form structs (RuniqueForm impls)
├── backend/                  # Domain handlers (auth, blog, doc, pages…)
│   └── seeds/                # DB seed functions (demo, doc, cour, ia)
└── (migration/ at demo-app/migration/)
```

---

## Documentation (`docs/`)

```
docs/
├── en/                       # English docs (14 sections)
│   ├── admin/
│   ├── architecture/
│   ├── auth/
│   ├── configuration/
│   ├── env/
│   ├── exemple/
│   ├── flash/
│   ├── formulaire/
│   ├── middleware/
│   ├── model/
│   ├── orm/
│   ├── routing/
│   ├── session/
│   └── template/
└── fr/                       # French docs (same sections)
```

---

## Quick Navigation

### Framework development
1. Public API: `runique/src/lib.rs`
2. App builder: `runique/src/app/builder/mod.rs`
3. Forms: `runique/src/forms/form.rs`
4. Middleware: `runique/src/middleware/`
5. Tests: `runique/tests/mod.rs`

### Adding a feature to demo-app
1. Entity: `demo-app/src/entities/`
2. Form: `demo-app/src/formulaire/`
3. Handler: `demo-app/src/backend/`
4. Route: `demo-app/src/url.rs`
5. Admin: `demo-app/src/admin.rs`

### Running tests
```bash
cargo test --workspace                     # all tests
cargo test --package runique               # framework only
cargo test --tests                         # integration only (no inline)
```

---

## Cargo Features

| Feature          | Description                              |
|------------------|------------------------------------------|
| `orm`            | SeaORM integration (default)             |
| `sqlite`         | SQLite — pick exactly one engine          |
| `postgres`       | PostgreSQL                               |
| `mysql`          | MySQL                                    |
| `mariadb`        | MariaDB (alias of `mysql`)               |
| `all-databases`  | All three engines, explicit opt-in only (multi-engine tooling, docs.rs) |
| `big-pk`         | `Pk = i64` instead of `i32`          |
| `pk-uuid`        | `Pk = Uuid` (`Uuid::now_v7()`), mutually exclusive with `big-pk` |
| `acme`           | Automatic TLS via Let's Encrypt (ACME)   |
| `test-utils`     | `runique_test` builder, `Prisme::for_test`, `Forms::mark_validated` |
