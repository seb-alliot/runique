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
        "/users/:id" => view!{ views::user_detail }, name = "user_detail",
        "/users/:id/delete" => view!{views::delete_user }, name = "user_delete",
    }
}
```

### With names (recommended)

Names allow URL resolution in templates via `{% link "name" %}`:

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
