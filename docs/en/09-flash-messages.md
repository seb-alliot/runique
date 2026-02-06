Voici la traduction complète en anglais de ton chapitre **Flash Messages** :

# 💬 Flash Messages

## Message System

Runique provides a flash message system for user notifications. There are **two types** of messages:

1. **Redirect messages** (`success!`, `error!`, `info!`, `warning!`) — stored in the session, displayed after a redirect
2. **Immediate messages** (`flash_now!`) — displayed on the current request without going through the session

---

## Redirect Macros

These macros store messages in the session via `request.notices`. They are displayed **after the next redirect** (Post/Redirect/Get pattern).

### `success!` — Success Message

```rust
success!(request.notices => "User created successfully!");
success!(request.notices => format!("Welcome {}!", username));

// Multiple messages at once
success!(request.notices => "Created", "Email sent", "Welcome!");
```

### `error!` — Error Message

```rust
error!(request.notices => "An error occurred");
error!(request.notices => format!("Error: {}", e));
```

### `info!` — Informational Message

```rust
info!(request.notices => "Please verify your email");
```

### `warning!` — Warning

```rust
warning!(request.notices => "This action cannot be undone");
```

> 💡 Each macro calls `.success()`, `.error()`, `.info()`, or `.warning()` on `request.notices` (of type `Message`).

---

## `flash_now!` Macro — Immediate Messages

`flash_now!` creates a `Vec<FlashMessage>` for **immediate display** in the current request. Ideal for cases where there is no redirect (e.g., re-displaying a form after validation errors).

```rust
// Single message
let msgs = flash_now!(error => "Please correct the errors");

// Multiple messages
let msgs = flash_now!(warning => "Field A is invalid", "Field B is missing");
```

### Available Types

| Type      | Generated CSS Class |
| --------- | ------------------- |
| `success` | `message-success`   |
| `error`   | `message-error`     |
| `info`    | `message-info`      |
| `warning` | `message-warning`   |

### Injecting into the Context

`flash_now!` returns a vector to manually inject into the context:

```rust
context_update!(request => {
    "title" => "Validation Error",
    "form" => &form,
    "messages" => flash_now!(error => "Please correct the errors"),
});
```

---

## Usage in Handlers

### Pattern with Redirect (Flash Messages)

```rust
pub async fn submit_registration(
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
            "title" => "Validation Error",
            "registration_form" => &form,
            "messages" => flash_now!(error => "Please correct the errors"),
        });
        return request.render("registration_form.html");
    }

    // GET → display the form
    context_update!(request => {
        "title" => "Registration",
        "registration_form" => &form,
    });
    request.render("registration_form.html")
}
```

### Multiple Message Types

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

## Display in Templates

### Automatic Tag `{% messages %}`

The `{% messages %}` tag automatically displays all messages:

```html
{% messages %}
```

It includes the internal template `message/message_include.html` which generates:

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

### Recommended Placement

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

### Customizing Display

To customize the output, loop manually over `messages`:

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

Flash messages stored in the session are **automatically consumed** when displayed:

```
1. POST /registration
   → success!("Welcome!")
   → Redirect::to("/")

2. GET /
   → Messages are read from the session
   → Displayed in the template
   → Removed from the session

3. GET / (reload)
   → No more messages (already consumed)
```

---

## Flash vs `flash_now`

|                       | `success!` / `error!` / etc. | `flash_now!`                             |
| --------------------- | ---------------------------- | ---------------------------------------- |
| **Storage**           | Session                      | Memory (Vec)                             |
| **Display**           | After redirect               | Current request                          |
| **Lifetime**          | Until next read              | Single request                           |
| **Typical Use**       | Post/Redirect/Get            | Form re-display                          |
| **Context Injection** | Automatic                    | Manual (`"messages" => flash_now!(...)`) |

---

## When to Use Which?

### ✅ Use Flash Macros (Session)

```rust
// After a successful action with redirect
success!(request.notices => "Saved!");
return Ok(Redirect::to("/").into_response());
```

### ✅ Use `flash_now!` (Immediate)

```rust
// Validation error → re-display page without redirect
context_update!(request => {
    "form" => &form,
    "messages" => flash_now!(error => "Invalid form"),
});
return request.render("form.html");
```

---

## Next Steps

← [**Middleware & Security**](https://github.com/seb-alliot/runique/blob/main/docs/en/08-middleware.md) | [**Practical Examples**](https://github.com/seb-alliot/runique/blob/main/docs/en/10-examples.md) →
