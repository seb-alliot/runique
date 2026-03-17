# Rate Limiting

Le rate limiter de Runique est granulaire : chaque handler peut avoir ses propres limites, appliquées directement dans le code.

---

## Usage

```rust
use runique::prelude::*;

static LIMITER: LazyLock<RateLimiter> = LazyLock::new(|| {
    RateLimiter::new()
        .max_requests(10)
        .window_secs(60)
});

pub async fn login(/* ... */) -> impl IntoResponse {
    if !LIMITER.is_allowed(&ip) {
        return StatusCode::TOO_MANY_REQUESTS.into_response();
    }
    // ...
}
```

---

## Configuration

```rust
RateLimiter::new().max_requests(5).window_secs(60)    // 5 requêtes par minute
RateLimiter::new().max_requests(3).window_secs(300)   // 3 requêtes par 5 minutes
RateLimiter::new().max_requests(100).window_secs(60)  // 100 requêtes par minute
```

---

## Comportement

- La clé de limitation est l'**adresse IP** de la requête
- Supporte les headers `X-Forwarded-For` et `X-Real-IP` (reverse proxy)
- Fenêtre **fixe** : le compteur repart à zéro après `window_secs` secondes
- Réponse `429 Too Many Requests` quand la limite est dépassée, avec header `Retry-After: <secondes>`

> **⚠️ Sécurité :** Ce middleware fait confiance aux headers `X-Forwarded-For` et `X-Real-IP`. Assurez-vous que votre reverse proxy (nginx, etc.) contrôle ces headers et ne les laisse pas être forgés par les clients. Sans proxy de confiance, un attaquant peut contourner le rate limiting en modifiant ces headers.

---

## API

### `RateLimiter::new()`

Crée un rate limiter avec les valeurs par défaut (60 req / 60 s).

### `.max_requests(max: u32)`

Nombre de requêtes autorisées dans la fenêtre.

### `.window_secs(secs: u64)`

Durée de la fenêtre en secondes.

### `is_allowed(key: &str) -> bool`

Retourne `true` si la clé est sous la limite, `false` sinon.

### `retry_after_secs(key: &str) -> u64`

Secondes restantes avant réinitialisation de la fenêtre pour cette clé. Retourne `0` si la fenêtre est déjà expirée ou si la clé est inconnue. Utilisé pour remplir le header `Retry-After` dans les réponses 429.

---

← [**Builder & configuration**](https://github.com/seb-alliot/runique/blob/main/docs/fr/middleware/builder/builder.md) | [**Flash Messages**](https://github.com/seb-alliot/runique/blob/main/docs/fr/flash/09-flash-messages.md) →
