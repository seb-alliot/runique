# Comparison: Runique vs Django

## CLI

| Command | Django | Runique |
|---------|--------|---------|
| Create project | `django-admin startproject name` | `runique new name` |
| Create app | `python manage.py startapp name` | — |
| Migrations (generate) | `python manage.py makemigrations` | `runique makemigrations` |
| Migrations (apply) | `python manage.py migrate` | `runique migration up` |
| Migrations (rollback) | `python manage.py migrate app 0001` | `runique migration down --files ...` |
| Migration status | — | `runique migration status` |
| Create superuser | `python manage.py createsuperuser` | `runique create-superuser` |
| Start | `python manage.py runserver` | `cargo run` — `runique start` to (re)generate the admin panel |

---

## Routing

| Feature | Django | Runique |
|---------|--------|---------|
| URL declaration | `urls.py` with `path()` | `url.rs` with `urlpatterns!{}` macro |
| Dynamic routes | `path('users/<int:id>/', view)` | `"/users/{id}"` in `urlpatterns!` |
| Namespaces | `app_name` + `include()` | `Router::new().nest("/prefix", ...)` |
| Reverse URL | `{% url "view_name" %}` native | `{% link "view_name" %}` → custom Tera function |
| Typed path param | `kwargs['id']` (always str in Django) | `request.get_path_as::<i32>("id")` |
| Raw path param | `kwargs['id']` | `request.get_path("id")` |
| Single query param | `request.GET.get('key')` | `request.get_query("key")` |
| Full query string | `request.GET` | `request.query::<MyStruct>()` (deserializes to `Deserialize` struct) |
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
| Definition | `class MyForm(ModelForm)` | `#[form]` struct or manual `RuniqueForm` |
| Validation | `form.is_valid()` | `form.is_valid().await` |
| Available fields | CharField, EmailField, FileField, etc. | TextField, EmailField, PasswordField, HiddenField, ChoiceField, NumericField, BooleanField, FileField, DateField, TimeField, DateTimeField, DurationField, PhoneField |
| HTML rendering | `{{ form.as_p }}` | `{% form.my_form %}` (full) or `{% form.my_form.field %}` |
| CSRF included | automatic | automatic — injected before the first field |
| Save to DB | `form.save()` | `form.save(&db).await` (if using `#[form]`) |
| Data access | `form.cleaned_data['key']` | `form.cleaned_string("key")`, `form.cleaned_i32(...)`, etc. |
| Async validation | no | yes (DB access in `clean()`) |
| Cross-field validation | `clean()` | `clean()` async |
| File fields | `FileField` | `FileField` native multipart with type/size validation |
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
| I18n | `{% trans "..." %}` | `{{ t("section.key") }}` / `{{ tf("...", ["var"]) }}` |

---

## ORM / Database

| Feature | Django | Runique |
|---------|--------|---------|
| ORM | Native Django ORM | SeaORM (Rust async) |
| Model definition | `class User(models.Model)` | `model!{}` macro (v1 SQL types or v2 semantic types) |
| Auto migrations | yes (change detection) | `runique makemigrations` |
| Chainable QuerySet | `User.objects.filter(...).order_by(...)` | `User::objects.filter(...).order_by(...)` |
| Strict `.get()` | raises `MultipleObjectsReturned` | `.one()` — returns `Err` if multiple rows |
| Random ordering | `order_by('?')` | `.order_by_random()` |
| Expression ordering | — | `.order_by_expr(expr, order)` |
| Relations | ForeignKey, ManyToMany, OneToOne | Standard SeaORM relations |
| Transactions | `with transaction.atomic()` | `db.transaction(...)` |
| Multi-engine SQL | yes | PostgreSQL, MySQL, SQLite |
| Secondary connections | `DATABASES` multiple entries | `.with_custom_db::<T>()` × N types (TypeMap) |
| Framework table extension | — | `extend!{}` — `ALTER TABLE ADD COLUMN` on `eihwaz_*` tables |

---

## Authentication

| Feature | Django | Runique |
|---------|--------|---------|
| Login / Logout | `authenticate()` + `login()` | `auth_login(...)`, `logout()` |
| Is authenticated | `request.user.is_authenticated` | `is_authenticated(&session).await` |
| Current user | `request.user` | `CurrentUser` (injected via middleware) |
| Route protection | `@login_required` | `if !is_authenticated(...).await { redirect }` |
| Sessions | native | tower-sessions (MemoryStore + DB fallback) |
| Brute force protection | `django-axes` (3rd party) | Native `LoginGuard` (auto lockout) |
| Password hashing | PBKDF2 / Argon2 | Argon2 by default |
| Password reset | native | native — email template customizable via `.email_template("...")` |
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
| Open Redirect | — | native — all 3xx responses validated (slot 25) |
| CORS | `django-cors-headers` (3rd party) | native via `.with_cors(...)` |
| Permissions-Policy | — | native — secure preset by default |
| Trusted Proxies / XFF | `SECURE_PROXY_SSL_HEADER` (partial) | native — full XFF chain validation, RFC 1918 preset |
| Secret key | manual | auto-generated on `runique new` |

---

## Admin Panel

| Feature | Django | Runique |
|---------|--------|---------|
| Registration | `admin.site.register(Model)` | `admin!{}` macro |
| Full CRUD | native | native |
| List pagination | native | `.page_size(n)` (list + history) |
| `list_display` | native | `list_display: [["col", "Label"], ...]` |
| FK resolution in list | — | 3rd element: `["fk_id", "Label", "table.column"]` |
| Search / filters | native | `list_filter` + automatic full-text SQL search — direct columns only, filters not combinable (UI) |
| Group actions | `actions` | `group_action` — bool (2 elements) or enum (3 elements, exact value) |
| Bulk create | — | `bulk_create: field` — comma-split, inserts N records |
| Bulk edit | — | native bulk edit on multi-row selection |
| M2M relations | `filter_horizontal` / `ManyRelatedField` | `m2m: [...]` — junction table, automatic diff |
| Custom admin routes | `get_urls()` | `.extra_routes(vec![...])` |
| Custom templates | yes | yes (Tera hierarchy) |
| Permissions | per resource | Dynamic RBAC (Groups / Scoped permissions) |
| Change history | `django-simple-history` (3rd party) | native history (created/modified/deleted) with field diff |
| Builtin config | — | `configure {}` block in `admin!{}` |
| Optimistic locking | — | native — detects concurrent edits (`__original_updated_at`) |
| Admin login brute-force protection | `django-axes` (3rd party) | native — `LoginGuard` + rate limiter on admin login |
| Account creation email | 3rd party | native — `inject_password: true` generates temporary password + email |
| Partial list refresh | — | native — `list_partial` HTMX without full page reload |
| Fieldsets | `fieldsets` | — |
| Read-only fields | `readonly_fields` | — |
| Date hierarchy | `date_hierarchy` | — |
| Export | native CSV | — |
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

## Internationalization

| Feature | Django | Runique |
|---------|--------|---------|
| Languages | unlimited | 9 default languages (compiled JSON) |
| Fallback | yes | yes (`Lang::En`) |
| Translation | `_("...")` | `t("section.key")` / `tf("...", ["var"])` |

---

## Performance & Deployment

| Aspect | Django | Runique |
|--------|--------|---------|
| Runtime | CPython (interpreted) | Tokio async Rust (compiled) |
| Memory usage | ~50–100 MB | ~5–15 MB |
| Compilation | — | single static binary |
| ACME / auto-TLS | `certbot` (external) | native via `acme` feature |

---

## What Runique is still missing (compared to Django)

- **Image resizing**: no server-side auto resize/cropping.
- **Generic public CRUD views**: no equivalent to Django's `DetailView`, `ListView`, `CreateView` for public views — planned via `crud!{}` (in development). The admin panel covers backoffice CRUD via `admin!{}`.
- **Model signals / hooks**: `before_save`, `after_save`, `before_delete`, `after_delete` — infrastructure in place, generator integration in progress.
- **Management commands**: no equivalent to `manage.py custom_command` — one-shot operations go through `src/bin/`.
- **Built-in test client**: no native HTTP test client — use `reqwest` or `axum::test`.
- **Fixtures**: no `loaddata`/`dumpdata` — seeds are plain Rust functions.
- **Admin inline**: no editing of related objects directly inside the parent form.
- **Admin custom group actions**: `group_action` only supports SQL column updates (`GroupAction::bool` and `GroupAction::val`) — no arbitrary Rust logic on the selection (Django equivalent: `actions` with any callable).
- **Admin combinable filters**: clicking a filter value resets other active column filters — the backend supports multiple simultaneous filters (`Vec`), but the generated template links do not preserve filters from other columns.
- **Admin FK filters**: `list_filter` supports direct columns only — no relation traversal (`article__author__name`).
- **Admin fieldsets**: no field grouping by section in admin forms (`fieldsets` in Django).
- **Admin readonly fields**: no `readonly_fields` — non-editable fields must be excluded from the form.
- **Admin date hierarchy**: no `date_hierarchy` to filter the list by year/month/day.
- **Admin export**: no native CSV/JSON export from the list view.
- **Admin advanced save buttons**: no "Save and continue editing" / "Save and add another".
- **Full i18n**: `t()`/`tf()` available, but no pluralization or Tera template translation.
- **Sitemap / RSS**: not built-in.
- **Third-party authentication**: OAuth / OIDC structured (Google, Microsoft, Apple, LDAP, SAML) but flow not implemented — stub only. JWT and API key auth absent.
- **Security observability**: CSP `report-uri`/`report-to` absent — violations are not collected. Auth event log (successful/failed logins, lockouts) not stored in DB.
