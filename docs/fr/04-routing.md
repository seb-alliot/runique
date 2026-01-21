# 🛣️ Routage

## Macro urlpatterns!

Définir les routes de l'application:

```rust
use runique::urlpatterns;
use axum::routing::{get, post, put, delete};

pub fn routes() -> Router {
    urlpatterns! {
        "index" => "/" => get(index),
        "user_list" => "/users" => get(user_list),
        "user_detail" => "/users/<id>" => get(user_detail),
        "user_create" => "/users" => post(create_user),
        "user_update" => "/users/<id>" => put(update_user),
        "user_delete" => "/users/<id>" => delete(delete_user),
        "search" => "/search" => post(search),
    }
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
    ctx: RuniqueContext,
) -> Response { }

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
    ExtractForm(form): ExtractForm<LoginForm>,
) -> Response {
    if form.is_valid().await {
        // Traiter
    }
}
```

---

## Retourner des Réponses

### HTML Template

```rust
async fn index(
    ctx: RuniqueContext,
    template: TemplateContext,
) -> Response {
    template.render("index.html", &context! {
        "title" => "Accueil",
        "items" => vec![1, 2, 3]
    })
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

async fn login_submit() -> Response {
    success!(ctx.flash => "Connecté!");
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
use serde_json::json;

pub fn routes() -> Router {
    Router::new()
        .route("/", get(index))
        .route("/users", get(user_list).post(create_user))
        .route("/users/:id", get(user_detail).put(update_user).delete(delete_user))
}

async fn index(template: TemplateContext) -> Response {
    template.render("index.html", &context! {
        "title" => "Accueil"
    })
}

async fn user_list(
    ctx: RuniqueContext,
    template: TemplateContext,
) -> Response {
    let db = ctx.engine.db.clone();
    
    match users::Entity::find().all(&*db).await {
        Ok(users) => {
            template.render("users/list.html", &context! {
                "users" => users
            })
        }
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

async fn user_detail(
    Path(id): Path<i32>,
    ctx: RuniqueContext,
    template: TemplateContext,
) -> Response {
    let db = ctx.engine.db.clone();
    
    match users::Entity::find_by_id(id).one(&*db).await {
        Ok(Some(user)) => {
            template.render("users/detail.html", &context! {
                "user" => user
            })
        }
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

async fn create_user(
    mut ctx: RuniqueContext,
    template: TemplateContext,
    ExtractForm(form): ExtractForm<UserForm>,
) -> Response {
    if form.is_valid().await {
        let db = ctx.engine.db.clone();
        match form.save(&*db).await {
            Ok(_) => {
                success!(ctx.flash => "Utilisateur créé!");
                return Redirect::to("/users").into_response();
            }
            Err(e) => {
                error!(ctx.flash => format!("Erreur: {}", e));
            }
        }
    }
    
    template.render("users/form.html", &context! {
        "form" => form
    })
}
```

---

## Prochaines étapes

→ [**Formulaires**](./05-forms.md)
