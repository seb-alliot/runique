# 🛣️ Routage

## Macro urlpatterns!

Définir les routes de l'application:

```rust
use crate::views;
use runique::prelude::*;
use runique::{urlpatterns, view}; // Macros explicites et obligatoire pour cette synthax

pub fn routes() -> Router {
    let router = urlpatterns! {
        "/" => view!{ GET => views::index }, name = "index",
        "/inscription" => view! {
            GET => views::inscription,
            POST => views::soumission_inscription },
            name = "inscription",
         "/users/<id>" => view!{
            Get => view::recherche_user get},
            name = "user_detail",
    };
    router
}

```

---

## Extracteurs de Paramètres

### Path - Paramètres d'URL

```rust
use axum::extract::Path;

// Simple
async fn user_detail(
    Path(id): Path<i32>,
    mut template: TemplateContext,
) -> AppResult<Response> { }

// Multiple
#[derive(Deserialize)]
pub struct UserSearchPath {
    user_id: i32,
    post_id: i32,
}

async fn user_post(
    Path(UserSearchPath { user_id, post_id }): Path<UserSearchPath>,
) -> Response { }
```

### Query - Paramètres de requête

```rust
use axum::extract::Query;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct PaginationQuery {
    page: Option<u32>,
    limit: Option<u32>,
}

async fn list(
    Query(query): Query<PaginationQuery>,
) -> Response {
    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(10);
}
```

### Body - Contenu POST

```rust
use axum::Json;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct CreateUserRequest {
    username: String,
    email: String,
}

async fn create(
    Json(payload): Json<CreateUserRequest>,
) -> Response { }
```

### Formulaires - ExtractForm

```rust
use runique::formulaire::ExtractForm;
use demo_app::forms::LoginForm;

async fn login(
    Prisme(mut form): Prisme<RegisterForm>,
) -> AppResult<Response> {
    let db = template.engine.db.clone();
    if form.is_valid().await {
        match form.save(&*db).await {
        // Traiter
        }
    }
}
```

---

## Retourner des Réponses

### HTML Template

```rust
async fn index(
    template: TemplateContext,
) -> Response {
    context_update!(template => {
        "title" => "Accueil",
        "items" => vec![1, 2, 3]
    });
    template.render("index.html")
}
```

### JSON

```rust
use axum::Json;
use serde_json::json;

async fn api_list() -> Json<serde_json::Value> {
    Json(json!({
        "status": "success",
        "data": vec![1, 2, 3]
    }))
}
```

### Redirect

```rust
use axum::response::Redirect;

async fn login_submit(
    Message(mut messages): Message,
) -> Response {
    messages.success("Connecté!");
    Redirect::to("/dashboard").into_response()
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

## Structure Complète

```rust
use runique::prelude::*;
use axum::{
    extract::Path,
    http::StatusCode,
    response::Redirect,
    routing::{get, post},
    Json, Router,
};
use crate::views;
use runique::{urlpatterns, view}; // Macros explicites

pub fn routes() -> Router {
    urlpatterns! {
        "/" => view!{
            GET => views::index
            },
            name = "index",

        "/users" => view!{
            GET => views::user_list,
            POST => views::create_user
            },
            name = "users",

        "/users/:id" => view!{
            GET => views::user_detail,
            POST => views::update_user
            },
            name = "user_detail",

        // Pour delete, route séparée :
        "/users/:id/delete" => view!{
            POST => views::delete_user
            },
            name = "user_delete",
    }
}

async fn index(template: TemplateContext) -> Response {
    context_update!(template => {
        "title" => "Accueil"
    });
    template.render("index.html")
}

async fn user_list(
    mut template: TemplateContext,
) -> AppResult<Response> {
    let users = UserEntity::find()
        .all(&template.engine.db)
        .await?;

    context_update!(template => {
        "users" => users
    });

    template.render("users/list.html")
}

async fn user_detail(
    Path(id): Path<i32>,
    mut template: TemplateContext,
) -> AppResult<Response> {
    let user = UserEntity::find_by_id(id)
        .one(&template.engine.db)
        .await?;

    match user {
        Some(user) => {
            context_update!(template => {
                "user" => user
            });
            template.render("users/detail.html")
        }
        None => Ok(StatusCode::NOT_FOUND.into_response()),
    }
}

async fn create_user(
    mut template: TemplateContext,
    Prisme(mut form): Prisme<UserForm>,
) -> AppResult<Response> {
    if form.is_valid().await {
        match form.save(&template.engine.db).await {
            Ok(_) => {
                success!(template.notices => "Utilisateur créé !");
                return Ok(Redirect::to("/users").into_response());
            }
            Err(e) => {
                form.get_form_mut().database_error(&e);
            }
        }
    }

    context_update!(template => {
        "form" => form
    });

    template.render("users/form.html")
}
```

---

## Prochaines étapes

→ [**Formulaires**](https://github.com/seb-alliot/runique/blob/main/docs/fr/05-forms.md)
