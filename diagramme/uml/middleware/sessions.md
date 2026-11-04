# UML — Sessions (store mémoire + fallback DB)

[`cleaning_store.rs`](../../../runique/src/middleware/session/cleaning_store.rs),
[`session_db.rs`](../../../runique/src/middleware/session/session_db.rs)

```mermaid
classDiagram
    class CleaningMemoryStore {
        -Arc~Mutex~HashMap~Id,Record~~~ data
        -Arc~AtomicUsize~ size_bytes
        -usize low_watermark
        -usize high_watermark
        -bool exclusive_login
        -Option~Arc~RuniqueSessionStore~~ db_fallback
        +with_watermarks(low, high)
        +with_exclusive_login(b)
        +with_db_fallback(db)
        +is_saturated() bool
        -persist_to_db(record) [snapshot complet]
        +save(record) / load(id) / delete(id)
        +create(record) / delete_expired()
    }
    class RuniqueSessionStore {
        -ADb db
        -SessionKey key
        +new(db, secret_key)
        +create(cookie_id, user_id, session_id, expires_at) Result
        +upsert_session(cookie_id, user_id, expires_at, data) Result [chiffre]
        +update_session(cookie_id, user_id, expires_at, data) Result~bool~ [chiffre]
        +find_by_cookie_id(cookie_id) Result~Option~Model~~
        +open_session_data(model) Option~String~ [déchiffre]
        +delete(cookie_id) / invalidate_other_sessions / invalidate_all
    }
    class SessionKey {
        <<Debug masqué>>
        -cle 32 octets
        +derive(secret_key)$ [HMAC-SHA256, libellé fixe]
        +seal(plain, bound_to) Option~String~
        +open(sealed, bound_to) Option~String~
    }
    class SessionModel {
        +i32 id
        +String cookie_id
        +Pk user_id
        +String session_id
        +Option~String~ session_data [v1: + base64 nonce + chiffré]
        +NaiveDateTime expires_at
    }
    CleaningMemoryStore "1" o-- "0..1" RuniqueSessionStore : db_fallback
    RuniqueSessionStore ..> SessionModel : eihwaz_sessions
    RuniqueSessionStore *-- SessionKey
    CleaningMemoryStore ..|> SessionStore : tower_sessions
```

Stratégie : **mémoire d'abord** (autoritaire) + **snapshot complet** vers la DB pour les
sessions **authentifiées** uniquement (présence `user_id`). `persist_to_db` écrit tous les
champs ensemble pour interdire un backup partiel/périmé. `load` : miss mémoire → DB →
reconstruit le `Record`. `delete` purge mémoire **et** DB (couvre le cleanup `cycle_id`).

**Chiffrement** ([`session_cipher.rs`](../../../runique/src/utils/crypto/session_cipher.rs)) :
`session_data` est chiffré en XChaCha20-Poly1305, nonce aléatoire de 192 bits, clé dérivée de
`SECRET_KEY`, hash de `cookie_id` en donnée authentifiée. Une ligne en clair, modifiée, copiée sur
une autre ligne ou écrite sous une autre clé n'est pas restaurée : l'utilisateur se reconnecte.
Échec de chiffrement → écriture refusée, jamais de repli en clair. `user_id` et `expires_at`
restent en clair, le store en a besoin.

| Moment | Lit | Déchiffre |
|--------|-----|-----------|
| Requête, session en mémoire | `user_id` de la ligne (`load`) | non |
| Connexion, déconnexion, purge, invalidation | `cookie_id`, `user_id`, `expires_at` | non |
| Restauration après un redémarrage | `session_data` | **oui**, puis retour en mémoire |

**Fermeture dans tous les stores** : `login` copie `eihwaz_users.session_version` dans la session ;
`auth_middleware` et `fresh_user` la comparent au compte relu (`session_is_current`). Voir
[../../flux/auth-session-et-makemigrations.md](../../flux/auth-session-et-makemigrations.md).

## Anomalies / flux suspects

### 🟠 S1 — Sessions anonymes (et leur CSRF) perdues au redémarrage
`persist_to_db` est **no-op sans `user_id`** (sessions anonymes non persistées). Or le token
CSRF d'un visiteur non connecté vit dans sa session anonyme. Après un restart serveur, sa
session anonyme disparaît → le **POST d'un formulaire public** (login, contact…) échoue en
CSRF invalide jusqu'à rechargement de la page. UX dégradée non signalée. À confirmer dans le
flux CSRF/session.

### 🟠 S2 — Lock relâché avant l'écriture DB async (fenêtre de course)
[`cleaning_store.rs:378`](../../../runique/src/middleware/session/cleaning_store.rs#L378)
Le `save` relâche le `Mutex` mémoire **avant** le `persist_to_db().await`. Deux `save`
concurrents sur la même session peuvent donc écrire en DB dans un ordre ≠ de l'ordre mémoire
→ backup DB potentiellement périmé d'un cran (le dernier writer DB gagne, pas le dernier
writer mémoire). Mémoire reste cohérente (autoritaire), mais le fallback peut diverger.

### 🟠 S3 — High watermark : refus de nouvelles sessions sous pression — ✅ CORRIGÉ
**Corrigé (2.1.21).** Le refus au-delà du high watermark était déjà propre côté store (Err
typé, loggé), mais remontait en 500 générique. Ajout de `is_saturated()` +
`RuniqueEngine::session_store_saturated()` → le login admin renvoie un **503 + Retry-After**
clair (template `503.html` + `render_503`, 9 locales) au lieu d'un 500. Le login public se
protège pareil via l'accesseur engine.

### 🟡 S4 — Double identifiant `cookie_id` / `session_id` (rappel D4)
`SessionModel` porte `cookie_id` (UNIQUE, identifiant d'opposabilité) et `session_id`
(stable par device). La distinction est subtile et a déjà causé une collision (corrigée
2.1.19). À documenter dans le flux session pour éviter une régression.

### 🟠 S5 — Contenu des sessions lisible en base — ✅ CORRIGÉ (2026-11-04)
Une lecture de la base (sauvegarde, injection SQL) donnait le contenu des sessions, jeton CSRF
compris. Voir « Chiffrement » ci-dessus. Le passage à la 4.0 vide `eihwaz_sessions`.

### 🟠 S6 — Store personnalisé non révoqué après un reset — ✅ CORRIGÉ (2026-11-04)
Le trait `SessionStore` ne cherche pas par utilisateur : un cookie volé restait valide après un
reset sous `with_session_store`. `session_version` le ferme à sa requête suivante, sans requête
SQL de plus.
