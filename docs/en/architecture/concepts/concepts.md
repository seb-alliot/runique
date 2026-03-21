# Key Concepts

## 1. RuniqueEngine

The application's main **shared state**:

```rust
pub struct RuniqueEngine {
    pub db: Arc<DatabaseConnection>,
    pub tera: Arc<Tera>,
    pub config: Arc<RuniqueConfig>,
}
```

Injected as an Axum Extension, accessible in every handler via `request.engine`.

---

## 2. Request — The Main Extractor

`Request` is Runique's central extractor. It replaces the former `TemplateContext` and contains everything required:

```rust
pub struct Request {
    pub engine: AEngine,       // Arc<RuniqueEngine>
    pub session: Session,      // tower-sessions session
    pub notices: Message,      // Flash messages
    pub csrf_token: CsrfToken, // CSRF token
    pub context: Context,      // Tera context
    pub method: Method,        // HTTP method
}
```

**Usage inside a handler:**

```rust
pub async fn index(mut request: Request) -> AppResult<Response> {
    context_update!(request => {
        "title" => "Home",
    });
    request.render("index.html")
}
```

**Methods:**

- `request.render("template.html")` — Render with the current context
- `request.is_get()` / `request.is_post()` — Check the HTTP method

---

## 3. `Prisme<T>` — Form Extractor

```rust
pub async fn handler(
    mut request: Request,
    Prisme(mut form): Prisme<RegisterForm>,
) -> AppResult<Response> {
    if request.is_post() && form.is_valid().await {
        let user = form.save(&request.engine.db).await?;
        success!(request.notices => "User created!");
        return Ok(Redirect::to("/").into_response());
    }

    context_update!(request => {
        "form" => &form,
    });
    request.render("form.html")
}
```

Automatically:

1. Parses the request body
2. Creates a form instance
3. Injects the CSRF token
4. Fills in submitted data

---

## See also

| Section | Description |
| --- | --- |
| [Macros](/docs/en/architecture/macros) | Context, flash, routing, error macros |
| [Tera tags & filters](/docs/en/architecture/tera) | Django-like tags, filters, functions |
| [Middleware stack](/docs/en/architecture/middleware) | Slot order, dependency injection |
| [Request lifecycle](/docs/en/architecture/lifecycle) | Lifecycle, best practices |

## Back to summary

- [Architecture](/docs/en/architecture)
