# LoginGuard — Protection contre le brute-force

`LoginGuard` suit les tentatives de connexion échouées **par username**, indépendamment de l'adresse IP.

Contrairement au rate limiting IP, il n'y a pas de faux positifs liés au NAT ou aux proxies partagés : chaque compte est suivi individuellement.

---

## Usage

```rust
use runique::prelude::*;

static GUARD: LazyLock<LoginGuard> = LazyLock::new(|| LoginGuard::new(5, 300));

pub async fn login(
    session: Session,
    State(db): State<DatabaseConnection>,
    Prisme(form): Prisme<LoginForm>,
) -> impl IntoResponse {
    let username = form.username();

    if GUARD.is_locked(&username) {
        let remaining = GUARD.remaining_lockout_secs(&username).unwrap_or(0);
        return (StatusCode::TOO_MANY_REQUESTS, format!("Réessayez dans {remaining}s")).into_response();
    }

    match authenticate(&username, &form.password(), &db).await {
        Some(user) => {
            GUARD.record_success(&username);
            login(&session, user.id, &user.username).await.unwrap();
            Redirect::to("/dashboard").into_response()
        }
        None => {
            GUARD.record_failure(&username);
            (StatusCode::UNAUTHORIZED, "Identifiants invalides").into_response()
        }
    }
}
```

`LoginGuard::new(max_attempts, lockout_secs)` — le dev déclare ses limites où il veut, comme il veut.

---

## Configuration

### Par code

```rust
LoginGuard::new(5, 300)    // 5 échecs → blocage 5 minutes
LoginGuard::new(3, 900)    // 3 échecs → blocage 15 minutes
LoginGuard::new(10, 3600)  // 10 échecs → blocage 1 heure
```

### Par variables d'environnement

```env
RUNIQUE_LOGIN_MAX_ATTEMPTS=5
RUNIQUE_LOGIN_LOCKOUT_SECS=300
```

```rust
static GUARD: LazyLock<LoginGuard> = LazyLock::new(LoginGuard::from_env);
```

---

## API

### `record_failure(username)`

Incrémente le compteur d'échecs pour ce username. À appeler après chaque authentification échouée.

### `record_success(username)`

Réinitialise le compteur. À appeler après une connexion réussie.

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
static IP_LIMITER: LazyLock<RateLimiter> = LazyLock::new(|| RateLimiter::new(10, 60));
static GUARD:      LazyLock<LoginGuard>  = LazyLock::new(|| LoginGuard::new(5, 300));

pub async fn login(/* ... */) -> impl IntoResponse {
    if !IP_LIMITER.is_allowed(&ip) { /* 429 */ }
    if GUARD.is_locked(&username)  { /* 429 */ }
    // ...
}
```

---

← [**Middlewares de protection**](https://github.com/seb-alliot/runique/blob/main/docs/fr/auth/middleware/middleware.md) | [**Exemple complet**](https://github.com/seb-alliot/runique/blob/main/docs/fr/auth/exemple/exemple.md) →
