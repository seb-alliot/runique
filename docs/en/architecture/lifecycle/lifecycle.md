# Request Lifecycle

## Lifecycle

```text
1. The HTTP request arrives
2. The middlewares run in slot order (see Middleware stack):
   Engine/Tera/Config injected, client IP, host checked, security headers…
3. The session is loaded, then the signed-in account read from the database (request.user)
4. CSRF: a token sent as a header (X-CSRF-Token) is checked here; a form
   token (csrf_token field) is checked by Prisme in the next step
5. The handler receives Request: the body is read (Prisme), request.form("name")
   builds the form, ValidationForm::try_form validates it
6. The handler returns AppResult<Response>
7. The middlewares run in reverse order (the session is saved)
8. The HTTP response is sent
```

---

## Best Practices

### 1. Clone Arcs

```rust
let db = request.engine.db.clone();
```

### 2. Forms = per-request copies

```rust
let mut form: MyForm = request.form("my");
// Each request = isolated form, zero concurrency
```

### 3. `context_update!` for context

```rust
context_update!(request => {
    "title" => "My page",
    "data" => &my_data,
});
```

### 4. Flash messages for redirects

```rust
success!(request.notices => "Action successful!");
return Ok(Redirect::to("/").into_response());
```

### 5. `flash_now!` for direct rendering

```rust
context_update!(request => {
    "messages" => flash_now!(error => "Validation error"),
});
```

---

## See also

| Section | Description |
| --- | --- |
| [Key concepts](/docs/en/architecture/concepts) | `RuniqueEngine`, `Request`, `request.form("name")` |
| [Macros](/docs/en/architecture/macros) | Context, flash, routing, error macros |
| [Tera tags & filters](/docs/en/architecture/tera) | Django-like tags, filters, functions |
| [Middleware stack](/docs/en/architecture/middleware) | Slot order, dependency injection |

## Back to summary

- [Architecture](/docs/en/architecture)
