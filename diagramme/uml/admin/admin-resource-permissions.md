# UML — Admin 4.0 : ModelAdmin, AdminSite, gate, droits

Refonte 4.0 : plus de `AdminRegistry` / `ResourceEntry` / closures CRUD optionnelles.
Un modèle s'enregistre via un builder typé `ModelAdmin<E: ModelMeta>` ; le site
efface le type derrière `dyn RegisteredModel` ; **chaque écriture passe par le gate**
(`auth::gate`) qui délivre une preuve `Authorized<O, E>` consommée une seule fois.

Sources :
[`admin/builder/`](../../../runique/src/admin/builder/),
[`admin/router/`](../../../runique/src/admin/router/),
[`auth/gate/`](../../../runique/src/auth/gate/),
[`auth/meta.rs`](../../../runique/src/auth/meta.rs),
[`auth/permissions/`](../../../runique/src/auth/permissions/)

## Vue d'ensemble

```mermaid
classDiagram
    class AdminStaging {
        +AdminConfig config
        +bool enabled
        +AdminSite site
        +register(ModelAdmin) Self
        +accounts_users(ModelAdmin) Self
        +path / prefix / resource_order / templates…
        +validate() CheckReport
    }
    class AdminSite {
        -Vec~Box~RegisteredModel~~ models
        -Box~RegisteredModel~ accounts
        -Box~RegisteredModel~ groups
        -HashMap overrides
        +register(ModelAdmin) Self
        +accounts_users(ModelAdmin) Self
        +model(key) Option~&dyn RegisteredModel~
        +navigation(user, order) Vec~NavEntry~
        +history_scopes(db, user) Vec~HistoryScope~
        +check() Vec~String~
    }
    class ModelAdmin~E: ModelMeta~ {
        +ListConfig~E~ list
        +FormConfig~E~ form
        +Option~String~ title
        +ModelTemplates templates
        +new() / title() / list() / form() / templates()
        +validate() Result
    }
    class RegisteredModel {
        <<trait async>>
        +key() / pk_column() / title() / check()
        +visible_to(user) bool
        +rights(user) CrudAction
        +list / detail / create / edit / delete / bulk
        +reset_password / history_scope / count_query
        +keeps_account_rules() bool
    }
    class ModelMeta {
        <<trait>>
        +const KEY
        +const OWNER / SECRET / LABEL
        +const IS_ACCOUNT_TABLE / IS_FRAMEWORK_TABLE
        +type Form
        +schema() ModelSchema
    }
    AdminStaging *-- AdminSite
    AdminStaging *-- AdminConfig
    AdminSite o-- "*" RegisteredModel
    ModelAdmin ..|> RegisteredModel : impl for E
    ModelAdmin ..> ModelMeta : E:
```

`AdminSite::default()` pose déjà l'admin des comptes (`eihwaz_users`) et des groupes
(`eihwaz_groupes`). Un projet ne peut remplacer les comptes que via
`accounts_users()` (un `register` sous la même clé est refusé au boot).

## ListConfig & FormConfig (déclaratif, typé colonnes)

```mermaid
classDiagram
    class ListConfig~E~ {
        -Vec~Column~ display / links / sortable / filter / search
        -Vec~(Column,Order)~ ordering
        -Option~u64~ per_page
        -Vec~Action~ actions
        +display / links / sortable / ordering / search / filter
        +per_page / action / empty_value / search_help
        +validate() / select(proof) / resolve_*
    }
    class Action~E~ {
        +Column col
        +Value value
        +String label
    }
    class ListQuery {
        +u64 page
        +Option sort_by / sort_dir / search
        +Vec~(col,value)~ filters
        +from_params(StrMap, max_len)
    }
    class FormConfig~E~ {
        -Vec~Column~ fields / readonly / create_only
        -Vec~(Column,String)~ labels
        +fields / readonly / create_only / labels
        +writable_on_create / writable_on_edit
        +read_columns() / validate()
    }
    class ListPage {
        +Vec columns / sortable / rows
        +u64 total / page / page_size
        +Vec~ListFilter~ filters
        +Vec~OfferedAction~ actions
    }
    ListConfig --> Action
    ListConfig ..> ListQuery : résout
    ModelAdmin *-- ListConfig
    ModelAdmin *-- FormConfig
    RegisteredModel ..> ListPage : list()
```

Rien de l'URL n'est crédible : `ListQuery` ne fait que parser ; `ListConfig` décide
quelles colonnes un nom peut atteindre (tri, filtre, recherche). Les secrets
(`ModelMeta::SECRET`) et les flags de rang (`is_superuser` / `is_dirigeant` /
`is_admin`) sont refusés dès `ModelAdmin::validate()`.

## Gate — preuves typées (source unique d'autorisation)

```mermaid
classDiagram
    class GateCtx {
        +&ADb db
        +&CurrentUser user
    }
    class AdminOperation {
        <<trait>>
        +const CRUD_NATURE
        +const RIGHT / RIGHT_OWN
        +const PROTECTS_RANK
    }
    class List
    class View
    class Create
    class Update
    class Delete
    class ResetPassword
    class Authorized~O,E~ {
        -Option~Pk~ id
        -Condition conditions
        -ReadableColumns readable
        +id() / readable()
        +into_parts()  « consommé »
    }
    class ReadableIds~E~
    class WritesLinks~E~
    class Refusal {
        +RefusalReason reason
        +message() String
    }
    class CrudAction {
        +can_create/read/read_own
        +can_update/update_own
        +can_delete/delete_own
        +is_superuser
        +resolve(user, key) Self
        +has(Crud) bool
    }
    class Scope~E~ {
        <<enum>>
        All
        Own(Column, Pk)
    }
    AdminOperation <|.. List
    AdminOperation <|.. View
    AdminOperation <|.. Create
    AdminOperation <|.. Update
    AdminOperation <|.. Delete
    AdminOperation <|.. ResetPassword
    GateCtx ..> Authorized : authorize_*
    Authorized ..> Refusal : Err
    CrudAction ..> Scope
```

Trois niveaux, dans l'ordre (tout « non » → même `Refusal`) :

1. **Plancher Runique** — tables framework, secrets, comptes superuser, rangs ;
2. **Droits du compte** — rang ou groupes (`CrudAction` / `ColumnRights`) ;
3. **Règles métier du projet** — ne peuvent que restreindre.

`Authorized` n'est ni `Clone` ni `Copy` : une écriture le consomme. Son
constructeur est `pub(in crate::auth::gate)` — un court-circuit hors du module
apparaît comme un changement de visibilité en revue.

## Rang & résolution des droits

```mermaid
classDiagram
    class Rang {
        <<enum>>
        Lambda
        Admin
        Dirigeant
        Superuser
        +of(flags) Self
        +cibles_gerables() Option~Condition~
    }
    class CurrentUser {
        +Pk id
        +String username
        +Rang rang
        +Vec~Groupe~ groupes
        +Vec~Permission~ droits
        +rights_on(key) Rights
        +can_access_admin() bool
        +is_staff() bool
    }
    class Rights {
        +CrudAction crud
        +ColumnRights columns
    }
    class ColumnRights {
        <<enum>>
        All
        Only(Vec~ColumnRight~)
        +readable() ReadableColumns
        +can_create/can_update(col)
    }
    class Permission {
        +resource_key
        +can_* flags
        +Vec~ColumnRight~ colonnes
    }
    CurrentUser --> Rang
    CurrentUser --> Permission
    CurrentUser ..> Rights : rights_on
    Rights *-- CrudAction
    Rights *-- ColumnRights
```

| Rang | Source des droits |
|------|-------------------|
| Superuser | tout (`is_superuser` bypass) |
| Dirigeant | tout sauf bypass superuser |
| Admin | grille posée sur **son compte** (`droits`, `user_id` dans `eihwaz_groupes_droits`) |
| Lambda | union des **groupes** ; **jamais** sur une table framework |

`is_staff` n'est plus un flag user : c'est un flag **groupe**. Un lambda entre
dans l'admin s'il appartient à au moins un groupe `is_staff`.

## Router admin (routes générées + couches)

```mermaid
classDiagram
    class AdminPaths {
        +login / dashboard / logout
        +list / detail / create / edit / delete
        +bulk / rank / reset_password
        +rights / groups / transfer / history…
        +list_of(key) / detail_of(site,key,id)
    }
    class AdminState {
        +Arc~AdminConfig~ config
        +Option~Arc~LoginGuard~~ login_guard
        +AdminPaths paths
        +Arc~PageSheets~ sheets
    }
    class AdminView {
        <<enum>>
        Detail
        Standalone
    }
    class build_admin_router {
        <<fn>>
        login public + CRUD protégé
        + layers : load_user, admin_access, noindex, no-store
    }
    build_admin_router ..> AdminState
    build_admin_router ..> AdminPaths
    build_admin_router ..> AdminSite
```

Routes modèle (extrait) : `{prefix}/{resource}/list|create|bulk`,
`{prefix}/{resource}/{id}/detail|edit|delete|reset-password|rights|groups|transfer`,
plus history, rank, group_select. Les handlers ne font que router : auth,
`CurrentUser`, `AdminConstraint` et chrome viennent des layers.

## Flux d'une page liste (résumé)

```mermaid
sequenceDiagram
    participant MW as admin_access
    participant H as list_view
    participant S as AdminSite
    participant M as dyn RegisteredModel
    participant G as gate (authorize_list)
    participant DB as DB

    MW->>MW: load_admin_user + can_access_admin
    MW->>MW: insert AdminConstraint(user)
    H->>S: model(resource)
    H->>M: list(db, user, ListQuery, page_size)
    M->>G: authorize_list → Authorized~List,E~
    G-->>M: proof + Scope + ReadableColumns
    M->>DB: SELECT filtré (conditions du proof)
    M-->>H: ListPage
    H->>H: chrome + template list
```

## Anomalies / flux suspects

| ID | Observation | Statut |
|----|-------------|--------|
| **A1/A2** (closures Option no-op / own_field) | Architecture 3.x — **disparue**. Plus de closures CRUD ; ownership = `ModelMeta::OWNER` + preuves gate | ✅ sans objet en 4.0 |
| **A3** (`list_filter` builtins sans `filter_fn`) | Plus de builtins manuels ni `filter_fn` : filtres = `ListConfig::filter` + valeurs lues en base sous le scope du proof | ✅ sans objet |
| **A7** (pas de `get_queryset`) | Toujours ouvert côté design : le gate accepte des règles métier restrictives, pas encore d'API publique « queryset » arbitraire type Django | 🟡 backlog |
| **Gate** | Preuve non clonable + `WHERE` rejoué à l'écriture → TOCTOU d'une ligne concurrente = refus (0 row), pas d'écriture fantôme | design voulu |
| **can_read_own** | Colonne présente en base ; résolution rang/grille opérationnelle. Vérifier la couverture UI grille si un admin doit la cocher | à surveiller |

Voir aussi [admin-complements.md](admin-complements.md) (templates, history, middleware)
et [../auth/authentification.md](../auth/authentification.md).
