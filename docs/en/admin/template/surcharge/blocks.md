# Block reference — admin

The keys to pass to `.block(…)`, `.block_stylesheet(…)`, `.global_block(…)` and `.global_block_stylesheet(…)`. How they work is described in [Template override](/docs/en/admin/template-surcharge).

```rust
use runique::admin::template::{Block, GlobalBlock, Page};
```

Each block's Runique template is the best starting point: copy it into your templates and modify it. It lives in the crate's sources, under `templates/` (for a crate installed from crates.io: `~/.cargo/registry/src/*/runique-<version>/templates/`).

---

## Blocks of a model's pages — `Block`

Replaceable for every model (`AdminTemplate`) or for one (`ModelTemplates`).

| Rust key | Name (`dev_block`) | Page | Runique's template | Runique's stylesheet |
| --- | --- | --- | --- | --- |
| `Block::ListTable` | `list_table` | list | `admin/blocks/list/table.html` | `css/admin/blocks/list.css` |
| `Block::ListFilters` | `list_filters` | list | `admin/blocks/list/filters.html` | — |
| `Block::ListSearch` | `list_search` | list | `admin/blocks/list/search.html` | — |
| `Block::ListPagination` | `list_pagination` | list | `admin/blocks/list/pagination.html` | — |
| `Block::ListBulk` | `list_bulk` | list | `admin/blocks/list/bulk.html` | — |
| `Block::Kebab` | `kebab` | list (each row's menu) | `admin/blocks/common/kebab.html` | — |
| `Block::DetailFields` | `detail_fields` | detail | `admin/blocks/detail/fields.html` | `css/admin/blocks/detail.css` |
| `Block::DetailActions` | `detail_actions` | detail | `admin/blocks/detail/actions.html` | — |
| `Block::DetailGroups` | `detail_groups` | detail | `admin/blocks/detail/groups.html` | `css/admin/blocks/rights.css` |
| `Block::DetailTransfer` | `detail_transfer` | detail | `admin/blocks/detail/transfer.html` | — |
| `Block::FormFields` | `form_fields` | create, edit | `admin/blocks/form/fields.html` | — |
| `Block::FormActions` | `form_actions` | create, edit | `admin/blocks/form/actions.html` | — |
| `Block::DeleteWarning` | `delete_warning` | delete | `admin/blocks/delete/warning.html` | — |
| `Block::DeleteActions` | `delete_actions` | delete | `admin/blocks/delete/actions.html` | — |

`FormFields` and `FormActions` are shared by create and edit: replacing them changes both pages.

## Blocks of the layout and of pages about no model — `GlobalBlock`

Replaceable for every model only (`AdminTemplate`).

| Rust key | Name (`dev_block`) | Page | Runique's template | Runique's stylesheet |
| --- | --- | --- | --- | --- |
| `GlobalBlock::Sidebar` | `sidebar` | all but the login | `admin/blocks/layout/sidebar.html` | — |
| `GlobalBlock::Topbar` | `topbar` | all but the login (right of the top bar: theme, back to the site, sign out) | `admin/blocks/layout/topbar_actions.html` | — |
| `GlobalBlock::DashboardTable` | `dashboard_table` | dashboard | `admin/blocks/dashboard/table.html` | `css/admin/blocks/dashboard.css` |
| `GlobalBlock::DashboardGroups` | `dashboard_groups` | dashboard | `admin/blocks/dashboard/my_groups.html` | — |
| `GlobalBlock::HistoryTable` | `history_table` | history | `admin/blocks/history/table.html` | — |
| `GlobalBlock::HistoryFilters` | `history_filters` | history | `admin/blocks/history/filters.html` | — |
| `GlobalBlock::HistoryActionBadge` | `history_action_badge` | history, history entry | `admin/blocks/history/action_badge.html` | — |
| `GlobalBlock::RightsTable` | `rights_table` | rights | `admin/blocks/rights/table.html` | — |

Reminder: when a block has a stylesheet in Runique, `block_stylesheet` / `global_block_stylesheet` **replaces** it; otherwise yours is **added**.

---

## Pages — `Page`

Used by `ModelTemplates::page` and `page_stylesheet` (global and per model).

| Rust key | Global replacement | Per model |
| --- | --- | --- |
| `Page::Login` | `.with_login` | no |
| `Page::Dashboard` | `.with_dashboard` | no |
| `Page::List` | `.with_list` | yes |
| `Page::Create` | `.with_create` | yes |
| `Page::Edit` | `.with_edit` | yes |
| `Page::Detail` | `.with_detail` | yes |
| `Page::Delete` | `.with_delete` | yes |
| `Page::History` | `.with_history` | no |
| `Page::HistoryDiff` | `.with_history_diff` | no |
| `Page::Rights` | no | no |
| `Page::Transfer` | no | no |

---

## Tera blocks of Runique's templates

A page that **extends** one of Runique's templates can redefine its Tera `{% block %}`s (not to be confused with the `dev_block` blocks above). Use `{{ super() }}` to keep the original content and add around it.

| Template | Tera blocks |
| --- | --- |
| `admin/admin_template.html` | `lang`, `title`, `extra_css`, `layout`, `sidebar`, `topbar`, `messages`, `content`, `extra_js`, `footer` |
| `admin_base.html` | fills `extra_css` (stylesheets), `sidebar`, `topbar` (holds `breadcrumb`), `extra_js` |
| `admin/list.html` | `title`, `breadcrumb`, `content`, `list_header`, `extra_css`, `extra_js` |
| `admin/list_partial.html` | `list_search`, `list_group_action`, `list_table`, `list_pagination`, `list_filters` |
| `admin/create.html` | `title`, `breadcrumb`, `content`, `create_header`, `create_form`, `create_form_fields`, `create_form_actions`, `extra_js` |
| `admin/edit.html` | `title`, `breadcrumb`, `content`, `edit_header`, `edit_form`, `edit_form_fields`, `edit_form_actions`, `extra_js` |
| `admin/detail.html` | `title`, `breadcrumb`, `content`, `detail_header`, `detail_actions`, `detail_table`, `detail_groups`, `detail_transfer` |
| `admin/delete.html` | `title`, `breadcrumb`, `content`, `delete_header`, `delete_warning`, `delete_actions` |
| `admin/dashboard.html` | `title`, `breadcrumb`, `content`, `dashboard_header`, `dashboard_groups`, `dashboard_stats`, `dashboard_table` |
| `admin/history.html` | `title`, `breadcrumb`, `content`, `history_header`, `history_filters`, `history_table`, `history_pagination` |
| `admin/history_diff.html` | `title`, `breadcrumb`, `content`, `history_entry_header`, `history_entry_changes` |

To change a single block, prefer `.block(…)`: it doesn't depend on the page's structure and can target one model.

---

## CSS theme — variables

Every admin stylesheet reads the variables of `css/admin/variables.css`. The default theme is dark (`:root`); the light theme is defined under `[data-theme="light"]`, set on `<html>` by the theme button. A stylesheet changing the theme must therefore redefine both.

This stylesheet is always loaded, even on a page replaced with `page_stylesheet`.

| Group | Variables |
| --- | --- |
| Backgrounds | `--bg-main`, `--bg-card`, `--bg-sidebar`, `--bg-input`, `--bg-hover`, `--bg-active`, `--overlay` |
| Text | `--text-main`, `--text-muted`, `--text-sidebar`, `--label`, `--on-accent` |
| Accent | `--accent`, `--accent-hover`, `--accent-light`, `--accent-gradient`, `--focus-ring` |
| States | `--success`, `--success-light`, `--danger`, `--danger-light`, `--danger-border`, `--danger-ring`, `--warning`, `--warning-light`, `--info`, `--info-light` |
| Borders and shadows | `--border`, `--border-light`, `--radius-sm`, `--radius`, `--radius-lg`, `--shadow-sm`, `--shadow`, `--shadow-lg` |
| Sizes | `--sidebar-width`, `--sidebar-collapsed`, `--topbar-height`, `--content-max`, `--control-height`, `--control-height-sm` |
| Fonts | `--font-sans`, `--font-mono` |
| Stacking | `--z-sticky`, `--z-dropdown`, `--z-bar`, `--z-sidebar`, `--z-drawer`, `--z-overlay`, `--z-modal` |
| Other | `--transition` |

There is no dedicated method for a theme stylesheet. It loads on every page (but the login) when attached to a block shown everywhere that has no stylesheet in Runique:

```rust
.templates(|t| t.global_block_stylesheet(GlobalBlock::Topbar, "css/admin/theme.css"))
```

```css
/* static/css/admin/theme.css */
:root {
  --accent: #e11d48;
  --accent-hover: #be123c;
}
[data-theme="light"] {
  --accent: #be123c;
  --accent-hover: #9f1239;
}
```

It loads after Runique's stylesheets: at equal specificity, its values win.

The CSS classes used by Runique's templates are listed in the [CSS class reference](/docs/en/admin/template-surcharge-classes).

---

## Back to summary

| Section | Description |
| --- | --- |
| [Template override](/docs/en/admin/template-surcharge) | Block, CSS, page: how to declare |
| [Context keys](/docs/en/admin/template-clef) | Variables injected by the backend |
| [Template summary](/docs/en/admin/template) | Admin templates |
