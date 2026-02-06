
# 🛡️ Middleware & Security

## Overview

Runique integrates configurable security middlewares. The **Smart Builder** automatically applies them in the optimal order using a slot system.

---

## Middleware Stack (Execution Order)

```
Incoming Request
    ↓
1. Extensions (slot 0)      → Inject Engine, Tera, Config
2. ErrorHandler (slot 10)   → Capture and render errors
3. Custom (slot 20+)        → Your custom middlewares
4. CSP (slot 30)             → Content Security Policy & headers
5. Cache (slot 40)           → No-cache in development
6. Session (slot 50)         → Session management (MemoryStore by default)
7. CSRF (slot 60)            → Cross-Site Request Forgery protection
8. Host (slot 70)            → Allowed hosts validation
    ↓
Handler (your code)
    ↓
Outgoing Response (middlewares in reverse order)
```

> 💡 With Axum, the last `.layer()` is executed first on the request. The Smart Builder handles this order automatically via slots.

---

## CSRF Protection

### How it works

* Token **generated automatically** per session
* **Double Submit Cookie** pattern (cookie + hidden input)
* Checked on POST, PUT, PATCH, DELETE requests
* Ignored on GET, HEAD, OPTIONS requests

### In Runique Forms

When using `{% form.xxx %}`, the CSRF token is **included automatically**. No need to add it manually.

### In Manual HTML Forms

```html
<form method="post" action="/submit">
    {% csrf %}
    <input type="text" name="data">
    <button type="submit">Submit</button>
</form>
```

### For AJAX Requests

```javascript
const csrfToken = document.querySelector('[name="csrf_token"]').value;

fetch('/api/endpoint', {
    method: 'POST',
    headers: {
        'Content-Type': 'application/json',
        'X-CSRF-Token': csrfToken
    },
    body: JSON.stringify(data)
});
```

---

## Content Security Policy (CSP)

### How it works

* **Nonce** generated automatically per request
* Injected into the Tera context as `csp_nonce`
* CSP headers added to every response

### Usage in Templates

```html
<!-- Secure inline scripts -->
<script {% csp_nonce %}>
    console.log("Script with CSP nonce");
</script>

<!-- Or using the variable directly -->
<script nonce="{{ csp_nonce }}">
    console.log("Alternative usage");
</script>
```

### CSP Profiles

| Profile                   | Description                     |
| ------------------------- | ------------------------------- |
| `CspConfig::strict()`     | Strict policy (production)      |
| `CspConfig::permissive()` | Permissive policy (development) |
| `CspConfig::default()`    | Default profile                 |

---

## Allowed Hosts Validation

### How it works

* Compares the request `Host` header against `ALLOWED_HOSTS`
* Blocks requests with unauthorized hosts (HTTP 400)
* Protects against Host Header Injection attacks

### `.env` Configuration

```env
# Allowed hosts (comma-separated)
ALLOWED_HOSTS=localhost,127.0.0.1,example.com

# Supported patterns:
# localhost       → exact match
# .example.com    → matches example.com AND *.example.com
# *               → ALL hosts (⚠️ DANGEROUS in production!)
```

### Debug Mode

With `DEBUG=true`, host validation is **disabled by default** for easier development.

---

## Cache-Control

### Development Mode (`DEBUG=true`)

No-cache headers are added to force reload:

```
Cache-Control: no-cache, no-store, must-revalidate
Pragma: no-cache
```

### Production Mode (`DEBUG=false`)

Cache headers are enabled for performance.

---

## Security Headers

Runique automatically injects standard security headers:

| Header                    | Value                             | Protection             |
| ------------------------- | --------------------------------- | ---------------------- |
| `X-Content-Type-Options`  | `nosniff`                         | Prevent MIME sniffing  |
| `X-Frame-Options`         | `DENY`                            | Prevent clickjacking   |
| `X-XSS-Protection`        | `1; mode=block`                   | Browser XSS protection |
| `Referrer-Policy`         | `strict-origin-when-cross-origin` | Limit referrers        |
| `Content-Security-Policy` | Dynamic (with nonce)              | CSP                    |

---

## Sessions

### Default Store

Runique uses `MemoryStore` by default (data in memory, lost on restart).

### Configuration

```rust
// Custom session duration
let app = RuniqueApp::builder(config)
    .with_session_duration(time::Duration::hours(2))
    .build()
    .await?;
```

### Session Durations

| Duration                | Usage                     |
| ----------------------- | ------------------------- |
| `Duration::minutes(30)` | Short sessions (security) |
| `Duration::hours(2)`    | Standard usage            |
| `Duration::hours(24)`   | Runique default           |
| `Duration::days(7)`     | "Remember me"             |

### Custom Store (Production)

```rust
use tower_sessions::MemoryStore;

let app = RuniqueApp::builder(config)
    .with_session_store(MemoryStore::default())
    .build()
    .await?;
```

### Accessing Session in Handlers

```rust
pub async fn dashboard(request: Request) -> AppResult<Response> {
    // Read a session value
    let user_id: Option<i32> = request.session
        .get("user_id")
        .await
        .ok()
        .flatten();

    // Write a value
    let _ = request.session.insert("last_visit", "2026-02-06").await;

    // ...
}
```

---

## Builder Configuration

### Classic Builder

```rust
let app = RuniqueApp::builder(config)
    .routes(url::routes())
    .with_database(db)
    .with_error_handler(true)   // Capture errors
    .with_csp(true)             // CSP & security headers
    .with_allowed_hosts(true)   // Host validation
    .with_cache(true)           // No-cache in dev
    .with_static_files()        // Static files service
    .build()
    .await?;
```

### Smart Builder (New)

```rust
use runique::app::RuniqueAppBuilder as IntelligentBuilder;

let app = IntelligentBuilder::new(config)
    .routes(url::routes())
    .with_database(db)
    .statics()                  // Enable static files
    .build()
    .await?;
```

The Smart Builder:

* **Automatically** applies middlewares in the correct order (slots)
* Uses the **debug profile** for default values (permissive in dev, strict in prod)
* Allows **customization** via `middleware(|m| { ... })`

### Customizing Middlewares

```rust
let app = IntelligentBuilder::new(config)
    .routes(url::routes())
    .with_database(db)
    .middleware(|m| {
        m.disable_csp();              // Disable CSP
        m.disable_host_validation();  // Disable host validation
    })
    .build()
    .await?;
```

---

## Security-Related Environment Variables

| Variable                         | Default      | Description                            |
| -------------------------------- | ------------ | -------------------------------------- |
| `SECRETE_KEY`                    | *(required)* | Secret key for CSRF                    |
| `ALLOWED_HOSTS`                  | `*`          | Allowed hosts                          |
| `DEBUG`                          | `true`       | Debug mode (affects CSP, cache, hosts) |
| `RUNIQUE_ENABLE_CSP`             | *(auto)*     | Force CSP on/off                       |
| `RUNIQUE_ENABLE_HOST_VALIDATION` | *(auto)*     | Force host validation                  |
| `RUNIQUE_ENABLE_CACHE`           | *(auto)*     | Force cache control                    |

> In debug mode, security middlewares are permissive by default. `RUNIQUE_ENABLE_*` variables allow forcing specific behavior regardless of debug mode.

---

## Next Steps

← [**ORM & Database**](https://github.com/seb-alliot/runique/blob/main/docs/en/07-orm.md) | [**Flash Messages**](https://github.com/seb-alliot/runique/blob/main/docs/en/09-flash-messages.md) →
