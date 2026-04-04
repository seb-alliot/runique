# Project Structure Guide

Navigate the Runique Framework codebase.

**Version**: 1.1.54 — **Updated**: 2026-04-04

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
├── audit.ps1                 # Audit script (Windows)
│
├── docs/                     # EN/FR documentation (sections)
├── runique/                  # Main framework crate
├── demo-app/                 # Test/validation application
└── target/                   # Build output (git-ignored)
```

---

## Framework Crate (`runique/src/`)

### Entry Point
- **`lib.rs`** — public API, prelude, module exports

### `admin/` — Admin Panel (beta)
- **`admin_main.rs`** — admin request handler (login, list, create, edit, delete)
- **`cli_admin.rs`** — CLI entry point for admin daemon
- **`dyn_form.rs`** — dynamic form dispatch for admin resources
- **`registry.rs`** — resource registry (admin! macro output)
- **`resource.rs`** — `AdminResource` trait definition
- **`resource_entry.rs`** — entry type for registered resources
- **`roles.rs`** — role-based access for admin views
- **`template.rs`** — admin template rendering helpers
- **`config/`** — admin configuration struct
- **`daemon/`** — code generator + file watcher (`generator.rs`, `parser.rs`, `watcher.rs`)
- **`middleware/`** — admin-specific middleware
- **`permissions/`** — droit, groupe, groupes_droits, users_droits, users_groupes
- **`router/`** — admin router builder

### `app/` — Application Lifecycle
- **`builder.rs`** — `RuniqueApp::builder(config)` / `RuniqueAppBuilder::new(config)`
- **`runique_app.rs`** — `RuniqueApp` struct, `.run()` entry point
- **`error_build.rs`** — build-time error types
- **`templates.rs`** — Tera engine initialization & template loader
- **`staging/`** — builder stages: `core`, `middleware`, `admin`, `csp_config`, `host_config`, `static`

### `bin/`
- **`runique.rs`** — CLI: `new`, `start`, `makemigrations`, `migration`, `create-superuser`

### `composant-bin/` — Scaffolding Templates
- **`code/`** — generated code templates (main.rs, forms.rs, url.rs, views.rs, users.rs…)
- `css/`, `image/`, `template/`, `readme/` — default assets for new projects

### `config/` — Configuration
- **`app.rs`** — `RuniqueConfig` struct
- **`server.rs`** — server settings (host, port, domain)
- **`security.rs`** — security config (CSRF, CSP)
- **`static_files.rs`** — static file paths
- **`router.rs`** — route configuration

### `context/` — Request Context
- **`request_extensions.rs`** — `RequestExtensions` (injects CSRF, user data into requests)
- **`template.rs`** — template context builder
- **`request/extractor.rs`** — `Request` extractor for handlers
- **`request/mod.rs`** — request module exports
- **`tera/form.rs`** — form rendering Tera functions
- **`tera/static_tera.rs`** — `static_url` filter
- **`tera/url.rs`** — `url` reverse routing filter

### `db/` — Database
- **`config.rs`** — connection pool config, `DATABASE_URL` parsing
- **`mod.rs`** — `DatabaseConnection` re-export

### `engine/` — Core Engine
- **`core.rs`** — `Engine` struct (holds config, db, Tera engine)

### `errors/` — Error Types
- **`error.rs`** — `AppError`, `AppResult`, `IntoResponse` impl, `html_escape` helpers

### `flash/` — Flash Messages
- **`flash_struct.rs`** — `FlashMessage` data structure
- **`flash_manager.rs`** — session-backed message manager

### `forms/` — Form System
- **`base.rs`** — base form state
- **`form.rs`** — `Forms` struct, `RuniqueForm` trait, `impl_form_access!`
- **`field.rs`** — `FormField` trait
- **`generic.rs`** — generic field implementation
- **`extractor.rs`** — `Prisme<T>` extractor (parses + validates form body)
- **`renderer.rs`** — HTML renderer for form fields
- **`validator.rs`** — validation logic
- **`model_form/`** — `ModelForm` (form bound to a SeaORM model)
- **`fields/`** — field types: `text`, `number`, `boolean`, `choice`, `datetime`, `file`, `special`, `hidden`
- **`options/`** — `length`, `bool_choice` option structs
- **`prisme/`** — validation pipeline: `aegis` (security), `csrf_gate`, `sentinel`, `rules`

### `macros/` — Macros
- **`admin/macros_admin.rs`** — `admin!` declaration macro
- **`bdd/`** — `impl_objects!`, `filter!`, ORM query macros
- **`context/`** — `context_simplifier`, `flash!`, `helper`, `impl_error!`
- **`forms/`** — `impl_form!`, `enum_kind!`, `kind!`
- **`routeur/get_post.rs`** — `get_post!` macro
- **`template/`** — template context macros

### `middleware/` — Middleware
- **`auth/`**
  - `admin_auth.rs` — `AdminAuth` trait + `AdminLoginResult`
  - `auth_session.rs` — `auth_login()`, `auth_logout()`, session management
  - `default_auth.rs` — `DefaultAdminAuth<Entity>` implementation
  - `login_guard.rs` — `LoginGuard` brute-force protection
  - `permissions_cache.rs`— permission cache for admin roles
  - `reset.rs` — password reset token handling
  - `user.rs` — `CurrentUser` extractor
  - `user_trait.rs` — `RuniqueUser` trait
  - `form/login.rs` — admin login form
- **`config.rs`** — middleware configuration builder
- **`dev/cache.rs`** — dev-mode cache control headers
- **`errors/error.rs`** — error handling middleware (500, 404, custom pages)
- **`security/allowed_hosts.rs`** — `HostPolicy` middleware
- **`security/csp.rs`** — CSP header injection
- **`security/csrf.rs`** — CSRF middleware (slot 60, always active)
- **`security/rate_limit.rs`** — `RateLimiter` with 429 + `Retry-After`
- **`session/cleaning_store.rs`** — `CleaningMemoryStore` (128MB/256MB watermarks)
- **`session/session_db.rs`** — DB-backed session store
- **`session/session_parametre.rs`** — session config helpers

### `migration/` — Migration Tooling
- **`mod.rs`** — public migration API
- **`schema/`**, **`column/`**, **`primary_key/`**, **`foreign_key/`**, **`index/`**, **`relation/`**, **`hooks/`** — schema DSL
- **`utils/`** — `convertisseur`, `diff`, `generators`, `parser_seaorm`, `parser_builder`, `paths`, `types`

### `utils/` — Utilities
- **`aliases/`** — `AEngine`, `JsonMap`, `TResult` type aliases
- **`cli/`** — `makemigration`, `migrate`, `new_project`, `start` implementations
- **`config/lecture_env.rs`** — `.env` reader
- **`constante/`** — session/CSRF/admin key constants, error strings, template names
- **`env.rs`** — `RuniqueEnv` enum, `is_debug()`, `css_token()`
- **`forms/`** — `parse_html`, `sanitizer`
- **`init_error/init.rs`** — `init_logging()` — reads `DEBUG`/`RUST_LOG`
- **`mailer/`** — email sending utilities
- **`middleware/csrf.rs`** — `CsrfToken`, `mask_csrf_token()`, `unmask_csrf_token()`
- **`middleware/csp_nonce.rs`** — CSP nonce generation
- **`password/`** — Argon2 hashing helpers
- **`pk.rs`** — `UserId` type alias (`i32` / `i64` with `big-pk`)
- **`reset_token/`** — secure token generation for password resets
- **`resolve_ogimage/`** — Open Graph image resolution
- **`runique_log.rs`** — structured log helpers
- **`trad/`** — i18n utilities, `switch_lang`
- **`url_params.rs`** — URL query parameter helpers

---

## Tests (`runique/tests/`)

**1833 tests, 0 failed** (2 ignored: SQLx Windows UTF-8 issue)

```
tests/
├── mod.rs                    # Root: declares all test modules
├── helpers/                  # Shared test infrastructure
│   ├── server.rs             # build_engine(), test server
│   ├── request.rs            # get(), post(), request helpers
│   ├── assert.rs             # assert_body_str(), assert_status()
│   ├── db.rs                 # fresh_db() (SQLite)
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
├── migration/                # column, diff, foreign key, generators, parser, schema…
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
2. App builder: `runique/src/app/builder.rs`
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
| `all-databases`  | All DB backends (default)                |
| `sqlite`         | SQLite only                              |
| `postgres`       | PostgreSQL only                          |
| `mysql`          | MySQL only                               |
| `mariadb`        | MariaDB only                             |
| `big-pk`         | `UserId = i64` instead of `i32`          |
