# Comparison: Runique vs Cot

Checked on November 5, 2026 in [the guide](https://cot.rs/guide/latest/) and [the repository](https://github.com/cot-rs/cot) of Cot 0.7. "—": nothing of the kind found in its documentation or its code.

## CLI

| Command | Cot | Runique |
|---------|-----|---------|
| Install | `cargo install --locked cot-cli` | `cargo install runique --features sqlite --locked` |
| Create a project | `cot new name` | `runique new name` |
| Migrations (generate) | `cot migration make` | `runique makemigrations` |
| Empty migration | `cot migration new <name>` | — |
| Migrations (apply) | automatic at startup | `runique migration up` |
| Migrations (roll back) | `cargo run migration rollback <name>` | `runique migration down -n 1` |
| Migration status | `cot migration list` | `runique migration status` |
| Create a superuser | code in `App::init` in the guide's example | `runique create-superuser` |
| Collect static files | `cargo run -- collect-static public/` | — |
| Start | `cargo run` | `cargo run` |
| Code reload | `bacon` (generated `bacon.toml`) | — |

## Routing

| Feature | Cot | Runique |
|---------|-----|---------|
| Route declaration | `Router::with_urls([Route::with_handler_and_name("/", index, "index")])` | `url.rs` with the `urlpatterns!{}` macro |
| Dynamic routes | `"/hello/{name}"` | `"/users/{id}"` in `urlpatterns!` |
| Reverse URL | `cot::reverse!(urls, "name")` | `{% link "view_name" %}` |
| Path parameter | `Path(name): Path<String>` extractor | Axum `Path` or `request.get_path_as::<i32>("id")` |

## Views / Handlers

| Feature | Cot | Runique |
|---------|-----|---------|
| Handler | `async fn index() -> cot::Result<Html>` | `async fn my_view(mut request: Request) -> AppResult<Response>` |
| Database access | `db: Database` extractor | `request.engine.db` |
| Template rendering | `IndexTemplate { .. }.render()?` | `request.render("template.html")` |
| Redirect | `reverse_redirect!(request, "name")` | `Redirect::to("/url")` |
| Flash messages | — | `success!`, `error!`, `info!`, `warning!` macros |
| Error pages | `error_handler()`, debug page with `debug = true` | `404.html`, `500.html` templates, debug page with `DEBUG=true` |
| API | `openapi`, `swagger-ui` features | — |

## Forms

| Feature | Cot | Runique |
|---------|-----|---------|
| Definition | `#[derive(Form)]` | `#[form(schema = blog, fields = [...])]` or a manual `RuniqueForm` |
| Reading and validation | `ContactForm::from_request(&mut request).await?` | `ValidationForm::try_form(form, &request).await` |
| Built-in validation | `#[form(opts(max_length = 100))]` | model or field constraints |
| Custom validation | `context.add_error(FormErrorTarget::Field("title"), ...)` | async `clean()`, with database access |
| HTML rendering | `{{ form }}` or `form.fields()` | `{% form.form_name %}` or `{% form.form_name.field %}` |
| Built-in CSRF | — | automatic, injected before the first field |
| Files | `form/fields/files.rs` | `FileField` and `MediaField`: type, size, content signature for images, audio and video |

## Templates

| Feature | Cot | Runique |
|---------|-----|---------|
| Engine | Askama, templates bound to a struct | Tera |
| Inheritance | `{% extends %}` / `{% block %}` | same |
| Inclusion | `{% include %}` | same |
| Static files | `static_files!(...)`, `StaticFiles` extractor | `{% static "file" %}`, with SRI in production |
| Static file versioning | `rewrite = "query_param"` | automatic `?v=` |
| Reverse URL | `cot::reverse!(urls, "name")` | `{% link "name" %}` |
| Internationalization | — | `t()` and `tf()` in Rust, 9 languages for the framework's texts |

## ORM / Database

| Feature | Cot | Runique |
|---------|-----|---------|
| ORM | its own ORM (`sea-query`, `sqlx`) | SeaORM |
| Engines | SQLite, PostgreSQL, MySQL, all on by default | SQLite, PostgreSQL, MySQL/MariaDB, one engine per build |
| Model definition | `#[model]` on a struct | `model!{}` macro |
| Foreign key | `ForeignKey<User>` | `belongs_to`, with a required on-delete action |
| Queries | `query!(Customer, $id == 5)`, `Customer::objects()` | `search!`, `Model::objects` |
| Bulk insert | `bulk_insert` | SeaORM |
| Transactions | `db.begin()`, nesting through savepoints | `request.db_atomic()` and SeaORM |
| Custom migration | `#[migration_op]`, `forwards` and `backwards` | hand-written SeaORM migration |

## Authentication

| Feature | Cot | Runique |
|---------|-----|---------|
| Login / Logout | `Auth` extractor: `auth.login(...)`, `auth.logout()` | `authenticate_user()` + `login()`, `logout()` |
| Users | `DatabaseUser`, `DatabaseUserApp` | `eihwaz_users` table, extensible through `extend!{}` |
| Sessions | memory, database, file, Redis stores | tower-sessions (memory + database fallback), custom store possible |
| Session cookie | `Secure`, `HttpOnly`, `SameSite=Strict` by default, configurable | `HttpOnly`, `SameSite=Strict`, `Secure` in production |
| Password hashing | Argon2 | Argon2 by default |
| Brute-force protection | — | `LoginGuard` |
| Password reset | — | tokens hashed in the database, single-use, with an expiry |

## Security

| Feature | Cot | Runique |
|---------|-----|---------|
| CSRF | — | always on |
| CSP and security headers | — | on by default, nonce per request |
| Rate limiting | — | `RateLimiter`, declared per route |
| CORS | — | `.with_cors(...)` |
| Allowed hosts | — | `.with_allowed_hosts(...)` |
| Open redirect | — | middleware always on |

## Admin panel

| Feature | Cot | Runique |
|---------|-----|---------|
| Setup | `#[derive(AdminModel)]` + `DefaultAdminModelManager` | typed `ModelAdmin` builder + `AdminSite.register(...)` |
| Full CRUD | built in | built in |
| Pagination | built in | `.page_size(n)`, `.per_page(n)` |
| Search / filters | — | `.search([...])`, `.filter([...])` |
| Bulk actions | — | `.action(Col, value, "Label")` |
| Access | signed-in user | ranks, admins' permissions per account (table and column) |
| History | — | built in |

## Email and cache

| Feature | Cot | Runique |
|---------|-----|---------|
| Sending email | `EmailMessage::builder()`, `email` feature | `Email::new().send()` |
| Dev backend (console) | `type = "console"` | `EMAIL_BACKEND=console` |
| Configuration | `[email]` in TOML | `.env` |
| Cache | memory, Redis, file (`Cache`) | — |

## Observability / Logging

| Feature | Cot | Runique |
|---------|-----|---------|
| Foundation | `tracing` | `tracing` |
| Configuration | — | `RuniqueLog` tree, per domain, stdout, file, `LogSink` |

## Tests

| Feature | Cot | Runique |
|---------|-----|---------|
| Command | `cargo test` | `runique test` |
| Unit | `TestRequestBuilder` | `tower::ServiceExt::oneshot` |
| Integration | `Client::new(MyProject)` | `reqwest` |
| Test database | `TestDatabase::new_sqlite()`, `new_postgres`, `new_mysql` | `runique_test`, one transaction per test |
| End to end | `#[cot::e2e_test]` | — |

## Deployment

| Aspect | Cot | Runique |
|--------|-----|---------|
| Configuration | `config/dev.toml`, `config/prod.toml` | `.env`, `RUNIQUE_ENV` |
| Automatic TLS | — | `acme` feature |
| License | MIT or Apache-2.0 | MIT |
