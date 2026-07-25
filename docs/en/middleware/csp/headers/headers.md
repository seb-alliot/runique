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
| `Permissions-Policy` | Secure preset (see below) | Denies ~20 sensitive features (camera, microphone, geolocation, USB, Bluetooth, payment, sensors…); allows WebAuthn, fullscreen and picture-in-picture same-origin |
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

**Reverse proxy (Nginx, Caddy, Cloudflare…)** — Runique sends all these headers on every dynamic response. A reverse proxy configured with `proxy_hide_header` or duplicate `add_header` directives can silently overwrite them. In production, do not declare these headers in Nginx — let them pass through from the application as-is.

For static files served directly by Nginx (assets, media), the headers do not go through Runique: they must be declared explicitly in the relevant `location` block:

```nginx
location /media/ {
    add_header X-Content-Type-Options "nosniff" always;
    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;
    add_header X-Frame-Options "DENY" always;
}
```

**HSTS (`Strict-Transport-Security`)** — This header is emitted **only when Runique actually serves HTTPS**: either `enforce_https` or ACME enabled (`should_emit_hsts()`). Sending it over plain HTTP is useless (browsers ignore it) and risky (a one-year HTTPS lock-in on a domain that may not be ready). If your proxy (nginx, Caddy, Cloudflare…) terminates TLS without Runique knowing, declare the header on the proxy side.

The value is **configurable** (single source of truth, same settings everywhere: middleware, error pages):

| Env variable | Default | Purpose |
| --- | --- | --- |
| `HSTS_MAX_AGE` | `31536000` (1 year) | `max-age` duration in seconds |
| `HSTS_INCLUDE_SUBDOMAINS` | `true` | Adds `includeSubDomains` — ⚠️ breaks any non-HTTPS subdomain |
| `HSTS_PRELOAD` | `false` | Adds `preload` — **opt-in**: near-irreversible commitment (submission to the browser preload list). Requires `includeSubDomains` + `max-age ≥ 1 year`, otherwise a boot warning is logged and it is ignored for preload |

Static files no longer carry the header themselves: HSTS is *host-scoped*, so once a dynamic page emits it the browser applies it to the whole host (assets included).

**`Permissions-Policy` — default preset** (source: `PermissionsPolicy::default`). All of these features are **denied** (`=()`): `accelerometer`, `ambient-light-sensor`, `bluetooth`, `camera`, `gyroscope`, `hid`, `magnetometer`, `microphone`, `midi`, `serial`, `usb`, `geolocation`, `idle-detection`, `display-capture`, `payment`, `interest-cohort`, `local-fonts`, `sync-xhr`, `xr-spatial-tracking`, `window-management`. **Allowed same-origin** (`=(self)`): `publickey-credentials-create`, `publickey-credentials-get` (WebAuthn / passkeys), `fullscreen`, `picture-in-picture`. Customizable via `.with_permissions_policy(|p| …)`.

**COEP (`Cross-Origin-Embedder-Policy: require-corp`)** — Required to use `SharedArrayBuffer` and certain high-performance APIs. It may block loading of cross-origin resources (images, scripts, fonts) that do not return the `Cross-Origin-Resource-Policy` header. If you load resources from third-party CDNs, verify their compatibility or disable COEP via a custom `SecurityPolicy`.

**`X-XSS-Protection`** — Legacy header, ignored by modern browsers (Chrome, Firefox). Kept for compatibility with older browsers.

---

## Back

- [CSP — Overview](/docs/en/middleware/csp)
