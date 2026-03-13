# LoginGuard — Protection contre le brute-force

`LoginGuard` suit les tentatives de connexion échouées **par username**, indépendamment de l'adresse IP.

Contrairement au rate limiting IP, il n'y a pas de faux positifs liés au NAT ou aux proxies partagés : chaque compte est suivi individuellement.

---

## Installation rapide

```rust
use runique::prelude::*;
use std::sync::Arc;

// 5 échecs → blocage 5 minutes
let guard = Arc::new(LoginGuard::new(5, 300));
```

---

## Utilisation dans un handler de login

```rust
async fn login_post(
    session: Session,
    State(guard): State<Arc<LoginGuard>>,
    State(db): State<DatabaseConnection>,
    Prisme(form): Prisme<LoginForm>,
) -> impl IntoResponse {
    let username = form.username();

    // 1. Vérifier si le compte est verrouillé
    if guard.is_locked(&username) {
        let remaining = guard.remaining_lockout_secs(&username).unwrap_or(0);
        // afficher un message d'erreur avec `remaining` secondes
        return (StatusCode::TOO_MANY_REQUESTS, "Compte temporairement bloqué").into_response();
    }

    // 2. Authentifier
    match authenticate(&username, &form.password(), &db).await {
        Some(user) => {
            guard.record_success(&username);
            login(&session, user.id, &user.username).await.unwrap();
            Redirect::to("/dashboard").into_response()
        }
        None => {
            guard.record_failure(&username);
            // afficher erreur de connexion
            (StatusCode::UNAUTHORIZED, "Identifiants invalides").into_response()
        }
    }
}
```

---

## Configuration

### Par code

```rust
// 5 échecs → blocage 5 minutes
LoginGuard::new(5, 300)

// 3 échecs → blocage 15 minutes
LoginGuard::new(3, 900)

// 10 échecs → blocage 1 heure
LoginGuard::new(10, 3600)
```

### Par variables d'environnement

```env
RUNIQUE_LOGIN_MAX_ATTEMPTS=5
RUNIQUE_LOGIN_LOCKOUT_SECS=300
```

```rust
let guard = Arc::new(LoginGuard::from_env());
```

---

## API

### `record_failure(username)`

Incrémente le compteur d'échecs pour ce username.
À appeler après chaque authentification échouée.

### `record_success(username)`

Réinitialise le compteur.
À appeler après une connexion réussie.

### `is_locked(username) -> bool`

Retourne `true` si le nombre d'échecs dépasse `max_attempts` et que le délai de verrouillage n'est pas écoulé.

### `attempts(username) -> u32`

Nombre d'échecs en cours pour ce username.

### `remaining_lockout_secs(username) -> Option<u64>`

Secondes restantes avant déverrouillage. `None` si le compte n'est pas verrouillé.

---

## Combinaison avec le rate limiting IP

Les deux mécanismes sont complémentaires :

| | `RateLimiter` | `LoginGuard` |
|---|---|---|
| Clé | Adresse IP | Username |
| Cible | Toutes les routes | Login uniquement |
| Faux positifs | Possible (NAT) | Aucun |
| Objectif | Réduire le volume | Protéger les comptes |

```rust
// Protection maximale : les deux en même temps
let ip_limiter = Arc::new(RateLimiter::new(10, 60));
let login_guard = Arc::new(LoginGuard::new(5, 300));

Router::new()
    .route("/login", post(login_post))
    .layer(axum::middleware::from_fn_with_state(
        ip_limiter,
        rate_limit_middleware,
    ))
```

---

← [**Middlewares de protection**](https://github.com/seb-alliot/runique/blob/main/docs/fr/auth/middleware/middleware.md) | [**Exemple complet**](https://github.com/seb-alliot/runique/blob/main/docs/fr/auth/exemple/exemple.md) →
