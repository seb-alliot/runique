# CSS classes and JS hooks — admin

The classes Runique's templates use, block by block, and the `id`s / attributes the admin's scripts look for. Useful for:

- **the CSS only**: knowing what to target in your stylesheet;
- **the template only**: knowing what to keep so Runique's JS and stylesheets still apply.

The source of truth: `runique/templates/admin/` and `runique/static/css/admin/`.

---

## To keep when you replace the HTML

Runique's scripts find their elements by these `id`s, classes and attributes. Without them, the matching behavior stops (with no visible error).

| Block | To keep | Role |
| --- | --- | --- |
| `sidebar` | `#adminSidebar`, `#sidebarToggle`, `#sidebarOverlay` | Open / collapse the sidebar (`admin.js`); `#mobileBurger` is in `admin_base.html` |
| `topbar` | `#themeToggle` | Theme button (`theme-toggle.js`) |
| `kebab` | `.admin-menu__trigger`, `.admin-menu__dropdown` | Open the row menu (`admin-kebab.js`) |
| `list_bulk`, `list_table` | `#bulk-form`, `#bulk-action-bar`, `#bulk-count`, `#bulk-action-input`, `#bulk-ids-input`, `#bulk-check-all`, `.admin-table__bulk-check`, `[data-bulk-action]` | Selection and grouped actions (`admin-bulk.js`) |
| `list_filters` | `#filterSidebar`, `#filterOverlay`, `#filterToggle`, `.admin-filter__group`, `.admin-filter__group-title`, `.admin-filter__group-body`, `.admin-filter__chevron`, `[data-filter-col]` | Filter panel (`admin-list.js`) |
| `list_search` | `#mobileFilterToggle` | Button opening the filters on mobile (`admin-list.js`) |
| `list_table` | `.admin-list__cards`, `.admin-list__field-value` | List display (`admin-list.js`) |
| create, edit, detail pages | `[data-admin-auto-init]` | Field initialization (`admin-actions.js`, `checkbox-toggle.js`, `choice-filter.js`) |
| rights | `.admin-rights__menu a`, `select[data-autosubmit]` | Grid menu, picking a group |
| layout | `#rqModalOverlay`, `#rqModalTitle`, `#rqModalMsg`, `#rqModalOk`, `#rqModalCancel` | Confirmation modal, in `admin/admin_template.html` (not replaceable) |

The list's search, sort and pagination go through HTMX attributes (`hx-get`, `hx-target="#list-content"`, `hx-swap`): a replaced block that wants to keep refreshing without a reload must copy them from Runique's.

---

## Classes per block

### Layout

| Template | Classes |
| --- | --- |
| `admin/admin_template.html` | `admin-layout`, `admin-main`, `admin-content`, `rq-modal-overlay`, `rq-modal-box`, `rq-modal-title`, `rq-modal-msg`, `rq-modal-actions` |
| `admin_base.html` | `admin-topbar`, `admin-topbar__burger`, `admin-topbar__breadcrumb`, `admin-topbar__sep` (pages) |
| `sidebar` | `admin-sidebar`, `admin-sidebar__header`, `admin-sidebar__logo`, `admin-sidebar__title`, `admin-sidebar__nav`, `admin-sidebar__footer`, `admin-sidebar__footer-center`, `admin-sidebar__toggle`, `admin-sidebar__overlay`, `admin-nav__item`, `admin-nav__icon`, `admin-nav__label`, `admin-nav__divider`, `admin-nav__section-label` |
| `topbar` | `admin-theme-toggle`, `admin-theme-icon`, `admin-theme-icon--sun`, `admin-theme-icon--moon`, `admin-topbar__user`, `admin-topbar__avatar`, `admin-logout__form`, `admin-logout__btn` |

### Page headers

`admin-page__header`, `admin-page__title`, `admin-page__subtitle`, `admin-page__header-actions`, `admin-page__actions`, `admin-page__create-btn`.

### List

| Block | Classes |
| --- | --- |
| `list_search` | `admin-search__wrapper`, `admin-search__form`, `admin-search__row`, `admin-search__input`, `admin-search__btn`, `admin-search__filter-toggle`, `admin-search__filter-badge` |
| `list_bulk` | `admin-group-action__bar`, `admin-group-action__info`, `admin-group-action__btns` |
| `list_table` | `admin-list__toolbar`, `admin-list__select-all`, `admin-list__sort-bar`, `admin-table__sort-link`, `admin-table__sort-indicator`, `admin-list__cards`, `admin-list__card`, `admin-list__card-header`, `admin-list__card-body`, `admin-list__card-actions`, `admin-list__field`, `admin-list__field-label`, `admin-list__field-value`, `admin-table__bulk-check`, `admin-table__td-actions` |
| `list_pagination` | `admin-pagination`, `admin-pagination__info` |
| `list_filters` | `admin-filter__sidebar`, `admin-filter__overlay`, `admin-filter__header`, `admin-filter__title`, `admin-filter__toggle`, `admin-filter__body`, `admin-filter__group`, `admin-filter__group-title`, `admin-filter__group-body`, `admin-filter__chevron`, `admin-filter__option`, `admin-filter__option--clear` |
| `kebab` | `admin-menu`, `admin-menu__trigger`, `admin-menu__dropdown`, `admin-menu__item`, `admin-menu__item--danger`, `admin-menu__item--create-mobile`, `admin-menu__separator` |

### Detail

| Block | Classes |
| --- | --- |
| page | `admin-detail__column` |
| `detail_fields` | `admin-detail__fields`, `admin-detail__field`, `admin-detail__field-label`, `admin-detail__field-value` |
| `detail_actions` | `admin-page__actions`, `admin-form--inline`, `admin-search__input--sm` |
| `detail_groups` | `admin-rights__table`, `admin-rights__box` |
| `detail_transfer` | `admin-search__input--sm` |

### Forms and delete

| Block | Classes |
| --- | --- |
| create / edit pages | `admin-card--form` |
| `form_fields` | `form-grid` (the rest comes from the framework's field templates) |
| `form_actions`, `delete_actions` | `admin-form__actions`, `admin-form--inline` |
| `delete_warning` | `admin-warning` |

### Dashboard, history, rights, transfer

| Block | Classes |
| --- | --- |
| `dashboard_table` | `admin-stats__grid`, `admin-stats__card`, `admin-stats__body`, `admin-stats__label`, `admin-stats__value`, `admin-stats__action` |
| `history_filters` | `admin-search__row`, `admin-search__input` |
| `history_table`, entry page | `admin-table__wrapper`, `admin-table`, `admin-table__td-actions`, `admin-table__key` |
| `rights_table` | `admin-rights__card`, `admin-rights__table`, `admin-rights__row`, `admin-rights__row--head`, `admin-rights__label`, `admin-rights__box`, `admin-rights__columns`, `admin-rights__column` |
| rights page | `admin-rights`, `admin-rights__form`, `admin-rights__toc`, `admin-rights__toc-title`, `admin-rights__menu`, `admin-rights__links`, `admin-rights__select` |
| transfer page | `admin-transfer__lines`, `admin-transfer__line`, `admin-transfer__keep`, `admin-transfer__gained-right`, `admin-transfer__lost-right` |

### Login

`login-wrapper`, `login-card`, `login-header`, `login-logo`, `login-title`, `login-subtitle`, `login-form`, `login-btn`, `form-group`, `form-label`, `form-input`, `form-error`, `alert`, `alert-danger`.

---

## Shared components

| Component | Classes |
| --- | --- |
| Card | `admin-card`, `admin-card__header`, `admin-card__title`, `admin-card__body`, `admin-card--flat`, `admin-card--full-width` |
| Buttons | `btn`, `btn-primary`, `btn-secondary`, `btn-danger`, `btn-ghost`, `btn-sm`, `btn-icon` |
| Badges | `admin-badge`, `admin-badge--id`, `admin-badge--green`, `admin-badge--red`, `admin-badge--blue`, `admin-badge--yellow`, `admin-badge--neutral` |
| Empty state | `admin-empty-state`, `admin-empty-state__icon`, `admin-empty-state__title`, `admin-empty-state__desc` |
| Text | `admin-text--muted`, `mt-sm` |

Colors and sizes go through the CSS variables: see [CSS theme](/docs/en/admin/template-surcharge-blocks).

---

## Back to summary

| Section | Description |
| --- | --- |
| [Override](/docs/en/admin/template-surcharge) | Block, CSS, page: how to declare |
| [Block reference](/docs/en/admin/template-surcharge-blocks) | Block keys, CSS variables |
| [Template summary](/docs/en/admin/template) | Admin templates |
