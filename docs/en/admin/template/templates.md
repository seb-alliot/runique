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
| [Context keys](https://github.com/seb-alliot/runique/blob/main/docs/en/admin/template/clef/context.md) | Variables injected by the backend into each template |
| [Override](https://github.com/seb-alliot/runique/blob/main/docs/en/admin/template/surcharge/surcharge.md) | Replace the layout or a CRUD component |
| [CSRF](https://github.com/seb-alliot/runique/blob/main/docs/en/admin/template/csrf/csrf.md) | CSRF token, `csrf.js`, custom login checklist |
| [Clef context](https://github.com/seb-alliot/runique/blob/main/docs/en/admin/template/clef/clef_context.md) | token CSRF, `csrf.js`, checklist login custom

## See also

| Section | Description |
| --- | --- |
| [Setup](https://github.com/seb-alliot/runique/blob/main/docs/en/admin/setup/setup.md) | Wire the admin into an existing project, create a superuser |
| [CLI](https://github.com/seb-alliot/runique/blob/main/docs/en/admin/declaration/cli.md) | `runique start` command, general workflow |
| [Permissions](https://github.com/seb-alliot/runique/blob/main/docs/en/admin/permission/permissions.md) | Roles, `is_staff` / `is_superuser`, runtime check |
| [Roadmap](https://github.com/seb-alliot/runique/blob/main/docs/en/admin/evolution/evolution.md) | Planned features and beta status |

## Back to menu

- [Admin Summary](https://github.com/seb-alliot/runique/blob/main/docs/en/admin/11-Admin.md)
