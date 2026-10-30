# Usage in Handlers

## Pattern with Redirect (Flash Messages)

```rust
pub async fn submit_signup(mut request: Request) -> AppResult<Response> {
    let form: RegisterForm = request.form();

    let validated = match ValidationForm::try_form(form, &request, "my_form").await {
        Ok(validated) => validated,
        Err(form) => {
            // ❌ Validation failed → immediate message (no redirect)
            context_update!(request => {
                "title" => "Validation error",
                "signup_form" => &form,
                "messages" => flash_now!(error => "Please fix the errors"),
            });
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

```rust
pub async fn about(mut request: Request) -> AppResult<Response> {
    success!(request.notices => "This is a success message.");
    info!(request.notices => "This is an informational message.");
    warning!(request.notices => "This is a warning message.");
    error!(request.notices => "This is an error message.");

    context_update!(request => {
        "title" => "About",
    });
    request.render("about/about.html")
}
```

---

## Flash Behavior (Single Read)

Flash messages stored in the session are **automatically consumed** upon display:

```
1. POST /signup
   → success!("Welcome!")
   → Redirect::to("/")

2. GET /
   → Messages read from session
   → Displayed in template
   → Removed from session

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
