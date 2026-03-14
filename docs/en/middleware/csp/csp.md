# Content Security Policy (CSP)

Runique applies a CSP policy via the security middleware, configured exclusively through the builder. A unique nonce is generated per request and injected into Tera templates.

---

## Table of contents

| Section | Description |
| --- | --- |
| [CSP Profiles](https://github.com/seb-alliot/runique/blob/main/docs/en/middleware/csp/profiles.md) | `default()`, `strict()`, `permissive()` — comparison and use cases |
| [Directives](https://github.com/seb-alliot/runique/blob/main/docs/en/middleware/csp/directives.md) | All configurable directives |
| [CSP Nonce](https://github.com/seb-alliot/runique/blob/main/docs/en/middleware/csp/nonce.md) | How the nonce works, template usage |
| [Security Headers](https://github.com/seb-alliot/runique/blob/main/docs/en/middleware/csp/headers.md) | All automatically injected headers |

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

## See also

| Section | Description |
| --- | --- |
| [CSRF](https://github.com/seb-alliot/runique/blob/main/docs/en/middleware/csrf/csrf.md) | CSRF protection |
| [Builder & configuration](https://github.com/seb-alliot/runique/blob/main/docs/en/middleware/builder/builder.md) | Builder configuration |

## Back to summary

- [Middleware & Security](https://github.com/seb-alliot/runique/blob/main/docs/en/middleware/08-middleware.md)
