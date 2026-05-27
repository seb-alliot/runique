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
| [CORS](/docs/en/middleware/cors) | Cross-Origin Resource Sharing — origins, credentials, preflight |
| [Trusted Proxies](/docs/en/middleware/trusted-proxies) | Real client IP, RFC 1918, CIDR, `ClientIp` |
| [Permissions-Policy](/docs/en/middleware/permissions-policy) | Browser API restrictions via HTTP header |
| [Open Redirect](/docs/en/middleware/open-redirect) | Automatic blocking of redirects to external origins |
| [Anti-Bot Honeypot](/docs/en/middleware/anti-bot) | Invisible trap field — automatic bot rejection |

## Execution Stack

```text
Incoming request
    ↓
slot  0  Extensions          → Inject Engine, Tera, Config (always active)
slot  2  TrustedProxies      → Real client IP from X-Forwarded-For (always active)
slot  5  Compression         → Response compression (always active)
slot  8  CORS                → Cross-Origin Resource Sharing (if with_cors() configured)
slot 10  ErrorHandler        → Capture and render errors (always active)
slot 20+ Custom              → Your custom middlewares
slot 25  OpenRedirect        → Block external redirects (always active)
slot 30  SecurityHeaders     → X-Frame-Options, HSTS, Permissions-Policy… (always active)
slot 31  CSP                 → Content Security Policy (always active)
slot 40  Cache               → No-cache in development (always active)
slot 50  Session             → Session management (always active)
slot 55  SessionUpgrade      → Upgrade anonymous session → authenticated (always active)
slot 57  Auth                → Load CurrentUser from session (always active)
slot 60  CSRF                → Cross-Site Request Forgery protection (always active)
slot 65  AntiBotHoneypot     → Invisible trap field, force_invalid on fill (if with_anti_bot() configured)
slot 70  HostValidation      → Allowed host validation (if with_allowed_hosts() configured)
    ↓
Handler (your code)
```

> **"Always active" slots** apply to every request with no extra configuration. Others only insert into the stack when their builder method is called.

## Next Steps

← [**ORM & Database**](/docs/en/orm) | [**Flash Messages**](/docs/en/flash) →
