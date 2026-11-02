# Rust Macros

Runique provides a set of macros to simplify development.

---

## Context Macros

| Macro | Description | Example |
| ----- | ----------- | ------- |
| `context!` | Create a Tera context | `context!("title" => "Page", "count" => 3)` |
| `context_update!` | Add to a Request's context | `context_update!(request => { "key" => value })` |

---

## Flash Message Macros

| Macro | Description | Example |
| ----- | ----------- | ------- |
| `success!` | Success message (session) | `success!(request.notices => "OK!")` |
| `error!` | Error message (session) | `error!(request.notices => "Error")` |
| `info!` | Info message (session) | `info!(request.notices => "Info")` |
| `warning!` | Warning (session) | `warning!(request.notices => "Warning")` |
| `flash_now!` | Immediate message(s), no session | `flash_now!(error => "Error")`, or several: `flash_now!(warning => "One", "Two")` |

---

## Routing Macros

| Macro | Description | Example |
| ----- | ----------- | ------- |
| `urlpatterns!` | Define named routes | `urlpatterns!("/" => view!{...}, name = "index")` |
| `view!` | Handler for every HTTP method (GET, POST, PUT, DELETE, PATCH) | `view!{ handler }` |

For a route that must accept one method only, pass Axum's router instead of `view!`: `"/logout" => post(views::logout), name = "logout"`.

---

## ORM macros

| Macro | Description | Example |
| ----- | ----------- | ------- |
| `impl_objects!` | Django-like manager (`Entity::objects.filter(...)`) | `impl_objects!(Entity)` |

`model!{}` already generates that manager: `impl_objects!` is only for a SeaORM entity written by hand. See [ORM](/docs/en/orm).

---

## Error Macros

| Macro | Description |
| ----- | ----------- |
| `impl_from_error!` | Generates `From<Error>` for `AppError` |

---

## In context

The macros combine in a typical handler:

```rust
use runique::prelude::*;

pub async fn contact(mut request: Request) -> AppResult<Response> {
    let form: ContactForm = request.form();

    let form = match ValidationForm::try_form(form, &request, "my_form").await {
        Ok(_validated) => {
            // Session flash + redirect (Post/Redirect/Get pattern)
            success!(request.notices => "Message sent!");
            return Ok(Redirect::to("/contact").into_response());
        }
        Err(form) => form,
    };

    // Add variables to the request context
    context_update!(request => {
        "title" => "Contact",
        "contact_form" => &form,
    });
    request.render("contact.html")
}
```

> `success!` / `error!` / `info!` / `warning!` write to the session (visible after a redirect). `flash_now!` produces an immediate message without a session — useful when re-rendering the same page without redirecting.

---

## See also

| Section | Description |
| --- | --- |
| [Key concepts](/docs/en/architecture/concepts) | `RuniqueEngine`, `Request`, `request.form()` |
| [Tera tags & filters](/docs/en/architecture/tera) | Django-like tags, filters, functions |
| [Middleware stack](/docs/en/architecture/middleware) | Slot order, dependency injection |
| [Request lifecycle](/docs/en/architecture/lifecycle) | Lifecycle, best practices |

## Back to summary

- [Architecture](/docs/en/architecture)
