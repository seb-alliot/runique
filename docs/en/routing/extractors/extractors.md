# Parameter Extractors

## Path — URL parameters

```rust
use axum::extract::Path;

// Simple
async fn user_detail(
    Path(id): Path<i32>,
    mut request: Request,
) -> AppResult<Response> {
    // id = 42 for /users/42
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

## Query — Query parameters

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

## Prisme — Forms

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

---

## Json — JSON body

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

## See also

| Section | Description |
| --- | --- |
| [Macros](/docs/en/routing/macros) | `urlpatterns!`, `view!`, `impl_objects!` |
| [Responses](/docs/en/routing/responses) | Response types |

## Back to summary

- [Routing](/docs/en/routing)
