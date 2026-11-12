# Comparison: Runique vs Loco

Checked on November 5, 2026 in [the documentation](https://loco.rs/docs/) and [the repository](https://github.com/loco-rs/loco) of Loco 1.3.1. "—": nothing of the kind found in its documentation or its code.

## CLI

| Command | Loco | Runique |
|---------|------|---------|
| Install | `cargo install loco` | `cargo install runique --features sqlite --locked` |
| Create a project | `loco new` (wizard: app type, database, worker) | `runique new name` |
| Migrations (generate) | `cargo loco generate model` / `generate migration` | `runique makemigrations` |
| Migrations (apply) | `cargo loco db migrate` | `runique migration up` |
| Migrations (roll back) | `cargo loco db down` | `runique migration down -n 1` |
| Migration status | `cargo loco db status` | `runique migration status` |
| Regenerate entities | `cargo loco db entities` | — (`model!{}` generates them at compile time) |
| Code generators | model, controller, scaffold, worker, mailer, task, scheduler, deployment | — |
| List routes | `cargo loco routes` | — |
| List middlewares | `cargo loco middleware` | — |
| Diagnostics | `cargo loco doctor` | — |
| Create a superuser | — | `runique create-superuser` |
| Start | `cargo loco start` | `cargo run` |
| Code reload | `cargo loco watch` (with `cargo-watch`) | — |

## Routing

| Feature | Loco | Runique |
|---------|------|---------|
| Route declaration | `Routes::new().prefix("notes").add("/", get(list))` | `url.rs` with the `urlpatterns!{}` macro |
| Dynamic routes | `.add("/{id}", get(get_one))` | `"/users/{id}"` in `urlpatterns!` |
| Reverse URL | — | `{% link "view_name" %}` |
| Path parameter | Axum `Path` extractor | Axum `Path` or `request.get_path_as::<i32>("id")` |
| Query string | `QueryValidate<T>` or an Axum extractor | `request.query::<MyStruct>()?` |
| Application prefix | `AppRoutes` | `Router::new().nest("/prefix", ...)` |

## Views / Handlers

| Feature | Loco | Runique |
|---------|------|---------|
| Handler | `async fn hello() -> Result<Response>` | `async fn my_view(mut request: Request) -> AppResult<Response>` |
| Database access | `State(ctx): State<AppContext>`, then `ctx.db` | `request.engine.db` |
| Rendering | `format::render().view(&v, "home/hello.html", data)` | `request.render("template.html")` |
| JSON response | `format::json(res)` | Axum's `Json` |
| Redirect | `format::redirect("/url")` | `Redirect::to("/url")` |
| Flash messages | — | `success!`, `error!`, `info!`, `warning!` macros |
| Secondary connections | `MultiDbInitializer`, named connections | `.with_custom_db(client)` |

## Forms

| Feature | Loco | Runique |
|---------|------|---------|
| Definition | `#[derive(Deserialize, Validate)]` struct (`validator` crate) | `#[form(schema = blog, fields = [...])]` or a manual `RuniqueForm` |
| Reading and validation | `FormValidate<T>`, `JsonValidate<T>`, `QueryValidate<T>` extractors | `ValidationForm::try_form(form, &request).await` |
| HTML rendering | — | `{% form.form_name %}` or `{% form.form_name.field %}` |
| Built-in CSRF | — | automatic, injected before the first field |
| Saving | model code (SeaORM) | `validated.save(&db).await`, the form carries its entity (`FormEntity`) |
| Cross-field validation | `ValidatorTrait` | async `clean()`, with database access |
| Files | Axum's `Multipart`, configurable storage | `FileField` and `MediaField`: type, size, content signature for images, audio and video |
| Storage | disk, memory, S3, GCP, Azure | `MEDIA_ROOT` folder |

## Templates

| Feature | Loco | Runique |
|---------|------|---------|
| Engine | Tera, replaceable engine (`ViewRenderer`) | Tera |
| Location | `assets/views/` | `templates/` |
| Static files | `static` middleware (`assets/static`) | `{% static "file" %}`, with SRI in production |
| Media files | — | `{% media "file" %}` |
| Reverse URL | — | `{% link "name" %}` |
| CSRF | — | `{% csrf %}` |
| Messages | — | `{% messages %}` |
| Self-contained binary | `embedded_assets` feature | — |

## ORM / Database

| Feature | Loco | Runique |
|---------|------|---------|
| ORM | SeaORM | SeaORM |
| Engines | SQLite, PostgreSQL (default features) | SQLite, PostgreSQL, MySQL/MariaDB, one engine per build |
| Model definition | entities generated from the migrations | `model!{}` macro |
| Migrations | `loco_rs::schema` DSL, written or generated | `runique makemigrations` compares with the last snapshot |
| Queries | `query::condition()` and SeaORM | `search!` and SeaORM |
| Pagination | `query::paginate` | SeaORM's `.paginate(...)` |
| Transactions | SeaORM | `request.db_atomic()` and SeaORM |
| Multi-tenant | `multi-tenancy` feature | — |
| Extending a framework table | — | `extend!{}` on the `eihwaz_*` tables |

## Authentication

| Feature | Loco | Runique |
|---------|------|---------|
| Login / Logout | generated `/api/auth/*` routes | `authenticate_user()` + `login()`, `logout()` |
| JWT | `auth::JWT`, `auth::JWTWithUser<T>` | — |
| API key | `find_by_api_key` | — |
| Sessions | signed or encrypted cookie, database and Redis stores (`auth_session` feature) | tower-sessions (memory + database fallback) |
| OAuth2 / OIDC | `auth_oauth` feature | — |
| Brute-force protection | — | `LoginGuard` |
| Password hashing | Argon2id (`hash::hash_password`) | Argon2 by default |
| Password reset | generated `forgot` and `reset` routes | tokens hashed in the database, single-use, with an expiry |
| Permissions | — | ranks, and admins' permissions per account |

## Security

| Feature | Loco | Runique |
|---------|------|---------|
| CSRF | — | always on |
| CSP and headers | `secure_headers` to turn on, `github` and `owasp` presets | on by default, nonce per request |
| Rate limiting | a `tower` layer chosen by the project | `RateLimiter`, declared per route |
| CORS | `cors` middleware | `.with_cors(...)` |
| Allowed hosts | — | `.with_allowed_hosts(...)` |
| Open redirect | — | middleware always on |
| Real IP behind a proxy | `remote_ip` middleware | `.with_trusted_proxies(...)` |

## Admin panel

| Feature | Loco | Runique |
|---------|------|---------|
| Setup | — | typed `ModelAdmin` builder + `AdminSite.register(...)` |
| CRUD, search, filters | — | built in |
| Permissions | — | ranks, admins' permissions per account (table and column) |
| History | — | built in |

## Background tasks

| Feature | Loco | Runique |
|---------|------|---------|
| Workers | `Worker` trait, Redis, PostgreSQL, SQLite or thread queues | — |
| Scheduler | `cargo loco scheduler`, cron expressions | — |
| One-off tasks | `cargo loco task` | — |
| Cache | memory, Redis, database | — |

## Email

| Feature | Loco | Runique |
|---------|------|---------|
| Sending | mailers sent by a worker | `Email::new().send()` |
| Templates | `subject.t`, `html.t`, `text.t` (Tera) | Tera templates via `.template(&tera, "emails/my.html", ctx.into())` |
| SMTP backend | `config/*.yaml` files | `.env` |
| Dev backend (console) | — | `EMAIL_BACKEND=console` |

## Observability / Logging

| Feature | Loco | Runique |
|---------|------|---------|
| Logging config | `logger` keys in YAML | `RuniqueLog` tree, per domain |
| Foundation | `tracing` | `tracing` |
| Formats | `compact`, `pretty`, `json` | stdout, JSON or text file, `LogSink` |
| Priority | `RUST_LOG` | `RUST_LOG` |

## Tests

| Feature | Loco | Runique |
|---------|------|---------|
| Command | `cargo test` | `runique test` |
| HTTP requests | `request::<App, _, _>()`, `TestServer` | `reqwest` or `tower::ServiceExt::oneshot` |
| Database isolation | `request_with_create_db`, `#[serial]` | `runique_test`, one transaction per test |

## Deployment

| Aspect | Loco | Runique |
|--------|------|---------|
| Configuration | `config/*.yaml`, `LOCO_ENV` | `.env`, `RUNIQUE_ENV` |
| Delivery | `<package>-cli` binary | binary |
| Generators | Docker, nginx, AWS Lambda | — |
| Automatic TLS | — | `acme` feature |
| License | Apache-2.0 | MIT |
