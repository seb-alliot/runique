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

Deserializes the full query string into any struct deriving `Deserialize`.
Unknown keys are ignored; an empty value (`?page=`) is dropped, so an `Option` field gets `None`.

A query string that doesn't fit the struct (`?page=abc` for a `u32`) is a **400 Bad Request**: with `?`, the handler stops and `400.html` is shown. Override that template like `404.html`, by putting your own in `templates/`. In debug mode, the page says which parameter failed and why.

```rust
#[derive(Deserialize)]
pub struct Filters {
    page: Option<u32>,
    limit: Option<u32>,
    search: Option<String>,
}

async fn list(mut request: Request) -> AppResult<Response> {
    let filters: Filters = request.query()?;
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
    // An absolute URL (a link sent by email…) is built on the public URL
    // (`.with_public_url()`), never on
    // the Host header: the client chooses it, and a link to their own site
    // would carry away the token it holds.
    let base_url = request.engine.config.server.public_url.clone();

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

Returns the engine's connection (`&ADb`), ready to pass to SeaORM queries.

```rust
async fn handler(mut request: Request) -> AppResult<Response> {
    let db = request.db();
    let item = MyEntity::find_by_id(1).one(db).await?;
    // ...
}
```

Equivalent to `&request.engine.db`, only shorter.

---

## Forms — `req.form("name")`

```rust
use runique::prelude::*;

async fn signup(mut request: Request) -> AppResult<Response> {
    let form: SignupForm = request.form("signup");
    let form = match ValidationForm::try_form(form, &request).await {
        Ok(validated) => match validated.save(&request.engine.db).await {
            Ok(_) => return Ok(Redirect::to("/").into_response()),
            Err(form) => form,
        },
        Err(form) => form,
    };
    context_update!(request => { "signup_form" => &form });
    request.render("signup.html")
}
```

`request.form("name")` builds the form with the submitted data; `ValidationForm::try_form` validates it when it was actually submitted. See [Key concepts](/docs/en/architecture/concepts).

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
