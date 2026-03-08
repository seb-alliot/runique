```markdown id="apf4a9"
# Middleware & Security

Runique includes configurable security middlewares automatically applied in the optimal order through the slot system.

## Table of Contents

| Module | Description |
| --- | --- |
| [CSRF Protection](https://github.com/seb-alliot/runique/blob/main/docs/en/middleware/csrf/csrf.md) | Token, Double Submit Cookie, AJAX |
| [Content Security Policy](https://github.com/seb-alliot/runique/blob/main/docs/en/middleware/csp/csp.md) | Nonce, profiles, headers |
| [Sessions](https://github.com/seb-alliot/runique/blob/main/docs/en/middleware/sessions/sessions.md) | Store, durations, access in handlers |
| [Hosts & Cache](https://github.com/seb-alliot/runique/blob/main/docs/en/middleware/hosts-cache/hosts-cache.md) | Allowed Hosts, Cache-Control, security headers |
| [Builder & Configuration](https://github.com/seb-alliot/runique/blob/main/docs/en/middleware/builder/builder.md) | Classic Builder, Intelligent Builder, environment variables |

## Execution Stack

```

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

← [**ORM & Database**](https://github.com/seb-alliot/runique/blob/main/docs/en/orm/07-orm.md) | [**Flash Messages**](https://github.com/seb-alliot/runique/blob/main/docs/en/flash/09-flash-messages.md) →
```
