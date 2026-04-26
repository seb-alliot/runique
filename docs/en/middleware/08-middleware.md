# Middleware & Security

Runique includes configurable security middlewares automatically applied in the optimal order through the slot system.

## Table of Contents

| Module | Description |
| --- | --- |
| [CSRF Protection](/docs/en/middleware/csrf) | Token, Double Submit Cookie, AJAX |
| [Content Security Policy](/docs/en/middleware/csp) | Nonce, profiles, headers |
| [Sessions](/docs/en/middleware/sessions) | Store, durations, access in handlers |
| [Hosts & Cache](/docs/en/middleware/hosts-cache) | Allowed Hosts, Cache-Control, security headers |
| [Builder & Configuration](/docs/en/middleware/builder) | Classic Builder, Intelligent Builder, environment variables |
| [Rate Limiting](/docs/en/middleware/rate-limit) | Per-IP, per-route rate limiting, configurable |
| [Login Required](/docs/en/middleware/login-required) | Route protection — redirects if not authenticated |

## Execution Stack

```text
Incoming request
↓
1. Extensions (slot 0)     → Inject Engine, Tera, Config
2. ErrorHandler (slot 10)  → Capture and render errors
3. Custom (slot 20+)       → Your custom middlewares
4. CSP (slot 30)           → Content Security Policy & headers
5. Cache (slot 40)         → No-cache in development
6. Session (slot 50)       → Session management
7. CSRF (slot 60)          → Cross-Site Request Forgery protection
8. Host (slot 70)          → Allowed host validation
   ↓
   Handler (your code)
```

## Next Steps

← [**ORM & Database**](/docs/en/orm) | [**Flash Messages**](/docs/en/flash) →
