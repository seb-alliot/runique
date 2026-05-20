# CSS classes per block — admin

Reference of CSS classes used in each Tera block.  
Useful for targeting specific elements via CSS selectors without rewriting the whole block.

> To modify global colors and spacing, prefer [CSS custom properties](/docs/en/admin/template/surcharge/blocks#css-theme--custom-properties).

---

## `list_header` — `list.html`

| Class | Element | Role |
| --- | --- | --- |
| `.admin-page__header` | `<div>` | Flex header container |
| `.admin-page__title` | `<h1>` | Resource title |
| `.admin-page__subtitle` | `<p>` | Entry count |
| `.btn .btn-primary` | `<a>` | Create button |

---

## `list_search` — `list_partial.html`

| Class | Element | Role |
| --- | --- | --- |
| `.admin-search__row` | `<div>` | Search bar container |
| `.admin-search__form` | `<form>` | Search form |
| `.admin-search__wrapper` | `<div>` | Flex wrapper for input + buttons |
| `.admin-search__input` | `<input>` | Search field |
| `.admin-search__btn` | `<button>` | Submit button (magnifier) |
| `.admin-search__filter-toggle` | `<button>` | Mobile filter toggle button |
| `.admin-search__filter-badge` | `<span>` | Active filter count (mobile) |

---

## `list_group_action` — `list_partial.html`

| Class | Element | Role |
| --- | --- | --- |
| `.admin-group-action__bar` | `<div>` | Bar container (hidden by default) |
| `.admin-group-action__bar--visible` | modifier | Makes the bar visible (added by JS) |
| `.admin-group-action__info` | `<span>` | "N selected" text |
| `.admin-group-action__btns` | `<div>` | Action buttons container |
| `.admin-group-action__selects` | `<div>` | Group action selects row |
| `.admin-group-action__select` | `<select>` | Group action select |

---

## `list_table` — `list_partial.html`

| Class | Element | Role |
| --- | --- | --- |
| `.admin-card` | `<div>` | Card container |
| `.admin-table__wrapper` | `<div>` | Horizontal scroll |
| `.admin-table` | `<table>` | Main table |
| `.admin-table__col-bulk` | `<col>` | Checkbox column |
| `.admin-table__col-id` | `<col>` | ID column |
| `.admin-table__col-secondary` | `<col>` / `<td>` / `<th>` | Secondary columns (hidden on mobile) |
| `.admin-table__col-expand` | `<col>` | Expand button column |
| `.admin-table__col-actions` | `<col>` | Kebab actions column |
| `.admin-table__th-bulk` | `<th>` | Checkbox header |
| `.admin-table__th-right` | `<th>` | Actions header (right-aligned) |
| `.admin-table__th-expand` | `<th>` | Expand header (empty) |
| `.admin-table__sort-link` | `<a>` | Sort link in header |
| `.admin-table__sort-indicator` | `<span>` | ▲▼ sort indicator |
| `.admin-table__td-bulk` | `<td>` | Checkbox cell |
| `.admin-table__bulk-check` | `<input>` | Selection checkbox |
| `.admin-table__td-data` | `<td>` | Data cell (truncatable) |
| `.admin-table__td-expand` | `<td>` | Expand button cell |
| `.admin-table__td-actions` | `<td>` | Kebab menu cell |
| `.admin-table__td-content` | `<span>` | Content truncated to 2 lines |
| `.admin-badge--id` | `<span>` | Monospace ID badge |
| `.admin-badge.admin-badge--green` | `<span>` | Boolean true badge |
| `.admin-badge.admin-badge--neutral` | `<span>` | Boolean false badge |
| `.admin-text--muted` | `<span>` | Empty value `—` |
| `.admin-table__expand-btn` | `<button>` | Row expand button |
| `.admin-table__expand-icon` | `<svg>` | Expand chevron icon |
| `.admin-table__row-detail` | `<tr>` | Detail row (hidden by default) |
| `.admin-table__detail-grid` | `<div>` | Secondary columns grid |
| `.admin-table__detail-item` | `<div>` | Label + value item |
| `.admin-table__detail-label` | `<span>` | Item label |
| `.admin-table__detail-value` | `<span>` | Item value |
| `.admin-empty-state` | `<div>` | Empty state container |
| `.admin-empty-state__icon` | `<svg>` | Empty state icon |
| `.admin-empty-state__title` | `<p>` | Empty state title |
| `.admin-empty-state__desc` | `<p>` | Empty state description |

---

## `list_pagination` — `list_partial.html`

| Class | Element | Role |
| --- | --- | --- |
| `.admin-pagination` | `<div>` | Pagination container |
| `.admin-pagination__info` | `<span>` | "page / total" text |
| `.btn .btn-sm .btn-secondary` | `<a>` / `<span>` | Previous / next buttons |
| `.disabled` | modifier | Disabled button state |

---

## `list_filters` — `list_partial.html`

| Class | Element | Role |
| --- | --- | --- |
| `.admin-filter__overlay` | `<div>` | Mobile overlay |
| `.admin-filter__sidebar` | `<aside>` | Filter sidebar container |
| `.admin-filter__header` | `<div>` | Sidebar header |
| `.admin-filter__title` | `<span>` | "Filters" title |
| `.admin-filter__toggle` | `<button>` | Hide/show button |
| `.admin-filter__body` | `<div>` | Scrollable body |
| `.admin-filter__group` | `<div>` | Single filter group |
| `.admin-filter__group-title` | `<button>` | Group title (accordion) |
| `.admin-filter__chevron` | `<svg>` | Accordion chevron |
| `.admin-filter__group-body` | `<div>` | Group body |
| `.admin-filter__option` | `<a>` | Filter option link |
| `.admin-filter__option--active` | modifier | Active option |
| `.admin-filter__option--clear` | modifier | Clear filter link |
| `.admin-filter__pagination` | `<div>` | Filter value pagination |
| `.admin-filter__page-btn` | `<a>` / `<span>` | Previous/next page button |
| `.admin-filter__page-info` | `<span>` | Filter page info |

---

## `create_header` / `edit_header` / `delete_header` / `group_edit_header`

| Class | Element | Role |
| --- | --- | --- |
| `.admin-page__header` | `<header>` / `<div>` | Header container |

---

## `create_form` / `edit_form`

| Class | Element | Role |
| --- | --- | --- |
| `.admin-card.admin-card--form` | `<div>` | Form card |
| `.admin-card__header` | `<div>` | Card header |
| `.admin-card__body` | `<div>` | Card body |
| `.form-grid` | `<div>` | Field grid (Runique class) |
| `.admin-m2m__fields` | `<div>` | M2M fields section |
| `.form-group.admin-m2m__group` | `<div>` | Single M2M field group |
| `.form-label` | `<label>` | Field label (Runique class) |
| `.admin-m2m__choices` | `<div>` | M2M choices container |
| `.admin-m2m__choice` | `<label>` | Individual M2M choice |

---

## `create_form_actions` / `edit_form_actions` / `delete_actions` / `group_edit_actions`

| Class | Element | Role |
| --- | --- | --- |
| `.admin-form__actions` | `<div>` | Buttons container |
| `.btn .btn-secondary` | `<a>` | Cancel button |
| `.btn .btn-primary` | `<button>` | Submit button |
| `.btn .btn-danger` | `<button>` | Delete confirm button |
| `.admin-form--inline` | `<form>` | Inline form (delete) |

---

## `create_denied` / `edit_denied` / `delete_denied` / `group_edit_denied`

| Class | Element | Role |
| --- | --- | --- |
| `.admin-card` | `<div>` | Container |
| `.admin-card__body` | `<div>` | Body |
| `.admin-empty-state` | `<div>` | Empty state container |
| `.admin-empty-state__desc` | `<p>` | Access denied message |

---

## `detail_header`

| Class | Element | Role |
| --- | --- | --- |
| `.admin-page__header` | `<div>` | Flex container |
| `.admin-page__title` | `<h1>` | Title |
| `.admin-page__subtitle` | `<p>` | Subtitle with ID |

## `detail_actions`

| Class | Element | Role |
| --- | --- | --- |
| `.admin-page__actions` | `<div>` | Action buttons container |
| `.admin-btn--hide-mobile` | modifier | Hidden on mobile |
| `.admin-btn--show-mobile` | modifier | Visible on mobile only |
| `.admin-menu` | `<div>` | Mobile kebab container |
| `.admin-menu__trigger` | `<button>` | Kebab trigger |
| `.admin-menu__dropdown` | `<div>` | Dropdown menu |
| `.admin-menu__item` | `<a>` / `<button>` | Menu item |
| `.admin-menu__separator` | `<div>` | Separator |
| `.admin-menu__item--danger` | modifier | Destructive item (red) |

## `detail_table`

| Class | Element | Role |
| --- | --- | --- |
| `.admin-card` | `<div>` | Container |
| `.admin-table__wrapper` | `<div>` | Horizontal scroll |
| `.admin-table` | `<table>` | Key → value table |
| `.admin-table__key` | `<td>` | Key cell (left column) |
| `.admin-badge--id` | `<span>` | ID badge |
| `.admin-badge.admin-badge--green` | `<span>` | Boolean true |
| `.admin-badge.admin-badge--neutral` | `<span>` | Boolean false |
| `.admin-text--muted` | `<span>` | Empty value |

---

## `delete_warning`

| Class | Element | Role |
| --- | --- | --- |
| `.admin-warning` | `<div>` | Red warning banner |

---

## `group_edit_fields`

| Class | Element | Role |
| --- | --- | --- |
| `.admin-card.admin-card--form.admin-group-edit__section` | `<div>` | Section card |
| `.form-grid` | `<div>` | Field grid (Runique class) |
| `.admin-group-edit__toggle-row` | `<div>` | Check-all button row |

## `group_edit_permissions`

| Class | Element | Role |
| --- | --- | --- |
| `.admin-group-edit__section` | `<div>` | Permissions card section |
| `.admin-group-edit__section-header` | `<div>` | Header with check-all button |
| `.admin-group-edit__perm-grid` | `<div>` | Permissions grid (populated by JS) |

---

## `dashboard_header`

| Class | Element | Role |
| --- | --- | --- |
| `.admin-page__header` | `<div>` | Flex container |
| `.admin-page__title` | `<h1>` | Title |
| `.admin-page__subtitle` | `<p>` | Subtitle |

## `dashboard_stats`

| Class | Element | Role |
| --- | --- | --- |
| `.admin-stats__grid` | `<div>` | Card grid |
| `.admin-stats__card` | `<a>` | Resource card |
| `.admin-stats__label` | `<div>` | Resource name |
| `.admin-stats__value` | `<div>` | Count |
| `.admin-card.admin-card--full-width` | `<div>` | Empty state card (full width) |

## `dashboard_table`

| Class | Element | Role |
| --- | --- | --- |
| `.admin-card` | `<div>` | Container |
| `.admin-card__header` | `<div>` | Card header |
| `.admin-card__title` | `<h2>` | Card title |
| `.admin-table__wrapper` | `<div>` | Horizontal scroll |
| `.admin-table.admin-table--resources` | `<table>` | Resources table |
| `.admin-table__col-secondary` | `<td>` / `<th>` | Hidden on mobile |
| `.admin-badge--id` | `<span>` | Resource key badge |
| `.admin-badge.admin-badge--blue` | `<span>` | Group badge |
| `.admin-table__actions` | `<div>` | Action buttons container |

---

## Back to summary

| Section | Description |
| --- | --- |
| [Block reference](/docs/en/admin/template/surcharge/blocks) | Block list + CSS custom properties |
| [Template override](/docs/en/admin/template/surcharge/surcharge) | Principle and examples |
| [Template summary](/docs/en/admin/template) | Admin templates |
