# Sessions (middleware)

## Store par défaut

Runique utilise `CleaningMemoryStore` par défaut — mémoire en premier lieu, avec nettoyage automatique des sessions expirées et fallback DB pour les sessions authentifiées (qui survivent aux redémarrages). Détail complet : [Sessions](/docs/fr/session).

---

## Données de session en base

Une session connectée est sauvegardée dans `eihwaz_sessions` pour survivre à un redémarrage. Son contenu y est **chiffré** (XChaCha20-Poly1305, clé dérivée de `SECRET_KEY`), sans rien à configurer :

- une lecture de la base (sauvegarde, injection SQL) ne donne pas le contenu des sessions, ni le jeton CSRF qu'il contient ;
- un contenu modifié en base, ou copié sur une autre ligne, est refusé ;
- `user_id` et `expires_at` restent en clair : le store en a besoin pour ses requêtes.

Le déchiffrement n'a lieu qu'à la restauration après un redémarrage. Une session encore en mémoire n'est vérifiée que sur sa ligne.

**Changer `SECRET_KEY`** rend les sessions sauvegardées illisibles : au prochain redémarrage, leurs utilisateurs se reconnectent. Un attaquant qui lit aussi votre `.env` possède la clé. Un store passé à `with_session_store(...)` n'est pas chiffré par Runique ; un reset ferme quand même ses sessions, par `session_version`.

---

## Durées de session

| Durée | Usage |
|-------|-------|
| `Duration::minutes(30)` | Sessions courtes (sécurité) |
| `Duration::hours(2)` | Usage standard |
| `Duration::hours(24)` | Défaut Runique |
| `Duration::days(7)` | Sessions longues, pour tous les comptes |

---

## Configuration

La durée d'une session **authentifiée** se règle **uniquement via le builder**
`with_session_duration(...)` — il n'existe **pas** de variable `.env` pour cette durée
(choix volontaire : une **source unique** de vérité). C'est l'équivalent du
`SESSION_COOKIE_AGE` de Django.

```rust
use runique::tower_sessions::cookie::time::Duration;

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
> via `with_anonymous_session_duration(Duration)` (défaut 5 min). Le token CSRF vivant dans
> la session, un formulaire public resté ouvert plus longtemps voit son token expirer —
> voir [Protection CSRF](/docs/fr/middleware/csrf), section "Expiration du token & rafraîchissement automatique".

### « Se souvenir de moi » — pas encore possible

Une durée différente sur une session précise (case « se souvenir de moi » → 30 jours) **ne tient pas aujourd'hui** :

- `login()` pose lui-même l'expiration de la session à la durée globale : un `set_expiry` fait avant lui est écrasé ;
- après la connexion, le middleware `session_ttl_upgrade` remet la durée globale à **chaque** requête d'une session connectée : un `set_expiry` posé après `login` ne dure que jusqu'à la requête suivante.

La seule durée qui s'applique est celle de `with_session_duration`, la même pour toutes les sessions connectées.

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
    let cart_id: Option<i32> = request.session
        .get("cart_id")
        .await
        .ok()
        .flatten();

    // L'utilisateur connecté : request.user, relu en base à chaque requête
    let username = request.user.as_ref().map(|user| user.username.clone());

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
