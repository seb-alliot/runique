# Content Security Policy (CSP)

Runique automatically applies a CSP policy to every response via the `security_headers_middleware`. A unique nonce is generated per request and injected into Tera templates.

---

## Table of contents

| Section | Description |
| --- | --- |
| [CSP Profiles](https://github.com/seb-alliot/runique/blob/main/docs/en/middleware/csp/profiles.md) | `default()`, `strict()`, `permissive()` — comparison and use cases |
| [Directives & env vars](https://github.com/seb-alliot/runique/blob/main/docs/en/middleware/csp/directives.md) | All configurable directives and their variables |
| [CSP Nonce](https://github.com/seb-alliot/runique/blob/main/docs/en/middleware/csp/nonce.md) | How the nonce works, template usage |
| [Security Headers](https://github.com/seb-alliot/runique/blob/main/docs/en/middleware/csp/headers.md) | All automatically injected headers |

---

## Quick start

CSP is enabled by default. No configuration required. In your templates:

```html
<script {% csp_nonce %}>
    // This script is allowed by the CSP nonce
    console.log("OK");
</script>
```

To customize the policy via `.env`:

```env
RUNIQUE_POLICY_CSP_IMAGES='self',data:
RUNIQUE_POLICY_CSP_SCRIPTS='self',https://cdn.example.com
```

---

## See also

| Section | Description |
| --- | --- |
| [CSRF](https://github.com/seb-alliot/runique/blob/main/docs/en/middleware/csrf/csrf.md) | CSRF protection |
| [Builder & configuration](https://github.com/seb-alliot/runique/blob/main/docs/en/middleware/builder/builder.md) | Builder configuration |

## Back to summary

- [Middleware & Security](https://github.com/seb-alliot/runique/blob/main/docs/en/middleware/08-middleware.md)
