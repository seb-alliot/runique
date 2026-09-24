# Usage in Handlers

## Pattern with Redirect (Flash Messages)

```rust
pub async fn submit_signup(mut request: Request) -> AppResult<Response> {
    let form: RegisterForm = request.form();

    let mut validated = match ValidationForm::try_new(form, &request).await {
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

    let user = validated.save(&request.engine.db).await.map_err(|err| {
        validated.database_error(&err);
        AppError::from(err)
    })?;

    // ✅ Flash message → displayed after redirect
    success!(request.notices => format!(
        "Welcome {}, your account has been created!",
        user.username
    ));
    Ok(Redirect::to("/").into_response())
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
