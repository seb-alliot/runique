# Tera context keys — Admin

Variables injected by the backend into each admin template.

---

## Available everywhere (auth middleware)

| Variable | Type | Description |
| --- | --- | --- |
| `csrf_token` | `String` | Session CSRF token (masked) |
| `current_user.username` | `String` | Authenticated username |
| `current_user.is_staff` | `bool` | Admin access |
| `current_user.is_superuser` | `bool` | Full access |
| `current_user.roles` | `Vec<String>` | Custom roles |

---

## Available on all CRUD pages

| Variable | Type | Description |
| --- | --- | --- |
| `site_title` | `String` | Site title (from `AdminConfig`) |
| `resource` | `AdminResource` | Active resource metadata |
| `resource.key` | `String` | URL key (e.g. `"users"`) |
| `resource.title` | `String` | Displayed title (e.g. `"Users"`) |
| `resources` | `Vec<AdminResource>` | All registered resources |
| `current_resource` | `String` | Active resource key (= `resource.key`) |

---

## Dashboard

| Variable | Type | Description |
| --- | --- | --- |
| `resource_counts` | `HashMap<String, u64>` | Entry count per resource |
| `current_page` | `String` | Equals `"dashboard"` |

---

## Login

| Variable | Type | Description |
| --- | --- | --- |
| `error` | `String` (optional) | Error message on failed login |

---

## List (`list.html`)

| Variable | Type | Description |
| --- | --- | --- |
| `entries` | `Vec<JSON Object>` | Records serialised from the model |
| `total` | `usize` | Total number of entries |
| `current_page` | `String` | Equals `"list"` |

---

## Create (`create.html`)

| Variable | Type | Description |
| --- | --- | --- |
| `form_fields` | `Forms` | Form with pre-rendered HTML and errors |
| `is_edit` | `bool` | Equals `false` |

---

## Edit (`edit.html`)

| Variable | Type | Description |
| --- | --- | --- |
| `form_fields` | `Forms` | Form pre-filled with existing values |
| `is_edit` | `bool` | Equals `true` |
| `object_id` | `i32` | Record identifier |

---

## Detail & Delete (`detail.html`, `delete.html`)

| Variable | Type | Description |
| --- | --- | --- |
| `entry` | `JSON Object` (optional) | Serialised record |
| `object_id` | `i32` | Record identifier |

## Sub-sections

| Section | Description |
| --- | --- |
| [Override](https://github.com/seb-alliot/runique/blob/main/docs/en/admin/template/surcharge/surcharge.md) | Replace the layout or a CRUD component |
| [CSRF](https://github.com/seb-alliot/runique/blob/main/docs/en/admin/template/csrf/csrf.md) | CSRF token, `csrf.js`, custom login checklist |
| [Clef context](https://github.com/seb-alliot/runique/blob/main/docs/en/admin/template/clef/clef_context.md) | token CSRF, `csrf.js`, checklist login custom

## Back to summary

| Section | Description |
| --- | --- |
| [Template summary](https://github.com/seb-alliot/runique/blob/main/docs/en/admin/template/templates.md) | Admin templates |
| [Admin summary](https://github.com/seb-alliot/runique/blob/main/docs/en/admin/11-Admin.md) | Admin |
