# Flash Macros

## Redirect Macros

These macros store messages in the session via `request.notices`. They are displayed **after the next redirect** (Post/Redirect/Get pattern).

### success!

```rust
success!(request.notices => "User created successfully!");
success!(request.notices => format!("Welcome {}!", username));

// Multiple messages at once
success!(request.notices => "Created", "Email sent", "Welcome!");
```

### error!

```rust
error!(request.notices => "An error occurred");
error!(request.notices => format!("Error: {}", e));
```

### info!

```rust
info!(request.notices => "Please check your email");
```

### warning!

```rust
warning!(request.notices => "This action cannot be undone");
```

> Each macro calls `.success()`, `.error()`, `.info()`, or `.warning()` on `request.notices` (of type `Message`).

---

## flash_now! Macro — Immediate Messages

`flash_now!` creates a `Vec<FlashMessage>` for **immediate display** in the current request. Ideal when there is no redirect (for example, re-rendering a form after validation errors).

```rust
// Single message
let msgs = flash_now!(error => "Please fix the errors");

// Multiple messages
let msgs = flash_now!(warning => "Field A is incorrect", "Field B is missing");
```

### Available Types

| Type | `MessageLevel::as_css_class()` |
|------|-------------------|
| `success` | `message-success` |
| `error` | `message-error` |
| `info` | `message-info` |
| `warning` | `message-warning` |

`as_css_class()` returns the same class as the built-in `{% messages %}` tag, whose template emits `class="message message-{{ message.level }}"`: `message.level` serializes in lowercase (`success`, `error`, `info`, `warning`). See [Flash templates](/docs/en/flash/templates) to loop manually.

### Injecting into the context

`flash_now!` returns a vector that must be manually injected into the context:

```rust
context_update!(request => {
    "title" => "Validation error",
    "form" => &form,
    "messages" => flash_now!(error => "Please fix the errors"),
});
```

> **Watch out:** the `messages` key already holds the session messages read at the start of the request (those set before a redirect). Replacing it with `flash_now!` drops them from this page: they are shown nowhere. On a page that can receive both, inject `flash_now!` only where no redirect has just set any.

---

> **Current limit:** the redirect macros ignore a failed session write. When the session can't be saved (database down with the database store), the message is lost: nothing tells the visitor, and the failure only shows in the log, on the `session.store` channel (see [Tracing](/docs/en/configuration/tracing)). The action itself isn't affected.

---

## Difference: Flash vs Flash Now

| | `success!` / `error!` / etc. | `flash_now!` |
|---|---|---|
| **Storage** | Session | Memory (Vec) |
| **Display** | After redirect | Current request |
| **Lifetime** | Until next read | Single request |
| **Typical use** | Post/Redirect/Get | Re-render form |
| **Context injection** | Automatic | Manual (`"messages" => flash_now!(...)`) |

---

## When to Use Which?

### Use flash macros (session)

```rust
// After a successful action with redirect
success!(request.notices => "Saved!");
return Ok(Redirect::to("/").into_response());
```

### Use flash_now! (immediate)

```rust
// Validation error → re-render page without redirect
context_update!(request => {
    "form" => &form,
    "messages" => flash_now!(error => "Invalid form"),
});
return request.render("form.html");
```

---

## See also

| Section | Description |
| --- | --- |
| [Handlers](/docs/en/flash/handlers) | Usage in handlers |
| [Templates](/docs/en/flash/templates) | Displaying messages in templates |

## Back to summary

- [Flash Messages](/docs/en/flash)
