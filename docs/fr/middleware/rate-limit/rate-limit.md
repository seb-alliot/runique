# Rate Limiting

Le rate limiter de Runique est un middleware applicatif par IP, à appliquer sur les routes sensibles (login, API, etc.).

Contrairement à une règle nginx globale, il est **granulaire** : chaque route peut avoir ses propres limites.

---

## Installation rapide

```rust
use runique::prelude::*;
use std::sync::Arc;

let limiter = Arc::new(RateLimiter::new(5, 60)); // 5 requêtes par 60 secondes

let app = Router::new()
    .route("/login", post(login_view))
    .layer(axum::middleware::from_fn_with_state(
        limiter,
        rate_limit_middleware,
    ));
```

---

## Configuration

### Par code

```rust
// 5 requêtes maximum par fenêtre de 60 secondes
RateLimiter::new(5, 60)

// 3 requêtes par 10 secondes
RateLimiter::new(3, 10)

// 100 requêtes par minute
RateLimiter::new(100, 60)
```

### Par variables d'environnement

```env
RUNIQUE_RATE_LIMIT_REQUESTS=60
RUNIQUE_RATE_LIMIT_WINDOW_SECS=60
```

```rust
let limiter = Arc::new(RateLimiter::from_env());
```

---

## Limites différentes par route

```rust
let login_limiter = Arc::new(RateLimiter::new(5, 60));   // strict
let api_limiter   = Arc::new(RateLimiter::new(100, 60)); // souple

let app = Router::new()
    .route("/login", post(login_view))
    .layer(axum::middleware::from_fn_with_state(
        login_limiter,
        rate_limit_middleware,
    ))
    .route("/api/data", get(api_view))
    .layer(axum::middleware::from_fn_with_state(
        api_limiter,
        rate_limit_middleware,
    ));
```

---

## Comportement

- La clé de limitation est l'**adresse IP** de la requête
- Supports les headers `X-Forwarded-For` et `X-Real-IP` (reverse proxy)
- Fenêtre **fixe** : le compteur repart à zéro après `window_secs` secondes
- Réponse `429 Too Many Requests` quand la limite est dépassée

---

## API

### `RateLimiter::new(max_requests, window_secs)`

| Paramètre | Type | Description |
|-----------|------|-------------|
| `max_requests` | `u32` | Nombre de requêtes autorisées dans la fenêtre |
| `window_secs` | `u64` | Durée de la fenêtre en secondes |

### `RateLimiter::from_env()`

Construit depuis `RUNIQUE_RATE_LIMIT_REQUESTS` et `RUNIQUE_RATE_LIMIT_WINDOW_SECS`.

### `rate_limit_middleware`

Fonction middleware axum — à passer à `from_fn_with_state`.

---

← [**Builder & configuration**](https://github.com/seb-alliot/runique/blob/main/docs/fr/middleware/builder/builder.md) | [**Flash Messages**](https://github.com/seb-alliot/runique/blob/main/docs/fr/flash/09-flash-messages.md) →
