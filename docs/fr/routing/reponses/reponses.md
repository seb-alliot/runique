# Retourner des réponses

## HTML Template (le plus courant)

```rust
async fn index(mut request: Request) -> AppResult<Response> {
    context_update!(request => {
        "title" => "Accueil",
    });
    request.render("index.html")
}
```

---

## Redirect

```rust
use axum::response::Redirect;

async fn after_submit(request: Request) -> AppResult<Response> {
    success!(request.notices => "Sauvegardé !");
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

## Structure complète d'une app

```rust
// src/url.rs
use crate::views;
use runique::prelude::*;
use runique::{urlpatterns, view};

pub fn routes() -> Router {
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
    context_update!(request => { "title" => "Accueil" });
    request.render("index.html")
}

pub async fn about(mut request: Request) -> AppResult<Response> {
    success!(request.notices => "Bienvenue !");
    context_update!(request => { "title" => "À propos" });
    request.render("about/about.html")
}

pub async fn soumission_inscription(mut request: Request) -> AppResult<Response> {
    let mut form: RegisterForm = request.form();
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

## Voir aussi

| Section | Description |
| --- | --- |
| [Macros](/docs/fr/routing/macros) | `urlpatterns!`, `view!`, `impl_objects!` |
| [Extracteurs](/docs/fr/routing/extracteurs) | Path, Query, req.form(), Json |

## Retour au sommaire

- [Routage](/docs/fr/routing)
