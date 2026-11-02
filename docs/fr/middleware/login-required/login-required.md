# Login Required

`login_required` protège une route contre les accès non authentifiés. Si l'utilisateur n'est pas connecté, il est redirigé vers l'URL de login spécifiée.

---

## Utilisation — au niveau des routes

Dans `url.rs`, via le trait `RouterExt` :

```rust
use runique::prelude::*;

pub fn routes() -> Routes {
    urlpatterns! {
        "/" => view!{ index }, name = "index",
        // ...
    }
    .login_required("/profil", "profil", view!(profil), "/login")
}
```

Plusieurs routes protégées :

```rust
urlpatterns! { ... }
    .login_required("/profil",    "profil",    view!(profil),    "/login")
    .login_required("/tableau",   "tableau",   view!(tableau),   "/login")
```

---

## Signature

```rust
fn login_required(
    self,
    path: impl Into<String>,
    name: impl Into<String>,
    handler: MethodRouter,
    redirect_url: impl Into<String>,
) -> Self;
```

| Paramètre      | Description                                        |
| -------------- | -------------------------------------------------- |
| `path`         | Chemin de la route (ex: `"/profil"`)               |
| `name`         | Nom de la route, pour `{% link %}`                 |
| `handler`      | Vue à protéger (via `view!{}`)                     |
| `redirect_url` | URL de redirection si non authentifié (ex: `"/login"`) |

---

## Comportement

- Vérifie qu'un compte est connecté. Le compte est relu en base à chaque requête : un compte supprimé ou désactivé est déconnecté avant d'arriver ici, et redirigé
- Authentifié → la requête passe normalement au handler
- Non authentifié → `303 See Other` vers `redirect_url`
- Compatible avec toutes les méthodes HTTP (GET, POST, etc.)

---

## Notes

- Cette protection s'applique à **n'importe quel utilisateur connecté**, sans distinction de rôle
- Pour l'interface admin, Runique utilise son propre middleware `admin_access` (rang admin, dirigeant ou superuser, ou membre d'un groupe qui ouvre l'admin) — `login_required` n'est pas nécessaire côté admin

---

← [**Rate Limiting**](/docs/fr/middleware/rate-limit) | [**Flash Messages**](/docs/fr/flash) →
