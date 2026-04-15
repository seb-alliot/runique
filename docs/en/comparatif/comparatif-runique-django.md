# Comparison: Runique vs Django

## CLI

| Command | Django | Runique |
|---------|--------|---------|
| Create project | `django-admin startproject name` | `runique new name` |
| Create app | `python manage.py startapp name` | — |
| Migrations (generate) | `python manage.py makemigrations` | `runique makemigrations` |
| Migrations (apply) | `python manage.py migrate` | `runique migration up` (wrapper for `sea-orm-cli migrate up`) |
| Migrations (rollback) | `python manage.py migrate app 0001` | `runique migration down --files ...` (wrapper for `sea-orm-cli migrate down`) |
| Migration status | — | `runique migration status` (wrapper for `sea-orm-cli migrate status`) |
| Create superuser | `python manage.py createsuperuser` | `runique create-superuser` |
| Start services | `python manage.py runserver` | `cargo run` — `runique start` only for first init/admin view generation |

---

## Routing

| Feature | Django | Runique |
|---------|--------|---------|
| URL Declaration | `urls.py` with `path()` | `url.rs` with `urlpatterns!{}` macro |
| Dynamic Routes | `path('users/<int:id>/', view)` | `"/users/{id}"` in `urlpatterns!` |
| Namespaces | `app_name` + `include()` | `Router::new().nest("/prefix", ...)` |
| Reverse URL | `{% url "view_name" %}` native | `{% link "view_name" %}` → custom Tera function |
| Get Path Param | `kwargs['id']` | `form.cleaned_*` or `request.path_param("id")` |
| Get Query Param | `request.GET.get('key')` | `form.cleaned_*` or `request.from_url("key")` |

---

## Views / Handlers

| Feature | Django | Runique |
|---------|--------|---------|
| Function view | `def my_view(request)` | `async fn my_view(...)` |
| Class view | `class MyView(View)` | — |
| Session access | `request.session` | `request.session` via `context::template::Request` |
| DB access | `Model.objects.get(...)` | `Model::objects.get(...)` (via `impl_objects!`) or SeaORM query builders |
| Template render | `render(request, "template.html", ctx)` | `request.render("template.html")` — context already in `request.context` |
| Redirect | `redirect("url_name")` | `Redirect::to("/url")` or `reverse(&engine, "name")` (prelude) |
| Flash messages | `messages.success(request, "...")` | `success!(message => "...")` — `success!`, `error!`, `info!`, `warning!` macros |

---

## Forms

| Feature | Django | Runique |
|---------|--------|---------|
| Definition | `class MyForm(forms.Form)` / `class MyForm(ModelForm)` | `#[form]` struct (ModelForm equiv) or manual `RuniqueForm` |
| Validation | `form.is_valid()` | `form.is_valid().await` |
| Available Fields | CharField, EmailField, etc. | TextField, EmailField, PasswordField, HiddenField, ChoiceField, NumericField, BooleanField, FileField, DateTimeField, DurationField |
| HTML Rendering | `{{ form.as_p }}` | `{% form.my_form %}` (full) or `{% form.my_form.field %}` (individual) |
| CSRF included | automatic | automatic — injected before the first field |
| Save to DB | `form.save()` | `form.save(&db).await` (if using `#[form]`) |
| Data access | `form.cleaned_data['key']` | `form.cleaned_*("key")` (e.g., `string`, `i32`, `bool`, `uuid`, etc.) |
| Async validation | no | yes (direct DB access in `clean()`) |
| File forms | `FileField` | Native multipart with dimensions/format validation |

---

## Templates

| Feature | Django | Runique |
|---------|--------|---------|
| Engine | Django Template Language | Tera (Jinja2 / Django-like syntax) |
| Inheritance | `{% extends %}` / `{% block %}` | same with Tera |
| Static files | `{% load static %}` `{% static "file" %}` | `{% static "file" %}` native |
| Media files | `{{ MEDIA_URL }}file` | `{% media "file" %}` native |
| URL reverse | `{% url "name" %}` | `{% link "name" %}` |
| CSRF | `{% csrf_token %}` | `{% csrf %}` |
| Messages | `{% for m in messages %}` | `{% messages %}` |
| I18n | `{% trans "..." %}` | `{{ t("section.key") }}` or `{{ tf("...", ["var"]) }}` |

---

## ORM / Database

| Feature | Django | Runique |
|---------|--------|---------|
| ORM | Native Django ORM | SeaORM (Rust async) |
| Model definition | `class User(models.Model)` | Rust struct with annotations + `model!{}` macro |
| Auto migrations | yes (change detection) | `runique makemigrations` |
| Chainable QuerySet | `User.objects.filter(...).order_by(...)` | `User::objects.filter(...).order_by(...)` (via `impl_objects!`) |
| Relations | ForeignKey, ManyToMany, OneToOne | Standard SeaORM relations |
| Transactions | `with transaction.atomic()` | `db.transaction(...)` |
| Multi-DB | yes | PostgreSQL, MySQL, SQLite |
| NoSQL | via 3rd party | via 3rd party crates (e.g., `mongodb`) |

---

## Authentication

| Feature | Django | Runique |
|---------|--------|---------|
| Login / Logout | `authenticate()` + `login()` | `auth_login(...)`, `logout()` |
| Is Authenticated | `request.user.is_authenticated` | `is_authenticated(&session).await` |
| Current User | `request.user` | `CurrentUser` (injected via middleware) |
| Route protection | `@login_required` | `if !is_authenticated(&session).await { ... }` pattern |
| Sessions | native | tower-sessions (DB backend) |
| Brute force protection | `django-axes` (3rd party) | Native `LoginGuard` (auto lockout) |
| Password Hashing | PBKDF2 / Argon2 | Argon2 by default, multi-algo support |
| Account Activation | native (`auth`) | Integrated into password creation/reset flow |
| Reset password | native | `handle_forgot_password` + `handle_password_reset` native |
| Force logout | yes | `RuniqueSessionStore::invalidate_all(user_id)` |

---

## Security

| Feature | Django | Runique |
|---------|--------|---------|
| CSRF | native | native (constant-time validation) |
| CSP | `django-csp` (3rd party) | native (`use_nonce: true` by default) |
| HSTS | `SECURE_HSTS_SECONDS` | native |
| SameSite cookies | configurable | `Strict` by default |
| HttpOnly cookies | by default | always `true` |
| Rate limiting | `django-ratelimit` (3rd party) | Native `RateLimiter` |
| Input sanitization | — | Native `sanitize` middleware |
| Secret key | manual | auto-generated on `runique new` |

---

## Admin Interface

| Feature | Django | Runique |
|---------|--------|---------|
| Activation | `admin.site.register(Model)` | `admin!{}` macro |
| Full CRUD | native | native |
| List pagination | native | `.pagination(n)` in `DisplayConfig` |
| `list_display` | native | `.columns_include()` / `.columns_exclude()` |
| Search / filters | native | `.list_filter()` + auto search field |
| Custom templates | yes | yes (Tera hierarchy) |
| Permissions | per resource | Dynamic RBAC (Groups / Permissions) |

---

## Email

| Feature | Django | Runique |
|---------|--------|---------|
| Send mail | `send_mail()` native | `utils::Email::new().send()` native |
| Email templates | native | Tera templates supported via `html(body)` |
| SMTP Backend | configurable | configuration via `.env` |

---

## Internationalization

| Feature | Django | Runique |
|---------|--------|---------|
| Languages | unlimited | 9 default languages (compiled JSON) |
| Fallback | yes | yes (`Lang::En`) |
| `t("key")` | `_("...")` | `t("section.key")` |

---

## Performance & Deployment

| Aspect | Django | Runique |
|--------|--------|---------|
| Runtime | CPython (interpreted) | Tokio async Rust (compiled) |
| Memory usage | ~50–100 MB | ~5–15 MB |
| Compilation | — | single static binary |

---

## What Runique is still missing (compared to Django)

Runique is getting close to Django''s functional completeness, but some pillars are still being worked on:

- **Enhanced File Upload**: Server-side automatic image resizing/cropping.
- **Equivalent to `django-simple-history`**: Integrated audit log system to track row changes history.
- **Native NoSQL** (Still out of main scope, but simplified MongoDB integration is planned).
- `request.path_param()` / `request.query_param()` — currently via raw Axum extractors (see [roadmap](../../ROADMAP.md#4c-requestpath_param-et-requestquery_param)).
