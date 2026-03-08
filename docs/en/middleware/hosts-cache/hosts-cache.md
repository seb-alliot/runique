# Host Validation & Cache-Control

## Host Validation (Allowed Hosts)

### How it works

- Compares the request `Host` header against `ALLOWED_HOSTS`
- Blocks requests with a non-allowed host (HTTP 400)
- Protects against Host Header Injection attacks

### `.env` Configuration

```env
# Allowed hosts (comma-separated)
ALLOWED_HOSTS=localhost,127.0.0.1,example.com

# Supported patterns:
# localhost       → exact match
# .example.com   → matches example.com AND *.example.com
# *              → ALL hosts (⚠️ DANGEROUS in production!)
```

### Debug mode

With `DEBUG=true`, host validation is **disabled by default** to make development easier.

---

## Cache-Control

### Development mode (`DEBUG=true`)

`no-cache` headers are added to force reloads:

```
Cache-Control: no-cache, no-store, must-revalidate
Pragma: no-cache
```

### Production mode (`DEBUG=false`)

Caching headers are enabled for performance.

---

## Security-related environment variables

| Variable | Default | Description |
|----------|--------|-------------|
| `SECRETE_KEY` | *(required)* | Secret key for CSRF |
| `ALLOWED_HOSTS` | `*` | Allowed hosts |
| `DEBUG` | `true` | Debug mode (affects CSP, cache, hosts) |
| `RUNIQUE_ENABLE_CSP` | *(auto)* | Force-enable/disable CSP |
| `RUNIQUE_ENABLE_HOST_VALIDATION` | *(auto)* | Force host validation |
| `RUNIQUE_ENABLE_CACHE` | *(auto)* | Force cache control |

---

## See also

| Section | Description |
| --- | --- |
| [CSP & headers](https://github.com/seb-alliot/runique/blob/main/docs/en/middleware/csp/csp.md) | Content Security Policy |
| [Builder](https://github.com/seb-alliot/runique/blob/main/docs/en/middleware/builder/builder.md) | Builder configuration |

## Back to summary

- [Middleware & Security](https://github.com/seb-alliot/runique/blob/main/docs/en/middleware/08-middleware.md)
