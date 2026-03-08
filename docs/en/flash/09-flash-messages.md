# 💬 Flash Messages

## Messaging System

Runique provides a flash messaging system for user notifications. There are **two types** of messages:

1. **Redirect messages** (`success!`, `error!`, `info!`, `warning!`) — stored in the session, displayed after a redirect
2. **Immediate messages** (`flash_now!`) — displayed in the current request, without using the session

---

## Redirect Macros

These macros store messages in the session via `request.notices`. They are displayed **after the next redirect** (Post/Redirect/Get pattern).

### `success!` — Success message

```rust
success!(request.notices => "User created successfully!");
success!(request.notices => format!("Welcome {}!", username));

// Multiple messages at once
success!(request.notices => "Created", "Email sent", "Welcome!");
```

### `error!` — Error message

```rust
error!(request.notices => "An error occurred");
error!(request.notices => format!("Error: {}", e));
```

### `info!` — Informational message

```rust
info!(request.notices => "Please check your email");
```

### `warning!` — Warning message

```rust
warning!(request.notices => "This action cannot be undone");
```

> 💡 Each macro calls `.success()`, `.error()`, `.info()`, or `.warning()` on `request.notices` (of type `Message`).

---

## `flash_now!` Macro — Immediate Messages

`flash_now!` creates a `Vec<FlashMessage>` for **immediate display** in the current request. Ideal when there is no redirect (for example, re-rendering a form after validation errors).

```rust
// Single message
let msgs = flash_now!(error => "Please fix the errors");

// Multiple messages
let msgs = flash_now!(warning => "Field A is incorrect", "Field B is missing");
```

### Available Types

| Type      | Generated CSS Class |
| --------- | ------------------- |
| `success` | `message-success`   |
| `error`   | `message-error`     |
| `info`    | `message-info`      |
| `warning` | `message-warning`   |

### Injecting into the context

`flash_now!` returns a vector that must be manually injected into the context:

```rust
context_update!(request => {
    "title" => "Validation error",
    "form" => &form,
    "messages" => flash_now!(error => "Please fix the errors"),
});
```

---

## Usage in Handlers

### Pattern with redirect (flash messages)

```rust
pub async fn submit_signup(
    mut request: Request,
    Prisme(mut form): Prisme<RegisterForm>,
) -> AppResult<Response> {
    if request.is_post() {
        if form.is_valid().await {
            let user = form.save(&request.engine.db).await.map_err(|err| {
                form.get_form_mut().database_error(&err);
                AppError::from(err)
            })?;

            // ✅ Flash message → displayed after redirect
            success!(request.notices => format!(
                "Welcome {}, your account has been created!",
                user.username
            ));
            return Ok(Redirect::to("/").into_response());
        }

        // ❌ Validation failed → immediate message (no redirect)
        context_update!(request => {
            "title" => "Validation error",
            "signup_form" => &form,
            "messages" => flash_now!(error => "Please fix the errors"),
        });
        return request.render("signup_form.html");
    }

    // GET → display form
    context_update!(request => {
        "title" => "Sign up",
        "signup_form" => &form,
    });
    request.render("signup_form.html")
}
```

### Multiple message types

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

## Displaying Messages in Templates

### Automatic `{% messages %}` tag

The `{% messages %}` tag automatically renders all messages:

```html
{% messages %}
```

It includes the internal template `message/message_include.html`, which generates:

```html
{% if messages %}
    <div class="flash-messages">
        {% for message in messages %}
        <div class="message message-{{ message.level }}">
            {{ message.content }}
        </div>
        {% endfor %}
    </div>
{% endif %}
```

### Recommended placement

Place `{% messages %}` in your base template, just before the main content:

```html
<!-- base.html -->
<body>
    <header>...</header>

    {% messages %}

    <main>
        {% block content %}{% endblock %}
    </main>

    <footer>...</footer>
</body>
```

### Custom display

To fully customize rendering, manually loop over `messages`:

```html
{% if messages %}
    {% for msg in messages %}
        <div class="alert alert-{{ msg.level }}" role="alert">
            <strong>
                {% if msg.level == "success" %}✅
                {% elif msg.level == "error" %}❌
                {% elif msg.level == "warning" %}⚠️
                {% elif msg.level == "info" %}ℹ️
                {% endif %}
            </strong>
            {{ msg.content }}
        </div>
    {% endfor %}
{% endif %}
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

## Difference: Flash vs Flash Now

|                       | `success!` / `error!` / etc. | `flash_now!`                             |
| --------------------- | ---------------------------- | ---------------------------------------- |
| **Storage**           | Session                      | Memory (Vec)                             |
| **Display**           | After redirect               | Current request                          |
| **Lifetime**          | Until next read              | Single request                           |
| **Typical use**       | Post/Redirect/Get            | Re-render form                           |
| **Context injection** | Automatic                    | Manual (`"messages" => flash_now!(...)`) |

---

## When to Use Which?

### ✅ Use flash macros (session)

```rust
// After successful action with redirect
success!(request.notices => "Saved!");
return Ok(Redirect::to("/").into_response());
```

### ✅ Use `flash_now!` (immediate)

```rust
// Validation error → re-render page without redirect
context_update!(request => {
    "form" => &form,
    "messages" => flash_now!(error => "Invalid form"),
});
return request.render("form.html");
```

---

## Next Steps

← [**Middleware & Security**](https://github.com/seb-alliot/runique/blob/main/docs/en/08-middleware.md) | [**Practical Examples**](https://github.com/seb-alliot/runique/blob/main/docs/en/10-examples.md) →
