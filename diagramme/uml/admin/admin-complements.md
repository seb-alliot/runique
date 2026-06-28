# UML — Admin : daemon, helper, forms admin, history

Complément de [admin-resource-permissions.md](admin-resource-permissions.md).

## Helper (templates, dyn form, params liste)

[`admin/helper/`](../../../runique/src/admin/helper/)

```mermaid
classDiagram
    class DynForm {
        <<trait>>
        +is_valid() async bool
        +save(db) async Result
        +get_form() / get_form_mut()
    }
    class AdminTemplate {
        +PathAdminTemplate list/create/edit/detail/delete/dashboard/login…
        +resolve()
    }
    class PathAdminTemplate {
        +Option~String~ dev
        +&str runique
        +resolve() &str
    }
    class ListParams {
        +u64 offset / limit
        +Option~String~ sort_by
        +SortDir sort_dir
        +Option~String~ search
        +Vec~(String,String)~ column_filters
    }
    class SortDir { <<enum>> Asc / Desc }
    class M2mFieldOptions { +field_name +label +choices +selected }
    class GroupAction { +field +label +choices }
    AdminTemplate *-- "*" PathAdminTemplate
    ListParams *-- SortDir
```

`DynForm` = effacement de type pour stocker des forms hétérogènes dans `ResourceEntry`
(le `FormBuilder` renvoie `Box<dyn DynForm>`). `PathAdminTemplate.dev` permet d'override le
template Runique par un template projet (dev prioritaire).

## Forms admin builtin

[`admin/forms/mod.rs`](../../../runique/src/admin/forms/mod.rs)

```mermaid
classDiagram
    class DroitAdminForm
    class GroupeAdminForm
    class UserAdminCreateForm
    class UserAdminEditForm
    note for UserAdminEditForm "password skippé par fill() → add_value() (hash pré-calculé)"
```

## History (audit `eihwaz_history`)

[`admin/history.rs`](../../../runique/src/admin/history.rs)

```mermaid
classDiagram
    class HistoryModel {
        +i64 id
        +String resource_key / object_pk / action
        +Pk user_id
        +String username
        +NaiveDateTime created_at
        +Option~String~ summary / batch_id
    }
    class AdminActionLog {
        +Pk user_id
        +&str username / resource_key / object_pk / action
        +Option~String~ summary / batch_id
    }
    AdminActionLog ..> HistoryModel : log_admin_action() insert
```

## Daemon admin (génération de code)

[`admin/daemon/`](../../../runique/src/admin/daemon/) — fonctions (pas de struct) :

```mermaid
flowchart LR
    SRC[src/admins/admin.rs admin!{}] --> PAR[parser.rs]
    PAR --> GEN[generator.rs → src/admins/generated.rs]
    GEN --> REG[AdminRegistry au boot]
    WAT[watcher.rs] -->|hot-reload dev| GEN
```

## Anomalies / flux suspects

### 🟢 `log_admin_action` désormais tracé (rappel)
L'insert audit loggue son échec (`warn!`) au lieu de l'avaler — cf. correctifs tracing.

### 🟡 Rappel A1/A2 — closures `Option` + `own_field`
Voir [admin-resource-permissions.md](admin-resource-permissions.md) : closures CRUD `Option`
(no-op silencieux) et `own_field = None` (droits `*_own` inopérants sans warning).
