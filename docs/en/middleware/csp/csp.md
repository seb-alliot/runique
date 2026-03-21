# Content Security Policy (CSP)

Runique applies a CSP policy via the security middleware, configured exclusively through the builder. A unique nonce is generated per request and injected into Tera templates.

---

## Table of contents

| Section | Description |
| --- | --- |
| [CSP Profiles](/docs/en/middleware/csp) | `default()`, `strict()`, `permissive()` — comparison and use cases |
| [Directives](/docs/en/middleware/csp) | All configurable directives |
| [CSP Nonce](/docs/en/middleware/csp) | How the nonce works, template usage |
| [Security Headers](/docs/en/middleware/csp) | All automatically injected headers |

---

## Quick start

CSP is disabled by default — it is activated only through the builder:

```rust
RuniqueApp::new()
    .middleware(|m| {
        m.with_csp(|c| c)
    })
    .build()
    .await?;
```

To customize:

```rust
.middleware(|m| {
    m.with_csp(|c| {
        c.with_nonce(true)
         .scripts(vec!["'self'", "https://cdn.example.com"])
         .images(vec!["'self'", "data:"])
    })
})
```

In your templates:

```html
<script {% csp_nonce %}>
    // This script is allowed by the CSP nonce
    console.log("OK");
</script>
```

---

## Forced HTTPS (`enforce_https`)

The `ENFORCE_HTTPS=true` directive enables a 301 redirect to HTTPS for all HTTP requests. This redirect relies on the `X-Forwarded-Proto` header to detect whether the request arrived over HTTP or HTTPS.

> **⚠️ Proxy requirement:** `enforce_https` trusts the `X-Forwarded-Proto` header. Without a trusted reverse proxy (nginx, Caddy, etc.) controlling this header, an attacker can forge `X-Forwarded-Proto: https` to bypass the redirect.
>
> **In production**, always place Runique behind a reverse proxy that controls this header:
> strip any client-supplied `X-Forwarded-Proto` headers and inject the correct value (`https` or `http`) based on the actual connection.

```env
# .env
ENFORCE_HTTPS=true
```

```nginx
# nginx — correct configuration example
proxy_set_header X-Forwarded-Proto $scheme;
```

---

## See also

| Section | Description |
| --- | --- |
| [CSRF](/docs/en/middleware/csrf) | CSRF protection |
| [Builder & configuration](/docs/en/middleware/builder) | Builder configuration |

## Back to summary

- [Middleware & Security](/docs/en/middleware)
