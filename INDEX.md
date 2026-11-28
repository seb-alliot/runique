# Project Structure Guide

Navigate the Runique Framework codebase.

**Version**: 4.0.0 — **Updated**: 2026-11-12

---

## Documents

Every document exists in English and in French.

| Document | English | Français |
|---|---|---|
| Presentation | [README.md](README.md) | [README.fr.md](README.fr.md) |
| Changelog | [CHANGELOG.md](docs/en/CHANGELOG.md) | [CHANGELOG.fr.md](docs/fr/CHANGELOG.fr.md) |
| Upgrading from 3.0 to 4.0 | [MIGRATION-4.0.md](docs/en/MIGRATION-4.0.md) | [MIGRATION-4.0.fr.md](docs/fr/MIGRATION-4.0.fr.md) |
| Roadmap | [ROADMAP.md](docs/en/ROADMAP.md) | [ROADMAP.fr.md](docs/fr/ROADMAP.fr.md) |
| Project status | [PROJECT_STATUS.en.md](docs/en/PROJECT_STATUS.en.md) | [PROJECT_STATUS.md](docs/fr/PROJECT_STATUS.md) |
| Known issues | [KNOWN_ISSUES.md](docs/en/KNOWN_ISSUES.md) | [KNOWN_ISSUES.fr.md](docs/fr/KNOWN_ISSUES.fr.md) |
| Compatibility | [COMPATIBILITY.en.md](docs/en/COMPATIBILITY.en.md) | [COMPATIBILITY.md](docs/fr/COMPATIBILITY.md) |
| Running the tests | [TESTING.md](docs/en/TESTING.md) | [TESTING.fr.md](docs/fr/TESTING.fr.md) |
| Use of AI | [AI_USAGE.md](docs/en/AI_USAGE.md) | [AI_USAGE.fr.md](docs/fr/AI_USAGE.fr.md) |
| Security policy | [SECURITY.md](SECURITY.md) | [SECURITY.fr.md](SECURITY.fr.md) |
| Contributing | [CONTRIBUTING.md](CONTRIBUTING.md) | [CONTRIBUTING.fr.md](CONTRIBUTING.fr.md) |
| User documentation | [docs/en/](docs/en/README.md) | [docs/fr/](docs/fr/README.fr.md) |

`README`, `SECURITY` and `CONTRIBUTING` stay at the root: GitHub only shows them from there.

## Root Level

```
runique/
├── README.md, SECURITY.md, CONTRIBUTING.md   # + their `.fr.md`
├── INDEX.md                  # This file
├── LICENSE                   # MIT License
├── Cargo.toml                # Workspace configuration
├── Cargo.lock                # Dependency lock file
├── audit.toml                # Cargo audit configuration
│
├── docs/                     # EN/FR documentation: the documents above, then one folder per section
├── runique/                  # The framework crate, and beside it runique_cli, runique_dsl, derive_form, runique_i18n, runique_atomic
├── demo-app/                 # Test/validation application
└── target/                   # Build output (git-ignored)
```

---

## Framework Crate (`runique/src/`)

### Entry Point
- **`lib.rs`** — public API, prelude, module exports

### `admin/` — Admin Panel (beta)
- **`builder/`** — typed `ModelAdmin` + `AdminSite`: `model_admin.rs`, `list.rs`, `form.rs`, `site.rs`, `registered.rs`, bulk/confirm helpers
- **`router/`** — CRUD handlers and routing: `routes.rs`, `context.rs`, `history.rs`, `crud/`, `account/`, `path/`, `permission/`
- **`template/`** — page/block overrides (`page.rs`, `block.rs`, `assemble.rs`, `stylesheet.rs`)
- **`config/`** — `settings.rs` (admin configuration struct)
- **`middleware/`** — admin-specific middleware
- **`translation/`** — admin-specific i18n

### `acme/` — TLS certificates (`acme` feature)
- **`mod.rs`** — `obtain_certificate` (Let's Encrypt, ACME HTTP-01), `ChallengeStore`
- **`renewal.rs`** — when a certificate is renewed, where it is stored
- **`serve.rs`** — `RuniqueApp::run_with_acme`: port 80 for the challenge, port 443 for the application

### `app/` — Application Lifecycle
- **`builder/`** — `RuniqueApp::builder(config)` / `RuniqueAppBuilder<S>` (`mod.rs`: once-only settings tracked in the type; `build.rs`: the order of the steps, each in a file beside it — `start.rs`, `checks.rs`, `templates.rs`, `routes.rs`, `static_files.rs`)
- **`runique_app.rs`** — `RuniqueApp` struct, `.run()` entry point
- **`staging/`** — builder stages, one file each: `core.rs`, `middleware.rs` (its settings are in `middleware/config/`), `applicator.rs` (the slots, `SLOT_*`), `admin.rs`, `statics.rs`

### `auth/` — Authentication
- **`session.rs`** — `login`/`auth_login`/`logout`, `CurrentUser`, session keys
- **`permissions/`** — `groupe`, `groupes_droits`, `users_groupes` (SeaORM entities for the rights tables)
- **`guard.rs`** — `LoginGuard` brute-force protection
- **`password.rs`** — Argon2 hashing / verification helpers
- **`user.rs`** — user extractor
- **`user_trait.rs`** — `RuniqueUser` trait
- **`form.rs`** — login form
- **`create_superuser.rs`** — `runique create-superuser`: interactive wizard, hashing, insert

### `bin/`
- **`runique.rs`** — CLI: `new`, `makemigrations`, `migration up`, `create-superuser`, `test`

### `new_project/` — `runique new`
- **`mod.rs`** — scaffolds a project: layout, bundled files, `SECRET_KEY`, its `migration` crate
- **`starter/`** — templates embedded at compile time (`include_str!`/`include_bytes!`): `code/` (main.rs, forms, url.rs, views.rs…), `config/`, `css/`, `image/`, `template/`, `readme/`

### `config/` — Configuration
- **`app.rs`** — `RuniqueConfig` struct
- **`server.rs`** — server settings (host, port, domain)
- **`security.rs`** — `SecurityConfig` from `.env` (`ENFORCE_HTTPS`, HSTS, ACME)
- **`static_files.rs`** — static file paths

### `http/` — What a handler receives
- **`request.rs`** — `Request`: the extractor handlers take, and its rendering helpers
- **`context.rs`** — `RuniqueContext` (engine, session, CSRF, flash)
- **`extension.rs`** — `RequestExtensions` (injects the engine, which carries Tera and the config, CSRF, CSP nonce, current user)

### `templates/` — Template engine (Tera)
- **`loader.rs`** — Tera engine initialization & template loader (`TemplateLoader`, `preprocess`)
- **`form.rs`** — form rendering Tera functions
- **`static_tera.rs`** — `static_url` filter, `| markdown`
- **`url.rs`** — `url` reverse routing function
- **`contrib.rs`** — the filters and functions Tera 1 shipped by itself

### `db/` — Database
- **`adb.rs`** — `ADb`, the framework's database handle (`ConnectionTrait` + `TransactionTrait`, wraps a test transaction with `test-utils`)
- **`config.rs`** — connection pool config, `DATABASE_URL` parsing
- **`engine.rs`**, **`runique_db.rs`**, **`builder.rs`** — engine detection, connection wrapper, `DatabaseConfig` builder

### `engine/` — Core Engine
- **`core.rs`** — `RuniqueEngine` (config, `db: ADb`, Tera, `close_user_sessions`, `extension::<T>()`)
- **`security.rs`** — `SecurityPolicies` (`engine.security`: CSP, allowed hosts, CSRF-exempt paths, Permissions-Policy, trusted proxies)
- **`sessions.rs`** — `SessionStores` (`engine.sessions`: the memory store and the database store)

### `errors/` — Error Types
- **`build.rs`** — build-time error types (`BuildError`, `BuildFailure`, `CheckError`, `CheckReport`)
- **`app_error.rs`** — `AppError`: the error a handler returns, turned into the response
- **`context.rs`** — `ErrorContext`, `ErrorType`: what an error page is rendered from
- **`debug_page.rs`** — what the debug page reads from the project (template source, package name and version)
- **`error.rs`** — `RuniqueError`, `RuniqueResult`: what went wrong, by kind

### `flash/` — Flash Messages
- **`message.rs`** — `FlashMessage` data structure
- **`extractor.rs`** — `Message`, the extractor that reads and writes them in the session

### `forms/` — Form System
- **`config/`** — a field's configuration: `field_config` (`FieldConfig`, `TextConfig`, `NumericConfig`, `FieldBounds`), `limit` (`Limit<T>`), `bool_choice`
- **`form.rs`** — `Forms` struct, `RuniqueForm` trait, `impl_form_access!`
- **`validation_form/`** — `ValidationForm<F>` (validation proven by the type) in `mod.rs`, its `save` / `delete` in `write.rs`, their tests in `tests.rs`
- **`database_error.rs`** — a database error turned into a form error (`Forms::database_error`)
- **`generic.rs`** — generic field implementation
- **`renderer.rs`** — HTML renderer for form fields
- **`validator.rs`** — validation logic
- **`traits/`** — what a form implements: `runique_form` (`RuniqueForm`, `FormAccess`), `model_form` (`ModelForm`, form bound to a model), `form_entity` (`FormEntity`), `form_field` (`FormField`)
- **`data/`** — from a form's values to its model's: `column_value` (field text → column value), `form_data` (values → entity types, `FormDataError`), `save` (the row a write points at)
- **`fields/`** — field types: `text`, `number`, `boolean`, `choice`, `datetime`, `file`, `media`, `binary`, `special`, `hidden`
- **`options/`** — `length`, `bool_choice` option structs
- **`prisme/`** — the request body: `extractor` (`Prisme`, `prisme_pipeline`: CSRF check + body parsing, consumed via `request.form::<T>("name")`), `aegis` (body extraction), `multipart` (multipart reading, uploaded files), `refusal`

### `log/` — Log channels
- **`mod.rs`** — `RuniqueLog`, one file per category beside it (`admin.rs`, `auth.rs`, `db.rs`, `forms.rs`…), the `log_level!` macro
- **`trace_ext.rs`** — `TraceResult`: logs a `Result` that is otherwise dropped
- **`init.rs`** — `init_logging()`, for the command

### `macros/` — Macros
- **`database/`** — `impl_objects!`, `search!` / `search_cond!` (+ internal `search_apply_op!`, `search_munch!`), `Objects` query builder (`objects.rs`, `query.rs`), list fields `HasLists` / `ListField` (`list.rs`)
- **`context/`** — `context!` / `context_update!` (`context_simplifier.rs`), flash macros `flash_now!` / `info!` / `success!` / `warning!` / `error!` (`flash.rs`), `impl_from_error!` (`impl_error.rs`)
- **`forms/`** — `impl_form_access!` (`impl_form.rs`), `define_enum_kind!` (`enum_kind.rs`), `delegate_to_kind!` (`kind.rs`)
- **`template/`** — template context macros

### `mailer/` — Email
- **`mod.rs`** — `Email`, `dispatch_email`, `MailerConfig`, `mailer_configured`

### `middleware/` — Middleware
(authentication lives in the top-level `auth/` module since the refactor)
- **`config.rs`** — middleware configuration builder (slot ordering)
- **`dev/cache.rs`** — dev-mode cache control headers
- **`errors/error.rs`** — error handling middleware (500, 404, custom pages)
- **`extensions.rs`** — `extensions_middleware` (slot 0): puts the engine in the request's extensions
- **`security/allowed_hosts.rs`** — `HostPolicy` middleware (slot 15)
- **`security/anti_bot.rs`** — honeypot / anti-bot heuristics
- **`security/csp.rs`** — CSP header injection (slot 30), HTTPS redirect behind a proxy (slot 17)
- **`security/csrf.rs`** — CSRF middleware (slot 60, always active)
- **`security/open_redirect.rs`** — open-redirect guard for `next`/redirect params
- **`security/permissions_policy.rs`** — `Permissions-Policy` header (allow/deny per feature)
- **`security/rate_limit.rs`** — `RateLimiter` with 429 + `Retry-After`
- **`security/trusted_proxies.rs`** — trusted proxy / client-IP resolution (CIDR, private nets)
- **`session/cleaning_store.rs`** — `CleaningMemoryStore` (128MB/256MB watermarks)
- **`session/session_db.rs`** — DB-backed session store, content encrypted
- **`session/auth.rs`** — `auth_middleware` (slot 57): reads the signed-in account again on every request, `CurrentUser` or the session closed
- **`session/ttl_upgrade.rs`** — `session_ttl_upgrade` (slot 55): the authenticated session lifetime

### `migration/` — Migration Tooling
Folders follow the steps `makemigrations` goes through.
- **`mod.rs`** — public migration API (everything re-exported at `runique::migration::…`)
- **`schema/`** — ① describe a model: `ModelSchema`, `column` (`ColumnDef`), `primary_key`, `foreign_key`, `index`, `hooks`
- **`parse/`** — ② read source into `ParsedSchema`: `builder/` (`model!{}` through `runique_dsl`), `extend`, `seaorm` (existing migration files), `col_types`, `syn_extract`
- **`diff.rs`** — ③ snapshot vs model → `Changes`
- **`changes/`** — ④ order and check the `Changes`: `ordering`, `destructive_changes`, `name_length_limit`
- **`generate/`** — ⑤ write the migration files: `create`, `alter`, `columns`, `constraints` (FKs, indexes), `enums`
- **`cli_migration/`** — the commands' logic: `run` (makemigrations: `scan`, `snapshot`, `plan`, `lib_file`, `report`), `migrate` (up/down/status), `init_migrate` (writes a project's `migration` crate, templates in `composant-bin/`)
- **`paths.rs`**, **`types.rs`**, **`helpers/`** (`naming.rs`, `columns.rs`, `schemas.rs`, `markers.rs`, `file.rs`), **`tests_pipeline.rs`**

### `cli/` — CLI entry points
Calls only: each command's logic lives in its own module, the binary goes through `runique::cli`.
- `makemigration.rs` → `migration::cli_migration`, `migrate.rs` → `migration::cli_migration::migrate`, `new_project.rs` → `new_project`, `create_superuser.rs` → `auth::create_superuser`
- `test_runner.rs` (`runique test`) — still holds its logic, to be moved out

### `router/` — Routing
- **`urlpatterns.rs`** — `urlpatterns!` macro
- **`get_post.rs`** — `view!` macro (GET and POST on one route)
- **`routes.rs`** — `Routes`, what `urlpatterns!` produces
- **`register_url.rs`** — registry of URL names: `register_name_url`, `reverse`, `reverse_with_parameters`
- **`router_ext.rs`** — `RouterExt`: rate limit and login guard on a route

### `security/` — Security primitives
- **`password.rs`** — Argon2 / Bcrypt / Scrypt hashing and verification, `PasswordConfig`
- **`csrf.rs`** — `CsrfToken`, mask / unmask
- **`csp_nonce.rs`** — `CspNonce`
- **`session_cipher.rs`** — session encryption (`SessionKey`)
- **`reset_token/`** — single-use, hashed tokens for password resets

### Root files
- **`lib.rs`** — module declarations, the prelude, and `translation` (re-export of `runique_i18n`: `t`, `tf`, `Lang`)
- **`versions.rs`** — the crate's version and SeaORM's, read by the `runique` command
- **`atomic.rs`** — re-export of `runique_atomic`

Every constant is in the `constants.rs` of its part: `admin/`, `auth/`, `errors/`, `flash/`, `forms/`, `security/`, `templates/`. The anti-bot's session key (`HP_FIELD_KEY`) is in `middleware/security/anti_bot.rs`, its only user.

### `utils/` — Utilities
- **`aliases/`** — `ADb`, `AEngine`, `JsonMap`, `StrMap`… type aliases

---

## Tests (`runique/tests/`)

**2454 integration tests passing** (3 ignored: `render_admin_pages` + 2 `migration up` against a live database — run them with `-- --ignored`). Package `runique`: **3308 passing**, 110 ignored (823 unit + 2429 integration + 56 documentation).

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
│   └── db_mysql.rs         # fresh_db_mysql()
├── admin/                    # registry, form filter, renderer, URL registry
├── app/                      # robots.txt, runique_app
├── auth/                     # admin auth, current user, login form, middlewares, session
├── config/                   # app config, builder, router, security, server, static
├── http/                     # Request, RuniqueContext, request extensions
├── templates/                # Tera filters, URL function, autoescape
├── db/                       # SQLite, Postgres, MySQL config tests
├── errors/                   # error rendering
├── flash/                    # flash manager
├── forms/               # all form tests (aegis, fields, prisme, validator, renderer…)
├── macros/                   # context helper, register URL
├── middleware/               # CSRF, CSP, hosts, rate limit, login guard, session, auth
├── migration/                # column, diff, foreign key, generators, parser, schema, list fields, belongs_to…
├── test_builder/             # runique_test builder
└── utils/                    # constants, flash message, logging, password, sanitizer
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
├── admin.rs                  # ModelAdmin + AdminSite registration
├── demo_toggle.rs            # Feature toggle helpers
├── entities/                 # SeaORM models (30+ entities)
├── forms/               # Form structs (RuniqueForm impls)
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
│   ├── forms/
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
2. Form: `demo-app/src/forms/`
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
| `all-databases`  | All three engines, explicit opt-in only (multi-engine tooling, docs.rs) |
| `big-pk`         | `Pk = i64` instead of `i32`          |
| `pk-uuid`        | `Pk = Uuid` (`Uuid::now_v7()`), mutually exclusive with `big-pk` |
| `acme`           | Automatic TLS via Let's Encrypt (ACME)   |
| `test-utils`     | `runique_test` builder, `Prisme::for_test`, `Forms::mark_validated` |
