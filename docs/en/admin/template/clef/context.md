# Runique Admin — Tera Variables per View

This document lists all variables available in the **Tera context** when a developer overrides an admin template.

---

## Global Variables (All Routes)

These variables are injected into **every route** by the framework's `Request` extractor, before any admin handler runs.

| Variable | Rust Type | Description |
|---|---|---|
| `debug` | `bool` | Whether debug mode is enabled |
| `csrf_token` | `String` | Masked CSRF token — include in every POST form |
| `csp_nonce` | `&str` | CSP nonce for `<script>` and `<style>` tags |
| `static_runique` | `StaticConfig` | Runique static assets config (see below) |
| `messages` | `Vec<FlashMessage>` | Flash messages from the current session |
| `current_user` | `CurrentUser` *(optional)* | Authenticated user data — absent if not logged in |

### `static_runique` Fields

```html
{{ static_runique.static_url }}   {# Base URL for assets, e.g. /static #}
{{ static_runique.static_dir }}   {# Physical directory on disk #}
```

---

## Variables Injected by CRUD Handlers

These variables are injected into **all CRUD admin views** via `inject_context`, after the base extractor.

| Variable | Type | Description |
|---|---|---|
| `site_title` | `String` | Site title configured in `AdminConfig` |
| `resource_key` | `&str` | Key of the current resource (e.g. `"users"`) |
| `current_resource` | `&str` | Same as `resource_key` |
| `resource` | `AdminResource` | Full metadata of the current resource (see below) |
| `resources` | `Vec<AdminResource>` | All resources registered in the registry |
| `lang` | `String` | Current language code (e.g. `"en"`) |

> Keys declared in `extra: {}` in `admin!{}` are also injected as **top-level Tera variables**.
> Example: `extra: { "icon" => "user" }` → `{{ icon }}` (direct access) AND `{{ resource.extra_context.icon }}`.

### `AdminResource` Structure

| Tera Field | Type | Description |
|---|---|---|
| `resource.key` | `&str` | Unique resource key (`"users"`) |
| `resource.title` | `&str` | Human-readable title (`"Users"`) |
| `resource.model_path` | `&str` | SeaORM model path (`"crate::entities::users::Model"`) |
| `resource.permissions.list` | `Vec<String>` | Roles allowed for list |
| `resource.permissions.view` | `Vec<String>` | Roles allowed for detail |
| `resource.permissions.create` | `Vec<String>` | Roles allowed for create |
| `resource.permissions.edit` | `Vec<String>` | Roles allowed for edit |
| `resource.permissions.delete` | `Vec<String>` | Roles allowed for delete |
| `resource.display.icon` | `String` *(optional)* | Icon name declared in config |
| `resource.display.pagination` | `usize` | Entries per page (default: `25`) |
| `resource.extra_context` | `HashMap<String, String>` | Custom keys declared in `extra: {}` |
| `resource.id_type` | `AdminIdType` | Primary key type (`I32`, `I64`, `Uuid`) |

---

## Global i18n Variables (All CRUD Views)

Automatically injected via `insert_admin_messages`. The Tera variable name is the i18n key with `.` replaced by `_`.

### `base` Section

| Tera Variable | i18n Key |
|---|---|
| `admin_base_title` | `admin.base.title` |
| `admin_base_breadcrumb` | `admin.base.breadcrumb` |
| `admin_base_toggle` | `admin.base.toggle` |
| `admin_base_logout_title` | `admin.base.logout_title` |

### `list` Section

| Tera Variable | i18n Key |
|---|---|
| `admin_list_breadcrumb_admin` | `admin.list.breadcrumb_admin` |
| `admin_list_entries_count` | `admin.list.entries_count` |
| `admin_list_btn_create` | `admin.list.btn_create` |
| `admin_list_th_id` | `admin.list.th_id` |
| `admin_list_th_actions` | `admin.list.th_actions` |
| `admin_list_bool_true` | `admin.list.bool_true` |
| `admin_list_bool_false` | `admin.list.bool_false` |
| `admin_list_btn_detail` | `admin.list.btn_detail` |
| `admin_list_btn_edit` | `admin.list.btn_edit` |
| `admin_list_btn_delete` | `admin.list.btn_delete` |
| `admin_list_confirm_delete` | `admin.list.confirm_delete` |
| `admin_list_empty_title` | `admin.list.empty_title` |
| `admin_list_empty_desc` | `admin.list.empty_desc` |
| `admin_list_btn_create_first` | `admin.list.btn_create_first` |

### `create` Section

| Tera Variable | i18n Key |
|---|---|
| `admin_create_title` | `admin.create.title` |
| `admin_create_breadcrumb` | `admin.create.breadcrumb` |
| `admin_create_card_info` | `admin.create.card_info` |
| `admin_create_no_fields` | `admin.create.no_fields` |
| `admin_create_btn_cancel` | `admin.create.btn_cancel` |
| `admin_create_btn_submit` | `admin.create.btn_submit` |

### `edit` Section

| Tera Variable | i18n Key |
|---|---|
| `admin_edit_title` | `admin.edit.title` |
| `admin_edit_breadcrumb` | `admin.edit.breadcrumb` |
| `admin_edit_card_info` | `admin.edit.card_info` |
| `admin_edit_no_fields` | `admin.edit.no_fields` |
| `admin_edit_btn_cancel` | `admin.edit.btn_cancel` |
| `admin_edit_btn_submit` | `admin.edit.btn_submit` |

### `detail` Section

| Tera Variable | i18n Key |
|---|---|
| `admin_detail_title` | `admin.detail.title` |
| `admin_detail_breadcrumb` | `admin.detail.breadcrumb` |
| `admin_detail_entry_label` | `admin.detail.entry_label` |
| `admin_detail_btn_list` | `admin.detail.btn_list` |
| `admin_detail_btn_edit` | `admin.detail.btn_edit` |
| `admin_detail_btn_delete` | `admin.detail.btn_delete` |
| `admin_detail_confirm_delete` | `admin.detail.confirm_delete` |

### `delete` Section

| Tera Variable | i18n Key |
|---|---|
| `admin_delete_title` | `admin.delete.title` |
| `admin_delete_breadcrumb` | `admin.delete.breadcrumb` |
| `admin_delete_heading` | `admin.delete.heading` |
| `admin_delete_btn_cancel` | `admin.delete.btn_cancel` |
| `admin_delete_btn_confirm` | `admin.delete.btn_confirm` |
| `admin_delete_warning_title` | `admin.delete.warning.title` |
| `admin_delete_warning_desc` | `admin.delete.warning.desc` |
| `admin_delete_warning_of` | `admin.delete.warning.of` |
| `admin_delete_warning_irreversible` | `admin.delete.warning.irreversible` |

---

## `login` View

**Route:** `GET /admin/login`

> `inject_context` is **not** called — variables `resource`, `resources`, and `resource_key` are not available.

| Variable | Type | Description |
|---|---|---|
| `site_title` | `String` | Site title |
| `lang` | `String` | Current language code |
| `csrf_token` | `String` | CSRF token (injected by the base extractor) |
| `admin_login_title` | `String` | i18n `admin.login.title` |
| `admin_login_subtitle` | `String` | i18n `admin.login.subtitle` |
| `admin_login_label_username` | `String` | i18n `admin.login.label_username` |
| `admin_login_label_password` | `String` | i18n `admin.login.label_password` |
| `admin_login_btn_submit` | `String` | i18n `admin.login.btn_submit` |
| `admin_login_error_session` | `String` | i18n `admin.login.error_session` |
| `admin_login_error_credentials` | `String` | i18n `admin.login.error_credentials` |
| `error` *(optional)* | `String` | Verbatim error message — injected only on POST failure |

> On POST failure, `base` section i18n keys are also injected.

**Minimal example:**

```html
{% extends "admin/admin_template.html" %}

{% block title %}{{ admin_login_title }}{% endblock %}

{% block content %}
<div class="login-container">
    <h1>{{ admin_login_title }}</h1>
    <p>{{ admin_login_subtitle }}</p>

    {% if error %}
    <div class="alert alert-danger">{{ error }}</div>
    {% endif %}

    <form method="POST" action="/admin/login">
        <input type="hidden" name="csrf_token" value="{{ csrf_token }}">
        <div>
            <label>{{ admin_login_label_username }}</label>
            <input type="text" name="username" required>
        </div>
        <div>
            <label>{{ admin_login_label_password }}</label>
            <input type="password" name="password" required>
        </div>
        <button type="submit">{{ admin_login_btn_submit }}</button>
    </form>
</div>
{% endblock %}
```

> The login template extends `admin_template.html` (empty blocks contract), not `admin_base.html` — sidebar and topbar must not appear on the login page.

---

## `dashboard` View

**Route:** `GET /admin/`

> `inject_context` is **not** called. `current_resource` is explicitly `None`.

| Variable | Type | Description |
|---|---|---|
| `site_title` | `String` | Site title |
| `lang` | `String` | Current language code |
| `resources` | `Vec<AdminResource>` | All registered resources |
| `resource_counts` | `HashMap<String, u64>` | Entry count per resource (key = `resource.key`) |
| `current_page` | `&str` | Value `"dashboard"` |
| `current_resource` | `None` | Not set — no resource selected |
| `admin_base_*` | — | i18n `base` section keys (see above) |
| `admin_dashboard_title` | `String` | i18n `admin.dashboard.title` |
| `admin_dashboard_subtitle` | `String` | i18n `admin.dashboard.subtitle` |
| `admin_dashboard_card_resources` | `String` | i18n `admin.dashboard.card_resources` |
| `admin_dashboard_th_resource` | `String` | i18n `admin.dashboard.th_resource` |
| `admin_dashboard_th_key` | `String` | i18n `admin.dashboard.th_key` |
| `admin_dashboard_th_permissions` | `String` | i18n `admin.dashboard.th_permissions` |
| `admin_dashboard_th_actions` | `String` | i18n `admin.dashboard.th_actions` |
| `admin_dashboard_btn_list` | `String` | i18n `admin.dashboard.btn_list` |
| `admin_dashboard_btn_create` | `String` | i18n `admin.dashboard.btn_create` |
| `admin_dashboard_see_list` | `String` | i18n `admin.dashboard.see_list` |
| `admin_dashboard_empty_title` | `String` | i18n `admin.dashboard.empty_title` |
| `admin_dashboard_empty_desc` | `String` | i18n `admin.dashboard.empty_desc` |

**Minimal example:**

```html
{% extends "admin/admin_base.html" %}

{% block title %}{{ admin_dashboard_title }}{% endblock %}

{% block content %}
<h1>{{ admin_dashboard_title }}</h1>
<p>{{ admin_dashboard_subtitle }}</p>

{% if resources %}
<h2>{{ admin_dashboard_card_resources }}</h2>
<table class="table">
    <thead>
        <tr>
            <th>{{ admin_dashboard_th_resource }}</th>
            <th>{{ admin_dashboard_th_key }}</th>
            <th>{{ admin_dashboard_th_permissions }}</th>
            <th>{{ admin_dashboard_th_actions }}</th>
        </tr>
    </thead>
    <tbody>
        {% for res in resources %}
        <tr>
            <td>{{ res.title }}</td>
            <td><code>{{ res.key }}</code></td>
            <td>{{ res.permissions.list | join(sep=", ") }}</td>
            <td>
                <a href="/admin/{{ res.key }}/list">
                    {{ admin_dashboard_btn_list }}
                    {% if resource_counts[res.key] %}({{ resource_counts[res.key] }}){% endif %}
                </a>
                <a href="/admin/{{ res.key }}/create">{{ admin_dashboard_btn_create }}</a>
            </td>
        </tr>
        {% endfor %}
    </tbody>
</table>
{% else %}
<p>{{ admin_dashboard_empty_title }}</p>
<p>{{ admin_dashboard_empty_desc }}</p>
{% endif %}
{% endblock %}
```

---

## `list` View

**Route:** `GET /admin/{resource}/list`

| Variable | Type | Description |
|---|---|---|
| `entries` | `Vec<Value>` | Records serialized as JSON |
| `total` | `usize` | Total number of entries |
| `current_page` | `&str` | Value `"list"` |

> The i18n variables for the `list` section are listed in the global variables above.

**Minimal example:**

```html
{% extends "admin/admin_base.html" %}

{% block title %}{{ resource.title }}{% endblock %}

{% block content %}
<div class="d-flex justify-content-between align-items-center mb-3">
    <h1>{{ resource.title }}
        <small class="text-muted fs-6">{{ total }} {{ admin_list_entries_count }}</small>
    </h1>
    <a href="/admin/{{ resource_key }}/create" class="btn btn-primary">
        {{ admin_list_btn_create }}
    </a>
</div>

{% if entries %}
<table class="table">
    <thead>
        <tr>
            <th>{{ admin_list_th_id }}</th>
            {# add columns based on the model #}
            <th>{{ admin_list_th_actions }}</th>
        </tr>
    </thead>
    <tbody>
        {% for entry in entries %}
        <tr>
            <td>{{ entry.id }}</td>
            <td>
                <a href="/admin/{{ resource_key }}/{{ entry.id }}/detail">{{ admin_list_btn_detail }}</a>
                <a href="/admin/{{ resource_key }}/{{ entry.id }}/edit">{{ admin_list_btn_edit }}</a>
                <a href="/admin/{{ resource_key }}/{{ entry.id }}/delete">{{ admin_list_btn_delete }}</a>
            </td>
        </tr>
        {% endfor %}
    </tbody>
</table>
{% else %}
<p>{{ admin_list_empty_title }}</p>
<p>{{ admin_list_empty_desc }}</p>
<a href="/admin/{{ resource_key }}/create">{{ admin_list_btn_create_first }}</a>
{% endif %}
{% endblock %}
```

---

## `create` View

**Route:** `GET /admin/{resource}/create`

| Variable | Type | Description |
|---|---|---|
| `form_fields` | `Forms` | Form generated by Prisme — rendered via `{% form.field_name %}` or `form_fields.html` |
| `is_edit` | `bool` | Value `false` |

> The i18n variables for the `create` section are listed in the global variables above.

**Minimal example:**

```html
{% extends "admin/admin_base.html" %}

{% block title %}{{ admin_create_title }} — {{ resource.title }}{% endblock %}

{% block content %}
<h1>{{ admin_create_title }} — {{ resource.title }}</h1>
<p class="text-muted">{{ admin_create_card_info }}</p>

<form method="POST" action="/admin/{{ resource_key }}/create">
    {# csrf.js handles the token automatically for admin forms #}
    {% if form_fields.html %}
        {{ form_fields.html | safe }}
    {% else %}
        <p>{{ admin_create_no_fields }}</p>
    {% endif %}
    <button type="submit" class="btn btn-primary">{{ admin_create_btn_submit }}</button>
    <a href="/admin/{{ resource_key }}/list" class="btn btn-secondary">{{ admin_create_btn_cancel }}</a>
</form>
{% endblock %}
```

---

## `edit` View

**Route:** `GET /admin/{resource}/{id}/edit`

| Variable | Type | Description |
|---|---|---|
| `form_fields` | `Forms` | Form pre-filled with existing data |
| `is_edit` | `bool` | Value `true` |
| `object_id` | `String` | ID of the entry being edited |

> The i18n variables for the `edit` section are listed in the global variables above.

**Minimal example:**

```html
{% extends "admin/admin_base.html" %}

{% block title %}{{ admin_edit_title }} — {{ resource.title }} #{{ object_id }}{% endblock %}

{% block content %}
<h1>{{ admin_edit_title }} — {{ resource.title }} <small>#{{ object_id }}</small></h1>
<p class="text-muted">{{ admin_edit_card_info }}</p>

<form method="POST" action="/admin/{{ resource_key }}/{{ object_id }}/edit">
    {% if form_fields.html %}
        {{ form_fields.html | safe }}
    {% else %}
        <p>{{ admin_edit_no_fields }}</p>
    {% endif %}
    <button type="submit" class="btn btn-primary">{{ admin_edit_btn_submit }}</button>
    <a href="/admin/{{ resource_key }}/list" class="btn btn-secondary">{{ admin_edit_btn_cancel }}</a>
</form>
{% endblock %}
```

---

## `detail` View

**Route:** `GET /admin/{resource}/{id}/detail`

| Variable | Type | Description |
|---|---|---|
| `entry` | `Value` *(optional)* | Record serialized as JSON — absent if `get_fn` is not configured |
| `object_id` | `String` | Entry ID |

> The i18n variables for the `detail` section are listed in the global variables above.

**Minimal example:**

```html
{% extends "admin/admin_base.html" %}

{% block title %}{{ admin_detail_title }} — {{ resource.title }} #{{ object_id }}{% endblock %}

{% block content %}
<h1>{{ admin_detail_title }} — {{ resource.title }} <small>#{{ object_id }}</small></h1>

{% if entry %}
<dl class="row">
    {% for key, value in entry %}
    <dt class="col-sm-3">{{ key }}</dt>
    <dd class="col-sm-9">{{ value }}</dd>
    {% endfor %}
</dl>
{% endif %}

<a href="/admin/{{ resource_key }}/list" class="btn btn-secondary">{{ admin_detail_btn_list }}</a>
<a href="/admin/{{ resource_key }}/{{ object_id }}/edit" class="btn btn-primary">{{ admin_detail_btn_edit }}</a>
<a href="/admin/{{ resource_key }}/{{ object_id }}/delete" class="btn btn-danger">{{ admin_detail_btn_delete }}</a>
{% endblock %}
```

---

## `delete` View

**Route:** `GET /admin/{resource}/{id}/delete`

| Variable | Type | Description |
|---|---|---|
| `entry` | `Value` *(optional)* | Record serialized as JSON — absent if `get_fn` is not configured |
| `object_id` | `String` | ID of the entry to delete |

> The i18n variables for the `delete` section are listed in the global variables above.

**Minimal example:**

```html
{% extends "admin/admin_base.html" %}

{% block title %}{{ admin_delete_title }} — {{ resource.title }} #{{ object_id }}{% endblock %}

{% block content %}
<h1>{{ admin_delete_heading }}</h1>

<div class="alert alert-danger">
    <strong>{{ admin_delete_warning_title }}</strong>
    <p>{{ admin_delete_warning_desc }}</p>
    {% if entry %}
    <p>{{ admin_delete_warning_of }} <strong>#{{ object_id }}</strong></p>
    {% endif %}
    <p>{{ admin_delete_warning_irreversible }}</p>
</div>

<form method="POST" action="/admin/{{ resource_key }}/{{ object_id }}/delete">
    <button type="submit" class="btn btn-danger">{{ admin_delete_btn_confirm }}</button>
    <a href="/admin/{{ resource_key }}/list" class="btn btn-secondary">{{ admin_delete_btn_cancel }}</a>
</form>
{% endblock %}
```

---

## Recommended Pattern for i18n Variables

```html
{{ admin_create_title }}
```

> Do not use `| default(value="...")` — i18n variables are always present when a language is configured.

---

## Overriding a Template

Using `.templates()` in the builder replaces the template for **all resources**:

```rust
RuniqueApp::builder(config)
    .with_admin(|a| a
        .templates(|t| t
            .with_list("my_theme/list")
            .with_create("my_theme/create")
            .with_edit("my_theme/edit")
            .with_detail("my_theme/detail")
            .with_delete("my_theme/delete")
            .with_dashboard("my_theme/dashboard")
            .with_login("my_theme/login")
            .with_base("my_theme/admin_base")
        )
    )
    .build().await?
```

Overridden templates have access to the **same variables as the default templates**.

## Sub-sections

| Section | Description |
| --- | --- |
| [Override](https://github.com/seb-alliot/runique/blob/main/docs/en/admin/template/surcharge/surcharge.md) | Replace the layout or a CRUD component |
| [CSRF](https://github.com/seb-alliot/runique/blob/main/docs/en/admin/template/csrf/csrf.md) | CSRF token, `csrf.js`, custom login checklist |

## Back to summary

| Section | Description |
| --- | --- |
| [Template summary](https://github.com/seb-alliot/runique/blob/main/docs/en/admin/template/templates.md) | Admin templates |
| [Admin summary](https://github.com/seb-alliot/runique/blob/main/docs/en/admin/11-Admin.md) | Admin |
