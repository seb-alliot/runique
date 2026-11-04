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

pub fn routes() -> Routes {
    urlpatterns! {
        "/" => view!{ views::index }, name = "index",
        "/about" => view! { views::about }, name = "about",
        "/signup" => view! { views::sign_up }, name = "signup",
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
    request.render("about.html")
}

pub async fn sign_up(mut request: Request) -> AppResult<Response> {
    let form: SignupForm = request.form("signup");

    let validated = match ValidationForm::try_form(form, &request).await {
        Ok(validated) => validated,
        Err(form) => {
            // Pas envoyé (premier affichage) : formulaire vierge, pas de flash.
            // Envoyé mais invalide : ré-affichage avec le flash d'erreur.
            if !form.is_submitted() {
                context_update!(request => {
                    "title" => "Inscription",
                    "signup_form" => &form,
                });
            } else {
                context_update!(request => {
                    "title" => "Erreur",
                    "signup_form" => &form,
                    "messages" => flash_now!(error => "Veuillez corriger les erreurs"),
                });
            }
            return request.render("signup.html");
        }
    };

    match validated.save(&request.engine.db).await {
        Ok(_) => {
            success!(request.notices => "Bienvenue !");
            Ok(Redirect::to("/").into_response())
        }
        Err(form) => {
            context_update!(request => {
                "title" => "Erreur",
                "signup_form" => &form,
                "messages" => flash_now!(error => "Veuillez corriger les erreurs"),
            });
            request.render("signup.html")
        }
    }
}
```

---

## Voir aussi

| Section | Description |
| --- | --- |
| [Macros](/docs/fr/routing/macros) | `urlpatterns!`, `view!`, `impl_objects!` |
| [Extracteurs](/docs/fr/routing/extracteurs) | Path, Query, req.form("nom"), Json |

## Retour au sommaire

- [Routage](/docs/fr/routing)
