# Middlewares de protection & CurrentUser

## Protection de routes — pattern recommandé

`login_required` et `redirect_if_authenticated` ont été supprimés. La protection s'écrit directement dans le handler, ce qui est plus explicite et laisse le contrôle de l'URL au dev.

```rust
use runique::middleware::auth::is_authenticated;

// Protéger une route
async fn dashboard(mut request: Request) -> AppResult<Response> {
    if !is_authenticated(&request.session).await {
        return Ok(Redirect::to("/login").into_response());
    }
    // ...
}

// Rediriger si déjà connecté (page login/register)
async fn login_page(mut request: Request) -> AppResult<Response> {
    if is_authenticated(&request.session).await {
        return Ok(Redirect::to("/").into_response());
    }
    // ...
}
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
    pub id: UserId,      // i32 par défaut, i64 avec la feature "big-pk"
    pub username: String,
    pub is_staff: bool,
    pub is_superuser: bool,
    pub droits: Vec<Droit>,
    pub groupes: Vec<Groupe>,
}
```

### Méthodes disponibles

```rust
// Droits effectifs (directs + hérités des groupes, dédupliqués)
user.droits_effectifs()           // → Vec<Droit>

// Vérifier un droit précis
user.has_droit("editor")          // → bool

// Vérifier au moins un droit parmi une liste
user.has_any_droit(&["editor", "moderator"])  // → bool

// Accès à l'admin (is_staff || is_superuser)
user.can_access_admin()           // → bool

// Vérifier une permission admin (is_superuser bypass tout)
user.can_admin(&["editor"])       // → bool
```

---

## Voir aussi

| Section | Description |
| --- | --- |
| [Modèle utilisateur](/docs/fr/auth/modele) | Built-in, trait `RuniqueUser` |
| [Helpers de session](/docs/fr/auth/session) | `login`, `logout`, vérifications |

## Retour au sommaire

- [Authentification](/docs/fr/auth)
