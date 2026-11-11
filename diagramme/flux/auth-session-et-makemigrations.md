# Flux — Login/session & makemigrations

## Séquence : login authentifié

[`auth/session.rs:192`](../../runique/src/auth/session.rs#L192)

```mermaid
sequenceDiagram
    participant H as Handler login
    participant S as Session (tower)
    participant M as CleaningMemoryStore
    participant DB as RuniqueSessionStore

    H->>S: existing? logout si user différent
    H->>S: cycle_id() si élévation de privilège (anti-fixation)
    H->>S: insert(session_version, user_id)
    Note over S: l'identité seule : le compte est relu à chaque requête
    H->>S: set_expiry(24h)
    H->>S: save() → persist_to_db → upsert_session
    Note over M,DB: upsert INSERT (session_id=uuid_B, session_data chiffré)<br/>on_conflict update [SessionData, ExpiresAt]
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

## Séquence : fermeture des sessions d'un compte

[`auth/session.rs:380`](../../runique/src/auth/session.rs#L380),
[`auth/password.rs`](../../runique/src/auth/password.rs)

```mermaid
sequenceDiagram
    participant R as Reset réussi / close_user_sessions
    participant U as eihwaz_users
    participant S as eihwaz_sessions
    participant V as Requête volée (cookie d'avant)
    participant A as auth_middleware

    R->>U: session_version + 1 (dans la transaction du reset)
    R->>S: supprime les lignes du compte
    Note over R: refus de la base : tout est annulé, sessions intactes
    V->>A: requête
    A->>U: relit le compte (déjà fait à chaque requête)
    alt version de la session différente
        A->>A: logout, request.user = None
    else même version
        A->>A: CurrentUser injecté
    end
```

`update_password_by_id` n'augmente pas la version : le re-hachage à la connexion passe par lui et
ne doit déconnecter personne. Une session sans version compte pour 0 et prend fin à la première
hausse. `fresh_user` refait la comparaison : si `logout` a échoué, le `user_id` est encore là.

## Séquence : makemigrations (diff)

Chemin réel (3.0.0, 2026-10-06) — le `ModelSchema::diff` du schéma initial n'est pas sur ce
chemin (voir AM1) :

```mermaid
sequenceDiagram
    participant CLI as runique makemigrations
    participant DSL as runique_dsl (même parseur que la macro)
    participant SC as scan_entities
    participant SN as snapshots (parse::seaorm)
    participant D as diff_schemas
    participant G as generate (SeaQuery)

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
La CLI utilise `diff_schemas` ([cli_migration/plan.rs:33](../../runique/src/migration/cli_migration/plan.rs#L33))
qui calcule `modified_columns`. Le `ModelSchema::diff` limité (add/drop) n'est **pas** le
chemin de la CLI. Détecté en traçant le flux jusqu'au vrai `diff` appelé.

### ❌ AM2 — FAUX POSITIF sur la divergence (résidu = perf seulement)
Voir le diagramme ci-dessus : les `on_conflict.update_columns` sont disjoints et **aucun** ne
touche `session_id`. `upsert` (1ʳᵉ) le fixe, `create` (2ᵉ) ne fait qu'un UPDATE
`[UserId, ExpiresAt]`. `session_id` est donc déterministe — pas de divergence. Résidu réel :
**double aller-retour DB** au login (🟡 perf, fusionnable).

### 🟡 AM3 — TTL 24h codé en dur en double — ✅ CORRIGÉ
Une seule durée, `AUTH_SESSION_TTL_SECS`, pour le cookie et la base : réglable par le builder,
24 h par défaut (`DEFAULT_AUTH_SESSION_TTL_SECS`, `auth/session.rs`).

### 🟡 AM4 — `pull_groupes_db` à chaque login + cache mémoire process-local — ✅ CORRIGÉ (2026-10-03)
Cache supprimé. L'état du compte et les droits sont relus en base à chaque requête admin
(`load_admin_user`) ; la session ne garde que l'identité.
