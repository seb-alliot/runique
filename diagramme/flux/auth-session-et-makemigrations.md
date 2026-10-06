# Flux — Login/session & makemigrations

## Séquence : login authentifié

[`auth/session.rs:291`](../../runique/src/auth/session.rs#L291)

```mermaid
sequenceDiagram
    participant H as Handler login
    participant S as Session (tower)
    participant M as CleaningMemoryStore
    participant DB as RuniqueSessionStore

    H->>S: existing? logout si user différent
    H->>S: cycle_id() si élévation de privilège (anti-fixation)
    H->>S: insert(user_id, username, is_staff, is_superuser)
    H->>S: set_expiry(24h)
    H->>S: save() → persist_to_db → upsert_session
    Note over M,DB: upsert INSERT (session_id=uuid_B)<br/>on_conflict update [SessionData, ExpiresAt]
    H->>DB: create(cookie_id, user_id, session_id=uuid_A, expires)
    Note over DB: create arrive en CONFLIT (ligne déjà insérée)<br/>on_conflict update [UserId, ExpiresAt] — PAS session_id<br/>⇒ session_id final = uuid_B, déterministe (pas de divergence)
    alt exclusive_login
        H->>DB: invalidate_other_sessions(user_id, keep=cookie_id)
    end
```

> **Lecture du diagramme** : les deux écritures ont des `on_conflict.update_columns`
> disjoints et **aucune** ne touche `session_id` sur conflit. `upsert` (1ʳᵉ, via `save()`)
> fixe `session_id` ; `create` (2ᵉ) ne fait qu'un UPDATE de `[UserId, ExpiresAt]`. Donc
> `session_id` est cohérent — le seul coût réel est **2 aller-retours DB** (AM2 = perf, pas
> divergence). C'est précisément le genre de faux positif qu'un flux complet (avec les
> `update_columns`) élimine d'emblée.

## Séquence : makemigrations (diff)

Chemin réel (3.0.0, 2026-10-06) — le `ModelSchema::diff` du schéma initial n'est pas sur ce
chemin (voir AM1) :

```mermaid
sequenceDiagram
    participant CLI as runique makemigrations
    participant DSL as runique_dsl (même parseur que la macro)
    participant SC as scan_entities
    participant SN as snapshots (parser_seaorm)
    participant D as diff_schemas
    participant G as generators SeaQuery

    CLI->>DSL: model!{} / extend!{} de chaque fichier
    DSL-->>CLI: ParsedModel (schéma + tables des listes) ou erreur fichier:ligne:colonne
    CLI->>SC: résout les cibles belongs_to (vraie table + PK)
    CLI->>SN: snapshot précédent (longueurs adoptées si ancien)
    SN->>D: previous
    SC->>D: current
    D-->>G: Changes { added, dropped, modified (type/nullable/unique/default/longueur), renamed, fk, index, enums }
    G-->>CLI: CREATE (FK inline, cycles par moteur à l'exécution) / ALTER / index
```

## Anomalies / flux suspects

### ❌ AM1 — FAUX POSITIF (makemigrations gère bien les `ALTER COLUMN`)
La CLI utilise `diff_schemas` ([makemigration.rs:489](../../runique/src/cli/makemigration.rs#L489))
qui calcule `modified_columns`. Le `ModelSchema::diff` limité (add/drop) n'est **pas** le
chemin de la CLI. Détecté en traçant le flux jusqu'au vrai `diff` appelé.

### ❌ AM2 — FAUX POSITIF sur la divergence (résidu = perf seulement)
Voir le diagramme ci-dessus : les `on_conflict.update_columns` sont disjoints et **aucun** ne
touche `session_id`. `upsert` (1ʳᵉ) le fixe, `create` (2ᵉ) ne fait qu'un UPDATE
`[UserId, ExpiresAt]`. `session_id` est donc déterministe — pas de divergence. Résidu réel :
**double aller-retour DB** au login (🟡 perf, fusionnable).

### 🟡 AM3 — TTL 24h codé en dur en double — ✅ CORRIGÉ
Extrait en constante unique `AUTH_SESSION_TTL_HOURS` (cookie + DB).

### 🟡 AM4 — `pull_groupes_db` à chaque login + cache mémoire process-local — ✅ CORRIGÉ (2026-10-03)
Cache supprimé. L'état du compte et les droits sont relus en base à chaque requête admin
(`load_admin_user`) ; la session ne garde que l'identité.
