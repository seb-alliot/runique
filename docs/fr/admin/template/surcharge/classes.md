# Classes CSS et crochets JS — admin

Les classes utilisées par les templates de Runique, bloc par bloc, et les `id` / attributs que les scripts de l'admin vont chercher. Utile pour :

- **le CSS seul** : savoir quoi cibler dans votre feuille ;
- **le template seul** : savoir quoi garder pour que le JS et les feuilles de Runique continuent de s'appliquer.

La source qui fait foi : `runique/templates/admin/` et `runique/static/css/admin/`.

---

## À garder si vous remplacez le HTML

Les scripts de Runique trouvent leurs éléments par ces `id`, classes et attributs. Sans eux, le comportement correspondant s'arrête (sans erreur visible).

| Bloc | À garder | Rôle |
| --- | --- | --- |
| `sidebar` | `#adminSidebar`, `#sidebarToggle`, `#sidebarOverlay` | Ouvrir / réduire la barre (`admin.js`) ; `#mobileBurger` est dans `admin_base.html` |
| `topbar` | `#themeToggle` | Bouton de thème (`theme-toggle.js`) |
| `kebab` | `.admin-menu__trigger`, `.admin-menu__dropdown` | Ouvrir le menu de ligne (`admin-kebab.js`) |
| `list_bulk`, `list_table` | `#bulk-form`, `#bulk-action-bar`, `#bulk-count`, `#bulk-action-input`, `#bulk-ids-input`, `#bulk-check-all`, `.admin-table__bulk-check`, `[data-bulk-action]` | Sélection et actions groupées (`admin-bulk.js`) |
| `list_filters` | `#filterSidebar`, `#filterOverlay`, `#filterToggle`, `.admin-filter__group`, `.admin-filter__group-title`, `.admin-filter__group-body`, `.admin-filter__chevron`, `[data-filter-col]` | Panneau des filtres (`admin-list.js`) |
| `list_search` | `#mobileFilterToggle` | Bouton d'ouverture des filtres sur mobile (`admin-list.js`) |
| `list_table` | `.admin-list__cards`, `.admin-list__field-value` | Affichage de la liste (`admin-list.js`) |
| pages création, édition, détail | `[data-admin-auto-init]` | Initialisation des champs (`admin-actions.js`, `checkbox-toggle.js`, `choice-filter.js`) |
| droits | `.admin-rights__menu a`, `select[data-autosubmit]` | Menu de la grille, choix d'un groupe |
| layout | `#rqModalOverlay`, `#rqModalTitle`, `#rqModalMsg`, `#rqModalOk`, `#rqModalCancel` | Fenêtre de confirmation, dans `admin/admin_template.html` (non remplaçable) |

La recherche, le tri et la pagination de la liste passent par des attributs HTMX (`hx-get`, `hx-target="#list-content"`, `hx-swap`) : un bloc remplacé qui veut garder le rafraîchissement sans rechargement doit les reprendre de celui de Runique.

---

## Classes par bloc

### Layout

| Template | Classes |
| --- | --- |
| `admin/admin_template.html` | `admin-layout`, `admin-main`, `admin-content`, `rq-modal-overlay`, `rq-modal-box`, `rq-modal-title`, `rq-modal-msg`, `rq-modal-actions` |
| `admin_base.html` | `admin-topbar`, `admin-topbar__burger`, `admin-topbar__breadcrumb`, `admin-topbar__sep` (pages) |
| `sidebar` | `admin-sidebar`, `admin-sidebar__header`, `admin-sidebar__logo`, `admin-sidebar__title`, `admin-sidebar__nav`, `admin-sidebar__footer`, `admin-sidebar__footer-center`, `admin-sidebar__toggle`, `admin-sidebar__overlay`, `admin-nav__item`, `admin-nav__icon`, `admin-nav__label`, `admin-nav__divider`, `admin-nav__section-label` |
| `topbar` | `admin-theme-toggle`, `admin-theme-icon`, `admin-theme-icon--sun`, `admin-theme-icon--moon`, `admin-topbar__user`, `admin-topbar__avatar`, `admin-logout__form`, `admin-logout__btn` |

### En-têtes de page

`admin-page__header`, `admin-page__title`, `admin-page__subtitle`, `admin-page__header-actions`, `admin-page__actions`, `admin-page__create-btn`.

### Liste

| Bloc | Classes |
| --- | --- |
| `list_search` | `admin-search__wrapper`, `admin-search__form`, `admin-search__row`, `admin-search__input`, `admin-search__btn`, `admin-search__filter-toggle`, `admin-search__filter-badge` |
| `list_bulk` | `admin-group-action__bar`, `admin-group-action__info`, `admin-group-action__btns` |
| `list_table` | `admin-list__toolbar`, `admin-list__select-all`, `admin-list__sort-bar`, `admin-table__sort-link`, `admin-table__sort-indicator`, `admin-list__cards`, `admin-list__card`, `admin-list__card-header`, `admin-list__card-body`, `admin-list__card-actions`, `admin-list__field`, `admin-list__field-label`, `admin-list__field-value`, `admin-table__bulk-check`, `admin-table__td-actions` |
| `list_pagination` | `admin-pagination`, `admin-pagination__info` |
| `list_filters` | `admin-filter__sidebar`, `admin-filter__overlay`, `admin-filter__header`, `admin-filter__title`, `admin-filter__toggle`, `admin-filter__body`, `admin-filter__group`, `admin-filter__group-title`, `admin-filter__group-body`, `admin-filter__chevron`, `admin-filter__option`, `admin-filter__option--clear` |
| `kebab` | `admin-menu`, `admin-menu__trigger`, `admin-menu__dropdown`, `admin-menu__item`, `admin-menu__item--danger`, `admin-menu__item--create-mobile`, `admin-menu__separator` |

### Détail

| Bloc | Classes |
| --- | --- |
| page | `admin-detail__column` |
| `detail_fields` | `admin-detail__fields`, `admin-detail__field`, `admin-detail__field-label`, `admin-detail__field-value` |
| `detail_actions` | `admin-page__actions`, `admin-form--inline`, `admin-search__input--sm` |
| `detail_groups` | `admin-rights__table`, `admin-rights__box` |
| `detail_transfer` | `admin-search__input--sm` |

### Formulaires et suppression

| Bloc | Classes |
| --- | --- |
| pages création / édition | `admin-card--form` |
| `form_fields` | `form-grid` (le reste vient des templates de champs du framework) |
| `form_actions`, `delete_actions` | `admin-form__actions`, `admin-form--inline` |
| `delete_warning` | `admin-warning` |

### Tableau de bord, historique, droits, transfert

| Bloc | Classes |
| --- | --- |
| `dashboard_table` | `admin-stats__grid`, `admin-stats__card`, `admin-stats__body`, `admin-stats__label`, `admin-stats__value`, `admin-stats__action` |
| `history_filters` | `admin-search__row`, `admin-search__input` |
| `history_table`, page d'une entrée | `admin-table__wrapper`, `admin-table`, `admin-table__td-actions`, `admin-table__key` |
| `rights_table` | `admin-rights__card`, `admin-rights__table`, `admin-rights__row`, `admin-rights__row--head`, `admin-rights__label`, `admin-rights__box`, `admin-rights__columns`, `admin-rights__column` |
| page des droits | `admin-rights`, `admin-rights__form`, `admin-rights__toc`, `admin-rights__toc-title`, `admin-rights__menu`, `admin-rights__links`, `admin-rights__select` |
| page de transfert | `admin-transfer__lines`, `admin-transfer__line`, `admin-transfer__keep`, `admin-transfer__gained-right`, `admin-transfer__lost-right` |

### Connexion

`login-wrapper`, `login-card`, `login-header`, `login-logo`, `login-title`, `login-subtitle`, `login-form`, `login-btn`, `form-group`, `form-label`, `form-input`, `form-error`, `alert`, `alert-danger`.

---

## Composants partagés

| Composant | Classes |
| --- | --- |
| Carte | `admin-card`, `admin-card__header`, `admin-card__title`, `admin-card__body`, `admin-card--flat`, `admin-card--full-width` |
| Boutons | `btn`, `btn-primary`, `btn-secondary`, `btn-danger`, `btn-ghost`, `btn-sm`, `btn-icon` |
| Badges | `admin-badge`, `admin-badge--id`, `admin-badge--green`, `admin-badge--red`, `admin-badge--blue`, `admin-badge--yellow`, `admin-badge--neutral` |
| État vide | `admin-empty-state`, `admin-empty-state__icon`, `admin-empty-state__title`, `admin-empty-state__desc` |
| Texte | `admin-text--muted`, `mt-sm` |

Les couleurs et tailles passent par les variables CSS : voir [Thème CSS](/docs/fr/admin/template-surcharge-blocks).

---

## Revenir au sommaire

| Section | Description |
| --- | --- |
| [Surcharge](/docs/fr/admin/template-surcharge) | Bloc, CSS, page : comment déclarer |
| [Référence des blocs](/docs/fr/admin/template-surcharge-blocks) | Clés des blocs, variables CSS |
| [Sommaire template](/docs/fr/admin/template) | Sommaire templates |
