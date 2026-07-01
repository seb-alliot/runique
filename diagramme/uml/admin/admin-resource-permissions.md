# UML — Admin : registry, resource, permissions, dispatch

## Registry & ResourceEntry

[`registry.rs`](../../../runique/src/admin/registry.rs),
[`helper/resource_entry.rs`](../../../runique/src/admin/helper/resource_entry.rs)

```mermaid
classDiagram
    class AdminRegistry {
        +IndexMap~String, ResourceEntry~ resources
        +register(entry)
        +get(key) Option~&ResourceEntry~
        +configure(key, DisplayConfig)
        +configure_group_actions(key, actions)
        +remove(key) / reorder(order)
    }
    class ResourceEntry {
        +AdminResource meta
        +FormBuilder form_builder
        +Option~FormBuilder~ edit_form_builder
        +Option~ListFn~ list_fn
        +Option~GetFn~ get_fn
        +Option~DeleteFn~ delete_fn
        +Option~UpdateFn~ update_fn
        +Option~UpdateFn~ partial_update_fn
        +Option~CreateFn~ create_fn
        +Option~CountFn~ count_fn
        +Option~FilterFn~ filter_fn
        +Vec~GroupAction~ group_actions
        +Option~M2mLoaderFn~ m2m_loader
        +&[&str] unique_fields
        +Option~&str~ own_field
    }
    class AdminResource {
        +&str key
        +DisplayConfig display
        +Option~String~ template_list/create/edit/detail/delete
        +HashMap~String,String~ extra_context
        +Vec~(col,fk_table,label)~ fk_display
        +Option~ParentScope~ parent_scope
        +parent_scope(parent_key, fk_col, local_key) Self
    }
    class ParentScope {
        +&str parent_key
        +&str fk_col
        +Option~&str~ local_key
        +is_composite() bool
    }
    class DisplayConfig {
        +Option~String~ icon
        +ColumnFilter columns
        +usize pagination
        +Vec~(String,String,u64)~ list_filter
    }
    AdminRegistry "1" o-- "*" ResourceEntry
    ResourceEntry "1" *-- "1" AdminResource
    AdminResource "1" *-- "1" DisplayConfig
    AdminResource "1" o-- "0..1" ParentScope
```

`ParentScope` déclare une resource comme **enfant scopé** d'une autre (atteinte via
`/{parent}/{parent_id}/{child}/…`) : la liste enfant est filtrée `WHERE fk_col = parent_id`,
le formulaire fixe/masque `fk_col`, et le détail parent la rend en sous-liste inline.
`local_key = Some(col)` = enfant composite (jonction, closure-id `"{parent_id}:{col}"`,
ex. `groupes_droits`) ; `None` = enfant à PK propre. Un enfant scopé reste visible au
top-level **et** inline (les deux coexistent) ; il n'est pas route-bloqué.

`FormBuilder`/`ListFn`/`GetFn`/`DeleteFn`/`UpdateFn`/`CreateFn`/`CountFn`/`FilterFn`/
`M2mLoaderFn` = closures `Arc<dyn Fn(...) -> BoxFuture<...>>` (effacement de type pour
stocker des resources hétérogènes dans une seule map → pattern Strategy + type erasure).

## Permissions (RBAC) & dispatch

[`permissions/mod.rs`](../../../runique/src/admin/permissions/mod.rs),
[`admin_main/action.rs`](../../../runique/src/admin/admin_main/action.rs)

```mermaid
classDiagram
    class CurrentUser {
        +Pk id
        +String username
        +bool is_staff
        +bool is_superuser
        +Vec~Groupe~ groupes
        +permission_for(key) Option~Permission~
        +permissions_effectives() Vec~Permission~
        +can_access_resource(key) bool
    }
    class Groupe {
        +Pk id
        +String nom
        +Vec~Permission~ permissions
    }
    class Permission {
        +String resource_key
        +bool can_create/read/update/delete
        +bool can_update_own/delete_own
        +merge_from(other)
    }
    class ResourcePerms {
        +bool can_create/read/update/delete
        +bool can_update_own/delete_own
        +resolve(user, key) Self
        +can_edit(owns) bool
        +can_remove(owns) bool
    }
    class CollectionAction {
        <<enum>> List / Create / Bulk
        +authorize_get(perms) Access
        +authorize_post(perms, bulk) Access
    }
    class MemberAction {
        <<enum>> Detail / Edit / Delete / ResetPassword
        +authorize(perms, owns) Access
    }
    CurrentUser "1" o-- "*" Groupe
    Groupe "1" o-- "*" Permission
    ResourcePerms ..> CurrentUser : resolve()
    CollectionAction ..> ResourcePerms
    MemberAction ..> ResourcePerms
```

## Anomalies / flux suspects

### 🟠 A1 — Toutes les closures CRUD sont `Option` → no-op silencieux
`list_fn`, `get_fn`, `create_fn`, `update_fn`, `delete_fn`… sont `Option`. Les handlers font
`match &entry.get_fn { Some(f) => …, None => None }`. Si une resource est enregistrée sans
sa closure (bug de génération du daemon), l'action **échoue en silence** (page vide / pas
d'erreur) au lieu de 501/500. À tracer : un `None` inattendu devrait logger/erreur, pas
dégrader silencieusement. (Lié à la règle « zéro erreur avalée ».)

### 🟠 A2 — `own_field = None` bloque les permissions `*_own` (sain) mais silencieux
[`resource_entry.rs:162`](../../../runique/src/admin/helper/resource_entry.rs#L162)
Si `own_field` n'est pas déclaré, `check_owns_record` renvoie toujours `false` → les droits
`can_update_own`/`can_delete_own` sont **inopérants**. C'est le défaut sûr, mais un admin qui
coche « modifier les siens » sans `own_field` déclaré ne verra **aucun effet ni avertissement**.
Candidat à un warning au boot (validation de cohérence resource).

### 🟠 A3 — `list_filter` dans `configure {}` builtin → 500 (bug connu)
Bug déjà répertorié : `list_filter` dans le bloc `configure {}` des resources builtin
provoque `Variable filter_values[col] not found in context`. `DisplayConfig.list_filter`
existe, mais le flux `configure` ne pousse pas `filter_values` dans le contexte. À confirmer
dans le flux liste admin et relier au bug connu.

### 🟡 A4 — `bulk POST` exige `can_create` (quirk préservé)
[`admin_main/action.rs`](../../../runique/src/admin/admin_main/action.rs) `authorize_post` :
un bulk edit/delete exige `can_create` **en plus** du droit d'opération. Comportement
historique préservé volontairement, mais probablement une sur-restriction non intentionnelle.
À trancher.
