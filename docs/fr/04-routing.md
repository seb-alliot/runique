# 🛣️ Routage

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

        // For delete, separate route:
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

### Handler unique pour toutes les méthodes

Un même handler gère GET et POST ainsi que PUT et DELETE (pattern recommandé avec `request.is_get()` / `request.is_post()`) :

```rust
// Dans les routes
"/inscription" => view!{ views::inscription }, name = "inscription",
```

```rust
// Dans le handler
pub async fn inscription(
    mut request: Request,
    Prisme(mut form): Prisme<RegisterForm>,
) -> AppResult<Response> {
    if request.is_get() {
        // Afficher le formulaire vide
        context_update!(request => { "form" => &form });
        return request.render("form.html");
    }

    if request.is_post() {
        // Traiter la soumission
        if form.is_valid().await {
            // ...
        }
    }

    request.render("form.html")
}
```

---

## Extracteurs de paramètres

### Path — Paramètres d'URL

```rust
use axum::extract::Path;

// Simple
async fn user_detail(
    Path(id): Path<i32>,
    mut request: Request,
) -> AppResult<Response> {
    // id = 42 pour /users/42
}

// Multiple
#[derive(Deserialize)]
pub struct UserPostPath {
    user_id: i32,
    post_id: i32,
}

async fn user_post(
    Path(params): Path<UserPostPath>,
    mut request: Request,
) -> AppResult<Response> {
    // params.user_id, params.post_id
}
```

### Query — Paramètres de requête

```rust
use axum::extract::Query;

#[derive(Deserialize)]
pub struct PaginationQuery {
    page: Option<u32>,
    limit: Option<u32>,
}

async fn list(
    Query(query): Query<PaginationQuery>,
    mut request: Request,
) -> AppResult<Response> {
    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(10);
    // ...
}
```

### Prisme — Formulaires

```rust
use runique::prelude::*;

async fn inscription(
    mut request: Request,
    Prisme(mut form): Prisme<RegisterForm>,
) -> AppResult<Response> {
    if form.is_valid().await {
        form.save(&request.engine.db).await?;
    }
    // ...
}
```

### Json — Corps JSON

```rust
use axum::Json;

#[derive(Deserialize)]
pub struct CreateUserRequest {
    username: String,
    email: String,
}

async fn create_api(
    Json(payload): Json<CreateUserRequest>,
) -> impl IntoResponse {
    // payload.username, payload.email
}
```

---

## Retourner des réponses

### HTML Template (le plus courant)

```rust
async fn index(mut request: Request) -> AppResult<Response> {
    context_update!(request => {
        "title" => "Accueil",
    });
    request.render("index.html")
}
```

### Redirect

```rust
use axum::response::Redirect;

async fn after_submit(request: Request) -> AppResult<Response> {
    success!(request.notices => "Sauvegardé !");
    Ok(Redirect::to("/").into_response())
}
```

### JSON

```rust
use axum::Json;
use serde_json::json;

async fn api_list() -> Json<serde_json::Value> {
    Json(json!({
        "status": "success",
        "data": [1, 2, 3]
    }))
}
```

### Status Code

```rust
use axum::http::StatusCode;

async fn not_found() -> StatusCode {
    StatusCode::NOT_FOUND
}
```

### Tuple Response

```rust
async fn created(Json(data): Json<Data>) -> (StatusCode, Json<Data>) {
    (StatusCode::CREATED, Json(data))
}
```

---

## Structure complète d'une app

```rust
// src/url.rs
use crate::views;
use runique::prelude::*;
use runique::{urlpatterns, view};

pub fn routes() -> Router {
    let router = urlpatterns! {
        "/" => view!{ views::index }, name = "index",

        "/about" => view! { views::about }, name = "about",
        "/inscription" => view! { views::soumission_inscription }, name = "inscription",
    };
    router
}

```

```rust
// src/views.rs
use runique::prelude::*;

pub async fn index(mut request: Request) -> AppResult<Response> {
    context_update!(request => {
        "title" => "Accueil",
    });
    request.render("index.html")
}

pub async fn about(mut request: Request) -> AppResult<Response> {
    success!(request.notices => "Bienvenue !");

    context_update!(request => {
        "title" => "À propos",
    });
    request.render("about/about.html")
}

pub async fn soumission_inscription(
    mut request: Request,
    Prisme(mut form): Prisme<RegisterForm>,
) -> AppResult<Response> {
    if request.is_get() {
        context_update!(request => {
            "title" => "Inscription",
            "inscription_form" => &form,
        });
        return request.render("inscription_form.html");
    }

    if request.is_post() {
        if form.is_valid().await {
            let user = form.save(&request.engine.db).await.map_err(|err| {
                form.get_form_mut().database_error(&err);
                AppError::from(err)
            })?;
            success!(request.notices => format!("Bienvenue {} !", user.username));
            return Ok(Redirect::to("/").into_response());
        }

        context_update!(request => {
            "title" => "Erreur",
            "inscription_form" => &form,
            "messages" => flash_now!(error => "Veuillez corriger les erreurs"),
        });
        return request.render("inscription_form.html");
    }

    request.render("inscription_form.html")
}
```

---

## Macro impl_objects! (bonus)

Pour les routes qui font des requêtes ORM, `impl_objects!` ajoute un manager Django-like :

```rust
use runique::impl_objects;

impl_objects!(users::Entity);

// Ensuite dans les handlers :
let user = users::Entity::objects
    .filter(users::Column::Username.eq("john"))
    .first(&db)
    .await?;
```

Voir le [guide ORM](07-orm.md) pour plus de détails.

---

## Prochaines étapes

← [**Routing**](https://github.com/seb-alliot/runique/blob/main/docs/fr/04-routing.md) | [**Forms**](https://github.com/seb-alliot/runique/blob/main/docs/fr/05-forms.md) →
