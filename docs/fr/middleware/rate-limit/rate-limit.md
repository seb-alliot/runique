# Rate Limiting

Le rate limiter de Runique est granulaire : chaque handler peut avoir ses propres limites, appliquées directement dans le code.

---

## Usage

```rust
use runique::prelude::*;

static LIMITER: LazyLock<RateLimiter> = LazyLock::new(|| RateLimiter::new(10, 60));

pub async fn login(/* ... */) -> impl IntoResponse {
    if !LIMITER.is_allowed(&ip) {
        return StatusCode::TOO_MANY_REQUESTS.into_response();
    }
    // ...
}
```

`RateLimiter::new(max_requests, window_secs)` — le dev déclare ses limites où il veut, comme il veut.

---

## Configuration

### Par code

```rust
RateLimiter::new(5, 60)    // 5 requêtes par minute
RateLimiter::new(3, 300)   // 3 requêtes par 5 minutes
RateLimiter::new(100, 60)  // 100 requêtes par minute
```

### Par variables d'environnement

```env
RUNIQUE_RATE_LIMIT_REQUESTS=60
RUNIQUE_RATE_LIMIT_WINDOW_SECS=60
```

```rust
static LIMITER: LazyLock<RateLimiter> = LazyLock::new(RateLimiter::from_env);
```

---

## Comportement

- La clé de limitation est l'**adresse IP** de la requête
- Supporte les headers `X-Forwarded-For` et `X-Real-IP` (reverse proxy)
- Fenêtre **fixe** : le compteur repart à zéro après `window_secs` secondes
- Réponse `429 Too Many Requests` quand la limite est dépassée

> **⚠️ Sécurité :** Ce middleware fait confiance aux headers `X-Forwarded-For` et `X-Real-IP`. Assurez-vous que votre reverse proxy (nginx, etc.) contrôle ces headers et ne les laisse pas être forgés par les clients. Sans proxy de confiance, un attaquant peut contourner le rate limiting en modifiant ces headers.

---

## API

### `RateLimiter::new(max_requests, window_secs)`

| Paramètre | Type | Description |
|-----------|------|-------------|
| `max_requests` | `u32` | Nombre de requêtes autorisées dans la fenêtre |
| `window_secs` | `u64` | Durée de la fenêtre en secondes |

### `RateLimiter::from_env()`

Construit depuis `RUNIQUE_RATE_LIMIT_REQUESTS` et `RUNIQUE_RATE_LIMIT_WINDOW_SECS`.

### `is_allowed(key: &str) -> bool`

Retourne `true` si la clé est sous la limite, `false` sinon.

---

← [**Builder & configuration**](https://github.com/seb-alliot/runique/blob/main/docs/fr/middleware/builder/builder.md) | [**Flash Messages**](https://github.com/seb-alliot/runique/blob/main/docs/fr/flash/09-flash-messages.md) →
