# Routing Macros

## Macro urlpatterns!

Define application routes with names for URL resolution:

```rust
use crate::views;
use runique::prelude::*;
use runique::{urlpatterns, view};

pub fn routes() -> Router {
    urlpatterns! {
        "/" => view!{ views::index }, name = "index",
        "/users" => view! { views::user_list }, name = "users",
        "/users/{id}" => view!{ views::user_detail }, name = "user_detail",
        "/users/{id}/delete" => view!{ views::delete_user }, name = "user_delete",
    }
}
```

> Dynamic path segments use the `{param}` syntax (Axum 0.8). The old `:param` syntax is no longer supported.

### With names (recommended)

Names allow URL resolution in templates via `{% link "name" %}`:

```rust
urlpatterns! {
    "/" => view!{ views::index }, name = "index",
    "/users/{id}" => view!{ views::user_detail }, name = "user_detail",
}
```

```html
<a href='{% link "index" %}'>Home</a>
<a href='{% link "user_detail" id="42" %}'>Profile</a>
```

### Without names

```rust
urlpatterns! {
    "/" => view!{ views::index },
    "/about" => view!{ views::about },
}
```

---

## Macro view!

A single handler handles GET and POST as well as PUT and DELETE (recommended pattern with `request.is_get()` / `request.is_post()`):

```rust
// In routes
"/inscription" => view!{ views::inscription }, name = "inscription",
```

```rust
// In handler
pub async fn inscription(mut request: Request) -> AppResult<Response> {
    let mut form: RegisterForm = request.form();
    if request.is_get() {
        context_update!(request => { "form" => &form });
        return request.render("form.html");
    }

    if request.is_post() {
        if form.is_valid().await {
            // ...
        }
    }

    request.render("form.html")
}
```

---

## Routing a specific method (GET / POST / PUT…)

`view!{ handler }` wires the **same** handler onto all five methods (GET, POST, PUT, DELETE, PATCH). This is the recommended pattern when a single handler serves the form on both GET and POST via `request.is_get()` / `request.is_post()`.

When you want **a distinct handler per method**, don't use `view!`: pass the Axum combinators directly (`get`, `post`, `put`, `delete`, `patch`), already re-exported by the prelude. `urlpatterns!` accepts any expression that produces a `MethodRouter` as a handler:

```rust
use crate::views;
use runique::prelude::*; // get, post, put, delete, patch included
use runique::urlpatterns;

pub fn routes() -> Router {
    urlpatterns! {
        // Single handler, all methods (dispatch via request.is_get/is_post)
        "/inscription"   => view!{ views::inscription },                     name = "inscription",

        // Distinct handlers per method
        "/login"         => get(views::login_get).post(views::login_post),   name = "login",

        // Only one method allowed — others return 405
        "/users/{id}"    => delete(views::delete_user),                      name = "user_delete",

        // Chaining several methods
        "/articles/{id}" => get(views::article_get)
                                .put(views::article_update)
                                .delete(views::article_delete),              name = "article",
    }
}
```

> With the Axum combinators, only the declared methods are accepted: any other method automatically returns `405 Method Not Allowed`. By contrast, `view!` accepts all five methods — it's up to the handler to filter.

Per-method handlers share the same signature as any other Runique handler:

```rust
pub async fn login_get(mut request: Request) -> AppResult<Response> {
    request.render("login.html")
}

pub async fn login_post(mut request: Request) -> AppResult<Response> {
    let mut form: LoginForm = request.form();
    if form.is_valid().await {
        // ...
    }
    request.render("login.html")
}
```

### Choosing the route type for the job

Each combinator explicitly fixes the accepted methods. A few typical cases:

```rust
urlpatterns! {
    // Display-only page (GET) — a POST returns 405
    "/about"            => get(views::about),                            name = "about",

    // Endpoint that only receives data (POST) — webhook, action
    "/webhooks/stripe"  => post(views::stripe_webhook),                  name = "stripe",

    // Form: GET displays, POST processes — two separate handlers
    "/contact"          => get(views::contact_form)
                               .post(views::contact_submit),             name = "contact",

    // REST API on a resource
    "/api/articles"     => get(views::articles_index)
                               .post(views::articles_create),            name = "articles",
    "/api/articles/{id}" => get(views::article_show)
                               .put(views::article_update)
                               .patch(views::article_patch)
                               .delete(views::article_destroy),          name = "article",
}
```

On the handler side, each method gets its own function, with no `is_get()` / `is_post()` check:

```rust
// GET /contact — render the empty form
pub async fn contact_form(mut request: Request) -> AppResult<Response> {
    let form: ContactForm = request.form();
    context_update!(request => { "contact_form" => &form });
    request.render("contact.html")
}

// POST /contact — validate and process the submission
pub async fn contact_submit(mut request: Request) -> AppResult<Response> {
    let mut form: ContactForm = request.form();
    if form.is_valid().await {
        // send the email, flash success, redirect…
        return Ok(Redirect::to("/contact").into_response());
    }
    context_update!(request => { "contact_form" => &form });
    request.render("contact.html")
}
```

> Simple rule: use `view!` when a single handler serves several methods (internal dispatch), and the `get`/`post`/… combinators when each method deserves its own handler or when you want to restrict the allowed methods.

---

## Macro impl_objects! (bonus)

`impl_objects!` is **automatically generated** by the daemon in each entity file. It adds a Django-like manager usable directly in handlers:

```rust
use crate::entities::users;

let user = users::Entity::objects
    .filter(users::Column::Username.eq("john"))
    .first(&db)
    .await?;
```

See the [ORM guide](/docs/en/orm) for more details.

---

## See also

| Section | Description |
| --- | --- |
| [Extractors](/docs/en/routing/extractors) | Path, Query, req.form(), Json |
| [Responses](/docs/en/routing/responses) | Response types |

## Back to summary

- [Routing](/docs/en/routing)
