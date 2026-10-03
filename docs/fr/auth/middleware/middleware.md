# Middlewares de protection & CurrentUser

## Protection de routes — pattern recommandé

Deux approches coexistent. Le pattern manuel ci-dessous s'écrit directement dans le handler — plus explicite, contrôle total de l'URL de redirection. Pour protéger une route entière sans y toucher, `login_required` (via le trait `RouterExt`, au niveau de `urlpatterns!{}`) reste disponible — voir [Login Required](/docs/fr/middleware/login-required).

```rust
use runique::prelude::*;

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

## `request.user` — l'utilisateur connecté

Aucun middleware à ajouter : Runique injecte un `CurrentUser` dans chaque requête d'un
utilisateur connecté, accessible par `request.user`.

La session ne garde que **qui** s'est connecté. Le compte (actif, staff, superuser) est relu en
base à chaque requête — une lecture par clé primaire. Un compte supprimé ou désactivé voit donc
sa session fermée dès sa requête suivante : `request.user` vaut `None` et `is_authenticated`
renvoie `false`, quel que soit le chemin de la modification (admin, CLI, SQL, autre instance).

```rust
use runique::prelude::*;

async fn profile(request: Request) -> impl IntoResponse {
    if let Some(user) = &request.user {
        println!("Connecté : {}", user.username);
    }
}
```

## CurrentUser

```rust
pub struct CurrentUser {
    pub id: Pk,      // i32 par défaut, i64 avec "big-pk", Uuid avec "pk-uuid"
    pub username: String,
    pub is_staff: bool,
    pub is_superuser: bool,
    pub groupes: Vec<Groupe>,
}
```

Les groupes et leurs droits ne sont **pas** chargés par défaut : la plupart des pages ne les
consultent jamais. Une vue qui vérifie un droit les charge d'abord — ils sont relus en base, et
`current_user` est mis à jour dans le contexte du template :

```rust
async fn articles(mut request: Request) -> AppResult<Response> {
    request.load_user_rights().await;
    let can_edit = request
        .user
        .as_ref()
        .is_some_and(|u| u.permission_for("articles").is_some_and(|p| p.can_update));
    // ...
}
```

### Méthodes disponibles

```rust
// Permissions effectives (toutes ressources, OR logique sur tous les groupes)
user.permissions_effectives()                 // → Vec<Permission>

// Permission pour une ressource précise
user.permission_for("users")                  // → Option<Permission>

// Accès en lecture à une ressource (is_superuser bypass tout)
user.can_access_resource("users")             // → bool

// Accès au panneau admin (is_staff || is_superuser)
user.can_access_admin()                       // → bool
```

---

## Voir aussi

| Section | Description |
| --- | --- |
| [Modèle utilisateur](/docs/fr/auth/modele) | Built-in, trait `RuniqueUser` |
| [Helpers de session](/docs/fr/auth/session) | `login`, `logout`, vérifications |

## Retour au sommaire

- [Authentification](/docs/fr/auth)
