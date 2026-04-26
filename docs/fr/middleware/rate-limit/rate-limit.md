# Rate Limiting

Runique propose deux approches de rate limiting : **déclarative** au niveau des routes, ou **fine** dans le handler.

---

## Approche déclarative — au niveau des routes

Directement dans `url.rs`, via le trait `RouterExt` :

```rust
use runique::prelude::*;

pub fn routes() -> Router {
    urlpatterns! {
        "/" => view!{ index }, name = "index",
        // ...
    }
    // Route unique
    .rate_limit("/upload-image", "upload_image", view!(upload_image_submit), 5, 60)
}
```

Plusieurs routes partageant le **même compteur** :

```rust
.rate_limit_many(5, 60, vec![
    ("/upload-image".into(), "upload_image".into(), view!(upload_image_submit)),
    ("/inscription".into(),  "inscription".into(),  view!(soumission_inscription)),
])
```

> Le `spawn_cleanup` est appelé automatiquement — pas de fuite mémoire.

---

## Approche handler — logique fine

Pour une logique par utilisateur, par action, ou avec une clé custom :

```rust
use runique::prelude::*;

static LIMITER: LazyLock<RateLimiter> = LazyLock::new(|| {
    RateLimiter::new()
        .max_requests(10)
        .retry_after(60)
});

pub async fn login(/* ... */) -> impl IntoResponse {
    if !LIMITER.is_allowed(&ip) {
        return StatusCode::TOO_MANY_REQUESTS.into_response();
    }
    // ...
}
```

---

## Quand utiliser laquelle ?

| Cas                                           | Approche                      |
| --------------------------------------------- | ----------------------------- |
| Route publique exposée, limite globale par IP | Déclarative (`.rate_limit()`) |
| Limite par utilisateur connecté               | Handler                       |
| Logique différente selon le contexte          | Handler                       |
| Plusieurs routes, même quota partagé          | `.rate_limit_many()`          |

---

## Configuration

```rust
RateLimiter::new().max_requests(5).retry_after(60)    // 5 requêtes par minute
RateLimiter::new().max_requests(3).retry_after(300)   // 3 requêtes par 5 minutes
RateLimiter::new().max_requests(100).retry_after(60)  // 100 requêtes par minute
```

---

## Comportement

- La clé de limitation est l'**adresse IP** de la requête
- Supporte les headers `X-Forwarded-For` et `X-Real-IP` (reverse proxy)
- Fenêtre **fixe** : le compteur repart à zéro après `retry_after` secondes
- Réponse `429 Too Many Requests` quand la limite est dépassée, avec header `Retry-After: <secondes>`

> **⚠️ Sécurité :** Ce middleware fait confiance aux headers `X-Forwarded-For` et `X-Real-IP`. Assurez-vous que votre reverse proxy (nginx, etc.) contrôle ces headers et ne les laisse pas être forgés par les clients. Sans proxy de confiance, un attaquant peut contourner le rate limiting en modifiant ces headers.

---

## API

### `RateLimiter::new()`

Crée un rate limiter avec les valeurs par défaut (60 req / 60 s).

### `.max_requests(max: u32)`

Nombre de requêtes autorisées dans la fenêtre.

### `.retry_after(secs: u64)`

Durée de la fenêtre en secondes.

### `is_allowed(key: &str) -> bool`

Retourne `true` si la clé est sous la limite, `false` sinon.

### `retry_after_secs(key: &str) -> u64`

Secondes restantes avant réinitialisation de la fenêtre pour cette clé. Retourne `0` si la fenêtre est déjà expirée ou si la clé est inconnue. Utilisé pour remplir le header `Retry-After` dans les réponses 429.

### `.spawn_cleanup(period: Duration)`

Lance une tâche de nettoyage en arrière-plan qui purge périodiquement les entrées expirées. Sans ce nettoyage, la map interne croît indéfiniment pour chaque IP distincte. À appeler une fois après construction du limiter.

```rust
let limiter = RateLimiter::new().max_requests(5).retry_after(60);
limiter.spawn_cleanup(Duration::from_secs(60));
let limiter = Arc::new(limiter);
```

> Avec l'approche déclarative (`.rate_limit()` / `.rate_limit_many()`), `spawn_cleanup` est appelé automatiquement.

---

← [**Builder & configuration**](/docs/fr/middleware/builder) | [**Flash Messages**](/docs/fr/flash) →
