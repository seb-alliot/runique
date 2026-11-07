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

Désérialise la query string complète vers une struct dérivant `Deserialize`.
Les clés inconnues sont ignorées ; une valeur vide (`?page=`) est retirée, un champ `Option` reçoit alors `None`.

Une query string qui ne correspond pas à la struct (`?page=abc` pour un `u32`) est une **400 Bad Request** : avec `?`, le handler s'arrête et `400.html` est affiché. Ce template se surcharge comme `404.html`, en plaçant le vôtre dans `templates/`. En mode debug, la page indique quel paramètre a échoué et pourquoi.

```rust
#[derive(Deserialize)]
pub struct Filtres {
    page: Option<u32>,
    limit: Option<u32>,
    recherche: Option<String>,
}

async fn liste(mut request: Request) -> AppResult<Response> {
    let filtres: Filtres = request.query()?;
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
    // Une URL absolue (lien envoyé par email…) part de l'URL publique
    // (`.with_public_url()`), jamais du
    // header Host : le client le choisit, et un lien vers son site emporterait
    // le jeton qu'il contient.
    let base_url: Option<String> = request.public_url(); // sans `/` final

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

Retourne la connexion du moteur (`&ADb`), prête à passer aux requêtes SeaORM.

```rust
async fn handler(mut request: Request) -> AppResult<Response> {
    let db = request.db();
    let item = MyEntity::find_by_id(1).one(db).await?;
    // ...
}
```

Équivalent à `&request.engine.db`, en plus court.

---

## Formulaires — `req.form("nom")`

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

`request.form("nom")` construit le formulaire avec les données envoyées ; `ValidationForm::try_form` le valide s'il a bien été envoyé. Voir [Concepts clés](/docs/fr/architecture/concepts).

---

## Json — Corps JSON

Un handler qui prend `Json` à la place de `Request` lit le corps lui-même. Le middleware CSRF s'applique quand même : un POST JSON doit porter l'en-tête `X-CSRF-Token` (le jeton que le template reçoit dans `{{ csrf_token }}`), sinon il est refusé en 403 avant d'atteindre le handler. Une route appelée par un service externe (webhook) s'exempte avec `csrf_exempt`, et vérifie elle-même la signature de l'appel.

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
