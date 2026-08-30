# Middleware Stack

## Slot Order

Runique applies middlewares in an **optimal order** using the slot system:

```text
Incoming request
    ↓
1.  Extensions (slot 0)       → Inject Tera, Config, Engine
2.  TrustedProxies (slot 2)   → Real client IP
3.  Compression (slot 5)      → External compression
4.  CORS (slot 8)             → Before ErrorHandler (OPTIONS preflight)
5.  ErrorHandler (slot 10)    → Capture and render errors
6.  HostValidation (slot 15)  → Allowed Hosts validation
7.  Custom (slot 20+)         → Custom middlewares
8.  OpenRedirect (slot 25)    → Inspects 3xx responses
9.  Security Headers (slot 30) → HSTS, X-Frame-Options, etc.
10. CSP (slot 31)             → Content Security Policy
11. Cache (slot 40)           → No-cache in development
12. Session (slot 50)         → Session management
13. SessionUpgrade (slot 55)  → Reads/writes in session
14. Auth (slot 57)            → Loads CurrentUser from the session
15. CSRF (slot 60)            → CSRF protection
16. AntiBot (slot 65)         → Honeypot
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
