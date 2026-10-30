# Returning Responses

## HTML Template (most common)

```rust
async fn index(mut request: Request) -> AppResult<Response> {
    context_update!(request => {
        "title" => "Home",
    });
    request.render("index.html")
}
```

---

## Redirect

```rust
use axum::response::Redirect;

async fn after_submit(request: Request) -> AppResult<Response> {
    success!(request.notices => "Saved!");
    Ok(Redirect::to("/").into_response())
}
```

---

## JSON

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

---

## Status Code

```rust
use axum::http::StatusCode;

async fn not_found() -> StatusCode {
    StatusCode::NOT_FOUND
}
```

---

## Tuple Response

```rust
async fn created(Json(data): Json<Data>) -> (StatusCode, Json<Data>) {
    (StatusCode::CREATED, Json(data))
}
```

---

## Full app structure

```rust
// src/url.rs
use crate::views;
use runique::prelude::*;
use runique::{urlpatterns, view};

pub fn routes() -> Routes {
    urlpatterns! {
        "/" => view!{ views::index }, name = "index",
        "/about" => view! { views::about }, name = "about",
        "/inscription" => view! { views::soumission_inscription }, name = "inscription",
    }
}
```

```rust
// src/views.rs
use runique::prelude::*;

pub async fn index(mut request: Request) -> AppResult<Response> {
    context_update!(request => { "title" => "Home" });
    request.render("index.html")
}

pub async fn about(mut request: Request) -> AppResult<Response> {
    success!(request.notices => "Welcome!");
    context_update!(request => { "title" => "About" });
    request.render("about/about.html")
}

pub async fn soumission_inscription(mut request: Request) -> AppResult<Response> {
    let form: RegisterForm = request.form();

    let validated = match ValidationForm::try_form(form, &request, "my_form").await {
        Ok(validated) => validated,
        Err(form) => {
            // GET (nothing submitted): blank form, no flash.
            // Submitted but invalid: re-render with the error flash.
            if request.method.is_safe() {
                context_update!(request => {
                    "title" => "Sign up",
                    "inscription_form" => &form,
                });
            } else {
                context_update!(request => {
                    "title" => "Error",
                    "inscription_form" => &form,
                    "messages" => flash_now!(error => "Please fix the errors"),
                });
            }
            return request.render("inscription_form.html");
        }
    };

    match validated.save(&request.engine.db).await {
        Ok(_) => {
            success!(request.notices => "Welcome!");
            Ok(Redirect::to("/").into_response())
        }
        Err(form) => {
            context_update!(request => {
                "title" => "Error",
                "inscription_form" => &form,
                "messages" => flash_now!(error => "Please fix the errors"),
            });
            request.render("inscription_form.html")
        }
    }
}
```

---

## See also

| Section | Description |
| --- | --- |
| [Macros](/docs/en/routing/macros) | `urlpatterns!`, `view!`, `impl_objects!` |
| [Extractors](/docs/en/routing/extractors) | Path, Query, req.form(), Json |

## Back to summary

- [Routing](/docs/en/routing)
