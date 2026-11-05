# Host Validation & Cache-Control

## Host Validation (Allowed Hosts)

### How it works

- Compares the request `Host` header against the list of allowed hosts
- Blocks requests with a non-allowed host (HTTP 400)
- Protects against Host Header Injection attacks

### HTTP/2 and the `Host` header

HTTP/2 does not send a `Host` header — it uses the `:authority` pseudo-header instead.
Runique handles this automatically: if the `Host` header is absent, the middleware
reads the authority from `request.uri().authority()`.

This covers direct HTTP/1.1, HTTP/2 (e.g. ACME or Cloudflare), and reverse proxies
that proxy over HTTP/2.

### Logging

Rejections can be logged via `RuniqueLog`:

```rust
.with_log(|l| l.middleware(|m| m.host_validation(LogLevel::WARN)))
```

In production, `WARN` is recommended to detect attacks or misconfigurations.

### Configuration via the builder

Host validation is configured in `main.rs` via the builder — the list doesn't go through an environment variable:

```rust
.middleware(|m| {
    m.with_allowed_hosts(|h| {
        h.host("example.com")
         .host(".example.com")  // matches example.com AND all its subdomains
    })
})
```

### Supported patterns

- `"localhost"` — exact match
- `".example.com"` — matches `example.com` and `*.example.com`
- `"*"` — all hosts (⚠️ dangerous in production)

### By environment

Validation is on with `RUNIQUE_ENV=prod` as soon as a host is declared, off with `RUNIQUE_ENV=dev`. `.enabled(true)` or `.enabled(false)` has the last word, for instance to try the validation locally.

---

## Cache-Control

### Development mode (`RUNIQUE_ENV=dev` **and** accessed via `localhost`/`127.0.0.1`/`[::1]`)

Both conditions are required — accessing with `RUNIQUE_ENV=dev` through a different host name (LAN, tunnel, staging) won't trigger these headers. Added to force reloads:

```http
Cache-Control: no-cache, no-store, must-revalidate
Pragma: no-cache
```

### Production mode (`RUNIQUE_ENV=prod`)

No no-cache header is added: the browser applies its usual cache. Static and media files caching is set with `.static_files(|s| s.static_cache("…").media_cache("…"))`.

---

## See also

| Section | Description |
| --- | --- |
| [CSP & headers](/docs/en/middleware/csp) | Content Security Policy |
| [Builder](/docs/en/middleware/builder) | Builder configuration |

## Back to summary

- [Middleware & Security](/docs/en/middleware)
