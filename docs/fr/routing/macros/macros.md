# Macros de routage

## Macro urlpatterns!

Définir les routes de l'application avec des noms pour la résolution d'URL :

```rust
use crate::views;
use runique::prelude::*;
use runique::{urlpatterns, view};

pub fn routes() -> Router {
    urlpatterns! {
        "/"                  => view!{ views::index },       name = "index",
        "/users"             => view! { views::user_list },  name = "users",
        "/users/{id}"        => view!{ views::user_detail }, name = "user_detail",
        "/users/{id}/delete" => view!{ views::delete_user }, name = "user_delete",
    }
}
```

> Les segments de chemin dynamiques utilisent la syntaxe `{param}` (Axum 0.8). L'ancienne syntaxe `:param` n'est plus supportée.

### Avec noms (recommandé)

Les noms permettent la résolution d'URL dans les templates via `{% link "nom" %}` :

```rust
urlpatterns! {
    "/"          => view!{ views::index },       name = "index",
    "/users/{id}" => view!{ views::user_detail }, name = "user_detail",
}
```

```html
<a href='{% link "index" %}'>Accueil</a>
<a href='{% link "user_detail" id="42" %}'>Profil</a>
```

> Toutes les routes dans un même `urlpatterns!` doivent être soit toutes nommées, soit toutes sans nom — le mélange n'est pas supporté.

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

## Router une méthode précise (GET / POST / PUT…)

`view!{ handler }` branche le **même** handler sur les cinq méthodes (GET, POST, PUT, DELETE, PATCH). C'est le pattern recommandé quand un seul handler gère le formulaire en GET et en POST via `request.is_get()` / `request.is_post()`.

Quand tu veux **un handler distinct par méthode**, n'utilise pas `view!` : passe directement les combinateurs Axum (`get`, `post`, `put`, `delete`, `patch`), déjà ré-exportés par le prelude. `urlpatterns!` accepte n'importe quelle expression qui produit un `MethodRouter` comme handler :

```rust
use crate::views;
use runique::prelude::*; // get, post, put, delete, patch inclus
use runique::urlpatterns;

pub fn routes() -> Router {
    urlpatterns! {
        // Handler unique, toutes méthodes (dispatch via request.is_get/is_post)
        "/inscription"   => view!{ views::inscription },                     name = "inscription",

        // Handlers distincts par méthode
        "/login"         => get(views::login_get).post(views::login_post),   name = "login",

        // Une seule méthode autorisée — les autres renvoient 405
        "/users/{id}"    => delete(views::delete_user),                      name = "user_delete",

        // Chaînage de plusieurs méthodes
        "/articles/{id}" => get(views::article_get)
                                .put(views::article_update)
                                .delete(views::article_delete),              name = "article",
    }
}
```

> Avec les combinateurs Axum, seules les méthodes déclarées sont acceptées : toute autre méthode renvoie automatiquement `405 Method Not Allowed`. À l'inverse, `view!` accepte les cinq méthodes — c'est au handler de filtrer.

Les handlers par méthode ont la même signature que les autres handlers Runique :

```rust
pub async fn login_get(mut request: Request) -> AppResult<Response> {
    request.render("login.html")
}

pub async fn login_post(mut request: Request) -> AppResult<Response> {
    let mut form: LoginForm = request.form();
    if form.is_valid().await {
        // ...
    }
    request.render("login.html")
}
```

### Choisir le type de route selon le besoin

Chaque combinateur fixe explicitement les méthodes acceptées. Quelques cas typiques :

```rust
urlpatterns! {
    // Page affichée seulement (GET) — un POST renverra 405
    "/about"            => get(views::about),                            name = "about",

    // Endpoint qui ne reçoit que des données (POST) — webhook, action
    "/webhooks/stripe"  => post(views::stripe_webhook),                  name = "stripe",

    // Formulaire : GET affiche, POST traite — deux handlers séparés
    "/contact"          => get(views::contact_form)
                               .post(views::contact_submit),             name = "contact",

    // API REST sur une ressource
    "/api/articles"     => get(views::articles_index)
                               .post(views::articles_create),            name = "articles",
    "/api/articles/{id}" => get(views::article_show)
                               .put(views::article_update)
                               .patch(views::article_patch)
                               .delete(views::article_destroy),          name = "article",
}
```

Côté handlers, chaque méthode a sa propre fonction, sans test `is_get()` / `is_post()` :

```rust
// GET /contact — affiche le formulaire vide
pub async fn contact_form(mut request: Request) -> AppResult<Response> {
    let form: ContactForm = request.form();
    context_update!(request => { "contact_form" => &form });
    request.render("contact.html")
}

// POST /contact — valide et traite l'envoi
pub async fn contact_submit(mut request: Request) -> AppResult<Response> {
    let mut form: ContactForm = request.form();
    if form.is_valid().await {
        // envoi de l'email, flash de succès, redirection…
        return Ok(Redirect::to("/contact").into_response());
    }
    context_update!(request => { "contact_form" => &form });
    request.render("contact.html")
}
```

> Règle simple : `view!` quand un même handler gère plusieurs méthodes (dispatch interne), les combinateurs `get`/`post`/… quand chaque méthode mérite son propre handler ou que tu veux restreindre les méthodes autorisées.

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
