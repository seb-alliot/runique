# Open Redirect Protection

## What it does

Intercepts all 3xx redirect responses and validates the `Location` header.
If the redirect destination is an external host not in the allowed list, the middleware returns HTTP 400.

Protects against phishing attacks where an attacker crafts a link like
`https://yoursite.com/login?next=https://evil.com` that silently redirects users elsewhere.

---

## How it works

A redirect destination is considered safe if:

- It is a **relative path** (`/dashboard`, `../profile`) — always safe
- The host is **localhost or loopback** (`localhost`, `127.x.x.x`, `[::1]`, IPv4-mapped IPv6) — always safe
- The host matches an entry in **`with_allowed_hosts`** (exact or wildcard subdomain)

Any other absolute URL is blocked with HTTP 400.

---

## Configuration

No dedicated configuration — the middleware reads `with_allowed_hosts` automatically:

```rust
.middleware(|m| {
    m.with_allowed_hosts(|h| {
        h.enabled(true)
         .host("example.com")
         .host(".example.com")  // example.com + all subdomains
    })
})
```

The open redirect middleware is **always active** and uses the same host list.

---

## Protocol-relative URLs

URLs starting with `//` (e.g. `//evil.com/path`) are treated as absolute and subject to the same check.
They are blocked unless the host is in the allowed list.

---

## See also

| Section | Description |
| --- | --- |
| [Hosts & cache](/docs/en/middleware/hosts-cache) | Allowed hosts configuration |
| [CSP & headers](/docs/en/middleware/csp) | Content Security Policy |
| [Builder](/docs/en/middleware/builder) | Builder configuration |

## Back to summary

- [Middleware & Security](/docs/en/middleware)
