# LoginGuard — Protection contre le brute-force

`LoginGuard` suit les tentatives de connexion échouées **par username**, indépendamment de l'adresse IP.

Contrairement au rate limiting IP, il n'y a pas de faux positifs liés au NAT ou aux proxies partagés : chaque compte est suivi individuellement.

`LoginGuard` s'utilise **dans le handler**, pas comme middleware — c'est la seule façon d'accéder au username soumis dans le formulaire sans consommer le body avant `Prisme`.

---

## Usage complet avec `effective_key`

`effective_key` détermine automatiquement la bonne clé selon que l'utilisateur a soumis un username ou non :

- Username soumis → clé par compte (protection ciblée)
- Username vide → `"anonym:{ip}"` (protection par IP, compteurs indépendants)

```rust
use runique::prelude::*;

static GUARD: LazyLock<LoginGuard> = LazyLock::new(|| {
    LoginGuard::new()
        .max_attempts(5)
        .lockout_secs(300)
});

pub async fn login(
    session: Session,
    State(db): State<DatabaseConnection>,
    Prisme(form): Prisme<LoginForm>,
) -> impl IntoResponse {
    let username = form.username();
    let ip = /* extraire l'IP depuis les headers */;

    // username soumis → "alice"  |  username vide → "anonym:1.2.3.4"
    let key = LoginGuard::effective_key(&username, &ip);

    if GUARD.is_locked(&key) {
        let remaining = GUARD.remaining_lockout_secs(&key).unwrap_or(0);
        return (StatusCode::TOO_MANY_REQUESTS, format!("Réessayez dans {remaining}s")).into_response();
    }

    match authenticate(&username, &form.password(), &db).await {
        Some(user) => {
            GUARD.record_success(&key);
            auth_login(&session, &db, user.id).await.unwrap();
            Redirect::to("/dashboard").into_response()
        }
        None => {
            GUARD.record_failure(&key);
            (StatusCode::UNAUTHORIZED, "Identifiants invalides").into_response()
        }
    }
}
```

---

## Pourquoi dans le handler et pas en middleware ?

Un middleware s'exécute avant le handler et ne peut pas lire le body sans le consommer.
`Prisme` extrait le formulaire une seule fois — le username n'est disponible qu'après cette extraction.

La session connaît l'état d'authentification, mais au moment du login l'utilisateur n'est pas encore connecté : `session.username` serait toujours `"anonym"` sur cette route, ce qui ne protège pas le compte ciblé.

| Source | Disponible en middleware | Fiable pour LoginGuard |
| --- | --- | --- |
| Adresse IP | ✅ | ✅ (utilisée par `effective_key` pour anonym) |
| Username (session) | ✅ | ❌ (toujours anonym sur `/login`) |
| Username (form body) | ❌ (consomme le body) | ✅ (via `Prisme` dans le handler) |

---

## Configuration

```rust
LoginGuard::new().max_attempts(5).lockout_secs(300)    // 5 échecs → blocage 5 minutes
LoginGuard::new().max_attempts(3).lockout_secs(900)    // 3 échecs → blocage 15 minutes
LoginGuard::new().max_attempts(10).lockout_secs(3600)  // 10 échecs → blocage 1 heure
```

---

## API

### `LoginGuard::new()`

Crée un LoginGuard avec les valeurs par défaut (5 tentatives / 300 s).

### `.max_attempts(max: u32)`

Nombre d'échecs avant verrouillage du compte.

### `.lockout_secs(secs: u64)`

Durée du verrouillage en secondes.

### `LoginGuard::effective_key(username, ip) -> Cow<str>`

Retourne la clé à utiliser selon le contexte :

- Username non vide → clé par username (protection compte ciblé)
- Username vide ou absent → `"anonym:{ip}"` (protection anonyme par IP)

Garantit que deux IPs anonymes différentes ont des compteurs indépendants — bloquer `"anonym:1.2.3.4"` n'affecte pas `"anonym:5.6.7.8"`.

> **Pourquoi pas une clé `"anonym"` globale ?** Une seule tentative abusive suffirait à verrouiller tous les utilisateurs anonymes du monde entier. La clé par IP isole chaque attaquant.

### `record_failure(key)`

Incrémente le compteur d'échecs pour cette clé. À appeler après chaque authentification échouée.

### `record_success(key)`

Réinitialise le compteur. À appeler après une connexion réussie.

### `is_locked(key) -> bool`

Retourne `true` si le nombre d'échecs dépasse `max_attempts` et que le délai de verrouillage n'est pas écoulé.

### `attempts(key) -> u32`

Nombre d'échecs en cours pour cette clé.

### `remaining_lockout_secs(key) -> Option<u64>`

Secondes restantes avant déverrouillage. `None` si non verrouillé.

---

## Combinaison avec le rate limiting IP

Les deux mécanismes couvrent des vecteurs d'attaque différents et sont complémentaires :

| | `RateLimiter` | `LoginGuard` |
| --- | --- | --- |
| Clé | Adresse IP | Username ou `anonym:{ip}` |
| Où | Middleware (automatique) | Handler (manuel) |
| Cible | Toutes les routes | Login uniquement |
| Protège contre | Volume d'attaque par IP | Brute-force par compte |
| Faux positifs | Possible (NAT) | Aucun (par compte) |

```rust
static IP_LIMITER: LazyLock<RateLimiter> = LazyLock::new(|| RateLimiter::new().max_requests(10).window_secs(60));
static GUARD:      LazyLock<LoginGuard>  = LazyLock::new(|| LoginGuard::new().max_attempts(5).lockout_secs(300));

pub async fn login(/* ... */) -> impl IntoResponse {
    // 1. Volume par IP → RateLimiter (middleware ou manuel)
    if !IP_LIMITER.is_allowed(&ip) { /* 429 */ }

    // 2. Brute-force par compte ou IP anonyme → LoginGuard
    let key = LoginGuard::effective_key(&username, &ip);
    if GUARD.is_locked(&key) { /* 429 */ }

    // 3. Authentification
    match authenticate(&username, &password, &db).await {
        Some(user) => { GUARD.record_success(&key); /* ... */ }
        None       => { GUARD.record_failure(&key); /* ... */ }
    }
}
```

---

← [**Middlewares de protection**](/docs/fr/auth/middleware) | [**Exemple complet**](/docs/fr/auth/exemple) →
