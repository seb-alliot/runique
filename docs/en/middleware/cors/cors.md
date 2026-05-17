# CORS

## What it does

CORS (Cross-Origin Resource Sharing) controls which origins can make browser requests to your API.
Without CORS headers, browsers block cross-origin requests by default.

Runique does **not** enable CORS by default. Add it only when a frontend on a different domain needs to call your API.

---

## Basic configuration

```rust
.middleware(|m| {
    m.with_cors(|c| {
        c.origin("https://app.example.com")
         .origin("https://www.example.com")
    })
})
```

---

## Allow any origin

```rust
m.with_cors(|c| c.any_origin())
```

> This sets `Access-Control-Allow-Origin: *`.
> Safe only for **fully public, read-only** APIs.

---

## With credentials

Cookies and `Authorization` headers require explicit opt-in:

```rust
m.with_cors(|c| {
    c.origin("https://app.example.com")
     .allow_credentials(true)
})
```

> **Security constraint**: `any_origin()` and `.allow_credentials(true)` cannot be combined.
> Runique rejects this configuration at build time with a `BuildError`.

---

## Cache preflight duration

```rust
m.with_cors(|c| {
    c.origin("https://app.example.com")
     .max_age(3600)  // seconds, default: 3600
})
```

---

## Stripe / third-party webhooks

Stripe and similar services POST directly from their servers — **not from a browser**.
CORS does not apply to server-to-server calls.
For Stripe webhooks, use `.csrf_exempt()` instead (and verify the `Stripe-Signature` header in your handler).

---

## See also

| Section | Description |
| --- | --- |
| [CSRF](/docs/en/middleware/csrf) | CSRF protection and exempt paths |
| [Hosts & cache](/docs/en/middleware/hosts-cache) | Host header validation |
| [Builder](/docs/en/middleware/builder) | Builder configuration |

## Back to summary

- [Middleware & Security](/docs/en/middleware)
