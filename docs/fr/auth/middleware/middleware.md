# Middlewares de protection & CurrentUser

## `login_required` — protéger une route

Redirige vers `REDIRECT_ANONYMOUS` si l'utilisateur n'est pas connecté.

```rust
use runique::middleware::auth::login_required;

let protected = Router::new()
    .route("/dashboard", get(dashboard))
    .layer(axum::middleware::from_fn(login_required));
```

---

## `redirect_if_authenticated` — page login/register

Redirige vers `USER_CONNECTED_URL` si l'utilisateur est déjà connecté. Utile pour éviter qu'un utilisateur connecté accède à `/login`.

```rust
use runique::middleware::auth::redirect_if_authenticated;

let public = Router::new()
    .route("/login", get(login_page).post(login_post))
    .layer(axum::middleware::from_fn(redirect_if_authenticated));
```

---

## `load_user_middleware` — charger le contexte utilisateur

Injecte un `CurrentUser` dans les extensions de la requête. Permet d'accéder aux informations de l'utilisateur dans vos handlers.

```rust
use runique::middleware::auth::load_user_middleware;

let app = Router::new()
    .route("/profile", get(profile))
    .layer(axum::middleware::from_fn(load_user_middleware));
```

Accès dans un handler :

```rust
use runique::middleware::auth::CurrentUser;
use runique::context::RequestExtensions;

async fn profile(req: RuniqueRequest) -> impl IntoResponse {
    if let Some(user) = req.extensions().current_user() {
        println!("Connecté : {}", user.username);
    }
}
```

---

## CurrentUser

Structure injectée par `load_user_middleware` dans les extensions de requête.

```rust
pub struct CurrentUser {
    pub id: i32,
    pub username: String,
    pub is_staff: bool,
    pub is_superuser: bool,
    pub roles: Vec<String>,
}
```

### Méthodes disponibles

```rust
// Vérifier un rôle précis
user.has_role("editor")           // → bool

// Vérifier au moins un rôle parmi une liste
user.has_any_role(&["editor", "moderator"])  // → bool

// Accès à l'admin (is_staff || is_superuser)
user.can_access_admin()           // → bool

// Vérifier une permission admin (is_superuser bypass tout)
user.can_admin(&["editor"])       // → bool
```

---

## Voir aussi

| Section | Description |
| --- | --- |
| [Modèle utilisateur](https://github.com/seb-alliot/runique/blob/main/docs/fr/auth/modele/modele.md) | Built-in, trait `RuniqueUser` |
| [Helpers de session](https://github.com/seb-alliot/runique/blob/main/docs/fr/auth/session/session.md) | `login`, `logout`, vérifications |

## Retour au sommaire

- [Authentification](https://github.com/seb-alliot/runique/blob/main/docs/fr/auth/13-authentification.md)
