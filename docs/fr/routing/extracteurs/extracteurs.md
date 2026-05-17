# Extracteurs de paramètres

## Path — Paramètres d'URL

### Valeur typée — `request.get_path_as()`

Parse le segment d'URL directement dans le type cible.
Retourne `None` si la clé est absente ou si la valeur ne peut pas être parsée.

```rust
// Route : "/menus/{id}"
async fn menu_detail(mut request: Request) -> AppResult<Response> {
    let Some(id) = request.get_path_as::<i32>("id") else {
        return Ok((StatusCode::NOT_FOUND, "Introuvable").into_response());
    };
    // id = 42 pour /menus/42
}
```

### Chaîne brute — `request.get_path()`

```rust
async fn article(mut request: Request) -> AppResult<Response> {
    let slug = request.get_path("slug").unwrap_or_default();
}
```

### Plusieurs segments — extracteur Axum `Path`

Pour extraire plusieurs segments à la fois, l'extracteur Axum reste disponible :

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

## Query — Paramètres de requête

### Struct typée — `request.query()`

Désérialise la query string complète vers une struct dérivant `Deserialize + Default`.
Les clés inconnues sont ignorées ; les clés absentes prennent leur valeur `Default`.

```rust
#[derive(Deserialize, Default)]
pub struct Filtres {
    page: Option<u32>,
    limit: Option<u32>,
    recherche: Option<String>,
}

async fn liste(mut request: Request) -> AppResult<Response> {
    let filtres: Filtres = request.query();
    let page = filtres.page.unwrap_or(1);
    // ...
}
```

### Valeur unique — `request.get_query(clé)`

```rust
async fn liste(mut request: Request) -> AppResult<Response> {
    let page: u32 = request.get_query("page")
        .and_then(|v| v.parse().ok())
        .unwrap_or(1);
    // ...
}
```

---

## Headers — `request.headers`

La map HTTP complète des en-têtes est accessible directement sur `Request`. Utile pour lire `Host`, `Accept-Language`, des en-têtes custom, etc.

```rust
async fn handler(mut request: Request) -> AppResult<Response> {
    // Construire une URL absolue depuis le header Host
    let base_url = request.headers
        .get("host")
        .and_then(|v| v.to_str().ok())
        .map(|h| format!("https://{h}"))
        .unwrap_or_else(|| "http://localhost:3000".to_string());

    // Lire n'importe quel header
    let lang = request.headers
        .get("accept-language")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("fr");
}
```

`request.headers` est de type `axum::http::HeaderMap`, re-exporté dans le prelude.

---

## Base de données — `request.db()`

Retourne une `&DatabaseConnection` depuis le moteur, prête à passer aux requêtes SeaORM.

```rust
async fn handler(mut request: Request) -> AppResult<Response> {
    let db = request.db();
    let item = MyEntity::find_by_id(1).one(db).await?;
    // ...
}
```

Équivalent à `&*request.engine.db`, en plus court et lisible.

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
