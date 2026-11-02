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

La session ne garde que **qui** s'est connecté. Le compte (actif, rang, groupes) est relu en
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
    pub id: Pk,                    // i32 par défaut, i64 avec "big-pk", Uuid avec "pk-uuid"
    pub username: String,
    pub rang: Rang,                // Lambda, Admin, Dirigeant, Superuser
    pub groupes: Vec<Groupe>,      // chargés par load_user_rights()
    pub droits: Vec<Permission>,   // les droits propres d'un admin, chargés par load_user_rights()
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
        .is_some_and(|u| u.rights_on("articles").crud.can_update);
    // ...
}
```

### Méthodes disponibles

```rust
// Droits sur une ressource, selon le rang : tout pour un dirigeant ou un
// superuser, la grille propre à un admin, l'union des groupes pour les autres
user.rights_on("articles")              // → Rights { crud, columns }
user.rights_on("articles").crud.can_update

// Lecture d'une ressource autorisée ?
user.can_access_resource("articles")    // → bool

// Accès au panneau admin : rang admin ou plus, ou membre d'un groupe staff
user.can_access_admin()                 // → bool
user.is_staff()                         // membre d'un groupe qui ouvre l'admin
```

Ces méthodes lisent les groupes et les droits : appelez `request.load_user_rights().await` d'abord.

---

## Voir aussi

| Section | Description |
| --- | --- |
| [Modèle utilisateur](/docs/fr/auth/modele) | Built-in, trait `RuniqueUser` |
| [Helpers de session](/docs/fr/auth/session) | `login`, `logout`, vérifications |

## Retour au sommaire

- [Authentification](/docs/fr/auth)
