# Classes CSS par block — admin

Référence des classes CSS utilisées dans chaque block Tera.  
Utile pour cibler des éléments précis via sélecteurs CSS sans réécrire le block entier.

> Pour modifier les couleurs et espacements globaux, préférer les [custom properties CSS](/docs/fr/admin/template-surcharge-blocks#theme-css-custom-properties).

---

## `list_header` — `list.html`

| Classe | Élément | Rôle |
| --- | --- | --- |
| `.admin-page__header` | `<div>` | Container flex en-tête |
| `.admin-page__title` | `<h1>` | Titre de la ressource |
| `.admin-page__subtitle` | `<p>` | Compteur d'entrées |
| `.btn .btn-primary` | `<a>` | Bouton Créer |

---

## `list_search` — `list_partial.html`

| Classe | Élément | Rôle |
| --- | --- | --- |
| `.admin-search__row` | `<div>` | Container barre de recherche |
| `.admin-search__form` | `<form>` | Formulaire de recherche |
| `.admin-search__wrapper` | `<div>` | Flex wrapper input + boutons |
| `.admin-search__input` | `<input>` | Champ de recherche |
| `.admin-search__btn` | `<button>` | Bouton submit loupe |
| `.admin-search__filter-toggle` | `<button>` | Bouton filtres mobile |
| `.admin-search__filter-badge` | `<span>` | Compteur filtres actifs (mobile) |

---

## `list_group_action` — `list_partial.html`

| Classe | Élément | Rôle |
| --- | --- | --- |
| `.admin-group-action__bar` | `<div>` | Container barre (masqué par défaut) |
| `.admin-group-action__bar--visible` | modifier | Rend la barre visible (ajouté par JS) |
| `.admin-group-action__info` | `<span>` | Texte "N sélectionné(s)" |
| `.admin-group-action__btns` | `<div>` | Container boutons d'action |
| `.admin-group-action__selects` | `<div>` | Ligne des selects group actions |
| `.admin-group-action__select` | `<select>` | Select d'une group action |

---

## `list_table` — `list_partial.html`

| Classe | Élément | Rôle |
| --- | --- | --- |
| `.admin-card` | `<div>` | Container card |
| `.admin-table__wrapper` | `<div>` | Scroll horizontal |
| `.admin-table` | `<table>` | Tableau principal |
| `.admin-table__col-bulk` | `<col>` | Colonne checkbox |
| `.admin-table__col-id` | `<col>` | Colonne ID |
| `.admin-table__col-secondary` | `<col>` / `<td>` / `<th>` | Colonnes secondaires (masquées mobile) |
| `.admin-table__col-expand` | `<col>` | Colonne bouton expand |
| `.admin-table__col-actions` | `<col>` | Colonne actions kebab |
| `.admin-table__th-bulk` | `<th>` | Header checkbox |
| `.admin-table__th-right` | `<th>` | Header actions (aligné droite) |
| `.admin-table__th-expand` | `<th>` | Header expand (vide) |
| `.admin-table__sort-link` | `<a>` | Lien de tri dans le header |
| `.admin-table__sort-indicator` | `<span>` | Indicateur ▲▼ |
| `.admin-table__td-bulk` | `<td>` | Cellule checkbox |
| `.admin-table__bulk-check` | `<input>` | Checkbox de sélection |
| `.admin-table__td-data` | `<td>` | Cellule donnée (tronquable) |
| `.admin-table__td-expand` | `<td>` | Cellule bouton expand |
| `.admin-table__td-actions` | `<td>` | Cellule kebab menu |
| `.admin-table__td-content` | `<span>` | Contenu tronqué à 2 lignes |
| `.admin-badge--id` | `<span>` | Badge ID monospace |
| `.admin-badge.admin-badge--green` | `<span>` | Badge booléen vrai |
| `.admin-badge.admin-badge--neutral` | `<span>` | Badge booléen faux |
| `.admin-text--muted` | `<span>` | Valeur vide `—` |
| `.admin-table__expand-btn` | `<button>` | Bouton expand ligne |
| `.admin-table__expand-icon` | `<svg>` | Icône chevron expand |
| `.admin-table__row-detail` | `<tr>` | Ligne de détail (masquée par défaut) |
| `.admin-table__detail-grid` | `<div>` | Grille des colonnes secondaires |
| `.admin-table__detail-item` | `<div>` | Item label + valeur |
| `.admin-table__detail-label` | `<span>` | Label de l'item |
| `.admin-table__detail-value` | `<span>` | Valeur de l'item |
| `.admin-empty-state` | `<div>` | Container état vide |
| `.admin-empty-state__icon` | `<svg>` | Icône état vide |
| `.admin-empty-state__title` | `<p>` | Titre état vide |
| `.admin-empty-state__desc` | `<p>` | Description état vide |

---

## `list_pagination` — `list_partial.html`

| Classe | Élément | Rôle |
| --- | --- | --- |
| `.admin-pagination` | `<div>` | Container pagination |
| `.admin-pagination__info` | `<span>` | Texte "page / total" |
| `.btn .btn-sm .btn-secondary` | `<a>` / `<span>` | Boutons précédent / suivant |
| `.disabled` | modifier | Bouton désactivé |

---

## `list_filters` — `list_partial.html`

| Classe | Élément | Rôle |
| --- | --- | --- |
| `.admin-filter__overlay` | `<div>` | Overlay mobile |
| `.admin-filter__sidebar` | `<aside>` | Container sidebar filtres |
| `.admin-filter__header` | `<div>` | En-tête sidebar |
| `.admin-filter__title` | `<span>` | Titre "Filtres" |
| `.admin-filter__toggle` | `<button>` | Bouton masquer/afficher |
| `.admin-filter__body` | `<div>` | Corps scrollable |
| `.admin-filter__group` | `<div>` | Groupe d'un filtre |
| `.admin-filter__group-title` | `<button>` | Titre du groupe (accordéon) |
| `.admin-filter__chevron` | `<svg>` | Chevron accordéon |
| `.admin-filter__group-body` | `<div>` | Corps du groupe |
| `.admin-filter__option` | `<a>` | Option de filtre |
| `.admin-filter__option--active` | modifier | Option active |
| `.admin-filter__option--clear` | modifier | Lien "effacer" le filtre |
| `.admin-filter__pagination` | `<div>` | Pagination des valeurs |
| `.admin-filter__page-btn` | `<a>` / `<span>` | Bouton page précédente/suivante |
| `.admin-filter__page-info` | `<span>` | Info page filtre |

---

## `create_header` / `edit_header` / `delete_header` / `group_edit_header`

| Classe | Élément | Rôle |
| --- | --- | --- |
| `.admin-page__header` | `<header>` / `<div>` | Container en-tête |

---

## `create_form` / `edit_form`

| Classe | Élément | Rôle |
| --- | --- | --- |
| `.admin-card.admin-card--form` | `<div>` | Card formulaire |
| `.admin-card__header` | `<div>` | En-tête card |
| `.admin-card__body` | `<div>` | Corps card |
| `.form-grid` | `<div>` | Grille des champs (classe Runique) |
| `.admin-m2m__fields` | `<div>` | Section champs M2M |
| `.form-group.admin-m2m__group` | `<div>` | Groupe d'un champ M2M |
| `.form-label` | `<label>` | Label du champ (classe Runique) |
| `.admin-m2m__choices` | `<div>` | Container choix M2M |
| `.admin-m2m__choice` | `<label>` | Choix individuel M2M |

---

## `create_form_actions` / `edit_form_actions` / `delete_actions` / `group_edit_actions`

| Classe | Élément | Rôle |
| --- | --- | --- |
| `.admin-form__actions` | `<div>` | Container boutons |
| `.btn .btn-secondary` | `<a>` | Bouton Annuler |
| `.btn .btn-primary` | `<button>` | Bouton Valider |
| `.btn .btn-danger` | `<button>` | Bouton Supprimer (delete) |
| `.admin-form--inline` | `<form>` | Form inline (delete) |

---

## `create_denied` / `edit_denied` / `delete_denied` / `group_edit_denied`

| Classe | Élément | Rôle |
| --- | --- | --- |
| `.admin-card` | `<div>` | Container |
| `.admin-card__body` | `<div>` | Corps |
| `.admin-empty-state` | `<div>` | Container état vide |
| `.admin-empty-state__desc` | `<p>` | Message d'accès refusé |

---

## `detail_header`

| Classe | Élément | Rôle |
| --- | --- | --- |
| `.admin-page__header` | `<div>` | Container flex |
| `.admin-page__title` | `<h1>` | Titre |
| `.admin-page__subtitle` | `<p>` | Sous-titre avec ID |

## `detail_actions`

| Classe | Élément | Rôle |
| --- | --- | --- |
| `.admin-page__actions` | `<div>` | Container boutons d'action |
| `.admin-btn--hide-mobile` | modifier | Masqué sur mobile |
| `.admin-btn--show-mobile` | modifier | Visible uniquement sur mobile |
| `.admin-menu` | `<div>` | Container kebab mobile |
| `.admin-menu__trigger` | `<button>` | Déclencheur kebab |
| `.admin-menu__dropdown` | `<div>` | Menu déroulant |
| `.admin-menu__item` | `<a>` / `<button>` | Item du menu |
| `.admin-menu__separator` | `<div>` | Séparateur |
| `.admin-menu__item--danger` | modifier | Item destructif (rouge) |

## `detail_table`

| Classe | Élément | Rôle |
| --- | --- | --- |
| `.admin-card` | `<div>` | Container |
| `.admin-table__wrapper` | `<div>` | Scroll horizontal |
| `.admin-table` | `<table>` | Tableau clé → valeur |
| `.admin-table__key` | `<td>` | Cellule clé (colonne gauche) |
| `.admin-badge--id` | `<span>` | Badge ID |
| `.admin-badge.admin-badge--green` | `<span>` | Booléen vrai |
| `.admin-badge.admin-badge--neutral` | `<span>` | Booléen faux |
| `.admin-text--muted` | `<span>` | Valeur vide |

---

## `delete_warning`

| Classe | Élément | Rôle |
| --- | --- | --- |
| `.admin-warning` | `<div>` | Bandeau d'avertissement rouge |

---

## `group_edit_fields`

| Classe | Élément | Rôle |
| --- | --- | --- |
| `.admin-card.admin-card--form.admin-group-edit__section` | `<div>` | Section card |
| `.form-grid` | `<div>` | Grille des champs (classe Runique) |
| `.admin-group-edit__toggle-row` | `<div>` | Ligne bouton tout-cocher |

## `group_edit_permissions`

| Classe | Élément | Rôle |
| --- | --- | --- |
| `.admin-group-edit__section` | `<div>` | Section card permissions |
| `.admin-group-edit__section-header` | `<div>` | Header avec bouton tout-cocher |
| `.admin-group-edit__perm-grid` | `<div>` | Grille permissions (remplie par JS) |

---

## `dashboard_header`

| Classe | Élément | Rôle |
| --- | --- | --- |
| `.admin-page__header` | `<div>` | Container flex |
| `.admin-page__title` | `<h1>` | Titre |
| `.admin-page__subtitle` | `<p>` | Sous-titre |

## `dashboard_stats`

| Classe | Élément | Rôle |
| --- | --- | --- |
| `.admin-stats__grid` | `<div>` | Grille de cards |
| `.admin-stats__card` | `<a>` | Card ressource |
| `.admin-stats__label` | `<div>` | Nom de la ressource |
| `.admin-stats__value` | `<div>` | Compteur |
| `.admin-card.admin-card--full-width` | `<div>` | Card état vide (toute la largeur) |

## `dashboard_table`

| Classe | Élément | Rôle |
| --- | --- | --- |
| `.admin-card` | `<div>` | Container |
| `.admin-card__header` | `<div>` | En-tête card |
| `.admin-card__title` | `<h2>` | Titre card |
| `.admin-table__wrapper` | `<div>` | Scroll horizontal |
| `.admin-table.admin-table--resources` | `<table>` | Tableau ressources |
| `.admin-table__col-secondary` | `<td>` / `<th>` | Colonnes masquées mobile |
| `.admin-badge--id` | `<span>` | Badge clé ressource |
| `.admin-badge.admin-badge--blue` | `<span>` | Badge groupe |
| `.admin-table__actions` | `<div>` | Container boutons actions |

---

## Revenir au sommaire

| Section | Description |
| --- | --- |
| [Référence des blocks](/docs/fr/admin/template-surcharge-blocks) | Liste des blocks + custom properties CSS |
| [Surcharge templates](/docs/fr/admin/template-surcharge) | Principe et exemples |
| [Sommaire template](/docs/fr/admin/template) | Sommaire templates |
