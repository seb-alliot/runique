
---

# 🛣️ Routing

## urlpatterns! Macro

Define application routes with names for URL resolution:

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
        "/users/:id/delete" => view!{ views::delete_user }, name = "user_delete",
    }
}
```

### With Names (recommended)

Names allow URL resolution in templates using `{% link "name" %}`:

```rust
urlpatterns! {
    "/" => view!{ views::index }, name = "index",
    "/users/:id" => view!{ views::user_detail }, name = "user_detail",
}
```

```html
<a href='{% link "index" %}'>Home</a>
<a href='{% link "user_detail" id="42" %}'>Profile</a>
```

### Without Names

```rust
urlpatterns! {
    "/" => view!{ views::index },
    "/about" => view!{ views::about },
}
```

---

## view! Macro

### Single handler for all methods

A single handler handles GET, POST, PUT, and DELETE (recommended pattern with `request.is_get()` / `request.is_post()`):

```rust
// In routes
"/register" => view!{ views::register }, name = "register",
```

```rust
// In handler
pub async fn register(
    mut request: Request,
    Prisme(mut form): Prisme<RegisterForm>,
) -> AppResult<Response> {
    if request.is_get() {
        // Display empty form
        context_update!(request => { "form" => &form });
        return request.render("form.html");
    }

    if request.is_post() {
        // Process submission
        if form.is_valid().await {
            // ...
        }
    }

    request.render("form.html")
}
```

---

## Parameter Extractors

### Path — URL Parameters

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

### Query — Query Parameters

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

### Prisme — Forms

```rust
use runique::prelude::*;

async fn register(
    mut request: Request,
    Prisme(mut form): Prisme<RegisterForm>,
) -> AppResult<Response> {
    if form.is_valid().await {
        form.save(&request.engine.db).await?;
    }
    // ...
}
```

### JSON — POST Body

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

## Returning Responses

### HTML Template (most common)

```rust
async fn index(mut request: Request) -> AppResult<Response> {
    context_update!(request => {
        "title" => "Home",
    });
    request.render("index.html")
}
```

### Redirect

```rust
use axum::response::Redirect;

async fn after_submit(request: Request) -> AppResult<Response> {
    success!(request.notices => "Saved!");
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

## Complete App Structure

```rust
// src/url.rs
use crate::views;
use runique::prelude::*;
use runique::{urlpatterns, view};

pub fn routes() -> Router {
    let router = urlpatterns! {
        "/" => view!{ views::index }, name = "index",
        "/about" => view! { views::about }, name = "about",
        "/register" => view! { views::register_submit }, name = "register",
    };
    router
}
```

```rust
// src/views.rs
use runique::prelude::*;

pub async fn index(mut request: Request) -> AppResult<Response> {
    context_update!(request => {
        "title" => "Home",
    });
    request.render("index.html")
}

pub async fn about(mut request: Request) -> AppResult<Response> {
    success!(request.notices => "Welcome!");

    context_update!(request => {
        "title" => "About",
    });
    request.render("about/about.html")
}

pub async fn register_submit(
    mut request: Request,
    Prisme(mut form): Prisme<RegisterForm>,
) -> AppResult<Response> {
    if request.is_get() {
        context_update!(request => {
            "title" => "Register",
            "register_form" => &form,
        });
        return request.render("register_form.html");
    }

    if request.is_post() {
        if form.is_valid().await {
            let user = form.save(&request.engine.db).await.map_err(|err| {
                form.get_form_mut().database_error(&err);
                AppError::from(err)
            })?;
            success!(request.notices => format!("Welcome {}!", user.username));
            return Ok(Redirect::to("/").into_response());
        }

        context_update!(request => {
            "title" => "Error",
            "register_form" => &form,
            "messages" => flash_now!(error => "Please correct the errors"),
        });
        return request.render("register_form.html");
    }

    request.render("register_form.html")
}
```

---

## impl_objects! Macro (bonus)

For routes performing ORM queries, `impl_objects!` adds a Django-like manager:

```rust
use runique::impl_objects;

impl_objects!(users::Entity);

// Then in handlers:
let user = users::Entity::objects
    .filter(users::Column::Username.eq("john"))
    .first(&db)
    .await?;
```

See the [ORM guide](https://github.com/seb-alliot/runique/blob/main/docs/en/07-orm.md) for more details.

---

## Next Steps

← [**Routing**](https://github.com/seb-alliot/runique/blob/main/docs/en/04-routing.md) | [**Forms**](https://github.com/seb-alliot/runique/blob/main/docs/en/05-forms.md) →

