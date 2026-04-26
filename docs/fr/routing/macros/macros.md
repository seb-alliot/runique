# Macros de routage

## Macro urlpatterns!

Définir les routes de l'application avec des noms pour la résolution d'URL :

```rust
use crate::views;
use runique::prelude::*;
use runique::{urlpatterns, view};

pub fn routes() -> Router {
    urlpatterns! {
        "/" => view!{ views::index }, name = "index",
        "/users" => view! { views::user_list }, name = "users",
        "/users/:id" => view!{ views::user_detail }, name = "user_detail",
        "/users/:id/delete" => view!{views::delete_user }, name = "user_delete",
    }
}
```

### Avec noms (recommandé)

Les noms permettent la résolution d'URL dans les templates via `{% link "nom" %}` :

```rust
urlpatterns! {
    "/" => view!{ views::index }, name = "index",
    "/users/:id" => view!{ views::user_detail }, name = "user_detail",
}
```

```html
<a href='{% link "index" %}'>Accueil</a>
<a href='{% link "user_detail" id="42" %}'>Profil</a>
```

### Sans noms

```rust
urlpatterns! {
    "/" => view!{ views::index },
    "/about" => view!{ views::about },
}
```

---

## Macro view!

Un même handler gère GET et POST ainsi que PUT et DELETE (pattern recommandé avec `request.is_get()` / `request.is_post()`) :

```rust
// Dans les routes
"/inscription" => view!{ views::inscription }, name = "inscription",
```

```rust
// Dans le handler
pub async fn inscription(mut request: Request) -> AppResult<Response> {
    let mut form: RegisterForm = request.form();
    if request.is_get() {
        context_update!(request => { "form" => &form });
        return request.render("form.html");
    }

    if request.is_post() {
        if form.is_valid().await {
            // ...
        }
    }

    request.render("form.html")
}
```

---

## Macro impl_objects! (bonus)

`impl_objects!` est **générée automatiquement** par le daemon dans chaque fichier d'entité. Elle ajoute un manager Django-like utilisable directement dans les handlers :

```rust
use crate::entities::users;

let user = users::Entity::objects
    .filter(users::Column::Username.eq("john"))
    .first(&db)
    .await?;
```

Voir le [guide ORM](/docs/fr/orm) pour plus de détails.

---

## Voir aussi

| Section | Description |
| --- | --- |
| [Extracteurs](/docs/fr/routing/extracteurs) | Path, Query, req.form(), Json |
| [Réponses](/docs/fr/routing/reponses) | Types de réponses |

## Retour au sommaire

- [Routage](/docs/fr/routing)
