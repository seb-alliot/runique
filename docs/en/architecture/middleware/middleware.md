# Middleware Stack

## Slot Order

Runique applies middlewares in an **optimal order** using the slot system:

```text
Incoming request
    ↓
1. Extensions (slot 0)     → Inject Tera, Config, Engine
2. ErrorHandler (slot 10)  → Capture and render errors
3. Custom (slot 20+)       → Custom middlewares
4. CSP (slot 30)           → Content Security Policy & headers
5. Cache (slot 40)         → No-cache in development
6. Session (slot 50)       → Session management
7. CSRF (slot 60)          → CSRF protection
8. Host (slot 70)          → Allowed Hosts validation
    ↓
Handler (your code)
    ↓
Outgoing response (middlewares in reverse order)
```

> **Important**: With Axum, the last `.layer()` applied is executed first. The Intelligent Builder manages this order automatically.

---

## Dependency Injection

Via **Axum Extensions**, automatically injected by the Extensions middleware:

```rust
// Automatically registered by the builder:
// Extension(engine)  → Arc<RuniqueEngine>
// Extension(tera)    → Arc<Tera>
// Extension(config)  → Arc<RuniqueConfig>

// Accessible inside handlers via Request:
pub async fn handler(request: Request) -> AppResult<Response> {
    let db = request.engine.db.clone();
    let config = &request.engine.config;
    // ...
}
```

---

## See also

| Section | Description |
| --- | --- |
| [Key concepts](/docs/en/architecture/concepts) | `RuniqueEngine`, `Request`, `request.form()` |
| [Macros](/docs/en/architecture/macros) | Context, flash, routing, error macros |
| [Tera tags & filters](/docs/en/architecture/tera) | Django-like tags, filters, functions |
| [Request lifecycle](/docs/en/architecture/lifecycle) | Lifecycle, best practices |

## Back to summary

- [Architecture](/docs/en/architecture)
