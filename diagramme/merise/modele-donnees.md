# Merise — Modèle de données framework (`eihwaz_*`)

Tables natives du framework, définies dans
[`runique/src/auth/migrations_table.rs`](../../runique/src/auth/migrations_table.rs)
(déplacées hors `admin/` en 4.0 : nécessaires avec ou sans feature `admin`).
La table utilisateur est toujours `eihwaz_users` : les FK du framework la visent
directement. `history` est créée ici ; son usage UI reste côté admin.

## MCD (conceptuel)

```mermaid
erDiagram
    USER ||--o{ USER_GROUPE : "appartient à"
    GROUPE ||--o{ USER_GROUPE : "regroupe"
    GROUPE ||--o{ GROUPE_DROIT : "possède (table/colonne)"
    USER ||--o{ GROUPE_DROIT : "grille admin (user_id)"
    USER ||--o{ GROUPE : "responsable de"
    USER ||--o{ SESSION : "ouvre"
    USER ||--o{ RESET_TOKEN : "demande"
    USER }o..o{ HISTORY : "trace (réf. faible)"

    USER {
        pk id
        string username UK
        string email UK
        string password
        bool is_active
        bool is_dirigeant
        bool is_admin
        bool is_superuser
        datetime created_at
        datetime updated_at
        datetime activated_at
    }
    GROUPE {
        int id PK
        string nom UK
        bool is_staff
        bool is_active
        pk responsable_id FK
        pk repris_par FK
    }
    GROUPE_DROIT {
        int id PK
        int groupe_id FK
        pk user_id FK
        string resource_key
        string colonne
        bool can_create
        bool can_read
        bool can_read_own
        bool can_update
        bool can_update_own
        bool can_delete
        bool can_delete_own
    }
    USER_GROUPE {
        pk user_id FK
        int groupe_id FK
    }
    SESSION {
        int id PK
        string cookie_id UK
        pk user_id FK
        string session_id
        text session_data
        datetime expires_at
    }
    HISTORY {
        bigint id PK
        string resource_key
        string object_pk
        string object_repr
        string action
        pk user_id
        string username
        datetime created_at
        text summary
        string batch_id
    }
    RESET_TOKEN {
        bigint id PK
        string token_hash UK
        pk user_id FK
        datetime expires_at
    }
```

## MLD (logique relationnel)

| Table | PK | Colonnes | FK | Contraintes |
|-------|----|----------|----|-------------|
| `eihwaz_users` | `id` (int/bigint/uuid¹) | username, email, password, is_active, **is_dirigeant**, **is_admin**, is_superuser, created_at?, updated_at?, activated_at? | — | username UNIQUE, email UNIQUE ; CHECK `ACTIVE_NEEDS_ACTIVATION` ; CHECK `ONE_RANK_AT_MOST` |
| `eihwaz_groupes` | `id` (int) | nom, **is_staff**, **is_active**, **responsable_id**, **repris_par** | responsable_id → users.id **RESTRICT** ; repris_par → users.id **SET NULL** | nom UNIQUE |
| `eihwaz_groupes_droits` | `id` (int) | groupe_id?, user_id?, resource_key, **colonne** (défaut `""`), can_create/read/**read_own**/update/update_own/delete/delete_own | groupe_id → groupes.id **CASCADE** ; user_id → users.id **CASCADE** | CHECK un seul holder ; CHECK groupe ≠ framework keys ; CHECK ligne colonne = read/create/update only ; UNIQUE (groupe, resource, colonne) ; UNIQUE (user, resource, colonne) |
| `eihwaz_users_groupes` | (`user_id`,`groupe_id`) | — | user_id → users.id **CASCADE**, groupe_id → groupes.id **CASCADE** | PK composite |
| `eihwaz_sessions` | `id` (int) | cookie_id, user_id, session_id, session_data?, expires_at | user_id → users.id **CASCADE** | cookie_id UNIQUE |
| `eihwaz_history` | `id` (bigint) | resource_key, object_pk, **object_repr**, action, user_id, username, created_at, summary?, batch_id? | **aucune** | object_repr tronqué à 255 |
| `eihwaz_reset_tokens` | `id` (bigint) | token_hash, user_id, expires_at | user_id → users.id **CASCADE** | token_hash UNIQUE |

¹ `id` / `user_id` = `uuid` si `pk-uuid`, sinon `bigint` si `big-pk`, sinon `integer`.
`eihwaz_groupes.id` et `groupe_id` restent **toujours** `integer` (indépendants de `Pk`).

## Évolutions 4.0 (par rapport aux diagrammes 3.x)

| Avant (3.x) | Après (4.0) |
|-------------|-------------|
| `users.is_staff` | **supprimé** — staff = `groupes.is_staff` |
| rang binaire staff/superuser | **4 rangs** : Lambda / Admin / Dirigeant / Superuser (`is_admin`, `is_dirigeant`, `is_superuser`, au plus un) |
| `groupes` = id + nom | + `is_staff`, `is_active`, `responsable_id`, `repris_par` |
| `groupes_droits` PK (`groupe_id`,`resource_key`) | PK `id` ; holder groupe **ou** compte ; granularité **colonne** ; `can_read_own` |
| history sans `object_repr` | + `object_repr` (libellé figé) |
| migrations dans `admin/table_admin/` | `auth/migrations_table.rs` |

## Anomalies / dette données

| ID | Anomalie | Statut |
|----|----------|--------|
| **D1** | `users_groupes.user_id` hors cfg big-pk | ✅ corrigé (`user_pk_reference`) |
| **D2** | `history.user_id` sans FK | voulu (audit) — à documenter côté produit |
| **D3** | index manquants possibles (history composite, sessions.user_id, reset_tokens.expires_at) | ouvert — valider sous DB réelle |
| **D4/S4** | double id session `cookie_id` / `session_id` | à documenter |
| **D5** | table fantôme `eihwaz_droits` | ✅ supprimée |
| Rank CHECK | `ONE_RANK_AT_MOST` + `ACTIVE_NEEDS_ACTIVATION` | ✅ en base (pas seulement ORM) |
