# Référence des blocs — admin

Les clés à passer à `.block(…)`, `.block_stylesheet(…)`, `.global_block(…)` et `.global_block_stylesheet(…)`. Le fonctionnement est décrit dans [Surcharge](/docs/fr/admin/template-surcharge).

```rust
use runique::admin::template::{Block, GlobalBlock, Page};
```

Le template de Runique de chaque bloc est le meilleur point de départ : copiez-le dans vos templates et modifiez-le. Il se trouve dans les sources du crate, sous `templates/` (pour un crate installé depuis crates.io : `~/.cargo/registry/src/*/runique-<version>/templates/`).

---

## Blocs des pages d'un modèle — `Block`

Remplaçables pour tous les modèles (`AdminTemplate`) ou pour un seul (`ModelTemplates`).

| Clé Rust | Nom (`dev_block`) | Page | Template de Runique | Feuille de Runique |
| --- | --- | --- | --- | --- |
| `Block::ListTable` | `list_table` | liste | `admin/blocks/list/table.html` | `css/admin/blocks/list.css` |
| `Block::ListFilters` | `list_filters` | liste | `admin/blocks/list/filters.html` | — |
| `Block::ListSearch` | `list_search` | liste | `admin/blocks/list/search.html` | — |
| `Block::ListPagination` | `list_pagination` | liste | `admin/blocks/list/pagination.html` | — |
| `Block::ListBulk` | `list_bulk` | liste | `admin/blocks/list/bulk.html` | — |
| `Block::Kebab` | `kebab` | liste (menu de chaque ligne) | `admin/blocks/common/kebab.html` | — |
| `Block::DetailFields` | `detail_fields` | détail | `admin/blocks/detail/fields.html` | `css/admin/blocks/detail.css` |
| `Block::DetailActions` | `detail_actions` | détail | `admin/blocks/detail/actions.html` | — |
| `Block::DetailGroups` | `detail_groups` | détail | `admin/blocks/detail/groups.html` | `css/admin/blocks/rights.css` |
| `Block::DetailTransfer` | `detail_transfer` | détail | `admin/blocks/detail/transfer.html` | — |
| `Block::FormFields` | `form_fields` | création, édition | `admin/blocks/form/fields.html` | — |
| `Block::FormActions` | `form_actions` | création, édition | `admin/blocks/form/actions.html` | — |
| `Block::DeleteWarning` | `delete_warning` | suppression | `admin/blocks/delete/warning.html` | — |
| `Block::DeleteActions` | `delete_actions` | suppression | `admin/blocks/delete/actions.html` | — |

`FormFields` et `FormActions` sont partagés par la création et l'édition : les remplacer change les deux pages.

## Blocs du layout et des pages sans modèle — `GlobalBlock`

Remplaçables pour tous seulement (`AdminTemplate`).

| Clé Rust | Nom (`dev_block`) | Page | Template de Runique | Feuille de Runique |
| --- | --- | --- | --- | --- |
| `GlobalBlock::Sidebar` | `sidebar` | toutes sauf la connexion | `admin/blocks/layout/sidebar.html` | — |
| `GlobalBlock::Topbar` | `topbar` | toutes sauf la connexion (droite de la barre du haut : thème, retour au site, déconnexion) | `admin/blocks/layout/topbar_actions.html` | — |
| `GlobalBlock::DashboardTable` | `dashboard_table` | tableau de bord | `admin/blocks/dashboard/table.html` | `css/admin/blocks/dashboard.css` |
| `GlobalBlock::DashboardGroups` | `dashboard_groups` | tableau de bord | `admin/blocks/dashboard/my_groups.html` | — |
| `GlobalBlock::HistoryTable` | `history_table` | historique | `admin/blocks/history/table.html` | — |
| `GlobalBlock::HistoryFilters` | `history_filters` | historique | `admin/blocks/history/filters.html` | — |
| `GlobalBlock::HistoryActionBadge` | `history_action_badge` | historique, entrée de l'historique | `admin/blocks/history/action_badge.html` | — |
| `GlobalBlock::RightsTable` | `rights_table` | droits | `admin/blocks/rights/table.html` | — |

Rappel : quand un bloc a une feuille chez Runique, `block_stylesheet` / `global_block_stylesheet` la **remplace** ; sinon la vôtre **s'ajoute**.

---

## Pages — `Page`

Utilisé par `ModelTemplates::page`, `page_stylesheet` (global et par modèle).

| Clé Rust | Remplacement global | Par modèle |
| --- | --- | --- |
| `Page::Login` | `.with_login` | non |
| `Page::Dashboard` | `.with_dashboard` | non |
| `Page::List` | `.with_list` | oui |
| `Page::Create` | `.with_create` | oui |
| `Page::Edit` | `.with_edit` | oui |
| `Page::Detail` | `.with_detail` | oui |
| `Page::Delete` | `.with_delete` | oui |
| `Page::History` | `.with_history` | non |
| `Page::HistoryDiff` | `.with_history_diff` | non |
| `Page::Rights` | non | non |
| `Page::Transfer` | non | non |

---

## Blocs Tera des templates de Runique

Une page qui **étend** un template de Runique peut redéfinir ses `{% block %}` Tera (à ne pas confondre avec les blocs `dev_block` ci-dessus). Utilisez `{{ super() }}` pour garder le contenu d'origine et ajouter autour.

| Template | Blocs Tera |
| --- | --- |
| `admin/admin_template.html` | `lang`, `title`, `extra_css`, `layout`, `sidebar`, `topbar`, `messages`, `content`, `extra_js`, `footer` |
| `admin_base.html` | remplit `extra_css` (feuilles de style), `sidebar`, `topbar` (contient `breadcrumb`), `extra_js` |
| `admin/list.html` | `title`, `breadcrumb`, `content`, `list_header`, `extra_css`, `extra_js` |
| `admin/list_partial.html` | `list_search`, `list_group_action`, `list_table`, `list_pagination`, `list_filters` |
| `admin/create.html` | `title`, `breadcrumb`, `content`, `create_header`, `create_form`, `create_form_fields`, `create_form_actions`, `extra_js` |
| `admin/edit.html` | `title`, `breadcrumb`, `content`, `edit_header`, `edit_form`, `edit_form_fields`, `edit_form_actions`, `extra_js` |
| `admin/detail.html` | `title`, `breadcrumb`, `content`, `detail_header`, `detail_actions`, `detail_table`, `detail_groups`, `detail_transfer` |
| `admin/delete.html` | `title`, `breadcrumb`, `content`, `delete_header`, `delete_warning`, `delete_actions` |
| `admin/dashboard.html` | `title`, `breadcrumb`, `content`, `dashboard_header`, `dashboard_groups`, `dashboard_stats`, `dashboard_table` |
| `admin/history.html` | `title`, `breadcrumb`, `content`, `history_header`, `history_filters`, `history_table`, `history_pagination` |
| `admin/history_diff.html` | `title`, `breadcrumb`, `content`, `history_entry_header`, `history_entry_changes` |

Pour changer un seul bloc, préférez `.block(…)` : il ne dépend pas de la structure de la page et peut viser un seul modèle.

---

## Thème CSS — variables

Toutes les feuilles de l'admin lisent les variables de `css/admin/variables.css`. Le thème par défaut est sombre (`:root`) ; le thème clair est défini sous `[data-theme="light"]`, posé sur `<html>` par le bouton de thème. Une feuille qui change le thème doit donc redéfinir les deux.

Cette feuille est toujours chargée, même sur une page remplacée avec `page_stylesheet`.

| Groupe | Variables |
| --- | --- |
| Fonds | `--bg-main`, `--bg-card`, `--bg-sidebar`, `--bg-input`, `--bg-hover`, `--bg-active`, `--overlay` |
| Texte | `--text-main`, `--text-muted`, `--text-sidebar`, `--label`, `--on-accent` |
| Accent | `--accent`, `--accent-hover`, `--accent-light`, `--accent-gradient`, `--focus-ring` |
| États | `--success`, `--success-light`, `--danger`, `--danger-light`, `--danger-border`, `--danger-ring`, `--warning`, `--warning-light`, `--info`, `--info-light` |
| Bordures et ombres | `--border`, `--border-light`, `--radius-sm`, `--radius`, `--radius-lg`, `--shadow-sm`, `--shadow`, `--shadow-lg` |
| Tailles | `--sidebar-width`, `--sidebar-collapsed`, `--topbar-height`, `--content-max`, `--control-height`, `--control-height-sm` |
| Polices | `--font-sans`, `--font-mono` |
| Empilement | `--z-sticky`, `--z-dropdown`, `--z-bar`, `--z-sidebar`, `--z-drawer`, `--z-overlay`, `--z-modal` |
| Divers | `--transition` |

Il n'y a pas de méthode dédiée pour une feuille de thème. Elle se charge sur toutes les pages (sauf la connexion) en l'attachant à un bloc présent partout et sans feuille chez Runique :

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

Elle est chargée après les feuilles de Runique : à spécificité égale, ses valeurs l'emportent.

Les classes CSS utilisées par les templates de Runique sont listées dans la [référence des classes CSS](/docs/fr/admin/template-surcharge-classes).

---

## Revenir au sommaire

| Section | Description |
| --- | --- |
| [Surcharge templates](/docs/fr/admin/template-surcharge) | Bloc, CSS, page : comment déclarer |
| [Clés de contexte](/docs/fr/admin/template-clef) | Variables injectées par le backend |
| [Sommaire template](/docs/fr/admin/template) | Sommaire templates |
