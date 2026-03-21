# Runique vs Django — Feature Comparison

## CLI

| Command | Django | Runique |
|---------|--------|---------|
| Create a project | `django-admin startproject name` | `runique new name` |
| Create an app | `python manage.py startapp name` | — |
| Generate migrations | `python manage.py makemigrations` | `runique makemigrations` (detects changes from entities) |
| Apply migrations | `python manage.py migrate` | `runique migration up` (wraps `sea-orm-cli migrate up`) |
| Rollback migrations | `python manage.py migrate app 0001` | `runique migration down --files ...` (wraps `sea-orm-cli migrate down`) |
| Migration status | — | `runique migration status` (wraps `sea-orm-cli migrate status`) |
| Create superuser | `python manage.py createsuperuser` | `runique create-superuser` |
| Start services | `python manage.py runserver` | `cargo run` — `runique start` only to initialize/refresh the admin view |

---

## Routing

| Feature | Django | Runique |
|---------|--------|---------|
| Route declaration | `urls.py` with `path()` | `url.rs` with axum `Router` |
| Dynamic routes | `path('users/<int:id>/', view)` | `.route("/users/:id", get(view))` |
| Namespaces | `app_name` + `include()` | `Router::new().nest("/prefix", ...)` |
| Reverse URL | `{% url "view_name" %}` native | `{% link "name" %}` → custom Tera function |

---

## Views / Handlers

| Feature | Django | Runique |
|---------|--------|---------|
| Function view | `def my_view(request)` | `async fn my_view(...)` |
| Class-based view | `class MyView(View)` | — |
| Session access | `request.session` | `request.session` via `context::template::Request` (or `Session` axum extractor directly) |
| DB access | `Model.objects.get(...)` | sea-orm query builders |
| Template rendering | `render(request, "template.html", ctx)` | `request.render("template.html")` — context already in `request.context` |
| Redirect | `redirect("url_name")` | `Redirect::to("/url")` or `reverse(&engine, "name")` / `reverse_with_parameters(...)` (prelude) |
| Flash messages | `messages.success(request, "...")` | `success!(message => "...")` — macros `success!`, `error!`, `info!`, `warning!` (prelude) |

---

## Forms

| Feature | Django | Runique |
|---------|--------|---------|
| Definition | `class MyForm(forms.Form)` / `class MyForm(ModelForm)` | `#[derive(RuniqueForm)] struct MyForm` (equivalent to `ModelForm`) |
| Validation | `form.is_valid()` | `form.is_valid().await` |
| Available fields | CharField, EmailField, etc. | TextField, EmailField, PasswordField, HiddenField, etc. (fixed list, no custom widget) |
| HTML rendering | `{{ form.as_p }}` | `{% form.my_form %}` (full form) or `{% form.my_form.field %}` (individual field) |
| CSRF included | automatic | automatic — injected by Tera `form_filter` before the first field |
| Save | `form.save()` | `form.save(&db).await` |
| Async validation | no | yes (DB access possible) |
| File forms | `FileField` | Multipart partial |

---

## Templates

| Feature | Django | Runique |
|---------|--------|---------|
| Engine | Django Template Language | Tera (Jinja2 syntax) |
| Inheritance | `{% extends %}` / `{% block %}` | same in Tera |
| Static files | `{% load static %}` `{% static "file" %}` | `{% static "file" %}` native |
| Media files | `{{ MEDIA_URL }}file` | `{% media "file" %}` native |
| Reverse URL | `{% url "name" %}` | `{% link "name" %}` |
| CSRF | `{% csrf_token %}` | `{% csrf %}` |
| Messages | `{% for m in messages %}` | `{% messages %}` |
| Internationalization | `{% trans "..." %}` | `{{ t("section.key") }}` |

---

## ORM / Database

| Feature | Django | Runique |
|---------|--------|---------|
| ORM | Django ORM native | sea-orm |
| Model definition | `class User(models.Model)` | sea-orm entity (Rust struct) in `src/entities/` (required folder, read by the parser) |
| Auto migrations | yes (change detection) | `runique makemigrations` (change detection from entities) |
| Chainable QuerySet | `User.objects.filter(...).order_by(...)` | sea-orm Select builder |
| Relations | ForeignKey, ManyToMany, OneToOne | sea-orm relations |
| Transactions | `with transaction.atomic()` | `db.transaction(...)` sea-orm |
| Multi-DB | yes | PostgreSQL, MySQL, SQLite |
| NoSQL | via third-party packages | via third-party crates (e.g. `mongodb`) |
| Re-export | — | `runique::sea_orm` + `sea_query` |

---

## Authentication

| Feature | Django | Runique |
|---------|--------|---------|
| Login / Logout | `authenticate()` + `login()` | `login()`, `login_staff()`, `logout()` — `LoginGuard` = brute-force middleware |
| Auth check | `request.user.is_authenticated` | `is_authenticated(&session).await` |
| Current user | `request.user` | `CurrentUser` (injected via `load_user_middleware`) |
| Route protection | `@login_required` | `login_required` middleware |
| Redirect if authenticated | manual | `redirect_if_authenticated` middleware |
| Sessions | native | tower-sessions |
| Brute-force protection | `django-axes` (third-party) | `LoginGuard` native (attempts + lockout) |
| Password hashing | PBKDF2 / argon2 | argon2, bcrypt, scrypt, custom (auto-detected at verification) |
| Email account activation | native (`auth`) | **missing** |
| Password reset | native | **missing** (planned via `lettre`) |
| Force logout all sessions | yes | **missing** |

---

## Security

| Feature | Django | Runique |
|---------|--------|---------|
| CSRF | native | native (constant-time via `subtle`) |
| CSP | `django-csp` (third-party) | native (`use_nonce: true` by default) |
| HSTS | `SECURE_HSTS_SECONDS` | native (`max-age=31536000; includeSubDomains`) |
| SameSite cookies | configurable | `Strict` by default |
| HttpOnly cookies | by default | always `true` |
| Host validation | `ALLOWED_HOSTS` | `RUNIQUE_ALLOWED_HOSTS` + `RUNIQUE_ENABLE_HOST_VALIDATION` |
| Rate limiting | `django-ratelimit` (third-party) | `RateLimiter` native |
| Input sanitization | — | native sanitize middleware |
| Secret key generation | manual | `runique new` generates 32 bytes hex automatically |

---

## Admin View

| Feature | Django | Runique |
|---------|--------|---------|
| Activation | `admin.site.register(Model)` | `admin!{}` macro + `runique start` |
| List / Create / Edit / Detail / Delete | native | native |
| List pagination | native | **missing** |
| `list_display` | native | **missing** |
| Search / filters | native | **missing** |
| Customizable templates | yes | yes (Tera hierarchy) |
| Per-resource permissions | native | stored, not injected into Tera context |
| Admin account creation | `createsuperuser` | `runique create-superuser` |
| Admin account from the app | no | no (same) |

---

## Email

| Feature | Django | Runique |
|---------|--------|---------|
| Send email | `send_mail()` native | **missing** — plug `lettre` |
| Email templates | native | **missing** |
| SMTP/console backend | configurable | — |

---

## Internationalization

| Feature | Django | Runique |
|---------|--------|---------|
| Supported languages | unlimited (`.po` files) | 9 (`en`, `fr`, `it`, `es`, `de`, `pt`, `ja`, `zh`, `ru`) |
| Auto language detection | `LocaleMiddleware` | `LANG` / `LC_ALL` env |
| Fallback | yes | yes (`Lang::En`) |
| Framework translations | `.po`/`.mo` | JSON files (14 sections, compiled into the binary) |
| `t("key")` | `_("...")` | `t("section.key")` → `Cow<'static, str>` |

---

## Performance & Deployment

| Aspect | Django | Runique |
|--------|--------|---------|
| Runtime | CPython (GIL) | Tokio async Rust |
| Production server | Gunicorn + Nginx | compiled binary (Axum/Hyper) |
| Memory footprint | ~50–100 MB | ~5–15 MB |
| Compilation | — | `cargo build --release` |
| Docker | yes | yes |
| Deployment | fly.io, Heroku, Azure, etc. | same (static binary = simpler) |

---

## What Runique is still missing (summary)

- Full auth flow (email activation, password reset)
- Native email integration
- Robust file upload (MIME validation, resize)
- Admin pagination + `list_display` + filters
- Runtime permissions in admin
- Equivalent to `django-simple-history`
- Native NoSQL (out of scope, plug `mongodb`)
