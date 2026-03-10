# Admin template examples

Three approaches are available. Tera inheritance is **not required** — a plain HTML file works too, as long as the injected variables are used correctly.

---

## Approach 1 — Extend the default Runique layout

The most common case: keep the Runique admin layout and only customise the content.

```html
{# templates/my_theme/users_list.html #}
{% extends "admin/admin_base.html" %}

{% block title %}{{ resource.title }}{% endblock %}

{% block content %}
<h1>{{ resource.title }}</h1>
<p>{{ total }} {{ admin_list_entries_count }}</p>

{% for entry in entries %}
<div class="card">
    <span>#{{ entry.id }}</span>
    <span>{{ entry.username }}</span>
    <a href="/admin/{{ resource_key }}/{{ entry.id }}/edit">{{ admin_list_btn_edit }}</a>
</div>
{% endfor %}
{% endblock %}
```

Declaration in the builder (`src/main.rs`):

```rust
RuniqueApp::builder(config)
    .with_admin(|a| a
        .routes(admins::routes("/admin"))
        .with_state(admins::admin_state())
        .templates(|t| t
            .with_list("templates/my_theme/users_list.html")
        )
    )
    .build().await?
```

---

## Approach 2 — Extend a custom layout

You have created your own `template.html` (see [Template override](https://github.com/seb-alliot/runique/blob/main/docs/en/admin/template/surcharge/surcharge.md)) and use it as the base.

```html
{# templates.html #}

{% block content %}
<h1>{{ resource.title }}</h1>

{% if entries %}
<table>
    <tbody>
    {% for entry in entries %}
    <tr>
        <td>{{ entry.id }}</td>
        <td>{{ entry.username }}</td>
        <td>
            <a href="/admin/{{ resource_key }}/{{ entry.id }}/detail">{{ admin_list_btn_detail }}</a>
            <a href="/admin/{{ resource_key }}/{{ entry.id }}/delete" class="danger">
                {{ admin_list_btn_delete }}
            </a>
        </td>
    </tr>
    {% endfor %}
    </tbody>
</table>
{% else %}
<p>{{ admin_list_empty_title }}</p>
{% endif %}
{% endblock %}
```

Declaration in the builder (`src/main.rs`):

```rust
RuniqueApp::builder(config)
    .with_admin(|a| a
        .routes(admins::routes("/admin"))
        .with_state(admins::admin_state())
        .templates(|t| t
            .with_base("template.html")
            .with_list("templates/my_theme/users_list.html")
        )
    )
    .build().await?
```

---

## Approach 3 — Standalone HTML (no inheritance)

No `{% extends %}` required. The template is a complete HTML file. Useful for frontend framework integrations (Alpine.js, HTMX, etc.) or when the Runique admin layout is not desired.

```html
{# templates/my_theme/users_list.html #}
<!DOCTYPE html>
<html lang="{{ lang }}">
<head>
    <meta charset="UTF-8">
    <title>{{ resource.title }} — {{ site_title }}</title>
    <link rel="stylesheet" href="/static/css/my_theme.css">

    {# CSRF is required for any POST action from this page #}
    <meta name="csrf-token" content="{{ csrf_token }}">
    <script src="/static/js/csrf.js" defer></script>
</head>
<body>
    <nav>
        <strong>{{ site_title }}</strong>
        {% for res in resources %}
        <a href="/admin/{{ res.key }}/list"
           {% if res.key == current_resource %}class="active"{% endif %}>
            {{ res.title }}
        </a>
        {% endfor %}
    </nav>

    <main>
        <h1>{{ resource.title }}</h1>

        {# Flash messages #}
        {% messages %}

        <a href="/admin/{{ resource_key }}/create">{{ admin_list_btn_create }}</a>

        {% for entry in entries %}
        <div>{{ entry.id }} — {{ entry.username }}</div>
        {% endfor %}
    </main>
</body>
</html>
```

> If the template does not extend `admin_template.html`, CSRF elements are no longer injected automatically. You must add them manually (see above). See [CSRF](https://github.com/seb-alliot/runique/blob/main/docs/en/admin/template/csrf/csrf.md) for details.

---

## Summary

| Approach | When to use |
| --- | --- |
| Extend `admin_base.html` | Content customisation only |
| Extend a custom layout | Full admin layout redesign |
| Standalone HTML | Frontend integration, or no shared layout needed |

---

## Back to summary

| Section | Description |
| --- | --- |
| [Override](https://github.com/seb-alliot/runique/blob/main/docs/en/admin/template/surcharge/surcharge.md) | Principle and inheritance levels |
| [Context keys](https://github.com/seb-alliot/runique/blob/main/docs/en/admin/template/clef/context.md) | Variables available per view |
