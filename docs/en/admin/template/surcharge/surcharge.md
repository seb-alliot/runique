# Admin template override

The admin's look changes at three levels, lightest first:

1. **The theme**: the CSS variables (colors, radii, sizes), see [Block reference](/docs/en/admin/template-surcharge-blocks).
2. **A block**: one piece of a page (the list's table, the form's buttons, the sidebar…). You replace its HTML, its stylesheet, or both.
3. **A whole page**: the list, the create form, the detail…

Everything is declared in the builder. At boot:

- a missing block template refuses the boot (Tera checks every `{% include %}`);
- a `{% dev_block "name" %}` with an unknown name (typo) refuses the boot: it stays as is and Tera doesn't know that tag;
- a template path that doesn't end in `.html` / `.xml`, or that holds `"`, `{`, `}`, `%`, `#`, `\` or a control character, refuses the boot;
- a missing **page** template refuses the boot in development (`RUNIQUE_ENV=dev`) or with `DEBUG=true`, naming the page and the path. In production it is not looked up at boot: the error comes the first time the page is shown;
- a missing stylesheet is not detected: the page loads without it (404 on the file).

---

## How a page is made

```
admin/admin_template.html   ← contract: <head>, CSRF meta, csrf.js, modal
        ↓ extends
admin_base.html             ← layout: stylesheets, sidebar, top bar
        ↓ extends
admin/list.html, admin/create.html …   ← the pages
```

Each page is made of blocks. In Runique's templates, a replaceable block is written `{% dev_block "name" %}`. At boot, Runique turns each `{% dev_block %}` into an `{% include %}` of your template when you declared one, of its own otherwise.

---

## Where to declare

**For every model**: in `with_admin`, through `.templates(|t| …)` (type `AdminTemplate`).

```rust
use runique::admin::template::{Block, GlobalBlock, Page};

.with_admin(|a| a
    .templates(|t| t
        .block(Block::ListTable, "my_theme/table.html")
        .global_block(GlobalBlock::Sidebar, "my_theme/sidebar.html")
    )
    .site(site)
)
```

**For one model**: on its `ModelAdmin`, through `.templates(|t| …)` (type `ModelTemplates`).

```rust
fn article() -> ModelAdmin<article::Entity> {
    ModelAdmin::new()
        .templates(|t| t.block(Block::ListTable, "articles/table.html"))
}
```

Priority: what the model declares, then what is declared for every model, then Runique. A block's HTML and its stylesheet resolve **separately**: a model can replace a block's HTML and keep the stylesheet declared for every model.

Paths:

- a **template** is relative to your templates folder (`TEMPLATES_DIR`) and must end in `.html` or `.xml` (Tera escapes only those extensions);
- a **stylesheet** is relative to your static folder (`STATICFILES_DIRS`), served the way `{% static %}` serves it, with its integrity hash.

---

## Override a block

Blocks come in two kinds:

| Kind | Type | Replaceable | Examples |
| --- | --- | --- | --- |
| Block of a model's pages | `Block` | for every model **or** for one | `ListTable`, `FormActions`, `DeleteWarning` |
| Block of the layout, or of a page about no model | `GlobalBlock` | for every model only | `Sidebar`, `Topbar`, `DashboardTable` |

The full list of keys, with the page showing each block and Runique's template to copy as a starting point, is in the [block reference](/docs/en/admin/template-surcharge-blocks).

### The template only

The block takes your HTML; Runique's stylesheet stays loaded.

```rust
.templates(|t| t.block(Block::DeleteWarning, "my_theme/delete_warning.html"))
```

```html
{# templates/my_theme/delete_warning.html #}
<div class="admin-warning">
  <p>
    <strong>{{ admin_delete_warning_title }}</strong>
    {{ resource.title }} #{{ object_id }} will be deleted for good.
  </p>
</div>
```

A block's template gets the context of the page including it (here the delete page's: `resource`, `object_id`, the `admin_delete_*` translations…). See [Context keys](/docs/en/admin/template-clef). The safest start is Runique's template (`runique/templates/admin/blocks/…`), modified.

### The CSS only

The block keeps Runique's HTML and loads your stylesheet.

```rust
.templates(|t| t.block_stylesheet(Block::ListTable, "css/admin/table.css"))
```

Two cases:

- the block has a stylesheet in Runique (`ListTable`, `DetailFields`, `DetailGroups`, `DashboardTable`): **yours replaces it**. It must then define everything for that block, not only what you change;
- the block has none: yours is **added**, after Runique's.

### Both

The two declarations add up:

```rust
.templates(|t| t
    .block(Block::ListTable, "my_theme/table.html")
    .block_stylesheet(Block::ListTable, "css/admin/table.css")
)
```

For a block of the layout or of a page about no model, the same three cases with `global_block` and `global_block_stylesheet`:

```rust
.templates(|t| t
    .global_block(GlobalBlock::Sidebar, "my_theme/sidebar.html")
    .global_block_stylesheet(GlobalBlock::Sidebar, "css/admin/sidebar.css")
)
```

---

## Override a whole page

### For every model

| Method | Page |
| --- | --- |
| `.with_dashboard(path)` | dashboard |
| `.with_list(path)` | list |
| `.with_htmx(path)` | list fragment returned to HTMX requests (search, sort, pagination) |
| `.with_create(path)` | create |
| `.with_edit(path)` | edit |
| `.with_detail(path)` | detail |
| `.with_delete(path)` | delete confirmation |
| `.with_login(path)` | login |
| `.with_history(path)` | history |
| `.with_history_diff(path)` | one history entry |

The rights page and the group transfer page can't be replaced.

`.with_htmx` only touches the HTMX response: the full list page still includes Runique's fragment. Replacing one without the other renders the list differently depending on whether it is loaded or refreshed.

### For one model

```rust
ModelAdmin::new()
    .templates(|t| t.page(Page::List, "articles/list.html"))
```

Only `Page::List`, `Page::Create`, `Page::Edit`, `Page::Detail` and `Page::Delete` can be replaced per model. Another page refuses the boot. The detail page can also be replaced by a route ([declared admin routes](/docs/en/admin/declaration)); both at once refuse the boot: the route would win and the template never render.

### A page's template

Extending `admin_base.html` keeps Runique's layout (stylesheets, sidebar, top bar, CSRF):

```html
{# templates/articles/list.html #}
{% extends "admin_base.html" %}

{% block title %}{{ resource.title }}{% endblock %}

{% block content %}
<h1>{{ resource.title }} ({{ total }})</h1>
{% for entry in entries %}
  <p><a href="{{ entry.links.detail }}">#{{ entry.id }}</a></p>
{% endfor %}
{% if resource_links.create %}
  <a href="{{ resource_links.create }}" class="btn btn-primary">{{ admin_list_btn_create }}</a>
{% endif %}
{% endblock %}
```

A replaced page can still include Runique's blocks with `{% dev_block "name" %}`: they then follow the declared block overrides.

### A replaced page's stylesheet

By default your page gets Runique's stylesheets (layout + the page's blocks). With `page_stylesheet`, it loads only Runique's **variables**, then yours:

```rust
.templates(|t| t
    .with_list("my_theme/list.html")
    .page_stylesheet(Page::List, "css/admin/list.css")
)
// or for one model:
ModelAdmin::new().templates(|t| t
    .page(Page::List, "articles/list.html")
    .page_stylesheet(Page::List, "css/admin/articles_list.css")
)
```

`page_stylesheet` is read only when the page is replaced.

---

## Points of attention

- The stylesheets are written by `admin_base.html`'s `{% block extra_css %}`. If your page redefines `extra_css`, call `{{ super() }}`, otherwise no stylesheet is loaded at all. Same for `extra_js` (`admin.js`, the modal, the theme).
- The CSRF meta and `csrf.js` come from `admin/admin_template.html`. A page extending nothing must write them itself (see [Examples](/docs/en/admin/template-surcharge-examples)).
- Links come from the context (`nav.*`, `resource_links.*`, `entry.links.*`, `form_action`, `cancel_url`): don't build URLs by hand, the admin prefix and the rights are already applied. A missing link means the user has no right to it; the gate checks again on the target page anyway.
- `{{ var }}` is escaped, `{{ var | safe }}` is not: use `safe` only on HTML you produce yourself.
- A file of your templates with the same name as one of Runique's (for instance `admin/blocks/list/table.html`) replaces it for everyone, with no declaration and no check. Keep your admin templates under another folder (`my_theme/…`).

## Current limits

- No global theme stylesheet: to change the CSS variables on every page, go through a block shown everywhere, for instance `.global_block_stylesheet(GlobalBlock::Topbar, "css/admin/theme.css")` (every page but the login). See [CSS theme](/docs/en/admin/template-surcharge-blocks).
- `.with_htmx` is global: no per-model list fragment.
- In production, a missing page template is only reported on its first display, and a missing stylesheet never (see above).

## Sub-sections

| Section | Description |
| --- | --- |
| [Block reference](/docs/en/admin/template-surcharge-blocks) | Block keys, the pages showing them, CSS variables |
| [Examples](/docs/en/admin/template-surcharge-examples) | Complete cases: block, CSS, page, standalone page |
| [CSS classes](/docs/en/admin/template-surcharge-classes) | Classes used by Runique's templates |
| [Context keys](/docs/en/admin/template-clef) | Variables injected by the backend into each template |

## Back to summary

| Section | Description |
| --- | --- |
| [Template summary](/docs/en/admin/template) | Admin templates |
| [Admin summary](/docs/en/admin) | Admin |
