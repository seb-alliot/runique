# Extracteurs de paramètres

## Path — Paramètres d'URL

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

---

## Query — Paramètres de requête

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

---

## Formulaires — `req.form()`

```rust
use runique::prelude::*;

async fn inscription(mut request: Request) -> AppResult<Response> {
    let mut form: RegisterForm = request.form();
    if form.is_valid().await {
        form.save(&request.engine.db).await?;
    }
    // ...
}
```

---

## Json — Corps JSON

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

## Voir aussi

| Section | Description |
| --- | --- |
| [Macros](/docs/fr/routing/macros) | `urlpatterns!`, `view!`, `impl_objects!` |
| [Réponses](/docs/fr/routing/reponses) | Types de réponses |

## Retour au sommaire

- [Routage](/docs/fr/routing)
