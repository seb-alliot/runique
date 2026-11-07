# Admin — Tera variables per page

The variables each admin page gets, and so does any block or page replacing it (see [Template override](/docs/en/admin/template-surcharge)). The source of truth is `runique/src/admin/router/context.rs`: each page has its struct there (`ListContext`, `DetailContext`…).

Links always come ready-made (`nav.*`, `resource_links.*`, `links.*`, `form_action`, `cancel_url`…): the admin prefix and the user's rights are already applied. A link set to `none` means the user may not go there. Don't build URLs by hand.

---

## Shared by every page (but the login)

| Variable | Content |
| --- | --- |
| `site_title` | Title declared with `.site_title(…)` |
| `lang` | Current language code (`fr`, `en`…) |
| `admin_prefix` | The admin's base address (mount prefix + path), no trailing `/` |
| `stylesheets` | The page's stylesheets, each `{ href, integrity }`; written by `admin_base.html` |
| `resources` | Models the user may see, in menu order: `{ key, title, list_url }` |
| `current_resource` | The page's model key (a string), empty on the dashboard |
| `current_page` | `"dashboard"`, `"list"`, `"detail"`, `"create"`, `"edit"`, `"delete"`, `"history"`, `"rights"`, `"transfer"` |
| `current_user` | Signed-in account: `id`, `username`, `rang`, `groupes`, `droits` |
| `nav.dashboard` | Dashboard |
| `nav.history` | History, `none` when the account reads none |
| `nav.logout` | Where the sign-out form posts (POST + CSRF) |
| `nav.site` | "Back to the site" link |
| `nav.toggle_template` | Dashboard switch, see below |
| `csrf_token` | The request's CSRF token |
| `admin_base_*` | The layout's translations |

Plus the variables of any Runique template (`{% messages %}`, `{% csp %}`, `{% static %}`…), see [Templates](/docs/en/template).

---

## Translations

Each page gets its section's translations, as `admin.<section>.<key>` → `{{ admin_<section>_<key> }}` (for instance `admin.list.btn_create` → `{{ admin_list_btn_create }}`). The key list is in `runique/src/utils/constante/admin_key.rs` (`ADMIN_MESSAGE_KEYS`), the texts in `runique/src/utils/trad/<language>.json`.

| Page | Sections |
| --- | --- |
| login | `login` |
| dashboard | `base`, `dashboard` |
| list | `base`, `list` |
| detail | `base`, `detail`, `list` |
| create | `base`, `create` |
| edit | `base`, `edit` |
| delete | `base`, `delete` |
| history, history entry | `base`, `history` |
| rights | `base`, `rights` |
| group transfer | `base`, `transfer` |

---

## Login

| Variable | Content |
| --- | --- |
| `admin_prefix` | The form posts to `{{ admin_prefix }}/login` |
| `csrf_token` | To send back in the hidden `csrf_token` field |
| `error` | Failure message (credentials, CSRF, locked account, session), absent otherwise |
| `site_url` | Address of the "back to the site" link below the sign-in button: `.view_site_url(…)`, else `/` |
| `admin_login_back_to_site` | Translated text of that link ("← Back to the site") |
| `site_title`, `lang`, `stylesheets` | — |

Fields the POST expects: `username`, `password`, `csrf_token`.

---

## Dashboard

| Variable | Content |
| --- | --- |
| `resource_counts` | Row count per model: `resource_counts[resource.key]`; empty when counting failed |
| `my_groups` | The account's groups: `{ name, staff, rights: [{ table, rights }] }` |
| `admin_has_session_override` | `true` when the switch brought back Runique's dashboard |

When the dashboard is replaced (`.with_dashboard`), `nav.toggle_template` shows Runique's one and back; the choice is kept in the session.

---

## List

Also rendered, without layout, for HTMX requests (`admin/list_partial.html`).

| Variable | Content |
| --- | --- |
| `resource` | `{ key, title, display: { list_filter: [(column, label)] } }` |
| `resource_links` | `{ list, create, bulk, rights }`; `create`, `bulk`, `rights` are `none` without the right |
| `entries` | The page's rows: `{ id, values, links: { detail, edit, delete }, taken_by, account }` |
| `visible_columns` | Shown columns, in order (key left out) |
| `sortable_columns` | Sortable columns (declared `.sortable()`) |
| `column_labels` | Each column's header, by name |
| `rich_fields` | Columns holding HTML, shown as plain text |
| `group_actions` | Grouped actions offered: `{ index, label }` |
| `bulk_delete` | Whether the grouped actions' bar offers delete |
| `return_qs` | Current query (search, sort, filters, page), to come back to the same place |
| `filter_values` | Each filter's values, by column: `value`, `active`, `qs` (link turning it on or off) |
| `filter_clear` | Link turning a whole filter off, for the filtered columns |
| `active_filters` | Applied filters, column → values |
| `filter_qs` | Applied filters as a query string (`&filter_col=value…`) |
| `search` | Current search |
| `sort_by`, `sort_dir`, `sort_dir_toggle` | Current sort (`asc` / `desc`) and the other direction |
| `total` | Matching rows, all pages |
| `page`, `page_count`, `page_size` | Current page, page count, rows per page |
| `list_pager` | The list's arrows and selector: `page`, `page_count`, `prev_qs`, `next_qs` (`None` with no page that way), `size`, `size_options`, `size_param`, `keep` |
| `filter_pagers` | Each filter group's arrows, by column (same shape, `page_count` is `None`) |
| `filter_size_pager` | The filter panel's selector (same shape), `None` without a filter |
| `size_qs`, `filter_nav_qs` | The list's size, and the panel's pages and size, as a query string to keep on a link |
| `filter_nav` | The panel's pages and size, as pairs for a form's hidden inputs |

`entry.values[column]` gives a shown column's value.

---

## Detail

| Variable | Content |
| --- | --- |
| `resource`, `resource_key` | The model (`resource_key` = `resource.key`) |
| `object_id` | The row's key |
| `entry` | The row: one entry per readable column |
| `detail_columns` | The row's readable columns, in display order |
| `rich_fields` | HTML columns, shown sanitized |
| `list_url` | Back to the list |
| `links` | `{ detail, edit, delete }` |
| `reset_password_url` | On an account, with the right to update it |
| `rank_choices`, `current_rank`, `rank_url` | On an account whose rank may be changed: ranks offered `(value, label)`, current rank, where to post |
| `rights_url` | An admin's rights grid, when it may be set |
| `memberships`, `memberships_url` | On an account: groups managed `(field name, group name, ticked)`, where to post |
| `transfer_receivers`, `transfer_url` | On a group, for a dirigeant: accounts it may be handed to `(id, name)`, transfer page |
| `taken_by` | Group taken back after a departure: by whom |
| `account` | On an account: `{ rank, rank_label, groups }`, read-only |

---

## Create and edit

| Variable | Content |
| --- | --- |
| `resource`, `resource_key` | The model |
| `object_id` | The row's key (edit only) |
| `list_url` | Back to the list |
| `form_fields` | The form: `html` (fields, CSRF and form name included), `errors`, `form_errors`, `fields` |
| `form_action` | Where the form posts |
| `cancel_url` | "Cancel": the row on edit, the list on create |
| `cancel_label`, `submit_label` | Button texts |
| `no_fields_label` | Shown when the form has no field |

Write `{{ form_fields.html }}`: at load, Runique rewrites it to `{{ form_fields.html | safe }}` (the form's HTML is produced by Runique, its values already escaped). The `form_fields` and `form_actions` blocks are included inside the `<form>` tag.

---

## Delete

| Variable | Content |
| --- | --- |
| `resource`, `resource_key` | The model |
| `object_id` | The row's key |
| `list_url` | Back to the list |
| `form_action` | Where the confirmation posts |
| `form_fields` | Confirmation form, with no field: `html` (CSRF token and name), `errors`, `form_errors` |
| `cancel_url` | Back to the row |

---

## History

| Variable | Content |
| --- | --- |
| `history_url` | Whole history: where the filters post, and their reset |
| `entries` | Entries, newest first (see below) |
| `history_resources` | Models whose history the account reads: `{ key, title }` |
| `filter_resource`, `filter_object`, `filter_action`, `filter_user` | Current filters |
| `page`, `total_pages` | Pagination |

An entry: `{ id, created_at, username, resource_key, resource_title, object_pk, object_label, action, has_changes, links: { user, object, entry } }`. `action` is `create`, `update` or `delete`; `created_at` is UTC (`YYYY-MM-DD HH:MM:SS`); `links.entry` is `none` without a readable change.

## History entry

| Variable | Content |
| --- | --- |
| `history_url` | Back to the history |
| `entry` | The entry (same shape as above) |
| `changes` | Changed, readable columns: `{ column, old, new }` |

---

## Rights and transfer

These two pages can't be replaced; only the rights page's `rights_table` block can. Their context (`RightsContext`, `TransferContext`) is described in `context.rs`.

---

## Sub-sections

| Section | Description |
| --- | --- |
| [Override](/docs/en/admin/template-surcharge) | Replace a block (HTML, CSS or both) or a whole page |

## Back to summary

| Section | Description |
| --- | --- |
| [Template summary](/docs/en/admin/template) | Admin templates |
| [Admin summary](/docs/en/admin) | Admin |
