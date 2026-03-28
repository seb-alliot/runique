# Host Validation & Cache-Control

## Host Validation (Allowed Hosts)

### How it works

- Compares the request `Host` header against the list of allowed hosts
- Blocks requests with a non-allowed host (HTTP 400)
- Protects against Host Header Injection attacks

### Configuration via the builder

Host validation is configured in `main.rs` via the builder — there is no environment variable for this:

```rust
.middleware(|m| {
    m.with_allowed_hosts(|h| {
        h.enabled(!is_debug())
         .host("example.com")
         .host(".api.example.com")  // matches example.com AND *.example.com
    })
})
```

### Supported patterns

- `"localhost"` — exact match
- `".example.com"` — matches `example.com` and `*.example.com`
- `"*"` — all hosts (⚠️ dangerous in production)

### Debug mode

In `DEBUG=true`, typically use `.enabled(!is_debug())` to disable validation during development.

---

## Cache-Control

### Development mode (`DEBUG=true`)

`no-cache` headers are added to force reloads:

```http
Cache-Control: no-cache, no-store, must-revalidate
Pragma: no-cache
```

### Production mode (`DEBUG=false`)

Caching headers are enabled for performance.

---

## See also

| Section | Description |
| --- | --- |
| [CSP & headers](/docs/en/middleware/csp) | Content Security Policy |
| [Builder](/docs/en/middleware/builder) | Builder configuration |

## Back to summary

- [Middleware & Security](/docs/en/middleware)
