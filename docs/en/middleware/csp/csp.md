# CSP & Security Headers

## Content Security Policy (CSP)

### How it works

- **Nonce** generated automatically per request
- Injected into the Tera context as `csp_nonce`
- CSP headers added to every response

### Usage in templates

```html
<!-- Secured inline scripts -->
<script {% csp_nonce %}>
    console.log("Script with CSP nonce");
</script>

<!-- Or using the variable directly -->
<script nonce="{{ csp_nonce }}">
    console.log("Alternative");
</script>
```

### CSP Profiles

| Profile | Description |
|--------|-------------|
| `CspConfig::strict()` | Strict policy (production) |
| `CspConfig::permissive()` | Permissive policy (development) |
| `CspConfig::default()` | Default profile |

---

## Security Headers

Runique automatically injects standard security headers:

| Header | Value | Protection |
|--------|--------|------------|
| `X-Content-Type-Options` | `nosniff` | Prevents MIME sniffing |
| `X-Frame-Options` | `DENY` | Prevents clickjacking |
| `X-XSS-Protection` | `1; mode=block` | Browser XSS protection |
| `Referrer-Policy` | `strict-origin-when-cross-origin` | Limits referrers |
| `Content-Security-Policy` | Dynamic (with nonce) | CSP |

---

## See also

| Section | Description |
| --- | --- |
| [CSRF](https://github.com/seb-alliot/runique/blob/main/docs/en/middleware/csrf/csrf.md) | CSRF protection |
| [Hosts & cache](https://github.com/seb-alliot/runique/blob/main/docs/en/middleware/hosts-cache/hosts-cache.md) | Host validation |

## Back to summary

- [Middleware & Security](https://github.com/seb-alliot/runique/blob/main/docs/en/middleware/08-middleware.md)
