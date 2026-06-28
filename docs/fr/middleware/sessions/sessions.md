# Sessions (middleware)

## Store par défaut

Runique utilise `MemoryStore` par défaut (données en mémoire, perdues au redémarrage).

---

## Durées de session

| Durée | Usage |
|-------|-------|
| `Duration::minutes(30)` | Sessions courtes (sécurité) |
| `Duration::hours(2)` | Usage standard |
| `Duration::hours(24)` | Défaut Runique |
| `Duration::days(7)` | "Se souvenir de moi" |

---

## Configuration

La durée d'une session **authentifiée** se règle **uniquement via le builder**
`with_session_duration(...)` — il n'existe **pas** de variable `.env` pour cette durée
(choix volontaire : une **source unique** de vérité). C'est l'équivalent du
`SESSION_COOKIE_AGE` de Django.

```rust
use tower_sessions::cookie::time::Duration;

// Durée de session personnalisée
let app = RuniqueApp::builder(config)
    .with_session_duration(Duration::hours(2))   // défaut si non appelé : 24h
    .build()
    .await?;
```

Cette valeur s'applique **partout et de façon cohérente** :

- le **cookie** de session (expiry navigateur) ;
- la ligne **`eihwaz_sessions`** en base (colonne `expires_at`) ;
- le **rafraîchissement** à chaque requête (middleware `session_ttl_upgrade`).

`login()` lit cette même valeur (posée au build) → le cookie, la base et le refresh
**ne peuvent pas diverger**. Si `with_session_duration` n'est jamais appelé, le défaut est
**24h** (86 400 s).

> Les sessions **anonymes** (visiteurs non connectés) ont leur propre durée, plus courte,
> via `with_anonymous_session_duration(Duration)` (défaut 5 min).

### « Se souvenir de moi » (durée par session)

Le défaut global ci-dessus convient à la majorité des cas. Pour une durée **différente sur
une session précise** (ex. case « se souvenir de moi » → 30 jours), garde le défaut global
et appelle `set_expiry` sur la session concernée, au login :

```rust
use tower_sessions::{Expiry, cookie::time::Duration};

if remember_me {
    request.session.set_expiry(Some(Expiry::OnInactivity(Duration::days(30))));
}
```

`expires_at` étant stocké **par ligne** dans `eihwaz_sessions`, aucune modification de schéma
n'est nécessaire : seule cette session-là sera prolongée.

### Store personnalisé (production)

```rust
// Exemple avec un store Redis
let app = RuniqueApp::builder(config)
    .middleware(|m| m.with_session_store(RedisStore::new(client)))
    .build()
    .await?;
```

---

## Accès à la session dans les handlers

```rust
pub async fn dashboard(request: Request) -> AppResult<Response> {
    // Lire une valeur de session
    let user_id: Option<i32> = request.session
        .get("user_id")
        .await
        .ok()
        .flatten();

    // Écrire une valeur
    let _ = request.session.insert("last_visit", "2026-02-06").await;
}
```

> Pour le système de sessions complet avec watermarks, voir [Sessions](/docs/fr/session).

---

## Voir aussi

| Section | Description |
| --- | --- |
| [CSRF](/docs/fr/middleware/csrf) | Protection CSRF |
| [Builder](/docs/fr/middleware/builder) | Configuration du builder |

## Retour au sommaire

- [Middleware & Sécurité](/docs/fr/middleware)
