# Request Lifecycle

## Lifecycle

```text
1. HTTP request arrives
2. Middlewares traversed (slot order)
3. Extensions injected (Engine, Tera, Config)
4. Session loaded, CSRF verified
5. Handler called with extractors (Request, request.form())
6. Handler returns AppResult<Response>
7. Middlewares traversed in reverse order
8. HTTP response sent
```

---

## Best Practices

### 1. Clone Arcs

```rust
let db = request.engine.db.clone();
```

### 2. Forms = per-request copies

```rust
let mut form: MyForm = request.form();
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
| [Key concepts](/docs/en/architecture/concepts) | `RuniqueEngine`, `Request`, `request.form()` |
| [Macros](/docs/en/architecture/macros) | Context, flash, routing, error macros |
| [Tera tags & filters](/docs/en/architecture/tera) | Django-like tags, filters, functions |
| [Middleware stack](/docs/en/architecture/middleware) | Slot order, dependency injection |

## Back to summary

- [Architecture](/docs/en/architecture)
