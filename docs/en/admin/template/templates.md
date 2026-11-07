# Admin template system

## Hierarchy

```
admin/admin_template.html   ← contract: <head>, CSRF meta, csrf.js, modal
        ↓ extends
admin_base.html             ← layout: stylesheets, sidebar, top bar
        ↓ extends
admin/list.html, admin/create.html …   ← the pages, made of blocks
```

**`admin/admin_template.html`**: the HTML skeleton and what must always be there. Not replaceable.

**`admin_base.html`**: the layout. It isn't replaced in one go: its sidebar and top bar change through their blocks (`GlobalBlock::Sidebar`, `GlobalBlock::Topbar`), its stylesheets through the CSS overrides.

**The pages**: each is made of replaceable blocks (`{% dev_block "name" %}`). You replace a block, or the whole page, for every model or for one. See [Template override](/docs/en/admin/template-surcharge).

---

## Tera blocks of the skeleton

A page extending `admin_base.html` (or `admin/admin_template.html`) can redefine:

| Block | Role |
| --- | --- |
| `{% block lang %}` | The `<html lang>` tag |
| `{% block title %}` | Page title (`<title>`) |
| `{% block extra_css %}` | Stylesheets in `<head>`: `admin_base.html` writes the page's there, keep `{{ super() }}` |
| `{% block layout %}` | The whole layout (sidebar + main area) |
| `{% block sidebar %}` | Sidebar |
| `{% block topbar %}` | Top bar (holds `breadcrumb`) |
| `{% block breadcrumb %}` | Breadcrumb |
| `{% block messages %}` | Flash messages, holds `{% messages %}` |
| `{% block content %}` | Page content |
| `{% block extra_js %}` | Scripts before `</body>`: `admin_base.html` puts the admin's there, keep `{{ super() }}` |
| `{% block footer %}` | Footer |

### Outside blocks (always present)

Written directly in `admin/admin_template.html`, no override removes them (except a page extending nothing):

- `<meta name="csrf-token" content="{{ csrf_token }}">` and `<meta name="robots" content="noindex, nofollow">` in `<head>`
- `<script {% csp %} src="{% runique_static "js/csrf.js" %}" defer></script>` before `</body>`
- the confirmation modal

---

## Sub-sections

| Section | Description |
| --- | --- |
| [Context keys](/docs/en/admin/template-clef) | Variables injected by the backend into each template |
| [Override](/docs/en/admin/template-surcharge) | Replace a block (HTML, CSS or both) or a whole page |

## See also

| Section | Description |
| --- | --- |
| [Setup](/docs/en/admin/setup) | Wire the admin into an existing project, create a superuser |
| [Declaration](/docs/en/admin/declaration) | `AdminStaging`, `AdminSite`, `ModelAdmin` workflow |
| [Permissions](/docs/en/admin/permission) | Ranks, rights per account and per group, checked on every request |
| [Roadmap](/docs/en/admin/evolution) | Planned features and beta status |

## Back to menu

- [Admin Summary](/docs/en/admin)
