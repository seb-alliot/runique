# Usage in Handlers

## Pattern with Redirect (Flash Messages)

```rust
pub async fn submit_signup(mut request: Request) -> AppResult<Response> {
    let form: SignupForm = request.form("signup");

    let validated = match ValidationForm::try_form(form, &request).await {
        Ok(validated) => validated,
        Err(form) => {
            context_update!(request => { "title" => "Sign up", "signup_form" => &form });
            // ❌ Sent but invalid → immediate message (no redirect).
            // On the first display (GET), nothing was sent: no message.
            if form.is_submitted() {
                context_update!(request => {
                    "messages" => flash_now!(error => "Please fix the errors"),
                });
            }
            return request.render("signup_form.html");
        }
    };

    // ✅ Flash message → displayed after redirect. `save` returns the form.
    match validated.save(&request.engine.db).await {
        Ok(_) => {
            success!(request.notices => "Welcome, your account has been created!");
            Ok(Redirect::to("/").into_response())
        }
        Err(form) => {
            context_update!(request => {
                "title" => "Validation error",
                "signup_form" => &form,
                "messages" => flash_now!(error => "Please fix the errors"),
            });
            request.render("signup_form.html")
        }
    }
}
```

---

## Multiple Message Types

For the page this handler renders, `flash_now!` (no session):

```rust
pub async fn about(mut request: Request) -> AppResult<Response> {
    context_update!(request => {
        "title" => "About",
        "messages" => flash_now!(info => "This is an informational message.", "And a second one."),
    });
    request.render("about.html")
}
```

For the next page, the session macros, then a redirect:

```rust
pub async fn publish(request: Request) -> AppResult<Response> {
    success!(request.notices => "This is a success message.");
    warning!(request.notices => "This is a warning message.");
    Ok(Redirect::to("/articles").into_response())
}
```

> `success!` followed by a `render` in the same request shows nothing now: session messages are read **at the start** of the request, before the handler. They would appear on the next request, on another page.

---

## Flash Behavior (Single Read)

Flash messages stored in the session are **read then removed** at the start of the next request (when `Request` is extracted), whether they are shown or not:

```
1. POST /signup
   → success!("Welcome!")
   → Redirect::to("/")

2. GET /
   → Messages read from the session and removed
   → Put in the context (`messages`), shown by {% messages %}

3. GET / (reload)
   → No messages (already consumed)
```

---

## See also

| Section | Description |
| --- | --- |
| [Macros](/docs/en/flash/macros) | All flash macros + differences |
| [Templates](/docs/en/flash/templates) | Displaying messages in templates |

## Back to summary

- [Flash Messages](/docs/en/flash)
