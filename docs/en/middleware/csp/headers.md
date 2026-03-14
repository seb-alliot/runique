# Security Headers

The `security_headers_middleware` automatically injects a set of security headers into every response, in addition to the CSP header. It is activated via `.with_header_security(true)` in the builder.

---

## Injected headers

| Header | Value | Protection |
| --- | --- | --- |
| `Content-Security-Policy` | Dynamic (unique nonce per request) | Restricts allowed sources for scripts, styles, images, etc. |
| `X-Content-Type-Options` | `nosniff` | Prevents the browser from guessing the MIME type — blocks MIME sniffing attacks |
| `X-Frame-Options` | `DENY` | Prevents embedding the page in an iframe — protects against clickjacking |
| `X-XSS-Protection` | `1; mode=block` | Enables the XSS filter in legacy browsers (older IE/Edge) |
| `Referrer-Policy` | `strict-origin-when-cross-origin` | Sends full referrer on same-origin, origin only on cross-origin, nothing on HTTP→HTTPS |
| `Permissions-Policy` | `geolocation=(), microphone=(), camera=()` | Disables access to geolocation, microphone and camera |
| `Cross-Origin-Embedder-Policy` | `require-corp` | Requires cross-origin resources to be explicitly opted in (CORP) |
| `Cross-Origin-Opener-Policy` | `same-origin` | Isolates the browsing context — prevents cross-origin attacks via `window.opener` |
| `Cross-Origin-Resource-Policy` | `same-origin` | Prevents resources from being loaded by other origins |
| `Strict-Transport-Security` | `max-age=31536000; includeSubDomains` | Enforces HTTPS for 1 year, subdomains included (HSTS) |

---

## Activation

### CSP only (without additional security headers)

```rust
.middleware(|m| {
    m.with_csp(|c| c)
})
```

### CSP + all security headers

```rust
.middleware(|m| {
    m.with_csp(|c| {
        c.with_header_security(true)
         .with_nonce(true)
    })
})
```

### Full strict preset

```rust
.middleware(|m| {
    m.with_csp(|c| {
        c.policy(SecurityPolicy::strict())
         .with_header_security(true)
    })
})
```

---

## Notes

**HSTS (`Strict-Transport-Security`)** — This header is always sent, even if the application runs on HTTP behind a reverse proxy. Browsers only honor it over HTTPS connections. In production, ensure your proxy (nginx, Caddy, Cloudflare…) terminates TLS.

**COEP (`Cross-Origin-Embedder-Policy: require-corp`)** — Required to use `SharedArrayBuffer` and certain high-performance APIs. It may block loading of cross-origin resources (images, scripts, fonts) that do not return the `Cross-Origin-Resource-Policy` header. If you load resources from third-party CDNs, verify their compatibility or disable COEP via a custom `SecurityPolicy`.

**`X-XSS-Protection`** — Legacy header, ignored by modern browsers (Chrome, Firefox). Kept for compatibility with older browsers.

---

## Back

- [CSP — Overview](https://github.com/seb-alliot/runique/blob/main/docs/en/middleware/csp/csp.md)
