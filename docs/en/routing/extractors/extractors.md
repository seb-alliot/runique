# Parameter Extractors

## Path — URL parameters

### Typed value — `request.get_path_as()`

Parses the path segment directly into the target type.
Returns `None` if the key is absent or the value cannot be parsed.

```rust
// Route: "/users/{id}"
async fn user_detail(mut request: Request) -> AppResult<Response> {
    let Some(id) = request.get_path_as::<i32>("id") else {
        return Ok((StatusCode::NOT_FOUND, "Not found").into_response());
    };
    // id = 42 for /users/42
}
```

### Raw string — `request.get_path()`

```rust
async fn user_detail(mut request: Request) -> AppResult<Response> {
    let slug = request.get_path("slug").unwrap_or_default();
}
```

### Multiple params — Axum `Path` extractor

For multiple path segments at once, the Axum extractor is still available:

```rust
use axum::extract::Path;

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

### Typed struct — `request.query()`

Deserializes the full query string into any struct deriving `Deserialize + Default`.
Unknown keys are ignored; missing keys fall back to `Default`.

```rust
#[derive(Deserialize, Default)]
pub struct Filters {
    page: Option<u32>,
    limit: Option<u32>,
    search: Option<String>,
}

async fn list(mut request: Request) -> AppResult<Response> {
    let filters: Filters = request.query();
    let page = filters.page.unwrap_or(1);
    // ...
}
```

### Single value — `request.get_query(key)`

```rust
async fn list(mut request: Request) -> AppResult<Response> {
    let page: u32 = request.get_query("page")
        .and_then(|v| v.parse().ok())
        .unwrap_or(1);
    // ...
}
```

---

## Headers — `request.headers`

The full HTTP header map is available directly on `Request`. Useful for reading `Host`, `Accept-Language`, custom headers, etc.

```rust
async fn handler(mut request: Request) -> AppResult<Response> {
    // Build an absolute URL from the Host header
    let base_url = request.headers
        .get("host")
        .and_then(|v| v.to_str().ok())
        .map(|h| format!("https://{h}"))
        .unwrap_or_else(|| "http://localhost:3000".to_string());

    // Read any header
    let lang = request.headers
        .get("accept-language")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("fr");
}
```

`request.headers` is of type `axum::http::HeaderMap`, which is re-exported in the prelude.

---

## Database — `request.db()`

Returns a `&DatabaseConnection` from the engine, ready to pass to SeaORM queries.

```rust
async fn handler(mut request: Request) -> AppResult<Response> {
    let db = request.db();
    let item = MyEntity::find_by_id(1).one(db).await?;
    // ...
}
```

This is equivalent to `&*request.engine.db` but shorter and more readable.

---

## Forms — `req.form()`

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
