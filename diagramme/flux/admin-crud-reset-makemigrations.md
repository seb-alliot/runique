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

    H->>H: ListQuery::from_params (page, sort, search, filter_* multiples, per_page, fsize, fpage_*)
    H->>M: list(db, user, query, site_page_size, site_filter_size)
    M->>G: proof List + Scope All|Own
    M->>L: resolve_sort / resolve_search / resolve_filters
    Note over L: colonnes hors config ignorées, tailles hors PER_PAGE_OPTIONS ignorées
    M->>M: SELECT + window __total, colonnes lisibles only
    M->>M: filter_values(scope, col, page, size) par filtre
    M-->>H: ListPage + OfferedAction filtrées par droits
    H->>H: Pager (liste + chaque filtre) + FilterOption (qs on/off réencodé)
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
    M->>VF: try_form (nom renvoyé, CSRF, validate async)
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

Déclenchable depuis le flux public forgot/reset et depuis l'admin
(`ResetPassword` : droit `update` sur les comptes + compte cible de rang
strictement inférieur, tenu dans le `WHERE`).
[`auth/password.rs`](../../runique/src/auth/password.rs)

```mermaid
sequenceDiagram
    participant U as User / Admin
    participant FP as handle_forgot_password | admin reset
    participant RT as reset_token (DB hashé)
    participant RP as handle_password_reset
    participant DB as txn : users + sessions

    U->>FP: POST email  ou  admin POST reset sur compte cible
    FP->>RT: generate() → token brut (mail) + hash + expires_at
    Note over FP: réponse identique si email inconnu (anti-énumération)<br/>mail intégré : nom et lien échappés (email_text)
    U->>RP: GET {reset_route}/{token}
    RP->>RT: resolve(token) → user_id, sans consommer (sinon flash + /)
    U->>RP: POST nouveau mot de passe (PasswordResetForm, .no_hash() pour comparer pw1/pw2)
    RP->>RP: try_form puis PasswordService::new(password_get()).hash
    RP->>DB: begin
    RP->>RT: consume_in_transaction(token) → user_id
    RP->>DB: set_password_and_activate + session_version + 1
    RP->>DB: revoke_in_transaction (sessions du compte)
    RP->>DB: commit puis forget_user_sessions (mémoire)
    Note over RP,DB: user_id vient du TOKEN, jamais de l'URL/form → IDOR-safe
```

- Le hachage suit la stratégie déclarée par `password_init` (`password_get()`), pas
  `config.password` du builder : un projet `Custom` ou `Delegated` garde la sienne.
- Les deux routes sont servies en `Cache-Control: no-store` (jeton dans le chemin et
  dans le CSRF du formulaire) et passent par un `RateLimiter` dédié.
- Lien du mail : `public_link_base(public_url)` seulement, jamais le `Host` de la requête.

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
