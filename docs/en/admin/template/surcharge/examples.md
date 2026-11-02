# Admin override examples

Complete cases, lightest first. Declarations go in `main.rs` (for every model) or on the model's `ModelAdmin` (for one). See [Template override](/docs/en/admin/template-surcharge) for the rules, and the [block reference](/docs/en/admin/template-surcharge-blocks) for the keys.

```rust
use runique::admin::template::{Block, GlobalBlock, Page};
```

---

## 1. The CSS only: restyle every list's table

The table's HTML stays Runique's. `ListTable` has a stylesheet in Runique (`css/admin/blocks/list.css`): yours replaces it, so start from a copy of Runique's.

```rust
.with_admin(|a| a
    .templates(|t| t.block_stylesheet(Block::ListTable, "css/admin/list.css"))
    .site(site)
)
```

```css
/* static/css/admin/list.css: copy of css/admin/blocks/list.css, modified */
.admin-list__cards { gap: 0.5em; }
```

---

## 2. The template only, for one model: the article form's buttons

Only the articles' create and edit forms change; the other models keep Runique's.

```rust
fn article() -> ModelAdmin<article::Entity> {
    ModelAdmin::new()
        .templates(|t| t.block(Block::FormActions, "articles/form_actions.html"))
}
```

```html
{# templates/articles/form_actions.html: included inside Runique's <form> #}
<div class="admin-form__actions">
  <p class="admin-page__subtitle">A saved article is published right away.</p>
  <a href="{{ cancel_url }}" class="btn btn-secondary">{{ cancel_label }}</a>
  <button type="submit" class="btn btn-primary">{{ submit_label }}</button>
</div>
```

The block is included inside the page's `<form>`: the CSRF token and the fields are already there, the `submit` button is enough.

---

## 3. Both: a sidebar of your own

```rust
.templates(|t| t
    .global_block(GlobalBlock::Sidebar, "my_theme/sidebar.html")
    .global_block_stylesheet(GlobalBlock::Sidebar, "css/admin/sidebar.css")
)
```

```html
{# templates/my_theme/sidebar.html #}
<aside class="admin-sidebar" id="adminSidebar">
  <nav class="my-nav">
    <a href="{{ nav.dashboard }}">{{ admin_base_dashboard }}</a>
    {% for resource in resources %}
      <a href="{{ resource.list_url }}"
         {% if current_resource == resource.key %}class="active"{% endif %}>
        {{ resource.title }}
      </a>
    {% endfor %}
    {% if nav.history %}<a href="{{ nav.history }}">{{ admin_base_history }}</a>{% endif %}
  </nav>
</aside>
```

`resources` holds only the models the signed-in user may see: no filter to write. Keep `id="adminSidebar"` if you want the mobile menu button to keep opening the sidebar (`admin.js` looks it up by that id).

---

## 4. A page extending Runique's

To change the articles list's header and keep everything else, the page extends `admin/list.html` and redefines its Tera block `list_header`:

```rust
ModelAdmin::new()
    .templates(|t| t.page(Page::List, "articles/list.html"))
```

```html
{# templates/articles/list.html #}
{% extends "admin/list.html" %}

{% block list_header %}
<div class="admin-page__header">
  <h1 class="admin-page__title">{{ resource.title }}</h1>
  <p class="admin-page__subtitle">{{ total }} article(s), newest first.</p>
  {% if resource_links.create %}
    <a href="{{ resource_links.create }}" class="btn btn-primary">{{ admin_list_btn_create }}</a>
  {% endif %}
</div>
{% endblock %}
```

The page's `dev_block` blocks (table, filters, pagination…) still follow your block overrides.

---

## 5. A whole page on Runique's layout

The page extends `admin_base.html`: stylesheets, sidebar, top bar and CSRF stay Runique's, the content is entirely yours.

```rust
.templates(|t| t
    .with_dashboard("my_theme/dashboard.html")
    .page_stylesheet(Page::Dashboard, "css/admin/dashboard.css")
)
```

```html
{# templates/my_theme/dashboard.html #}
{% extends "admin_base.html" %}

{% block title %}{{ site_title }}{% endblock %}

{% block content %}
<h1>{{ site_title }}</h1>
<ul class="my-board">
  {% for resource in resources %}
    <li><a href="{{ resource.list_url }}">{{ resource.title }}</a></li>
  {% endfor %}
</ul>
{% endblock %}
```

With `page_stylesheet`, the page loads Runique's variables then `css/admin/dashboard.css`, and none of Runique's other stylesheets. Without it, it keeps Runique's.

---

## 6. A standalone page (no inheritance)

No `{% extends %}`: useful to integrate another frontend tool. What `admin/admin_template.html` brought must then be written by hand: the CSRF meta, `csrf.js` (POST actions need it) and the `{% csp %}` attribute on every script.

```html
{# templates/my_theme/list.html #}
<!DOCTYPE html>
<html lang="{{ lang }}">
<head>
  <meta charset="UTF-8">
  <meta name="robots" content="noindex, nofollow">
  <meta name="csrf-token" content="{{ csrf_token }}">
  <title>{{ resource.title }} — {{ site_title }}</title>
  <link rel="stylesheet" href="{% static "css/admin/my_admin.css" %}">
</head>
<body>
  <nav>
    {% for res in resources %}
      <a href="{{ res.list_url }}">{{ res.title }}</a>
    {% endfor %}
  </nav>
  <main>
    {% messages %}
    <h1>{{ resource.title }}</h1>
    {% for entry in entries %}
      <p><a href="{{ entry.links.detail }}">#{{ entry.id }}</a></p>
    {% endfor %}
  </main>
  <script {% csp %} src="{% runique_static "js/csrf.js" %}" defer></script>
</body>
</html>
```

---

## Summary

| Need | Declaration |
| --- | --- |
| Change the colors | theme stylesheet, see [CSS theme](/docs/en/admin/template-surcharge-blocks) |
| Restyle a block | `.block_stylesheet(Block::…, css)` / `.global_block_stylesheet(GlobalBlock::…, css)` |
| Change a block's HTML | `.block(Block::…, html)` / `.global_block(GlobalBlock::…, html)` |
| The same, for one model | `.templates(…)` on the model's `ModelAdmin` |
| Change part of a page | a page extending Runique's (`{% extends "admin/list.html" %}`) |
| Redo a page | `.with_list(…)` etc., or `.page(Page::…, …)` for one model, with `page_stylesheet` if needed |

---

## Back to summary

| Section | Description |
| --- | --- |
| [Override](/docs/en/admin/template-surcharge) | Rules and declarations |
| [Context keys](/docs/en/admin/template-clef) | Variables available per view |
