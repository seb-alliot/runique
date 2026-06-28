# Flux — Admin CRUD, reset password, makemigrations

## Admin CRUD (pattern Command)

[`admin/admin_main/`](../../runique/src/admin/admin_main/)

```mermaid
sequenceDiagram
    participant MW as admin_required (middleware)
    participant E as admin_get/post[_id]
    participant IC as inject_context
    participant ACT as CollectionAction/MemberAction
    participant EN as enforce
    participant H as handle_* (closure resource)

    MW->>E: requête /admin/{res}/{action}
    E->>E: registry.get(res) → ResourceEntry (404 sinon)
    E->>IC: inject_context → ResourcePerms
    E->>ACT: parse (404 si action invalide pour la méthode)
    ACT->>ACT: authorize(perms, owns) → Access
    E->>EN: enforce(Access) → redirect si Denied
    Note over EN: Granted → continue
    E->>H: dispatch (list/create/edit/delete/bulk/reset-pwd)
    H->>H: get_fn/list_fn/create_fn… (Option!)
```

Source unique d'autorisation : `ResourcePerms` (template + serveur). Voir
[../uml/admin/admin-resource-permissions.md](../uml/admin/admin-resource-permissions.md).

## Reset password (IDOR-safe, single-use)

```mermaid
sequenceDiagram
    participant U as User
    participant FP as handle_forgot_password
    participant RT as reset_token (DB hashé)
    participant RP as handle_password_reset
    participant DB as UserEntity

    U->>FP: POST email
    FP->>RT: generate() → token brut (mail) + hash en DB + expires_at
    Note over FP: réponse identique si email inconnu (anti-énumération)
    U->>RP: GET /reset?token=…
    RP->>RT: peek(token) → valide sans consommer
    U->>RP: POST nouveau mot de passe
    RP->>RT: consume(token) → user_id (delete + rows_affected==1)
    RP->>DB: update_password_by_id(user_id, hash)
    Note over RP,DB: user_id vient du TOKEN, jamais de l'URL/form → IDOR-safe
```

## makemigrations (diff complet + commit atomique)

```mermaid
sequenceDiagram
    participant CLI as runique makemigrations
    participant P as parser (model!/extend!)
    participant DS as diff_schemas
    participant PL as build_plan
    participant CM as commit_plan (atomique)

    CLI->>P: parse sources → ParsedSchema
    CLI->>DS: diff_schemas(previous, current)
    DS-->>CLI: Changes { added, dropped, MODIFIED, fk, index, enum_rename }
    Note over DS: ✅ modified_columns géré (ALTER COLUMN émis)
    CLI->>PL: build_plan (SQL CREATE/ALTER/DROP + triggers updated_at)
    CLI->>CM: commit_plan — snapshots + rollback atomique si échec
```

## Anomalies / flux suspects

### 🟠 ACR1 — Closures resource `Option` : no-op silencieux (rappel A1)
Si `get_fn`/`create_fn`/… est `None` (bug daemon), l'action dégrade en silence (page vide)
au lieu d'erreur. À durcir (501/log).

### 🟢 ACR2 — Reset password : audit clean (pas d'anomalie)
Token haché, single-use strict, `user_id` dérivé du token, anti-énumération sur forgot,
logout avant reset. Chaîne saine.

### 🟢 ACR3 — makemigrations : diff complet (confirme M1 = faux positif)
`diff_schemas` gère added/dropped/**modified**/fk/index/enum-rename ; commit atomique avec
rollback. Le `ModelSchema::diff` limité (add/drop) n'est pas sur ce chemin.
