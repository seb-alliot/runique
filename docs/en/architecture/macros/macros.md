# Rust Macros

Runique provides a set of macros to simplify development.

---

## Context Macros

| Macro | Description | Example |
| ----- | ----------- | ------- |
| `context!` | Create a Tera context | `context!("title" => "Page")` |
| `context_update!` | Add to a Request's context | `context_update!(request => { "key" => value })` |

---

## Flash Message Macros

| Macro | Description | Example |
| ----- | ----------- | ------- |
| `success!` | Success message (session) | `success!(request.notices => "OK!")` |
| `error!` | Error message (session) | `error!(request.notices => "Error")` |
| `info!` | Info message (session) | `info!(request.notices => "Info")` |
| `warning!` | Warning (session) | `warning!(request.notices => "Warning")` |
| `flash_now!` | Immediate message (no session) | `flash_now!(error => "Errors")` |

---

## Routing Macros

| Macro | Description | Example |
| ----- | ----------- | ------- |
| `urlpatterns!` | Define named routes | `urlpatterns!("/" => view!{...}, name = "index")` |
| `view!` | Handler for all HTTP methods | `view!{ handler }` |
| `impl_objects!` | Django-like manager for SeaORM | `impl_objects!(Entity)` |

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
    let mut form: ContactForm = request.form();

    if request.is_post() && form.is_valid().await {
        // Session flash + redirect (Post/Redirect/Get pattern)
        success!(request.notices => "Message sent!");
        return Ok(Redirect::to("/contact").into_response());
    }

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
