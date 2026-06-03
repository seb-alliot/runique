# Admin template system

## 3-level hierarchy

```
admin_template.html   ← level 1: contract (defined blocks, fixed elements)
        ↓ extends
admin_base.html            ← level 2: default visual layout
        ↓ extends
list.html / create.html …  ← level 3: CRUD components
```

**Level 1 — `admin_template.html`**: elements outside blocks guaranteed (CSRF, messages). Do not override directly.

**Level 2 — `admin_base.html`**: default layout (sidebar, topbar, styles). This is the file the developer replaces to change the appearance.

**Level 3 — components**: CRUD pages that extend level 2 and fill `{% block content %}`.

---

## Available blocks

| Block | Role |
| --- | --- |
| `{% block title %}` | Page title (`<title>`) |
| `{% block extra_css %}` | Additional CSS in `<head>` |
| `{% block layout %}` | Wraps the entire layout (sidebar + main) |
| `{% block sidebar %}` | Navigation sidebar |
| `{% block topbar %}` | Top bar (breadcrumb, logout) |
| `{% block breadcrumb %}` | Breadcrumb (defined in `admin_base`) |
| `{% block messages %}` | Flash message area — contains `{% messages %}` by default |
| `{% block content %}` | Main page content |
| `{% block extra_js %}` | Additional JS scripts before `</body>` |

### Elements outside blocks (always present)

Written directly in `admin_template.html` — **cannot be removed** by overriding:

- `<meta name="csrf-token" content="{{ csrf_token }}">` in `<head>`
- `<script src="{{ "js/csrf.js" | runique_static }}" defer></script>` before `</body>`

---

## Sub-sections

| Section | Description |
| --- | --- |
| [Context keys](/docs/en/admin/template) | Variables injected by the backend into each template |
| [Override](/docs/en/admin/template) | Replace the layout or a CRUD component |
| [CSRF](/docs/en/admin/template) | CSRF token, `csrf.js`, custom login checklist |

## See also

| Section | Description |
| --- | --- |
| [Setup](/docs/en/admin/setup) | Wire the admin into an existing project, create a superuser |
| [CLI](/docs/en/admin/declaration) | `runique start` command, general workflow |
| [Permissions](/docs/en/admin/permission) | Roles, `is_staff` / `is_superuser`, runtime check |
| [Roadmap](/docs/en/admin/evolution) | Planned features and beta status |

## Back to menu

- [Admin Summary](/docs/en/admin)
