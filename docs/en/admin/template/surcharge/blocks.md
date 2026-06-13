# Tera block reference — admin

Each admin template exposes `{% block %}` sections that developers can override in their own templates.  
Use `{{ super() }}` to keep the default content and add around it.

---

## Layout blocks — `admin_base.html`

Defined in the global layout, available from any template that extends `admin_base`.

| Block | Default content | Note |
| --- | --- | --- |
| `title` | Page title | — |
| `extra_css` | Runique admin CSS (7 files) | Use `{{ super() }}` to accumulate |
| `layout` | Sidebar + main area | Advanced — replaces everything |
| `sidebar` | Resource navigation + history | — |
| `topbar` | Breadcrumb + site link + logout | — |
| `breadcrumb` | Breadcrumb trail (from `admin_base`) | — |
| `messages` | Flash messages | Keep `{% messages %}` inside |
| `content` | CRUD page content | — |
| `extra_js` | `admin.js` | Use `{{ super() }}` to accumulate |

---

## `list.html`

| Block | Content |
| --- | --- |
| `list_header` | Page header: title, count, Create button |

---

## `list_partial.html` *(HTMX-swapped)*

| Block | Content |
| --- | --- |
| `list_search` | Search bar + hidden sort/filter fields |
| `list_group_action` | Group action bar (selection + bulk actions) |
| `list_table` | Main table + empty state |
| `list_pagination` | Pagination controls |
| `list_filters` | Column filter sidebar |

---

## `create.html`

| Block | Content |
| --- | --- |
| `create_header` | Page header |
| `create_form` | Full card with form |
| `create_form_fields` | Field grid + M2M fields |
| `create_form_actions` | Cancel / Create buttons |
| `create_denied` | Access denied message |

---

## `edit.html`

| Block | Content |
| --- | --- |
| `edit_header` | Page header |
| `edit_form` | Full card with form |
| `edit_form_fields` | Field grid + M2M fields |
| `edit_form_actions` | Cancel / Save buttons |
| `edit_denied` | Access denied message |

---

## `detail.html`

| Block | Content |
| --- | --- |
| `detail_header` | Page header (includes `detail_actions`) |
| `detail_actions` | Edit / Delete / Reset password buttons + mobile menu |
| `detail_table` | Card with key → value table |

---

## `delete.html`

| Block | Content |
| --- | --- |
| `delete_header` | Page header |
| `delete_warning` | Warning banner |
| `delete_actions` | Cancel / Confirm deletion buttons |
| `delete_denied` | Access denied message |

---

## `bulk_edit.html`

| Block | Content |
| --- | --- |
| `group_edit_header` | Page header |
| `group_edit_fields` | Non-boolean fields section |
| `group_edit_permissions` | Boolean permissions section (populated by JS) |
| `group_edit_actions` | Cancel / Apply buttons |
| `group_edit_denied` | Access denied message |

---

## `dashboard.html`

| Block | Content |
| --- | --- |
| `dashboard_header` | Page header |
| `dashboard_stats` | Stat-card grid per resource |
| `dashboard_table` | Resource summary table |

---

## CSS theme — custom properties

To change colors and spacing without rewriting HTML, override variables inside `{% block extra_css %}`:

```html
{% block extra_css %}
{{ super() }}
<style>
  :root {
    --accent:       #e11d48;
    --accent-hover: #be123c;
    --bg-main:      #fafafa;
    --bg-card:      #ffffff;
    --bg-sidebar:   #1e1e2e;
    --text-main:    #111827;
  }
</style>
{% endblock %}
```

| Variable | Role |
| --- | --- |
| `--bg-main` | Main background |
| `--bg-card` | Card background |
| `--bg-sidebar` | Sidebar background |
| `--bg-input` | Input field background |
| `--bg-hover` | Hover background |
| `--bg-active` | Active element background |
| `--text-main` | Main text color |
| `--text-muted` | Secondary text color |
| `--text-sidebar` | Sidebar text color |
| `--accent` | Accent color (buttons, active links) |
| `--accent-hover` | Accent on hover |
| `--accent-light` | Translucent accent |
| `--border` | Standard border |
| `--border-light` | Light border |
| `--success` / `--danger` / `--warning` | Semantic colors |
| `--sidebar-width` | Expanded sidebar width |
| `--sidebar-collapsed` | Collapsed sidebar width |
| `--topbar-height` | Topbar height |
| `--radius` / `--radius-lg` | Border radii |
| `--shadow` | Card shadow |
| `--transition` | Duration/easing of transitions |

> CSS classes per block are documented in the [CSS class reference](/docs/en/admin/template-surcharge-classes). BEM renaming is planned for v2.2.

---

## Back to summary

| Section | Description |
| --- | --- |
| [Template override](/docs/en/admin/template-surcharge) | Principle, inheritance levels, examples |
| [Context keys](/docs/en/admin/template-clef) | Variables injected by the backend |
| [Template summary](/docs/en/admin/template) | Admin templates |
