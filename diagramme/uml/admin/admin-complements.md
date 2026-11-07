# UML — Admin 4.0 : config, templates, middleware, history, forms internes

Complément de [admin-resource-permissions.md](admin-resource-permissions.md).

## AdminConfig

[`admin/config/config_admin.rs`](../../../runique/src/admin/config/config_admin.rs)

```mermaid
classDiagram
    class AdminConfig {
        +String prefix
        +bool hot_reload / enabled
        +String site_title
        +Option~String~ view_site_url
        +AdminTemplate templates
        +u64 page_size
        +u64 filter_size
        +usize search_max_len
        +Option~ResetLinkConfig~ reset_link
        +Option reset_password_email_template
        +Vec~String~ resource_order
        +Option~Arc~RateLimiter~~ rate_limiter
        +Option~Arc~LoginGuard~~ login_guard
        +view_site_href() &str
    }
    class AdminStaging {
        +AdminConfig config
        +AdminSite site
        +String mount_prefix
        +Vec~DeclaredAdminRoute~ declared_routes
        +path() ≠ prefix()
        +register / accounts_users
        +with_rate_limiter / with_login_guard / templates
        +page_size / filter_size / search_max_len
        +view_site_url / sitemap / no_robots_txt
    }
    AdminStaging *-- AdminConfig
    AdminStaging *-- AdminSite
```

Deux segments indépendants : `path()` = où vit l'admin (`/admin`) ;
`prefix()` = ce qu'on met devant (`/secret` → `/secret/admin`). Ils ne
s'écrasent jamais, quel que soit l'ordre d'appel.

## Templates & stylesheets

[`admin/template/`](../../../runique/src/admin/template/)

```mermaid
classDiagram
    class PathAdminTemplate {
        +Option~String~ dev
        +&str runique
        +resolve() &str
    }
    class AdminTemplate {
        +PathAdminTemplate dashboard/login/list/create/edit/detail/delete
        +PathAdminTemplate htmx / history / history_diff
        +ModelBlocks blocks
        +Overrides~GlobalBlock~ global_blocks
        +HashMap~Page,String~ page_stylesheets
        +with_list / with_detail / … (pages)
        +block / global_block / *_stylesheet
        -page_for(page, model) &str
    }
    class ModelTemplates {
        -ModelBlocks blocks
        -HashMap~Page,String~ pages
        -HashMap~Page,String~ page_stylesheets
        +block / page / *_stylesheet
    }
    class Overrides~K~ {
        -HashMap~K,String~ html
        -HashMap~K,String~ stylesheet
        +block(k, path) / block_stylesheet(k, path)
    }
    class BlockTemplate {
        +PathAdminTemplate html
        +Option~Stylesheet~ stylesheet
    }
    class Stylesheet {
        <<enum>>
        Runique(&str)
        Dev(String)
    }
    class Page {
        <<enum>>
        Login Dashboard List Create Edit Detail Delete
        History HistoryDiff Rights Transfer
        +per_model() bool  « List…Delete »
    }
    class Block {
        <<enum>> blocs d'une page modèle
    }
    class GlobalBlock {
        <<enum>> layout + pages sans modèle
    }
    class PageSheets {
        +build(templates, models, statics)
        +without_integrity()  « dev »
        +of(page, resource) &[ServedStylesheet]
    }
    AdminTemplate *-- "*" PathAdminTemplate
    AdminTemplate *-- Overrides
    ModelTemplates *-- Overrides
    Overrides ..> BlockTemplate : resolve
    BlockTemplate *-- Stylesheet
    ModelAdmin --> ModelTemplates
    AdminState --> PageSheets
```

Priorité, pour une page comme pour un bloc : **modèle > global > Runique**
(`page_for`, `Block::resolve`). Une page ne se remplace par modèle que si
`Page::per_model()` (List, Create, Edit, Detail, Delete) ; sinon refus au boot.
Les feuilles de style sont résolues et hashées **au boot** (`PageSheets`), sans
`integrity` en développement (`without_integrity`), servies par page + ressource
courante via `chrome()`.

### Contrôles des surcharges au boot

Toujours actifs quand l'admin est activé, **quel que soit `RUNIQUE_ENV` ou `DEBUG`**
([`app/builder/build.rs`](../../../runique/src/app/builder/build.rs),
[`assemble.rs`](../../../runique/src/admin/template/assemble.rs)) :

```mermaid
sequenceDiagram
    participant B as build()
    participant A as assemble
    participant T as TemplateLoader
    participant P as Tera vierge

    B->>A: check_replaced_views(models, routes Detail déclarées)
    Note over A: une vue remplacée par template ET par route → refus
    B->>A: dispatch(templates, models) → config.block_dispatch
    Note over A: page globale-seulement remplacée par modèle → refus
    B->>T: init_with_integrity (blocs écrits en include modèle > global > Runique)
    B->>A: missing_pages(templates, models, loaded)
    Note over A: page surchargée (htmx compris) absente de Tera → refus, page + chemin nommés
    B->>T: load_without_replacements (config clonée, block_dispatch vidé)
    T-->>P: pages Runique d'origine
    B->>A: unknown_variables(pristine, loaded, templates, models)
    Note over A: variable lue par une surcharge ∉ clés fournies par la page → refus + clé la plus proche (Levenshtein ≤ 2)
```

Clés permises pour une page : celles que lit la page Runique d'origine
(`get_template_variables` sur le Tera vierge), plus `provided_unread(page)` —
clés de requête (`REQUEST_KEYS`), sections de traduction `admin_<section>_*`,
`page` (List), `resource_key` (Create/Edit/Delete/Rights) — plus `__tera_context`.
Tera refuse déjà lui-même, au chargement, un filtre / test / fonction / composant
inconnu ou un `include` absent ; une variable, il ne la refuse qu'au rendu : d'où ce contrôle.

### Contexte des pages (contrat des templates)

[`admin/router/context.rs`](../../../runique/src/admin/router/context.rs)

```mermaid
classDiagram
    class ListContext {
        +Vec~ListEntry~ entries
        +visible_columns / sortable_columns / column_labels
        +HashMap filter_values  « col → Vec~FilterOption~ »
        +HashMap filter_clear   « col → qs »
        +HashMap active_filters « col → Vec~valeur~ »
        +Pager list_pager
        +HashMap~col,Pager~ filter_pagers
        +Option~Pager~ filter_size_pager
        +u64 total / page / page_count / page_size
        +size_qs / filter_nav_qs / filter_nav / return_qs
    }
    class ListEntry {
        +id / values / links
        +Option~String~ taken_by
        +Option~AccountSummary~ account
    }
    class Pager {
        +u64 page
        +Option~u64~ page_count  « None pour un filtre »
        +Option~String~ prev_qs / next_qs
        +u64 size + size_options + size_param
        +Vec keep  « paramètres en champs cachés »
    }
    class FilterOption {
        +String value
        +bool active
        +String qs  « bascule on/off, page=1 »
    }
    class AccountSummary {
        +&str rank
        +String rank_label
        +Vec~String~ groups
    }
    class DetailContext {
        +Value entry
        +Vec detail_columns  « ordre de la fiche »
        +Option~AccountSummary~ account
        +Option reset_password_url
    }
    ListContext *-- "*" ListEntry
    ListContext *-- "*" Pager
    ListContext *-- "*" FilterOption
    ListEntry *-- "0..1" AccountSummary
    DetailContext *-- "0..1" AccountSummary
```

- Tailles : liste (`per_page`) et filtres (`fsize`) bornées à `PER_PAGE_OPTIONS`
  (10/25/50/100) ; une taille libre de l'URL est ignorée.
- `AccountSummary` (rang + groupes, lecture seule) : chargé par `account_summaries`
  pour la liste et la fiche des comptes, en deux requêtes par page.
- `detail_columns` : colonnes de la liste dans leur ordre, puis celles du formulaire,
  dates et heures en dernier (`ModelAdmin::detail_columns`).
- `admin_page_context(req, admin, site, page, resource)` : une route déclarée `admin`
  reçoit le chrome (feuilles, menu avec droits, barre du haut) sans aucun droit en plus.
- La page de connexion porte un lien « ← Retour au site » (`site_url`, soit `.view_site_url(…)`, défaut `/`).

## Middleware admin

[`admin/middleware/admin_middleware.rs`](../../../runique/src/admin/middleware/admin_middleware.rs)

```mermaid
sequenceDiagram
    participant R as Requête
    participant LU as load_admin_user
    participant AA as admin_access
    participant DR as declared_route_access
    participant H as handler

    R->>LU: session → user_id
    LU->>LU: relit compte + groupes + droits en DB, compare session_version
    Note over LU: pas de cache process-local (AM4 sans objet)
    LU-->>R: Extension~CurrentUser~ si actif + (rang>Lambda ‖ staff)
    R->>AA: can_access_admin ?
    AA-->>AA: sinon redirect login
    AA->>AA: insert AdminConstraint(user)
    Note over AA: le gate lit AdminConstraint, jamais req.user mutable
    R->>DR: routes déclarées `admin` hors préfixe
    R->>H: chrome + CRUD
```

`AdminConstraint` est `pub(crate)` : seul le middleware le pose. Le handler peut
muter `req.user` sans influencer le gate.

## History (audit)

[`auth/history.rs`](../../../runique/src/auth/history.rs) — écrit par `save_as`
**dans la même transaction** que l'écriture (admin et `save_as` projet).

```mermaid
classDiagram
    class ActionLog {
        +Pk user_id
        +&str username / resource_key / object_pk
        +&str object_repr / action
        +Option summary / batch_id
    }
    class HistoryModel {
        +i64 id
        +String resource_key / object_pk / object_repr
        +String action / username
        +Pk user_id
        +NaiveDateTime created_at
        +Option summary / batch_id
    }
    class HistoryScope {
        +&str key + String title
        +ReadableColumns readable
        +bool label_readable
    }
    ActionLog ..> HistoryModel : log_action
    RegisteredModel ..> HistoryScope : history_scope (read plein seulement)
```

- `object_repr` : libellé figé au moment de l'action (lisible après rename/delete), max 255.
- Pas de FK sur `user_id` (audit survit à la suppression du compte) — **D2** à documenter.
- `read_own` seul n'ouvre **pas** l'historique d'une ressource (impossible de savoir
  à qui appartenait une ligne passée).

## Forms internes admin (bulk / confirm)

[`admin/builder/bulk_form.rs`](../../../runique/src/admin/builder/bulk_form.rs),
[`confirm_form.rs`](../../../runique/src/admin/builder/confirm_form.rs)

```mermaid
classDiagram
    class BulkForm {
        +ids + action index
        +from_form / register_fields
    }
    class ConfirmForm {
        +confirmation CSRF-bound
        +from_form / register_fields
    }
    class BulkOutcome {
        <<enum>>
        Updated(u64)
        Deleted(u64)
        Refused(String)
    }
    class FormOutcome {
        <<enum>>
        Show(Value)
        Saved(Option~email~)
    }
    BulkForm ..> BulkOutcome
    ConfirmForm ..> FormOutcome
```

Les actions groupées ne portent **jamais** la valeur à écrire : l'URL/form ne
nomme que l'index d'une `Action` déclarée (colonne + valeur fixées dans le builder).
Sur la table comptes, rangs et `is_active` sont interdits en action groupée.

## Router — pages spéciales

| Route | Rôle |
|-------|------|
| `…/rank` | POST changement de rang (gate + `Rang::cibles_gerables`) |
| `…/{id}/rights` | grille de droits d'un admin ou d'un groupe |
| `…/{id}/groups` | memberships compte ↔ groupes |
| `…/{id}/transfer` | transfert de responsabilité de groupe |
| `…/{id}/reset-password` | lien reset (gate `ResetPassword`, rang protégé) |
| `…/history` | journal filtré par `history_scopes` |

## Anomalies / notes

| ID | Note |
|----|------|
| **D2** | `user_id` history sans FK — voulu (survie audit) ; documenter |
| **D3** | Index manquants possibles (`history` composite, `sessions.user_id`, `reset_tokens.expires_at`) — valider sous DB réelle |
| **AM4** | Cache permissions process-local **supprimé** : droits relus à chaque requête admin |
| Templates | Contrat de contexte : `chrome()` injecte `resources`, `nav`, `current_user`, `stylesheets`, messages i18n ; toute variable lue par une surcharge est vérifiée au boot |
