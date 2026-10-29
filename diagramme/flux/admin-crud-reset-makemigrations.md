# Flux — Admin CRUD 4.0, reset password, makemigrations

## Admin CRUD (gate + RegisteredModel)

Plus de pattern Command / `ResourceEntry` / closures optionnelles.
Le router pose les layers ; le handler résout le modèle ; **le modèle demande
une preuve au gate** puis lit/écrit sous cette preuve.

[`admin/router/`](../../runique/src/admin/router/),
[`admin/builder/registered.rs`](../../runique/src/admin/builder/registered.rs),
[`auth/gate/`](../../runique/src/auth/gate/)

```mermaid
sequenceDiagram
    participant MW as load_admin_user + admin_access
    participant H as *_view (list/create/edit/…)
    participant S as AdminSite
    participant M as dyn RegisteredModel
    participant G as gate authorize_*
    participant W as save_as / write
    participant DB as DB + history

    MW->>MW: relit compte, rang, groupes, droits
    MW->>MW: AdminConstraint(user) si can_access_admin
    H->>S: model(resource) — 404 si inconnu
    H->>M: list|detail|create|edit|delete|bulk|…
    M->>G: authorize_list / authorize_row / authorize_create / …
    alt Refused
        G-->>H: AdminError::Refused → flash + redirect dashboard
    else Authorized~O,E~
        G-->>M: proof (id + Condition + ReadableColumns)
        M->>W: écriture consomme le proof (WHERE rejoué)
        W->>DB: txn : row ± links + log_action
        M-->>H: ListPage | FormOutcome | BulkOutcome
        H->>H: chrome + template
    end
```

Source unique d'autorisation : le **gate** (plancher Runique → droits rang/groupes
→ règles métier). Les flags UI (`can_edit`, etc.) sont dérivés de
`CrudAction::resolve` pour l'affichage seulement — le serveur re-vérifie toujours.

Voir [../uml/admin/admin-resource-permissions.md](../uml/admin/admin-resource-permissions.md).

### Liste (détail)

```mermaid
sequenceDiagram
    participant H as list_view
    participant M as ModelAdmin~E~
    participant G as authorize_list
    participant L as ListConfig

    H->>H: ListQuery::from_params (page, sort, search, filter_*)
    H->>M: list(db, user, query, site_page_size)
    M->>G: proof List + Scope All|Own
    M->>L: resolve_sort / resolve_search / resolve_filters
    Note over L: colonnes hors config ignorées
    M->>M: SELECT + window __total, colonnes lisibles only
    M-->>H: ListPage + OfferedAction filtrées par droits
```

### Écriture (create / edit)

```mermaid
sequenceDiagram
    participant H as create_view / edit_view
    participant M as ModelAdmin
    participant VF as ValidationForm
    participant G as gate
    participant SA as save_as

    H->>M: create|edit(req, user, id?)
    M->>VF: try_new (CSRF + validate async)
    alt invalide
        M-->>H: FormOutcome::Show(form+errors)
    else valide
        M->>G: authorize_create | authorize_row~Update~
        M->>SA: save_as(proof, form, columns autorisées)
        SA->>SA: txn + history + take_back groupes si besoin
        M-->>H: FormOutcome::Saved(email?)
        H->>H: redirect detail/list (+ mail welcome si compte)
    end
```

### Bulk

La barre n'envoie que `action=aN` + ids cochés. La valeur écrite est celle de
l'`Action` déclarée dans `ListConfig` — jamais fournie par le client.
Au plus une page de lignes ; tout ou rien via le gate.

## Reset password (IDOR-safe, single-use)

Inchangé dans l'esprit ; déclenchable aussi depuis l'admin
(`ResetPassword` + rang protégé) en plus du flux public forgot/reset.

```mermaid
sequenceDiagram
    participant U as User / Admin
    participant FP as handle_forgot_password | reset_password_view
    participant RT as reset_token (DB hashé)
    participant RP as handle_password_reset
    participant DB as BuiltinUserEntity

    U->>FP: POST email  ou  admin POST reset sur compte cible
    FP->>RT: generate() → token brut (mail) + hash + expires_at
    Note over FP: réponse identique si email inconnu (anti-énumération)
    U->>RP: GET /reset?token=…
    RP->>RT: peek(token) → valide sans consommer
    U->>RP: POST nouveau mot de passe
    RP->>RT: consume(token) → user_id (delete + rows_affected==1)
    RP->>DB: update_password_by_id / activate
    Note over RP,DB: user_id vient du TOKEN, jamais de l'URL/form → IDOR-safe
```

## makemigrations (diff complet + commit atomique)

```mermaid
sequenceDiagram
    participant CLI as runique makemigrations
    participant P as runique_dsl (model!/extend!)
    participant DS as diff_schemas
    participant PL as build_plan
    participant CM as commit_plan (atomique)

    CLI->>P: parse sources → ParsedModel (erreur = arrêt, fichier:ligne:colonne)
    CLI->>DS: diff_schemas(previous, current)
    DS-->>CLI: Changes { added, dropped, MODIFIED, fk, index, enum_rename }
    Note over DS: modified_columns géré (ALTER COLUMN émis)
    CLI->>PL: build_plan (CREATE/ALTER/DROP, index FK via unless_mysql_fk, CycleKeys)
    CLI->>CM: commit_plan — snapshots + rollback atomique
```

Tables framework (`eihwaz_*`) : migrations dédiées dans
`auth/migrations_table.rs`, branchées par le `Migrator` généré du projet —
pas via le DSL `model!{}`.

## Anomalies liées

| ID | Note |
|----|------|
| A1/A2/A3 | architecture 3.x — sans objet en 4.0 |
| Gate TOCTOU | 0 row sur WHERE rejoué → refus, pas d'écriture |
| D3 | index purge/requête à valider sous DB |
| S1 | CSRF forms publics vs sessions anonymes non persistées |
