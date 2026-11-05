# Comparison: Runique vs Django

Compared with Django 6.1, checked on November 5, 2026 in [its documentation](https://docs.djangoproject.com/en/stable/) and in Runique's code.

## CLI

Real commands of the `runique` binary (see `runique/src/bin/runique.rs`): `new`, `create-superuser`, `test`, `makemigrations`, `migration up`. There is no `runique start` — admin is plain Rust, `cargo run` is enough. Rolling back / listing migrations goes through SeaORM, not a Runique subcommand.

| Command | Django | Runique |
|---------|--------|---------|
| Create project | `django-admin startproject name` | `runique new name` |
| Create app | `python manage.py startapp name` | — |
| Migrations (generate) | `python manage.py makemigrations` | `runique makemigrations` |
| Migrations (apply) | `python manage.py migrate` | `runique migration up` |
| Migrations (rollback) | `python manage.py migrate app 0001` | `runique migration down -n 1` |
| Migration status | `python manage.py showmigrations` | `runique migration status` |
| Create superuser | `python manage.py createsuperuser` | `runique create-superuser` |
| Tests | `python manage.py test` | `runique test` |
| Start | `python manage.py runserver` | `cargo run` |

---

## Routing

| Feature | Django | Runique |
|---------|--------|---------|
| URL declaration | `urls.py` with `path()` | `url.rs` with `urlpatterns!{}` macro |
| Dynamic routes | `path('users/<int:id>/', view)` | `"/users/{id}"` in `urlpatterns!` |
| Namespaces | `app_name` + `include()` | `Router::new().nest("/prefix", ...)` |
| Reverse URL | `{% url "view_name" %}` native | `{% link "view_name" %}` → custom Tera function |
| Typed path param | `<int:id>` in `path()`, converted to `int` | `request.get_path_as::<i32>("id")` |
| Raw path param | `<str:id>` | `request.get_path("id")` |
| Single query param | `request.GET.get('key')` | `request.get_query("key")` |
| Full query string | `request.GET` | `request.query::<MyStruct>()?` (deserializes to a `Deserialize` struct, 400 when the query doesn't fit) |
| HTTP headers | `request.META['HTTP_X_FOO']` | `request.headers.get("x-foo")` |

---

## Views / Handlers

| Feature | Django | Runique |
|---------|--------|---------|
| Function view | `def my_view(request)` | `async fn my_view(...)` |
| Class view | `class MyView(View)` | — |
| Session access | `request.session` | `request.session` via `context::template::Request` |
| DB access | `Model.objects.get(...)` | `Model::objects.get(...)` or SeaORM query builders |
| Template render | `render(request, "template.html", ctx)` | `request.render("template.html")` |
| Redirect | `redirect("url_name")` | `Redirect::to("/url")` or `reverse(&engine, "name")` |
| Flash messages | `messages.success(request, "...")` | `success!`, `error!`, `info!`, `warning!` macros |
| Secondary connections | `DATABASES['mongo']` | `engine.extension::<mongodb::Client>()` (multi-type TypeMap) |

---

## Forms

| Feature | Django | Runique |
|---------|--------|---------|
| Definition | `class MyForm(ModelForm)` | `#[form(schema = blog, fields = [...])]`, tied to its entity by the schema, or manual `RuniqueForm` |
| Validation | `form.is_valid()` | `ValidationForm::try_form(form, &request).await` |
| Available fields | CharField, EmailField, FileField, etc. | TextField (+ `::email`/`::password`/`::url`/`::phone` constructors → correct `<input type>` + validation), NumericField, BooleanField, CheckboxField, RadioField, ChoiceField, FileField, DateField, TimeField, DateTimeField, DurationField, ColorField, SlugField, UUIDField, IPAddressField, JSONField, HiddenField, MediaField, BinaryField |
| HTML rendering | `{{ form.as_p }}` | `{% form.my_form %}` (full) or `{% form.my_form.field %}` |
| CSRF included | automatic | automatic — injected before the first field |
| Save to DB | `form.save()` | `validated.save(&db).await`, on the validated form, for any form carrying its entity (`FormEntity`): `#[form(schema = …)]` or `impl FormEntity` |
| Data access | `form.cleaned_data['key']` | `validated.cleaned::<String>("key")`, `validated.cleaned::<i32>(...)`, etc. |
| Async validation | no | yes (DB access in `clean()`) |
| Cross-field validation | `clean()` | `clean()` async |
| File fields | `FileField` (no content validation by default) | `FileField` native multipart — type/size validation, **magic bytes** for images, SVG rejected, written to **staging** only after the CSRF token, then committed after validation (**UUID** name, no path traversal) |
| HTML sanitization | manual | `sanitize_rich` / `sanitize_strict` applied to `richtext` fields |

---

## Templates

| Feature | Django | Runique |
|---------|--------|---------|
| Engine | Django Template Language | Tera (Jinja2 / Django-like syntax) |
| Inheritance | `{% extends %}` / `{% block %}` | same |
| Static files | `{% load static %}` + `{% static "file" %}` | `{% static "file" %}` native |
| Media files | `{{ MEDIA_URL }}file` | `{% media "file" %}` native (variables supported) |
| URL reverse | `{% url "name" %}` | `{% link "name" %}` |
| CSRF | `{% csrf_token %}` | `{% csrf %}` |
| Messages | `{% for m in messages %}` | `{% messages %}` |
| I18n | `{% trans "..." %}` | — (`t()` and `tf()` on the Rust side) |

---

## ORM / Database

| Feature | Django | Runique |
|---------|--------|---------|
| ORM | Native Django ORM | SeaORM (Rust async) |
| Model definition | `class User(models.Model)` | `model!{}` macro (semantic types) |
| Auto migrations | yes (change detection) | `runique makemigrations` |
| Chainable QuerySet | `User.objects.filter(...).order_by(...)` | `User::objects.filter(...).order_by(...)` |
| Strict `.get()` | raises `MultipleObjectsReturned` | `.one()` — returns `Err` if multiple rows |
| Random ordering | `order_by('?')` | `.order_by_random()` |
| Expression ordering | `order_by(Lower("title").desc())` | `.order_by_expr(expr, order)` |
| Relations | ForeignKey, ManyToMany, OneToOne | Standard SeaORM relations |
| Transactions | `with transaction.atomic()` | `request.db_atomic()` or a SeaORM transaction |
| Multi-engine SQL | yes | PostgreSQL, MySQL/MariaDB, SQLite (3 SeaORM backends; MariaDB rides the MySQL driver with its own migration dialect) |
| Secondary connections | `DATABASES` multiple entries | `.with_custom_db(client)`, one client per type (TypeMap) |
| Framework table extension | — | `extend!{}` — `ALTER TABLE ADD COLUMN` on `eihwaz_*` tables |

---

## Authentication

| Feature | Django | Runique |
|---------|--------|---------|
| Login / Logout | `authenticate()` + `login()` | `authenticate_user()` + `login()`, `logout()` |
| Is authenticated | `request.user.is_authenticated` | `is_authenticated(&session).await` |
| Current user | `request.user` | `request.user` |
| Route protection | `@login_required` | `.login_required("/profile", "profile", view!(profile), "/login")` |
| Sessions | native | tower-sessions (MemoryStore + DB fallback) |
| Brute force protection | `django-axes` (3rd party) | Native `LoginGuard` (auto lockout) |
| Password hashing | PBKDF2 by default, optional Argon2 | Argon2 by default |
| Password reset | signed token bound to the password hash (stateless) | DB-persisted **hashed**, single-use tokens (IDOR-hardened — keyed on `user_id`), configurable TTL via `.token_ttl(...)`, customizable email template |
| Force logout | changing the password closes its sessions | changing the password too; `invalidate_all(user_id)` on `RuniqueSessionStore` |

---

## Security

| Feature | Django | Runique |
|---------|--------|---------|
| CSRF | native | native (constant-time validation) |
| CSP | native since Django 6.0 (`ContentSecurityPolicyMiddleware`), to enable | native (per-request nonce always active) |
| HSTS | `SECURE_HSTS_SECONDS` | native — emitted **only over real HTTPS** (`enforce_https`/ACME), `max-age`/`includeSubDomains`/`preload` configurable (preload **opt-in**) |
| SameSite cookies | configurable | `Strict` by default |
| HttpOnly cookies | by default | always `true` |
| Rate limiting | `django-ratelimit` (3rd party) | Native `RateLimiter`, declared per route — keyed on the client IP resolved through trusted proxies, otherwise the `ConnectInfo` peer; never an unvalidated header |
| Open Redirect | — | native — `Location` of 3xx responses **and** the `Refresh` header of any response, target read by the WHATWG URL parser (the one browsers use), refused when in doubt |
| CORS | `django-cors-headers` (3rd party) | native via `.with_cors(...)` |
| Permissions-Policy | — | native — secure preset by default (~20 features denied) |
| Trusted Proxies / XFF | `SECURE_PROXY_SSL_HEADER` (partial) | native — full XFF chain validation (right→left), RFC 1918 preset, **edge-aware default**: auto `none()` when Runique terminates TLS (ACME = no proxy); a `/0` range (trusting everyone) is refused at startup |
| Secret key | manual | auto-generated on `runique new` — **refuses to boot in prod** if missing / default value / < 32 chars |

---

## Admin Panel

| Feature | Django | Runique |
|---------|--------|---------|
| Registration | `admin.site.register(Model)` | typed `ModelAdmin` + `AdminSite.register(...)` |
| Full CRUD | native | native |
| List pagination | native | `.page_size(n)` site-wide + `.per_page(n)` per model |
| Displayed columns | `list_display` | `.list(\|l\| l.display([Title, Status]))` — typed SeaORM columns |
| Search / filters | native | `.search([...])` + `.filter([...])` — direct columns only, combinable filters |
| Group actions | `actions` | `.action(Col, value, "Label")` — fixed value in code, UI sends the action index |
| Bulk edit | `list_editable` (several rows from the list) | batch edit on a selection |
| Bulk delete | native | native when the account may delete |
| M2M / list fields | `filter_horizontal` / `ManyRelatedField` | handled via model list fields (`multichoice` / `checkbox`) |
| Custom admin routes | `get_urls()` | `admin "…" => …` in `urlpatterns!` |
| Custom templates | yes | yes (Tera hierarchy + per-model `.templates(...)`) |
| Permissions | per resource | Ranks, admin rights per account (table and column) |
| Superuser accounts | a staff member allowed to edit users can make themselves a superuser (Django's docs warn about it) | `is_superuser` is never editable from the admin, and a superuser's account (email, password reset, deletion) can only be changed by a superuser |
| Change history | native (`LogEntry`, "History" link); field diffs with `django-simple-history` (3rd party) | native history (created/modified/deleted) with field diff, timeline view and filters (resource/action/user) |
| Extended accounts | `AUTH_USER_MODEL` (custom user model) | `extend!{}` + `AdminSite::accounts_users(ModelAdmin)` |
| Optimistic locking | — | native — detects concurrent edits (`__original_updated_at`) |
| Admin login brute-force protection | `django-axes` (3rd party) | native — `LoginGuard` + rate limiter on admin login |
| Account creation email | 3rd-party plugin | native — "choose my password" link sent to the account's address, activated on first set |
| Partial list refresh | — | native — `list_partial` HTMX without full page reload |
| Read-only / create-only fields | `readonly_fields` | `.readonly([...])` / `.create_only([...])` on the form builder |
| Fieldsets | `fieldsets` | — |
| Date hierarchy | `date_hierarchy` | — |
| Export | 3rd party (`django-import-export`) | — |
| Advanced save buttons | "Save and continue" / "Save and add another" | — |

---

## Email

| Feature | Django | Runique |
|---------|--------|---------|
| Send mail | `send_mail()` native | `Email::new().send()` native |
| Email templates | native | Tera templates via `.template("emails/my.html")` |
| SMTP backend | configurable | configuration via `.env` |
| Dev backend (console) | `EMAIL_BACKEND = 'console'` | `EMAIL_BACKEND=console` in `.env` |

---

## Observability / Logging

| Feature | Django | Runique |
|---------|--------|---------|
| Logging config | `LOGGING` dict (Python `logging`) | `RuniqueLog` per-domain tree (`forms`, `middleware`, `session`, `auth`, `admin`, `db`, `mailer`, `migration`, `templates`, `errors`, `builder`) — each leaf an `Option<LogLevel>` |
| Structured tracing | 3rd party (`structlog`) | native (`tracing`); swallowed `Result` sites log their `file:line` via `TraceResult::trace` / `trace_or` |
| Outputs | handlers (console / file / …) | `.output(...)`: colored stdout, rolling file (JSON or plain, non-blocking), custom `LogSink` (DB / HTTP / queue) |
| Runtime file override | — | `RUNIQUE_LOG_FILE` env var |
| App-owned subscriber | logging is app-owned | `.external()` — Runique installs nothing, still emits to the `tracing` facade |

---

## Internationalization

| Feature | Django | Runique |
|---------|--------|---------|
| Languages | unlimited | 9 default languages (compiled JSON) |
| Fallback | yes | yes (`Lang::En`) |
| Translation | `_("...")` | `t("section.key")` / `tf("...", &[var])` in Rust, framework texts |

---

## Performance & Deployment

| Aspect | Django | Runique |
|--------|--------|---------|
| Runtime | CPython (interpreted) | Tokio async Rust (compiled) |
| Compilation | — | single binary |
| ACME / auto-TLS | `certbot` (external) | native via `acme` feature |

---

## What Runique is still missing (compared to Django)

- **Image resizing**: no server-side auto resize/cropping.
- **Generic public CRUD views**: no equivalent to Django's `DetailView`, `ListView`, `CreateView` for public views — considered via `crud!{}` (roadmap, no commitment). The admin panel covers backoffice CRUD via `ModelAdmin`.
- **Model signals / hooks**: `before_save`, `after_save`, `before_delete`, `after_delete` can't be declared in `model!{}`. The internal `HooksDef` struct exists, with no link to the DSL; the behaviour `model!{}` generates only fills `auto_now` columns. On the roadmap, with no commitment.
- **Management commands**: no equivalent to `manage.py custom_command` — one-shot operations go through `src/bin/`.
- **Built-in test client**: no native HTTP test client. `runique test` and `runique_test` cover running tests and isolating the database; requests go through `reqwest` or `tower::ServiceExt::oneshot`.
- **Fixtures**: no `loaddata`/`dumpdata` — seeds are plain Rust functions.
- **Admin inline**: no editing of related objects directly inside the parent form.
- **Admin custom group actions**: `.action(...)` only supports fixed column updates — no arbitrary Rust logic on the selection (Django equivalent: `actions` with any callable).
- **Admin FK filters**: `.filter([...])` supports direct columns only — no relation traversal (`article__author__name`).
- **Admin fieldsets**: no field grouping by section in admin forms (`fieldsets` in Django).
- **Admin date hierarchy**: no `date_hierarchy` to filter the list by year/month/day.
- **Admin export**: no native CSV/JSON export from the list view.
- **Admin advanced save buttons**: no "Save and continue editing" / "Save and add another".
- **Full i18n**: `t()` and `tf()` are Rust functions for the framework's texts. No translation function in Tera templates, no pluralization, and a project can't add its own translations yet.
- **Background tasks**: no task framework; Django has one since 6.0 (`@task`, with an external worker).
- **Sitemap / RSS**: no native generation. `.sitemap(url)` only adds the `Sitemap:` directive to the generated `/robots.txt`.
- **Third-party authentication**: `Delegated` mode names providers (Google, Microsoft, Apple, LDAP, SAML, custom), but no flow is implemented. No JWT or API key auth.
- **Security observability**: no CSP `report-uri`/`report-to`, so violations aren't collected. Logins (successful, failed, lockouts) go through the logs (`RuniqueLog`) and aren't saved in the database; `eihwaz_history` traces admin writes, rights and resets.
