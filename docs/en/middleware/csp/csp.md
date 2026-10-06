# Content Security Policy (CSP)

Runique applies a CSP policy **by default, with zero configuration** — the security middleware (headers + CSP) runs on every response, even if `.with_csp(...)` is never called. A unique nonce is generated per request and injected into Tera templates.

---

## Table of contents

| Section | Description |
| --- | --- |
| [CSP Profiles](/docs/en/middleware/csp-profiles) | `default()`, `strict()`, `permissive()` — comparison and use cases |
| [Directives](/docs/en/middleware/csp-directives) | All configurable directives |
| [CSP Nonce](/docs/en/middleware/csp-nonce) | How the nonce works, template usage |
| [Security Headers](/docs/en/middleware/csp-headers) | All automatically injected headers |

---

## Quick start

Without any configuration, the default CSP (`SecurityPolicy::default()`) and all security headers (X-Frame-Options, X-Content-Type-Options, Referrer-Policy, Permissions-Policy, COEP/COOP/CORP, HSTS if actually served over HTTPS) are already sent on every response. `.with_csp(...)` doesn't **enable** them — it replaces the default policy with your own:

```rust
RuniqueApp::builder(config)
    .middleware(|m| {
        m.with_csp(|c| c.policy(SecurityPolicy::strict()))
    })
    .build()
    .await?;
```

To customize:

```rust
.middleware(|m| {
    m.with_csp(|c| {
        c.scripts(vec!["'self'", "https://cdn.example.com"])
         .images(vec!["'self'", "data:"])
    })
})
```

In your templates:

```html
<script {% csp %}>
    // This script is allowed by the CSP nonce
    console.log("OK");
</script>
```

---

## Forced HTTPS (`enforce_https`)

`ENFORCE_HTTPS=true` is for deployments **behind a reverse proxy that terminates TLS** (nginx, Caddy, Cloudflare…): Runique runs over HTTP behind it and can't see by itself whether the client came over HTTP or HTTPS. It reads it from the `X-Forwarded-Proto` header set by the proxy.

| `X-Forwarded-Proto` received | Effect |
| --- | --- |
| `http` | **308** redirect to the same URL in `https://` (same host, path and query) |
| `https` | the request goes through |
| missing | the request goes through — without the header, nothing tells an HTTP request apart from one the proxy received over HTTPS without saying so; redirecting it would loop forever |

With several proxies in a chain (`http, https`), only the first value counts: it's the one the client saw.

The redirect URL's host is read from the `Host` header. The redirect runs right after Host validation (slot 17, after slot 15): with host validation enabled (`.with_allowed_hosts(...)` in the builder), a forged `Host` is refused before it's used.

**With ACME** (`ACME_ENABLED=true`), the redirect is **not mounted**: Runique then serves TLS itself, no request carries `X-Forwarded-Proto`, and its port-80 listener already redirects to HTTPS (keeping the path and query). `ENFORCE_HTTPS` has no effect on redirection in that mode.

Either way, `ENFORCE_HTTPS` or ACME also enables the HSTS header (see [Security headers](/docs/en/middleware/csp-headers)).

> **⚠️ Required proxy configuration:**
> - the proxy must **set** `X-Forwarded-Proto` itself from the actual connection, overwriting any value sent by the client;
> - it must pass the original host in `Host`, otherwise the redirect would point to Runique's internal address (e.g. `https://127.0.0.1:3000/...`);
> - if it doesn't send `X-Forwarded-Proto`, no redirect happens (never a loop).
>
> A client forging `X-Forwarded-Proto: https` over direct HTTP only escapes the redirect for its own connection: no impact on other users.

```env
# .env
ENFORCE_HTTPS=true
```

```rust
// main.rs — host validation is set in the builder, not in .env
.middleware(|m| m.with_allowed_hosts(|h| h.enabled(true).host("mysite.com")))
```

```nginx
# nginx — headers to pass to Runique
proxy_set_header Host $host;
proxy_set_header X-Forwarded-Proto $scheme;
```

If the proxy already redirects HTTP to HTTPS itself, Runique's redirect never fires (every request reaches it with `X-Forwarded-Proto: https`): no double redirect, and `ENFORCE_HTTPS=true` remains useful for HSTS.

---

## See also

| Section | Description |
| --- | --- |
| [CSRF](/docs/en/middleware/csrf) | CSRF protection |
| [Builder & configuration](/docs/en/middleware/builder) | Builder configuration |

## Back to summary

- [Middleware & Security](/docs/en/middleware)
