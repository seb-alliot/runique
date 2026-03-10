# Admin template override

Runique's admin allows replacing any template with a custom one,
while preserving the contractual elements (CSRF, flash messages).

---

## Principle: 3 inheritance levels

```
admin_template.html   ← level 1: Runique contract (do not touch)
        ↓ extends
admin_base.html            ← level 2: default layout (can be replaced)
        ↓ extends
list.html / create.html …  ← level 3: CRUD components (can be replaced)
```

The developer can replace level 2 (global layout) and/or level 3 (individual pages).

---

## Override the global layout (`admin_base.html`)

Create a file that extends `admin_template` and fills the layout blocks.

### `templates/my_theme/admin_base.html`

```html
{% extends "admin/admin_template" %}

{% block extra_css %}
    <link rel="stylesheet" href="{{ "css/my_theme.css" | runique_static }}">
{% endblock %}

{% block sidebar %}
<nav class="my-sidebar">
    <h2>{{ site_title }}</h2>
    <ul>
    {% for res in resources %}
        <li>
            <a href="/admin/{{ res.key }}/list"
               {% if res.key == current_resource %}class="active"{% endif %}>
                {{ res.title }}
            </a>
        </li>
    {% endfor %}
    </ul>
</nav>
{% endblock %}

{% block topbar %}
<header class="my-topbar">
    {% block breadcrumb %}{% endblock %}
    <form method="POST" action="/admin/logout">
        <button type="submit">{{ current_user.username }} — Logout</button>
    </form>
</header>
{% endblock %}

{% block extra_js %}
    <script src="{{ "js/my_admin.js" | runique_static }}" defer></script>
{% endblock %}
```

> **Note**: `admin/admin_template` is the Tera key, not a file path.

---

## Declare the custom template in the config

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

---

## Override a specific CRUD component

To replace only the users list page:

### `templates/my_theme/users_list.html`

```html
{% extends "my_theme/admin_base" %}

{% block content %}
<h1>{{ resource.title }}</h1>
<p>{{ total }} entry(ies)</p>

{% for entry in entries %}
    <div class="user-card">
        <span>#{{ entry.id }}</span>
        <span>{{ entry.username }}</span>
    </div>
{% endfor %}
{% endblock %}
```
---

## Available blocks for override

| Block | Default content | Can be overridden |
| --- | --- | --- |
| `{% block title %}` | Page title | Yes |
| `{% block extra_css %}` | Runique admin CSS | Yes |
| `{% block layout %}` | Full layout (sidebar + main) | Yes (advanced) |
| `{% block sidebar %}` | Sidebar with resource navigation | Yes |
| `{% block topbar %}` | Topbar with breadcrumb + logout | Yes |
| `{% block breadcrumb %}` | Breadcrumb | Yes (from admin_base) |
| `{% block messages %}` | `{% messages %}` | Yes — keep `{% messages %}` |
| `{% block content %}` | CRUD page content | Yes |
| `{% block extra_js %}` | `admin.js` | Yes — use `{{ super() }}` to accumulate |

---

## Points of attention

- `current_resource` is a **String** (the key, e.g. `"users"`), not an object. Use `resource.key` and `resource.title` for metadata.
- If `{% block extra_js %}` is overridden, call `{{ super() }}` to avoid losing `admin.js`.
- Elements outside blocks (`<meta csrf-token>`, `<script csrf.js>`) are guaranteed by `admin_template` — they cannot be removed by overriding.
- If `{% block messages %}` is redefined, **keep** `{% messages %}` inside it.

## Sub-sections

| Section | Description |
| --- | --- |
| [Examples](https://github.com/seb-alliot/runique/blob/main/docs/en/admin/template/surcharge/examples.md) | 3 approaches: Runique inheritance, custom layout, standalone HTML |
| [Context keys](https://github.com/seb-alliot/runique/blob/main/docs/en/admin/template/clef/context.md) | Variables injected by the backend into each template |
| [CSRF](https://github.com/seb-alliot/runique/blob/main/docs/en/admin/template/csrf/csrf.md) | CSRF token, `csrf.js`, custom login checklist |

## Back to summary

| Section | Description |
| --- | --- |
| [Template summary](https://github.com/seb-alliot/runique/blob/main/docs/en/admin/template/templates.md) | Admin templates |
| [Admin summary](https://github.com/seb-alliot/runique/blob/main/docs/en/admin/11-Admin.md) | Admin |
